# Hum Work Order 17: Verify Checked Type Authority On Exact Core Operations

Date: 2026-08-09
Status: Closed. Work Order 17 Unit 1 is accepted, implemented, published, and
terminal-green. The accepted documentation package is commit
`5aca6eeab4ac427142e420f67e13d9a00ee9c706`; its publication status is commit
`1b35b43a07c536834e25564c473c0ce40cbe0e3c`. The accepted implementation is
commit `212e692f7db40b428e7857917dc04147b6776452` with subject
`feat(core): verify checked type authority on exact operations`, exactly nine
paths, and statistics `+1,857/-86`.

Unit 1 received one rejected initial implementation review, one bounded
correction, terminal confirmation that all findings were closed, and an
evidence-only `ACCEPT` after native Cargo stderr capture prevented two earlier
reviewers from completing Fast. The evidence-only Fast run exited 0 in
796.422 seconds with the root suite at 459/459, exact selectors at 95/95, and
all four isolated Work Order 17 selectors passing.

Required publication `ci` workflow `31424834716`, attempt 1, tested exact SHA
`212e692f7db40b428e7857917dc04147b6776452` and concluded success. Both
platforms selected `mode=full` with `reason=no_status_transition`, passed the
95/95 selector inventory and all four Work Order 17 selectors, passed task-
signature evidence, the sole-producer E0624 proof, the lifetime 0/101/0 proof
without E0382, F4/source inventories, text hygiene and public readiness for
525 files, alpha claims, and release readiness for version `0.0.1`.

Ubuntu job `93573945728` succeeded in 1,081 seconds with a 1,047-second full
preflight and the platform-filtered root suite at 444/444. Its Exhaustive
producer selected `parser::tests::exhaustive_canonical_seal_pair_matrix_is_complete_and_nonzero`
once and passed once with zero failures: F1 630, F2 4,950, F3/F4 8,646,
14,226 total pairs, 16.169 seconds, seed `0x48554D5F5345414C`. Windows job
`93573945837` succeeded in 1,577 seconds with a 1,543-second full preflight and
the root suite at 459/459; it correctly skipped only the duplicate Exhaustive
producer.

This closed record is frozen. A separately authorized successor-planning
transition installs `WORKORDER_18.md` as the sole active Work Order; it does
not reopen Work Order 17 or authorize successor implementation.

Owner: BDFL (Ocean).
Author: Work Order 17 architect-author. The author is permanently disqualified
from independently reviewing these document bytes or any implementation
candidate produced under them.
Planning baseline: `HEAD`, local `main`, cached `origin/main`, and live remote
`main` are all `e7af4ab0f3590c99a18db67b293292212056c9db`.

## Closed predecessor and planning authority

Work Order 16 is accepted, published, closed, and terminal-green. Its final
closeout commit is `e7af4ab0f3590c99a18db67b293292212056c9db` and its final fast-lane
workflow is `31336439272`. Work Order 16 establishes these accepted facts:

- a parser-authenticated `Program` owns the expected item traversal;
- each Core item is associated with exactly one Program-owned source item;
- `with_expected_core_operations_for_item` streams each exact source operation
  in source order through a higher-ranked callback;
- `ExpectedCoreOperation` cannot escape, become static, or be collected;
- each `CoreLowerOperation` owns a private `CoreOperationCandidateOrigin`;
- `core_operation_occupies_expected_slot` compares that private origin with the
  exact borrowed expectation inside the verifier callback;
- missing, duplicate, extra, foreign, reordered, or substituted item and
  operation candidates fail closed; and
- the compiler-enforced Core-body construction boundary prevents unvalidated
  first lineage issuance.

Those facts are immutable prerequisites. This Work Order neither reopens nor
reimplements them.

Work Orders 13 and 15 are terminally rejected, archived, closed, and
non-authoritative. Their global classification batch, later direct-authority
attempt, implementation code, tests, and schemas are failure evidence only.
Nothing in either archive may be copied, recovered, cherry-picked, treated as
accepted design, or used as production authority.

## Replacement planning history

Work Order 17 has not been issued or implemented. Two prior planning packages
were terminally rejected and are non-authoritative:

1. The initial draft misstated the current full-type baseline and specified a
   one-pass Core-verifier callback that could not preserve the existing
   readiness summary and diagnostic occurrences from the same report.
2. The terminal draft corrected those defects but required
   `structured_expression_outer_type_unchecked` to be replaced for every
   `IntegrityFailure` and `UnsupportedTargetLike` disposition. That row does
   not exist when the expression has no structured projection.

The independent terminal topology ruling established that `(None, None)` for
structured candidate and retained structured authority is intentional:

- Work Order 16 authenticates Program-owned item and operation identity and
  source order, not arbitrary expression structure;
- `structured_minimal_add_expression` remains intentionally limited to
  `Binary(Add, Identifier, Identifier)`;
- literal-containing and other unstructured additive forms may validly carry
  no structured projection and no retained structured authority; and
- authentication of an operation slot does not imply structured-expression
  support for the expression occupying it.

This fresh replacement preserves the independently validated producer,
direct-operation ownership, six-way disposition, and one-pass-per-report
full-type handoff designs. It replaces every ghost-row requirement with an
explicit projection-independent disposition block in Core verification. No
prior rejected document has correction or acceptance authority over these
bytes.

## Mandatory pre-draft sufficiency ruling

The landed Work Order 16 foundation is sufficient for this unit.

That sufficiency is deliberately bounded. Work Order 16 proves exact
Program-owned item and operation association and order. It does not promise a
structured projection for every expression and does not authorize this Work
Order to widen one. In particular:

- `structured_minimal_add_expression` remains bounded to
  `Binary(Add, Identifier, Identifier)`;
- honest literal-containing or otherwise unstructured additive expressions
  may retain `(None, None)` for their public structured projection and private
  structured authority;
- Core verification must reach the new disposition conclusion without relying
  on a structured row; and
- no implementation may invent a structured projection merely to obtain a
  convenient verifier insertion point.

That ruling is load-bearing. Current production code supplies every association
primitive this unit needs:

1. `Program::canonical_core_operation_owner_expectation` authenticates the live
   Program, file, item, `does:` section, parser owner, and task signature.
2. `with_expected_core_operations_for_item` supplies the exact body or
   predicate source artifact and checked slot order without an owned expected
   vector.
3. `CoreOperationCandidateOrigin` is attached directly to the one lowered
   operation created for that expectation.
4. `core_operation_occupies_expected_slot` is already load-bearing in Core
   verification and rejects a whole-operation swap even when public indices are
   coherently rewritten.
5. Parser canonical expressions already carry exact root and child identities,
   ordered roles, source ranges, operator kind, and identifier spellings.
6. `resolve_reference_summaries` already exposes canonical expression-node and
   child-position bindings plus resolver-owned semantic definition identity.
7. `type_env_report` and the existing `type_check` construction already expose
   the checked local declarations, resolver definition links, and accepted type
   references needed as inputs to the bounded producer. They do not expose an
   already-checked minimal-add return-expression conclusion.
8. `core_verify` already precedes and is consumed by `full_type_check`; a
   lifetime-bound verified access object can therefore be lent in the existing
   dependency direction without a cycle.

No missing invariant, producer, consumer, route, registry, manifest, or
association pass remains between these facts. In particular, neither
`src/parser.rs`, `src/resolve.rs`, `src/type_env.rs`, nor `src/ast.rs` needs a
new semantic fact. If implementation discovers that any of them must change,
the sufficiency ruling is false and implementation must stop before editing
outside the envelope.

## Complete sealed and special parser-fact inventory

Implementation begins by enumerating and testing the existing facts below; it
must not discover or invent another parser channel while coding:

- `CanonicalCoreFileWitness`: exact source revision bytes, normalized semantic
  file identity, semantic file index, and path spelling normalization;
- `CanonicalCoreOwnerWitness` and `CanonicalCoreOwnerBinding`: exact file
  binding, recursive item path, item kind, item span, task identity, and the
  parser-issued task-signature snapshot;
- task-signature facts: `task` keyword, task name, header extent and raw bytes,
  parameter delimiters, ordered parameters, permission spellings and
  explicitness, names, colons, type syntax, separators and whitespace gaps,
  optional result arrow and type, every token range, and authenticated order;
- `CanonicalCoreSealCapability` and `CanonicalCoreParseContext`: the existing
  parser-private section and parse-context issuance families;
- compiler-sealed `ValidatedCoreSection` first lineage and the retained
  canonical body report/statement lineage created from it;
- exact `does:` section slot, line identity, source span, parsed-body statement
  kind, block relationship, and source-node identity;
- canonical expression root identity, range and byte length, occurrence role,
  intent, payload and completion events, operator discriminant, associativity,
  ordered child identities and roles, identifier spellings, and child ranges;
- Work Order 16's body-versus-predicate source family, exact operation slot,
  checked slot arithmetic, and local-artifact missing/ambiguous/foreign/order
  failures; and
- special accepted source forms already covered by parser authority: task syntax
  with or without empty parentheses where currently legal, single and multiple
  valid spaces, forward- and backslash relative paths, absolute Windows paths,
  arbitrary non-corpus paths, CRLF, UTF-8 byte boundaries, and one-based
  line/column semantics.

No individual spelling, range, public node ID, or rendered expression is
authority by itself. Authentication is the complete retained relationship.
Malformed, overlapping, duplicated, omitted, relocated, reordered, overflowing,
underflowing, impossible, foreign-revision, or foreign-task facts fail closed.

## Purpose and one bounded result

Unit 1 closes backend-lowering gap 3 for one canonical expression family:

> A parser-authenticated task return whose exact Program-owned Core operation
> is canonical identifier `+` identifier, whose two operand references resolve
> to the two authenticated parameter declarations, and whose independently
> checked operand and result conclusion is `Int`, carries one operation-owned
> checked type authority. Core lower projects a separate non-authoritative type
> candidate and result value. Core verification compares both against the
> untouched authority only after same-slot and structural verification, then
> lends a lifetime-bound verified type result to full type.

The production chain is exactly:

```text
parser-authenticated task owner and canonical return expression
  -> Work Order 16 exact borrowed operation expectation
  -> existing resolver reference identities
  -> existing checked local declarations
  -> sole type_check producer context
  -> new private derivation of authenticated Int + Int -> Int
  -> one exact operation-owned untouched type authority
  -> separate public candidate and private non-authoritative claim
  -> same-callback Core operation and type verification
  -> report-bound lifetime-limited verified type access
  -> full-type statement consumer
  -> future-backend-shaped result-value projection
```

The unit does not produce a general arithmetic type system, backend artifact,
types table, ABI, IR, execution route, or readiness claim.

## Exact target and closed outcome partition

Target recognition begins from authenticated parser structure, never mutable
type projections, text search, path, filename, task name, parameter name,
public row ID, candidate vector position, or corpus count.

The only supported target has all of these facts:

- one authenticated task owner with exactly two ordered parameters;
- one exact Program-owned return operation;
- a canonical binary root whose operator is parser `Add`;
- two ordered identifier children with distinct parser node identities;
- left and right child references resolved to the first and second parameter
  declarations respectively;
- both parameter declarations locally complete and accepted as `Int`;
- the independently checked expression conclusion is `Int`; and
- a declared task result, when present, is retained only for compatibility
  comparison and never proves an operand or expression type.

The sole producer returns one of six mutually exclusive outcomes for the exact
operation being visited:

```rust
pub(crate) enum CanonicalMinimalAddTypeOutcome {
    Supported(CanonicalMinimalAddTypeAuthority),
    AuthenticatedOutOfScope(CanonicalMinimalAddOutOfScope),
    LegacyCompatibleAdditive,
    IntegrityFailure(CanonicalMinimalAddIntegrityFailure),
    UnsupportedTargetLike,
    NonTarget,
}
```

The names are normative; private field layout is not. None of these types is
public, serializable, deserializable, `Default`, or constructible outside
`src/type_check.rs`. `CanonicalMinimalAddTypeAuthority` has no production
`Clone` or `Copy` implementation.

The value returned here is the immutable operation-owned **source
disposition**. `type_check` produces it exactly once for the exact borrowed
operation expectation, and Core lower attaches that same value directly to the
corresponding operation. After issuance:

- its discriminant never changes;
- Core lower cannot replace or reclassify it;
- Core verification cannot rerun the producer, replace the disposition, or
  infer a second disposition from candidate state;
- full type cannot infer, reconstruct, or substitute it; and
- no production or test-only seam may replace one variant with another and
  present that replacement as producer evidence.

Verifier row status and the closed lookup/access outcome are a separate
immutable layer. They report whether the already-issued disposition and its
independently retained facts remain eligible for verified use; they never
rewrite source classification. The mandatory distinction is:

```text
source disposition
  != verifier row status
  != lookup/access outcome
  != report-global blocker
  != full-type acceptance
```

`IntegrityFailure` is issued only when the sole producer encounters an
incomplete, inconsistent, missing, duplicate, ambiguous, rejected, blocked,
foreign, reordered, substituted, or arithmetically invalid required authority
fact during initial classification. Later corruption of an already-issued
`Supported` operation does not create an `IntegrityFailure` disposition; it
causes Supported or existing structural predicates to fail and closes lookup
or delivery as specified below.

Classification precedence is exact and is evaluated from the authenticated
source operation before any mutable candidate or claim:

1. Use the retained parser structure to recognize the additive task-return
   family, then authenticate its task owner, source revision, item, section,
   operation, and canonical expression. If a recognized family member has a
   missing, corrupt, foreign, reordered, ambiguous, blocked, rejected, or
   inconsistent required fact, choose `IntegrityFailure` and stop.
2. For canonical `Binary(Add, Identifier, Identifier)`, bind both ordered
   children to exact resolver references and semantic parameter-definition
   identities, then bind those definitions to locally complete checked
   declarations. Derive the expression result from those authenticated operand
   facts, independently of the declared result type. Exact builtin `Int` on
   both operands and result is `Supported`; one coherent equal builtin non-Int
   type is `AuthenticatedOutOfScope`; every other complete relationship is
   `UnsupportedTargetLike`.
3. An authenticated `Binary(Add, Identifier, UIntLiteral)` that satisfies the
   established legacy additive preconditions is
   `LegacyCompatibleAdditive`. This check follows integrity checking and is
   disjoint from the identifier-plus-identifier target.
4. Any other authenticated additive task-return shape is
   `UnsupportedTargetLike`. The frozen nonempty witness is
   `Binary(Add, UIntLiteral, Identifier)`.
5. Only a statement or expression outside the authenticated additive
   task-return family is `NonTarget`.

Outcome meanings are:

- `Supported`: every required fact is complete and coherent and both operands
  and the expression result are exactly builtin `Int`.
- `AuthenticatedOutOfScope`: the exact same parser/resolver/declaration chain is
  locally complete and coherent for one equal builtin non-`Int` type. The
  current required regression is genuine `UInt + UInt`. This outcome carries
  no checked Core type authority, claim, result value, or verified access.
- `LegacyCompatibleAdditive`: the authenticated canonical expression is
  `Binary(Add, Identifier, UIntLiteral)` and satisfies the established legacy
  additive route. It receives no new checked Core authority, claim, result
  value, or verified access. Its existing actual type, `additive_expression_v0`
  provenance, successful Core verification and full-type behavior, and callable
  execution behavior remain byte-for-byte authoritative compatibility facts.
- `IntegrityFailure`: the parser-authenticated target is recognized, but a
  required task, operation, resolver, definition, declaration, type, range, or
  correspondence fact is missing, duplicate, ambiguous, rejected, blocked,
  foreign, reordered, inconsistent, partially substituted, or arithmetically
  invalid.
- `UnsupportedTargetLike`: an authenticated additive return is neither the
  supported/out-of-scope identifier pair nor the exact legacy-compatible
  identifier-plus-UInt-literal shape. This includes the frozen unstructured
  `UIntLiteral + Identifier` witness, unstructured literal-plus-literal
  additive forms, and structured identifier-plus-identifier forms with
  unequal complete operand types.
- `NonTarget`: unrelated statements and expressions.

Disposition and structured projection are separate dimensions. The exact
current topology is:

| Disposition | Required source witness | Structured projection | Retained structured authority |
| --- | --- | --- | --- |
| `Supported` | authenticated `Int` identifier `+` `Int` identifier | required | required |
| `AuthenticatedOutOfScope` | authenticated equal non-`Int` identifier `+` identifier | required | required |
| `LegacyCompatibleAdditive` | authenticated identifier `+` `UIntLiteral` | absent | absent |
| `IntegrityFailure` | recognized-family corruption | optional | optional, matching honest projection state before corruption |
| `UnsupportedTargetLike` | unsupported authenticated additive return | optional | optional, matching the expression shape |
| `NonTarget` | outside the additive task-return family | optional across the broad family | optional |

Honest Core-lower construction keeps projection and retained structured
authority paired as `Some/Some` or `None/None`. Test-only corruption may create
`None/Some` and must fail through existing structural verification. No outcome
may infer structured authority from slot authentication alone.

A newly parsed, internally coherent source mutation may establish another
genuine source disposition only by reparsing and rerunning the sole producer
before issuance. By contrast, once a `Supported` disposition and authority have
been issued, later corruption of a candidate, claim, comparison input,
projection, range, structure, or operation association leaves the stored
source disposition exactly `Supported`. The apparent downgrade is rejected by
failed Supported or structural predicates and a closed lookup/access outcome;
it is never reclassified as `AuthenticatedOutOfScope`,
`LegacyCompatibleAdditive`, `IntegrityFailure`, `UnsupportedTargetLike`, or
`NonTarget`. Missing or rejected later facts never enable a lower-precedence
fallback. A recognized authority failure never reaches legacy text inference.

The three current production surfaces are distinct and must not be conflated:

- `hum.type_check.v0` reports JSON-null `actual_type`, JSON-null `type_source`,
  and `unchecked_return_expression_v0` for the minimal-add return;
- `hum.core_lower.v0` reports the outer Core expression `type_status` as
  `not_type_checked_v0` and carries no authenticated checked Core type
  authority; and
- `hum.full_type_check.v0` currently accepts the statement with
  `actual_type: "Int"`, `type_source: "additive_expression_v0"`, and
  `status: "accepted_statement_type_v0"`.

Full type therefore already accepts `examples/core/minimal_add.hum` as `Int`,
but only through the legacy additive-expression fallback. That fallback is not
verified Core authority and may not be consumed by a backend as one. This
unit's sole producer must newly derive `Int + Int -> Int` from the authenticated
parser signature, exact resolver references, and accepted local
checked-declaration facts. For a `Supported` target, Core gains the checked
canonical type/result projection, Core verification authenticates it against
the untouched authority and Work Order 16 same-slot origin, and full type
changes provenance from `additive_expression_v0` to
`verified_canonical_minimal_add_type_v0`. The Supported route can no longer
reach additive fallback. `LegacyCompatibleAdditive` retains
`additive_expression_v0` unchanged. No existing checked minimal-add authority
is inherited, copied, or reinterpreted.

## Sole producer and direct operation ownership

`src/type_check.rs` owns the only producer context and the only authority
constructor. The context is built once per Core-lower report from the same
`Program` and diagnostics used by the report. It may cache immutable resolver,
type-environment, and checked-declaration facts for efficient lookup, but it
must not contain outcomes, operations, classification records, candidate IDs,
or an association vector.

For each exact `ExpectedCoreOperation` callback invocation, `src/core_lower.rs`
passes only the authenticated item and canonical source expression facts to
that producer. The producer returns the outcome immediately for that exact
operation. Core lower attaches the outcome directly to the corresponding
`CoreLowerOperation` while it attaches the Work Order 16 candidate origin. The
attached source disposition is retained unchanged for the operation's
lifetime. Core lower may derive the separate candidate and claim permitted by
that source disposition, but it cannot replace the outcome after observing its
own projection.

There is no:

- global authority or classification batch;
- outcome vector parallel to Core operations;
- cardinality comparison across unrelated operations;
- filtered count or retained classification ordinal;
- lookup by candidate index alone;
- join by public ID, task name, filename, spelling, text, or corpus position;
- second traversal that later re-associates a type record; or
- assumption that every Core operation has a minimal-add disposition.

A blocked `try`, predicate operation, unrelated item, or noncanonical
expression has no effect on another operation's authority. Reordering records
is impossible because no detachable records exist. Missing, duplicate, extra,
or foreign Core candidates remain Work Order 16 failures before type
verification begins.

## Untouched authority, candidate, and claim

For `Supported`, the exact `CoreLowerOperation` owns three separate facts:

1. its existing immutable private `CoreOperationCandidateOrigin`;
2. one untouched `CanonicalMinimalAddTypeAuthority` returned by `type_check`;
3. one independently materialized non-authoritative candidate claim.

The untouched authority retains at least:

- the authenticated Program/source revision and semantic file identity;
- exact item and operation source ownership;
- canonical root and ordered child node identities and ranges;
- resolver reference semantic identities and child roles;
- resolver-owned parameter definition semantic identities;
- checked declaration identities, accepted statuses, and exact type syntax;
- exact operand types;
- exact checked result type;
- canonical type ID `hum-type:builtin:Int`; and
- canonical result value ID derived from the parser root as
  `core-value:<parser-node-id>`.

The authority is never serialized, reconstructed from the public report,
derived from the candidate, moved into the claim, or lent to full type.

The private claim repeats only the comparison facts needed by Core verification
and is explicitly non-authoritative. After issuance, the stored untouched
authority and source disposition remain fixed. Tests independently corrupt the
public candidate, private claim, structured/candidate facts, or the borrowed
expected/foreign comparison side. Public-only, private-only, coherent
public/private, and foreign-comparison substitution must all fail while every
stored disposition and authority field remains exactly unchanged. Facts from a
separately parsed same-spelled program cannot satisfy the original operation's
stored authority comparison.

`AuthenticatedOutOfScope`, `LegacyCompatibleAdditive`,
`UnsupportedTargetLike`, and `NonTarget` own no authority or claim.
`IntegrityFailure` owns the closed failure reason needed to suppress fallback
but no supported authority or result value.

## Exact Core-lower public projection

The existing `hum.core_lower.v0` schema remains the schema family. Human output
is unchanged for all outcomes. Private authority, origin, claim, resolver
identity, declaration identity, and failure reason never serialize.

For a `Supported` expression, the existing expression fields become:

```json
"type_status": "checked_canonical_minimal_add_type_v0",
"type_text": "Int",
"type_source": "canonical_minimal_add_type_authority_v0",
"result_value": {
  "id": "core-value:<parser-node-id>",
  "type_id": "hum-type:builtin:Int",
  "type_status": "checked_canonical_minimal_add_type_v0",
  "type_text": "Int",
  "provenance": "canonical_minimal_add_type_authority_v0"
}
```

Field order is the existing expression prefix through `structured_expression`,
then `type_status`, `type_text`, `type_source`, `result_value`,
`effect_status`, and `reason`. The `result_value` object order is exactly `id`,
`type_id`, `type_status`, `type_text`, `provenance`.

For a producer-issued `IntegrityFailure`, `type_status` is
`canonical_minimal_add_type_integrity_failure_v0`, `type_text` and
`type_source` are JSON null, and `result_value` is present and JSON null. These
four fields change atomically; no partial result-value object is permitted.

For `AuthenticatedOutOfScope`, `LegacyCompatibleAdditive`,
`UnsupportedTargetLike`, and `NonTarget`, the existing outer type fields retain
their exact pre-unit values and the `result_value` field is absent. Thus genuine
UInt, established identifier-plus-UInt-literal additive programs, and unrelated
programs retain their current Core-lower bytes.

The result-value object is a deterministic, non-authoritative projection shaped
for later inclusion in a backend artifact's value and type tables. This unit
does not create those tables, make the object backend-consumable by itself, or
claim that JSON carries authority.

## Projection-independent Core verification and all-or-nothing access

Core verification uses the same Work Order 16 expected-operation callback
already used for order verification. For an exact candidate it must:

1. establish exact Program-item association;
2. call `core_operation_occupies_expected_slot` on the same borrowed
   expectation and candidate;
3. complete every relevant existing item, operation, expression,
   structured-expression, range, identity, and candidate-origin check;
4. compare the untouched type authority with the exact expected source facts;
5. compare the private claim with the untouched authority;
6. compare the public type and result-value projection with both; and
7. only after every target-local required check succeeds, mark that exact
   target locally eligible for report-bound verified access.

The verifier must not rerun, collect, cache, or reconstruct an expected
operation producer. It must not construct a view early and later decide whether
to keep it. Verification state for an operation is accumulated first. Local
eligibility is recorded only after those target-local predicates complete; a
borrowed view becomes deliverable only after the complete report satisfies the
separate report-global blocker below.

The new disposition conclusion is emitted inside `verify_operation`'s
`Some(expression)` branch and is independent of structured projection
presence. The exact relative topology is:

1. preserve these six existing `operation_expression` rows unchanged and in
   order:
   - `expression_source_status_consistent`;
   - `expression_status_known`;
   - `expression_ast_status_known`;
   - `expression_ast_present`;
   - `type_claim_honesty`;
   - `effect_claim_honesty`;
2. run the existing optional structured-expression verification for every
   honest `Some/Some` projection/authority pair, including all currently
   applicable independent structure, identity, range, source, and outer-type
   conclusions; and
3. select the disposition-specific block solely from the immutable
   operation-owned source disposition and emit it immediately afterward,
   before the operation verifier returns and before later item/root
   propagation.

This location exists for `Some/Some`, corrupt `None/Some`, and honest
`None/None`. Every new row uses scope `operation_expression`, the existing
operation ID, and the existing operation span. No unconditional generic
operation-expression row is replaced or suppressed.

### Supported disposition rows

The four Supported rows are selected by immutable `Supported` source
disposition, not by successful structured projection. An honest `Supported`
operation begins as structured identifier plus identifier. Its checked outer
type makes only the old
`structured_expression_outer_type_unchecked` conclusion inapplicable. For
Supported, omit that one structured conclusion while preserving the preceding
thirteen structured rows, then emit these four consecutive rows:

1. rule `canonical_minimal_add_type_authority_matches_exact_operation`, detail
   `canonical minimal-add type authority matches parser, resolver, declarations, and exact Core operation`;
2. rule `canonical_minimal_add_type_claim_matches_untouched_authority`, detail
   `canonical minimal-add private type claim matches untouched checked authority`;
3. rule `canonical_minimal_add_result_value_matches_checked_type`, detail
   `canonical minimal-add public type and result value match untouched checked authority`;
4. rule `canonical_minimal_add_type_access_locally_eligible`, detail
   `canonical minimal-add target is locally eligible for report-bound verified type access`.

Status is `passed_v0` only when that row's predicate is true and `failed_v0`
otherwise. Ordering is exactly the list above. A failed prerequisite causes
the relevant row and every dependent row to fail. The fourth row proves local
eligibility only; it never claims that access was issued or delivered. Clean
minimal Int changes from 35 to exactly 38 checks.

After issuance, the same four rows remain emit-able when later corruption
removes or damages the structured projection, structured authority, candidate
claim, public type/result projection, range, child identity/order, or an
independently compared expected fact. The source disposition remains
`Supported`. Applicable Supported and existing structural predicates fail,
`canonical_minimal_add_type_access_locally_eligible` fails, private lookup
returns `LocallyIneligible`, no view is delivered, full type is blocked, and
`additive_expression_v0` fallback remains unreachable.

The permanent corruption probes freeze these representative projection cases:

- healthy `Some/Some`: retain thirteen Supported-applicable structured rows,
  then emit four Supported rows; 38 total checks;
- corrupt `Some/None`: retain the same structured row topology with authority
  predicates failed, omit outer-unchecked, then emit four Supported rows; 38
  total checks;
- corrupt `None/Some`: emit the existing failed
  `structured_expression_projection_present` row, then four Supported rows; 26
  total checks; and
- corrupt `None/None`: emit no fabricated structured scope, then emit four
  Supported rows; 25 total checks.

These counts apply to the isolated one-item/one-operation corruption probes
without additional diagnostics. Candidate, claim, type/result, child, and
range mutations that retain `Some/Some` preserve the 38-row topology while
changing the exact applicable statuses. No post-issuance case emits
`canonical_minimal_add_integrity_failure_rejected` merely because its Supported
candidate became invalid.

### IntegrityFailure disposition row

Every operation whose immutable source disposition was issued as
`IntegrityFailure`, structured or unstructured, adds exactly one failed row
after its applicable existing generic and structured checks:

- rule `canonical_minimal_add_integrity_failure_rejected`;
- detail `recognized canonical minimal-add target has incomplete or inconsistent checked type authority`;
- status `failed_v0`.

A structured IntegrityFailure retains every valid structured conclusion,
including `structured_expression_outer_type_unchecked`; the new failure row is
additive and does not replace it. The representative structured unresolved
identifier case changes from 37 to exactly 38 checks. The representative
unstructured unresolved literal-plus-identifier case changes from 23 to
exactly 24 checks. Both Core-verifier reports fail, propagate through the
existing item/root all-or-nothing rules, exit nonzero, lend no view, and block
full-type fallback.

This row records a producer-time authentication failure. It is never emitted
merely because a later candidate, claim, comparison input, projection,
structure, or association attached to an already-issued Supported operation
became invalid. Such an operation remains Supported and fails the Supported or
existing structural rows above.

### UnsupportedTargetLike disposition row

Every `UnsupportedTargetLike`, structured or unstructured, adds exactly one
failed row after its applicable existing generic and structured checks:

- rule `canonical_minimal_add_unsupported_target_like_rejected`;
- detail `unsupported additive task-return shape has no checked canonical type authority`;
- status `failed_v0`.

A structured target-like operation retains
`structured_expression_outer_type_unchecked` because its public type fields
remain honestly unchecked; the rejection row is additive. The representative
mixed identifier-plus-identifier case changes from 35 to exactly 36 checks.
The representative unstructured literal-containing cases change from 21 to
exactly 22 checks. Both forms fail Core verification, propagate through the
existing item/root all-or-nothing rules, exit nonzero, lend no authority or
view, and cannot reach full-type additive fallback.

### Unchanged dispositions and public behavior

`AuthenticatedOutOfScope`, `LegacyCompatibleAdditive`, and `NonTarget` emit no
new disposition row. Their representative Core-verifier totals remain exactly
35, 21, and 21 respectively. Existing structured conclusions are preserved
only when a structured projection actually exists; this document does not
claim that LegacyCompatibleAdditive or NonTarget owns an outer-type row.

Human and JSON verification surfaces carry identical rule, scope, ID, span,
status, detail, ordering, and count meanings. JSON preserves the existing
object and array ordering, with conditional disposition rows at the relative
location frozen above. Human output uses the existing check renderer and field
order; no second human-only conclusion or abbreviated success claim is
permitted.

No private type, type ID, result-value authority, outcome, origin, claim, or
verified access serializes in `hum.core_verify.v0`. Only the named public check
rows reveal Supported authority checks, target-local eligibility, or explicit
IntegrityFailure/UnsupportedTargetLike rejection.

## Lifetime-bound verified access

`src/core_verify.rs` owns the only complete-report construction and verified
type delivery route. The exact production types and entry point are:

```rust
pub(crate) struct CoreVerifyFullTypeHandoff {
    readiness_summary: CoreVerifyReadinessSummary,
    diagnostic_occurrences: DiagnosticOccurrenceSet,
}

pub(crate) struct CoreVerifyFullTypeReportAccess<'report> {
    report: &'report CoreVerifyReport,
    readiness_summary: &'report CoreVerifyReadinessSummary,
}

pub(crate) struct CoreVerifyDiagnosticOccurrenceAccess<'report> {
    occurrences: &'report DiagnosticOccurrenceSet,
}

pub(crate) struct VerifiedCanonicalMinimalAddTypeResult<'report> {
    /* private borrowed verified fields */
}

pub(crate) enum CanonicalMinimalAddTypeLookup<'report> {
    Delivered(VerifiedCanonicalMinimalAddTypeResult<'report>),
    LocallyIneligible,
    ReportBlocked,
    NonSupportedDisposition,
    MissingOperation,
    DuplicateOperation,
    AmbiguousOperation,
    ForeignOperation,
}

pub(crate) fn with_core_verify_for_full_type<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'report> FnOnce(
        CoreVerifyFullTypeReportAccess<'report>,
    ) -> R,
) -> (CoreVerifyFullTypeHandoff, R);
```

The fields shown on the two access structs and the handoff remain private. The
following methods are the complete parent-visible consumption surface:

```rust
impl CoreVerifyFullTypeHandoff {
    pub(crate) fn into_parts(
        self,
    ) -> (CoreVerifyReadinessSummary, DiagnosticOccurrenceSet);
}

impl<'report> CoreVerifyFullTypeReportAccess<'report> {
    pub(crate) fn readiness_summary(
        &self,
    ) -> &'report CoreVerifyReadinessSummary;

    pub(crate) fn diagnostic_occurrences(
        &self,
    ) -> CoreVerifyDiagnosticOccurrenceAccess<'report>;

    pub(crate) fn canonical_minimal_add_type_for(
        &self,
        item: &Item,
        statement: &ParsedBodyStatement,
    ) -> CanonicalMinimalAddTypeLookup<'report>;
}

impl<'report> CoreVerifyDiagnosticOccurrenceAccess<'report> {
    pub(crate) fn occurrences(
        self,
    ) -> impl Iterator<Item = &'report DiagnosticOccurrence> + 'report;
}
```

No listed type has `Clone`, `Copy`, `Default`, serialization, deserialization,
an owned conversion, a public field, or another constructor. The exact private
fields of `VerifiedCanonicalMinimalAddTypeResult` are not normative; its
existing permitted string/identity accessors remain those frozen below.

For every invocation of `full_type_check::build_report`, the entry point above
executes this exact sequence:

1. call the private Core-verifier `build_report` exactly once and retain that
   one complete `CoreVerifyReport`;
2. derive one `CoreVerifyReadinessSummary` from that report without rebuilding
   it;
3. invoke `consume` exactly once with access borrowing that same report, its
   same readiness summary, its diagnostic occurrence set, and its per-operation
   verification results;
4. require `R` to be owned and independent of `'report`, so the callback may
   copy only the already-permitted verified strings into owned full-type
   statement parts; and
5. after the callback borrow ends, move—not clone or recompute—the readiness
   summary and the exact existing `DiagnosticOccurrenceSet` from that same
   report into `CoreVerifyFullTypeHandoff`, then return the handoff and `R`.

`CoreVerifyFullTypeHandoff::into_parts` consumes the handoff. It is the only
route by which full type receives the owned readiness summary and occurrence
set that its existing owned report already stores. The borrowed report access,
borrowed diagnostic iterator and occurrences, expected-operation facts,
authority, and verified results cannot escape the HRTB invocation. Moving the
owned summary and exact occurrence set after all report borrows end preserves
current ownership without creating a detached copy or cache.

The no-escape rule applies to borrowed readiness access, borrowed diagnostic
access and occurrences, report identity, expectations, authorities, and
verified results. It does not forbid the existing owned
`CoreVerifyReadinessSummary` and `DiagnosticOccurrenceSet` from being moved
exactly once into the owned `FullTypeCheckReport` after every report-bound
borrow has ended.

Three facts are distinct and must never be collapsed:

1. **Target-local eligibility.** The exact operation's same-slot ownership,
   authority, claim, structure, public projection, and type checks all pass.
   The fourth public row records only this fact.
2. **Report-bound private access delivery.** The verifier first builds one
   complete report. If any check anywhere in that report fails, or its existing
   summary/status is not successful, the report-global blocker closes delivery
   for every locally eligible Supported operation. The HRTB callback is still
   invoked exactly once; lookup of such an operation returns the closed no-view
   `ReportBlocked` outcome while readiness and diagnostic access remain
   available from the same report. Otherwise, a locally eligible `Supported`
   target returns one borrowed verified result.
3. **Full-type acceptance.** Full type may accept only a borrowed result that
   was actually delivered after the report-global blocker passed and that
   matches the exact live statement. Local eligibility alone is never
   acceptance evidence.

The lookup contract is total and its precedence is exact. First resolve the
requested live item and statement against the report: missing, duplicate,
ambiguous, or foreign association returns the corresponding closed variant.
Next, a disposition other than `Supported` returns
`NonSupportedDisposition`. A Supported operation with any failed target-local
predicate returns `LocallyIneligible`. A locally eligible Supported operation
returns `ReportBlocked` when the report-global blocker is set and `Delivered`
otherwise. Exactly one variant is returned for every request. An unrelated
failed operation may therefore leave a supported target's local-eligibility row
`passed_v0` while the same report supplies its failure summary and diagnostic
occurrences, closes every private view, and leaves full type blocked. No public
row serializes delivery or acceptance.

Each lookup result is a closed access outcome over the immutable source
disposition; it is not a replacement disposition. Missing, duplicate,
ambiguous, or foreign operation association therefore leaves the stored
Supported value unchanged while returning the exact association variant.
Likewise, target-local corruption leaves Supported unchanged while returning
`LocallyIneligible`, and an unrelated failed report leaves it unchanged while
returning `ReportBlocked`.

The exact private layout may vary, but these properties are mandatory:

- access borrows the actual complete Core-verify artifact, readiness summary,
  diagnostics, and exact operation-owned untouched authorities;
- borrowed access owns none of them and cannot outlive the report;
- the report access, diagnostic access, and individual verified result have no
  `Clone`, `Copy`, `Default`, serialization, deserialization, owned, or
  `'static` conversion;
- no constructor or field is visible to tests or downstream modules;
- failed, missing, partial, or later-invalid target-local checks make that
  target ineligible; any failed report-global check closes delivery of all
  supported results;
- lookup accepts only the live Program item plus canonical statement/expression
  identity needed by full type, never public IDs, names, paths, text, or vector
  position alone;
- verified-result accessors expose only result value ID, type ID, type text
  `Int`, verified provenance, exact source statement identity, and
  declared-result compatibility;
- full type cannot receive the lower owner, expected operation, candidate
  origin, untouched authority, private claim, or verifier construction permit;
  and
- the higher-ranked callback prevents retention after the verifier artifact is
  dropped.

The production types are subject to a cfg-selected compile-fail proof with at
least these five actual misuse attempts:

- `verified_canonical_minimal_add_access_cannot_outlive_verify_artifact`;
- `verified_canonical_minimal_add_result_cannot_be_collected`;
- `verified_canonical_minimal_add_result_cannot_become_static`;
- `core_verify_full_type_report_access_cannot_escape`;
- `core_verify_diagnostic_occurrence_access_cannot_escape`.

A normal `cargo check --all-targets` must pass before and after. The cfg build
must exit 101 because of the intended lifetime relationships, name all five
functions, and contain no privacy, missing-symbol, unresolved-import,
unexpected-cfg, or unrelated first failure. `RUSTFLAGS` must be restored and
absent afterward.

## Full-type consumer and fallback partition

`src/full_type_check.rs::build_report` is the sole downstream consumer. Each
invocation calls `with_core_verify_for_full_type` exactly once. Inside the HRTB
callback it reads the borrowed readiness summary, collects owned full-type
statement parts, and consumes a verified result only for the exact live `Item`
and `ParsedBodyStatement` accepted by the access object. It immediately copies
the permitted verified strings into the existing owned `TypedStatement`
output. After the callback returns, `build_report` consumes
`CoreVerifyFullTypeHandoff`, receives the exact owned readiness summary and
diagnostic occurrence set from that same verifier report, extends the latter
with its existing full-type occurrences, and assembles the existing owned
`FullTypeCheckReport`.

The current separate calls to `core_verify_readiness_summary` and
`core_verify::diagnostic_occurrence_set` are removed from `build_report`; both
facts instead arrive through this single handoff. No invocation of
`build_report` calls Core verification a second time, reconstructs its summary,
copies a detached diagnostic set, or invokes another expected-operation
producer.

The public `hum.full_type_check.v0` structure, row count, nesting, and field
order remain unchanged. At `$.typed_items[i].statements[j]`:

- supported compatible `Int + Int` keeps `expected_type` from the declared
  task result, sets `actual_type` to `Int`, sets `type_source` to
  `verified_canonical_minimal_add_type_v0`, and uses
  `accepted_statement_type_v0` with null reason;
- supported declared-result mismatch retains verified actual `Int`, uses
  `rejected_statement_type_mismatch_v0`, and reason
  `statement_expression_type_mismatch`;
- a producer-issued `IntegrityFailure` suppresses legacy inference and uses
  the existing prior-error/null field behavior because Core verification
  failed;
- a post-issuance corrupted Supported operation remains Supported, receives a
  closed `LocallyIneligible` or association lookup outcome, and uses the same
  existing prior-error/null behavior without reaching fallback;
- `AuthenticatedOutOfScope` preserves the exact prior full-type behavior,
  including genuine UInt;
- `LegacyCompatibleAdditive` preserves the exact existing
  `additive_expression_v0` route, actual type, acceptance, and callable
  behavior without receiving the new authority;
- `UnsupportedTargetLike` is blocked by the Core-verifier failure and cannot
  fall through to additive inference; and
- `NonTarget` preserves exact prior behavior.

For a supported, integrity, or unsupported-target-like target, the old
string-splitting additive branch is unreachable. It remains reachable only for
the explicitly classified `LegacyCompatibleAdditive` compatibility route and
pre-existing non-target behavior. Declared result type is compatibility input
only. Within one `full_type_check::build_report` invocation, no full-type path
may call the type producer, Core lower, or Core verifier again, accept
serialized Core fields as authority, or retain the borrowed result. A
report-global blocker uses the existing prior-error/null behavior and cannot
accept a locally eligible target.

The existing CLI-level construction pattern is explicitly preserved and is not
the one-pass boundary in this Work Order:

- `full_type_check_has_errors` may invoke `full_type_check::build_report` once;
- `full_type_check_text` or `full_type_check_json` may independently invoke
  `full_type_check::build_report` again;
- one `full-type-check` CLI command may therefore build two independent
  full-type reports and execute Core verification once within each report
  build;
- no report or result is cached or shared between those calls;
- `src/main.rs` remains frozen; and
- consolidating the two CLI report builds is a possible future optimization
  requiring a separate Work Order.

No statement in this document claims command-level single execution. The
mandatory invariant is exactly one complete Core-verification execution per
individual `full_type_check::build_report` invocation.

## Closed behavior matrix

| Source/corruption | Projection topology | Lower | Verify | Full type | Legacy additive fallback |
| --- | --- | --- | --- | --- | --- |
| authenticated `Int + Int`, compatible `Int` result | structured `Some/Some` | checked candidate, result value, private claim, untouched authority | thirteen retained structured rows plus four Supported rows; 38 total; one view delivered after report-global success | accepted actual `Int` with verified provenance | unreachable |
| authenticated `Int + Int`, declared-result mismatch | structured `Some/Some` | same independently checked actual `Int` | same 38 successful checks | existing mismatch with actual `Int` | unreachable |
| genuine coherent `UInt + UInt` | structured `Some/Some` | exact old unchecked fields; no result value | exact old 35 successful checks and bytes | exact old behavior | preserved |
| authenticated `Identifier + UIntLiteral` satisfying the legacy route | unstructured `None/None` | exact old unchecked fields; no result value | exact old 21 successful checks and bytes; no structured row | exact old actual type and `additive_expression_v0` provenance | preserved |
| locally valid Supported target plus an unrelated report failure | structured `Some/Some` | Supported projection remains locally valid | four Supported rows pass; report-global blocker delivers no view | existing prior-error/null behavior | unreachable for the Supported target |
| producer-time structured recognized-family authentication failure, including unresolved identifier | structured `Some/Some` | producer-issued integrity fields; no supported authority/result value | retain all applicable structured rows and add one failed integrity row; representative total 38 | prior-error path | unreachable |
| producer-time unstructured recognized-family authentication failure, including unresolved `UIntLiteral + Identifier` | unstructured `None/None` | producer-issued integrity fields; no supported authority/result value | six generic rows plus one failed integrity row; representative total 24 including existing blocker rows | prior-error path | unreachable |
| issued Supported operation later made to resemble any lower disposition | projection may remain or be corrupted away; stored source disposition remains Supported | existing Supported outcome and authority remain fixed; only later comparison material changes | four Supported rows and every applicable structural row expose the corruption; lookup `LocallyIneligible`; no integrity row | prior-error path | unreachable |
| public-only, private-claim-only, or coherent public/private post-issuance substitution | structured state retained unless separately corrupted; source disposition remains Supported | corrupted candidate/claim; stored authority and disposition fixed | Supported comparison failure; lookup `LocallyIneligible`; no access | prior-error path | unreachable |
| post-issuance whole-operation swap, missing/duplicate/ambiguous/foreign operation association | any; source disposition remains Supported | candidate may appear self-consistent | Work Order 16 association failure and the exact closed association lookup outcome; no reclassification | prior-error path | unreachable |
| authenticated mixed-type `Identifier + Identifier` target-like shape | structured `Some/Some` | old unchecked fields; no result value | retain 35 old checks and add one failed target-like row; 36 total | prior-error path | unreachable |
| authenticated literal-containing target-like shape, including `UIntLiteral + Identifier` and literal plus literal | unstructured `None/None` | old unchecked fields; no result value | retain 21 old checks and add one failed target-like row; 22 total | prior-error path | unreachable |
| unrelated/noncanonical expression | optional across the broad family | exact old bytes | exact old checks; representative identifier return remains 21 | exact old behavior | preserved |

## Exact nine-path implementation envelope

Unit 1 may modify exactly these paths:

1. `src/type_check.rs` -- build the sole immutable producer context; classify
   the exact canonical operation from existing parser, resolver,
   type-environment, and checked-declaration inputs; newly derive the bounded
   `Int + Int -> Int` conclusion; construct the only supported authority; keep
   `hum.type_check.v0` output unchanged.
2. `src/core_expr.rs` -- define the exact checked type/result-value constants
   and bounded projection shape used by Core lower; add no general expression
   inference.
3. `src/core_lower.rs` -- invoke the producer only inside the Work Order 16
   expected-operation callback; attach one outcome/authority directly to the
   exact operation; create the independent claim and public projection; add no
   batch or reassociation pass.
4. `src/core_verify.rs` -- reuse the existing same-callback slot predicate,
   compare authority/claim/public candidate after all structural prerequisites,
   preserve the six generic operation-expression rows, emit the exact
   projection-independent disposition block after optional structured
   verification, build the sole per-report readiness/diagnostic/type handoff,
   and solely lend lifetime-bound verified access.
5. `src/full_type_check.rs` -- replace its two internal Core-verifier calls with
   one handoff per `build_report`, use verified actual type/provenance for the
   exact statement, preserve existing readiness and diagnostic projection, and
   enforce the fallback partition without a second producer or inference route.
6. `docs/HUM_CORE_LOWER_SCHEMA.md` -- document exact supported, integrity,
   legacy-compatible, absent, null, ordering, and private-boundary behavior.
7. `docs/HUM_CORE_VERIFY_SCHEMA.md` -- document the one Supported-only
   outer-type replacement, projection-independent IntegrityFailure and
   UnsupportedTargetLike rows, preserved generic/structured rows, exact
   ordering and counts, target-local eligibility, report-global blocking,
   failure propagation, no-view behavior, and serialization limits.
8. `docs/HUM_FULL_TYPE_CHECK_SCHEMA.md` -- document verified provenance,
   mismatch, prior-error, out-of-scope, legacy-compatible, non-target, and
   unchanged-row behavior.
9. `tools/check_all.ps1` -- register exactly the four focused selectors below
   in normal Fast, update the closed exact-selector inventory from 91/91 to
   95/95, and add named membership assertions. It may not change the
   classifier, lanes, workflows, Exhaustive producer, F4 audits, readiness
   gates, or any unrelated validation.

Every path is load-bearing. `src/ast.rs`, `src/parser.rs`, `src/resolve.rs`,
`src/type_env.rs`, `src/core_body.rs`, `src/core_preview.rs`, `src/main.rs`,
`src/ir_readiness.rs`, fixtures, examples, Cargo files, workflows, decisions,
research, snapshots, generated files, and every other path are frozen.

A tenth path is a stop. So is removing a listed path because its obligation was
silently dropped. No inline scope amendment is permitted during implementation.

The sustainability budget is separated so production machinery cannot hide
inside evidence volume:

- production Rust across the five source paths: at most 900 insertions and 100
  deletions;
- permanent Rust tests plus the four tool registrations: at most 900
  insertions and 60 deletions;
- the three schema documents: at most 400 insertions and 20 deletions;
- complete nine-path candidate: at most 2,200 insertions and fewer than 180
  deletions; and
- focused independent source review: at most four hours before a stop and BDFL
  reassessment.

All category and total limits apply. Moving code between categories or using
format suppression does not change the count. Exceeding a category or total
bound stops before commit and returns to the BDFL.

The topology ruling does not widen these budgets. One shared
projection-independent disposition helper must cover structured and
unstructured failure shapes; the permanent matrix must reuse production-path
builders and common exact-row assertions rather than duplicate a fixture-sized
test per cell. Under that constraint, the 900-production, 900-test/tool,
400-schema, and 2,200-total insertion ceilings remain credible and unchanged.

## Permanent focused evidence

These exact selectors are required:

1. `type_check::tests::canonical_minimal_add_type_authority_is_operation_bound`;
2. `core_lower::tests::canonical_minimal_add_type_authority_is_owned_by_exact_operation`;
3. `core_verify::tests::canonical_minimal_add_type_verification_withholds_invalid_access`;
4. `full_type_check::tests::minimal_add_consumes_only_verified_canonical_type`.

Each must list exactly one test, run exactly one test, pass, and earn exactly one
unique helper credit. Normal Fast must invoke each once in an isolated helper
block. The final inventory must be exactly 95 invocations and 95 unique
selectors and must contain all four names.

Tests use real production parsing, resolving, type checking, Core lowering,
Core verification, and full-type consumption. Source searches and toy types are
supplemental only. Test-only seams may provide corrupted parser/resolver/
declaration inputs to the real sole producer before issuance, or mutate
independently compared candidate/claim/projection/association facts after
issuance before production verification. They may not mint authority, replace
or directly construct a disposition, reconstruct production identity, rerun
the producer after issuance, or hard-code a production path/name/ID as semantic
selection.

Collectively the selectors and independent reviewer probes must cover:

- clean supported Int, genuine coherent UInt, established legacy-compatible
  identifier-plus-UInt-literal behavior, structured mixed-type target-like,
  unstructured UInt-literal-plus-identifier, and unstructured
  literal-plus-literal target-like witnesses;
- the exact disposition/projection topology: Supported and
  AuthenticatedOutOfScope structured, LegacyCompatibleAdditive unstructured,
  UnsupportedTargetLike structured and unstructured, IntegrityFailure
  structured and unstructured, and representative NonTarget unstructured;
- exact row scope, relative order, operation ID, span, rule, detail, status,
  summary-count effect, human/JSON ordering, root propagation, and exit for
  every conditional disposition row;
- preservation, exact order, and independent load-bearing behavior of all six
  generic operation-expression rows for every disposition;
- the Supported-only omission of
  `structured_expression_outer_type_unchecked`, with all preceding structured
  rows preserved and the four Supported rows emitted afterward;
- retention of `structured_expression_outer_type_unchecked` for structured
  IntegrityFailure and structured UnsupportedTargetLike, followed by the
  separate projection-independent failed disposition row;
- absence of any attempted structured-row replacement for unstructured
  IntegrityFailure, unstructured UnsupportedTargetLike,
  LegacyCompatibleAdditive, and unstructured NonTarget;
- declared-result mismatch without changing independently checked actual type;
- foreign revision, file, item, task, statement, expression, and authority;
- same-visible-ID and same-spelled foreign substitution;
- operand reorder, duplicate identity, range corruption, and root corruption;
- missing, duplicate, ambiguous, rejected, blocked, foreign, reordered,
  partial, and inconsistent resolver/declaration facts;
- equal-typed definition substitution and same-spelled foreign binders;
- complete pre-issuance classification through every genuine disposition by
  reparsing and rerunning the real producer, followed by a separate
  post-issuance apparent-downgrade matrix that preserves the exact Supported
  source disposition and closes verification/access instead of reclassifying;
- result value ID, type ID, status, text, and provenance corruption;
- operation deletion, duplication, insertion, and whole-operation swap with
  coherently rewritten public indices;
- all relevant existing structural checks failing before access;
- Supported corruption that removes the structured projection and retained
  structured authority,
  proving that the immutable Supported source disposition still selects all
  four Supported rows through `(None, None)`, becomes `LocallyIneligible`, and
  never emits the producer-time IntegrityFailure row;
- no access after an earlier or later target-local failure, and no delivered
  access after an unrelated report-global failure even when the target-local
  eligibility row passes;
- exactly one Core-verification execution inside each individual
  `full_type_check::build_report` invocation, with its readiness summary,
  diagnostic occurrences, local eligibility, blocker state, and borrowed type
  result proven to share one private report identity;
- `full_type_check_has_errors` and human/JSON rendering independently building
  reports without a cache, while each report build retains the one-execution
  invariant;
- the exact Core readiness and diagnostic occurrences remaining available for
  unrelated blockers even when type lookup returns `ReportBlocked`;
- no report access, diagnostic iterator or borrowed occurrence, expected
  operation, authority, or verified result escaping the HRTB callback;
- out-of-scope, legacy-compatible, and non-target byte preservation;
- checked slot, range, and ordinal overflow/underflow; and
- no fallback reachability for supported, integrity, or target-like outcomes,
  while legacy-compatible fallback remains reachable and unchanged.

Two separate permanent production-path matrices are mandatory. They may share
builders and assertions, but one may not substitute for or manufacture the
other.

### Matrix A: pre-issuance source classification

Matrix A begins from one clean canonical Supported source template. Each case
mutates source/parser/resolver/declaration inputs before outcome issuance,
reparses where source changed, and invokes the real `type_check` producer for
the exact expected operation. It covers:

1. `Supported`: the clean authenticated `Int + Int` source.
2. `AuthenticatedOutOfScope`: a coherent reparsed equal builtin non-Int source
   such as `UInt + UInt`.
3. `LegacyCompatibleAdditive`: a coherent reparsed
   `Identifier + UIntLiteral` source satisfying the established legacy route.
4. `IntegrityFailure`: genuine producer-time structured and unstructured
   authentication failures, including reparsed unresolved-reference witnesses
   and pre-producer missing, duplicate, ambiguous, rejected, blocked, foreign,
   reordered, substituted, or invalid resolver/declaration facts. Every case
   must call the real producer; no seam may directly construct or substitute
   the enum variant.
5. `UnsupportedTargetLike`: coherent reparsed structured mixed-type
   identifier-plus-identifier, unstructured `UIntLiteral + Identifier`, and
   unstructured literal-plus-literal witnesses.
6. `NonTarget`: a coherent reparsed operation outside the additive task-return
   family.

For every case, Matrix A asserts the exact immutable source-disposition
discriminant and private identity, authority presence/absence, Core projection,
verifier row set/count/status, lookup outcome, full-type result, and fallback
reachability. Producer-issued `IntegrityFailure` alone emits
`canonical_minimal_add_integrity_failure_rejected`.

### Matrix B: post-issuance Supported corruption

Matrix B begins once from one real production-issued Supported operation. It
captures the immutable source-disposition discriminant and every authority
identity/field before mutation. Each case mutates only independently compared
later material while preserving that stored Supported value exactly:

- `CoreOperationCandidateOrigin` or another candidate-origin comparison input;
- the private non-authoritative claim;
- the expected/foreign side of the untouched-authority comparison, never the
  stored authority itself;
- public type status, text, and provenance;
- result value ID, type ID, status, text, and provenance;
- structured projection presence and fields;
- separately retained structured authority presence and comparison facts;
- child identity, order, roles, and ranges;
- missing, duplicate, ambiguous, and foreign operation association; and
- an unrelated report-global failure.

The exact expected verifier/access behavior is:

| Post-issuance mutation | Required failed rows | Lookup/access outcome |
| --- | --- | --- |
| foreign or mismatched expected side against the fixed type authority | `canonical_minimal_add_type_authority_matches_exact_operation` and every dependent Supported row | `LocallyIneligible` |
| private claim | `canonical_minimal_add_type_claim_matches_untouched_authority` and every dependent Supported row | `LocallyIneligible` |
| public type or result projection | `canonical_minimal_add_result_value_matches_checked_type` and local eligibility | `LocallyIneligible` |
| structured projection removed with authority retained | existing `structured_expression_projection_present` plus every Supported row that depends on complete structure | `LocallyIneligible` |
| structured authority removed with projection retained | existing parser-authority/authority-comparison rows plus every dependent Supported row | `LocallyIneligible` |
| projection and structured authority both removed | no fabricated structured scope; every Supported row whose predicate requires the missing structure fails, including local eligibility | `LocallyIneligible` |
| child identity, order, role, or range | exact existing structured row, authority comparison, and dependent Supported rows | `LocallyIneligible` |
| missing operation association | existing Work Order 16 missing-operation failure | `MissingOperation` |
| duplicate operation association | existing Work Order 16 duplicate/extra association failure | `DuplicateOperation` |
| ambiguous operation association | existing Work Order 16 association failure | `AmbiguousOperation` |
| foreign operation association or corrupt candidate origin | existing Work Order 16 source-slot/origin failure | `ForeignOperation` |
| unrelated report failure | no target-local row is forced to fail; report-global blocker closes delivery | `ReportBlocked` |

Before and after every mutation, a module-private non-degenerate assertion
compares the stored source-disposition discriminant, authority identity, and
all byte-bearing/semantic authority fields for exact field equality. This is a
test-only private comparison, not serialization or public authority transport.
Replacing the enum, altering its stored authority, rerunning the producer, or
constructing an expected disposition in the assertion earns no credit.

Candidate, claim, authority-comparison, public type/result, range, child, and
structured corruption retains `Supported`, fails the exact applicable
Supported and structural rows, makes
`canonical_minimal_add_type_access_locally_eligible` fail, returns
`LocallyIneligible`, delivers no view, blocks full type, and leaves fallback
unreachable. The projection-state probes additionally freeze the 38/38/26/25
representative totals for `Some/Some`, `Some/None`, `None/Some`, and
`None/None` specified above.

Operation-association corruption also retains `Supported` and returns
`MissingOperation`, `DuplicateOperation`, `AmbiguousOperation`, or
`ForeignOperation` as applicable, with no view, no fallback, and blocked full
type. An unrelated failed operation may leave every target-local Supported row
passing but returns `ReportBlocked` and delivers no view. No Matrix B case
emits `canonical_minimal_add_integrity_failure_rejected`, changes source
disposition, or makes a lower-precedence fallback reachable.

## Production lifetime proof

The cfg selector is:

```text
hum_compile_fail_verified_canonical_minimal_add_type_escape
```

The proof uses the actual production access/result types and the five named
escape functions in this Work Order. Review requires normal check, expected
exit 101, intended lifetime diagnostics for all names, restored environment,
and a following normal check. A privacy failure, missing symbol, toy type,
unrelated target failure, or unexpected-cfg diagnostic earns no credit.

## Corpus and compatibility evidence

Implementation and independent review must reproduce the structured scan of
all current `.hum` files without using paths or counts as authority. The
expected current semantic inventory is:

- `examples/core/minimal_add.hum`: supported `Int + Int`;
- `examples/core/add.hum`: supported `Int + Int`;
- `fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum`: genuine
  authenticated out-of-scope `UInt + UInt`;
- every current authenticated `Identifier + UIntLiteral` task return that uses
  `additive_expression_v0` is `LegacyCompatibleAdditive`, including
  `examples/probes/passed_pure_callable.hum` and
  `fixtures/callable/session_al_lexical_identity_pass.hum`; the scan must report
  a nonzero exact count and preserve every such current root;
- reviewer-authored authenticated `UIntLiteral + Identifier`, literal-plus-
  literal, and mixed-type identifier-plus-identifier probes are
  `UnsupportedTargetLike`; each receives the exact failed disposition row and
  cannot reach legacy fallback;
- reviewer-authored producer-time structured and unstructured IntegrityFailure
  probes retain every applicable generic/structured row, add exactly one
  failed integrity row, and cannot reach legacy fallback;
- zero current integrity-failure targets; and
- every other root is non-target or an explicit unsupported target-like shape.

Traversal and structured-output failures must be zero. The implementation may
not add or edit a fixture or example.

Fresh baseline-versus-candidate evidence is required for representative human
and JSON Core lower, Core verify, and full type surfaces. JSON is run twice and
must be byte-deterministic. Required checks include:

- both supported Int programs show only the frozen new fields/rows/provenance;
- the current baseline for minimal add is independently recorded as null and
  `unchecked_return_expression_v0` in type check, `not_type_checked_v0` in Core
  lower, and accepted `Int` from `additive_expression_v0` in full type;
- after the candidate change, the Supported full-type path changes only its
  intended checked result/provenance to
  `verified_canonical_minimal_add_type_v0` and never reaches additive fallback;
- the representative Core-verifier totals are reproduced exactly as Supported
  38, AuthenticatedOutOfScope 35, LegacyCompatibleAdditive 21, structured
  UnsupportedTargetLike 36, unstructured UnsupportedTargetLike 22, structured
  IntegrityFailure 38, unstructured IntegrityFailure 24, and representative
  NonTarget 21; each failure disposition exits nonzero and each preserved
  disposition retains its frozen exit;
- genuine UInt retains exact pre-unit exits, stdout/stderr byte counts, hashes,
  766/766 Core-verifier checks, and its existing unrelated full-type blocker;
- representative legacy-compatible callable programs retain exact Core lower,
  Core verify, full-type, and execution exits and bytes, including their
  `additive_expression_v0` provenance;
- a supported target paired with an unrelated failing operation keeps a passing
  local-eligibility row, preserves the readiness summary and diagnostic
  occurrences from that same report, but receives `ReportBlocked`, no private
  view, and no full-type acceptance;
- instrumentation at the private handoff seam proves one Core-verifier report
  construction per `build_report`, common private report identity for summary,
  diagnostics and lookup, and no shared state between the CLI's independent
  status and rendering report builds;
- post-issuance Supported corruption preserves the exact private source
  disposition and authority field-for-field, fails Supported/structural rows,
  returns only `LocallyIneligible` or the exact association/report-blocker
  variant, emits no IntegrityFailure row, and cannot reach fallback;
- a representative non-task and noncanonical task retain exact bytes;
- Core preview, type-check, IR-readiness, capabilities, version, effect,
  ownership, resource, profile, runtime, and execution outputs are unchanged;
  and
- no private authority or claim term appears in any serialized output.

Historical hashes in closed Work Orders are comparison aids, not authority.
The reviewer must use identical invocations and resolve any environment/path or
stderr discrepancy rather than accepting unexplained drift.

## Proportional implementation evidence

On the final frozen implementation candidate, the implementer runs:

- the four exact selectors individually;
- the production lifetime compile-fail proof with normal checks before/after;
- targeted supported, integrity, legacy-compatible, target-like, UInt, and
  non-target probes;
- Matrix A's complete real-producer six-disposition classification evidence,
  Matrix B's immutable-Supported post-issuance corruption evidence, and the
  unrelated report-global blocker probe;
- the per-`build_report` one-verifier-execution and same-report handoff probes,
  including independent uncached status and human/JSON rendering builds;
- independent public/private/coherent substitution probes;
- the complete corpus classification;
- supported and unchanged deterministic output comparisons;
- `cargo fmt --all -- --check`;
- `cargo check --all-targets`;
- `cargo clippy --all-targets -- -D warnings`;
- `cargo test --all-targets` exactly once;
- `git diff --check`;
- text hygiene;
- public readiness;
- alpha claims;
- release readiness for `0.0.1`; and
- one Fast run after every preceding item is green.

Do not run Exhaustive locally. Do not edit or run a workflow. Full publication
CI owns the required Ubuntu and Windows full lanes and Ubuntu Exhaustive
producer. Performance evidence is out of scope.

The independent implementation reviewer repeats the four exact selectors,
lifetime proof, targeted adversarial probes, representative deterministic
outputs, corpus scan, fmt/check/clippy/diff/readiness checks, and one Fast run.
The reviewer does not mechanically repeat the complete root suite absent a
concrete dispute or contamination concern.

Compilation errors, ordinary rustfmt changes, Clippy findings, and a small
in-envelope line overrun discovered during implementation are implementer-inline
work only when they change no frozen semantic/public/evidence contract, require
no new path, stay within every final budget above, and are reported before the
candidate freezes. They do not consume the bounded post-review correction.
Anything semantic, architectural, public-contract-affecting, or outside a final
budget stops for amendment rather than being labeled cleanup.

If a broad evidence tier fails, reproduce the narrow failing selector, output,
or source audit before repeating the tier. Implementer and reviewer each run
Fast at most once on their final exact bytes. Neither runs local Exhaustive.
Publication CI must run full mode on the accepted commit, pass required Ubuntu
and Windows jobs, and show the Ubuntu Exhaustive selector green on those exact
accepted bytes; Windows skips only the platform-independent duplicate.

## Public-contract and readiness limits

This unit does not change:

- `hum.type_check.v0`;
- `hum.core_preview.v0`;
- `hum.ir_readiness.v0`;
- capabilities or version;
- any command or main routing;
- execution or IR readiness;
- effects, ownership, resources, profiles, termination, or safety claims; or
- any backend or runtime behavior.

The checked result and result-value projection are necessary future backend
inputs but are not a backend artifact. Only a later canonical backend input,
byte-bound IR verifier, and opaque `VerifiedBackendInput` may authorize backend
lowering. JSON, a Core-verifier report, or the verified type access in this unit
must never be presented as that later capability.

`Int` here is the existing Hum builtin type conclusion. This unit does not fix
machine width, layout, ABI, overflow lowering, signed instruction selection, or
trap behavior. Those remain governed by later contracts. The type ID is a
stable target-independent identity for future table association, not a
Cranelift type or ABI declaration.

## Explicitly deferred open-skeleton facts

Open-skeleton tooling is excluded, not partially designed here. The following
observations are preserved for its future, separately authorized planning:

- `hum graph` writes the complete index even when deliberately failing fixtures
  make the command exit 1, so future automation must not use `&&` or equate that
  exit with missing output;
- repeated loop use requires a release binary rather than rebuilding or using a
  development binary each iteration; and
- incremental-generation support and release-build full-graph wall-clock time
  remain unmeasured.

This Work Order may not edit graph output, build targets, hooks, dev-loop tools,
fixtures, workflows, or any open-skeleton document or implementation.

## Explicit bans and stop conditions

Unit 1 forbids:

- mutating, replacing, reclassifying, or reconstructing an issued source
  disposition;
- adding a verifier-side or full-type-side second classifier;
- directly substituting an enum variant in a test and treating it as producer
  or anti-downgrade evidence;
- widening structured-expression projection merely to obtain a verifier row;
- replacing a structured row for a disposition whose projection does not
  guarantee that row exists;
- suppressing any of the six unconditional generic operation-expression
  checks or any applicable independent structured check merely to preserve a
  historical check count;
- any global classification/authority batch or parallel vector;
- any second expected-operation producer, collected expectation, cached
  expectation, registry, manifest, ledger, or later reassociation pass;
- selection by path, filename, task/parameter name, text, public ID, public
  index, ordinal alone, vector position, or corpus count;
- treating declared result type, Core candidate fields, private claim, JSON,
  or a verifier row as checked type authority;
- deriving authority and candidate from the same mutable material;
- downgrading recognized integrity failure to out-of-scope, target-like,
  legacy-compatible, non-target, or legacy inference;
- treating later corruption of Supported as producer-issued IntegrityFailure
  rather than failed Supported/structural verification plus a closed lookup;
- constructing or retaining verified access before every relevant target-local
  check and the report-global blocker have completed;
- giving full type any authority, expected operation, candidate origin, lower
  owner, claim, or construction permit;
- adding general expression/operator/call typing or changing non-target
  fallback;
- adding a schema family, command, diagnostic code, fixture, example,
  dependency, workflow, Cargo change, source audit, backend artifact, IR
  verifier, runtime behavior, or readiness claim;
- unsafe code, unchecked arithmetic, saturation, wrapping, truncation, panic on
  adversarial candidate data, or hidden platform assumptions;
- archive recovery from Work Order 13 or 15; and
- open-skeleton integration, contract-hierarchy planning, termination work, or
  any later feature.

Stop without workaround if:

- any tenth path is required;
- `src/ast.rs`, parser, resolver, type environment, Core body, main, fixture,
  Cargo, workflow, or IR-readiness bytes must change;
- exact authority cannot be created directly for one expected operation;
- a dependency cycle appears;
- a batch, positional join, public-ID join, or second expected traversal is
  needed;
- current structural checks must be removed or weakened;
- public contract cannot be stated exactly;
- genuine UInt or unrelated bytes drift;
- the view can escape or a failed report exposes it;
- any focused selector is zero/multiple or absent from Fast;
- lifetime failure occurs for the wrong reason;
- implementation exceeds the size limit or cannot be reviewed in four focused
  hours; or
- any backend, execution, open-skeleton, or later-work behavior enters.

## Review, correction, commit, and publication gates

This fresh replacement is not a correction of either terminally rejected
planning package. Its first independent pre-issuance review returned `ACCEPT
WITH REQUIRED FIX` for one bounded contradiction in the source-disposition
lifecycle. This document now contains the sole authorized correction: it
freezes the producer-issued source disposition, separates it from later
verification and access outcomes, and divides the evidence into the real-
producer Matrix A and post-issuance immutable-Supported Matrix B.

The ordinary document correction cycle is consumed. The exact corrected
two-document package requires one fresh independent corrected-document
architect review. The reviewer did not author or edit it and must inspect the
current source chain, sufficiency ruling, immutable disposition lifecycle,
exact projection/row topology, exact nine-path envelope, Work Order 13/15
non-recurrence, public contract, lifetime boundary, evidence design,
sustainability, and document checks. The verdict is `ACCEPT`, `ACCEPT WITH
REQUIRED FIX`, or `REJECT`; no verdict authorizes an edit. Only an unqualified
`ACCEPT` advances. `ACCEPT WITH REQUIRED FIX` or `REJECT` stops this package and
returns it directly to the BDFL; no further correction is authorized.

Explicit BDFL acceptance of exact document bytes is required before a local
documentation commit. Publication, terminal full CI, publication-status
recording, and a Unit 1 implementation signal are separate gates.

The implementation receives one authoring pass, one fresh independent review,
and at most one separately authorized bounded correction within the nine-path
envelope. Acceptance, local commit, publication, terminal full CI, and status
closeout are separate. No gate implicitly begins the next activity.

## Document-author checks

Before presenting this draft, the author runs only:

- `git diff --check`;
- fail-closed no-index whitespace checking for untracked `WORKORDER_17.md`;
- the complete 123-case status-classifier suite twice with byte-identical
  output;
- text hygiene;
- public readiness;
- alpha claims;
- release readiness for `0.0.1`; and
- read-only source/call-graph inspection needed for this Work Order.

No Cargo command, Rust selector, Fast, full preflight, Exhaustive, performance
evidence, or CI is authorized during document authoring.

## Current authorization gate

Work Order 17 and Unit 1 are closed as accepted, implemented, published, and
terminal-green. Checked canonical minimal-add type authority now reaches exact
authenticated Core operations, Core verification, and full type. Supported
full type consumes `verified_canonical_minimal_add_type_v0`.

This bounded result does not implement a backend artifact, types table, IR
verifier, `VerifiedBackendInput`, execution, general expression typing, or
later lowering work. Native Cargo stderr and PowerShell capture remain a
deferred tooling defect; this closeout neither repairs nor authorizes repair of
that harness fragility.

Backend/IR work, types-table production, open-skeleton integration, harness
repair, a replacement or new Work Order, archive mutation, and every later
activity remained unauthorized until a separate BDFL signal. That signal
authorizes only the planning package recorded above. `WORKORDER_18.md` now
contains the sole active marker, and its implementation remains unauthorized
through this Work Order 17 closeout.
<!-- workorder-current-authorization-gate:end -->
