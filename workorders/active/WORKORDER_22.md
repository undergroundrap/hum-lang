# Hum Work Order 22: Verify Backend Input And Lower Minimal Add

Date: 2026-08-15
<!-- hum-active-workorder:v1 -->
Status: CORRECTED ISSUANCE CANDIDATE. The first independent pre-issuance review
rejected contradictory evidence reachability, frozen identities, scope, and
platform detail before implementation. The bounded redesign resolves those
findings. A later BDFL ruling simplified the planning gate after repeated
synthetic-transition transport and wrapper failures occurred before classifier
invocation and discovered no package or classifier defect. The candidate
remains unaccepted and unissued.

This Work Order carries forward only the compiler-critical facts left live by
Work Order 21. Work Order 21 is closed and immutable. Its Units A and B landed;
its Unit C did not. The repository therefore still has a transitional
legacy-root Work Order fallback, but this order neither repairs nor depends on
that fallback.

The sole objective is an honest two-boundary vertical slice for canonical
`examples/core/minimal_add.hum`:

1. the compiler verifies the exact canonical backend-input artifact and issues
   a compiler-owned, non-forgeable `VerifiedBackendInput`; then
2. a Cranelift adapter consumes only that verified capability, derives the
   first real Cranelift IR instruction from authenticated Hum facts, compiles
   and executes the fixed `minimal_add` probes, and reports per-requirement
   `GO` or `NO_GO` evidence.

Unit A may set `ir_ready=1` only after the real verifier accepts the live
compiler-produced artifact. It must keep `backend_ready=0`. Unit B may report
`backend_ready=1` only on its explicit backend-probe surface after every frozen
GO row succeeds. A complete, evidence-backed `NO_GO` with readiness false is a
valid terminal result. Partial evidence, guessed lowering, hard-coded answers,
or a readiness bit forced independently of its producer is never success.

Owner: BDFL (Ocean).

## Authority, baseline, and successor transition

The planning baseline is the published WO21 terminal closeout:

- branch, `HEAD`, local `main`, cached `origin/main`, and live `main`:
  `3d02538135769fcf0d0b3207eb27489747536ca2`;
- ahead/behind: `0/0`;
- subject: `docs(workorder): close work order 21`;
- clean worktree, empty index, and no untracked files before authorship;
- workflow `ci`, run `31915632449`, attempt `1`, tested the exact closeout SHA
  and concluded `success`;
- Ubuntu job `95086898845` and Windows job `95086898801` concluded success;
  both selected the required fast status-only lane; and
- the sole published active marker was
  `workorders/active/WORKORDER_21.md:4`.

This issuance package performs exactly the canonical successor transition:

1. move `workorders/active/WORKORDER_21.md` to
   `workorders/closed/WORKORDER_21.md`;
2. remove exactly the predecessor's standalone active-marker line, preserving
   every other WO21 byte; and
3. create this file at `workorders/active/WORKORDER_22.md` with the sole exact
   marker at line 4.

The transition is not an exact routine status edit. Before any implementation
begins, a fresh independent pre-issuance reviewer must authenticate the exact
closed-WO21 and active-WO22 bytes; prove byte-preserving marker removal;
authenticate a rename-disabled three-endpoint view that deletes active WO21 and
adds closed WO21 plus active WO22; derive the exact successor tree without
creating a synthetic commit; and verify topology, scope, budgets,
satisfiability, and planning-only evidence. Git's rename-aware `R099` rendering
is informational, never the normative endpoint inventory.

Permanent classifier case `canonical_successor_issuance_full` is the
pre-issuance behavioral proof. It must execute twice deterministically in each
credited 151-case suite and return `full` / `no_status_transition`. Planning
authorship and review must not create a bundle, clone, synthetic successor
commit, `commit-tree` object, hook-based fixture, or transition wrapper. The
retired attempts failed in transport or newly written orchestration before the
production classifier ran; they found no package or classifier defect and no
further planning-time recovery attempt is authorized.

No implementation is authorized by document authorship, review, commit, or
publication. Unit A requires a fresh explicit BDFL signal after the planning
commit, full publication CI, publication-status record, that status record's
fast CI, and a separately authenticated Unit A entry state.

## Protected repository state

The eight local preservation stashes are evidence, not implementation inputs:

1. `00845bc97d66f34729e03347d6bed78d814ad95e`;
2. `44262ceec1e895a3120133e8676387f2786ae3d0`;
3. `b9093901b8c92c626c3c23ee1a52366d2e54f698`;
4. `799d4eaa2fb473633b41bbf17ad82e67fe2386a3`;
5. `f9b310902f804a0b8b7a3bf58910c7ec4f639c18`;
6. `303ee9af93696409bea66d3f8a379cb1a8cf8e1a`;
7. `bd6d2722cffa50da8463201204a48f4a7305ae1b`; and
8. `73101039f5e3faf0c802d4f723add1b891c51602`.

No unit may apply, pop, drop, reorder, rename, inspect as runtime input, or
otherwise mutate a stash. All local, cached, and live archive refs are likewise
immutable. In particular, the WO20 Unit B archive is governed only by the
bounded archaeology gate below. No archive may be merged, cherry-picked,
rebased, applied wholesale, force-updated, deleted, or presented as authority.

The canonical Work Order topology throughout implementation is exactly:

- one active regular numbered Work Order:
  `workorders/active/WORKORDER_22.md`;
- thirteen closed regular numbered Work Orders:
  `workorders/closed/WORKORDER_9.md` through
  `workorders/closed/WORKORDER_21.md`;
- zero root `WORKORDER*.md` files; and
- exactly one raw standalone active marker, at WO22 line 4.

## Repository-grounded starting facts

The current compiler already owns a narrow canonical backend input. These are
the load-bearing facts, not permission to widen it:

- schema: `hum.backend_input.v0`;
- semantic contract: `hum.canonical_minimal_add_backend_facts.v0`;
- target context: `target_independent_checked_i64_v0`;
- feature set: `canonical_minimal_add_checked_i64_v0`;
- source: `examples/core/minimal_add.hum`;
- exact live artifact size: 8,715 bytes;
- exact live payload size: 8,582 bytes;
- exact live artifact ID:
  `sha256:a37707c23cc20a1720e45de901624e3101183a77ec1b5eb4ed55095b5097b82f`;
- exact source revision:
  `sha256:aeae6ae9de975eee9873c3d9ece891e66bd7d6881b5035c24b1a11f3902a52b6`;
- exactly fourteen ordered prerequisite passes, each selected once and passed;
- one task, one checked-add operation, two ordered distinct `Int` inputs, one
  `Int` result, the normal profile, checked-empty effects/authority/ownership/
  allocation/contracts/evidence/unsupported sets, and a signed-64 checked-add
  overflow edge; and
- current verifier state `not_implemented`, current top-level
  `ready_for_ir=0`, and no honest backend readiness.

`src/backend_input.rs` already has private canonical facts and a callback-owned
access object. That object is not the finished verifier capability: its public
test-sized `is_complete` observation checks only the semantic-contract string,
and the encoded bytes remain unverified. Unit A must add an independent artifact
verifier and cross-bind the decoded artifact to the live typed facts before it
issues the new capability.

The Cranelift experiments establish feasibility only. The accepted lowering
contract maps the verified two ordered signed-64 parameters to Cranelift `I64`,
maps checked addition to `sadd_overflow`, branches the overflow flag to a
failure-status return, and uses the internal ABI
`(i64, i64, *mut i64) -> i32` with status `0` for a written result and status
`1` for checked overflow. The feasibility lab proves Cranelift 0.133.1 can JIT,
finalize, and execute that shape on the observed host. It does not prove that
production Hum facts currently reach that lowering.

## Unit A opening archive and capture gate

Unit A begins by authenticating this exact archive ref and object:

```text
archive/workorder-20-unit-b-terminal-fast-rejection-2026-08-13
94b32ca95a14072b4a22adf6e56101118650c683
```

The implementer must record the archive commit, parent, tree, complete path
inventory, per-path blobs, and diff statistics through Git object plumbing
without checking out or applying it. The archive contains hypotheses and code
that may be read selectively. It is not an accepted implementation, verifier
specification, review verdict, stash substitute, or history authority.

The archived attempt's final direct Fast process exited 1 after 286.950 seconds.
Its root and subsidiary suites reached their recorded green boundaries, but
the exact terminal failing assertion was lost. That missing assertion may
remain unknowable. No agent may infer a semantic verifier defect, recreate an
answer from chat, or spend an unbounded session reconstructing deleted process
state. Bounded archaeology ends when the archived source, Work Order history,
and durable Git/CI records have been inspected once. Remaining causal uncertainty
is recorded as `unknown` and implementation starts from current `main`.

Before the single Unit A Fast allowance can be consumed, the candidate must
include and independently prove a narrow repository-owned capture adapter. This
is an opening checkpoint inside Unit A, not a third implementation unit and not
general harness consolidation.

The adapter contract is exact:

- `tools/run_fast_evidence.ps1` owns one production command only:
  `powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools/check_all.ps1
  -EvidenceTier Fast`;
- it uses `System.Diagnostics.Process` with shell execution disabled and copies
  `StandardOutput.BaseStream` and `StandardError.BaseStream` concurrently into
  separate binary captures outside the repository;
- it retains the child process object synchronously through terminal exit and
  records whether launch occurred, PID, exit code, monotonic duration, byte
  counts, SHA-256 values, terminal nonempty stdout line, and exact terminal
  success-marker count;
- prelaunch failure is structurally different from a launched child; the one
  Fast allowance is consumed when the Fast child successfully starts, not when
  a wrapper or environment check begins;
- the adapter never retries, repairs, relaunches, merges streams, substitutes
  `Write-Host` for native capture, relies on a yielded orchestration session ID,
  or lets a wrapper assertion overwrite the child's exit record;
- missing exit state, missing stream, truncated capture, malformed result,
  duplicate completion, hash mismatch, duration inversion, or cleanup failure
  fails closed;
- a process-local PATH may expose an already installed Cargo or Git utility only
  after a fresh child proves the exact executable; every inherited PATH entry is
  preserved and no environment or Git configuration persists; and
- all captures, manifests, and synthetic children live outside the repository
  and are removed after evidence has been reported.

`tools/test_fast_evidence_capture.ps1` must exercise the exact shared capture
primitive under Windows PowerShell 5.1 and the current `pwsh`, while the
production entry remains fixed to Fast. Its synthetic cases must prove:

1. exit 0 with distinct exact stdout and stderr bytes;
2. exit 23 with both streams preserved and no success inference;
3. an empty stream remains distinguishable from a missing stream;
4. a missing executable is a prelaunch failure and consumes no allowance;
5. a launched child that exits before output remains a launched attempt;
6. large interleaved streams do not deadlock, truncate, or reorder bytes within
   either stream;
7. Unicode and CR/LF bytes retain exact captured identity;
8. missing, duplicate, malformed, or hash-inconsistent completion metadata is
   rejected;
9. the last nonempty Fast assertion is retained even when the child is red;
10. timeout kills the complete child tree once, waits for termination, retains
    partial streams and exit/termination disposition, and cannot relaunch; and
11. parent PATH, environment, current directory, credential state, and local,
    global, and system Git configuration are byte-identical afterward.

Linux capture coverage is an exact executable contract, not an inference from
full CI:

- capture adapter: `tools/run_fast_evidence.ps1`;
- synthetic-child fixture and focused test entrypoint:
  `tools/test_fast_evidence_capture.ps1`; the file self-invokes only when given
  its private `-SyntheticChild success|failure` switch, and ordinary test entry
  never exposes that switch as a production capture route;
- Ubuntu job/stage: the existing `ci` Ubuntu job's `Run Hum preflight` stage,
  before compiler selectors and before any real Fast child could start;
- required working directory: the exact repository root in
  `$env:GITHUB_WORKSPACE`, after resolving it and proving it equals the current
  directory;
- exact Ubuntu invocation:

```text
pwsh -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File ./tools/test_fast_evidence_capture.ps1 -ShellContract pwsh -ScratchRoot "$env:RUNNER_TEMP/hum-fast-capture-$env:GITHUB_RUN_ID-$env:GITHUB_RUN_ATTEMPT-ubuntu"
```

The stage inherits the ordinary GitHub Actions environment, adds no token,
askpass, credential-helper, safe-directory, PATH, or Git-configuration value,
and must not read `GH_TOKEN` or `GITHUB_TOKEN`. Before any synthetic child, an
exact `Get-Command pwsh` plus `pwsh --version` preflight must succeed in a fresh
child and resolve the same executable. If `pwsh` is absent, resolves
differently, or cannot execute, the stage stops before a capture child starts;
the launcher-before-child disposition is recorded and no real Fast allowance
is consumed.

The success child writes exact ASCII bytes `CAPTURE_STDOUT\n` to stdout and
`CAPTURE_STDERR\n` to stderr and exits 0. The failing child writes exact ASCII
bytes `FAIL_STDOUT\n` and `FAIL_STDERR\n` to the separate streams and exits 23.
The focused entrypoint must authenticate all four byte sequences, both exit
codes, and launched-child dispositions, then exit 0 with exactly one stdout
line `Fast evidence capture tests passed for pwsh.`, zero stderr, and no other
output. A started failing synthetic child is observed as started and terminal
red; for a real Fast child, that boundary consumes the allowance. A missing or
prelaunch-failed executable remains unstarted and does not consume it.

The unique scratch root must be absent before the test and absent afterward.
The stage snapshots and byte-compares parent environment, PATH order, current
directory, and local/global/system Git configuration before and after. Any
stream merge, byte/hash/exit mismatch, cleanup failure, configuration drift,
unexpected output, or Windows/Linux contract divergence is terminal-red and
stops Unit A without fallback, repair, or a Fast launch.

This capture checkpoint gets one implementation attempt and one independent
review as part of Unit A's complete-candidate review. At most one bounded
nonarchitectural correction is allowed for qualification, quoting, stream-copy
plumbing, deterministic timing bookkeeping, or exact test expectation inside
the frozen two-script envelope. A required new path, changed output meaning,
changed process ownership, changed timeout model, architectural redesign, or a
second correction stops at the BDFL. It does not create another Work Order
amendment or authorize broad `check_all.ps1` refactoring.

## Unit A - verify backend input and issue the capability

### Unit A authorization and path envelope

Unit A may begin only after the full planning lifecycle and a fresh explicit
BDFL Unit A signal. It authorizes exactly these twenty paths:

| Path | Maximum insertions | Maximum deletions | Required ownership |
| --- | ---: | ---: | --- |
| `README.md` | 6 | 3 | honest milestone/readiness summary |
| `docs/ARCHITECTURE.md` | 12 | 8 | verifier/capability boundary |
| `docs/BACKEND_CONTRACT_SCHEMA.md` | 18 | 10 | verified-input prerequisite, backend still blocked |
| `docs/CAPABILITIES_SCHEMA.md` | 8 | 4 | exact `ir-verify` command catalog entry |
| `docs/HUM_BACKEND_INPUT_SCHEMA.md` | 36 | 20 | exact implemented verifier and readiness boundary |
| `docs/HUM_IR_CONTRACT_SCHEMA.md` | 16 | 10 | artifact-to-verifier handoff |
| `docs/HUM_IR_READINESS_SCHEMA.md` | 36 | 24 | `ir_ready=1`, `backend_ready=0` semantics |
| `docs/HUM_IR_VERIFY_SCHEMA.md` | 160 | 0 | new exact human/JSON verifier contract |
| `docs/LANGUAGE_REFERENCE.md` | 12 | 8 | current narrow verifier/readiness status |
| `src/backend_contract.rs` | 10 | 6 | backend prerequisite wording only |
| `src/backend_input.rs` | 540 | 120 | typed canonical record and live cross-binding |
| `src/capabilities.rs` | 45 | 12 | command/schema capability catalog |
| `src/ir_contract.rs` | 30 | 20 | exact verifier boundary facts |
| `src/ir_readiness.rs` | 330 | 80 | live verify callback and readiness output |
| `src/ir_verify.rs` | 2,100 | 0 | new canonical decoder, verifier, report, capability |
| `src/main.rs` | 130 | 35 | `ir-verify` CLI and routing |
| `src/version.rs` | 20 | 8 | schema catalog/version parity |
| `tools/check_all.ps1` | 220 | 50 | exact selectors/catalog/fixture integration |
| `tools/run_fast_evidence.ps1` | 240 | 0 | new fixed Fast capture adapter |
| `tools/test_fast_evidence_capture.ps1` | 360 | 0 | new focused capture evidence |
| **Unit A total** | **4,329** | **418** | **no borrowing between paths** |

The same per-path and aggregate ceilings apply to raw and whitespace-insensitive
statistics. Line counts are review telemetry, not permission to weaken evidence
or create an exception. A third semantic prerequisite, an unlisted path, a new
dependency, unsafe code, mode/type change, generated artifact, or change to the
canonical example/fixture is a mandatory stop.

The path `docs/HUM_BACKEND_INPUT_SCHEMA.md` is load-bearing. Unit A must replace
its current statements that `ir_verify` is unimplemented and readiness remains
zero with the exact implemented-verifier boundary, while retaining the honest
claim that backend lowering/readiness is still absent. Omitting that update or
claiming backend readiness is a documentation-gate failure.

Unit A also has these non-borrowable category ceilings:

| Category | Maximum insertions | Maximum deletions |
| --- | ---: | ---: |
| Production Rust | 2,600 | 260 |
| Tests and proof fixtures | 900 | 90 |
| Documentation, schemas, and tools | 829 | 68 |
| **Unit A category total** | **4,329** | **418** |

Every added/deleted line is attributed exactly once. Non-`#[cfg(test)]` Rust
implementation is Production Rust. Rust `#[cfg(test)]` hunks, compile-fail and
mutation harnesses, test-only `check_all.ps1` hunks, and
`tools/test_fast_evidence_capture.ps1` are Tests and proof fixtures. Markdown,
schema/catalog material, the production capture wrapper, and non-test catalog
or driver hunks are Documentation, schemas, and tools. Both the per-path and
category ceilings apply independently to raw and whitespace-insensitive
statistics; unused capacity in either accounting system cannot cure an overage
in the other.

### Unit A public contract

Unit A adds exactly one public command:

```text
hum ir-verify [--format human|json] <backend-input-file>
```

Human is the default. JSON schema is `hum.ir_verify.v0`. Exactly one file is
accepted; stdin, directories, multiple inputs, unknown flags, and unsupported
formats are rejected as invocation errors with exit 2, no stdout, and one
ordinary stderr diagnostic. An accepted artifact exits 0 and emits exactly one
report to stdout with no stderr. A syntactically readable but rejected artifact
exits 1 and emits the deterministic rejection report to stdout with no
unexpected stderr.

The exact terminal statuses are:

- `accepted_canonical_backend_input_v0`; and
- `rejected_backend_input_v0`.

Human and JSON reports carry the same ordered facts: schema, tool version,
status, artifact ID, computed payload digest, semantic contract, compiler
version, target context, source revision, task/function/operation counts,
ordered pass count, rejected check, and ordered findings. The accepted report
contains no capability bytes or constructor token. Reports are evidence, never
capabilities.

### Unit A verification algorithm

`src/ir_verify.rs` owns a strict, independent verifier for the complete artifact
byte string. It must:

1. reject empty input, UTF-8 BOM, every CR byte, missing final LF, extra framing
   bytes, invalid UTF-8, malformed JSON, duplicate keys, unknown keys, missing
   keys, reordered keys, noncanonical escaping, noncanonical numbers, and any
   parse/re-encode difference;
2. parse the exact `hum.backend_input.v0` framing and one payload object;
3. isolate and hash the exact authenticated payload bytes, compare that digest
   with the declared lowercase `sha256:` artifact ID, preserve the decoded
   declared ID unchanged while canonically re-encoding the payload and outer
   artifact, and require the re-encoded complete bytes to equal the input;
4. require current compiler version, semantic contract, target context, feature
   set, source path/revision, module identity, and artifact schema;
5. require exactly the fourteen named passes in the frozen order, each passed,
   selected once, and bound to the same nonzero live-program identity where the
   live route is used;
6. require exactly one internal task/function and one checked-add operation;
7. require two ordered, distinct signed-64 parameter definitions with exact
   source identities/spans, an `Int` result, a distinct result definition, and
   no missing or foreign binding;
8. require normal profile, no predicates/evidence obligations, and exact empty
   effects, external authority, ownership transfers, moves, borrows, aliases,
   allocations, unsupported facts, and weakened facts;
9. require exactly the signed-64 `checked_add` overflow edge with the current
   runtime-trap semantic meaning; and
10. reject zero/multiple selections, stale/different artifact bytes, producer
    corruption seams, inconsistent counts, and any fact it cannot authenticate.

The digest cases are frozen and nondegenerate: the honest artifact must pass
with its declared ID unchanged; changing only payload bytes while retaining the
declared ID must fail A-R02; changing only the declared ID over unchanged
payload must fail A-R03; a semantic payload corruption with its newly correct
payload digest must pass the hash check and fail its later semantic row; and a
mutation that has canonical re-encoding recalculate or replace the declared ID
must fail the declared-ID-preservation case. Thus hashing, declared-ID
preservation, identity agreement, canonical encoding, and semantic validation
cannot satisfy one another's credit.

No verification check may be satisfied from a fixture, report text, a prefilled
success enum, the expected constant copied into the verifier, or the existing
facts access object's shallow `is_complete` method. The file command and live
compiler callback must share the same byte verifier.

Unit A owns the complete invalid-input boundary. The following ten rejection
classes are distinct evidence rows but are not backend GO/NO-GO credit:

| ID | Rejected before capability issuance | Required observation |
| --- | --- | --- |
| A-R01 | framing, UTF-8, or noncanonical JSON | exact byte/canonical diagnostic; callback count 0 |
| A-R02 | authenticated payload bytes hash to a different digest | payload-digest mismatch; callback count 0 |
| A-R03 | declared artifact ID is changed, rewritten, or disagrees with the payload digest | declared-ID mismatch/preservation diagnostic; callback count 0 |
| A-R04 | schema, compiler, semantic, target, feature, source-path, or source-revision identity is stale/substituted | exact identity diagnostic; callback count 0 |
| A-R05 | prerequisite pass order, selection, result, or binding is invalid | exact pass/binding diagnostic; callback count 0 |
| A-R06 | task/function/parameter/definition/operation/result/type/span structure is invalid | exact structural diagnostic; callback count 0 |
| A-R07 | profile or any checked-empty set is unsupported/nonempty | exact profile/set diagnostic; callback count 0 |
| A-R08 | checked-add overflow/failure-edge facts disagree | exact overflow-edge diagnostic; callback count 0 |
| A-R09 | live logical identities are stale, mixed, substituted, or disagree with decoded facts | exact live-cross-binding diagnostic; callback count 0 |
| A-R10 | raw bytes, report, fixture, or fabricated typed data attempts authority | compile-time/privacy rejection; no capability and no backend call |

Each row starts from the same accepted control and changes only its named
dimension. The permanent matrix must prove that the immediately preceding and
following rows do not satisfy that assertion, that the capability callback and
a backend-call sentinel remain uninvoked for every A-R01 through A-R09
rejection, and that removing only the row's production check makes its exact
case reach the capability sentinel while all nonadjacent rows remain rejected.
A-R10 is load-bearing through compile-fail privacy mutations. This prevents a
verifier rejection from being recycled as Unit B evidence.

### `VerifiedBackendInput` authority

`VerifiedBackendInput` is compiler-owned, private-field, and non-forgeable. Its
constructor remains private to `src/ir_verify.rs`; it implements neither
`Default`, `Clone`, `Copy`, serialization, deserialization, string parsing, nor
conversion from raw artifact/report/source types. It is issued only inside a
higher-ranked callback after all verification succeeds and is lifetime-bound to
the exact artifact bytes and live typed facts that were cross-checked.

The live factory must follow this exact authority path:

```text
Program + diagnostics
  -> current canonical typed backend facts
  -> canonical backend-input encoding
  -> independent byte verifier
  -> cross-check decoded record against the same live typed facts
  -> callback-scoped VerifiedBackendInput
```

Program provenance is logical, not allocational. Two independently parsed
Programs with identical authenticated source revision and identical ordered
semantic identities for passes, task/function, parameters, definitions,
operation, result, and spans must both cross-bind successfully to the same
canonical artifact. Pointer, allocation, and parser-instance identity are not
authority. A dedicated positive case reparses the identical source and proves
acceptance. Separate negative controls use a second authenticated logical
context that differs in exactly one pass, definition, or operation/result
identity, then substitute or mix that component with the canonical artifact;
each must fail only A-R09 before capability issuance. Replacing the logical
comparison with a pointer comparison must make the positive reparse fail,
while accepting one mixed identity must make its exact negative case reach the
sentinel.

The capability owns or borrows an immutable typed verified projection. It
exposes only crate-private getters needed by the future adapter:

- schema/artifact/compiler/semantic/target/source identities;
- exact task/function identity and internal linkage;
- ordered parameter IDs, definition IDs, types, and source spans;
- operation/result IDs, checked-add operator, result type, and source span;
- exact overflow/failure-edge facts;
- profile and each checked-empty set; and
- ordered prerequisite-pass evidence.

It must not expose raw JSON as the Unit B authority. The future adapter may not
reparse artifact bytes, inspect source/AST/Core directly, consume reports, or
call an alternate facts accessor. Compile-fail evidence must prove private
construction, field access, lifetime escape, fabricated conversion, and
raw-artifact substitution all fail.

### Unit A readiness transition

The canonical live `minimal_add` readiness report advances only as follows:

| Boundary | `ir_ready` | `backend_ready` | Required status |
| --- | ---: | ---: | --- |
| Published parent | 0 | 0 | blocked before IR verification |
| Artifact encoding succeeds but verification is skipped/rejected | 0 | 0 | blocked at `ir_verify` |
| Real live artifact verifies and callback receives the capability | 1 | 0 | `ready_for_ir_with_verified_backend_input_v0` |

The top-level human and JSON readiness reports must carry both exact fields.
Legacy `ready_for_ir` remains an exact parity alias of top-level `ir_ready` for
this schema version; inconsistent values fail tests. All earlier substage
`ir_ready` fields retain their existing meanings and may not be rewritten to
claim final verification. The `ir_verify` pass becomes implemented, selected
once, passed once, and bound to the same live program. Facts replace
`ir_verify_pending_v0` and
`canonical_backend_input_bytes_produced_unverified_v0` with exact verified
facts. `backend_ready` remains 0 because no backend adapter has consumed the
capability.

### Unit A exact evidence

Focused Rust selectors must each select exactly one test:

```text
backend_input::tests::minimal_add_backend_input_bytes_are_canonical_and_deterministic
ir_verify::tests::canonical_minimal_add_artifact_corruption_matrix_is_complete
ir_verify::tests::verified_backend_input_is_sealed_typed_and_lifetime_bound
ir_readiness::tests::canonical_minimal_add_is_ir_ready_only_after_live_verification
```

The permanent adversarial matrix must include, at minimum:

- every framing, UTF-8, canonical-JSON, key, count, order, digest, stale-version,
  semantic-contract, target-context, feature-set, source-path/revision, pass,
  selection, binding, task/function, parameter, definition, type, operation,
  result, span, profile, checked-empty set, and overflow-edge corruption;
- a valid digest over semantically corrupted content;
- semantically valid content with a stale or foreign digest;
- raw bytes/report/fixture/fabricated typed data attempting capability issuance;
- an independently reparsed byte-identical Program with the same authenticated
  logical identities successfully cross-binding;
- separate substituted and mixed pass/definition/operation identities across
  two otherwise identical parses failing before capability issuance;
- a verifier callback that is never invoked on any rejection; and
- exact human/JSON parity and CLI exit/stdout/stderr contracts.

Required load-bearing mutations are:

| Mutation | Must fail at |
| --- | --- |
| bypass exact payload-byte digest comparison | payload-digest corruption cases |
| rewrite the declared artifact ID during re-encoding | declared-ID preservation and mismatch cases |
| bypass canonical re-encoding | noncanonical but parseable JSON cases |
| weaken one semantic completeness row | that row's exact corruption case |
| accept zero or foreign pass selection | pass selection/binding cases |
| expose or add a capability constructor/conversion | compile-fail privacy evidence |
| require pointer/parser-instance identity | independent-reparse portability case |
| issue capability before live-fact cross-check | substituted/mixed logical-identity cases |
| force `ir_ready=1` without callback success | readiness producer assertion |
| let report or fixture mint capability | capability origin assertion |

The capture test runs before compiler selectors. Then run format/check/Clippy,
the root and applicable subsidiary suites, exact selector inventory, human/JSON
CLI probes, fixture identity, documentation/catalog parity, text hygiene,
public readiness, alpha claims, and release readiness. The implementer does not
run Fast or local Exhaustive. After all focused evidence is green, the fresh
independent complete-candidate reviewer audits the capture adapter first and
then starts exactly one Fast child through it. Any launched Fast failure stops
without repair or rerun. Full publication CI must pass Ubuntu and Windows; the
Ubuntu full lane owns the existing platform-independent Exhaustive producer and
Windows skips only that duplicate producer.

Exact proposed Unit A commit subject:

```text
feat(ir): verify canonical backend input
```

Unit A's publication evidence must be recorded in a separately authorized
status/current-gate edit, committed and published separately, with dual-platform
fast status CI. Unit B remains unauthorized until that entire lifecycle is
terminal-green and a fresh BDFL signal is issued.

## Unit B - lower verified minimal add with Cranelift

### Unit B authorization and path envelope

Unit B may begin only from the accepted, published, status-recorded, terminal-
green Unit A tree. It authorizes exactly these nineteen paths:

| Path | Maximum insertions | Maximum deletions | Required ownership |
| --- | ---: | ---: | --- |
| `Cargo.toml` | 10 | 0 | pinned production Cranelift dependencies only |
| `Cargo.lock` | 550 | 30 | mechanical lock update only |
| `README.md` | 14 | 8 | honest first-lowering status |
| `docs/ARCHITECTURE.md` | 12 | 8 | verified capability to adapter boundary |
| `docs/BACKEND_STRATEGY.md` | 30 | 16 | first production backend rung and deferrals |
| `docs/BACKEND_CONTRACT_SCHEMA.md` | 45 | 25 | exact checked-add lowering/ABI contract |
| `docs/CAPABILITIES_SCHEMA.md` | 12 | 6 | exact `backend-probe` catalog entry |
| `docs/HUM_IR_CONTRACT_SCHEMA.md` | 30 | 18 | verified fact to CLIF mapping |
| `docs/HUM_IR_READINESS_SCHEMA.md` | 60 | 35 | distinguish IR readiness from explicit backend probe |
| `docs/HUM_BACKEND_PROBE_SCHEMA.md` | 170 | 0 | new GO/NO-GO report contract |
| `docs/LANGUAGE_REFERENCE.md` | 18 | 10 | exact supported execution slice |
| `src/backend_contract.rs` | 35 | 20 | machine-readable lowering contract rows |
| `src/backend_cranelift.rs` | 1,150 | 0 | new verified-only adapter, JIT, probe report |
| `src/capabilities.rs` | 60 | 20 | command/schema capability catalog |
| `src/ir_contract.rs` | 45 | 30 | lowering identity and provenance rows |
| `src/ir_readiness.rs` | 400 | 150 | static eligibility and honest readiness separation |
| `src/main.rs` | 240 | 90 | explicit backend-probe CLI and routing |
| `src/version.rs` | 25 | 12 | schema/version catalog parity |
| `tools/check_all.ps1` | 420 | 130 | selectors, probes, cross-platform gates |
| **Unit B total** | **3,326** | **608** | **no borrowing between paths** |

The same raw and whitespace-insensitive ceilings apply. `Cargo.lock` is
mechanical but remains inside the ceiling. The exact read-only fixtures are
`examples/core/minimal_add.hum` and
`fixtures/backend_input/minimal_add.backend_input.v0.json`; neither may change.
No new fixture, build script, workflow, unsafe helper, public library API,
runtime subsystem, AOT/object/link path, or twentieth modified path is implied.

Unit B also has these non-borrowable category ceilings:

| Category | Maximum insertions | Maximum deletions |
| --- | ---: | ---: |
| Production Rust | 1,700 | 320 |
| Tests and proof fixtures | 650 | 150 |
| Documentation, schemas, and tools | 426 | 108 |
| Dependency manifests and lock data | 550 | 30 |
| **Unit B category total** | **3,326** | **608** |

The Unit A attribution rules continue. `Cargo.toml` and `Cargo.lock` alone are
Dependency manifests and lock data. Every line is attributed once, and neither
per-path nor category headroom may be borrowed, under raw or whitespace-
insensitive accounting.

### Sequential overlap and Unit B baseline

Exactly fourteen paths occur in both unit envelopes:

| Overlap path | Why shared | Unit A responsibility | Later Unit B responsibility |
| --- | --- | --- | --- |
| `README.md` | public milestone truth | verified-IR milestone, backend blocked | first-lowering/GO-or-NO-GO milestone |
| `docs/ARCHITECTURE.md` | authority architecture | verifier and sealed capability | capability-to-Cranelift boundary |
| `docs/BACKEND_CONTRACT_SCHEMA.md` | backend prerequisites | verified-input prerequisite only | exact lowering, ABI, and row contract |
| `docs/CAPABILITIES_SCHEMA.md` | command catalog | `ir-verify` command | `backend-probe` command |
| `docs/HUM_IR_CONTRACT_SCHEMA.md` | IR/backend handoff | authenticated artifact-to-capability facts | fact-to-CLIF mapping |
| `docs/HUM_IR_READINESS_SCHEMA.md` | readiness truth | `ir_ready=1`, `backend_ready=0` | host-local and published backend evidence |
| `docs/LANGUAGE_REFERENCE.md` | supported-language status | narrow verification support | narrow execution support |
| `src/backend_contract.rs` | machine-readable boundary | verified prerequisite wording | lowering/ABI/backend rows |
| `src/capabilities.rs` | command/schema catalog | verifier capability entry | backend-probe capability entry |
| `src/ir_contract.rs` | contract identities | verifier identity/cross-binding | lowering identity/provenance |
| `src/ir_readiness.rs` | readiness producer | live verifier callback and IR state | backend eligibility and readiness separation |
| `src/main.rs` | CLI routing | `ir-verify` route | `backend-probe` route |
| `src/version.rs` | version/schema parity | verifier schema | backend-probe schema |
| `tools/check_all.ps1` | repository evidence driver | verifier/capture selectors and catalogs | backend selectors and cross-platform gates |

No overlap is concurrent ownership. Unit B is unauthorized until Unit A is
independently accepted, committed, published, status-recorded, and the status
commit's Ubuntu and Windows fast lanes are terminal-green, followed by a fresh
BDFL Unit B signal. The Unit A publication-status lifecycle must durably record
the accepted Unit A commit and one sorted manifest for all twenty Unit A paths:
path, regular mode, final Git blob, raw statistics, and whitespace-insensitive
statistics. Those observed identities are recorded only after they exist; this
Work Order invents no future blob.

Before editing, Unit B must authenticate that recorded commit as local, cached,
and live `main`, reproduce all twenty manifest entries from Git objects, and
reproduce the fourteen overlap blobs byte-for-byte in its worktree. A mismatch
stops before mutation. Unit B per-path, category, and aggregate statistics are
then computed against those accepted Unit A blobs, never against the original
WO22 planning base. Unit B's status record must preserve the consumed Unit A
manifest and record its own observed final manifest for review.

Root production dependencies are pinned exactly to Cranelift `0.133.1`:

- `cranelift-codegen`;
- `cranelift-frontend`;
- `cranelift-jit`;
- `cranelift-module`; and
- `cranelift-native`.

Default features must be audited and minimized without introducing a feature
matrix the repository does not test. A dependency-version change, second
backend, serde/report framework, async runtime, linker driver, or build-script
surface is a stop.

### Unit B public probe and terminal dispositions

Unit B adds exactly one explicit execution command:

```text
hum backend-probe [--format human|json] examples/core/minimal_add.hum
```

The command accepts only the canonical path and current exact source revision.
It is opt-in and may JIT/execute only the compiler-generated fixed
`minimal_add` function with the frozen integer probes below. Ordinary `check`,
`ir-readiness`, `backend-input`, and `ir-verify` commands remain non-executing.

The report schema is `hum.backend_probe.v0`. It records `decision=GO|NO_GO`,
`ir_ready`, `backend_ready`, target triple, Cranelift version, artifact ID,
source revision, verified capability origin, the fifteen ordered runtime row
results, emitted CLIF identity, compile/finalize disposition, and per-probe
result. It contains no timing field in the canonical result because timing is
evidence metadata, not a semantic readiness fact.

Exit behavior is exact:

- `GO`: exit 0, one complete report on stdout, zero stderr,
  `ir_ready=1`, `backend_ready=1`;
- evidence-backed `NO_GO`: exit 3, one complete report on stdout, zero
  unexpected stderr, `backend_ready=0`, and the exact blocking row(s);
- invocation misuse: exit 2, no stdout, one ordinary stderr diagnostic; and
- internal evidence loss or malformed adapter result: exit 1, no readiness
  claim, and a fail-closed diagnostic.

An accurate `NO_GO` is a valid terminal Work Order result, not permission to
change scope. It receives independent factual review and returns to the BDFL.
It cannot recommend a GO implementation commit, set backend readiness, or
invent a fallback. The BDFL may close WO22 terminally incomplete or separately
authorize a new plan; no repair follows implicitly.

### Verified-only adapter and first real instruction

`src/backend_cranelift.rs` accepts exactly one semantic input type:

```text
&VerifiedBackendInput<'_>
```

There is no overload or alternate route for `Program`, AST, Core, readiness
reports, backend-input bytes, JSON, source text, expected fixture data, or
primitive operands. The adapter reads typed getters only. Unit B negative
evidence always enters from the unchanged Unit A live factory and an honestly
issued valid capability; malformed, stale, substituted, mixed, or otherwise
invalid input is rejected and credited only in Unit A.

One crate-private `#[cfg(test)]` `BackendProbeFault` may expose exactly the
fifteen backend-stage failures frozen below through a private test entrypoint.
Production always uses `None`. The enum and entrypoint cannot construct,
clone, retain, or modify a capability; cannot change verified facts; and cannot
enter CLI routing in a non-test build. Every fault is injected only after the
valid-capability sentinel and immediately at its named backend boundary.
Compile-fail evidence proves the seam is absent outside the backend test module.

The exact supported mapping is:

| Verified Hum fact | Required Cranelift result |
| --- | --- |
| one internal `minimal_add` function | one internal JIT function definition |
| ordered distinct `Int` inputs | two ordered `I64` block parameters |
| checked-add operation | one `sadd_overflow` instruction derived from the verified operator |
| checked-add result | one `I64` sum value written only on success |
| signed overflow edge | `brif` to status-1 failure return |
| normal edge | store sum through result slot and return status 0 |
| authenticated source span | non-default Cranelift `SourceLoc` on the emitted operation |
| checked-empty unsupported/effect/resource facts | permission for this mapping and no broader one |

The internal ABI is exactly `(i64, i64, *mut i64) -> i32`. The result pointer is
non-null and valid only inside the fixed internal probe wrapper. Status 0 means
the result slot was written. Status 1 means signed overflow and the result slot
was not semantically produced. No native wraparound, saturating add, unchecked
`iadd`, interpreter delegation, constant-result table, source-name special case,
or prebuilt CLIF string can satisfy the mapping.

The only `unsafe` allowed in WO22 is the smallest reviewed JIT invocation
boundary required to convert the finalized Cranelift code pointer to the exact
internal function signature and call it. It must be in one named function with
a `SAFETY:` comment proving lifetime, finalized-module ownership, ABI, pointer,
and result-slot validity. No unsafe parser, artifact verifier, raw allocation,
global mutable state, transmute elsewhere, FFI library surface, or widening of
the source language is authorized.

### Mandatory GO/NO-GO matrix

Every row is emitted exactly once in this order. Missing, duplicate, reordered,
aggregated, or preselected rows fail closed. Every primary `NO_GO` names the
exact backend property, owner, observed value, and required value without
attempting a repair. If one row prevents a dependent stage from executing, the
later row is emitted as `NO_GO:blocked_by_<ID>` and receives no independent
property credit. Focused evidence for each row begins with every earlier row GO
and makes only that row the primary NO_GO.

| ID | Producer | Consumer | GO condition | Exact NO-GO class | Blocks `backend_ready` |
| --- | --- | --- | --- | --- | --- |
| B01 | valid Unit A callback | adapter admission | the valid callback-scoped capability reaches the backend admission boundary | `verified_capability_admission_unavailable` | yes |
| B02 | pinned Cranelift dependency | backend compatibility gate | exact 0.133.1 API/engine contract is available | `unsupported_cranelift_api` | yes |
| B03 | verified internal function identity | module declaration plan | one deterministic internal linkage/name can be declared | `function_declaration_unsupported` | yes |
| B04 | verified ordered parameters and result | signature/block builder | exact `(i64,i64,*mut i64)->i32` signature and two ordered `I64` block parameters can be built | `abi_construction_failed` | yes |
| B05 | verified checked-add operator/result | instruction selector | fact-derived `sadd_overflow` and sum result can be emitted | `checked_add_selection_failed` | yes |
| B06 | verified overflow and normal edges | CFG/status builder | branch, success store/status 0, and overflow status 1 can be emitted without a result on failure | `overflow_control_flow_failed` | yes |
| B07 | verified operation span | source-map builder | the operation maps to a non-default exact `SourceLoc` | `source_location_mapping_failed` | yes |
| B08 | required host target | ISA builder | exact x86_64 Windows-MSVC or x86_64 Linux-GNU native ISA initializes | `unsupported_or_unavailable_target` | yes |
| B09 | completed CLIF function | Cranelift verifier | emitted function verifies with the fact-derived instruction, CFG, ABI, and source location | `cranelift_verification_failed` | yes |
| B10 | verified function declaration plan | JIT module | internal function is declared with the exact signature/linkage | `jit_declaration_failed` | yes |
| B11 | verified CLIF body | JIT module | exact declared function body is defined | `jit_definition_failed` | yes |
| B12 | defined JIT function | JIT module | module finalizes and yields the exact owned code pointer | `jit_finalization_failed` | yes |
| B13 | finalized function | ordinary probe runner | all four ordinary cases execute and match an independent checked Rust oracle | `ordinary_execution_mismatch` | yes |
| B14 | same finalized function | overflow probe runner | both overflow cases return status 1 and no claimed result | `overflow_execution_mismatch` | yes |
| B15 | all preceding backend evidence | report/readiness gate | fifteen ordered rows, execution facts, and readiness are complete, deterministic, and internally consistent | `incomplete_backend_evidence` | yes |

The exact `BackendProbeFault` variants in row order are
`RejectVerifiedAdmission`, `RejectPinnedCraneliftApi`,
`RejectFunctionDeclaration`, `RejectAbiConstruction`,
`RejectCheckedAddSelection`, `RejectOverflowControlFlow`,
`RejectSourceLocation`, `RejectTargetIsa`, `RejectClifVerification`,
`RejectModuleDeclaration`, `RejectFunctionDefinition`,
`RejectFinalization`, `CorruptOrdinaryExecution`,
`CorruptOverflowExecution`, and `DropEvidenceRow`. A static exhaustive match and
an exact count-15 assertion forbid an alias, omitted variant, or catch-all.

The fifteen negative fixtures invoke the private backend test entrypoint inside
the valid Unit A factory callback. Each proves the valid-capability sentinel,
then selects exactly one same-named `BackendProbeFault`, observes its exact
primary NO_GO with `backend_ready=0`, and proves all earlier rows GO. The
immediately preceding and following fault variants must not satisfy its
assertion. A paired disposable source mutation removes or ignores only the
corresponding production check and must make that permanent case fail; the
fault enum alone is never accepted as proof of its own handling. The valid
control reaches B15 GO. Unit A A-R01 through A-R10 cases separately prove the
backend entrypoint was never called and receive no Unit B row credit.

The exact ordinary probes are:

| Left | Right | Result | Status |
| ---: | ---: | ---: | ---: |
| 2 | 3 | 5 | 0 |
| -7 | 11 | 4 | 0 |
| 0 | 0 | 0 | 0 |
| 1,000,000 | 24 | 1,000,024 | 0 |

The exact overflow probes are `i64::MAX + 1` and `i64::MIN + -1`; each must
return status 1 and must not claim a result. The same function produced from the
same verified capability runs all six cases. Recompiling or substituting a
case-specific function is forbidden.

Only x86_64 Windows-MSVC and x86_64 Linux-GNU are required GO configurations.
All other architectures, operating systems, environments, calling conventions,
or unavailable native-ISA builders are explicitly unexercised and must produce
B08 `NO_GO`, never a compile error or fallback. Unit reports must list every
unexercised configuration honestly.

Cross-platform publication is a separate lifecycle gate, not a sixteenth
runtime row that a local process could predict. A host-local backend probe may
report `backend_ready=1` only after B01-B15 are all GO on a required target.
The repository-wide WO22 milestone may be recorded as `backend_ready=1` only
after the exact commit is published and both required full lanes independently
reproduce that GO evidence. No ordinary `ir-readiness` invocation runs JIT code
or remembers prior process evidence.

### Unit B adversarial and mutation evidence

Focused selectors must each select exactly one test:

```text
backend_cranelift::tests::verified_minimal_add_emits_checked_cranelift_ir
backend_cranelift::tests::minimal_add_jit_probe_matrix_is_exact
backend_cranelift::tests::backend_go_no_go_rows_are_complete_and_load_bearing
backend_cranelift::tests::unsupported_targets_are_explicit_no_go
```

The candidate must prove:

- raw artifact, JSON, source, AST, Core, report, primitive operands, or expected
  probe values cannot satisfy the adapter signature;
- every backend negative begins after the valid-capability sentinel; no test
  seam fabricates, edits, retains, or substitutes verified input;
- declaration/linkage, ABI/block-parameter construction, fact-derived checked-
  add selection, overflow control flow, source location, target ISA, CLIF
  verification, JIT declaration/definition/finalization, both execution
  classes, and row inventory are independently load-bearing;
- `sadd_overflow` comes from the verified operator fact, not a constant branch
  selected because the source path or function is named `minimal_add`;
- CLIF verification and JIT declaration, definition, or finalization failures
  become B09, B10, B11, or B12 primary `NO_GO` with no invocation;
- a dropped/default operation `SourceLoc` becomes B07 primary `NO_GO`;
- unsupported subtraction, extra operations/functions/parameters, reordered
  inputs, foreign types, nonempty effects/resources/unsupported facts, stale
  identities, and mixed logical provenance remain Unit A rejections and must
  never produce a backend row;
- invalid/null result-slot arrangements never cross the unsafe boundary;
- ordinary and overflow expected values are computed by an independent checked
  Rust oracle in tests, not copied from the adapter; and
- human/JSON output and exit/stdout/stderr contracts remain exact.

Required targeted mutations are:

| Mutation | Required failure |
| --- | --- |
| add raw bytes/source overload | verified-only boundary/compile-time evidence before B01 |
| bypass valid callback admission | B01 focused admission evidence |
| hide pinned API/version rejection | B02 compatibility case |
| alter internal linkage/name declaration | B03 declaration case |
| swap parameter getters or alter signature | B04 ABI and ordinary execution evidence |
| replace fact-derived selection with hard-coded add or `iadd` | B05 and overflow probes |
| omit overflow branch or write result on overflow | B06 and B14 |
| drop or default the operation `SourceLoc` | B07 |
| force target support or hide ISA failure | B08 unsupported-target case |
| skip Cranelift verification | B09 verification case |
| hide JIT declaration, definition, or finalization failure | B10, B11, or B12 exact case |
| return expected ordinary constants without finalized invocation | B13 execution-origin assertion |
| claim a result on overflow or bypass the finalized call | B14 execution-origin assertion |
| force `backend_ready=1` with one row pending/NO_GO | B15 readiness assertion |
| delete, duplicate, reorder, or aggregate one row | complete row inventory |

### Unit B validation and terminal gate

The implementation order is fixed:

1. authenticate the accepted Unit A commit as local/cached/live `main`, its
   status-recorded twenty-path blob/stat manifest, all fourteen overlap blobs,
   Work Order gate, refs, stashes, fixtures, and supported host tools;
2. add pinned dependencies and compile the empty module boundary;
3. implement verified-only admission and GO/NO-GO rows before JIT invocation;
4. implement fact-derived CLIF, verification/finalization, and the narrow unsafe
   invocation boundary;
5. run focused positive, negative, unsupported-target, mutation, and CLI tests;
6. run format/check/Clippy, root and subsidiary suites, exact selectors, docs,
   catalogs, text hygiene, public readiness, alpha, and release readiness;
7. leave the exact candidate unstaged and uncommitted for fresh independent
   complete-tree review; and
8. only after every focused gate is green may that reviewer start exactly one
   Fast child through the accepted Unit A capture adapter.

No local Exhaustive run is required. A launched Fast failure, evidence loss,
GO/NO-GO row failure, unsupported required host, mutation failure, budget/path
breach, unsafe widening, or missing Cargo/Cranelift tool stops without repair or
rerun unless a later BDFL signal stays inside the already accepted architecture.

Exact proposed Unit B GO commit subject:

```text
feat(backend): lower verified minimal add with cranelift
```

Only an unqualified independent ACCEPT of a GO candidate may recommend that
commit. Publication is a separate normal non-force main push. Ubuntu and
Windows must both run full CI. Ubuntu must run the platform-independent
Exhaustive producer and Windows must skip only its duplicate. A terminal-red
lane, cross-platform evidence mismatch, target disagreement, missing evidence,
or different classifier binding stops. The publication/status lifecycle then
follows the same separately gated commit, push, and dual-platform fast status-
CI sequence as Unit A. WO22 closeout is separately authorized only afterward.

## Cross-unit invariants

The following invariants bind both units:

1. The artifact producer, verifier, capability, adapter, and report have one
   acyclic authority chain. No consumer can become an alternate producer.
2. Byte validity, canonical encoding, digest validity, semantic completeness,
   live-fact identity, capability origin, lowering eligibility, target support,
   compilation, and execution are separate checks with separate evidence.
3. A report is never authority. A fixture is never authority. A source path or
   expected output is never authority. Only the callback-scoped capability
   crosses the Unit A-to-B semantic boundary.
4. `ir_ready=1` means real live verification succeeded. Host-local
   `backend_ready=1` means all fifteen runtime rows are GO on one required
   target. The repository-wide milestone additionally requires separately
   observed required-platform CI. Neither is inferred from code presence, test
   names, a fixture, or a prior chat report.
5. The canonical backend input remains narrow: one `minimal_add`, checked
   signed-64 addition, normal profile, and checked-empty side conditions.
6. The adapter may reject all other shapes. It may not generalize addition,
   introduce new source syntax/semantics, optimize, or silently route to the
   interpreter.
7. Human and JSON reports are deterministic projections of the same facts and
   preserve unknown, unsupported, pending, GO, and NO-GO distinctly.
8. Every candidate remains unstaged for one fresh complete independent review.
   Acceptance authorizes only the stated local commit; every push, status edit,
   next unit, and closeout remains separate.
9. Current Work Order topology, the sole marker, eight stashes, all archives,
   remote refs, credential state, and unrelated files remain unchanged.
10. No test may satisfy a credited property by source-text counting when the
    executable boundary can be observed.

## Explicit deferrals and dead alternatives

WO22 does not authorize:

- general evidence-harness consolidation, arbitrary subprocess execution, a
  shared orchestration framework, CI workflow redesign, or replacement of the
  existing preflight driver;
- semantic-coordinate or canonical cognitive-layout implementation;
- adding provenance/status coordinates to production schemas before the first
  real lowering;
- inferred/declared/conflict coordinate states, presentation-only coordinates,
  semantic projections/digests, or broad positive/negative coordinate corpora;
- broad `src/main.rs` or CLI cleanup, parser refactoring, compiler-pipeline
  reorganization, new source syntax, general IR, optimization, register
  allocation policy, object/AOT production, native linking, runtime design,
  standard-library work, or a second backend;
- WO21 Unit C salvage, legacy-root fallback removal, restoration or use of
  either Unit C stash, or any Work Order topology work beyond this issuance;
- archive/stash mutation, history rewriting, replace/graft refs, force push,
  branch deletion, repository-setting changes, secrets, persistent PATH/env/Git
  configuration, or generated proof artifacts in the repository;
- treating the WO20 archive, experiments, research, or chat as authority;
- reconstructing an unsupported readiness claim from missing evidence; and
- WO23 drafting, release/tag work, or unrelated documentation cleanup.

The rejected alternatives are explicit: do not retry WO20 wholesale; do not
let the backend parse raw JSON; do not make `ir-readiness` execute JIT code; do
not treat Cranelift feasibility as production lowering; do not use hard-coded
`minimal_add` answers; and do not widen the narrow capture adapter into a
general harness project.

## Post-milestone advisories

Only after the first real lowering is accepted, published, status-recorded, and
terminal-green should the BDFL reconsider:

1. a dedicated harness-consolidation Work Order for repository-owned
   synchronous capture, one machine-readable result channel, deterministic
   timeout/child cleanup, and tests for lost stdout/stderr/nonzero exits and
   host-versus-pipeline behavior; and
2. semantic-coordinate research with explicit provenance and status, including
   inferred/declared/conflict states, presentation-only coordinates, semantic
   projections and digests, and representative positive/negative corpora.

These are advisories, not backlog authority. The first real lowering may reveal
that different abstractions are needed. No file, schema, dependency, or session
is reserved for them here.

## Review and publication lifecycle

Each unit follows this exact chain with no implied next step:

1. fresh explicit BDFL implementation signal;
2. implementer authenticates the exact baseline and leaves a complete unstaged
   candidate with empty index and no untracked artifacts;
3. fresh independent reviewer inspects the complete diff, producer/validator/
   consumer path, adversarial probes, targeted mutations, platform claims,
   path/budget inventory, and final state;
4. the reviewer runs the single allowed Fast process only after every focused
   gate and capture requirement is green;
5. only unqualified ACCEPT may recommend the exact local commit subject;
6. BDFL separately authorizes the normal non-force main push and terminal full
   Ubuntu/Windows CI inspection;
7. BDFL separately authorizes the immutable publication-status edit and local
   commit;
8. BDFL separately authorizes that status commit's push and terminal fast
   Ubuntu/Windows CI; and
9. only a fresh BDFL signal starts the next unit or closeout.

Any red job, classifier disagreement, unexpected path, evidence loss, forged or
ambiguous authority, missing mutation, budget breach, unsafe expansion, stash/
archive drift, or review finding stops at its actual boundary. No retry,
correction, commit, push, status edit, next unit, or closeout follows
implicitly. The direct Fast allowance does not transfer between attempts or
units and is consumed only by a child that actually starts.

## Aggregate implementation budget

| Unit | Maximum insertions | Maximum deletions |
| --- | ---: | ---: |
| Unit A verifier, capability, readiness, and narrow capture | 4,329 | 418 |
| Unit B verified Cranelift lowering and probe | 3,326 | 608 |
| **Maximum aggregate** | **7,655** | **1,026** |

The same ceilings govern whitespace-insensitive accounting. No unit may borrow
another unit's path or budget. Review reports per-path raw and whitespace-
insensitive statistics, modes, blob identities, and whether changes are
semantic, evidence, documentation, dependency-lock, or new-file content.

Path arithmetic is exact: twenty Unit A entries plus nineteen Unit B entries
minus the fourteen explicitly listed sequential overlaps equals twenty-five
distinct authorized repository paths. Evidence arithmetic is likewise
nonduplicative: ten Unit A invalid-input rejection rows stop before backend
entry; fifteen Unit B runtime rows start only after valid capability issuance.
Neither ledger grants credit to the other.

Aggregate category ceilings are likewise exact and non-borrowable:

| Category | Maximum insertions | Maximum deletions |
| --- | ---: | ---: |
| Production Rust | 4,300 | 580 |
| Tests and proof fixtures | 1,550 | 240 |
| Documentation, schemas, and tools | 1,255 | 176 |
| Dependency manifests and lock data | 550 | 30 |
| **Aggregate category total** | **7,655** | **1,026** |

The aggregate table is the arithmetic sum of the two unit category tables, not
a pool. A Unit A or Unit B category failure cannot borrow from its sibling or
from another category even when the aggregate would remain below its ceiling.

## Planning-package validation

Document authorship runs only:

- `git diff --check`;
- a fail-closed no-index whitespace check covering the moved predecessor and
  untracked successor;
- two complete independent executions of the published 151-case status
  classifier suite, each internally twice deterministic;
- raw stdout/stderr comparison for those executions, including byte counts,
  line endings, SHA-256, exit codes, and zero unexpected stderr;
- proof that `canonical_successor_issuance_full` executes once per suite
  invocation and twice internally as `full` / `no_status_transition`;
- canonical topology and exact-one-marker checks;
- proof that closed WO21 differs from its published blob only by deletion of the
  exact marker line and otherwise reconstructs byte-for-byte;
- rename-disabled proof of the exact delete-active-WO21, add-closed-WO21, and
  add-active-WO22 endpoints, exact endpoint blobs, and the derived successor
  tree identity, without creating a commit;
- text hygiene and public readiness for the resulting repository file count;
- alpha claims; and
- release readiness for `0.0.1`.

No Cargo, Rust selector, Fast, full preflight, Exhaustive, compiler probe,
archive code execution, backend-input verification, Cranelift lowering, JIT,
CI, performance probe, implementation check, production-classifier invocation,
bundle, clone, synthetic commit, or transition wrapper is authorized during
planning authorship or review.

## Current authorization gate

The sole current gate is one fresh independent WO22 pre-issuance architecture
review of this exact successor package. The reviewer must assess authority,
session sizing, typed capability ownership, capture-adapter containment,
GO/NO-GO honesty, cross-platform satisfiability, exact path/budget envelopes,
adversarial coverage, lifecycle, and all planning evidence without editing.

Only an unqualified `ACCEPT` may recommend, but not execute, one separately
authorized planning commit with exact subject:

```text
docs(workorder): verify backend input and lower minimal add
```

Only after that exact real local issuance commit exists may a separately
authorized implementer invoke the production classifier exactly once against
its exact parent and child, before any push. The required result is:

```text
mode=full
reason=no_status_transition
anchor=
run_id=0
run_attempt=0
ubuntu_job_id=0
windows_job_id=0
transitions=
```

A disagreement, nonzero exit, lost stdout/stderr or exit evidence, or missing
binding stops before push. No retry, reclassification, status edit, Unit A, or
later implementation follows implicitly. Publication remains a separately
authorized normal non-force push, and Ubuntu and Windows full CI must both
finish green on the exact published issuance commit.

The planning commit, publication, full CI, publication-status record, fast
status CI, Unit A, Unit B, Fast allowances, compiler/backend changes, stash or
archive operations, closeout, WO23, harness consolidation, and semantic-
coordinate research all remain unauthorized until their exact later gates.

<!-- workorder-current-authorization-gate:end -->
