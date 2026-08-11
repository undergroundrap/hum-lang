# Hum Work Order 18: Bind Exact Minimal-Add Backend Facts At IR Readiness

Date: 2026-08-10
<!-- hum-active-workorder:v1 -->
Status: Published planning package; Unit 1 is stopped for a material
implementation-envelope defect, and the remedial amendment below is under
review. The exact independently reviewed planning documents were committed as
`2f3131b8866703ecc40ae81cdce0458f93320ce2`, with parent
`6960a340263f1a032c17df50c3048e9b88cf9041` and subject
`docs(workorder): bind backend facts at ir readiness`, then published to
`origin/main` by normal fast-forward. Required full-lane CI passed in workflow
`ci`, run `31440656283`, attempt 1, with overall conclusion `success` and
`mode=full;reason=no_status_transition` on both required platforms. Ubuntu job
`93624472019` succeeded in 18m17s with a 17m39s full preflight; Windows job
`93624471931` succeeded in 26m58s with a 26m36s full preflight. Ubuntu passed
the Exhaustive producer at 14,226 pairs with seed `0x48554D5F5345414C`;
Windows correctly skipped the duplicate. Text hygiene and public readiness
passed for 526 files, alpha claims passed, and release readiness passed for
version `0.0.1`. Publication was then recorded in status-only commit
`a3b60708524edfc5631b8a6617ac6b2740fbeafe`, with parent
`2f3131b8866703ecc40ae81cdce0458f93320ce2` and subject
`docs(workorder): record work order 18 publication`. Required fast-lane CI
passed in workflow `ci`, run `31445562441`, attempt 1, with overall conclusion
`success`. Ubuntu job `93638901547` and Windows job `93638901617` both
reproduced `mode=fast;reason=eligible_status_chain`, anchored at
`2f3131b8866703ecc40ae81cdce0458f93320ce2`, for transition
`2f3131b8866703ecc40ae81cdce0458f93320ce2>a3b60708524edfc5631b8a6617ac6b2740fbeafe`.
The first Unit 1 candidate later stopped when final Fast proved that `README.md`
is a required durable consumer outside the original eleven-path envelope. That
exact stopped candidate is preserved locally and remotely at
`archive/workorder-18-unit1-envelope-stop-2026-08-10`, commit
`82c6f74a6392ad806b4ab1dd33960654595627fb`. Unit 1 remains stopped; archive
restoration, README editing, implementation, commit, and publication remain
unauthorized pending the separate gates below.

Owner: BDFL (Ocean).
Author: Work Order 18 architect-author. The author may not independently review
this document package or any implementation candidate produced under it.
Historical WO18 planning baseline: the published Work Order 17 closeout commit
`6960a340263f1a032c17df50c3048e9b88cf9041` was the `main` revision from which
WO18 planning began. It is not the current amendment baseline.
Current remedial-amendment and archived-candidate baseline: `HEAD`, local
`main`, cached `origin/main`, and live remote `main` are all
`a3b60708524edfc5631b8a6617ac6b2740fbeafe`.

## Closed predecessor and planning authority

Work Order 17 is accepted, implemented, published, closed, and terminal-green.
Its final closeout commit is
`6960a340263f1a032c17df50c3048e9b88cf9041`. Work Order 17 establishes, only
for the supported authenticated canonical `Int + Int` operation:

- one immutable source disposition produced by `src/type_check.rs`;
- exact ordered resolver references and distinct parameter definitions;
- a checked `Int` result attached to the exact Core operation and result value;
- independent Core verification of source authority and mutable projections;
- a report-global fail-closed access decision; and
- a lifetime-bound verified type result consumed by full type.

Work Order 16 remains the authority for Program-owned file/item/section and
operation order, exact expected-operation callbacks, private Core candidate
origins, and the compiler-enforced first-construction boundary. Neither Work
Order is reopened here.

Work Orders 13 and 15 are terminally rejected and archived. Their batch
classification, parallel records, later reassociation, candidate code, tests,
and schemas are failure evidence only. This Work Order may not recover, copy,
or treat them as authority.

## Mandatory sufficiency ruling

The preferred complete `hum.backend_input.v0` plus `hum.ir_verify.v0` result is
not yet an honest review-sized unit on the current tree. A smaller semantic
producer boundary is genuinely missing.

The current compiler has the following load-bearing facts for the supported
canonical operation:

| Required fact | Current production state |
| --- | --- |
| Canonical source revision, normalized path, and semantic file ordinal | Private parser/AST authority exists through the Work Order 16 Program-owned file binding. |
| Canonical item, section, statement, and operation order | Private Work Order 16 expected-operation authority exists. |
| Ordered expression root and children | Parser-owned canonical expression structure exists for the supported identifier-add shape. |
| Core preview | Production Core lowering consumes genuine `core_preview` authority; a rendered preview row is not authority for this prerequisite. |
| Resolver bindings | Work Order 17 type authority owns both ordered operand references and their distinct resolver definitions. |
| Checked result type and result value | Work Order 17 owns the exact `Int` type ID/text and result-value ID and Core verification exposes them through a borrowed verified result. |
| Full type | Full type consumes the verified Work Order 17 result and accepts the statement as `Int`. |
| Effect | The current effect report accepts the return as `pure_or_local` / `accepted_no_external_effect_v0`, but only as a detached report row. |
| Ownership | The current ownership report accepts `no_ownership_transfer`, but only as a detached report row. |
| Resource | `examples/core/minimal_add.hum` currently declares no allocation intent, so the current resource gate rejects it with `rejected_missing_allocation_declaration_v0`. |
| Profile | The default `normal` profile is recognized, but the current profile result is blocked when the resource gate fails. |
| Integer representation and overflow | Parser/runtime production code uses signed `i64`; the accepted language reference requires integer overflow to trap rather than wrap. No exact-operation backend-fact authority currently transports those semantics. |
| Contract/evidence set | The example has no predicate operation or evidence obligation, but no exact-operation checked-empty conclusion is carried to backend planning. |
| Canonical artifact bytes and digest | Absent. |
| `ir_verify` and `VerifiedBackendInput<'a>` | Absent. |

The existing JSON and human reports cannot be joined to repair these gaps.
They are observation projections, may be copied or coherently edited, are built
independently, and do not share one authority identity. Selection by filename,
task name, text, public ID, span alone, ordinal alone, or report position is
forbidden.

One dedicated producer can bind the semantic facts without a dependency cycle,
but only after genuine `core_preview` authority has been consumed by Core
lowering and the current post-type pass chain carries exact-operation private
authority through full type, effect, ownership, resource, and profile. A
rendered preview row cannot substitute for the production authority. The honest
construction stage is therefore IR readiness, after profile checking and before
`ir_verify`.

Artifact encoding and `ir_verify` must remain one later coherent unit. Splitting
an encoder from its byte verifier would deliberately create an authoritative-
looking, persisted, non-load-bearing intermediate. That later unit must encode
the facts established here, compute the canonical payload digest, parse and
verify exact bytes, and issue `VerifiedBackendInput<'a>` together. This Work
Order does none of those things.

The boundary selected here is the smallest genuine prerequisite:

```text
WO16 exact Program-owned operation
  -> WO17 verified checked type and resolver-use authority
  -> exact full-type handoff
  -> exact checked-empty effect authority
  -> exact checked-empty ownership authority
  -> exact checked-empty resource authority
  -> exact accepted-profile authority
  -> IR-readiness-owned canonical minimal-add backend facts
  -> lifetime-bound facts access
  -> IR-readiness corruption gate and blocked-before-ir_verify projection
```

This result is load-bearing because production IR readiness must consume it to
distinguish the exact candidate whose semantic backend facts are complete from
ordinary candidates that merely have aggregate reports. It must still report
`ir_ready=0`, keep `ir_verify` missing, and block backend entry.

## Unit 1 exact result

Unit 1 creates one private, operation-owned
`CanonicalMinimalAddBackendFacts` for each exact supported canonical operation
whose complete prerequisite chain succeeds. It exposes those facts only inside
a higher-ranked callback through
`CanonicalMinimalAddBackendFactsAccess<'facts>`.

The access is not `VerifiedBackendInput`, is not serializable, grants no
execution or lowering authority, and does not authenticate any persisted
bytes. It proves only that one current-compilation exact operation reached the
complete pre-`ir_verify` semantic boundary.

The final facts and access are private-field, non-`Clone`, non-`Copy`,
non-`Default`, non-serializable, and immutable after construction. Anti-forgery
and semantic completeness remain separate: private construction proves the
authorized producer route, while the complete fact inventory and corruption
matrix prove that the route checked enough. Neither property substitutes for
the other.

For `examples/core/minimal_add.hum`, Unit 1 also adds the explicit source intent
needed by the existing resource policy:

```text
  allocates:
    nothing
```

The source edit does not weaken resource policy. Missing allocation intent
continues to fail. The resource authority may issue only when this exact
declaration is present once, the existing resource check accepts it, and the
authenticated canonical operation independently has no allocation-capable
shape or call.

## Exact facts owned by the final private result

`CanonicalMinimalAddBackendFacts` owns or immutably borrows all of the
following. No field may be reconstructed from rendered output.

### Compiler and semantic context

- compiler package version `0.0.1` from production version authority;
- semantic contract `hum.canonical_minimal_add_backend_facts.v0`;
- target context `target_independent_checked_i64_v0`;
- active profile ID `normal`;
- fourteen-entry required pre-IR pass set in the exact order frozen below; and
- an explicit `ir_verify` state of `not_implemented`, outside the successful
  prerequisite set.

### Source and function identity

- canonical source-revision bytes, not a public path-derived surrogate;
- semantic file ordinal and normalized parser-owned path;
- declared module identity, with explicit `null` only when the source has none;
- Program identity used only as private current-process provenance;
- canonical item traversal and resolver-owned semantic item identity;
- item kind `task`;
- exact authenticated task signature;
- exact `does` section slot;
- exact source operation slot; and
- exact statement source-node identity and source range.

Display names may be projected for diagnostics, but they are never lookup keys
or authority.

### Function and expression facts

- one internal function record for the exact task;
- two ordered parameter ordinals, value IDs, type IDs, and source ranges;
- each parameter type `hum-type:builtin:Int` / `Int`;
- one exact return operation;
- canonical root node ID and result-value ID;
- operator discriminant `add`;
- two ordered operand child ordinals and node IDs;
- two distinct resolver definition IDs and semantic definition identities;
- each operand-use-to-definition relationship;
- checked result type `hum-type:builtin:Int` / `Int`;
- declared-result compatibility; and
- no additional function, block, operation, expression, or value record.

Parameter and operand value IDs must be deterministic producer-owned
identities derived from the authenticated semantic relationships, not from
names, spans, array position alone, or caller input.

### Checked-empty and nonempty semantic conclusions

The following states are distinct. Omission never means empty.

- `effects`: checked and empty for external authority; the exact effect row is
  `pure_or_local` / `accepted_no_external_effect_v0`;
- `ownership_transfers`: checked and empty; the exact ownership row is
  `pure_or_local` / `accepted_no_ownership_transfer_v0`;
- `allocations`: checked and empty for this closed operation after the exact
  `allocates: nothing` declaration and structural no-allocation recheck;
- `profile`: checked and accepted as the default `normal` profile;
- `contract_predicates`: checked and empty for this item;
- `evidence_obligations`: checked and empty for this item;
- `unsupported_or_weakened`: checked and empty;
- `external_authority`: checked and empty; and
- `failure_edges`: nonempty, containing exactly one typed signed-integer
  checked-add overflow trap edge.

The arithmetic facts are `signed_64`, `checked_add`, and
`runtime_trap_on_overflow`. They bind the current parser/runtime `Int` carrier
and accepted language-reference behavior for this narrow route. They do not
freeze a public native ABI, calling convention, result-slot ABI, Cranelift
instruction, target triple, object format, or runtime wrapper.

### Required pre-IR pass set

The final result binds these successful conclusions in order:

1. `parse`
2. `semantic_graph_build`
3. `resolve`
4. `body_grammar`
5. `core_preview`
6. `core_lowering`
7. `core_verify`
8. `type_check`
9. `full_type_check`
10. `effect_check`
11. `ownership_alias_check`
12. `allocation_resource_check`
13. `contract_evidence_linking_checked_empty_for_exact_item`
14. `profile_check`

`ir_verify` is never included as successful in this unit. A failed, skipped,
unchecked, absent, duplicate, foreign, reordered, or zero-selection required
conclusion withholds the final access.

`core_preview` is an independent required production conclusion in the exact
position above. Core lowering must have consumed its genuine authority. Its
presence in a rendered report, a copied status string, or an expected test
value is not authority and cannot satisfy this prerequisite.

## Producer chain and private authority boundaries

Every stage builds its own production report once for that invocation, carries
its normal summary and diagnostic occurrences forward unchanged, and issues a
private exact-operation authority only after its existing blocker precedence
and its new local checks succeed.

### Type and resolver identity extension

`src/type_check.rs` adds a private borrowed backend-identity view on
`CanonicalMinimalAddTypeAuthority`. It exposes to `src/core_verify.rs` only:

- root, statement, result-value, and checked-type facts already verified by
  Work Order 17;
- the ordered operand node identities;
- the ordered resolver definition IDs and semantic definition identities;
- parameter ordinals and authenticated source ranges; and
- declared-result compatibility.

It exposes no constructor, mutable reference, owned clone, public serializer,
or general type-query API. `src/core_verify.rs` makes that view available only
through the already delivered `VerifiedCanonicalMinimalAddTypeResult`.

### Exact report handoffs

The following crate-private function names and dependency direction are frozen:

```text
full_type_check::with_full_type_for_effect
effect_check::with_effect_for_ownership
ownership_check::with_ownership_for_resource
resource_check::with_resource_for_profile
profile_check::with_profile_for_ir_readiness
ir_readiness::with_canonical_minimal_add_backend_facts
```

Each uses an `impl for<'report> FnOnce(...Access<'report>) -> R` callback. The
access cannot escape, become `'static`, be collected, be serialized, or be
constructed by a sibling module. Each downstream authority retains private
current-process lineage to the exact upstream report and operation. Public
rows, summaries, IDs, names, spans, and indexes cannot substitute for lineage.

Exact lookup is by the live Program/item/statement references and retained
private source identity. A name/span/public-ID comparison may be an additional
integrity check but may not be the association mechanism.

### Full type

Full type issues its handoff only when the same report:

- received the Work Order 17 delivered verified type result;
- accepted the exact statement as `Int` from
  `verified_canonical_minimal_add_type_v0`;
- preserved the same Core-verifier readiness summary and diagnostic occurrence
  set; and
- has no local or report-global blocker.

### Effect

Effect consumes that handoff and issues only when the exact statement has one
accepted `pure_or_local` row, no external target or declaration, no rejected or
unchecked boundary row, and no report-global blocker. A public row rewritten to
look pure cannot mint authority.

### Ownership

Ownership consumes the effect authority and issues only when the exact
statement has one accepted no-transfer row, no move/borrow/alias/resource
transfer, no rejected or unchecked boundary row, and no report-global blocker.

### Resource

Resource consumes the ownership authority and issues only when:

- the exact task owns one normalized allocation-free declaration;
- the current resource row is accepted;
- the exact authenticated operation contains no call, record/list
  construction, allocation-capable operation, or unsupported shape;
- no other allocation claim competes with it; and
- no report-global blocker exists.

The existing `declared_not_proven` public reason remains honest. The private
authority is narrower: it proves checked-empty allocation only for this closed
canonical operation by combining the accepted declaration with exact
structure. It is not a general allocation-freedom proof.

### Profile

Profile consumes the resource authority and issues only when the exact task has
one accepted default `normal` profile conclusion, no explicit competing,
unknown, or strict profile declaration, and no report-global blocker.

### IR readiness

IR readiness consumes the profile authority, rechecks the authenticated source,
item, operation, ordered resolver/type facts, closed semantic conclusions, and
complete fourteen-entry required pass set--including genuine `core_preview`
authority in its production position--then constructs the sole final
`CanonicalMinimalAddBackendFacts`.

It must not parse any rendered report, call a second independent classifier,
or join independently emitted reports. Failure at any producer or final check
withholds access.

## Public IR-readiness contract

No new command or schema family is introduced. `hum.ir_readiness.v0` changes
only for an exact candidate that owns the final private facts.

For that candidate, human and JSON output must report:

```text
status=blocked_before_ir_verify_with_backend_input_facts_v0
missing_passes=[ir_verify]
blocking_reasons=[ir_verify_not_implemented]
```

Its `facts_available` array appends these literals in this exact order:

1. `canonical_minimal_add_backend_facts_v0`
2. `source_and_operation_identity_bound_v0`
3. `ordered_resolver_bindings_bound_v0`
4. `verified_checked_type_bound_v0`
5. `effect_checked_empty_v0`
6. `ownership_checked_empty_v0`
7. `resource_checked_empty_v0`
8. `normal_profile_checked_v0`
9. `checked_i64_overflow_trap_bound_v0`
10. `ir_verify_pending_v0`

The report summary remains `ready_for_ir=0`. Core lower, Core verify, full type,
effect, ownership, resource, and profile keep `execution_ready=0` and
`ir_ready=0`. No serialized field says `verified`, `backend_ready`, or
`lowerable`.

The `missing_passes=[ir_verify]` projection is available only after all fourteen
successful prerequisites have been authenticated. A missing, failed, blocked,
duplicate, foreign, or reordered `core_preview` conclusion withholds the new
status and facts; it retains the existing applicable failure and blocker
precedence rather than being collapsed into `ir_verify_not_implemented`.

Candidates without the final private facts retain their current status,
ordering, fields, null/omission behavior, and blocker precedence. A public
projection mutation can change rendered bytes in a corruption test but cannot
mint the private facts or the new candidate status.

## Remedial envelope-stop record and exact resumption base

The original eleven-path Unit 1 candidate was not rejected for semantic
behavior. Its evidence sequence stopped at three real boundaries:

1. The first completed Fast failure exposed a required second resource-stage
   Core-body consumer. The candidate retained both production consumers, and
   the Replacement F4 audit was strengthened to require exactly one report-
   builder call and one independent authority-issuance call.
2. The next completed Fast failure exposed obsolete Session AP production-
   source extraction. The candidate replaced that extraction non-vacuously,
   retained nested cfg items and later production code, rejected terminal-test
   spelling as production evidence, and added deletion and ambiguity controls.
3. The final Fast passed the root Rust tests, all four Work Order 18 selectors,
   the 99/99 exact-selector inventory, the compile-fail proof, Replacement F4,
   and Session AP. It then failed the unchanged README/fixture parity gate
   because the required `examples/core/minimal_add.hum` edit was not mirrored in
   the checked README block.

That last result is a material envelope defect, not a harness defect.
`README.md` is a real durable consumer of the edited source. The exact stopped
candidate is preserved and published at:

- branch: `archive/workorder-18-unit1-envelope-stop-2026-08-10`;
- commit: `82c6f74a6392ad806b4ab1dd33960654595627fb`;
- parent: `a3b60708524edfc5631b8a6617ac6b2740fbeafe`;
- complete tree: `05e60ef87b6a23b6a474323c25cdd5385f8c675c`; and
- scoped eleven-path tree:
  `f4b3594b1d4d745b7e8801b72ffa4a80204feabb`.

The archive is immutable evidence and a permitted exact byte source for a
later resumption. It is not semantic authority and must not be amended,
rebased, merged, or treated as an accepted implementation.

### Complete consumer audit and README synchronization

The complete tracked consumer audit found one synchronized public copy and no
additional implementation path:

- `README.md` contains the sole
  `<!-- hum-example:start examples/core/minimal_add.hum -->` block;
- the unchanged `Assert-ReadmeHumExamplesMatch` function in
  `tools/check_all.ps1` requires that marked code to be a contiguous substring
  of the referenced fixture;
- no second marked `minimal_add` block, live fixture checksum, manifest,
  snapshot, golden file, or other exact-copy gate exists;
- `src/core_lower.rs` and `src/core_verify.rs` consume the fixture dynamically
  through `include_str!` and are already represented by the archived candidate
  where their behavior changes;
- the Cranelift experiment probes read the live source dynamically, while the
  source SHA recorded in the historical lowering report is explicitly bound to
  its historical compiler commit and must not be rewritten; and
- unmarked `add` examples in `SPEC.md` and `docs/FORMAL_CORE.md` are independent
  teaching examples, not mirrors of this fixture. Historical Work Orders are
  records, not implementation consumers.

README is therefore the indispensable twelfth path, and the audit found no
thirteenth path. Its sole authorized implementation edit is to replace only the
code inside the existing marked block with this verbatim contiguous fixture
excerpt:

```hum
task add(a: Int, b: Int) -> Int {
  allocates:
    nothing

  does:
    return a + b
}
```

This adds exactly the `allocates: nothing` section and its separating blank
line. The marker comments, `Minimal form` teaching context, all other README
prose and examples, and `Assert-ReadmeHumExamplesMatch` remain unchanged.
README is synchronized public documentation, not semantic authority.

Against published `main`, the exact README edit is expected to be `+3/-0`,
with Git blob `d1f54f28c113de62c4d31f92a4eafbf6b6a26b2b`, SHA-256
`29c75e3c416811c433f64c2bb1800aff7e664794f5809575143707a9cf4dbd3e`,
32,429 bytes, and 487 lines. These identities must be reproduced before a
resumed candidate can be frozen; a discrepancy is a stop, not permission to
change another path.

### Frozen eleven archived blobs

The first resumed implementation candidate must restore these exact archive
blobs byte-for-byte:

| Path | Archived diff | Git blob |
| --- | ---: | --- |
| `src/type_check.rs` | `+68/-0` | `eb9f13d348aef04da255ff1b622f9b1005b5185d` |
| `src/core_verify.rs` | `+156/-2` | `afb4824c2718c6b7a05c747ebfbeb0e36156ac15` |
| `src/full_type_check.rs` | `+188/-13` | `4754748728581ff9f53cb5829275b9ac03e3fd2c` |
| `src/effect_check.rs` | `+127/-5` | `9e975b72c01a1046167e3155f4e3431a47e4e0bf` |
| `src/ownership_check.rs` | `+186/-16` | `8c3ea1909bea3fdef26b9351ac53a90545629c71` |
| `src/resource_check.rs` | `+146/-2` | `349648f08a0d0f7938b0b726bce9f1ca8d0af448` |
| `src/profile_check.rs` | `+168/-2` | `ac08e42c71a869651e0142dc2c0def1a7f2c8b7f` |
| `src/ir_readiness.rs` | `+668/-3` | `2e70944fd5ab6be2cfa6798001d28e120a6add25` |
| `examples/core/minimal_add.hum` | `+3/-0` | `86c7139d750d0b6e606eda248e3dc7b21f3923b2` |
| `docs/HUM_IR_READINESS_SCHEMA.md` | `+61/-1` | `3a1b9f618915126c1397f27bfaa9403fc4f455ac` |
| `tools/check_all.ps1` | `+87/-6` | `92828905e2f280be1d3f03397cea7bd6731b332b` |

Those eleven paths remain exactly `+1,858/-50`. The first resumed candidate
may add only the exact README edit above, producing exactly twelve paths and an
expected combined `+1,861/-50`. Its expected insertion accounting is 947
production-Rust lines, 760 permanent-test and compile-proof lines, and 154
combined schema/tool/example/README lines. These expected figures are identity
checks, not permission to ignore an honest discrepancy.

No other semantic or tool change is permitted before the resumed candidate's
final Fast run. A later implementer must verify the archive commit, scoped
tree, all eleven archived blob identities, and the exact README identity before
claiming the twelve-path candidate is restored.

## Complete implementation envelope

After this amendment is independently accepted, committed, published through
terminal full CI, recorded through terminal fast CI, and followed by a new
explicit BDFL resumption signal, exactly these twelve paths are authorized for
Unit 1:

1. `src/type_check.rs`
   - expose the narrow borrowed operand/resolver/type identity facts from the
     existing sole producer; no new disposition or general type API.
2. `src/core_verify.rs`
   - expose those facts only through the existing delivered verified result;
     no new Core-verifier classification or public row.
3. `src/full_type_check.rs`
   - add the same-report exact-operation handoff to effect.
4. `src/effect_check.rs`
   - produce exact checked-empty effect authority and hand it to ownership.
5. `src/ownership_check.rs`
   - produce exact checked-empty ownership authority and hand it to resource.
6. `src/resource_check.rs`
   - require the explicit claim plus structural no-allocation check and hand
     exact authority to profile.
7. `src/profile_check.rs`
   - bind the accepted default-normal conclusion and hand exact authority to
     IR readiness.
8. `src/ir_readiness.rs`
   - construct the final private facts, provide lifetime-bound access, consume
     it in production readiness, and freeze the new narrow projection.
9. `examples/core/minimal_add.hum`
   - add only the explicit `allocates: nothing` source intent.
10. `README.md`
    - update only the existing checked `hum-example` block for
      `examples/core/minimal_add.hum` by adding the exact `allocates: nothing`
      section and separating blank line; preserve verbatim contiguous fixture
      parity and make no unrelated prose or example change.
11. `docs/HUM_IR_READINESS_SCHEMA.md`
    - document the exact status/facts additions, fourteen-pass prerequisite
      set including `core_preview`, and non-authority boundary.
12. `tools/check_all.ps1`
    - register exact selectors and the compile-fail evidence; do not repair the
      native-stderr capture harness.

No listed path is speculative. No thirteenth path is permitted. In particular,
Unit 1 excludes:

- `src/ast.rs`, `src/parser.rs`, `src/resolve.rs`, `src/type_env.rs`;
- `src/core_body.rs`, `src/core_expr.rs`, `src/core_lower.rs`;
- `src/main.rs`, `src/lib.rs`, `src/version.rs`, `src/capabilities.rs`;
- `src/ir_contract.rs`, `src/backend_contract.rs`;
- Cargo files, workflows, other fixtures/examples, and docs or schemas other
  than the two explicitly listed documentation paths;
- a new backend-input, IR-verifier, digest, JSON parser, or backend module.

If any excluded path is required for an honest implementation, stop for BDFL
amendment. Do not widen the unit through cleanup or implementation momentum.

## Permanent evidence design

Four exact selectors are mandatory and must each list once, run once, pass
once, and receive one runtime credit:

```text
full_type_check::tests::minimal_add_backend_fact_handoff_is_exact_and_borrowed
ownership_check::tests::minimal_add_effect_and_ownership_authority_stays_operation_owned
profile_check::tests::minimal_add_resource_and_profile_authority_is_checked_empty
ir_readiness::tests::minimal_add_backend_facts_are_complete_but_ir_verify_blocked
```

The existing exact-selector inventory is 95 invocations / 95 unique selectors.
This unit adds exactly four normal-Fast invocations and four unique names, so
the final inventory must be 99/99 with independent named membership checks.

The IR-readiness selector must exercise production-path corruption of the
`core_preview` prerequisite. It must not obtain credit by substituting a
preselected failure enum or changing only expected output text.

### Positive matrix

Permanent evidence must prove:

- the edited minimal source has exactly one supported operation and one
  explicit allocation-free declaration;
- full type receives the exact Work Order 17 result from the same Core-verifier
  report and accepts `Int`;
- effect and ownership each issue exactly one checked-empty authority for that
  same operation;
- resource issues only after both the declaration and structural recheck;
- profile binds exactly the default `normal` conclusion;
- final facts contain every field and checked-empty/nonempty state frozen above
  in exact order, including all fourteen successful prerequisite passes;
- genuine `core_preview` authority is present once after `body_grammar` and
  before `core_lowering`, and Core lowering consumes that authority;
- the two operand definitions are distinct and remain bound left-to-first and
  right-to-second;
- the overflow failure edge is present exactly once;
- IR readiness reports the new status and ten facts in human/JSON parity;
- `ir_ready=0`, `ir_verify` is the only missing pass, and the blocker remains
  `ir_verify_not_implemented`; and
- repeated construction and rendering are byte-deterministic where public
  output is promised.

### Adversarial matrix

Evidence must independently reject or withhold final access for:

- foreign Program, source revision, file ordinal, normalized path, module,
  item traversal, section, statement, or operation;
- same-visible-name or same-public-ID substitution;
- missing, duplicate, extra, reordered, or foreign operation authority;
- root, child, result-value, type-ID, type-text, parameter-order, operand-order,
  resolver-definition, or semantic-definition substitution;
- duplicate operand node/value/definition identity;
- declared-result incompatibility;
- missing, duplicate, reordered, foreign, rejected, or unchecked full-type,
  effect, ownership, resource, or profile conclusion;
- effect target/declaration insertion or public pure-row fabrication;
- ownership move, borrow, alias, or transfer insertion;
- missing allocation declaration, duplicate/competing claim, visible
  allocation shape, call-like shape, or public accepted-row fabrication;
- explicit unknown or strict profile substitution and default-profile
  fabrication;
- omission of an empty set, replacement of checked-empty with not-checked, or
  replacement with unsupported-and-blocked;
- failure-edge omission, duplication, reorder, wrong type, wraparound, wrong
  width, or foreign trap semantics;
- missing, failed or blocked, duplicate, foreign, or reordered `core_preview`
  authority or evidence, exercised through the production path;
- a foreign `core_preview` from the wrong Program/source revision, item, or
  exact operation context even when rendered strings and public IDs agree;
- `core_preview` moved away from its frozen position after `body_grammar` and
  before `core_lowering`, proving the production order is load-bearing;
- failed, skipped, absent, duplicate, foreign, reordered, or zero-selection
  evidence for any of the fourteen required passes;
- coherent mutation of all public report fields while private lineage remains
  foreign or absent; and
- any report-global blocker.

Tests may use cfg/test-only corruption seams, but those seams may mutate only
existing candidates or public projections. They may not mint a valid private
authority, expected operation, verified type result, or final facts value.

### Lifetime and foreign-construction proof

`CanonicalMinimalAddBackendFactsAccess<'facts>` is a real production type. A
cfg-selected production compile-fail block in `src/ir_readiness.rs` must contain
four named probes:

```text
backend_facts_return_escape_must_not_compile
backend_facts_static_escape_must_not_compile
backend_facts_collection_escape_must_not_compile
backend_facts_foreign_construction_must_not_compile
```

`tools/check_all.ps1` must prove sequence `0/101/0`: normal all-target check,
cfg-selected expected failure, normal all-target check after restoration. The
failure must contain all four names and the intended lifetime/privacy classes,
must not be caused by an absent symbol, wrong cfg, import error, toy type, or
unrelated syntax error, and must restore the prior `RUSTFLAGS` environment.

## Compatibility and preservation evidence

The candidate must preserve, byte-for-byte unless this Work Order explicitly
changes the edited minimal source's resource/profile/readiness result:

- genuine authenticated `UInt + UInt` behavior;
- both legacy additive compatibility routes;
- representative non-target and unsupported target-like behavior;
- all existing schemas other than `hum.ir_readiness.v0`;
- blocker precedence for source, resolver, type, Core verify, full type,
  effect, ownership, resource, and profile errors;
- current CLI exits and traversal behavior; and
- all Work Order 16/17 authority and lifetime proofs.

For the edited minimal source, expected public deltas are limited to:

- parser-owned revision/section/span identities that necessarily reflect the
  authorized source edit;
- Core-lower and Core-verify source-section projections gaining the exact
  `allocates` section while operation kind/order/type/check conclusions stay
  unchanged;
- effect and ownership declaration counts gaining the one explicit allocation
  declaration while their exact statement conclusions stay accepted and
  unchanged;
- the existing resource report accepting the explicit allocation-free claim;
- the existing profile report accepting the default normal profile after the
  resource blocker clears; and
- IR readiness reporting the exact new pre-`ir_verify` status and facts above.

No other example or fixture changes.

## Sustainability and size boundary

This prerequisite is intentionally smaller than artifact encoding and byte
verification. Final implementation limits are:

- at most 1,900 insertions and fewer than 180 deletions across all twelve
  paths;
- at most 950 production-source insertions excluding `#[cfg(test)]` and
  compile-proof bodies;
- at most 760 permanent-test and compile-proof insertions;
- at most 190 combined schema, tool, example, and README insertions;
- no generated source, dependency, unsafe code, macro-generated protocol, or
  duplicated public projection; and
- no more than four focused independent-review hours.

Checked ordinal arithmetic must use checked operations. No truncation,
wrapping, saturation, unchecked indexing, or panic on adversarial candidate
data is allowed.

The original eleven-path authoring/correction lifecycle is consumed. Its
archived bytes and partial evidence receive no automatic acceptance credit.
The reviewed envelope amendment creates one new, explicit lifecycle: one
resumption pass from the exact archived blobs plus the exact README edit, one
fresh independent review of the complete twelve-path candidate, and at most one
separately authorized bounded correction inside this twelve-path envelope. A
correction may not add a path, public schema family, producer, mechanism, or
change to the frozen archive base. A thirteenth path or any architecture,
scope, or budget breach stops and returns to the BDFL.

## Resumed implementation evidence boundary

The resumed implementer must:

- verify all twelve path identities before running acceptance evidence;
- prove that README's checked block is the exact contiguous excerpt of the
  edited `examples/core/minimal_add.hum` fixture frozen above;
- run the proportional Rust, selector, source-audit, document, compatibility,
  and preservation checks already required by this Work Order;
- run final Fast exactly once on the frozen bytes with native stdout and stderr
  kept separate under the invocation rule below;
- run no local Exhaustive producer;
- stop on any completed Fast failure without correction, retry, or workaround;
  and
- leave the complete twelve-path candidate unstaged and uncommitted for one
  fresh independent implementation review.

Archive restoration, README editing, and every implementation check remain
unauthorized until the amendment's review, acceptance, commit, publication,
status, and explicit resumption gates are complete.

## Native-stderr evidence rule

Implementer and reviewer each run Fast at most once on their final exact bytes.
Fast must be invoked directly with native stdout and stderr kept separate. Do
not capture it through a terminating PowerShell transcript, `*>`, merged
pipeline, or wrapper that converts ordinary native stderr into
`NativeCommandError` under `ErrorActionPreference=Stop`.

Process-local Cargo `PATH` preparation is permitted and must be restored. A
completed failing repository check is a verdict and is not rerun. A launcher
or capture failure before candidate evidence is not a verdict and may be
retried only after correcting process-local invocation mechanics. This rule
does not authorize any change to `tools/check_all.ps1` beyond the selectors and
compile proof specified above and does not create a harness-repair unit.

No local Exhaustive run is authorized. Publication CI must select full mode,
pass both required platforms, and run the Ubuntu-owned platform-independent
Exhaustive producer; Windows skips only the duplicate producer.

## Explicit non-goals and bans

Unit 1 does not:

- emit `hum.backend_input.v0` bytes;
- compute an artifact ID or digest;
- parse or verify persisted bytes;
- implement `hum.ir_verify.v0`;
- construct `VerifiedBackendInput<'a>`;
- set `ir_ready` or execution readiness to one;
- emit Cranelift IR, an object, executable, ABI, or runtime wrapper;
- add a backend adapter or `lower` function;
- generalize expression typing, effects, ownership, resources, or profiles;
- change strict-profile policy or infer allocation freedom for arbitrary code;
- implement target selection, target probing, host facts, layout, linkage, or
  public ABI;
- repair the native-stderr harness;
- implement decision 0020 termination work;
- begin open-skeleton tooling; or
- begin backend/IR instruction work.

The following are forbidden:

- parsing rendered output or joining independent JSON/human reports;
- a global authority vector, batch cardinality, or later association pass;
- association by filename, task name, spelling, text, public ID, span alone,
  ordinal alone, or corpus count;
- a second type/effect/ownership/resource/profile producer;
- reconstructing Work Order 17 authority from public Core fields;
- treating Clone, forwarding, lineage, or a public accepted row as current-field
  integrity;
- treating checked-empty as omission;
- treating unsupported, blocked, or not-checked as checked-empty;
- weakening any existing blocker or fallback to manufacture backend facts;
- exposing a public/crate-visible constructor for any authority or final facts;
- serializing an authority or calling it `VerifiedBackendInput`;
- test-only minting of successful production authority;
- a thirteenth path or a new dependency; and
- reuse of rejected Work Order 13 or 15 implementation material.

Stop without workaround if any required fact cannot be tied to the exact live
operation; if the HRTB chain creates a cycle; if current resource policy must be
weakened; if a public report must become authority; if IR readiness would need
to claim `ir_ready=1`; if artifact encoding or `ir_verify` enters; if a listed
selector is absent/zero/multiple; if lifetime failure occurs for the wrong
reason; if compatibility bytes drift outside the frozen deltas; or if any size
or path limit is exceeded.

## Later coherent artifact-and-verifier unit

After this prerequisite is accepted, implemented, published, and closed, a
separate Work Order may consume only the final borrowed facts to:

```text
CanonicalMinimalAddBackendFactsAccess<'a>
  -> deterministic hum.backend_input.v0 canonical payload bytes
  -> sha256 payload identity and exact envelope
  -> hum.ir_verify.v0 over those exact bytes
  -> opaque lifetime-bound VerifiedBackendInput<'bytes>
  -> corruption-only consumer gate
```

That future unit must bind schema, exact canonical bytes/digest, source
revision, the complete fourteen-entry required pass set, semantic contract
version, and relevant profile and target-independent context. Deserialized
bytes carry no authority. Only the current verifier may issue the capability.
The eventual adapter entry remains exactly:

```text
lower(input: &VerifiedBackendInput) -> Result<BackendArtifact, BackendError>
```

It may not receive Program, raw Core, type environment, source, JSON, or an
unverified reconstruction path. The artifact encoder and verifier may not be
split merely to reduce line count.

Cranelift instruction emission, adapter implementation, ABI selection,
termination, open-skeleton work, and harness repair remain later and separately
authorized.

## Review, correction, commit, and publication gates

The original two-document planning package completed its review and
publication gates. This remedial `WORKORDER_18.md`-only amendment requires one
fresh independent pre-issuance architect review by a reviewer who did not
author or edit it. The reviewer must independently test authority validity,
the envelope-stop history, archive and blob identities, consumer-audit
completeness, the exact README synchronization rule, twelve-path necessity and
sufficiency, preserved architecture/evidence/budgets, lifecycle boundaries,
and all document checks.

The document verdict is `ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or `REJECT`; no
verdict authorizes an edit. At most one separately authorized bounded document
correction is allowed. Only an unqualified `ACCEPT`, followed by explicit BDFL
acceptance, authorizes a local documentation commit.

Documentation commit, publication, terminal full CI, publication-status
recording, and Unit 1 implementation are separate gates. No gate authorizes the
next one implicitly. Implementation acceptance, local commit, publication,
terminal full CI, and status closeout are also separate.

## Document-author checks

Before freezing this draft, the architect-author runs only:

- `git diff --check`;
- the complete 123-case status-classifier suite twice with byte-identical
  output;
- text hygiene;
- public readiness;
- alpha claims;
- release readiness for `0.0.1`; and
- read-only source/call-graph inspection.

No Cargo command, Rust selector, Fast, full preflight, Exhaustive, workflow,
performance evidence, or implementation probe is authorized during document
authoring.

## Current authorization gate

Work Order 18's original planning package and publication-status record are
published and terminal-green. Unit 1 is stopped at the material README envelope
defect. Only this remedial document amendment is under review.

Restoration of the archived eleven blobs, editing `README.md`, resuming
implementation, running implementation evidence, committing implementation,
pushing implementation, and later work remain unauthorized. Amendment review,
BDFL acceptance, the local documentation commit, publication and terminal full
CI, publication-status recording and terminal fast CI, and a new explicit BDFL
implementation-resumption signal are separate gates; none authorizes the next.

Backend artifact bytes, `ir_verify`, `VerifiedBackendInput`, Cranelift work,
native-stderr harness repair, open-skeleton tooling, termination, archive
mutation, another Work Order, and every later activity remain unauthorized
pending their explicit gates.
<!-- workorder-current-authorization-gate:end -->
