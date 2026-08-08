# Hum Work Order 16: Borrowed, Order-Bound Core Operation Expectations

Date: 2026-08-08
<!-- hum-active-workorder:v1 -->
Status: Drafted as a fresh, unissued architectural foundation. The first
independent pre-issuance review returned `ACCEPT WITH REQUIRED FIX` with three
P1 findings. This document records the sole BDFL-authorized bounded correction;
that correction cycle is consumed. The corrected bytes have not been
independently reviewed, accepted, committed, or published. Unit 1 is explicitly
unauthorized. Authoring or correcting this document grants no implementation,
staging, commit, push, archive mutation, or later-work authority.

Owner: BDFL (Ocean).
Author: Work Order 16 architect-author. The author may explain this draft but
is permanently disqualified from independently reviewing these bytes or any
implementation produced under them.
Planning baseline: `HEAD`, local `main`, cached `origin/main`, and live remote
`main` are all `39d373dea56a15a5f97b76e701305bcd4d3e1f02`.

## Closed predecessor and reason for this foundation

Work Order 15 Unit 1 is terminally rejected, archived, closed, and
non-authoritative. No Work Order 15 implementation was accepted, committed, or
published on `main`. Its rejected nine-path candidate is retained only as
failure evidence on
`archive/workorder-15-unit1-terminal-rejection-2026-08-08` at commit
`f47ec6f3b7586cd59077f90ffb66307af7d045ee`, parent
`39d373dea56a15a5f97b76e701305bcd4d3e1f02`, complete tree
`cb70a481879e9434d8b8be598f671c125ead9839`, and scoped tree
`d8356acb28e8a5441999ff550b5c0551c46f1723`. The archive contains exactly nine
paths at `+3,021/-124`; its publication triggered no CI.

The terminal review found that complete Core operation records could be
reordered and their public indices coherently rewritten without rejection.
The candidate carried an owned private identity, so the identity moved with
the candidate instead of remaining attached to the Program-owned source slot.
It therefore did not implement the required borrow-only expected operation.
That defect is prior to, and independent of, minimal-add type analysis.

This Work Order does not repair or retry Work Order 15. It specifies the
smallest missing source-order foundation. Checked minimal-add type authority,
resolver/declaration joins, type projection, verified type access, and
full-type consumption remain deferred to a later, separately reviewed Work
Order after this primitive is accepted, published, terminal-green, and proven
to reject the exact reorder attack.

The Work Order 11 and Work Order 13 archives remain closed failure evidence at
`a40fc65876a9224adecc492b18617ec60684136c` and
`f19c85748426867f0d4b3d5556ec5ed494a81e4c`. No archived source, test, patch,
assertion, or result from any rejected Work Order may be copied, restored,
adapted, merged, cherry-picked, or applied.

## Purpose and exact Unit 1 result

Unit 1 establishes one general, borrow-only relation between an exact
Program-owned operation slot and the independently lowered candidate occupying
that slot:

```text
Program-owned file/item/section traversal
  -> one stack-local borrowed expected Core operation for the exact slot
  -> one independently lowered candidate in that same slot
  -> Core verification compares source owner, source operation, slot, and origin
  -> coherent whole-record reorder fails closed
```

The accepted result is structural authority only. Every current Core item and
operation receives one private, owned, non-authoritative candidate origin while
it is lowered. During verification, expected items are borrowed in Program
order and expected operations are rebuilt one at a time
from the live `Program` and fresh stack-local body/predicate artifacts. The
verifier indexes the candidate at that exact expected slot and compares it with
the borrowed expectation. It never searches the candidate collection for a
matching identity. Swapping complete candidate records therefore moves only
candidate material; the Program-owned expected slots do not move. Rewriting
all public `operation.index` values after the swap still fails.

Unit 1 makes no type conclusion. It neither identifies a minimal-add target nor
produces, projects, verifies, or consumes any expression type.

Authenticated same-slot operation ownership is a hard prerequisite for checked
type authority, not an independent destination or a deferrable sibling. Current
verification enumerates candidate-controlled Core vectors.
`operation_index_consistent` proves only that a candidate's public index agrees
with its current candidate-vector position. A complete operation swap carries
its expression material and any future type claim with it; rewriting the public
indices restores candidate self-consistency without proving ownership of either
source slot. Therefore no checked type may be trusted until Core verification
binds the typed candidate to the independently borrowed, Program-authorized
source slot.

The mandatory one-way dependency is:

```text
live Program-owned source slot
  -> independently borrowed expected operation
  -> exact same-slot lowered candidate
  -> candidate-origin comparison
  -> Core verifier verdict
  -> later checked-type verification
```

Work Order 16 implements through the Core verifier verdict and stops there. It
does not reopen or implement checked type authority.

## Current-main producer and consumer audit

The source audit establishes this current path:

1. `src/ast.rs` owns the parser-issued `CanonicalCoreFileBinding` and
   `CanonicalCoreOwnerBinding`. The file binding retains the source revision,
   semantic file index, and normalized path. The owner binding retains the
   exact item path, item kind, section slots, and optional authenticated task
   signature. `Program::canonical_core_expectation` locates an exact item and
   section by live reference and source traversal.
2. `CanonicalCoreSectionExpectation::validate` rechecks the live Program/file,
   item path, section slot, parser file witness, owner witness, and section seal
   before producing `ValidatedCoreSection`. The validated type currently
   exposes only `section()`, so downstream code cannot borrow the complete
   authenticated owner without either weakening the boundary or rebuilding an
   owned identity. That is the load-bearing reason for the `src/ast.rs` edit.
3. `src/core_body.rs` consumes a fresh section expectation and creates
   `CanonicalBodyGrammarReport`. Its `statements` contain one
   `CanonicalBodyStatement` for every retained `does.body_syntax` entry in
   source order, including recognized, unsupported, and blocked shapes. This
   path already transports the parser-owned statement and expression facts and
   remains unchanged.
4. `src/predicate.rs` builds `PredicateAnalysis` in Program traversal order.
   For tasks it visits `needs` and then `ensures`, preserving source-line order,
   and exposes the resulting `PredicateFact` sequence and semantic task/line
   identities. `NonExecutableProse` facts do not become Core operations. This
   module remains unchanged.
5. `src/core_lower.rs::lower_operations` currently emits all body operations in
   body-statement order and appends executable or blocked predicate operations.
   It assigns each public `operation.index` by enumeration. `CoreLowerOperation`
   has no private source-origin field.
6. `src/core_verify.rs::build_report` already receives `&Program`, constructs
   the Core-lower report, and controls verification. Its private
   `verify_lower_report` currently receives only the report. `verify_item`
   iterates candidate operations, and `verify_operation` passes
   `operation_index_consistent` whenever the public index equals the candidate
   enumeration index.
7. Consequently a complete candidate record can move with every private and
   public field, receive its new public index, and pass the current order check.
   Candidate enumeration is not Program-owned order authority.

The required dependency direction is acyclic:

```text
ast -> core_body/predicate -> core_lower -> core_verify -> main
```

`core_verify::build_report` already holds both the `Program` and the lowered
artifact. It can pass the Program into its private verifier without a
`src/main.rs` edit. `src/ast.rs` imports no Core module; it exposes only an
opaque authenticated owner borrow. `src/core_lower.rs` owns both the expected
streaming primitive and candidate-origin comparison. `src/core_verify.rs`
consumes that interface. No dependency points upstream.

## Exact borrowed authority contract

`src/ast.rs` adds this private, borrow-only authority type:

```rust
pub(crate) struct CanonicalCoreOperationOwnerExpectation<'program> { /* private */ }
```

It is constructed only by the new
`Program::canonical_core_operation_owner_expectation` entry point. That method
locates the exact Program-owned item and section through the existing
`Program::canonical_core_expectation`, performs the existing full validation,
and returns the owner expectation from the successful validated result. Core
lowering obtains the body report through its existing independently validated
Core-body call; neither result substitutes for the other. The validated AST
result must retain enough private borrows to issue the owner expectation
without cloning or reconstructing the file or owner binding.

The expectation borrows, directly or through private authenticated bindings:

- the exact `Program` container;
- the exact `SourceFile` and parser-issued source revision;
- semantic file index and normalized semantic path;
- the exact item and its traversal path and kind;
- the exact section and section slot;
- the section seal relation; and
- for tasks, the exact retained authenticated task-signature owner facts.

This value is also the independently borrowed expected-item identity used
before any operation is visited. Its private identity is the conjunction of
the live Program container, file and source revision, semantic file ordinal and
normalized path, recursive item traversal path, item kind and source span,
exact `does` section slot, parser owner binding, and section seal. No one public
field is sufficient. Item name, emitted ID, candidate ordinal, and filename are
projections only and never select or authenticate a candidate item.

It exposes only bounded crate-private operations needed by `core_lower` to
construct and compare operation expectations. Its private bindings, source
revision bytes, item path, task signature, and seal cannot be returned as owned
values or serialized.

`CanonicalCoreOperationOwnerExpectation<'program>` must not implement or
derive `Clone`, `Copy`, `Default`, `Serialize`, or `Deserialize`; it has no
public conversion, owned conversion, `'static` conversion, alternate
constructor, cache insertion, or collection conversion. Tests may cause an
existing source fact or candidate origin to be corrupted before the production
consumer, but may not mint this expectation or replace its private authority.

`src/core_lower.rs` owns the actual one-operation authority:

```rust
pub(crate) struct ExpectedCoreOperation<'program, 'invocation> { /* private */ }
```

The first lifetime borrows the Program-owned owner, source statement or source
predicate line, parser node/range, and source revision. The second borrows the
stack-local `CanonicalBodyGrammarReport` statement or `PredicateAnalysis` fact
that has been checked against that Program-owned source slot. Both lifetimes
are load-bearing: neither the Program nor the temporary analysis artifact may
be dropped while an expected operation is in use.

The expected type is a closed private source variant:

```rust
enum ExpectedCoreOperationSource<'program, 'invocation> {
    Body { /* Program parsed statement + matching CanonicalBodyStatement */ },
    Predicate { /* Program SectionLine + matching PredicateFact */ },
}
```

This layout is conceptual rather than a public representation, but the two
variants and both lifetime classes are mandatory. The Body variant must bind
the exact `Section.body_syntax` slot, parsed statement kind/root/range, body
report statement, and checked operation slot. The Predicate variant must bind
the exact task owner, `needs` or `ensures` section slot, source-line slot,
semantic task/line identity, `SectionLine`, matching analysis fact, and checked
operation slot. A predicate fact is included if and only if the existing lower
path would emit it; `NonExecutableProse` remains excluded.

`ExpectedCoreOperation<'program, 'invocation>` is stack-local, constructed one
at a time, and passed immediately to one universally quantified callback
invocation. It must not
implement or derive `Clone`, `Copy`, `Default`, serialization, deserialization,
or an owned conversion. It may not be stored in a `Vec`, map, set, cache,
registry, side table, report, candidate, `Box`, `Arc`, or other collection or
owner. It cannot be returned from the streaming session or widened to
`'static`.

## Expected operation traversal

`src/core_lower.rs` adds one private streaming entry point with this exact
Rust-expressible shape:

```rust
pub(crate) fn with_expected_core_operations_for_item<'program>(
    owner: CanonicalCoreOperationOwnerExpectation<'program>,
    body: &CanonicalBodyGrammarReport,
    predicate_facts: &[PredicateFact],
    mut visit: impl for<'invocation> FnMut(
        ExpectedCoreOperation<'program, 'invocation>,
    ),
) -> Result<(), CoreOperationExpectationError>
```

The exact error type is private, closed, and nonserialized; it distinguishes
missing, ambiguous, foreign, ordering, and checked-arithmetic failure without
adding a public diagnostic. The callback receives one
`ExpectedCoreOperation` at a time. The function is the sole expected-operation
constructor and, for its already authenticated item, performs this exact
traversal:

1. retain the exact Program/file/item/section owner obtained from the AST entry
   point rather than locating an item from candidate data;
2. walk every retained body
   statement in source order, with no filtering by status or operation kind;
3. for task items only, append the independently built predicate facts
   in the exact existing lowering order: `needs`, then `ensures`, source-line
   order, excluding only `NonExecutableProse`; and
4. reborrow the local owner, body statement or predicate fact, and all related
   artifacts for one fresh late-bound `'invocation`; and
5. issue that expected operation to the callback with its checked,
   monotonically increasing per-item slot.

`'invocation` is late-bound on `FnMut`; it is not a named lifetime parameter on
the function and cannot be selected by the caller. Passing the owner by value
keeps the authenticated owner inside the streaming session. The body and
predicate arguments deliberately have no caller-selected shared artifact
lifetime in the callback type. The implementation reborrows them for the
single call. Consequently a closure must be valid for every invocation
lifetime and cannot unify two values into a caller-owned `Vec`, return one,
retain one through the Program or artifact lifetime, or widen one to owned or
`'static`. It may copy ordinary non-authority projections needed by the current
verifier, but it cannot preserve the borrowed expected value.

The caller remains responsible for the existing Program traversal and for
building its ordinary body and predicate artifacts. Core lowering already has
those artifacts. Core verification rebuilds them independently from the live
Program before calling the same streaming function. Neither caller may retain
an expected operation after its individual callback invocation, even while the
Program, body, or predicate artifact remains alive.

Construction is transactional per item without a side collection. A private
validation-only pass walks the same source/artifact slots and constructs no
expected value, candidate, vector, count equation, or retained record. If every
slot validates, the streaming pass constructs each borrowed expectation and
lowering attaches an `Authenticated` origin exactly once. If any slot rejects,
lowering still constructs the same ordinary public operations through their
existing families but attaches `Rejected` exactly once to every operation
candidate in that item; it never leaves a mixture whose earlier candidates
appear authenticated after the item transaction failed. Verification performs its own
validation from the live Program and fresh artifacts. The validation-only and
streaming passes share the same private slot predicate; they are not distinct
producers and may not diverge.

The traversal must locally validate the relationship between the Program
source slot and the corresponding body statement or predicate fact before
issuing an expectation. It uses a closed `NoMatch`, `One`, or `Ambiguous`
association for the one local source slot. It may not use `.find` first-hit
selection. `NoMatch`, `Ambiguous`, a foreign Program owner, a foreign source
revision, duplicate local fact, reordered local fact, impossible slot, or
arithmetic failure prevents an expectation from authenticating and produces a
fail-closed verification result.

The traversal is not a global expected vector. It does not materialize a
program-wide or item-wide batch and does not compare counts. The candidate
slot is selected only by the current expected cursor in the exact corresponding
item. The expected borrow establishes authority; the slot is navigation, not
authority. All additions and cursor movements use checked arithmetic.

Body statements remain total across existing operation families. Recognized
returns, failures, bindings, mutations, calls, unsupported surface statements,
blocked `try` expressions, and every other statement currently emitted by
`core_operation_for` each receive exactly one expected slot. A blocked
operation is not filtered out. A statement with no canonical expression still
has source/order authority.

Predicates participate only through the existing operation family. Their
placement after body operations and their `needs`-before-`ensures` ordering are
authenticated against Program section/line facts and the matching analysis
artifact. No predicate type, result, or semantic conclusion becomes order
authority.

Non-task items with a `does` section use the same body expectation mechanism.
Items without lowered operations produce no operation expectation and add no
count invariant.

## Program-driven item association

Item association is completed before operation association. The Program is the
sole traversal authority. Verification recursively walks live Program items in
the same file/item order as current `core_lower::collect_items` and visits
exactly each item for which the current lowering path finds a `does` section.
It obtains a fresh
`CanonicalCoreOperationOwnerExpectation<'program>` for that Program item
without consulting `CoreLowerReport.core_items`. An item with an empty `does`
body remains an expected item even though its expected-operation stream is
empty. A Program item without a `does` section remains outside both traversals.

Each `CoreLowerItem` receives one mandatory private, non-authoritative
candidate item origin at its direct construction site:

```rust
enum CoreItemCandidateOrigin {
    Authenticated(CoreItemCandidateOriginFacts),
    Rejected(CoreOperationExpectationError),
}
```

The authenticated facts snapshot the same source revision, semantic file
identity, recursive item path, item kind and span, `does` section slot, parser
owner relation, and section seal that were present when the item was lowered.
They neither borrow nor own the expected item and cannot authenticate
themselves. The field is private, nonserialized, non-optional, and attached in
the existing `core_item` constructor. `CoreLowerItem` cannot first exist
without it. The item candidate origin has the same bans on `Default`, public
construction, serialization, deserialization, caller conversion, and
post-construction production mutation as the operation candidate origin.

Verification uses one checked `candidate_item_cursor`, initially zero, and the
following closed direct states. These states are the explicit fail-closed
equivalent of `NoMatch | One | Ambiguous`; the algorithm never searches for a
match:

1. **ExactCurrent.** `lower.core_items.get(candidate_item_cursor)` exists and
   its private candidate item origin matches the complete independently
   borrowed Program item expectation. This exact candidate is associated once,
   ordinary item verification runs once, and only then may its expected
   operations be streamed. After the item is completely verified, the cursor
   advances by checked addition of one.
2. **MissingOrMismatchedExpected.** The current candidate is absent, its origin
   is `Rejected`, or its origin does not match the expected Program item. The
   verifier emits the exact failure-only expected-item row below, performs no
   operation association for that Program item, and does not advance the
   candidate cursor. Leaving the cursor in place permits a later Program item
   to associate with that same current candidate if and only if it is its exact
   source owner; it is not a search or recovery by visible identity.
3. **RemainingUnassociatedCandidate.** After Program traversal ends, every
   candidate at or after the cursor is verified exactly once without expected
   item authority. Its existing `row_identity` check fails. The cursor advances
   by checked addition for each such candidate until the vector is exhausted.

There is no state in which two candidates associate with one expected item.
The first current exact candidate consumes the expected item once; a duplicate
remains unassociated and fails. There is no state in which one candidate
associates with two expected items because an exact association advances the
cursor and consumes the candidate once. A missing, rejected, or mismatched
expected item never consumes a candidate. All `get`, cursor, traversal-path,
slot, and ordinal arithmetic is checked; overflow, underflow, or an impossible
cursor fails closed.

This direct cursor is navigation only. Association depends on the private
Program/item-origin comparison, never the cursor value, candidate vector
position, public item or operation ID, name, filename, spelling, serialized
field, or public ordinal. No global item batch, expected-item vector,
cardinality equation, positional side vector, registry, or first-match scan is
permitted.

The resulting item behavior is exact:

- deleting the sole, middle, or final candidate item emits the expected-item
  failure at that Program item; later exact candidates remain eligible only at
  the unchanged current cursor;
- reordering complete items, even with coherently rewritten public item and
  operation indices, produces at least one missing/mismatched expected item and
  at least one remaining unassociated candidate;
- duplicating or inserting an item leaves a remaining unassociated candidate;
- a foreign item, foreign revision, or same-visible-ID foreign-revision item
  cannot match the private expected owner; and
- valid multiple-item traversal associates each candidate once in Program
  order and preserves all existing public bytes.

The exact item-level comparison is a private `src/core_lower.rs` predicate used
by current Core verification. It accepts the borrowed
`CanonicalCoreOperationOwnerExpectation` and a candidate `CoreLowerItem`, and
returns only whether the mandatory private origin occupies that exact Program
item slot. It exposes no origin, expected owner, token, or owned authority.

## Private candidate-origin contract

`src/core_lower.rs` adds one owned private closed state:

```rust
enum CoreOperationCandidateOrigin {
    Authenticated(CoreOperationCandidateOriginFacts),
    Rejected(CoreOperationExpectationError),
}
```

Every `CoreLowerOperation` owns exactly one, non-optional
`CoreOperationCandidateOrigin`, attached inside the existing body or predicate
operation constructor. No constructor may first create a Core operation
without its origin. This includes blocked, unsupported, predicate, task, and
non-task operation families.

The authenticated origin is a private snapshot sufficient to compare the
candidate with a fresh borrowed expectation. It records the authenticated
source revision/item owner relationship, exact section and source-operation
identity, source family, and checked operation slot that were present during
lowering. A locally missing, ambiguous, foreign, or arithmetically invalid
source/artifact association attaches `Rejected` to the ordinary public
candidate instead of panicking, wrapping, omitting the operation, or minting
partial origin facts. Neither state contains or points to an
`ExpectedCoreOperation`, owns expected authority, or can establish order by
itself.

The origin is absent from all serializers and human renderers. It has no public
accessor, public conversion, `Default`, serialization, deserialization, or
caller-supplied constructor. Production code never mutates it after operation
construction. Test-only corruption may replace or alter a candidate origin
before production verification, but cannot alter the Program or construct an
expected operation.

The origin may own immutable comparison values because the candidate outlives
the lowering stack. That ownership is deliberately non-authoritative. Moving a
complete candidate moves its origin and is exactly why the verifier must
compare it against a separately rebuilt borrowed expected slot.

## Core verifier consumption

`src/core_verify.rs::build_report` retains its current public command route and
passes its existing `&Program` together with the lower report into private
verification. No `src/main.rs` change is permitted.

Verification first runs the Program-driven item-association algorithm above.
Only an `ExactCurrent` item enters operation traversal. For expected operation
slot `s`, it reads only `candidate_item.operations.get(s)`. It never searches,
sorts, filters, or scans the candidate collection for a matching origin. It
then verifies the candidate normally and strengthens the existing
`operation_index_consistent` predicate to require both:

1. `candidate.index == s`; and
2. the candidate's private origin matches the complete borrowed expected
   operation for source slot `s`.

The rule name, scope, scope ID, source span, detail, row position, and pass
status remain byte-identical for valid artifacts:

- scope: `operation`;
- scope ID: the existing operation ID;
- source span: the existing operation span;
- rule: `operation_index_consistent`;
- detail: `operation index is {operation.index}`; and
- status: `passed_v0` only when both predicates above pass.

The public index remains useful reporting data but is not authority. A
candidate with the right private origin in the wrong slot fails. A candidate in
the right slot with a foreign or altered origin fails. Swapping both complete
candidates and rewriting their public indices fails because each traveling
origin is compared with the other Program-owned expected slot.

If independent expected construction reports a local association error while
a candidate exists, the candidate is verified with no accepted expectation and
its `operation_index_consistent` row fails. A `Rejected` candidate origin also
always fails that row. Existing task-signature and body-grammar failures remain
independently load-bearing and are not removed, but no such earlier failure may
turn an order row into a pass.

`src/core_lower.rs` owns one private, currently used comparison predicate with
the semantic contract:

```rust
core_operation_occupies_expected_slot(
    expected: &ExpectedCoreOperation<'_, '_>,
    candidate: &CoreLowerOperation,
) -> bool
```

The exact Rust visibility may be no broader than crate-private and the
arguments remain borrowed. The predicate compares the candidate's mandatory
private origin with the complete expected source owner and slot. It is the sole
private reuse point for later checked-type work because Core verification uses
it now; it is not a dormant API. No verified order fact or borrowed expectation
is exposed outside the current callback/verification frame. There is no
operation-order view, token, report extension, serialized fact, or semantic
consumer in this unit. The load-bearing current result is the existing Core
verification verdict and CLI exit.

## Missing item, missing operation, and extra-candidate semantics

The current `CoreVerifyReport` has one flat `checks` stream and projects items
only from existing `CoreLowerItem` candidates. It can therefore represent a
missing Program-owned item without inventing a synthetic lower item. At the
Program item boundary, `MissingOrMismatchedExpected` emits exactly one
failure-only check into that existing flat stream:

- scope: `core_item`;
- scope ID: the expected public Core item ID computed from the borrowed Program
  item by the exact existing projection
  `node_id::span("core-item", item.span(), &format!("{} {}", item.kind(), item.name()))`;
- source span: the Program-owned expected item's exact source span;
- rule: `expected_core_item_present`;
- detail: `parser-owned Core item has one exact lowered candidate`; and
- status: `failed_v0`.

The projected scope ID is reporting only. It neither selects a candidate nor
participates in the private association predicate. The row is ordered after
the three existing summary checks and all rows for any preceding exactly
associated Program item, at the precise position where the missing Program
item would have been verified, and before rows for the next Program item. A
missing sole item follows the three summary checks; a missing final item follows
the preceding associated item completely. No nonexistent `CoreLowerItem` or
synthetic item JSON row is created.

This check increments the existing failed-check summary, makes the root status
`core_artifact_verification_failed_v0`, makes `verified_items` and
`verified_operations` zero under the existing all-or-nothing summary methods,
and makes the existing `core-verify` CLI exit nonzero. Existing candidates that
are otherwise exactly associated retain their own ordinary item projection;
there is no candidate item to receive a per-item status for a wholly missing
item. Full-type and every other downstream consumer observe the existing
failed Core-verification result and cannot treat the artifact as verified.

Every `RemainingUnassociatedCandidate` reuses its existing item
`row_identity` position, scope, scope ID, and span. On the exact valid path that
row remains byte-identical: `passed_v0` with detail `core item id is present`.
For an unassociated duplicate, extra, reordered, foreign, or substituted
candidate, the same row is `failed_v0` with exact detail
`core item has no exact Program-owned source-slot association`. That failure
marks the existing candidate item failed through current item/root propagation.
No additional success row or failure-only unexpected-item rule is added.

Ordinary candidate iteration cannot emit an operation-scoped row for a missing
candidate. Therefore Unit 1 also adds exactly one operation-level failure-only
rule family and no operation-level success row:

- scope: `core_item`;
- scope ID: the existing corresponding Core item ID;
- source span: the missing expected operation's Program-owned source span;
- rule: `expected_core_operation_present`;
- detail: `parser-owned Core operation slot has one lowered candidate`;
- status: `failed_v0`.

When `operations.get(s)` is absent, the verifier emits this row immediately for
the Program-owned source slot and continues the independent source traversal
safely. A local artifact-association error cannot erase that source slot: if
the candidate is absent, this missing row still appears; if it is present, the
existing order row fails as specified above. The row participates in existing
item/root failure propagation, the failed-check summary, and nonzero CLI exit.
It is never emitted for a valid artifact, so valid check counts and bytes do
not change.

After the expected stream for an item is exhausted, every remaining candidate
is verified once with no expected slot and fails its existing
`operation_index_consistent` row. This is direct end-of-stream handling, not a
cardinality equation. No count comparison authenticates or selects a record.

The resulting closed behavior is:

| Condition | Expected traversal | Candidate handling | Required result |
| --- | --- | --- | --- |
| Exact valid item order, including empty items | one borrowed expected item per current lowerable Program item | exact current candidate item and matching private origin | existing item rows pass and operation traversal may begin; output byte-identical |
| Sole, middle, or final candidate item deleted | Program item remains authoritative | current candidate absent or belongs to a later item; cursor does not advance | failure-only `expected_core_item_present` row at the Program item boundary |
| Complete candidate items reordered with public item/operation indices rewritten | Program item order does not move | a later item may match only its own expected owner; another candidate remains unassociated | missing-item row and/or existing item `row_identity` failure; nonzero exit |
| Candidate item duplicated or inserted | one expected item is consumed at most once | duplicate or inserted item remains unassociated | existing item `row_identity` fails with the exact unassociated detail |
| Foreign item/revision or same-visible-ID foreign revision | local expected owner unchanged | private item origin does not match; public IDs are ignored | expected-item row plus unassociated candidate failure |
| Exact valid order | one borrowed expectation per emitted operation | exact same-slot candidate and matching origin | existing rows all pass; output byte-identical |
| Public index only is corrupt | unchanged expected slot | same candidate, wrong public index | `operation_index_consistent` fails |
| Complete records swapped | expected slots remain Program-owned | candidate origins travel | both affected order rows fail |
| Complete records swapped and public indices rewritten | expected slots remain Program-owned | traveling origins disagree with new slots | both affected order rows fail |
| Private origins swapped only | expected slots remain Program-owned | public candidates stay in place | both affected order rows fail |
| Sole or final candidate deleted | expected slot still visited | `get(s)` is absent | failure-only `expected_core_operation_present` row |
| Middle candidate deleted | all expected slots still visited | later candidate mismatches; final slot absent | origin failure plus missing-slot row |
| Candidate duplicated | expected slot remains unique | duplicate mismatches a later slot or is extra | order row fails |
| Extra candidate inserted or appended | no new expected authority exists | shifted or trailing candidate has no matching expected slot | order row fails |
| Foreign item or revision | local expected owner unchanged | candidate origin is foreign | order row fails |
| Same visible ID from foreign revision | local expected owner unchanged | private revision/owner mismatch | order row fails |
| Identical text and public IDs reordered | expected parser roots remain in source slots | candidate origin travels | order row fails |
| Expected traversal is missing, ambiguous, or arithmetically invalid | no authority issued for that slot | candidate cannot self-authenticate | verification fails closed; no panic or fallback |

A failure may also trigger existing source-span, row-identity, family, status,
blocker, expression, or item consistency checks. Permanent corruption tests
must isolate the order predicate where the matrix requires it and prove the
borrowed expected/candidate comparison is load-bearing rather than relying on
an unrelated earlier failure.

## Public contract

`hum.core_lower.v0` is byte-for-byte unchanged. `CoreOperationCandidateOrigin`
is a private field omitted by the existing manual serializer and human output.
No Core-lower field, literal, ordering, status, count, schema, or exit behavior
changes.

`hum.core_verify.v0` keeps every valid row and aggregate byte-for-byte
unchanged. The existing item `row_identity` and operation
`operation_index_consistent` rows have stronger private pass predicates but
identical serialized fields and details on the valid path. The exact new
failure-only values are `expected_core_item_present` for an absent or
mismatched Program-owned item and `expected_core_operation_present` for a
missing Program-owned operation. Both are representable by the existing flat
`CoreVerifyCheck` structure. An unassociated candidate item uses the existing
`row_identity` rule with the exact failure detail frozen above. None of these
failure-only or conditional failure values appears for a valid artifact.

For any item- or operation-order rule failure:

- the row status is `failed_v0`;
- an existing candidate item containing the failed row becomes
  `core_artifact_item_verification_failed_v0` by existing propagation; a wholly
  missing expected item has no synthetic candidate-item projection;
- the root status becomes `core_artifact_verification_failed_v0`;
- `verified_items` and `verified_operations` become zero under the existing
  report-wide all-or-nothing summary methods;
- failed/passed counts are derived by the existing summary logic; and
- the existing `core-verify` command exits nonzero.

No private source revision, owner path, task signature, parser node, expected
operation, or candidate origin serializes. Failure detail does not reveal the
private mismatch reason. Human failure rendering uses the existing generic
failed-check path.

The following remain byte-for-byte and behaviorally unchanged:

- valid Core-lower human and JSON output;
- valid Core-verify human and JSON output and check counts;
- full-type human and JSON output;
- `hum.type_check.v0`, `hum.core_preview.v0`, and
  `hum.ir_readiness.v0`;
- capabilities and version;
- diagnostic selection and precedence;
- execution and IR readiness, both still zero where currently zero; and
- all downstream effect, ownership, resource, profile, IR, backend, and
  execution behavior.

## Mandatory next checked-type consumer obligation

Work Order 16 is a type-agnostic foundation with an immediate current consumer
in Core verification, but it is not an independent architectural destination.
After this unit is accepted, published, terminal-green, and closed, the next
checked-type-authority Work Order must consume this exact ownership predicate.
That later Work Order is constrained as follows:

1. Its non-authoritative type candidate claim must be attached to the same
   exact `CoreLowerOperation` and its `CoreOperationCandidateOrigin`; it may not
   live in a parallel record later joined back to an operation.
2. Within the same invocation of
   `with_expected_core_operations_for_item`, Core verification must first
   establish exact Program-item association and then call
   `core_operation_occupies_expected_slot` for the exact candidate. Type
   verification may begin only after that same-slot result and all relevant
   existing structural checks pass.
3. Missing, mismatched, ambiguous, rejected, reordered, duplicate, extra, or
   foreign item association; missing or mismatched operation association;
   failed public index/order comparison; failed candidate-origin comparison;
   or any other load-bearing structural failure must withhold every verified
   type result for the affected artifact. A previously constructed or partial
   type result may not survive a later failed check.
4. The later full-type consumer must receive only a lifetime-bound verified
   type result through its bounded borrowed/HRTB handoff. It may not receive
   the Program owner, expected operation, candidate origin, or unverified claim
   directly, and it may not retain the result beyond the verified artifact and
   authority lifetimes.
5. The later producer/verifier must reuse the exact authenticated
   `CoreOperationCandidateOrigin` and the currently load-bearing
   `core_operation_occupies_expected_slot` predicate in the same callback
   frame. It must not reconstruct, cache, collect, or rerun a second
   expected-operation producer.
6. Candidate-owned identity, public IDs or indices, serialized type fields,
   lowered-vector position, spelling, result declaration, and type-candidate
   self-consistency remain non-authoritative and cannot replace source-slot
   ownership.
7. The dependency remains one-way:
   `Program -> borrowed expectation -> Core lower candidate -> Core verify
   order verdict -> checked-type verification -> borrowed full-type
   consumption`. Full type cannot call back into Core verification or the type
   producer, and Core verification cannot depend on full type.

This section freezes a requirement on the next separately planned Work Order;
it adds no type field, type producer, verified type view, callback, public API,
serializer value, or dormant authority mechanism to Unit 1. The private reuse
point is already required and exercised by current Core order verification.

The established `UInt` fixture remains a frozen compatibility boundary. With
the exact relative path
`fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum`, all current
direct-process results must remain exact:

| Surface | Exit | stdout bytes / SHA-256 | stderr bytes / SHA-256 |
| --- | ---: | --- | --- |
| `core-lower` | 0 | 11,378 / `7c27ce1b320ecf24611a64ff356401cc2f129ae3f8bc8e8870e1efa38c7f69cf` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `core-lower --format json` | 0 | 75,957 / `ae44f177408c059db415bda2b53ac36dd3fbc07c44ac75e059752a5d037d561b` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `core-verify` | 0 | 1,826 / `4ecf43a884856ff090ea4ee1d90d3e3c640295f47320d54bb0c58d136664fe3f` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `core-verify --format json` | 0 | 344,937 / `94549e1a3a314fa497e45ebb63ebb5852e57affd3e4d36f1cd434a7f99ce4ac5` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `full-type-check` | 1 | 15,680 / `13aef5de9315cac1c5f600d80e73e251f6cc8a7e1b4c84d4d5a4f88ea1a6c0c3` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `full-type-check --format json` | 1 | 60,879 / `47bff4d4dcaedb53be8b0d1cf158f93192993d08dfec829b5ebbd33f6c9ca987` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Its Core verifier remains 766 of 766 checks passing with exit 0. Its existing
unrelated resolver blocker remains the sole full-type blocker. These values are
evidence for the current tree, never production selection authority.

## Exact four-path implementation envelope

Unit 1 may modify exactly these four paths:

1. `src/ast.rs`
   - extend the validated canonical Core section result so it can yield the
     opaque borrow-only `CanonicalCoreOperationOwnerExpectation<'program>`;
   - preserve parser/file/owner/section/task-signature authority without
     exposing or cloning private bindings; and
   - provide module-local corruption evidence for foreign source, item,
     section, and revision substitution.
2. `src/core_lower.rs`
   - define `ExpectedCoreOperation<'program, 'invocation>`, its two source
     variants, and the sole generative HRTB streaming constructor;
   - define and attach one `CoreItemCandidateOrigin` to every lowered item and
     one `CoreOperationCandidateOrigin` to every operation;
   - implement exact body and predicate source-order traversal and private
     item/operation origin comparison; and
   - own the focused lower selector and production-type compile-fail probes.
3. `src/core_verify.rs`
   - pass the existing Program into private report verification;
   - drive direct item association from Program order with checked cursor
     semantics, emit the failure-only missing-item row, and reject every
     remaining unassociated candidate item;
   - compare each expected operation with only its exact candidate slot;
   - strengthen `operation_index_consistent` without changing valid rows;
   - emit the failure-only missing-operation row; and
   - own all item- and operation-level reorder, deletion, duplication, extra,
     foreign, arithmetic, and byte-preservation evidence.
4. `docs/HUM_CORE_VERIFY_SCHEMA.md`
   - document the strengthened private predicates of item `row_identity` and
     `operation_index_consistent`, both failure-only missing rules, exact
     unassociated-item detail, propagation, private-authority boundary, and
     valid-output preservation.

Every path is load-bearing. Removing `src/ast.rs` leaves no source-owned opaque
owner borrow. Removing `src/core_lower.rs` leaves neither the expected stream
nor per-item/per-operation candidate origins. Removing `src/core_verify.rs`
leaves no production consumer. Removing the schema leaves new emitted failure
rules and conditional failure detail undocumented.

A fifth implementation path is an explicit stop and requires BDFL re-scope
before any edit. The following paths are deliberately excluded and must remain
byte-identical:

- `src/parser.rs`;
- `src/core_body.rs`;
- `src/predicate.rs`;
- `src/resolve.rs`;
- `src/type_env.rs`;
- `src/type_check.rs`;
- `src/full_type_check.rs`;
- `src/core_preview.rs`;
- `src/ir_readiness.rs`;
- `src/main.rs`;
- every Core-lower and full-type schema;
- `Cargo.toml` and lockfiles;
- tools and workflows;
- fixtures, examples, snapshots, and generated files; and
- all downstream semantic and runtime passes.

No listed path is a placeholder for later type work. No excluded path may be
changed to add an adapter, source audit, test harness, golden update, route, or
compatibility workaround.

## Focused permanent selectors

The final implementation must define exactly these two stable tests:

1. `core_lower::tests::core_operation_candidate_origin_is_attached_once`
2. `core_verify::tests::borrowed_core_operation_expectation_is_load_bearing`

The existing exact-selector helper must list exactly one nonzero test and run
exactly one passing test for each name. Each selector earns exactly one credit.
No tool edit is authorized.

The Core-lower selector must use the real production lowering constructors and
prove:

- every lowered item has exactly one non-optional candidate item origin and no
  alternate item constructor omits or duplicates it;
- every emitted body, blocked, unsupported, predicate, task, and non-task
  operation has exactly one non-optional origin;
- no alternate item or operation constructor omits or duplicates its origin;
- origin association is independent of public index, ID, text, and filename;
- origin values remain absent from human and JSON output; and
- moving an item or operation candidate moves only non-authoritative candidate
  material.

The Core-verify selector must use the real Program, expected stream, lowering,
and verifier. It must cover all positive and corruption rows below, inject each
corruption before the production verifier, and prove no unrelated check is the
sole reason for the load-bearing order failures.

## Permanent positive and corruption matrix

Positive production evidence must include:

- `examples/core/minimal_add.hum` without making a type claim;
- `fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum` with exact
  766/766 Core verification and frozen bytes;
- an existing input with multiple ordinary body operations;
- an existing blocked or unsupported body operation, including blocked `try`
  where present;
- an existing task with predicate-appended operations, proving body operations
  precede `needs` and `ensures` operations;
- a representative non-task item with a `does` body; and
- valid multiple-item traversal, including a task or item with no emitted
  operation, proving the item still associates directly while creating no
  operation batch/count invariant and not invalidating another item.

The table-driven corruption matrix must include at least:

| Corruption | Public index treatment | Required load-bearing evidence |
| --- | --- | --- |
| Delete the sole candidate item | none exists | Program item boundary emits exact `expected_core_item_present` row after summary rows |
| Delete a middle candidate item | rewrite later item/operation indices coherently | missing Program item fails; later exact association uses the unchanged cursor |
| Delete the final candidate item | earlier items unchanged | final Program item still emits the exact missing-item row |
| Reorder complete candidate items | rewrite all public item/operation indices coherently | direct Program-order association fails; traveling item origins cannot authenticate new slots |
| Duplicate or insert a candidate item | rewrite following indices coherently | duplicate/extra remains unassociated and its existing item `row_identity` fails |
| Append an extra candidate item | use plausible public identity | remaining candidate is verified once without expected authority and fails |
| Foreign item or source revision | make public item IDs and spans equal | private expected-item/origin comparison fails |
| Same-visible-ID foreign-revision item | copy every public identity projection | expected-item row and unassociated candidate failure remain load-bearing |
| Swap two complete operations | leave indices unchanged | existing order row fails; also show public-index failure is not the only protection |
| Swap two complete operations | rewrite both indices coherently | borrowed expected/origin comparison fails |
| Swap only private origins | public fields unchanged | borrowed expected/origin comparison fails |
| Delete the sole operation | none exists | missing-operation failure-only row appears |
| Delete final expected operation | earlier operations unchanged | final expected slot still produces missing row |
| Delete a middle operation | rewrite later public indices coherently | mismatch and final missing slot fail |
| Duplicate an operation | rewrite indices coherently | duplicate cannot authenticate later expected slot |
| Insert an extra operation | rewrite indices coherently | shifted/trailing operation fails without expected authority |
| Append an extra operation | use next plausible index | no expected slot; existing order row fails |
| Foreign item origin | make public item/operation IDs equal | private owner mismatch fails |
| Foreign source revision | preserve visible spans and IDs | private revision mismatch fails |
| Same-visible-ID foreign revision | copy all public identity text | private revision/owner mismatch fails |
| Coherent text/ID/range reorder | make candidate surface fields agree | Program parser-root slot remains authoritative and fails |
| Body/predicate boundary swap | rewrite indices and public family fields | source variant and source slot fail |
| Predicate `needs`/`ensures` reorder | rewrite public fields coherently | Program section/line expectation fails |
| Checked slot overflow | no wrapping candidate | verification fails without panic |
| Checked slot underflow or impossible ordering | no saturating workaround | verification fails without panic |
| Missing local body/predicate artifact match | candidate retained | expected construction fails closed |
| Duplicate local artifact match | candidate retained | `Ambiguous` fails closed, not first-hit acceptance |

For the first swap-without-rewrite case, the test must separately demonstrate
the strengthened private comparison by arranging a variant in which public
index failure is neutralized. For every foreign/substitution case, corruption
must happen before the real verifier; a source-text assertion or a direct call
to a comparison helper is supplemental only.

No test may choose production behavior by a fixture path, task name, parameter
name, statement spelling, public ID, vector count, or corpus position. Literal
paths may be used only to load the named regression inputs.

## Compile-time lifetime proof

`src/core_lower.rs`, beside the actual production
`ExpectedCoreOperation<'program, 'invocation>` type, adds four cfg-selected
functions:

- `expected_core_operation_artifact_escape_must_not_compile`;
- `expected_core_operation_program_escape_must_not_compile`;
- `expected_core_operation_static_escape_must_not_compile`;
- `expected_core_operation_collection_escape_must_not_compile`.

They are selected only by
`hum_compile_fail_expected_core_operation_escape` and attempt, respectively,
to return the expectation after the stack-local analysis artifact is dropped,
return it after the Program borrow is dropped, widen it into an owned or
`'static` authority-bearing value, and push expectations from multiple real
streaming callback invocations into one caller-owned `Vec`. The collection
probe calls the actual
`with_expected_core_operations_for_item` production route. It must fail
because the callback parameter's late-bound `'invocation` would escape the
closure or would have to unify distinct universally quantified invocations,
not because the type is private or lacks a named import. The probes use the
real production type and constructor route, not a toy re-declaration.

The collection probe has this concrete production-type body shape beside the
private types; its parameters are the real inputs and it calls the real stream:

```rust
#[cfg(hum_compile_fail_expected_core_operation_escape)]
fn expected_core_operation_collection_escape_must_not_compile<'program>(
    owner: CanonicalCoreOperationOwnerExpectation<'program>,
    body: &CanonicalBodyGrammarReport,
    predicate_facts: &[PredicateFact],
) {
    let mut escaped = Vec::new();
    let _result = with_expected_core_operations_for_item(
        owner,
        body,
        predicate_facts,
        |expected| escaped.push(expected),
    );
    drop(escaped);
}
```

The multi-operation production selector separately proves the stream invokes
the closure more than once. The compile-fail probe proves no invocation value
can enter the caller-owned collection at all.

The required proof sequence is:

```text
cargo check --all-targets
RUSTFLAGS=--cfg hum_compile_fail_expected_core_operation_escape cargo check --all-targets
cargo check --all-targets
```

On Windows the middle command sets `RUSTFLAGS` process-locally and restores the
prior environment even after failure. It must exit 101, name all four
functions, and contain the intended E0515/E0521 or equivalent lifetime
diagnostics. The collection function must be named in a diagnostic whose cause
is borrowed callback data escaping into the `Vec`.
It must contain no privacy, missing type, unresolved import, unexpected-cfg,
or unrelated first failure. The normal commands before and after must succeed,
and `RUSTFLAGS` must be absent or restored afterward.

The proof requires no Cargo edit, dependency, fixture, generated source, or
fifth path. If the actual type cannot support an unambiguous compile-fail proof
inside `src/core_lower.rs`, implementation stops instead of weakening the
requirement.

## Implementation evidence and platform coverage

An authorized implementation session must, in order:

1. run each focused selector through the exact-selector helper, proving list
   count one, run count one, pass count one, and one unique credit;
2. run the production-type lifetime proof with normal checks before and after;
3. exercise the positive body, blocked, predicate, and non-task cases through
   real Core lowering and Core verification;
4. exercise every item- and operation-level corruption through the production
   verifier, including valid multiple-item traversal and all sole/middle/final
   missing-item positions;
5. reproduce the six `UInt` surface exits and all twelve byte/hash values;
6. capture minimal Int, multiple-operation, blocked-operation,
   predicate-bearing, and non-task Core-lower/Core-verify human and JSON output;
7. run every required JSON surface twice and require byte-identical output;
8. prove valid Core verification check counts and row order are unchanged;
9. run `cargo fmt --all -- --check`;
10. run `cargo check --all-targets`;
11. run `cargo clippy --all-targets -- -D warnings`;
12. run `cargo test --all-targets` exactly once on the final candidate;
13. run `git diff --check`, text hygiene, public readiness, alpha claims, and
    release readiness for `0.0.1`; and
14. leave the exact candidate unstaged, with an empty index and no artifact.

Local implementation evidence covers the Windows host and every Rust target
compiled by `--all-targets`. Source inspection must cover all cfg branches.
Linux runtime remains unexercised locally. After independent implementation
acceptance, a separately authorized commit and publication must use full CI;
Ubuntu and Windows each own full preflight, while Ubuntu alone owns the
platform-independent Exhaustive producer. No local Fast, `tools/check_all.ps1`,
full preflight, Exhaustive, performance pair, validation ledger, actor
transcript, or publication CI run is authorized during implementation.

## Explicit exclusions and bans

Unit 1 forbids:

- checked type production or any expression type conclusion;
- minimal-add recognition or classification;
- resolver, type-environment, or checked-declaration joins;
- a supported/out-of-scope/integrity/noncanonical or six-way type disposition;
- typed Core fields, public or private type claims, verified type views,
  full-type callbacks, or fallback changes;
- using a declared result as expression evidence;
- global expected-operation vectors, classification batches, authority
  registries, manifests, caches, or ledgers;
- program-wide or item-wide cardinality equations or filtered operation counts;
- selection by first match, public index, ordinal alone, public ID, filename,
  spelling, source text, JSON, fixture identity, or corpus position;
- storing, cloning, owning, serializing, widening, or returning borrowed
  expected authority;
- reconstructing expected authority from candidate origin or candidate output;
- letting a candidate origin authenticate its own slot;
- removing or weakening any existing structural, signature, expression,
  blocker, item, or root check;
- new diagnostic codes, commands, schema families, routes, fixtures, examples,
  tools, workflows, dependencies, or validation infrastructure;
- reuse of any Work Order 15 archive implementation or test; and
- IR, IR readiness, backend, runtime, execution, effects, ownership,
  resources, profiles, optimization, performance, or safety work.

No unsafe code is permitted. All range, slot, and cursor arithmetic is checked.
No panic, wrap, saturation, truncation, unchecked addition, or unchecked
subtraction may turn corrupt authority into acceptance.

## Sustainability boundary and stop conditions

The expected implementation is approximately 800 to 1,200 insertions and no
more than 100 deletions across the four authorized paths. This is a mandatory
pre-implementation sustainability boundary, not merely telemetry. If an honest
candidate would exceed either bound, implementation stops before widening and
returns to the BDFL for re-scope.

The mechanism must remain one compact producer/consumer relation plus
table-driven evidence. It must be reviewable by one fresh architect in roughly
two to three focused hours. It may not split into incomplete subunits or leave
an intermediate state in which candidate origins exist without a production
expected consumer.

Stop without workaround if:

- a fifth path is required;
- `src/core_body.rs` or `src/predicate.rs` must expose new authority;
- a Program-owned owner expectation cannot be borrowed through `src/ast.rs`;
- the expected operation must be owned, cloned, collected, cached, or made
  `'static`;
- a global vector, batch, registry, count equation, first-match lookup, or
  positional side table appears necessary;
- a candidate must be selected by public index or visible identity;
- Program item order cannot directly associate every candidate item without a
  batch, side vector, search, unchecked cursor, or fifth path;
- a sole, middle, final, reordered, duplicate, extra, foreign, or
  same-visible-ID foreign-revision item does not fail by the exact item
  semantics above;
- whole-record swap plus coherent public-index rewrite can pass;
- a sole, middle, or final deletion is not observed;
- blocked, predicate, or non-task operations cannot participate honestly;
- valid bytes or check counts drift;
- the `UInt` fixture no longer reports 766/766 and exit 0;
- the compile-fail proof fails for the wrong reason;
- a selector lists zero or multiple tests;
- an existing check must be weakened or a public contract cannot be stated
  exactly;
- type authority or another later semantic feature becomes necessary; or
- the size or review-time boundary is exceeded.

Do not invent an adapter, broaden the envelope, recover archive code, update a
golden, special-case a fixture, or weaken the result after a stop.

## Independent review, correction, commit, and publication gates

The first independent pre-issuance reviewer returned
`ACCEPT WITH REQUIRED FIX` with three P1 findings: the callback lifetime was
caller-unifiable, Program-item/candidate-item association was incomplete, and
the next checked-type consumer obligation was implicit. The BDFL authorized
this one bounded correction to `WORKORDER_16.md`; that correction cycle is now
consumed.

The exact corrected two-document draft requires one fresh independent
corrected-document architect-reviewer who did not author or edit either
document or perform this correction. Review must trace current-main Program
item traversal, all body and predicate operation families, the generative
callback and candidate lifetimes, direct item association, same-slot operation
verification, missing and extra handling, the future checked-type reuse
obligation, schema compatibility, the two selectors, all four compile-fail
probes, four-path closure, and sustainability.

The corrected-document reviewer returns `ACCEPT`,
`ACCEPT WITH REQUIRED FIX`, or `REJECT` with P0/P1/P2 findings. No verdict
authorizes an edit. Only unconditional `ACCEPT` advances; any other verdict
stops this draft at the BDFL. No second document correction, wording repair,
envelope amendment, or workaround is authorized.

Only explicit BDFL acceptance of exact bytes may authorize a local
documentation commit. Publication requires a separate signal and normal
fast-forward push. Activation is not a status-only transition and must receive
terminal-green full CI. A separate status record and its publication are then
required. Only after all of those gates may the BDFL separately authorize Unit
1 implementation.

An implementation candidate receives a fresh independent review. Work Order
16 permits at most one separately authorized bounded in-envelope
implementation correction after the first implementation verdict. Independent
acceptance authorizes no commit or push; each remains a separate BDFL gate.

Post-publication CI owns full preflight. Local Fast and Exhaustive are never a
substitute for the required publication workflow.

## Current authorization gate

This Work Order remains a fresh planning draft only. Work Order 15 is
terminally rejected, archived, closed, inactive, and non-authoritative. The
first independent review and its three P1 findings are recorded above; the
sole bounded document correction is consumed. The next authorization may
permit only fresh independent corrected-document review of the exact Work
Order 15 closeout and corrected Work Order 16 draft package.

Implementation, further correction, staging, commit, push, publication, archive
mutation, status transition, checked type work, and later planning are
unauthorized. Document acceptance, if granted, will still require a separate
BDFL local-commit signal and a later publication signal. Unit 1 requires its
own separate signal after published terminal-green document and status gates.
<!-- workorder-current-authorization-gate:end -->
