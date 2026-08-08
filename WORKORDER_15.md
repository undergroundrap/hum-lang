# Hum Work Order 15: Bind Canonical Minimal-Add Type Authority Per Operation

Date: 2026-08-02
Status: Terminally rejected, archived, closed, and inactive. Unit 1 produced no
accepted implementation and no implementation commit or publication on
`main`. The rejected candidate is preserved only as failure evidence on the
dedicated archive branch identified below. All Work Order 15 implementation,
correction, review, commit, push, repair, reuse, merge, cherry-pick, and patch-
application authority is closed. Checked minimal-add type authority remains
deferred and receives no authority from this closeout.

Owner: BDFL (Ocean).
Author: Work Order 15 architect-author. The author may explain this draft but
is permanently disqualified from independently reviewing these bytes or an
implementation produced under them.
Original planning baseline: `88447ab178f372139fb8411d198b267b4d24e01f`.
Terminal closeout baseline: `HEAD`, local `main`, cached `origin/main`, and live
remote `main` are all `39d373dea56a15a5f97b76e701305bcd4d3e1f02`.

## Terminal Unit 1 closeout

Work Order 15 was issued and published, and its separate publication status
record reached terminal-green CI. A first implementation candidate was then
reviewed independently. That review returned `ACCEPT WITH REQUIRED FIX` with
no P0 or P2 finding and these three P1 findings:

1. the resolver/declaration association still began from public-ID-shaped
   material instead of a complete private source-owned join;
2. a public `view_issued` report row could pass while the private accessor
   withheld the verified view; and
3. the permanent corruption matrix did not cover all load-bearing authority
   and disposition combinations.

The BDFL authorized the sole bounded in-envelope implementation correction.
The corrected candidate remained exactly within the issued nine-path source
and schema envelope and was submitted to a fresh final independent review.
That final review returned `REJECT`. It reported no P0 finding, three terminal
P1 findings, and one P2 finding:

1. swapping complete Core operations and coherently rewriting their public
   indices was still accepted because the private identity traveled with each
   candidate record;
2. the required borrow-only
   `ExpectedCanonicalMinimalAddOperation<'program>` was replaced with an owned
   candidate-identity representation, so verification did not compare against
   independently retained Program-owned expected authority; and
3. the permanent anti-downgrade matrix did not prove `Supported` against every
   lower disposition.

The P2 finding was a schema defect: the phrase "unchecked outer type state"
was not an exact statement of the affected public state and could mislead a
future implementer or reviewer.

The terminally rejected candidate is preserved only at:

- branch:
  `archive/workorder-15-unit1-terminal-rejection-2026-08-08`;
- commit: `f47ec6f3b7586cd59077f90ffb66307af7d045ee`;
- parent: `39d373dea56a15a5f97b76e701305bcd4d3e1f02`;
- complete tree: `cb70a481879e9434d8b8be598f671c125ead9839`;
- scoped nine-path tree: `d8356acb28e8a5441999ff550b5c0551c46f1723`;
- inventory: exactly nine paths, `+3,021/-124`; and
- archive CI: none; publishing the archive branch triggered no workflow.

The local archive ref, cached origin-tracking ref, and live remote archive ref
all resolve to the exact archive commit. The archive is non-authoritative
failure evidence. No archived source, test, patch, assertion, or result may be
copied, restored, adapted, cherry-picked, merged, applied, or used as an
implementation base. No Work Order 15 implementation was accepted, committed,
or published on `main`.

## Closed predecessor and accepted foundations

Work Order 14 Unit 1 is independently accepted, published, terminal-green,
status-closed, complete, and frozen. Its accepted implementation commit is
`e6c38b70b97a3dcc205c9c1b0533352603541f95`. Full CI succeeded in workflow
`30763812498`, attempt `1`, on Ubuntu job `91538922265` and Windows job
`91538922222`. Its status-closeout commit is the planning baseline; the fast
workflow `30768159757`, attempt `1`, succeeded on Ubuntu job `91550488214` and
Windows job `91550488249`.

Two accepted foundations are now on `main`:

1. Work Order 12 transports the parser-owned, ordered task-return root
   `Binary(Add, Identifier, Identifier)` through Core lowering and Core
   verification while keeping its outer type honestly unchecked.
2. Work Order 14 authenticates the exact task signature against its retained
   source-revision bytes, owns that private authority directly in the
   corresponding Core item, and makes Core verification fail when the lowered
   signature disagrees.

Work Order 13 is terminally rejected, archived, closed, and non-authoritative.
Its rejected implementation exists only as failure evidence at
`archive/workorder-13-unit1-terminal-rejection-2026-08-02`, commit
`f19c85748426867f0d4b3d5556ec5ed494a81e4c`. Its global classification batch
compared 49 records with 47 ordinal-bearing operations and falsely failed the
established `UInt` witness. No archived source, test, patch, or asserted result
may be copied, restored, cherry-picked, merged, or treated as an implementation
base. The Work Order 11 archive at
`a40fc65876a9224adecc492b18617ec60684136c` is equally outside this unit.

The first independent pre-issuance review of this Work Order returned
`ACCEPT WITH REQUIRED FIX` with three P1 findings and no P0 or P2 finding:
unsupported additive target-like roots were not separated from genuine
non-targets, the verified-access callback had no total failed-report branch,
and ordinary operation iteration could not observe deletion of an expected
operation. The first BDFL correction instruction overgeneralized the first
finding by making every non-exact additive root fail; the author stopped before
editing after proving that instruction would break accepted
`Identifier + UIntLiteral` behavior. The BDFL superseded only that
contradictory instruction. This document is the sole bounded correction cycle.
One fresh final independent corrected-document review remains; any verdict
other than unqualified `ACCEPT` stops Work Order 15 at the BDFL, with no further
correction authorized.

## Purpose and exact Unit 1 result

Unit 1 establishes the smallest load-bearing semantic link from the two
accepted foundations to the existing minimal-add type conclusion:

```text
authenticated task owner + parser-owned return/add tree
  -> exact resolver references -> exact local declaration facts
  -> one private per-operation six-way decision
  -> one operation-owned untouched Int authority and non-authoritative claim
  -> Core projection -> Core verification
  -> total report-bound outcome -> lifetime-bound verified access when supported
  -> full-type statement result
```

The supported rule is deliberately narrow. It applies only to a task return
whose authenticated parser root is exactly
`Binary(Add, Identifier, Identifier)`, whose two identifier nodes each resolve
to a parameter definition of that exact task, and whose authenticated
signature facts, type-environment declarations, and accepted checked
declarations all say the selected operands are exactly `Int`. The sole
canonical producer concludes `Int`. The declared task result is checked later
for compatibility; it never proves an operand or expression type.

The result is load-bearing in two places: Core verification rejects a corrupt
candidate before it can yield verified access, and full type obtains the
supported statement's actual type only from that verified access. This is not
general addition typing, general expression inference, operator overloading,
typed-Core completeness, Hum IR, execution, backend work, optimization, or a
safety proof.

## Verified production dependency map

The planning audit found this current, acyclic route:

1. `src/ast.rs` already owns `AuthenticatedCanonicalTaskSignature`, backed by
   the parser-issued file revision, file index, normalized path, item path,
   header bytes, ordered syntax, and live-task validation. It currently exposes
   only lowered-signature matching; it lacks one opaque revision/item join key
   suitable for a later private authority.
2. `src/core_body.rs` already transports each return statement with its exact
   `CanonicalExpression`; no Core-body edit is required.
3. `src/resolve.rs` already emits exact canonical-node and child-position
   reference summaries plus resolver definition and semantic identities. The
   two operands of the target shape already resolve through this path.
4. `src/type_env.rs` already provides parameter declaration rows joined to
   resolver definition IDs. `src/type_check.rs` already constructs internal
   accepted checked declarations, but its public trivial-return checker
   deliberately leaves `a + b` unchecked.
5. `src/core_lower.rs` authenticates the task signature while directly
   constructing the corresponding `CoreLowerItem`. It lowers each bound
   `CanonicalBodyStatement` into its exact `CoreLowerOperation` and currently
   projects the structured add tree with outer type
   `not_type_checked_v0` / `null` / `null`.
6. `src/core_verify.rs` verifies the real in-memory `CoreLowerReport`. Existing
   item, operation, expression, and structured-expression checks are already
   load-bearing and already control the command exit.
7. `src/full_type_check.rs` runs after Core verification. Today its
   `infer_additive_expression_type` independently uses
   `split_once(" + ")` and a name-keyed environment. That branch produces the
   accepted `Int` result for the two supported examples and the accepted `UInt`
   result for established `Identifier + UIntLiteral` returns. It must become
   unreachable for supported, unsupported-target-like, or integrity-failed
   roots, while remaining unchanged for the frozen legacy-compatible shape.
8. `src/main.rs` already routes `core-lower`, `core-verify`, and
   `full-type-check` through those production APIs and derives failure from the
   existing reports. No route edit is required.

The allowed dependency order is therefore:

```text
ast -> core_body/resolve/type_env -> type_check -> core_lower
    -> core_verify -> full_type_check -> main
```

No authorized upstream module imports a downstream module. In particular,
`type_check` must not import Core lowering or verification, and Core lowering
must not call full type. Discovery of such a cycle is an immediate stop.

## Audited corpus and compatibility boundary

A fresh audit covered all 229 `.hum` files under `examples/`, `fixtures/`, and
`experiments/`. The existing structured surfaces identify sixteen additive-
shaped task-return previews in fifteen files and these three distinct child-
shape pairs:

| Parser/preview root | Current occurrences | Current full-type route |
| --- | ---: | --- |
| `Binary(Add, Identifier, Identifier)` | 3 | two accepted `Int` rows through `additive_expression_v0`; the coherent `UInt` row is behind its unrelated existing blocker |
| `Binary(Add, Identifier, UIntLiteral)` | 12 | every row reached by full type is accepted as `UInt` through `additive_expression_v0`; two files are stopped earlier by unrelated callable errors |
| preview-only `Binary(Add, call_candidate_atom, surface_phrase)` around an anonymous-task call | 1 | the semantic root is the outer call and is accepted through `task_call_result_v0`, not `infer_additive_expression_type` |

The preview-only anonymous-call split is not parser-authenticated additive
authority and remains `Noncanonical` for this unit. The exact canonical family
is still the following three task-return roots:

| Source | Authenticated facts | Unit 1 disposition |
| --- | --- | --- |
| `examples/core/minimal_add.hum`, return line 5 | `Int + Int`, declared `Int` | Supported |
| `examples/core/add.hum`, return line 16 | `Int + Int`, declared `Int` | Supported |
| `fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum`, return line 40 | `UInt + UInt`, declared `UInt` | Authenticated out of scope |

The twelve exact `Binary(Add, Identifier, UIntLiteral)` roots are the smallest
authenticated structural compatibility envelope that must keep the existing
full-type additive branch. They occur in:

- `examples/probes/passed_pure_callable.hum`;
- `fixtures/callable/session_al_argument_hws_fail.hum`;
- `fixtures/callable/session_al_lexical_identity_pass.hum`;
- `fixtures/callable/session_al_anonymous_callable_fail.hum`;
- `fixtures/callable/session_al_non_task_value_fail.hum`;
- `fixtures/callable/session_al_nested_application_fail.hum`;
- `fixtures/callable/session_al_fallible_task_fail.hum`;
- `fixtures/callable/session_al_resource_nonparticipant_fail.hum`;
- `fixtures/callable/session_al_cross_file_fail/caller.hum`;
- `fixtures/callable/session_al_shadowed_invalid_receiver_pass.hum`;
- `fixtures/callable/session_al_selected_invalid_receiver_fail.hum`; and
- `fixtures/diagnostics/session_ao_callable_prior_blocker_fail.hum`.

Only this current non-`Identifier + Identifier` additive shape is
`LegacyCompatibleAdditive`. It is recognized from the authenticated parser
root and child kinds before any mutable candidate or legacy inference result is
read. Classification does not decide that its type is `UInt`; it merely permits
the unchanged full-type branch to do exactly what it did before. This is not a
second inference implementation or new type authority.

The envelope is load-bearing in standing evidence. The task
`examples/probes/passed_pure_callable.hum::increment` currently reports
`actual_type = UInt`, `type_source = additive_expression_v0`, and
`accepted_statement_type_v0`; full type and Core verify exit `0`, Core verify is
116 of 116, and preflight requires two executions with stdout exactly `42\n`.
`fixtures/callable/session_al_lexical_identity_pass.hum` likewise keeps full
type and Core verify at exit `0`, with 89 of 89 Core checks. The shadowed-
receiver passing witness and the remaining positive and negative callable
fixtures above retain their existing execution, diagnostic, semantic-surface,
check-count, and exit behavior. Paths are evidence only and never participate
in production classification.

`Binary(Add, UIntLiteral, Identifier)` is the required nonempty
`UnsupportedTargetLike` witness. The parser supports both child kinds, but the
existing legacy rule cannot establish a type for an inline authenticated
`return 1 + value` when `value: UInt`: its left fact is `integer_literal`, its
right fact is `UInt`, the right is not an integer literal, and the types are not
equal. The focused production-path test uses inline source, not a fixture or a
production identity. Other authenticated additive task-return shapes outside
the exact canonical family and the single frozen legacy-compatible shape have
the same fail-closed disposition.

There is no current-corpus integrity failure or unsupported-target-like root.
All other semantic roots are non-targets. Counts and paths are implementation
evidence, never selection keys or runtime invariants.

The `UInt` fixture is a hard byte-compatibility boundary. With the exact
relative path shown above, current direct-process results are:

| Surface | Exit | stdout bytes / SHA-256 | stderr bytes / SHA-256 |
| --- | ---: | --- | --- |
| `core-lower` | 0 | 11,378 / `7c27ce1b320ecf24611a64ff356401cc2f129ae3f8bc8e8870e1efa38c7f69cf` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `core-lower --format json` | 0 | 75,957 / `ae44f177408c059db415bda2b53ac36dd3fbc07c44ac75e059752a5d037d561b` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `core-verify` | 0 | 1,826 / `4ecf43a884856ff090ea4ee1d90d3e3c640295f47320d54bb0c58d136664fe3f` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `core-verify --format json` | 0 | 344,937 / `94549e1a3a314fa497e45ebb63ebb5852e57affd3e4d36f1cd434a7f99ce4ac5` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `full-type-check` | 1 | 15,680 / `13aef5de9315cac1c5f600d80e73e251f6cc8a7e1b4c84d4d5a4f88ea1a6c0c3` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `full-type-check --format json` | 1 | 60,879 / `47bff4d4dcaedb53be8b0d1cf158f93192993d08dfec829b5ebbd33f6c9ca987` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

The implementation must reproduce all six exits and twelve byte/hash values.
The existing unrelated resolver blocker remains the sole reason its full-type
surface is blocked. Minimal `Int` Core verification remains green. The two
named legacy passing programs and representative non-task behavior remain
byte-identical because legacy compatibility and non-target preservation are
closed semantic outcomes, not path-based exemptions.

## Direct per-operation producer contract

`src/type_check.rs` owns the only type producer. Its production entry point is:

```rust
pub(crate) fn canonical_minimal_add_type_for_operation(
    program: &Program,
    diagnostics: &[Diagnostic],
    item: &Item,
    task_signature: Option<&AuthenticatedCanonicalTaskSignature>,
    statement_index: usize,
    statement: &CanonicalBodyStatement,
) -> CanonicalMinimalAddTypeDecision
```

Core lowering calls it exactly once while constructing each bound operation.
It never returns a program-wide collection. The closed private decision is:

```rust
enum CanonicalMinimalAddTypeDecision {
    Supported(CanonicalMinimalAddTypeAuthority),
    AuthenticatedOutOfScope,
    LegacyCompatibleAdditive,
    UnsupportedTargetLike,
    IntegrityFailure,
    Noncanonical,
}
```

The producer follows this precedence:

1. Establish the exact parser-authenticated file/revision, task/item,
   statement, and root identity. A root is not classified from statement text,
   a rendered preview, a candidate field, or a public ID. If parser-owned
   structure identifies an additive task return but its owner/statement/root
   authentication is missing, foreign, or inconsistent, the result is
   `IntegrityFailure`, not a lower-precedence class.
2. Determine whether the semantic parser root is an authenticated additive
   task return. A genuinely unrelated root or context is `Noncanonical`.
3. For exact `Binary(Add, Identifier, Identifier)`, require the current task's
   authenticated signature and its opaque source-revision/item join key.
   Missing or mismatched owner authority is `IntegrityFailure`; an exact
   canonical target cannot fall to a lower-precedence outcome.
4. Join each identifier child by exact parser node identity, child position,
   span, and spelling to exactly one `resolved_v0` reference, then to an exact
   parameter definition in the authenticated task's private semantic scope.
   Two children may reference the same parameter, but duplicate or ambiguous
   rows are forbidden.
5. Determine the operand annotations first from the authenticated task
   signature. This is the anti-downgrade boundary. An authenticated `Int`
   operand cannot become out of scope because a later resolver, declaration,
   claim, or public field says `UInt`.
6. Join each selected definition to exactly one locally complete type-environment
   parameter declaration and, for the supported `Int` path, exactly one
   `accepted_declaration_annotation_v0` checked declaration with accepted
   `Int` references. Missing, duplicate, rejected, foreign, or inconsistent
   relationships are `IntegrityFailure`.
7. If both authenticated operand annotations and checked declaration facts are
   exactly `Int`, produce one private authority whose expression result is
   independently `Int`. The declared result is retained only for later
   compatibility.
8. A coherent authenticated non-`Int` exact canonical target is
   `AuthenticatedOutOfScope` only when the signature, resolver relationships,
   and local type-environment declarations are complete and mutually exact.
   The globally blocked checked rows in the established `UInt` fixture do not
   become type authority. Unknown or incomplete facts are integrity failures.
9. For another authenticated additive task-return root, exact
   `Binary(Add, Identifier, UIntLiteral)` is
   `LegacyCompatibleAdditive`. This decision reads only authenticated parser
   structure; it does not invoke `infer_additive_expression_type`, inspect a
   produced type, or create authority.
10. Every other authenticated additive task-return root is
    `UnsupportedTargetLike`. It receives no authority and cannot reach the old
    additive/name/text fallback. The inline
    `Binary(Add, UIntLiteral, Identifier)` witness makes this class nonempty.

For the canonical family, the semantic joins in steps 4-8 remain mandatory:
missing, ambiguous, foreign, substituted, inconsistent, or rejected facts are
always `IntegrityFailure`, never a downgrade to a legacy, unsupported, or
non-target class.

Production selection must not use a path, task/parameter spelling, public row
ID, vector position, ordinal alone, statement text, declared result, serialized
JSON, current corpus count, or a candidate field.

## Opaque owner identity and untouched authority

`src/ast.rs` adds one private, nonserializing
`CanonicalTaskSignatureJoinKey`. Only an already authenticated
`AuthenticatedCanonicalTaskSignature` may derive it. It contains the opaque
facts needed to distinguish source revision, normalized semantic file, file
index, item path, task identity, and authenticated header/signature owner. Its
fields are private; downstream modules receive no raw constructor or public
conversion. The authenticated handle can compare a key with its untouched
snapshot without revealing or reparsing source bytes.

The join key and `CanonicalMinimalAddTypeAuthority` must not implement or derive
`Clone`, `Copy`, `Default`, `Serialize`, or `Deserialize`. Debug output must not
reveal source bytes or private authority. An authority contains the join key,
the exact item/statement/root/child parser identities and ranges, the selected
resolver reference and definition identities, the selected declaration
identities and accepted types, and the independently produced `Int` result.

Each `CoreLowerOperation` owns exactly one closed private state as an ordinary
field inside its already corresponding `CoreLowerItem`:

```rust
enum CoreLowerCanonicalMinimalAddType {
    Noncanonical,
    AuthenticatedOutOfScope,
    LegacyCompatibleAdditive,
    UnsupportedTargetLike,
    IntegrityFailure,
    Supported {
        authority: CanonicalMinimalAddTypeAuthority,
        claim: CanonicalMinimalAddTypeClaim,
    },
}
```

There is no classification vector, side table, retained ordinal, filtered
count, or later pairing step. The state is attached while the exact bound
statement becomes the exact operation. A second state cannot be attached to
one operation. `AuthenticatedOutOfScope`, `LegacyCompatibleAdditive`, and
`Noncanonical` carry no authority, claim, or verified view.

Whole-operation deletion is not delegated to iteration over the remaining
candidate operations. The expected direct-operation identity is borrowed
independently from the accepted parser/task/body facts: opaque file/revision and
task/item owner identity, exact statement identity, exact parser-root identity,
and the corresponding operation identity derived for that bound statement.
The lowered candidate retains its independently produced comparison identity.
Verification compares those two facts directly; it does not retain a second
program-wide record, global vector, or cardinality equation.

Concretely, `src/core_verify.rs` constructs one stack-local, borrow-only
`ExpectedCanonicalMinimalAddOperation<'program>` while walking the exact
authenticated `Program` item and `CanonicalBodyStatement`, before consulting
the lower candidate. It is not `Clone`, `Copy`, serializable, or stored as a
collection. `src/core_lower.rs` attaches the corresponding private
`CanonicalMinimalAddOperationIdentity` when it constructs that exact operation.
Both use the same pure identity encoding over the accepted opaque task join key,
statement identity, parser root, and existing operation identity; neither reads
candidate text or produces a type conclusion. Removing a candidate operation
therefore cannot remove or alter the Program-owned expected fact.

The per-item lookup is a one-pass closed state machine (`NoMatch`, `One`, or
`Ambiguous`), not a numeric filtered count or first-match selection. A public-ID
collision with another private revision or owner is `Foreign`, not a match.
Zero, `Ambiguous`, `Foreign`, deletion of the sole or final expected operation,
and an additive-disposition candidate with no expected additive identity are
all `IntegrityFailure`. Reordering
is rejected by the full statement/root/operation identity even when a visible
ordinal or ID happens to agree. A genuine non-additive parser/body statement is
the only path to `Noncanonical`/consumer `NonTarget`.

Core lowering creates the non-authoritative claim by projecting the supported
authority; it does not infer `Int`. Test-only corruption may alter the public
projection or claim before production verification, separately or coherently,
but cannot construct, replace, mutate, or extract the authority or a verified
view.

## Core verification and verified access

Before iterating a `CoreLowerItem`'s candidate operations, Core verification
walks that exact item's accepted parser/body statements and performs the direct
lookup above for every authenticated additive task-return root. Verification
therefore runs even when the expected candidate operation is absent. An honest
exactly-one lookup emits no extra row, preserving every valid out-of-scope and
legacy-compatible byte. Any `NoMatch`, `Ambiguous`, `Foreign`, deleted, or
same-visible-ID result emits one failure-only row at the item's operation-check
boundary before ordinary candidate-operation rows:

- scope: `core_item`;
- scope ID: the existing corresponding Core item ID;
- source span: the expected statement span;
- rule: `canonical_minimal_add_direct_operation_identity_unique`;
- detail: `one Core operation matches the parser-owned additive task-return identity`;
- status: `failed_v0`.

That row participates in ordinary item/root failure propagation and command
exit. It is not a pass-row protocol or a public authority record. Candidate
iteration may emit its normal additional structural failures, but cannot erase
the missing-operation failure.

For present `Supported` and canonical-family `IntegrityFailure` operations, the
existing `structured_expression_outer_type_unchecked` row is replaced at that
same location by exactly four `structured_expression` rows:

1. `canonical_minimal_add_type_state_consistent`
2. `canonical_minimal_add_public_projection_matches_authority`
3. `canonical_minimal_add_private_claim_matches_authority`
4. `canonical_minimal_add_verified_view_issued`

All four rows use `scope = structured_expression`, the existing operation ID as
`scope_id`, the existing operation span as `source_span`, and ordinary
sequential `core-verify-check-N` IDs. They occupy the old outer-type row's
position in exactly the order above and use these exact details:

1. `canonical minimal-add type state is complete for its closed disposition`;
2. `canonical minimal-add public projection matches untouched producer authority`;
3. `canonical minimal-add private claim matches untouched producer authority`;
4. `verified canonical minimal-add type access is gated by every required check`.

The first three compare the direct operation state, candidate, and claim with
the untouched authority and the already retained parser/task/structured facts.
For an honest `Supported` operation all four pass. For `IntegrityFailure`, the
first row passes only when the unavailable status and both nulls are atomic;
the authority, claim, and view rows fail. The fourth row also fails after any
required item, operation, expression, structured-expression, type-state,
public-candidate, or private-claim failure for that exact item. No view is
constructed until the complete report exists, so a failure before or after the
type rows cannot leave a retained view. Any failed row uses `failed_v0`,
propagates through the existing item/root summary, and makes the command exit
nonzero. No check reruns the type producer, reconstructs authority from
candidate data, or trusts JSON.

For a present `UnsupportedTargetLike`, the existing operation-expression
`type_claim_honesty` row is replaced at the same ordinal by one failed row:

- scope: `operation_expression`;
- scope ID and source span: the existing operation ID and span;
- rule: `canonical_minimal_add_unsupported_target_like_rejected`;
- detail: `unsupported additive task-return shape has no canonical type authority`;
- status: `failed_v0`.

It adds no row, produces no authority or claim, issues no view, and makes the
existing report and CLI fail through ordinary propagation. The exact inline
`UIntLiteral + Identifier` production-path probe must reach this row before
full type can attempt legacy inference.

`AuthenticatedOutOfScope` retains the existing
`structured_expression_outer_type_unchecked` row. `LegacyCompatibleAdditive`
and `Noncanonical` retain exactly whichever existing operation, expression, and
structured rows their pre-unit structures emitted; no structured row is
invented for the current `Identifier + UIntLiteral` shape. The `UInt` fixture
therefore retains 766 of 766 passing checks, exit 0, and its frozen bytes. The
legacy witnesses likewise retain their exact Core-verifier rows, counts,
human/JSON bytes, and exits. Supported rows add three net check rows; later
aggregate check IDs may shift only because those earlier supported rows
genuinely add evidence.

`src/core_verify.rs` adds a one-lifetime private
`VerifiedCanonicalMinimalAddType<'verified>`, a closed borrowed outcome, and a
read-only report session over the real, complete `CoreVerifyReport`:

```rust
enum CanonicalMinimalAddVerificationOutcome<'verified> {
    Supported(VerifiedCanonicalMinimalAddType<'verified>),
    AuthenticatedOutOfScope,
    LegacyCompatibleAdditive,
    UnsupportedTargetLike,
    IntegrityFailure,
    NonTarget,
}
```

The only full-type handoff is the following two-level total borrowed operation:

```rust
pub(crate) fn with_canonical_minimal_add_verification<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'report> FnOnce(CanonicalMinimalAddVerification<'report>) -> R,
) -> R

pub(crate) fn with_canonical_minimal_add_operation<'report, 'call, R>(
    verification: &'call CanonicalMinimalAddVerification<'report>,
    item: &Item,
    statement_index: usize,
    statement: &CanonicalBodyStatement,
    consume: impl FnOnce(
        CanonicalMinimalAddVerificationOutcome<'call>,
    ) -> R,
) -> R
where
    'report: 'call,
```

The outer function builds Core lowering and the complete verification report
exactly once, then invokes its callback exactly once whether the report has zero
or many failures. The report session exposes no owned report, authority, claim,
or view. For each requested direct parser/body operation,
`with_canonical_minimal_add_operation`
performs the closed lookup without rerunning lowering, verification, or the type
producer and invokes its callback exactly once with one outcome. Every branch
therefore produces generic `R`; no panic, fabricated default, second accessor,
or unstated error channel exists.

Only `Supported` contains a view, and only when the operation exists uniquely
and every required item, operation, expression, structured-expression,
type-state, public-candidate, private-claim, authority, direct-lookup, and view-
gating check for that exact target passed. A failed relevant check before or
after the four type rows yields `IntegrityFailure` and no view. An honest
unsupported row yields `UnsupportedTargetLike`; honest legacy and non-`Int`
rows retain their distinct outcomes without a view; unrelated parser/body
structure yields `NonTarget`.

View issuance is target-local. An unrelated report failure does not rewrite a
fully passing target-local outcome or fabricate an integrity failure. Full type
still applies its existing report-wide source/resolver/type/Core-verifier
blocker precedence before accepting any statement, so an unrelated failure
continues to produce the existing prior-error rows even if another operation's
borrowed outcome is locally `Supported`.

This boundary avoids an owned cross-report authority batch and prevents full
type from retaining a view after the report and its operation-owned authority
are dropped. The view owns no authority, implements no cloning, serialization,
default, or owned/`'static` conversion, and exposes only exact target identity,
actual type `Int`, and provenance needed by full type. The cfg-selected
compile-fail proof uses the actual outer session, per-operation outcome, and
view to prove artifact, report/authority, and owned/`'static` escape failures;
ordinary builds remain warning-clean.

## Full-type consumer and fallback partition

`src/full_type_check.rs::build_report` is the sole downstream consumer. It
constructs the report once inside
`with_canonical_minimal_add_verification`, requests each exact statement through
`with_canonical_minimal_add_operation`, and constructs the affected statement
row inside that operation's one total callback. It uses the existing canonical
body report for exact target identity and behaves as follows:

- `Supported`: actual type comes only from the verified view, with provenance
  `verified_canonical_minimal_add_type_v0`; the old additive inference branch
  is unreachable.
- `AuthenticatedOutOfScope`: no view or new semantic conclusion exists; the
  exact pre-unit full-type branch remains reachable and its output is preserved.
- `LegacyCompatibleAdditive`: no view or new semantic conclusion exists; exact
  `Binary(Add, Identifier, UIntLiteral)` statements alone may reach the
  unchanged `infer_additive_expression_type` branch and retain
  `additive_expression_v0` output.
- `UnsupportedTargetLike`: no view is available; its failed Core rule makes the
  existing report-wide blocker precedence emit the existing prior-error row.
  The old additive/name/text fallback is unreachable.
- `IntegrityFailure`, including missing/deleted/ambiguous direct operation or
  any failed required target check: no view is available, the existing
  prior-error row is emitted, and the old fallback is unreachable.
- consumer `NonTarget`: exact previous behavior remains, including any existing
  non-additive inference route.

For supported operands and an incompatible authenticated declared result, the
verified actual type remains `Int` and the existing
`rejected_statement_type_mismatch_v0` / `statement_expression_type_mismatch`
outcome is retained. The expected result is never reversed into expression
inference.

After Unit 1, `infer_additive_expression_type`, `split_once(" + ")`,
`place_type_fact`, `task_returns.get`, recursive name inference, and declared
result inference remain reachable only for `AuthenticatedOutOfScope`, the exact
`LegacyCompatibleAdditive` structure, and genuinely unrelated existing
`NonTarget` routes. They are unreachable for `Supported`,
`UnsupportedTargetLike`, and `IntegrityFailure`. Source audits supplement real
production-path behavioral tests for this partition.

## Closed outcome matrix

| Input | Core lower | Core verify | Full type | Legacy additive fallback |
| --- | --- | --- | --- | --- |
| Supported `Int + Int`, compatible result | typed candidate and private claim/authority | four replacement rows pass; one view | accepted `Int` from verified provenance | unreachable |
| Supported `Int + Int`, incompatible declared result | same independently typed candidate | same pass/view | existing mismatch with actual `Int` | unreachable |
| Recognized target with missing/foreign signature authority | unavailable candidate; no partial type | authority/claim/view rows fail | existing prior-error row | unreachable |
| Missing, duplicate, ambiguous, foreign, or wrong-kind resolver/declaration fact | unavailable candidate | same fail/no-view outcome | existing prior-error row | unreachable |
| Authenticated `Int` target with apparent `UInt` substitution | integrity failure, never out of scope | affected comparisons and view fail | existing prior-error row | unreachable |
| Public-only, claim-only, or coherent public/claim substitution | corrupted candidate may remain syntactically typed | comparison/view rows fail against untouched authority | existing prior-error row | unreachable |
| Valid authority plus structural corruption | candidate may remain typed | existing structural row and view row fail | existing prior-error row | unreachable |
| Expected additive operation deleted, duplicated, foreign, ambiguous, or same-visible-ID substituted | no trustworthy direct candidate | failure-only direct-operation row; no view | existing prior-error row | unreachable |
| Coherent authenticated non-`Int` target | exact prior lower bytes | exact prior checks and exit | exact prior behavior | reachable exactly as before |
| Exact `Identifier + UIntLiteral` legacy-compatible return | exact prior lower bytes; no new claim | exact prior checks/count/exit | exact accepted or existing blocked behavior with `additive_expression_v0` where reached | reachable exactly as before |
| Authenticated additive root outside canonical and legacy envelopes, including `UIntLiteral + Identifier` | unavailable outer type; no claim | replacement expression rule fails; no view | existing prior-error row | unreachable |
| Noncanonical or unrelated context | exact prior behavior | exact prior behavior | exact prior behavior | reachable only as it was before |

## Exact public contract

No top-level object, command, schema family, diagnostic code, or public identity
is added. Private join keys, resolver/declaration identities, authorities,
claims, accessors, and views never serialize.

For a supported operation, existing `hum.core_lower.v0`
`items[].operations[].expression` fields become:

- `expression.type_status`: `checked_canonical_minimal_add_v0`;
- `expression.type_text`: `"Int"`;
- `expression.type_source`: `canonical_minimal_add_type_authority_v0`.

For an integrity failure or `UnsupportedTargetLike` operation those same three
fields become:

- `expression.type_status`: `canonical_minimal_add_type_unavailable_v0`;
- `expression.type_text`: `null`;
- `expression.type_source`: `null`.

No child field is added. The structured parser projection and human Core-lower
rendering remain unchanged. Out-of-scope, legacy-compatible, and non-target
JSON remain exact prior bytes.

`hum.core_verify.v0` changes only through the four exact conditional
`checks[]` rows, the unsupported expression-row replacement, and the
failure-only missing/ambiguous direct-operation row above. Their literals,
status, scope/ID/span shape, placement, count propagation, and overall failure
behavior are frozen by this document. Honest out-of-scope and legacy-compatible
operations emit neither new nor replacement rows and retain exact prior bytes.
No private identity appears in a detail or any other serialized field.

For a supported `hum.full_type_check.v0` `items[].statements[]` row, existing
fields retain their current shape and ordering. `actual_type` is `"Int"`,
`type_source` is `verified_canonical_minimal_add_type_v0`, compatible status
remains `accepted_statement_type_v0`, and mismatch status/reason remain the
existing values. Integrity failure uses the existing
`not_checked_blocked_by_prior_errors_v0` row with null expression/expected/
actual/source fields and reason `source_resolver_type_or_core_verify_errors`.
`UnsupportedTargetLike` uses that same prior-error row because its required Core
rule fails. `LegacyCompatibleAdditive`, `AuthenticatedOutOfScope`, and
`NonTarget` preserve their exact existing fields, order, literals, and
provenance; in particular accepted `Identifier + UIntLiteral` remains
`actual_type = "UInt"`, `type_source = additive_expression_v0`, and
`accepted_statement_type_v0` where no unrelated earlier blocker applies.

`hum.type_check.v0`, Core preview, IR readiness, capabilities, version,
execution readiness, and every downstream effect, ownership, resource, profile,
IR, backend, and runtime surface remain unchanged.

## Exact nine-path implementation envelope

Unit 1 may modify exactly these paths:

1. `src/ast.rs` - add the opaque authenticated task-signature join key and
   comparison-only access from the accepted signature handle needed to bind the
   independently borrowed parser/body expected-operation identity.
2. `src/type_check.rs` - own the sole per-operation producer, local joins,
   six-way closed decision, legacy structural partition, and untouched
   supported authority.
3. `src/core_expr.rs` - own the two new Core expression type-status constants
   and canonical provenance constant.
4. `src/core_lower.rs` - call the producer in the bound-statement route, own one
   direct state and comparison identity per operation, create the claim, and
   project exact fields.
5. `src/core_verify.rs` - verify direct authority/candidate/claim relationships,
   look up parser/body-expected operations even when absent, gate view issuance
   on all required target checks, and provide the total report-bound outcome.
6. `src/full_type_check.rs` - consume the verified view and partition the old
   fallback across all six outcomes without creating another inference path.
7. `docs/HUM_CORE_LOWER_SCHEMA.md` - document exact conditional outer fields.
8. `docs/HUM_CORE_VERIFY_SCHEMA.md` - document exact conditional checks and
   private authority boundary.
9. `docs/HUM_FULL_TYPE_CHECK_SCHEMA.md` - document verified provenance,
   mismatch, prior-error, and preservation outcomes.

Every path is load-bearing. Removing `src/ast.rs` loses revision-safe joining;
removing the producer loses canonical type authority; removing lower or verify
leaves unused metadata or an unchecked candidate; removing full type leaves a
parallel same-shape inference; removing a schema makes a changed public V0
contract undocumented.

Explicitly excluded and frozen are `src/parser.rs`, `src/core_body.rs`,
`src/resolve.rs`, `src/type_env.rs`, `src/core_preview.rs`, `src/main.rs`,
`src/ir_readiness.rs`, `Cargo.toml`, tools, fixtures, examples, snapshots,
Work Orders, governance, decisions, architecture doctrine, and every downstream
semantic or execution pass. A tenth path is an immediate stop.

Expected implementation size is approximately 1,200-2,000 insertions and fewer
than 200 deletions across these nine paths, including focused tests and schema
text. The direct operation-owned state, four verification rows, one borrowed
report session with a total per-operation callback, one failure-only lookup row,
one unsupported-row replacement, and four selectors should be traceable in one
focused review sitting of roughly three to four hours. Substantially larger
machinery is scope pressure, not permission to recreate Work Order 13.

## Permanent focused evidence

These exact selectors are required:

1. `type_check::tests::canonical_minimal_add_type_authority_is_direct_and_bound`
2. `core_lower::tests::canonical_minimal_add_type_authority_is_owned_by_exact_operation`
3. `core_verify::tests::canonical_minimal_add_type_verification_withholds_invalid_view`
4. `full_type_check::tests::minimal_add_consumes_only_verified_canonical_type`

The existing exact-selector helper must list exactly one nonzero test, execute
exactly one test, pass, and award exactly one unique credit for each selector.
Tests must use real production producers and consumers. Hard-coded production
paths may only load named regression inputs; production selection may not use
them.

The four tests and module-local probes must establish:

- both supported corpus targets, the coherent out-of-scope `UInt` target, all
  twelve legacy-compatible corpus roots, and zero current unsupported or
  integrity-failure roots;
- exact preservation of `passed_pure_callable` and
  `session_al_lexical_identity_pass`, including full-type/Core-verifier exits,
  check counts, provenance, and required execution behavior;
- inline authenticated `UIntLiteral + Identifier` reaches
  `UnsupportedTargetLike`, fails the production verifier, receives no view or
  inferred type, and cannot reach the old fallback;
- genuinely non-additive `Noncanonical`/consumer `NonTarget` behavior remains
  distinct and unchanged;
- exact owner/revision/item/statement/root and ordered child binding;
- same-visible-ID foreign revision and foreign item rejection;
- missing, duplicate, ambiguous, extra, reordered, and foreign resolver,
  definition, declaration, operation-state, and authority relationships;
- same-spelled foreign binder and equal-typed foreign definition rejection;
- authenticated-`Int` to apparent-`UInt` anti-downgrade;
- statement-text sabotage independence;
- public-only, claim-only, and coherent public/claim co-substitution against
  untouched authority;
- structural failure with otherwise valid type facts withholding every view;
- the total report callback executes after both successful and failed reports;
  its operation callback runs once and returns generic `R` for each of
  `Supported`, `AuthenticatedOutOfScope`, `LegacyCompatibleAdditive`,
  `UnsupportedTargetLike`, `IntegrityFailure`, and `NonTarget`;
- only `Supported` contains one borrowed view; every other outcome contains
  zero views, and unrelated report failure obeys the frozen target-local view /
  report-global full-type precedence;
- deletion of the sole expected operation, deletion of the final expected
  operation among unrelated operations, duplicate/ambiguous candidates,
  foreign candidates, and same-visible-ID candidates from another revision all
  emit the direct-operation failure and produce `IntegrityFailure`;
- compatible and incompatible declared-result behavior;
- anti-downgrade from canonical `Int` to every lower-precedence disposition;
- fallback preserved only for exact legacy-compatible, authenticated
  out-of-scope, and existing unrelated routes, and suppressed for supported,
  unsupported-target-like, and integrity-failed operations;
- out-of-scope, legacy-compatible, and non-target preservation; and
- checked range/index arithmetic with no panic or wrap.

Duplicate or extra authority must be impossible in the production type shape,
not accepted and later deduplicated. Test seams may move or replace a direct
state before verification, but may not expose a constructor that production or
downstream code can use.

Because the verified view borrows the checked report, the actual production
types require a cfg-selected compile-fail proof under
`hum_compile_fail_verified_canonical_minimal_add_direct_escape`. A normal
`cargo check --all-targets` must pass before and after. The expected-failure run
must reach and name
`verified_canonical_minimal_add_artifact_escape_must_not_compile`,
`verified_canonical_minimal_add_report_escape_must_not_compile`, and
`verified_canonical_minimal_add_static_escape_must_not_compile`. All three use
the real production report session, closed operation outcome, and view and fail
for the intended report/view lifetime widening, with no privacy,
unresolved-import, missing-type,
unexpected-cfg, or unrelated failure. The command restores `RUSTFLAGS`
afterward. No toy type, generated source, dependency, fixture, Cargo edit, or
tenth path earns credit.

## Proportional implementation evidence

An authorized implementation session must, in order:

1. run the four focused selectors individually;
2. run the production compile-fail lifetime proof with normal checks before and
   after;
3. exercise both supported targets, the `UInt` out-of-scope target, the two
   named legacy-compatible passing targets, an inline unsupported-target-like
   target, and a representative non-target through Core-lower, Core-verify, and
   full-type human and JSON surfaces;
4. run changed JSON surfaces twice and require byte-identical output;
5. reproduce the structured 229-file audit with zero failures, three exact
   canonical `Identifier + Identifier` roots, twelve exact legacy-compatible
   `Identifier + UIntLiteral` roots, no current unsupported/integrity root, and
   the preview-only anonymous-call split classified by its semantic call root,
   while treating every count only as evidence;
6. reproduce the frozen `UInt` exits, byte counts, hashes, 766/766 Core checks,
   and unrelated full-type blocker;
7. confirm minimal `Int` Core verification stays green, the two supported
   full-type rows use verified provenance, both named legacy passing programs
   retain their output and execution contracts, and the inline unsupported
   witness fails before fallback; and
8. run `cargo fmt --all -- --check`, `cargo check --all-targets`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
   `git diff --check`, text hygiene, public readiness, alpha claims, and release
   readiness.

Run the root Rust suite once on the final exact candidate. Do not run local
Fast, `tools/check_all.ps1`, full preflight, Exhaustive, performance pairs,
validation ledgers, actor transcripts, or publication CI. Required
post-publication CI later owns the full lane independently on Ubuntu and
Windows.

## Explicit bans and stop conditions

This order forbids:

- a global classification vector, global cardinality equation, positional
  association, filtered count, retained classification ordinal, or corpus-wide
  synchronization mechanism;
- association by file spelling, task/parameter name, public ID, vector index,
  statement text, declared result, JSON, or corpus count;
- independent type inference in Core lowering or verification;
- a second supported-shape inference in full type or type check;
- public/private self-consistency without untouched authority;
- copied, serialized, cloneable, default, public, or caller-supplied authority;
- a view issued after any required item, operation, expression, structured,
  candidate, claim, or authority check fails;
- accepting missing, duplicate, extra, reordered, or foreign authority;
- letting a canonical or directly associated integrity failure downgrade to
  out of scope, legacy-compatible, unsupported-target-like, or non-target, or
  reach text/name/result fallback;
- classifying legacy compatibility from a mutable candidate, produced type,
  successful inference, filename, fixture identity, or corpus membership;
- allowing any additive root outside exact
  `Binary(Add, Identifier, UIntLiteral)` legacy compatibility to reach the old
  additive fallback unless it is the separately authenticated exact
  `Identifier + Identifier` out-of-scope family;
- treating no-view as sufficient evidence of out-of-scope/non-target, offering
  the generic callback only after report success, rerunning verification for a
  callback result, or fabricating a default branch;
- relying on ordinary candidate iteration to observe whole-operation deletion,
  or matching an expected operation by first hit, count, visible ID, or ordinal;
- general numeric/operator typing, coercion, overloads, new syntax, diagnostics,
  commands, schemas, fixtures, tools, caches, ledgers, manifests, or validation
  frameworks; and
- Hum IR, IR readiness, backend, runtime, execution, effects, ownership,
  resources, profiles, optimization, performance, or safety claims.

Stop without workaround if a tenth path is required, the join key cannot be
derived from accepted Work Order 14 authority, the direct operation state needs
a batch, a dependency cycle appears, `UInt` bytes drift, the view can escape,
the legacy-compatible callable bytes or required executions drift, the old
fallback remains reachable for a supported/unsupported/integrity target, a
whole-operation deletion is not observed, the callback is not total, a
selector does not select exactly one test, a public contract cannot be stated
exactly, or the unit ceases to be review-sized. Do not invent an adapter or
recover archived code.

## Review, correction, commit, and publication gates

The first independent pre-issuance review returned
`ACCEPT WITH REQUIRED FIX` with exactly three P1 findings: target-like fallback
partition, total failed-report callback behavior, and whole-operation deletion.
No P0 or P2 finding was reported. The initial correction instruction was found
internally contradictory before any edit because it would have rejected
established `Identifier + UIntLiteral` programs. The BDFL superseded that
instruction with the six-way disposition in this document. This edit consumes
the sole bounded correction cycle.

A fresh independent corrected-document architect-reviewer who did not author,
edit, or perform the first review must inspect repository ground truth, the
exact two-document transition, the nine-path satisfiability proof, the complete
legacy compatibility audit, all six outcomes, direct ownership and deletion
lookup, the total callback and lifetime boundary, public contracts, selectors,
and stop rules. The reviewer returns `ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or
`REJECT` with P0/P1/P2 findings. Only unqualified `ACCEPT` advances. Any other
verdict stops Work Order 15 at the BDFL and authorizes no edit, workaround,
second correction, implementation, or later review cycle.

Independent acceptance authorizes no edit, implementation, stage, commit, push,
archive mutation, or remote mutation. The BDFL must separately accept exact
document bytes, authorize a local documentation commit, authorize publication,
observe full-lane terminal-green CI, authorize a status record and its
publication, and only then issue a separate Unit 1 implementation signal.

Publication of the Work Order 14 closeout and Work Order 15 activation
transition must use `mode=full`, `reason=no_status_transition`; it is not a
status-only transition. Implementation remains unauthorized until the later
explicit signal even if document CI is green.

## Current authorization gate

Work Order 15 is terminally rejected, archived, closed, inactive, and frozen.
The sole implementation correction was consumed before the terminal verdict.
No further Work Order 15 implementation, correction, review, commit,
publication, status update, repair, workaround, archive mutation, reuse,
cherry-pick, merge, patch application, or later work is authorized.

The archive branch is failure evidence only. Its existence authorizes no
source recovery and makes no rejected implementation fact authoritative.
Checked minimal-add type authority did not ship and remains deferred.

The only proposed next action is independent pre-issuance review of the fresh
Work Order 16 document package under its own authorization gates. This closeout
does not accept that draft, authorize its documentation commit or publication,
or authorize Work Order 16 Unit 1.

Historical publication evidence follows. The Work Order 15 two-document
package received final independent `ACCEPT` with no P0, P1, or P2 findings and
was committed as
`6d859113ccd6a4a9f3af4ab4f2d38d972ae1f28e` with subject
`docs(workorder): define direct minimal add type authority`, scope
`WORKORDER_14.md` and `WORKORDER_15.md`, and statistics `+928/-9`. Publication
workflow `30781480361` attempt `1` concluded `success`: Ubuntu job
`91586859299` and Windows job `91586859347` both succeeded with `mode=full`,
`reason=no_status_transition`; Ubuntu Exhaustive passed all `14,226` pairs.

The unique production status-boundary anchor was restored and published in
commit `3b2edb116253364efb0191ca6442f85276ae87d5`, parent
`6d859113ccd6a4a9f3af4ab4f2d38d972ae1f28e`, subject
`fix(workorder): restore status boundary anchor`, with sole path
`WORKORDER_15.md` and statistics `+1/-1`. The active Work Order is mechanically
projectable by the production status classifier.

The final BDFL foundation package was independently accepted with verdict
`ACCEPT — GO` and no P0, P1, or P2 findings, then published by normal
fast-forward to `main` only as
`aa3fd134c18115d2e1d786f9412313b65a609333`, parent
`3b2edb116253364efb0191ca6442f85276ae87d5`, tree
`3c7fc6c5b7d618f6dafe360a5dc18b9006130148`, subject
`docs(decisions): adopt termination measures and algorithmic foundations
charter`, exactly nine paths, statistics `+4,113/-1`. Foundation research is
closed with GO.

Foundation workflow `ci` run `30793820652` attempt `1` tested exact SHA
`aa3fd134c18115d2e1d786f9412313b65a609333` and concluded `success` with
`mode=full`, `reason=no_status_transition`. Ubuntu job `91622829209` succeeded
in `25m24s`; full preflight succeeded in `24m46s`, Exhaustive succeeded, and
status-only evidence was skipped. Windows job `91622829261` succeeded in
`37m20s`; full preflight succeeded in `36m41s`, the platform-independent
Exhaustive duplicate was correctly skipped, and status-only evidence was
skipped. Cargo caching and Rust-toolchain preparation succeeded on both
platforms.

Those publication facts do not survive as implementation authority after the
terminal rejection recorded above.
<!-- workorder-current-authorization-gate:end -->
