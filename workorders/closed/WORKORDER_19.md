# Hum Work Order 19: Bind Backend Facts With Load-Bearing Final Lineage Authority

Date: 2026-08-11
Status: Work Order 19 is closed. Its planning package and sole implementation
unit are independently accepted, published, and terminal-green.
The accepted planning commit is
`e5ccb140419e81f4e4eb0338c6674cdb11c352be`, with parent
`dc1475a808c3cf47980cc429f2f8ecaa787a1680` and subject
`docs(workorder): make backend lineage validation load-bearing`. It contains
exactly two paths and `+861/-1`: `WORKORDER_18.md` at `+0/-1`, blob
`71f0f071dc3c2028f2a47f4767c80dab2ba4afec`, and `WORKORDER_19.md` at
`+861/-0`, blob `f7f8353b7f3899c09bace938612f0281582d0852`. Publication was a normal
non-force fast-forward of `main` only, over range
`dc1475a808c3cf47980cc429f2f8ecaa787a1680..e5ccb140419e81f4e4eb0338c6674cdb11c352be`.

Required full-lane CI passed in workflow `ci`, run `31536839041`, attempt 1,
testing `e5ccb140419e81f4e4eb0338c6674cdb11c352be`, with overall conclusion
`success`. Ubuntu job `93929841761` succeeded in `17m52s`, with `17m10s` in
full preflight. Windows job `93929841859` succeeded in `25m27s`, with `25m02s`
in full preflight. Both platforms reported
`mode=full;reason=no_status_transition` with empty or zero status-chain fields;
passed Cargo-cache preparation, Rust-toolchain preparation, and full Hum/Fast
preflight; skipped status-only evidence; passed text hygiene and public
readiness for 527 files, alpha claims, and release readiness for `0.0.1`; and
printed `All Hum preflight checks passed.`

Ubuntu alone passed
`parser::tests::exhaustive_canonical_seal_pair_matrix_is_complete_and_nonzero`
with selected/passed/failed `1/1/0`, F1 630, F2 4,950, F3/F4 8,646, total
14,226, producer elapsed `16.969s`, and seed `0x48554D5F5345414C`. Windows
correctly skipped the platform-independent duplicate.

The sole implementation unit was committed as
`811588db0bbdbd42e0637d5d50c84ef72923f214`, with parent
`abe1bd95cb98f013688cdf26cc54fb7db6be9b05` and subject
`feat(ir): bind backend facts to program lineage`. Ocean Bennett is both author
and committer, using the repository-required GitHub no-reply identity. The
commit contains exactly twelve paths and `+2,319/-46`.

Fresh independent corrected-candidate review reported no P0, P1, or P2
findings and returned `ACCEPT`. Complete backend facts are bound to one exact
Program-owned minimal-add operation. Full-type, effect, ownership, resource,
and profile authorities remain private and compiler-sealed. The final
Program-lineage comparison is independently load-bearing, and the test-only
two-real-profile HRTB seam is absent from production. The ordered fourteen
passes, seven checked-empty states, checked signed-i64 overflow trap edge, and
exact public IR-readiness fact order are bound. `ir_ready=0` remains unchanged,
`ir_verify_not_implemented` remains the sole blocker, and the result is not
`VerifiedBackendInput`. No backend artifact, digest, IR verification, or
backend lowering was implemented.

Publication was a normal non-force fast-forward of `main` only over range
`abe1bd95cb98f013688cdf26cc54fb7db6be9b05..811588db0bbdbd42e0637d5d50c84ef72923f214`.
No other remote ref changed.

Required full-lane CI passed in workflow `ci`, run `31553589478`, attempt 1,
testing `811588db0bbdbd42e0637d5d50c84ef72923f214`, with overall conclusion
`success`. Ubuntu job `93981274365` succeeded in `10m10s`, with `9m39s` in
full preflight and root suite `448/448`. Windows job `93981274258` succeeded in
`16m53s`, with `16m26s` in full preflight and root suite `463/463`.

Both platforms reported `mode=full;reason=no_status_transition` with empty or
zero status-chain fields. Cargo-cache and Rust-toolchain preparation passed;
full Hum/Fast preflight passed; status-only evidence was skipped; exact
selector inventory was `99/99`; all four isolated Work Order 19 selectors
passed `1/1`; the exact ordered ten-fact public sequence and exact blocker JSON
`["ir_verify_not_implemented"]` passed; the combined privacy/lifetime sequence
was `0/101/0` with no E0382 or unrelated compile-fail substitute; F4 inventory
was `14/18/1/19`, with zero unregistered consumers and issuers `4/4`; text
hygiene and public readiness passed for 527 files; alpha claims and release
readiness for `0.0.1` passed; and both printed
`All Hum preflight checks passed.`

Ubuntu alone passed
`parser::tests::exhaustive_canonical_seal_pair_matrix_is_complete_and_nonzero`
with selected/passed/failed `1/1/0`, F1 630, F2 4,950, F3/F4 8,646, total
14,226, producer elapsed `17.009s`, and seed `0x48554D5F5345414C`. Windows
correctly skipped the platform-independent duplicate.

Work Order 19 is closed, and its backend-facts authority is accepted and
present on `main`. The result remains blocked before IR verification. No later
session or implementation is authorized; a successor design requires a fresh
Work Order and separate BDFL authorization. Artifact encoding, SHA-256 artifact
identity, `hum.ir_verify.v0`, `VerifiedBackendInput`, Cranelift or other backend
lowering, ABI work, native-stderr harness repair, open-skeleton integration,
termination work, recovery-stash cleanup, archive mutation, and unrelated work
remain unauthorized.

Owner: BDFL (Ocean).
Author: Work Order 19 architect-author. The author may not independently review
or accept this planning package or any implementation candidate produced under
it.

Planning baseline: `HEAD`, local `main`, cached `origin/main`, and live remote
`main` are all `dc1475a808c3cf47980cc429f2f8ecaa787a1680`, the published Work
Order 18 closeout. The worktree was clean, the index empty, and no untracked
file existed before this two-document draft began.

## Closed predecessor and authority state

Work Order 18 is fully closed. Its closeout commit is
`dc1475a808c3cf47980cc429f2f8ecaa787a1680`. Closeout workflow `ci`, run
`31529509163`, attempt 1, passed on Ubuntu job `93905861490` and Windows job
`93905861566`. Both jobs selected the exact fast status chain anchored at
`0396399c94f5e43511f3811319320a6ca2db0b93`.

Work Orders 16 and 17 remain the accepted authority for:

- Program-owned source, item, section, statement, and exact Core-operation
  identity;
- compiler-sealed first construction of canonical Core reports;
- exact ordered resolver references and distinct parameter definitions;
- the immutable six-way minimal-add source disposition;
- independently verified checked `Int` type and result-value authority; and
- the report-bound HRTB handoff consumed by full type.

No backend-facts authority from Work Order 18 was accepted or published to
`main`. Work Order 19 must earn every production, evidence, compatibility, and
privacy claim again.

## Published Work Order 18 failure evidence

Two Work Order 18 archives exist locally, in origin tracking, and on the live
remote. They are immutable failure evidence only.

### Envelope-stop archive

- branch: `archive/workorder-18-unit1-envelope-stop-2026-08-10`;
- commit: `82c6f74a6392ad806b4ab1dd33960654595627fb`;
- parent: `a3b60708524edfc5631b8a6617ac6b2740fbeafe`;
- complete tree: `05e60ef87b6a23b6a474323c25cdd5385f8c675c`;
- scoped eleven-path tree:
  `f4b3594b1d4d745b7e8801b72ffa4a80204feabb`; and
- statistics: eleven paths, `+1,858/-50`.

This archive stopped because `README.md` is a checked durable mirror of
`examples/core/minimal_add.hum` and was missing from the implementation
envelope.

### Terminal-rejection archive

- branch: `archive/workorder-18-unit1-terminal-rejection-2026-08-11`;
- commit: `cc8af829bb8794daabee92298e4154be917b1de3`;
- parent: `6ca305e6d9de76968ec5a3abc22e6a3fed4bdc7f`;
- complete tree: `970a65ac80fbcecdf93994796fe792d46aeb632c`;
- scoped twelve-path tree:
  `10daf6b4598086769ef5a6de87de796e7858b619`; and
- statistics: twelve paths, `+2,334/-50`.

The terminal candidate's compiler-sealed intermediate wrappers and seven
distinct checked-empty states were independently found sound. They receive no
automatic acceptance credit here. The candidate was rejected because its
final Program-lineage evidence was not load-bearing:

- the absent-lineage source shape lost authority during full-type checking;
- the foreign-lineage source shape was rejected earlier by resource checking's
  Program-owned Core expectation;
- neither corruption reached the final backend-facts lineage comparison; and
- disabling that final comparison still left the claimed selector green.

The rejection was caused by evidence topology, not by the candidate's raw
`+2,334/-50` size. Natural source-level corruption that terminates upstream
cannot prove a downstream authority boundary.

The recovery stash remains older, parked, non-authoritative evidence:

- commit: `73101039f5e3faf0c802d4f723add1b891c51602`;
- tree: `535198cd6c9fdbd2fb713a30266530cb47e766c0`.

Neither archive may be checked out, merged, rebased, cherry-picked, amended,
or treated as presumptively correct under this planning authorization. The
stash may not be applied, popped, dropped, rewritten, or used as authority.

## Mandatory satisfiability ruling

The final Program-lineage comparison is satisfiable as an independent
authority gate without weakening production privacy.

The terminal archive's real path already delivers an actual private
`profile_check::VerifiedMinimalAddProfile<'report>` only after one complete
same-report chain has succeeded:

```text
WO17 verified exact type
  -> VerifiedMinimalAddFullType
  -> VerifiedMinimalAddEffect
  -> VerifiedMinimalAddOwnership
  -> VerifiedMinimalAddResource
  -> VerifiedMinimalAddProfile
  -> CanonicalMinimalAddBackendFacts assembly
  -> final Program-lineage comparison
  -> borrowed CanonicalMinimalAddBackendFactsAccess
```

Each wrapper has private fields and is issued only by its owning production
stage. `profile_check::with_profile_for_ir_readiness` permits an IR-readiness
consumer to receive a genuine wrapper without constructing it. Two such HRTB
callbacks can be nested while both report borrows remain live. Therefore a
test can obtain:

1. one honest profile authority issued from an honest Program; and
2. one foreign profile authority issued independently from a second Program
   parsed from the same source bytes and normalized path.

Both Programs can have byte-identical public reports and successful full-type,
effect, ownership, resource, and profile results while retaining distinct
private current-process Program identity.

The required test-only seam is inside `src/ir_readiness.rs`, after both real
profile authorities have issued and before final backend-facts access is
issued. It presents:

- the honest Program, item, statement, profile authority, source revision,
  operation, checked type, public report fields, required passes, checked-empty
  states, and failure edge to normal production assembly; and
- only the foreign actual profile authority as the lineage operand consumed by
  the final Program-lineage comparison.

The seam does not construct a wrapper, clone authority, mutate an upstream
report, substitute a scalar identity, select a canned error, or build a fake
backend-facts record. It changes only which already-issued actual profile
authority supplies the private Program lineage to the final comparison.

Production remains unchanged in meaning: the normal issuer supplies the same
honest profile authority both as semantic input and as the final lineage
operand. The test-only alternative is absent when `cfg(test)` is false.

This technique reaches the exact boundary Work Order 18 did not reach. If the
final Program-lineage comparison is removed or bypassed in a disposable
mutation, the foreign actual profile must receive final facts access and the
focused selector must fail. That mutation result makes the final comparison,
and no upstream rejection, load-bearing.

If implementation proves that this exact actual-type technique cannot compile
without exposing a constructor, fabricating authority, changing production
visibility, or adding another path, stop. Do not substitute an upstream
corruption matrix or a preselected failure.

## Unit 1 exact result

Unit 1 re-establishes the smallest coherent backend-facts prerequisite on the
current published tree. For the exact supported canonical `Int + Int`
operation only, it carries one compiler-sealed authority chain through full
type, effect, ownership, resource, and profile, then issues one private
`CanonicalMinimalAddBackendFacts` after the final Program-lineage comparison
passes.

The final access:

- is lifetime-bound through an `impl for<'facts> FnOnce(...) -> R` callback;
- cannot escape, become `'static`, be collected, serialized, or constructed by
  a sibling module;
- is not `VerifiedBackendInput` and grants no lowering or execution authority;
- authenticates no persisted bytes; and
- leaves `ir_ready=0` with `ir_verify_not_implemented` as the sole next blocker
  only after every prerequisite below succeeds.

The final private facts bind:

- compiler version `0.0.1`;
- semantic contract `hum.canonical_minimal_add_backend_facts.v0`;
- target context `target_independent_checked_i64_v0`;
- authenticated source revision, normalized path, semantic file ordinal,
  module, item, task signature, `does` slot, statement, and operation slot;
- root, ordered children, ordered parameter and resolver-definition identity,
  result value, checked `Int` type, and declared-result compatibility;
- accepted default profile `normal`;
- the fourteen ordered successful prerequisite passes;
- seven distinct checked-empty states; and
- exactly one `signed_64` / `checked_add` /
  `runtime_trap_on_overflow` failure edge.

The seven checked-empty states remain separate and ordered:

1. `effects`
2. `ownership_transfers`
3. `allocations`
4. `contract_predicates`
5. `evidence_obligations`
6. `unsupported_or_weakened`
7. `external_authority`

Omission, blocked, unsupported, unchecked, foreign, duplicate, or reordered is
not checked-empty.

The fourteen successful prerequisites remain:

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

`ir_verify` is not a successful prerequisite. It remains
`not_implemented`.

For `examples/core/minimal_add.hum`, the source continues to require exactly
one explicit allocation-free declaration:

```hum
task add(a: Int, b: Int) -> Int {
  allocates:
    nothing

  does:
    return a + b
}
```

The checked README block remains a verbatim contiguous mirror of that fixture.
README is a documentation consumer, never semantic authority.

## Producer, validator, and consumer integration map

| Stage and path | Producer | Validator | Consumer |
| --- | --- | --- | --- |
| `src/type_check.rs` | Existing sole canonical minimal-add disposition and borrowed backend identity | Ordered operands, distinct resolver definitions, checked `Int`, declared result | `src/core_verify.rs` through the WO17 delivered result |
| `src/core_verify.rs` | Verified exact-operation/type access plus genuine Core prerequisite sequence | Program-owned item/statement/operation and Core invariant checks | `src/full_type_check.rs` |
| `src/full_type_check.rs` | `VerifiedMinimalAddFullType` | Same report, accepted exact statement, no local/global blocker | `src/effect_check.rs` |
| `src/effect_check.rs` | `VerifiedMinimalAddEffect` | Exact accepted pure/local row, no target/declaration/external effect, no blocker | `src/ownership_check.rs` |
| `src/ownership_check.rs` | `VerifiedMinimalAddOwnership` | Exact accepted no-transfer row, no move/borrow/alias/transfer, no blocker | `src/resource_check.rs` |
| `src/resource_check.rs` | `VerifiedMinimalAddResource` | One `allocates: nothing`, accepted row, structural no-allocation recheck, no blocker | `src/profile_check.rs` |
| `src/profile_check.rs` | `VerifiedMinimalAddProfile` | One accepted default `normal` conclusion and no blocker | `src/ir_readiness.rs` |
| `src/ir_readiness.rs` | Private canonical backend-facts candidate | Complete facts plus final Program-lineage comparison | Production readiness projection and HRTB facts consumer |

The dependency direction is one-way. No downstream stage may call back into an
earlier producer to reconstruct authority. Public human/JSON rows, names,
spans, IDs, ordinals, summaries, or report position never substitute for the
private chain.

## Exact final-lineage validation technique

The implementation must factor final validation so normal production and the
test-only foreign-lineage path execute the same comparison and the same access
issuance logic.

The required shape is semantically equivalent to:

```text
assemble honest candidate facts from one real VerifiedMinimalAddProfile
  -> validate all semantic facts against that honest profile
  -> validate final Program lineage against a borrowed actual profile operand
  -> issue access only when every check is true
```

Production passes the facts' own honest profile as the final lineage operand.
The test-only seam passes an independently issued foreign profile only for that
operand.

The implementation must use one private final validator, named
`is_complete_with_final_profile_lineage` or an exact semantic equivalent. It
must contain the sole final comparison between:

- the Program identity carried by the supplied actual profile authority; and
- the address identity of the honest Program for which facts are being issued.

All other comparisons must read the honest facts/profile. The test-only helper,
named `issue_with_final_profile_lineage_for_test` or an exact semantic
equivalent, must:

- be guarded by `#[cfg(test)]`;
- remain private to the `ir_readiness` module/test child;
- accept actual `VerifiedMinimalAddProfile` values or borrows obtained through
  real production callbacks;
- call the same assembly, validator, and issuance functions as production;
- expose no constructor or mutable authority field; and
- be absent from normal production builds.

A thread-local test observation may count execution of the exact final
comparison. It may not choose the result. The negative result must come from
the real comparison itself.

Forbidden substitutes include:

- a string, integer, raw pointer, enum, or boolean supplied as fake lineage;
- a thread-local preselected failure;
- changing a public path, name, ID, span, or report row;
- asking an upstream stage to combine an authority from one Program with an
  item from another;
- manually constructing any verified wrapper;
- cloning a wrapper or final facts;
- rendering and reparsing a report; or
- duplicating the final validator in test code.

## Production and test configurations

The complete candidate must compile and remain honest in each applicable
configuration.

### Normal production configuration

- `cfg(test)` is false.
- The foreign-lineage helper and comparison counter do not exist.
- The normal issuer passes one honest profile in both semantic and final-
  lineage roles.
- No compile-fail cfg is active.
- Human/JSON behavior changes only for the exact supported minimal-add
  candidate specified here.

### Unit-test configuration

- `cfg(test)` is true.
- The private foreign-lineage helper is visible only to the same-file test
  child.
- Two real Program/report chains are built independently.
- No successful production authority is manually minted.
- Parallel tests cannot leak corruption state; any observation is thread-local
  and one-shot.

### Compile-fail configuration

The existing production access and every intermediate verified wrapper are
used in actual-type lifetime/privacy probes. The tool proves normal/failing/
normal sequence `0/101/0`, checks all required function names and intended
diagnostic classes, rejects absent-symbol/import/cfg/syntax causes, and restores
the prior `RUSTFLAGS` exactly.

### Platform configuration

The change is platform-independent Rust and documentation/tooling. Host checks
must cover Windows. Publication CI is final authority for Ubuntu and Windows.
Only Ubuntu runs the platform-independent Exhaustive producer; Windows skips
only that duplicate.

## Complete Unit 1 implementation envelope

The exact dependency graph requires these twelve paths:

1. `src/type_check.rs`
   - expose only the narrow borrowed backend identity already proved by WO17;
     add no new disposition, general query, or constructor.
2. `src/core_verify.rs`
   - deliver the exact verified type/backend identity and genuine ordered Core
     prerequisite evidence to full type.
3. `src/full_type_check.rs`
   - issue the compiler-sealed same-report full-type wrapper to effect.
4. `src/effect_check.rs`
   - issue exact checked-empty effect authority to ownership.
5. `src/ownership_check.rs`
   - issue exact checked-empty ownership authority to resource.
6. `src/resource_check.rs`
   - combine the explicit allocation-free claim with structural recheck and
     issue exact resource authority to profile.
7. `src/profile_check.rs`
   - issue the accepted default-normal profile authority to IR readiness.
8. `src/ir_readiness.rs`
   - assemble and validate final facts, implement the production-unavailable
     actual-type lineage seam, issue borrowed access, and consume it in the
     readiness projection.
9. `examples/core/minimal_add.hum`
   - add only the explicit `allocates: nothing` intent.
10. `README.md`
    - mirror only the exact checked minimal-add fixture block.
11. `docs/HUM_IR_READINESS_SCHEMA.md`
    - document the narrow status/facts contract and non-authority boundary.
12. `tools/check_all.ps1`
    - register the exact selectors, source/privacy audits, compile-fail proof,
      and deterministic Fast evidence.

No thirteenth path is permitted. In particular, the unit excludes:

- `WORKORDER_18.md`, `WORKORDER_19.md`, other Work Orders, governance, and
  decision records;
- `src/ast.rs`, `src/parser.rs`, `src/resolve.rs`, and `src/type_env.rs`;
- `src/core_body.rs`, `src/core_expr.rs`, and `src/core_lower.rs`;
- `src/main.rs`, `src/lib.rs`, `src/version.rs`, and `src/capabilities.rs`;
- `src/ir_contract.rs` and `src/backend_contract.rs`;
- Cargo manifests/locks, workflows, dependencies, other fixtures, other
  schemas, and other documentation; and
- a backend-input, artifact, digest, IR-verifier, backend adapter, or lowering
  module.

If an honest implementation requires any path outside this inventory, stop for
a BDFL ruling. Do not weaken a consumer, copy a public report, hide a path in a
generated artifact, or use implementation momentum to widen the unit.

## Archive relationship and review telemetry

The terminal archive's twelve paths and raw `+2,334/-50` statistics are review
telemetry, not an accepted base and not a size ceiling.

| Path | Terminal archive telemetry |
| --- | ---: |
| `README.md` | `+3/-0` |
| `docs/HUM_IR_READINESS_SCHEMA.md` | `+61/-1` |
| `examples/core/minimal_add.hum` | `+3/-0` |
| `src/type_check.rs` | `+68/-0` |
| `src/core_verify.rs` | `+156/-2` |
| `src/full_type_check.rs` | `+198/-13` |
| `src/effect_check.rs` | `+139/-5` |
| `src/ownership_check.rs` | `+197/-16` |
| `src/resource_check.rs` | `+156/-2` |
| `src/profile_check.rs` | `+178/-2` |
| `src/ir_readiness.rs` | `+1,041/-3` |
| `tools/check_all.ps1` | `+134/-6` |

A future implementer must begin from clean published `main`, inspect the real
current code, and account for every candidate line. Archive inspection is
read-only comparison evidence. No archive commit may be merged, cherry-picked,
or called accepted implementation.

Every candidate delta beyond the terminal archive must be categorized as one
of:

- the independently justified post-upstream actual-type lineage seam;
- direct permanent evidence that makes that seam or existing privacy
  load-bearing; or
- a mechanically necessary compile/format adjustment caused by those two.

Any other semantic, public, schema, fixture, tool, or production change is a
scope breach and stops the unit.

## Public IR-readiness contract

No command or schema family is added. For the exact candidate owning final
private facts, `hum.ir_readiness.v0` reports:

```text
status=blocked_before_ir_verify_with_backend_input_facts_v0
missing_passes=[ir_verify]
blocking_reasons=[ir_verify_not_implemented]
ready_for_ir=0
```

Its ordered `facts_available` additions are:

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

All other candidates preserve their current statuses, fields, ordering,
null/omission behavior, and blocker precedence. No output says `verified`,
`backend_ready`, `lowerable`, or `ir_ready=1`.

## Exact selector plan

Four exact selectors are mandatory. Each must independently list exactly one
test, run exactly one, pass exactly one, and receive exactly one runtime credit:

```text
full_type_check::tests::minimal_add_backend_fact_handoff_is_exact_and_borrowed
ownership_check::tests::minimal_add_effect_and_ownership_authority_stays_operation_owned
profile_check::tests::minimal_add_resource_and_profile_authority_is_checked_empty
ir_readiness::tests::minimal_add_backend_facts_are_complete_but_ir_verify_blocked
```

The current published exact-selector inventory is 95 invocations / 95 unique
selectors. Unit 1 adds exactly these four normal-Fast invocations and four
unique names. Final inventory is 99/99, with an independent named-membership
assertion for all four.

The final IR-readiness selector owns the post-upstream foreign-lineage proof.
Do not count an occurrence inside the ordinary root suite as isolated selector
evidence.

## Permanent positive and authority evidence

Permanent tests must prove:

- the exact edited source has one supported operation and one allocation-free
  declaration;
- the README checked block is a byte-contiguous mirror of that fixture;
- one honest same-Program chain issues all five intermediate wrappers and one
  final access;
- every wrapper has private fields and no production `Clone`, `Copy`,
  `Default`, public constructor, serializer, builder, conversion, macro, or
  crate-visible field route;
- the complete fourteen-pass sequence is present once and ordered;
- all seven checked-empty states are present once, distinct, and ordered;
- the checked signed-i64 overflow trap edge is present once;
- root/child/result/type/parameter/resolver identity remains exact;
- public human/JSON output is ordered, deterministic, and limited to the
  frozen delta;
- `ir_ready=0` and `ir_verify_not_implemented` remain honest; and
- ordinary UInt, legacy additive, unsupported target-like, integrity-failure,
  and non-target behavior remains compatible.

## Load-bearing final-lineage evidence

The final selector must build two real Programs from byte-identical source and
the same normalized path. The Programs must have different in-process address
identity while their relevant public reports are byte-identical.

For each Program independently, the selector must prove:

- full type, effect, ownership, resource, and profile summaries have zero
  blockers;
- the actual private `VerifiedMinimalAddProfile` is issued through the full
  production HRTB chain; and
- the ordinary production final-access path succeeds.

Then, with both report borrows live, the selector must invoke the private
test-only final-lineage seam using:

- the honest Program, item, statement, and honest profile for all assembly and
  semantic validation; and
- the second Program's actual profile solely for final Program lineage.

Required observations:

- the exact final comparison executes once;
- all prior validations remain successful;
- no public field, source byte, item, operation, type, pass, checked-empty
  state, failure edge, or upstream authority changes;
- no final access is issued for the foreign lineage; and
- the honest path still issues exactly one access.

An absent-lineage or natural foreign-source case may remain as compatibility
evidence, but it earns no credit for this final boundary because it stops
upstream.

## Independent mutation evidence

The implementer must self-probe, and the independent reviewer must repeat, one
disposable-copy mutation of the frozen candidate:

1. change only the final Program-lineage comparison so it unconditionally
   accepts after recording its normal observation;
2. run only the exact IR-readiness selector;
3. require the selector to fail because foreign actual lineage now receives
   access;
4. confirm the honest path and all upstream assertions reached the same point;
   and
5. remove the disposable copy and every artifact.

A mutation that breaks compilation, changes an upstream producer, disables the
test helper, changes expected text, or triggers another failure earns no
credit. The canonical repository remains byte-identical during reviewer
mutation evidence.

## Adversarial completeness matrix

In addition to the final-lineage case, permanent evidence must reject or
withhold access for:

- foreign Program/source/file/module/item/section/statement/operation;
- same visible names, public IDs, spans, and rendered rows with foreign private
  authority;
- missing, duplicate, reordered, extra, or foreign operation authority;
- root, child, result value, checked type, parameter order, operand order,
  resolver definition, or semantic definition substitution;
- duplicate operand node/value/definition identity;
- declared-result incompatibility;
- missing, rejected, unchecked, duplicate, reordered, foreign, or globally
  blocked full-type/effect/ownership/resource/profile conclusion;
- fabricated pure/no-transfer/allocation-free/default-profile public rows;
- effect target/declaration, ownership move/borrow/alias/transfer, missing or
  competing allocation claim, visible allocation/call shape, and unknown or
  strict profile;
- each checked-empty state missing, unchecked, unsupported, corrupted,
  duplicated, or reordered independently;
- failure edge missing, duplicated, reordered, wrong type, wrapping, wrong
  width, or foreign trap semantics;
- each required pass missing, failed, skipped, zero-selected, duplicate,
  foreign, or reordered; and
- genuine `core_preview` authority missing, blocked, duplicate, foreign, or
  moved away from its frozen position between `body_grammar` and
  `core_lowering`.

Every corruption must traverse the relevant production validator. A
preselected result, detached reconstruction, expected-output-only edit, or
zero-selected test earns no credit.

## Lifetime, privacy, and production-unavailability proofs

The final access compile-fail block retains these actual production probes:

```text
backend_facts_return_escape_must_not_compile
backend_facts_static_escape_must_not_compile
backend_facts_collection_escape_must_not_compile
backend_facts_foreign_construction_must_not_compile
```

The combined privacy proof also attempts sibling construction of each actual
intermediate wrapper:

```text
verified_minimal_add_full_type_sibling_construction_must_not_compile
verified_minimal_add_effect_sibling_construction_must_not_compile
verified_minimal_add_ownership_sibling_construction_must_not_compile
verified_minimal_add_resource_sibling_construction_must_not_compile
verified_minimal_add_profile_sibling_construction_must_not_compile
```

The tool proves intended lifetime/privacy diagnostics, not a missing import,
wrong cfg, toy type, absent symbol, move-after-use, or syntax error. Normal
checks pass before and after the expected failure and `RUSTFLAGS` is restored.

Source/configuration audit must prove:

- exactly one test-only final-lineage helper definition;
- exactly one focused-selector call to it;
- the helper and observation state are inside `#[cfg(test)]` production-source
  exclusion;
- no normal production call, public/crate-visible constructor, or second final
  validator exists;
- the normal issuer uses the honest profile as the final lineage operand; and
- deleting the test-only helper cannot change a normal production build.

## Compatibility and preservation evidence

The candidate must compare current `main` and candidate behavior in isolated
trees for all affected public surfaces. It must preserve byte-for-byte behavior
except the explicitly edited minimal-add resource/profile/readiness path.

Required preservation includes:

- genuine authenticated `UInt + UInt`;
- both legacy additive compatibility routes;
- representative unsupported target-like, integrity-failure, and non-target
  inputs;
- all schemas other than `hum.ir_readiness.v0`;
- source/resolver/type/Core/full-type/effect/ownership/resource/profile blocker
  precedence;
- CLI exits, corpus traversal, and deterministic JSON;
- every Work Order 16 and 17 authority/lifetime proof; and
- no private type name, lineage, permit, Program address, or test-seam name in
  public output.

For the edited minimal source, allowed public changes are only:

- identities/spans necessarily changed by `allocates: nothing`;
- the synchronized README block;
- resource acceptance of the explicit structurally verified no-allocation
  claim;
- profile acceptance after the resource blocker clears; and
- the narrow final IR-readiness status and facts listed above.

No other fixture or example changes.

## Validation protocol

### Implementer focused and proportional checks

On the final exact candidate, the implementer runs:

- `cargo fmt --all -- --check`;
- `cargo check --all-targets`;
- applicable warnings-denied Clippy;
- the four exact selectors with list/run/nonzero/membership evidence;
- the combined lifetime/privacy `0/101/0` proof;
- the final-lineage mutation probe in a disposable copy;
- source/F4/configuration audits;
- public compatibility and corpus checks proportional to the twelve paths;
- `git diff --check`;
- text hygiene and public readiness for the complete tracked file count;
- alpha claims; and
- release readiness for version `0.0.1`.

The implementer runs Fast exactly once on the final frozen bytes by invoking:

```powershell
.\tools\check_all.ps1 -EvidenceTier Fast
```

The invocation is direct. Native stdout and stderr remain separate. Do not use
`*>`, transcript merging, a terminating pipeline, or a wrapper that converts
ordinary Cargo stderr into `NativeCommandError`. Process-local Cargo `PATH`
preparation is allowed and must be restored.

A completed Fast failure stops the unit. No retry, repair, expected-text edit,
or workaround follows without a separate BDFL ruling. A launcher failure
before candidate evidence may be corrected only at the invocation boundary and
must be disclosed.

No local Exhaustive producer is authorized.

### Independent review

A fresh reviewer who did not author or edit the candidate must:

- authenticate the exact base, twelve paths, per-path blobs, raw and
  whitespace-insensitive statistics, index, untracked set, marker, archives,
  and stash;
- inspect the complete diff and every producer/validator/consumer path;
- independently obtain two real profile authorities and reproduce the
  post-upstream foreign-lineage rejection;
- repeat the disposable final-comparison mutation probe;
- inspect all production/test configurations and privacy boundaries;
- repeat focused, compatibility, and high-risk checks proportionally; and
- issue one findings-first verdict: `ACCEPT`, `ACCEPT WITH REQUIRED FIX`, or
  `REJECT`.

The reviewer does not automatically duplicate the implementer's complete Fast
run. A repeated Fast requires a concrete disputed or high-risk reason. Review
never authorizes a commit or push by itself.

### Publication CI

Only after independent acceptance, a separately authorized local commit, and a
separately authorized push may required CI run. It must test the exact accepted
commit on Ubuntu and Windows in full mode. Both platforms must pass full Fast
preflight and the exact four-selector/99-inventory evidence.

The Ubuntu job alone runs the platform-independent Exhaustive producer:

```text
parser::tests::exhaustive_canonical_seal_pair_matrix_is_complete_and_nonzero
```

Expected evidence remains one selected, one passed, zero failed, with F1 630,
F2 4,950, F3/F4 8,646, total 14,226, and seed
`0x48554D5F5345414C`. These numbers are not evidence until reproduced from the
workflow for the exact accepted commit. Windows skips only the duplicate.

## Sustainability accounting

The unit is sized by dependency coherence. The private chain cannot compile or
be meaningfully reviewed if split between its producer, intermediate
authorities, final validator, consuming readiness projection, fixture/README
pair, schema, and permanent harness.

The terminal archive's `+2,334/-50` is the comparison baseline for review
telemetry only. The candidate report must provide:

- raw and whitespace-insensitive statistics per path and combined;
- production, permanent-test/compile-proof, and
  schema/tool/example/README insertion categories;
- the exact delta from terminal-archive telemetry; and
- a line-by-line explanation of all new code beyond the final-lineage seam and
  its direct evidence.

No line-count quota may force omission, formatting suppression, a fake test,
or a deliberately broken intermediate. The hard sustainability boundaries are:

- exactly twelve paths and no thirteenth;
- one coherent backend-facts mechanism and no parallel producer;
- no dependency, unsafe code, generated source, macro-generated authority, or
  duplicated public projection;
- one direct implementer Fast and no local Exhaustive; and
- a complete candidate reviewable in one independent sitting.

If the candidate grows beyond the terminal archive for unrelated cleanup,
public redesign, another mechanism, another stage, or another consumer, stop.

## Explicit deferrals and bans

Work Order 19 does not authorize:

- `hum.backend_input.v0` artifact encoding;
- SHA-256 artifact or envelope identity;
- persisted canonical backend-input bytes;
- `hum.ir_verify.v0`;
- `VerifiedBackendInput`;
- Cranelift, LLVM, Wasm, C, or custom backend lowering;
- ABI, calling convention, layout, target, object, linker, runtime-wrapper, or
  executable decisions;
- native-stderr harness repair;
- open-skeleton integration;
- termination or quantitative-bound work;
- recovery-stash cleanup;
- archive mutation or deletion;
- another Work Order, governance change, or unrelated compiler cleanup; or
- any later artifact, backend, tooling, or release work.

The unit may not parse rendered output, join independent reports, introduce a
global authority vector, reassociate by public fields, serialize authority,
call a second classifier, or call any final value `VerifiedBackendInput`.

Stop without workaround if:

- the actual-type post-upstream seam cannot be implemented exactly as frozen;
- a thirteenth path or dependency is required;
- any verified wrapper must become constructible or mutable outside its owner;
- the foreign case fails before the final comparison;
- the mutation probe fails for compilation or another reason rather than
  granting the foreign access;
- a selector selects zero or multiple tests;
- public compatibility drifts outside the frozen delta;
- `ir_ready` would become one;
- artifact encoding or `ir_verify` enters; or
- the candidate cannot be reviewed as one coherent unit.

## Work Order 19 lifecycle

The gates are separate and no gate implies the next:

1. independent pre-issuance review of this two-document package;
2. explicit BDFL acceptance and local documentation commit;
3. separate BDFL publication and terminal full CI inspection;
4. isolated publication-status record and terminal fast CI;
5. explicit BDFL Unit 1 implementation signal;
6. one bounded implementation on clean published `main`;
7. fresh independent complete-candidate review;
8. separately authorized local implementation commit;
9. separately authorized publication and terminal full CI; and
10. separately authorized closeout/status record and publication.

This Work Order permits at most one separately authorized bounded correction
after the complete implementation review, and only if the correction remains
inside the twelve paths, mechanism, public contract, and evidence meaning
frozen here. A repeated final-lineage evidence failure, new mechanism,
thirteenth path, architecture change, or public-contract change returns
directly to the BDFL rather than beginning another correction loop.

## Planning-package validation

The architect-author runs only document-level evidence:

- `git diff --check`;
- fail-closed no-index whitespace checking for this new Work Order;
- the complete 123-case status-classifier suite twice with byte-identical
  output;
- text hygiene;
- public readiness;
- alpha claims; and
- release readiness for `0.0.1`.

No Cargo command, Rust selector, Fast, full preflight, Exhaustive, workflow,
performance check, archive restoration, or implementation probe is part of
planning validation.

## Current authorization gate

Work Order 19 Unit 1 is independently accepted, published as
`811588db0bbdbd42e0637d5d50c84ef72923f214`, and terminal-green in workflow
`ci`, run `31553589478`, attempt 1. Ubuntu job `93981274365` and Windows job
`93981274258` both succeeded in full mode with reason `no_status_transition`.

Work Order 19 is closed. Its backend-facts authority is accepted and present on
`main`, but the result remains blocked before IR verification with
`ir_ready=0` and `ir_verify_not_implemented` as the sole blocker. It is not
`VerifiedBackendInput` and grants no artifact, IR-verification, lowering, or
execution authority.

No later session or implementation is authorized. A successor design requires
a fresh Work Order and separate BDFL authorization. Artifact encoding, SHA-256
artifact identity, `hum.ir_verify.v0`, `VerifiedBackendInput`, Cranelift or
other backend lowering, ABI work, native-stderr harness repair, open-skeleton
integration, termination work, recovery-stash cleanup, archive mutation, and
unrelated work remain unauthorized.

Work Order 18 remains closed and terminally rejected. Both WO18 archives and
recovery stash `73101039f5e3faf0c802d4f723add1b891c51602` remain immutable
failure or parking evidence only and grant no implementation authority.
<!-- workorder-current-authorization-gate:end -->
