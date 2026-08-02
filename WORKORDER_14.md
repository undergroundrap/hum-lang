# Hum Work Order 14: Authenticate Task Signatures Through Core

Date: 2026-08-02
<!-- hum-active-workorder:v1 -->
Status: Work Order 14 Unit 1 is independently accepted, BDFL-accepted,
committed, published, terminal-green, and complete. The next gate is
publication of the local status-only closeout commit, pending separate BDFL
authorization. No later unit, implementation, planning transition, archive
action, new Work Order, repair, or unrelated work is authorized.

Owner: BDFL (Ocean).
Author: Work Order 14 architect-author. The author is permanently disqualified
from independently reviewing these bytes or an implementation produced under
them.
Planning baseline: `HEAD`, local `main`, cached `origin/main`, and live remote
`main` are all `1561a0fbb71cc9a51db0baf1beaf46842d5c736c`.

## Closed predecessor and archive boundary

Work Order 13 Unit 1 is terminally rejected, archived, and closed. Its initial
implementation review found three P1 defects, its sole bounded correction did
not earn `ACCEPT`, a direct BDFL technique-pinned terminal remediation was
attempted, and the terminal review still found one P1. The terminal defect was
that `canonical_minimal_add_batch_matches` compared 49 valid classification
records with only 47 ordinal-bearing operations: two legitimate blocked `try`
operations had no classification ordinal. The mismatch falsely changed the
established `UInt` Core-verification result from 766 of 766 checks passing with
exit 0 to 766 passing checks plus three failing canonical checks with exit 1.
No Work Order 13 implementation was accepted or published on `main`.

The rejected candidate is preserved only as rejection evidence on
`archive/workorder-13-unit1-terminal-rejection-2026-08-02` at commit
`f19c85748426867f0d4b3d5556ec5ed494a81e4c`, parent
`1561a0fbb71cc9a51db0baf1beaf46842d5c736c`, subject
`chore(archive): preserve rejected work order 13 unit 1`, complete tree
`09cdd585f73a20a58c2a1acec670d94fcc0fc954`, scoped ten-path OID
`58cce756824f04c629ef5550a8f16d3ffdabb1f1`, and inventory exactly ten paths at
`+4,005/-145`. Local, origin-tracking, and live remote archive refs all equal
that commit. Archive publication triggered no workflow.

Nothing in that archive is accepted implementation. This Work Order grants no
authority to merge or cherry-pick it, apply its patch, bulk-copy its files,
reuse its implementation or tests, repair Work Order 13, revive a correction
allowance, or claim archived behavior shipped on `main`. Read-only inspection
may establish failure history and dependency facts only.

## Purpose and exact Unit 1 result

Unit 1 establishes one type-agnostic, parser-owned task-signature authority and
makes the existing Core-lower owner carry it privately to existing Core
structural verification:

```text
raw parser header tokens and ranges
  -> parser-issued immutable task-signature authority
  -> direct ownership by the exact lowered Core item
  -> one existing Core structural-check ordinal becomes load-bearing
```

The result authenticates syntax ownership only. It makes no expression-type,
operand-type, result-type, supported-shape, or execution conclusion. A valid
canonical `Int` addition and the established valid `UInt` foundation fixture
are equally ordinary authenticated task signatures. Neither is classified as
supported, out of scope, an integrity class, or a semantic type case.

This boundary is load-bearing now: a task whose live or lowered signature no
longer agrees with untouched parser authority makes Core verification fail at
the normal per-item consumer, before any future semantic type claim could rely
on the item. The unit has no global classification batch and no association
between differently filtered vectors.

## Verified current dependency map

The planning audit traced current `main` as follows:

1. `src/parser.rs` parses the raw task header, its parameters, permissions,
   horizontal spacing, separators, optional result arrow, and result syntax.
   It already issues the file, item-owner, section, and parse-context witness
   families used by accepted Core ownership.
2. `src/ast.rs` owns the private witness types on `SourceFile`, `Item`, `Task`,
   and `Section`. `Program::canonical_core_expectation` already authenticates
   the live item pointer, file revision, normalized semantic file identity,
   traversal path, owner kind, section slots, and section capability.
3. `src/core_body.rs` consumes that expectation to validate parser-owned body
   structure. It needs no signature-field edit: its accepted Work Order 12
   expression authority remains unchanged and separate.
4. `src/core_lower.rs::build_core_lower_report_from_preview` visits each exact
   `Program` item and `core_item` constructs the corresponding
   `CoreLowerItem`. This is the direct one-to-one insertion point. A private
   field on that exact item needs no ordinal, name lookup, public ID, or global
   matching batch.
5. `src/core_verify.rs::verify_lower_report` iterates those exact lowered items,
   and `verify_item` emits the existing per-item structural checks. The current
   `body_grammar_consistency` ordinal can remain byte-identical for every valid
   item and become the single fail-closed signature check when authority is
   rejected.
6. `src/main.rs` already treats any failed Core verification check as an error.
   Full type and IR readiness already consume the Core-verification result and
   therefore need no new authority API or route.

The audit found no required resolver, type-environment, type-check, expression,
full-type, Core-preview, IR-readiness, command-routing, fixture, example, tool,
or downstream-pass edit. This is a direct producer-owner-validator chain, not a
preparatory object used only by tests.

## Private parser authority contract

### Issuance capability

`src/parser.rs` owns a zero-data `CanonicalCoreParserIssuance` capability with a
constructor private to that module. Every parser-owned Core witness constructor
in `src/ast.rs`, including the new task-signature snapshot constructor, must
require `&CanonicalCoreParserIssuance`. The parser creates the capability only
inside its existing parse route and uses it at exactly the four existing issuer
families: file, item owner, section capability, and parse context. The signature
snapshot extends the item-owner issuance; it is not a fifth issuer family.

The capability may be named where Rust visibility requires, but no module other
than `parser` may construct it. There is no `Default`, `Clone`, `Copy`, public or
crate-visible constructor, conversion, deserializer, macro escape, alternate
factory, runtime registry, random secret, `unsafe`, or test-only minting path.
A cfg-selected compile-fail attempt from outside `parser` must fail with Rust
E0624 at the private capability constructor and for no earlier unrelated reason.

### Immutable snapshot

The existing `CanonicalCoreOwnerBinding` is extended with the private field
`task_signature: Option<CanonicalTaskSignatureSnapshot>`: task owners receive
`Some`, while every non-task owner receives `None`. The private field prevents
downstream struct-literal construction; an `ast` constructor gated by
`&CanonicalCoreParserIssuance` is the only construction route. The snapshot is
constructed
from raw parser token/event facts while the header is parsed, never by reading
back mutable `Task.params`, `Task.result`, `result_syntax`, a Core projection,
serialized JSON, or downstream analysis.

The snapshot binds all of these facts:

- exact parser source revision and normalized semantic file identity;
- semantic file index, item traversal path, and owning task identity;
- the complete header range from the `task` keyword through the last result or
  closing-parenthesis token before the opening brace;
- raw header bytes, including each valid horizontal-space gap;
- task-keyword and task-name spelling, token range, and order;
- opening and closing parameter delimiters;
- every ordered parameter's explicit permission state and permission token when
  present, name token, colon, complete type-syntax spelling and range;
- every comma separator and its order;
- optional result-arrow presence, spelling, and range;
- optional result-type spelling, complete syntax, and range; and
- every token's one-based source span and checked byte length.

Header containment, non-overlap, source order, adjacency/gap facts, token count,
parameter count, and optional-result consistency are validated with checked
addition and subtraction. Overflow, underflow, impossible ends, duplicated or
overlapping ranges, absent or extra tokens, relocation, reordering, and ranges
outside the authenticated header fail closed. A same-spelled token in the body
cannot authenticate a header fact because its range is outside the authenticated
header and its raw slice position differs.

Multiple horizontal-space forms are valid only when the snapshot authenticates
the exact source-authored form. Path spelling is never a selection key:
relative, forward-slash, backslash, absolute-Windows, and arbitrary non-corpus
paths all use the existing normalized semantic file identity and source
revision rules.

Snapshot fields remain private and nonserialized. The parser-owned witness may
retain immutable `Arc`-backed data so existing AST and `Program` cloning keeps
working; cloning that already-issued immutable handle is not new issuance. The
snapshot and verified handle are not `Copy`, `Default`, serializable,
deserializable, publicly convertible, or publicly constructible.

### Live-owner authentication

`src/ast.rs` adds one fallible internal accessor on `Program` for the exact live
task item supplied by Core lowering. It first applies the existing exact
file/item/owner validation and then compares the private snapshot with the live
`Task` projection and its parser syntax:

- task name, span, item traversal identity, and source revision;
- ordered parameter names, permissions and explicitness, spans, type syntax,
  type whitespace validity, separator whitespace validity, and count;
- optional result spelling and exact `result_syntax` range; and
- the retained header token/range/order/containment facts.

The accessor returns an opaque authenticated handle only after every check
passes. Missing authority, live/retained disagreement, omission, duplication,
substitution, relocation, reordering, foreign task, foreign revision, overflow,
underflow, or impossible range returns a private rejection value. It never
panics, wraps, saturates, truncates, or reconstructs authority from the live
candidate.

## Direct Core ownership and consumption

`CoreLowerItem` receives one private, nonserializing field with this closed
state:

```text
NotATask
Authenticated(opaque parser-issued task-signature handle)
Rejected(private reason)
```

`core_item` obtains that state directly from the exact `Program` and `Item`
already used to build the item and stores it in the same `CoreLowerItem`. There
is no vector of signature records, no separate ordinal assignment, and no later
association pass. The identity chain is the parser file/revision identity,
exact item traversal identity and task owner, plus the existing direct
statement/root identity where body structure is checked. Ordinal alone, public
ID alone, vector position, task name, path spelling, and corpus counts are never
authority.

The authenticated handle remains separate from the public candidate fields.
Core lowering may borrow it to validate the candidate projection but may not
mint, replace, mutate, consume, reconstruct, or serialize it. There is no new
private candidate claim in this unit, so private-claim co-substitution is not
applicable. A test-only parser-verifier corruption operation may replace the
stored pre-lowering signature snapshot with a deliberately invalid clone (or
remove only that snapshot) without constructing a valid issuance capability or
authenticated handle. Tests may also corrupt public lowered fields. Once the
exact lower item owns its state, no seam may alter its untouched authenticated
handle; verifier corruption always changes candidate material or starts from a
pre-lowering rejected snapshot.

`core_verify` receives no `Program`, parser, source text, or public JSON. A
single private read-only method on `CoreLowerItem` returns only `NotATask`,
`Passed`, or `Failed` after comparing the candidate's task name, item span,
ordered parameters, permissions, type syntax and spans, and optional result with
the retained authenticated handle. It does not expose the handle or its private
fields. Coherently changing every public candidate signature field still fails
against the untouched handle. A separately parsed foreign task or revision
cannot authenticate the original item.

## Exact verifier and public-output contract

No Core-lower, full-type, Core-preview, type-check, IR-readiness, capability,
version, execution-readiness, or downstream public field is added or changed.
Private signature state is absent from every human and JSON serializer.

For `NotATask` and `Authenticated` items, every existing human and JSON byte,
check count, check ID, ordering, rule, detail, status, summary, and exit remains
unchanged. In particular, the existing `body_grammar_consistency` item check
keeps its current rule, detail `item keeps partial body grammar provenance`,
and pass/fail semantics. Valid canonical `Int` and valid canonical `UInt` inputs
therefore retain their exact current behavior.

For a task with `Rejected` signature authority, Core verification does not add
a row. At the existing `body_grammar_consistency` ordinal it emits exactly:

| Field | Frozen value |
| --- | --- |
| `scope` | `core_item` |
| `scope_id` | the existing item ID |
| `source_span` | the existing item span |
| `status` | `failed_v0` |
| `rule` | `task_signature_authority_matches_parser_owner` |
| `detail` | `task signature does not match retained parser authority` |

Signature rejection takes precedence at that ordinal if body grammar also
fails. No other existing check is removed or weakened. Existing item/root
failure propagation and CLI error behavior apply, so the command exits nonzero
through the current route. No semantic type conclusion is made. The only schema
change documents this conditional rule and its private authority boundary in
`docs/HUM_CORE_VERIFY_SCHEMA.md`.

## Closed outcome table

| Input and retained state | Core-lower private state | Public lower output | Core verification | Later consumers |
| --- | --- | --- | --- | --- |
| Non-task item | `NotATask` | exact prior bytes | exact prior checks | exact prior behavior |
| Valid task, including canonical `Int` addition | `Authenticated` | exact prior bytes | exact prior checks and exit | exact prior behavior; no type conclusion added |
| Valid established `UInt` foundation task | `Authenticated` | exact prior bytes | 766/766 existing checks pass, exit 0 | exact prior full-type behavior, including unrelated blockers |
| Missing, foreign, substituted, reordered, relocated, inconsistent, or arithmetically invalid signature authority | `Rejected` | existing field set and serializer; no authority or type field is added | same row count; frozen conditional rule fails and normal failure propagation applies | blocked by the existing Core-verification dependency; no new type behavior |
| Valid body grammar plus corrupted public lowered signature | retained authority remains untouched | candidate bytes may be corrupt in test seam only | frozen conditional rule fails | no self-validation |

The table is type-agnostic. No branch names or implements `Supported`,
`AuthenticatedOutOfScope`, `IntegrityFailure`, `Noncanonical`, `Int`, `UInt`,
minimal-add inference, or a verified type view.

## Exact Unit 1 writable envelope

Implementation may modify exactly these five paths:

1. `src/ast.rs` - private snapshot, parser-capability-gated constructors,
   checked live-owner validation, opaque handle, and test-only corruption seams.
2. `src/parser.rs` - parser-private issuance capability, raw header token/range
   capture, owner-snapshot construction, and focused parser tests.
3. `src/core_lower.rs` - direct per-item private ownership, the restricted
   verifier verdict method, and focused ownership/candidate-corruption tests.
4. `src/core_verify.rs` - load-bearing consumption at the existing item-check
   ordinal, failure propagation, and focused end-to-end verifier tests.
5. `docs/HUM_CORE_VERIFY_SCHEMA.md` - the exact conditional failure rule and
   statement that parser authority remains private.

A sixth implementation path is an immediate stop. In particular, these paths
remain byte-for-byte unchanged:

- `WORKORDER_13.md`, `WORKORDER_14.md`, `AGENTS.md`, and governance/decisions;
- `src/core_body.rs`, whose accepted parser-owned body structure remains intact;
- `src/type_check.rs`, `src/core_expr.rs`, `src/full_type_check.rs`,
  `src/resolve.rs`, `src/type_env.rs`, `src/core_preview.rs`,
  `src/ir_readiness.rs`, and `src/main.rs`;
- Core-lower and full-type schema documents;
- `Cargo.toml`, tools, workflows, fixtures, examples, snapshots, generated
  files, and every downstream effect, ownership, resource, profile, IR,
  backend, runtime, and execution path.

If parser signature authority cannot become load-bearing through these five
paths without a type layer, a batch, a route edit, a fixture/tool edit, or a
sixth path, implementation stops and reports the exact dependency. It may not
manufacture an adapter or broaden the unit.

This envelope is materially smaller than Work Order 13's rejected ten-path,
`+4,005/-145` candidate. It has one parser producer, one direct Core owner, one
existing verifier consumer, and one public schema consequence. It has no
resolver/declaration join, expression type producer, four-way semantic batch,
typed projection, verified type view, HRTB full-type handoff, corpus classifier,
or cross-operation cardinality. A reviewer can trace the complete authority
path in one sitting.

## Permanent focused evidence

Implementation must add exactly these stable focused tests inside the authorized
source files:

1. `parser::tests::canonical_task_signature_authority_rejects_substitution`
2. `core_lower::tests::task_signature_authority_is_owned_one_to_one`
3. `core_verify::tests::task_signature_authority_is_load_bearing`

The existing exact-selector helper must list exactly one nonzero test, run
exactly one test, pass it, and award one unique selector credit for each name.
No tool edit is authorized.

Together the real production-path tests must prove:

- valid canonical `Int` and valid canonical `UInt` task signatures authenticate
  without any type classification or conclusion;
- valid single and multiple horizontal-space forms authenticate;
- relative, forward-slash, backslash, absolute-Windows, and arbitrary
  non-corpus paths authenticate without becoming selectors;
- missing retained signature, moved result range into a same-spelled body token,
  coherent retained token/range relocation, and live/retained disagreement fail;
- parameter omission, duplication, reorder, permission/type/name substitution,
  result-arrow/result-type reorder, foreign task, and foreign revision fail;
- overlapping, duplicated, absent, extra, impossible, overflow, and underflow
  ranges fail without panic or wrap;
- Core lowering stores exactly one authority state in the exact produced item,
  with no batch, ordinal association, unmatched record, or record duplication;
- corrupting or removing untouched parser authority changes the real Core
  verifier result;
- corrupting only one or many public candidate fields cannot validate itself;
- public-field co-substitution cannot alter the untouched authority;
- all ordinary valid Core-lower, Core-verify, and full-type human/JSON output is
  byte-identical before and after the unit, including the established `UInt`
  fixture's 766/766 Core checks and exit 0; and
- no downstream module can mint the parser issuance capability or signature
  authority.

Corruption occurs before the production consumer validates. Source searches
may confirm issuer counts and bans but never replace behavioral evidence.
Fixture paths may be literals only for loading named regression inputs; no
production selection may use them.

## Foreign-issuance compile proof

Beside the actual production capability, `src/ast.rs` contains the exact
cfg-selected function
`canonical_core_owner_foreign_issue_must_not_compile`, whose only purpose is to
attempt foreign construction from outside `parser`. A narrowly placed
`allow(unexpected_cfgs)` may cover only this proof item so normal builds remain
warning-clean. With
`RUSTFLAGS=--cfg hum_compile_fail_canonical_core_owner_foreign_issue`,
`cargo check --all-targets` must exit 101 and identify the intended function
plus Rust E0624 at the private parser constructor. The failure must not arise
from privacy of the capability type itself, an unresolved import, missing type,
unexpected-cfg warning, or another target error. Normal checks immediately
before and after must succeed, and the process-local environment must be
restored even after the expected failure. No Cargo edit, dependency, fixture,
generated source, permanent artifact, or toy replacement type is permitted.

## Proportional implementation evidence

The implementer runs, on the final exact candidate and in this order:

1. Each of the three exact selectors through the existing helper.
2. A normal `cargo check --all-targets`, the expected foreign-issuance compile
   failure, and a second normal check after environment cleanup.
3. Targeted valid `Int`, valid `UInt`, whitespace, path, and corruption probes
   through the production Core-lower and Core-verify commands. Required JSON
   outputs run twice and must be byte-identical.
4. Exact before/after human and JSON byte/hash comparison for representative
   valid task inputs, including the established `UInt` fixture and its 766/766
   passing Core-verification result with exit 0.
5. `cargo fmt --all -- --check`.
6. `cargo check --all-targets`.
7. Applicable `cargo clippy --all-targets -- -D warnings`.
8. `cargo test --all-targets` exactly once on the final candidate.
9. `git diff --check`.
10. `tools/check_text_hygiene.ps1`, `tools/check_public_readiness.ps1`, and
    `tools/check_release_readiness.ps1`.

Local Fast, `tools/check_all.ps1`, Exhaustive, performance pairs, phase
ledgers, manifests, actor transcripts, dual-shell candidate identities, and
new validation infrastructure are forbidden. Full preflight belongs to
post-publication Ubuntu and Windows CI.

## Architectural and sustainability bans

Unit 1 must not introduce or perform:

- canonical minimal-add or other expression classification;
- resolver or checked-declaration joins;
- expression, operand, or result type production, including `Int` or `UInt`;
- Core typed projections, candidate type claims, verified type views, or
  full-type consumption;
- a global signature/classification vector or association by filtered counts;
- authority from text matching, spelling lookup, public IDs, vector position,
  ordinals alone, filenames, corpus counts, public JSON, or downstream reports;
- a new diagnostic family, command, schema family, cache, ledger, manifest,
  performance harness, parser, runtime registry, or secret;
- IR-readiness, backend, runtime, execution, optimization, effects, ownership,
  resource, profile, or later language-feature work; or
- archived Work Order 13 code, tests, patches, or asserted evidence.

The implementation is one complete dependency-coherent unit, with no subunit
or deferred compilation, formatting, lint, tests, schema synchronization, or
evidence. It receives one implementation authoring pass and at most one
separately BDFL-authorized bounded correction inside the five-path envelope
after its first fresh independent implementation review. A second non-`ACCEPT`
verdict stops the train. Difficulty, elapsed time, or a nearly passing result
does not authorize another correction or scope expansion.

Later semantic type authority is not promised or automatically authorized.
Any type classification, typed Core projection, or full-type handoff requires a
separate Work Order or separately reviewed BDFL authorization only after this
producer-to-verifier boundary is accepted, committed, published, and green.

## Review, commit, publication, and stop gates

The exact document package first goes to a fresh independent pre-issuance
architect-reviewer who did not author or edit it. Review acceptance grants no
edit, implementation, commit, push, or archive authority. Only the BDFL may
accept exact document bytes and separately authorize a local documentation
commit and later publication.

After terminal-green document publication and any required status record, a
separate BDFL Unit 1 signal is required. The completed unstaged implementation
then receives one fresh independent review. Only `ACCEPT` can return it to the
BDFL for a separately authorized scoped local commit. Commit, push, status, and
later work are distinct gates.

Stop without workaround if a sixth path, semantic type layer, global batch,
public-output change beyond the frozen conditional verifier rule, new command,
new fixture, new tooling, parser-authority forgery path, unchecked arithmetic,
self-validating candidate, or inability to preserve valid output bytes appears.

## Current authorization gate

This gate records only the final independent acceptance, accepted implementation
commit, publication, and terminal-green required CI for Work Order 14 Unit 1.

Final independent review returned `ACCEPT` with no P0, P1, or P2 findings.

Accepted implementation commit:

- commit: `e6c38b70b97a3dcc205c9c1b0533352603541f95`;
- parent: `ce2909a87d48c4c05403ed6810a812089f684482`;
- subject: `feat(core): authenticate task signatures through core`;
- committed scope: exactly five paths; and
- statistics: `+1,819/-54`.

Publication was the normal non-force fast-forward `ce2909a..e6c38b7`; only
`main` was pushed.

Required CI completed successfully for exact tested commit
`e6c38b70b97a3dcc205c9c1b0533352603541f95` in workflow `ci`, run
`30763812498`, attempt `1`, with overall conclusion `success`. Both platforms
selected `mode=full`, `reason=no_status_transition`.

Ubuntu evidence:

- job `91538922265` on `ubuntu-latest` concluded `success` in `25m50s`;
- full Hum preflight succeeded in `25m10s`;
- Exhaustive selected and passed exactly one test,
  `parser::tests::exhaustive_canonical_seal_pair_matrix_is_complete_and_nonzero`;
- Exhaustive reported selected/passed/failed `1/1/0`, F1 `630`, F2 `4,950`,
  F3/F4 `8,646`, total `14,226`, producer elapsed `16.297s`, and seed
  `0x48554D5F5345414C`; and
- status-only evidence correctly skipped.

Windows evidence:

- job `91538922222` on `windows-latest` concluded `success` in `36m06s`;
- full Hum preflight succeeded in `35m26s`;
- Exhaustive correctly skipped because Ubuntu owned the platform-independent
  producer; and
- status-only evidence correctly skipped.

Both platforms passed text hygiene and public readiness for `517` files, alpha
claims, and release readiness for version `0.0.1`.

Work Order 14 Unit 1 is complete. No later unit, implementation, planning
transition, archive action, new Work Order, repair, or unrelated work is
authorized. The next gate is publication of this status-only closeout commit,
pending a separate BDFL signal.
<!-- workorder-current-authorization-gate:end -->
