# Hum Work Order 13: Canonical Minimal-Add Type Authority Before Core

Date: 2026-08-01
<!-- hum-active-workorder:v1 -->
Status: Issued, uniquely active, published, and terminal-green. The fresh
re-scoped replacement received independent pre-issuance `ACCEPT` with no P0,
P1, or P2 findings, and the BDFL accepted the exact reviewed bytes. The exact
two-document package was committed and published as
`ec2c3e02d1e1a02ad7e5c83331454b8f534e3490` with parent
`63b7ca45a67aa0ed3e5c0c5f639592d2dc666612`, paths `WORKORDER_12.md` and
`WORKORDER_13.md`, and statistics `+767/-1`.

Workflow `30715057713`, attempt 1, completed successfully. Ubuntu job
`91409140530` succeeded in 25m24s; its full preflight succeeded in 24m52s and
its one selected and passing Exhaustive test proved F1 630, F2 4,950, F3/F4
8,646, and total 14,226 pairs in 16.159s. Windows job `91409140499` succeeded
in 36m03s; its full preflight succeeded in 35m17s and it correctly skipped the
platform-independent Exhaustive duplicate. Both jobs selected `mode=full` with
`reason=no_status_transition` and correctly skipped status-only evidence.

No implementation, repair, archive mutation, or later work occurred. Unit 1
remains explicitly unauthorized pending a separate BDFL implementation signal.

Owner: BDFL (Ocean).
Author: Work Order 13 re-scope architect. The author may report facts but is
disqualified from issuing an independent verdict on these bytes.
Planning baseline: `HEAD`, local `main`, cached `origin/main`, and live remote
`main` are all `63b7ca45a67aa0ed3e5c0c5f639592d2dc666612`.

Work Order 12 Unit 1 is accepted, published, terminal-green, and closed. Its
accepted implementation commit is `92cc5042903c4afe3c738acee9cd7a0ea4afd72b`,
its bounded audit repair is `e3f0f1720867c24dcf13f295cf3ee592e1b38737`,
and its final status commit is the planning baseline above. Its frozen working
file is 18,052 bytes and 355 lines, with SHA-256
`cd0cbea7596f8f546b33097a0340fde0efc06c6ca42387f2c59fc516c831a260` and
non-writing Git blob OID `802b4bfbba20fd72b291edb05b5eea436c06fded`.
The archived rejected implementation remains read-only at
`refs/heads/archive/workorder-11-unit1-sustainability-failure-2026-07-30`,
commit `a40fc65876a9224adecc492b18617ec60684136c`.

## Fresh scope boundary and superseded predecessor

The former untracked Work Order 13 draft was 37,700 bytes and 624 lines, with
SHA-256
`d3b4559bb487664ed1e27b64d8f1e55b8444e5790193728fb167b9edd92e937f` and
non-writing Git blob OID `e20adfb9a1adeed1355c4d6d475aea9ade9ac1f8`. It received an independent
pre-issuance `REJECT`, used its one stated correction cycle, and still contained
a terminal satisfiability defect. It was never accepted, issued, committed, or
implementation-authorizing. This document is a fresh re-scope under the
BDFL's explicit authoring signal, not another correction of those bytes.

The terminal defect was concrete. A structured scan of all 229 `.hum` files
under `examples/`, `fixtures/`, and `experiments/` found three, not two,
parser-owned task-return roots with exact
`Binary(Add, Identifier, Identifier)` shape. The predecessor treated every
non-`Int` instance as an integrity failure, which would make an established
`UInt` foundation witness fail Core verification. At the same time it excluded
that fixture and the standing harness from its writable envelope. The result
could not satisfy both the requested semantics and the repository's existing
production-path evidence.

No review verdict, finding disposition, correction quota, or authorization
state carries from the superseded predecessor. Load-bearing research and code
facts were re-audited from the planning baseline. The next actor must review
this complete fresh draft, not compare it as a patch to the rejected one.

## Purpose and one bounded result

Work Order 12 carries a parser-owned ordered
`Binary(Add, Identifier, Identifier)` return tree through validated Core-body
transport, Core lowering, and Core verification. It deliberately leaves that
tree's outer type as `not_type_checked_v0` / `null` / `null`. Today full type
later accepts the two `Int` examples through `infer_additive_expression_type`,
which splits display text with `split_once(" + ")` and consults a name-keyed
environment.

This order authorizes, only after every issuance gate below, one compiler-facing
result:

```text
authenticated parser task signature + parser-owned canonical add return
  -> exact resolver references -> exact checked parameter declarations
  -> private four-way classification
  -> typed Core candidate only for a supported Int target
  -> untouched-authority verification -> lifetime-bound verified view
  -> full-type return row sourced only from that verified view
```

The bounded supported rule is exact: both parser-authenticated operand binders
resolve to parameters of the current task, both parameter annotations and
their exact checked declaration rows are accepted `Int`, and the independently
produced additive expression type is `Int`. The authenticated declared result
is retained for a later compatibility comparison; it is never evidence for an
operand or for the produced expression type.

This is not general addition typing, general expression inference, typed Core
completeness, a new diagnostic family, Hum IR, backend input, execution,
optimization, effect/ownership/resource checking, or a safety proof.

## Baseline corpus classification

The structured corpus audit used `hum core-lower --format json`, never text
matching, to inspect all 229 `.hum` files. Exactly these three current roots
match the target shape:

| File and authenticated row | Signature and current facts | Required class |
| --- | --- | --- |
| `examples/core/minimal_add.hum`, task `add`, return line 5 | both exact references resolve to current-task parameters; accepted `Int` declarations; declared `Int` result | `Supported` |
| `examples/core/add.hum`, task `add`, return line 16 | both exact references resolve to current-task parameters; accepted `Int` declarations; declared `Int` result | `Supported` |
| `fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum`, task `payload`, return line 40 | both exact references resolve to current-task parameters; locally complete reserved `UInt` declarations; public type-check rows are globally blocked by the unrelated unresolved `values` reference at line 87 | `AuthenticatedOutOfScope` |

There is no current-corpus `IntegrityFailure`. Every other source expression is
`Noncanonical` for this unit. `examples/core/add.hum` still has an unrelated
unsupported test row, so its full-type command remains exit 1 even though its
task return becomes a supported accepted row.

The UInt witness is a hard compatibility boundary. Against the baseline debug
binary, its direct-process contracts are:

| Command surface | Exit | stdout bytes / SHA-256 | stderr bytes / SHA-256 |
| --- | ---: | --- | --- |
| `core-lower` human | 0 | 11,378 / `7c27ce1b320ecf24611a64ff356401cc2f129ae3f8bc8e8870e1efa38c7f69cf` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `core-lower --format json` | 0 | 75,957 / `ae44f177408c059db415bda2b53ac36dd3fbc07c44ac75e059752a5d037d561b` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `core-verify` human | 0 | 1,826 / `4ecf43a884856ff090ea4ee1d90d3e3c640295f47320d54bb0c58d136664fe3f` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `core-verify --format json` | 0 | 344,937 / `94549e1a3a314fa497e45ebb63ebb5852e57affd3e4d36f1cd434a7f99ce4ac5` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `full-type-check` human | 1 | 15,680 / `13aef5de9315cac1c5f600d80e73e251f6cc8a7e1b4c84d4d5a4f88ea1a6c0c3` | 2,534 / `f66275a8c20ef98ec444c6ef96b892cf8e92498034ba7ca871743ffdc0194cb5` |
| `full-type-check --format json` | 1 | 60,879 / `47bff4d4dcaedb53be8b0d1cf158f93192993d08dfec829b5ebbd33f6c9ca987` | 0 / `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

The implementation must reproduce all twelve byte counts/hashes and all six
exit codes exactly after rebuilding `target/debug/hum.exe`. Capture stdout and
stderr as raw process bytes; PowerShell pipeline re-encoding earns no credit.
The fixture's existing Core-lower structured tree, unchecked outer type,
existing verifier checks/counts/order/IDs/details, full-type blocked row,
diagnostics, and callable/predicate material must remain byte-identical.

## Exact four-way classifier

`src/type_check.rs` owns the only batch producer:

```rust
pub(crate) fn canonical_minimal_add_type_classifications(
    program: &Program,
    diagnostics: &[Diagnostic],
) -> CanonicalMinimalAddTypeClassifications
```

It is called exactly once for each construction of a Core-lower artifact. It
uses the live `Program`, the same diagnostic slice as lower, one internal
`TypeCheckReport` built from those inputs, resolver summaries built from those
inputs, and the validated parser-owned task signature and canonical body tree.
No caller supplies component rows or reconstructs a classification.

The private closed classification is:

```rust
enum CanonicalMinimalAddTypeClassification {
    Supported(CanonicalMinimalAddTypeAuthority),
    AuthenticatedOutOfScope(CanonicalMinimalAddOutOfScope),
    IntegrityFailure(CanonicalMinimalAddIntegrityFailure),
    Noncanonical,
}
```

The batch retains one deterministic target record per task return encountered
in parser traversal order. Absence of the exact parser-owned target shape is
`Noncanonical`; it is never inferred from path, task name, statement text,
public row ID, or vector position. The three payload types and batch have no
`Serialize`, `Deserialize`, `Default`, or public conversion. The supported
authority and later verified view additionally have no `Clone` or `Copy`.
Private read-only accessors expose only the fields required by lower and verify.

Classification occurs in this precedence order:

1. Recognize a task `return` whose retained parser root is exactly
   `Binary(Add, Identifier, Identifier)` with two ordered parser children.
   Every other task/statement/expression is `Noncanonical`.
2. Authenticate the owning parser source revision, file/item traversal, task
   signature snapshot, live task projection, statement/root/child identities,
   ranges, operator, roles, and spellings. A recognized root whose signature
   authority is absent or mismatched is `IntegrityFailure`.
3. Join each child independently to exactly one `resolved_v0` resolver
   reference by private canonical node identity and child position. Require
   exact spelling/span, the current task's private semantic scope, exactly one
   target, and a target definition that is an authenticated parameter of that
   task. The two children may legitimately reference the same parameter; row
   duplication or ambiguity is still forbidden. Any missing, extra, ambiguous,
   foreign, wrong-kind, same-spelled-foreign, or substituted relationship is
   `IntegrityFailure`.
4. Determine each operand's type first from the authenticated parser signature
   parameter selected by that resolver relationship. Only then inspect
   declaration rows. This ordering is the downgrade guard: if both authenticated
   operand annotations are `Int`, no later mutation to resolver/type/public facts
   may reclassify the target as out of scope.
5. Join each selected definition to exactly one internal type-environment
   parameter declaration and exactly one checked declaration. Require exact
   declaration/definition/owner/kind/name/span/type-syntax/type-reference links.
   Also require one exact result declaration matching the authenticated result
   syntax. Missing, duplicate, unknown, rejected, inconsistent, or substituted
   local facts are `IntegrityFailure`.
6. If both authenticated operand types are exactly `Int`, require each selected
   checked declaration to be `accepted_declaration_annotation_v0`, every checked
   type reference to be `accepted_type_reference_v0`, and checked type text to
   be exactly `Int`. The result declaration must also be locally complete and
   accepted, but its type may differ. Success is `Supported` and independently
   produces expression type `Int`; any failure is `IntegrityFailure`.
7. If at least one authenticated operand type is not `Int`, the target may be
   `AuthenticatedOutOfScope` only when all selected resolver relationships are
   exact and all underlying `TypeDeclaration` rows are locally complete and
   consistent with the authenticated signature. Reserved roots require
   `reserved_type_annotation_v0` and `reserved_type_v0`; declared roots require
   their corresponding existing locally resolved facts. Globally blocked
   `CheckedDeclaration` rows may corroborate identity/linkage but are explicitly
   not type authority. Unknown/rejected/local-incomplete rows yield
   `IntegrityFailure`, never out of scope.

The current UInt fixture reaches step 7: its parameter and result declarations
are locally exact `UInt` / `reserved_type_annotation_v0` with
`reserved_type_v0` references. Its public checked rows remain
`not_checked_blocked_by_prior_errors_v0` / `not_checked_prior_errors_v0` because
of the unrelated H0601. The classifier uses those blocked rows only to verify
that the expected linked rows exist; it publishes and authorizes nothing from
them.

An authenticated `Int` target with a checked row coherently changed to `UInt`,
with public and private projections changed together, remains
`IntegrityFailure`. An independently parsed real `UInt` program can be
`AuthenticatedOutOfScope` for its own source revision, but its classification
cannot validate an artifact from the original `Int` revision.

## Parser signature authority

`CanonicalCoreOwnerBinding` gains an optional private
`CanonicalTaskSignatureSnapshot`, populated only for tasks through the existing
parser owner-witness issuance path. The snapshot retains:

- source revision plus exact file/item traversal and task item identity;
- ordered parameter count and ordinal;
- raw parameter spelling, permission token and explicitness, separator/type
  whitespace validity, name span, complete `TypeSyntax`, and complete type span;
- result-arrow token presence/range, raw result spelling, and complete result
  syntax/range; and
- the task header span needed to reject a foreign or relocated signature.

The AST exposes one fallible `pub(crate)` validated task-signature accessor. It
rechecks program traversal, owner file/path/kind/section slots, the optional
snapshot, and exact equality with live `Task.params`, `Task.result`, and
`Task.result_syntax`. Missing, duplicate, extra, reordered, respelled,
repermissioned, reranged, rewhitespaced, retyped, foreign-task, or
foreign-revision material returns an integrity error.

This accessor is separate from the existing
`CanonicalCoreSectionExpectation::validate` path. Adding or corrupting the
signature snapshot must not make established Core-body analysis panic before
the classifier can emit `IntegrityFailure`; existing section validation keeps
its exact behavior. There remain exactly four `parser_issue` issuer families:
file witness, owner witness, section capability, and parse context. The
snapshot adds no issuer, public field, schema row, serializable authority, or
second parser.

## Private ownership, candidate, and verified-view model

The exact lower owner is:

```rust
pub(crate) struct CanonicalMinimalAddLowering {
    report: CoreLowerReport,
    classifications: CanonicalMinimalAddTypeClassifications,
}
```

`core_lower` provides construction both with its ordinary preview authority
and with the existing caller-supplied preview authority used by Core verify.
Each construction calls the classifier batch producer once, owns its untouched
results beside the report, and lends only immutable borrows while lowering.
Text/JSON/readiness serialization can see only `report`. No API may extract,
clone, replace, serialize, or rebuild a supported authority from that report.

For `Supported`, lower projects the exact public typed fields below and retains
a distinct private `CanonicalMinimalAddTypeClaim` on the recognized operation.
For `IntegrityFailure`, lower projects the all-null unavailable form and retains
an incomplete claim whose optional fields describe only what was actually
available. No partial binder value is public. `AuthenticatedOutOfScope` and
`Noncanonical` retain no type claim and execute the exact pre-unit lower path.
The claim is candidate material, never authority; it has no public constructor,
accessor, serialization, or conversion. A test-only corruption seam may alter
the claim or public projection after construction but may not alter/mint the
untouched classifier batch or a verified view.

The production supported verifier has this ownership shape:

```rust
fn verify_canonical_minimal_add_type<'artifact, 'authority>(
    item: &'artifact CoreLowerItem,
    operation: &'artifact CoreLowerOperation,
    authority: &'authority CanonicalMinimalAddTypeAuthority,
    checks: &mut Vec<CoreVerifyCheck>,
) -> Option<VerifiedCanonicalMinimalAddType<'artifact, 'authority>>
```

Dispatch starts from the untouched private classification and matches source
revision, file/item traversal, statement/root identity, never candidate text,
names, public IDs, or ordinal alone. For a supported classification the
verifier compares independently:

1. the public item signature, operation/expression slots, structured root,
   ordered children, resolver IDs, definition IDs, checked-declaration IDs, and
   checked types; and
2. every private claim field

against the separately retained producer authority. Missing, duplicate, extra,
public-only substitution, private-only substitution, coherent public/private
co-substitution, foreign authority, and cross-revision authority all fail. For
an integrity classification, all three authority checks below fail and no
supported verifier constructor is called. Out-of-scope and noncanonical rows
emit no new authority checks.

`VerifiedCanonicalMinimalAddType<'artifact, 'authority>` has private fields
containing `&'artifact CoreLowerItem`, `&'artifact CoreLowerOperation`, and
`&'authority CanonicalMinimalAddTypeAuthority`. It owns nothing, has no
`Clone`, `Copy`, `Serialize`, `Deserialize`, or `Default`, and has no public,
owned, test-only, deserialization, or `'static` constructor. Its sole
constructor is the success branch in the production verifier after every
required structural, public, private, and authority check passes.

The only full-type handoff is:

```rust
pub(crate) fn with_verified_canonical_minimal_add_types<R>(
    program: &Program,
    diagnostics: &[Diagnostic],
    consume: impl for<'artifact, 'authority> FnOnce(
        &[VerifiedCanonicalMinimalAddType<'artifact, 'authority>],
    ) -> R,
) -> (CoreVerifyReadinessSummary, R)
```

The function owns `CanonicalMinimalAddLowering`, performs production Core
verification, builds views only for successful supported targets, invokes the
higher-ranked callback while both owners live, and drops all views before the
owners. `full_type_check::build_report` is the only production consumer. It
uses the already available canonical body report to match the view's
parser-owned statement/root identity and constructs affected `TypedStatement`
rows inside the callback. It receives no constructor, authority, claim, lower
owner, owned view, public-report substitute, or text/name/ordinal lookup path.

## Closed outcome matrix

`typed` below means `checked_canonical_minimal_add_v0` / `"Int"` /
`canonical_minimal_add_type_authority_v0`. `unavailable` means
`canonical_minimal_add_type_unavailable_v0` / `null` / `null`. `prior bytes`
means every pre-unit human/JSON byte and exit behavior for that row is retained.
`blocked` full type means `expression_text`, `expected_type`, `actual_type`, and
`type_source` are null, `status` is
`not_checked_blocked_by_prior_errors_v0`, and `reason` is
`source_resolver_type_or_core_verify_errors`.

| Input classification/corruption | Lower | Verify | Full type | Old additive fallback |
| --- | --- | --- | --- | --- |
| Supported exact `Int` operands, compatible declared result | `typed`, complete child binder projection, private claim | replacement outer-state rule and three authority rules pass; one view | accepted, authenticated expected type, actual `Int`, verified source | unreachable |
| Supported exact `Int` operands, incompatible but accepted declared result | same `typed` authority; result never proves actual type | all checks pass; one view | `rejected_statement_type_mismatch_v0`, authenticated expected, actual `Int`, mismatch reason | unreachable |
| Recognized target with absent/mutated signature authority | `unavailable`; all new child fields present null | replacement outer-state rule passes; three authority rules fail; no view | `blocked` | unreachable |
| Missing/ambiguous/foreign resolver relationship | `unavailable`; null binder fields | same fail/no-view outcome | `blocked` | unreachable |
| Missing/duplicate/rejected/mismatched Int declaration | `unavailable`; null binder fields | same fail/no-view outcome | `blocked` | unreachable |
| Int signature with checked/public/private UInt substitution | `unavailable` or corrupted candidate; never out of scope | affected comparison rules and issuance fail against untouched class/authority | `blocked` | unreachable |
| Supported candidate with public-only, private-only, or coherent co-substitution | candidate may still display `typed` | corresponding public/private rules and issuance fail | `blocked`; candidate is never accepted | unreachable |
| Authenticated locally coherent non-Int operand target | exact prior bytes; no new child fields/claim | exact prior checks/count/order/IDs/details; no view | exact prior row and command behavior | reachable exactly as before |
| Exact shape with non-Int but locally incomplete/unknown/substituted facts | `unavailable`; null binder fields | three rules fail; no view | `blocked` | unreachable |
| Noncanonical task return or same expression in another context | exact prior behavior | exact prior behavior, subject only to global ordinal shifts after an earlier supported target in the same aggregate report | exact prior behavior | reachable exactly as before |

A recognized target is identified from parser structure before authority lookup.
`IntegrityFailure` therefore cannot fall back to text inference. No supported or
integrity row may call `infer_additive_expression_type`, `split_once(" + ")`,
`place_type_fact`, `task_returns.get`, or recursive name/spelling/result
inference to obtain its actual type. The old branch remains in source and is
reachable for out-of-scope and noncanonical behavior exactly where it was
before.

## Exact `hum.core_lower.v0` projection

No top-level field, command, schema family, report, cache, manifest, diagnostic,
or public authority is added. Existing human Core-lower rendering remains
unchanged. JSON changes only for `Supported` and `IntegrityFailure` operations.
The operation is selected privately by authenticated task/statement/root
identity, never file path, task name, text, public ID, or ordinal.

For a supported operation, existing
`$.core_items[i].operations[j].expression` fields have:

- `type_status`: string `checked_canonical_minimal_add_v0`;
- `type_text`: string `Int`;
- `type_source`: string `canonical_minimal_add_type_authority_v0`.

For an integrity operation those same always-present fields are, respectively,
`canonical_minimal_add_type_unavailable_v0`, `null`, and `null`. The existing
structured expression remains present in both cases.

Each of its exactly two existing child objects conditionally appends these four
fields, in this exact order, after existing `identifier`:

| Child-relative JSON field | Supported type/value | Integrity type/value |
| --- | --- | --- |
| `resolver_reference_id` | string, exact resolver row ID | null |
| `resolved_definition_id` | string, exact target definition ID | null |
| `checked_declaration_id` | string, exact accepted checked-declaration ID | null |
| `checked_type` | string `Int` | null |

Existing child field order remains `index`, `role`, `parser_node_id`,
`source_range`, `kind`, `identifier`; then the four fields above. Child order
remains index 0 / `left`, then index 1 / `right`. `identifier` remains the
parser-owned spelling. The four fields are all present together with complete
values or all present together as null; partial projection is forbidden.

For `AuthenticatedOutOfScope` and `Noncanonical`, the four new fields are
absent, not null, and the old expression slots and serializer branches run
unchanged. Private classification, authority, and claim are always absent from
human and JSON output, not represented by null.

## Exact `hum.core_verify.v0` projection

No public field is added. Existing checks before the final outer-type rule keep
their exact order and meaning. For out-of-scope and noncanonical structured
rows, the existing final rule remains byte-for-byte:

- scope `structured_expression`;
- rule `structured_expression_outer_type_unchecked`;
- detail `structured add preserves the authoritative unchecked outer type state`.

For a supported or integrity target only, that one ordinal is instead:

- scope `structured_expression`;
- rule `structured_expression_outer_type_matches_canonical_minimal_add_classification`;
- detail `structured expression outer type state matches canonical minimal-add classification`;
- status `passed_v0` only when all three outer slots equal the exact supported
  or integrity projection required by its untouched classification.

Exactly three consecutive checks then follow immediately for that operation:

1. `canonical_minimal_add_public_projection_matches_authority`;
2. `canonical_minimal_add_private_claim_matches_authority`;
3. `canonical_minimal_add_verified_view_issued`.

Each uses the existing check object and field order: `id`, `scope`, `scope_id`,
`source_span`, `status`, `rule`, `detail`. `scope` is `operation`, `scope_id` is
the existing operation ID, `source_span` is the existing non-null operation
span, and `id` continues the deterministic global check ordinal. Success
details are exactly:

- `canonical minimal-add public projection matches untouched producer authority`;
- `canonical minimal-add private claim matches untouched producer authority`;
- `canonical minimal-add verified view issued from successful checks`.

Failure details are exactly:

- `canonical minimal-add public projection does not match untouched producer authority`;
- `canonical minimal-add private claim does not match untouched producer authority`;
- `canonical minimal-add verified view withheld after failed or missing authority check`.

An integrity classification has no supported authority, so all three rows fail.
Public corruption fails the first and third; private corruption fails the
second and third; simultaneous corruption fails all three. Any failed new row
contributes to existing counts, makes the root
`verification_status` `core_artifact_verification_failed_v0`, makes the owning
item `core_artifact_item_verification_failed_v0`, and makes the CLI exit 1
through the existing `core_verify_has_errors` route. No private identity,
authority, classification reason, claim, or verified view is serialized.

An out-of-scope target emits no replacement rule and no new rule: it executes
the exact old structured-verifier calls, preserving the UInt fixture's check
count, IDs, ordering, details, statuses, summary, and exit 0 byte-for-byte.

## Exact `hum.full_type_check.v0` projection

No public field is added. The affected existing row is
`$.typed_items[i].statements[j]`, selected by the verified view's authenticated
item/statement/root identity. Existing item and statement order and all field
order remain unchanged.

For a successfully verified supported target:

- `statement_kind` remains string `return`;
- `expression_text` remains the existing parser-carried display string;
- `expected_type` is the authenticated, locally accepted declared-result type;
- `actual_type` is string `Int` from the verified view;
- `type_source` is string `verified_canonical_minimal_add_type_v0`;
- `status` is `accepted_statement_type_v0` when compatible or
  `rejected_statement_type_mismatch_v0` when incompatible;
- `reason` is null on acceptance or
  `statement_expression_type_mismatch` on incompatibility; and
- unrelated failure/call fields retain their exact existing null values.

For an integrity target, failed Core verification activates the existing global
prior-error path: `expression_text`, `expected_type`, `actual_type`, and
`type_source` are present null; `status` is
`not_checked_blocked_by_prior_errors_v0`; `reason` is
`source_resolver_type_or_core_verify_errors`. The view/authority/claim are
absent from JSON.

For out-of-scope and noncanonical targets, the old full-type branch is unchanged.
In particular, the UInt fixture remains globally blocked by its existing
resolver error and retains its exact row bytes. `hum.type_check.v0`,
`hum.core_preview.v0`, `hum.ir_readiness.v0`, capabilities, version, execution
readiness, and IR readiness are unchanged.

## Exact ten-path writable envelope

Implementation may modify exactly these ten paths and no others:

| Path | Required role |
| --- | --- |
| `src/ast.rs` | Add the private optional task-signature snapshot and fallible validated accessor without changing general section validation. |
| `src/parser.rs` | Populate the snapshot through the existing owner witness; retain exactly four issuer families; host signature-substitution evidence/audits. |
| `src/type_check.rs` | Solely build the four-way batch and supported authority from parser, resolver, type-environment, and checked-declaration facts; keep public type-check output unchanged. |
| `src/core_expr.rs` | Add only the two bounded outer status literals and two source/provenance literals used by lower/full type. |
| `src/core_lower.rs` | Own classifications beside report, call the producer once, project supported/integrity rows, retain private claims, preserve out-of-scope bytes, and host lower evidence. |
| `src/core_verify.rs` | Compare public/private candidates with untouched classification/authority, solely construct/lend verified views, preserve out-of-scope checks, and host corruption/lifetime evidence. |
| `src/full_type_check.rs` | Consume views inside the HRTB callback, match parser identity, and bypass old inference only for supported/integrity targets. |
| `docs/HUM_CORE_LOWER_SCHEMA.md` | Document only the conditional lower projection and byte-preserved out-of-scope branch. |
| `docs/HUM_CORE_VERIFY_SCHEMA.md` | Document only the conditional verifier rules and private boundary. |
| `docs/HUM_FULL_TYPE_CHECK_SCHEMA.md` | Document only verified-view consumption, integrity blocking, and fallback preservation. |

Explicitly excluded are `src/core_body.rs`, `src/resolve.rs`, `src/type_env.rs`,
`src/core_preview.rs`, `src/ir_readiness.rs`, `src/main.rs`, `Cargo.toml`, all
tools, fixtures, examples, snapshots, generated files, and every other path.
The existing public and `pub(crate)` facts in the excluded Rust modules are
sufficient. Tests and source audits live module-locally in the authorized Rust
files. The accepted examples and UInt fixture are production evidence and must
not be edited.

If an eleventh path proves indispensable, stop before editing and report the
exact producer/validator/consumer dependency. The envelope cannot be expanded
inline. No replacement IR-readiness surface is needed; readiness remains zero.

## Five stable focused selectors

The implementation must create exactly these selectors:

1. `parser::tests::canonical_task_signature_authority_rejects_substitution`
2. `type_check::tests::canonical_minimal_add_type_authority_is_unique_and_bound`
3. `core_lower::tests::typed_minimal_add_consumes_untouched_canonical_authority`
4. `core_verify::tests::typed_minimal_add_verifier_rejects_authority_corruption`
5. `full_type_check::tests::minimal_add_consumes_only_verified_canonical_type`

For each, the exact-selector helper must list exactly one nonzero test and run
exactly one passing test:

```powershell
$Cargo = (Get-Command cargo -ErrorAction Stop).Source
. .\tools\test_exact_rust_selector.ps1
$selectors = @(
  'parser::tests::canonical_task_signature_authority_rejects_substitution',
  'type_check::tests::canonical_minimal_add_type_authority_is_unique_and_bound',
  'core_lower::tests::typed_minimal_add_consumes_untouched_canonical_authority',
  'core_verify::tests::typed_minimal_add_verifier_rejects_authority_corruption',
  'full_type_check::tests::minimal_add_consumes_only_verified_canonical_type'
)
Reset-ExactRustSelectorCredits
foreach ($selector in $selectors) {
  Invoke-ExactRustTest "Work Order 13 Unit 1: $selector" $Cargo $selector
}
$credits = @(Get-ExactRustSelectorCredits)
if ($credits.Count -ne 5) {
  throw "expected 5 exact selector credits; got $($credits.Count)"
}
```

Collectively, using production paths and parser-derived identities, they prove:

- the exact 229-file corpus inventory has the two supported Int targets, one
  authenticated out-of-scope UInt target, and no other exact target;
- clean supported lower/verify/full-type output, result compatibility and
  mismatch, deterministic human/JSON, and one classifier call per lower owner;
- every named signature/resolver/declaration integrity failure, including
  missing, duplicate, ambiguous, foreign, wrong-kind, same-spelled, swapped,
  rejected, blocked Int, and link/type substitutions;
- classification precedence: an authenticated Int target cannot downgrade to
  out of scope after checked/public/private UInt substitution;
- the UInt fixture's six exits and twelve raw byte/hash contracts above;
- lower never publishes partial binder facts and refuses `typed` without a
  supported authority;
- public-only, private-only, coherent public/private co-substitution, and an
  authority from a separately parsed substituted program all reach the real
  verifier and fail against untouched ownership;
- no failed/missing/partial check issues a view, and full type cannot accept a
  candidate or public report in place of a view;
- statement-text sabotage cannot affect target selection or actual type;
- old additive/name/result inference is unreachable for supported/integrity
  targets and remains reachable for the out-of-scope/noncanonical controls; and
- existing structural corruptions still reach production verification without
  overflow, wrap, panic, or a parallel validator.

Module-local source audits fail on a second classifier/authority/view
constructor, more than one classifier call in a lower construction, a
full-type producer call, authority/view cloning or serialization, report/JSON
reconstruction, a fifth parser issuer, target matching by path/name/text/public
ID/ordinal, result annotation used as actual-type evidence, targeted old
fallback, owner leaks, or signature-validation bypass. Hard-coded fixture
identities are forbidden; literal fixture paths are allowed only for loading
the named regression input.

## Executable lifetime proof on the production type

`src/core_verify.rs` contains these three cfg-selected functions beside the
actual production type, with no stand-in type:

```rust
#[allow(unexpected_cfgs)]
#[cfg(hum_compile_fail_verified_canonical_minimal_add_escape)]
fn verified_canonical_minimal_add_cannot_outlive_lower_artifact<'artifact, 'authority>(
    view: VerifiedCanonicalMinimalAddType<'artifact, 'authority>,
) -> VerifiedCanonicalMinimalAddType<'static, 'authority> {
    view
}

#[allow(unexpected_cfgs)]
#[cfg(hum_compile_fail_verified_canonical_minimal_add_escape)]
fn verified_canonical_minimal_add_cannot_outlive_producer_authority<'artifact, 'authority>(
    view: VerifiedCanonicalMinimalAddType<'artifact, 'authority>,
) -> VerifiedCanonicalMinimalAddType<'artifact, 'static> {
    view
}

#[allow(unexpected_cfgs)]
#[cfg(hum_compile_fail_verified_canonical_minimal_add_escape)]
fn verified_canonical_minimal_add_cannot_become_owned_static_authority<'artifact, 'authority>(
    view: VerifiedCanonicalMinimalAddType<'artifact, 'authority>,
) -> VerifiedCanonicalMinimalAddType<'static, 'static> {
    view
}
```

They change only the relevant return lifetimes of the real view. All three must
fail for lifetime/borrow reasons even inside the defining module; privacy is
not the proof. Run this exact expected-failure check from the repository root:

```powershell
$priorRustFlags = $env:RUSTFLAGS
try {
  $env:RUSTFLAGS = '--cfg hum_compile_fail_verified_canonical_minimal_add_escape'
  $proof = cargo check --all-targets 2>&1 | Out-String
  $proofExit = $LASTEXITCODE
} finally {
  if ($null -eq $priorRustFlags) {
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue
  } else {
    $env:RUSTFLAGS = $priorRustFlags
  }
}
if ($proofExit -eq 0) { throw 'verified view escape unexpectedly compiled' }
$needles = @(
  'verified_canonical_minimal_add_cannot_outlive_lower_artifact',
  'verified_canonical_minimal_add_cannot_outlive_producer_authority',
  'verified_canonical_minimal_add_cannot_become_owned_static_authority',
  'lifetime may not live long enough'
)
foreach ($needle in $needles) {
  if (-not $proof.Contains($needle)) {
    throw "missing compile-fail evidence: $needle"
  }
}
```

Normal checks run without that cfg and must compile. The proof adds no
dependency, Cargo edit, generated source, permanent artifact, toy type,
privacy-only failure, or eleventh path. A source-text search cannot substitute
for compiler failure. If any attempted escape compiles or fails for a reason
other than the real lifetime relationship, stop.

## Implementation validation and acceptance

The implementation candidate runs the five exact selectors, the expected
compile failure, direct human/JSON commands for all three corpus targets, each
JSON form twice byte-identically, and the UInt raw-byte contract. It then runs:

```powershell
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
.\tools\check_release_readiness.ps1
```

The root Rust suite runs once locally. No local Fast, Exhaustive, actor
transcript, validation ledger, performance pair, or `tools/check_all.ps1` run
is required. The unchanged UInt contract and module tests protect the standing
harness expectation; required post-publication CI owns the full preflight and
must reach terminal success independently on Ubuntu and Windows for the exact
published commit.

After implementation, a fresh independent architect-reviewer inspects the
complete diff and real producer/validator/consumer route, repeats focused and
adversarial probes, verifies the ten-path inventory, and returns one verdict:
`ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or `REJECT`, with P0/P1/P2 findings. One
bounded in-envelope implementation correction may be authorized after review;
it may not alter classification, public contracts, authority ownership, byte
preservation, evidence meaning, or the envelope. A second non-`ACCEPT`, an
indispensable eleventh path, or a semantic redesign stops for the BDFL.

## Explicit bans and stop conditions

This order does not authorize:

- treating a globally blocked declaration as supported type authority;
- treating a genuine authenticated non-Int target as corruption merely to
  avoid preserving its established output;
- allowing an Int target to downgrade to out of scope after substituted
  resolver, declaration, candidate, public, or private facts;
- a second additive inference, moving a full-type result backward, or using
  source text, spelling, a name environment, declared result, public JSON, or a
  candidate claim as produced expression-type authority;
- independent `Int` conclusions in lower/verify, self-consistency without an
  untouched producer comparison, cloned/serialized authority, owned/`'static`
  views, owner leaks, or access after failure/drop;
- hard-coded production paths, task/parameter names, ordinals, ranges, parser
  nodes, resolver/definition/declaration IDs, or current corpus counts as the
  selector mechanism;
- another operator/shape/context, overload, literal coercion, import, generic,
  trait, call, field, effect, ownership/resource/profile, IR, backend,
  execution, optimization, or safety/performance claim;
- new H-code, dependency, Cargo feature, command, schema family, top-level
  object, fixture, example, tool, cache, ledger, manifest, transcript,
  generated file, or validation framework; or
- subunits, deferred schema/test/format work, archived-code recovery, an
  eleventh path, or an inline scope amendment.

Stop for an unauthenticated signature, producer cycle, authority/candidate
collapse, parallel same-shape inference, out-of-scope byte drift, a view that
escapes, a selector that lists zero/multiple tests, corruption that does not
reach production validation, broader redesign, or inability to finish the
coherent unit in one review-sized implementation session.

## Independent issuance and publication gates

The next actor is a fresh cold-start architect-reviewer who did not author,
edit, generate, or direct this draft. The reviewer must inspect the complete
two-document package (`WORKORDER_12.md` frozen plus this raw untracked file),
repeat the code/corpus satisfiability audit, confirm the four-way precedence
and UInt byte-preservation contract, verify exact ten-path closure, and issue
one pre-issuance verdict with P0/P1/P2 findings.

Only an unqualified `ACCEPT` can return to the BDFL for possible issuance.
Anything else stops; it grants no edit. A further document revision requires a
new explicit BDFL authoring signal and a fresh independent review. Review
acceptance itself authorizes no implementation, staging, commit, push, archive
mutation, or remote mutation.

The BDFL must separately accept the exact document bytes, authorize any scoped
document commit, authorize publication, observe required terminal CI/status
evidence, and issue a separate Unit 1 implementation signal. Implementation
acceptance later authorizes only a separately signaled scoped implementation
commit; push and all remote changes remain separately reserved.

## Re-scope author document checks

The author runs only these checks on the fresh document package:

```powershell
git diff --check
git -c core.autocrlf=false -c core.safecrlf=false diff --no-index --check -- NUL WORKORDER_13.md
.\tools\test_workorder_status_boundary.ps1
.\tools\test_workorder_status_boundary.ps1
.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
.\tools\check_release_readiness.ps1
```

The NUL command must exit exactly 1 with zero output. Each status-boundary run
must exercise all 123 cases with zero failures, and the two complete captured
outputs must be byte-identical. No Cargo command, Rust selector, lifetime
proof, target command, Fast, Exhaustive, `tools/check_all.ps1`, implementation
validation, or performance measurement is an authoring check.

## Current authorization gate

Work Order 13 is issued, uniquely active, published, and terminal-green at
`ec2c3e02d1e1a02ad7e5c83331454b8f534e3490`. Unit 1 remains explicitly
unauthorized pending a separate BDFL implementation signal. No implementation,
repair, archive mutation, or later work is authorized by this status record.
<!-- workorder-current-authorization-gate:end -->
