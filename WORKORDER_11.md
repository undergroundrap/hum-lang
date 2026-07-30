# Hum Work Order 11: Validation Throughput and Compiler-Work Recovery

Date: 2026-07-29
<!-- hum-active-workorder:v1 -->
Status: issued and active. This exact Work Order 11 document received final
independent `ACCEPT` with no P0, P1, or P2 findings and was explicitly
BDFL-accepted on its reviewed bytes. It was committed and durably published
as `8b1788d2c5325f95eb41d8b696d0446ba85fe112`, with exact
`WORKORDER_11.md` blob `331a15e87efc176799846ca55b4a90e6c1eb1abd`.

Required workflow `30501525217`, attempt 1, completed successfully for that
exact commit. Ubuntu job `90742024284` succeeded in 25m30s; Cargo caching and
Rust-toolchain preparation succeeded, full Hum preflight succeeded in 24m50s,
and the exhaustive canonical-seal evidence succeeded in 16.132s with exactly
14,226 pairs: F1 630, F2 4,950, and F3/F4 8,646. Windows job
`90742024251` succeeded in 27m55s; Cargo caching and Rust-toolchain preparation
succeeded, full Hum preflight succeeded in 26m48s, and the
platform-independent Exhaustive duplicate correctly skipped. Both jobs
selected `mode=full` with `reason=no_status_transition` and skipped
`Run status-only evidence`.

Work Order 11 is issued and is now the unique active Work Order. The reviewed
proposal language below is preserved as issuance history. Throughput
implementation, compiler recovery, C1/C1R salvage, C2 work, commit, push,
repair, and every later transition remain separately gated.

Owner: BDFL (Ocean).
Author: post-rejection recovery architect acting only under the bounded Work
Order 11 authoring authority and therefore disqualified from this document's
independent verdict.
Planning baseline: clean `main`, with `HEAD`, local `main`, `origin/main`, and
the live remote `main` all equal to
`16d375888e151d7e849b2202c902ec46f9cf462c`.
Predecessor: Work Order 10 remains unchanged. Its Increment 10B.C1R
implementation ended in a terminal final `REJECT` and was archived without
acceptance. Work Order 10, the active-work-order marker, governance, accepted
decisions, and all earlier Work Orders remain frozen.

## Authority and purpose

The BDFL has made validation throughput the highest-priority enabling work.
The present validation architecture repeatedly loads and substantively
analyzes the same small source corpus in fresh Hum processes. That cost made
the final C1R review last longer than the implementation evidence could
reasonably support, even though every mechanical command was green.

This Work Order authorizes, after all planning gates, one small
validation-throughput project before any new compiler increment. Its outcome
is not a faster shell loop. Its outcome is a private in-process validation
slice that reuses one semantically immutable parsed-and-checked source
artifact across multiple existing observations, proves exact product parity,
and then migrates only enough of the measured dominant repeated corpus for an
Optimized Fast run to take no more than half the wall time of a paired,
same-bytes, same-workload Reference Fast run.

The project is evidence infrastructure. It adds no Hum feature, diagnostic
code, schema, command, dependency, runtime behavior, backend artifact, public
API, or semantic rule. It does not rehabilitate the rejected C1R design. It
does not make C1 or C2 eligible automatically.

## Terminal C1R disposition and archive record

The C1R state machine is exhausted. The final verdict was `REJECT`; no C1R2,
third correction, third review, reviewer override, or further C1R test run is
authorized. All mechanical checks were green, but the evidence architecture
remained incomplete. Green commands did not convert the rejected
implementation into accepted compiler behavior.

Before this Work Order was authored, the exact rejected state was preserved
and removed from active `main` through the BDFL-authorized archive lifecycle:

- rejected base, `HEAD`, local `main`, `origin/main`, and live `main`:
  `16d375888e151d7e849b2202c902ec46f9cf462c`;
- rejected worktree: exactly 17 modified tracked paths, 7,416 insertions and
  1,579 deletions, empty index, and no untracked path;
- rejected scoped tree OID:
  `52e84a7439f7e6e9d785a1e995ecab894d55f694`;
- rejected base-overlaid complete tree OID:
  `77ef6495a7f802cd7db2d128059f9063703245a3`;
- archive branch:
  `archive/workorder-10-c1r-rejected-2026-07-29`;
- archive commit:
  `5624ccca16620097919a5b0c354c3d9d65046d37`;
- archive parent:
  `16d375888e151d7e849b2202c902ec46f9cf462c`;
- archive tree:
  `77ef6495a7f802cd7db2d128059f9063703245a3`;
- archive subject:
  `chore(archive): preserve rejected C1R tree`; and
- archive author and committer used the repository-required GitHub no-reply
  identity, verified directly from the commit object.

The local archive ref, `origin` tracking ref, and live archive ref were
verified at the same archive commit. Its exactly 17 committed blobs, byte
lengths, SHA-256 values, Git blob OIDs, parent, tree, and 7,416/1,579 path
statistics matched the external pre-archive manifest. All 17 blobs were then
read back from the published commit into a fresh external temporary
directory, reverified byte-for-byte against that manifest, and the retrieval
directory was removed. The manifest had 17 rows, 2,304 bytes, and SHA-256
`ca5e18728e7d9b2c02e310ad9090700a7e521e19e57c99a95653abcc7c13c4c4`.

Only after those checks passed was `git switch main` used as the sole clearing
mechanism. The resulting main worktree was clean, its index tree equaled the
HEAD tree, its index was empty, and it had no untracked path. A before/after
remote-ref comparison found exactly one added ref, the authorized archive
branch; no other ref changed. No tag, pull request, release, issue, repository
setting, or other remote object was changed.

The archive is recoverable rejection evidence, not an implementation source.
No work under this order may cherry-pick it, copy from it, infer acceptance
from it, or recreate its 111-by-34 cross-product.

## Recorded throughput evidence

The final C1R evidence and review timings are now planning inputs:

| Evidence | Recorded result |
| --- | ---: |
| Final independent C1R review | 1h43m53s |
| C1R 111-row selector | 767.47s |
| C1R Rust suite | 768.42s |
| Implementer Fast | 44m45.4s |
| Reviewer Fast | 44m12.9s |

Those C1R timings motivate this project but are not a performance denominator,
baseline, or ceiling for it. The rejected C1R worktree added four guarded Fast
selectors to clean main's 90-name inventory, including the 111-row C1R
selector. That selector ran once inside the root suite and again through the
then-current exact-selector helper. Clean main does not contain those four
selectors or the rejected matrix. Therefore the C1R 44m45.4s workload is not
equivalent to either implementation unit's candidate workload, and the derived
1,342.7s value is invalid for acceptance. No comparison in this Work Order may
restore the rejected matrix or treat historical C1R bytes as current workload
bytes.

The prior bounded profiling run completed green and measured:

| Measurement | Recorded result |
| --- | ---: |
| Hum invocations | 1,752 |
| Distinct source paths | 216 |
| Distinct stage/source pairs | 1,003 |
| Cumulative Hum runtime | 790.087s |
| Average process start | 1.646ms |
| Conservative startup/initialization upper bound | 21.8s, or 2.8% |
| Repeated parsing and substantive analysis lower bound | 768.3s, or 97.2% |

The largest cumulative stages were:

| Stage | Calls | Cumulative Hum time |
| --- | ---: | ---: |
| `run` | 252 | 157.072s |
| `ir-readiness` | 29 | 126.582s |
| `resource-check` | 122 | 101.074s |
| `ownership-check` | 158 | 76.774s |
| `profile-check` | 32 | 49.965s |
| `effect-check` | 149 | 49.374s |
| `full-type-check` | 165 | 38.876s |
| `check` | 112 | 36.237s |

The startup bound deliberately over-attributes a complete minimal Hum
invocation to startup and initialization. Even under that conservative bound,
shell and process creation cannot explain the observed cost. Process start
averaged only 1.646ms. Shell micro-tuning, asynchronous spawn tricks, process
pools that still repeat analysis, and output-capture rewrites are therefore
refuted as the primary optimization.

The credible direction is an in-process corpus runner, reusable
parsed-and-checked and derived static artifacts, or a measured equivalent that
eliminates repeated front-end work without weakening product-CLI or physical
corruption evidence. This Work Order chooses the smallest code-derived
in-process boundary described below. If that boundary does not accelerate the
real Fast corpus, the project stops for BDFL re-scope; it does not widen into
an unreviewed compiler refactor.

## Accepted-decision and governance locks

All accepted decisions remain authoritative.

- Decision 0001 keeps evidence as an architectural product. Faster evidence
  must remain independently checkable and fail closed.
- Decision 0002 keeps Rust as the bootstrap implementation. This order adds
  no build system, language runtime, package, or dependency.
- Decision 0004 keeps tests first-class evidence. Migration may change where
  an assertion executes, but not the assertion, fixture, misuse, format,
  platform, selector, or corruption relationship it proves.
- Decision 0005 keeps verifiers as evidence producers. Cached observations
  cannot manufacture verifier acceptance or bypass a producer.
- Decision 0007 requires migration discipline and compatibility. Existing
  human and JSON bytes, diagnostic order, channels, and exit semantics remain
  exact.
- Decision 0011 keeps the checked resolver before execution. A reusable
  session cannot bypass resolver, type, effect, ownership, resource, profile,
  or readiness blockers.
- Decisions 0014 through 0018 keep accepted ownership, runtime-contract,
  typed-failure, app-authority, and effect-polymorphism semantics. Static
  cacheability is never inferred for runtime state, adapters, authority
  grants, corruption, or mutation.
- Decision 0019 and the existing license remain unchanged.

`docs/GOVERNANCE.md` remains controlling. Authors do not review their own
planning or implementation. Reviewers do not edit. Commits, publication, and
remote actions require their own BDFL authority. A green check is evidence,
not self-acceptance.

## Complete code-derived integration map

This map was derived from the clean planning baseline before the writable
envelope was frozen. It covers the likely crate, loader, harness, stage-entry,
runtime, tooling, and CI seams.

### Crate and loader boundary

`Cargo.toml` defines one binary, `hum`, at `src/main.rs`. There is no
`src/lib.rs` or library target. The only declared dependency is the existing
local `windows-drive-locality` package. A new library target or dependency is
not required for this project and is outside the envelope.

`src/main.rs` privately declares the compiler modules. Its `run()` function
parses CLI options, handles source-free commands, and then calls
`load_program(&options.inputs)` once for each product process. `LoadedProgram`
owns:

- the parsed `Program`;
- ordered public diagnostics;
- authoritative diagnostic occurrences;
- the reanalyzable projection ledger; and
- per-file and total load timings.

`load_program` reads every ordered input path with `fs::read_to_string`, calls
`parser::parse_source_at_index`, calls `check::check_parse_output`, composes
app-entry diagnostics and occurrence authority, and pushes each parsed file
into one `Program`. The parser and checker already produce all information a
private reusable validation session needs. They do not need a semantic change
or a new public interface.

After loading, `src/main.rs` composes path-boundary, callable, capability, and
stage-type blockers, then dispatches the requested observation. The same
loaded value serves only one command today because stdout, stderr, and
`ExitCode` decisions are intertwined with `print!`, `println!`, and
`eprintln!`. That product-observation seam, not a public library, is the
required extraction point.

### Existing immutable stage entries

The existing command surfaces already accept borrowed immutable compiler
inputs. The relevant entry files and functions are:

| File | Existing observation entries |
| --- | --- |
| `src/diagnostics.rs` | `check_json` and diagnostic catalog renderers |
| `src/json.rs` | `program_to_json` |
| `src/evidence.rs` | human and JSON evidence renderers |
| `src/math_obligations.rs` | human, JSON, and obligation-file derivation |
| `src/resource_report.rs` | summary, human, and JSON resource reports |
| `src/resolve.rs` | summary/view helpers and human/JSON resolver reports |
| `src/type_env.rs` | `type_env_report` and human/JSON reports |
| `src/type_check.rs` | summaries, checked-return views, and human/JSON reports |
| `src/core_preview.rs` | readiness summary and human/JSON preview |
| `src/core_lower.rs` | readiness summary and human/JSON lowering |
| `src/core_verify.rs` | readiness/error views and human/JSON verification |
| `src/full_type_check.rs` | readiness/error views and human/JSON reports |
| `src/effect_check.rs` | readiness/error views and human/JSON reports |
| `src/ownership_check.rs` | readiness/error views and human/JSON reports |
| `src/resource_check.rs` | readiness/error views and human/JSON reports |
| `src/profile_check.rs` | readiness/error/transport views and human/JSON reports |
| `src/ir_readiness.rs` | human/JSON readiness join |
| `src/graph.rs` | graph facts and occurrence-projection validation |
| `src/run.rs` | product runner plus existing `pub(crate)` adapter seams |

These functions take `&Program`, `&[Diagnostic]`, or other borrowed immutable
views. A private child module of the binary crate can call the same entries
without exposing a library API or changing a stage's semantics.

The audit also found why one loaded `Program` alone is not a complete
optimization. Most human, JSON, summary, error, and diagnostic-occurrence
entries call a private `build_report` anew. Their dependency graph is:

```text
resolve
  -> type-env
    -> type-check

core-preview
  -> core-lower
    -> core-verify

type-check + core-verify
  -> full-type-check
    -> effect-check
      -> ownership-check
        -> resource-check
          -> profile-check

all summaries + profile diagnostic transport
  -> ir-readiness
```

Several nodes also rebuild callable, capability, predicate, typed-failure,
field-place, or resource-report analyses. `ir-readiness` requests summaries
across nearly the whole graph and then requests profile transport and
prior-blocker validation again. This is substantive repeated work.

The first project does not respond by making every private report public or by
editing this whole graph. It reuses the authoritative parsed-and-checked
artifact and exact immutable static observation results at the binary's
existing command boundary. If measured results later show that report-level
stage refactoring is required, that is a new dependency graph and requires a
new reviewed Work Order.

### Runtime boundary

`src/run.rs` already provides `pub(crate)` execution functions with output,
replay, file-locality, and file-read adapter seams. The product CLI composes
them through `src/main.rs::execute_run_command`. Parsed source can be reused,
but each runtime observation must receive fresh execution state, fresh
adapters, fresh grant policy, and fresh output buffers. Runtime results,
external file contents, replay state, and authority decisions are never
cacheable under this order.

The private in-process runner may reuse only the immutable load and eligible
static analysis artifacts before execution. It must still exercise the real
runtime path. A bounded real-product sentinel set remains responsible for
process-channel and native-argument behavior.

### Fast tooling boundary

`tools/check_all.ps1` currently:

1. runs status-boundary and exact-selector helper self-tests;
2. runs `cargo fmt --check`;
3. runs one full default `cargo test`;
4. invokes guarded exact Rust selectors that are already members of that full
   suite;
5. runs the separate `windows-drive-locality` package and effect-bakeoff
   manifest tests;
6. runs Clippy and builds `target/debug/hum`; and
7. calls that Hum executable through `Read-NativeOutput`,
   `Read-NativeOutputWithExit`, or `Read-NativeChannelsWithExit` for hundreds
   of corpus observations.

The current script credits exactly 90 unique guarded Fast selectors.
`tools/test_exact_rust_selector.ps1::Invoke-ExactRustTest` first runs
`cargo test <selector> -- --exact --list` and then runs
`cargo test <selector> -- --exact`. Because the full default suite ran first,
each selected test executes once there and once again through the helper; the
90 per-selector listing processes are additional overhead. The rejected C1R
worktree instead had 94 guarded selectors. Its added 111-row selector executed
inside the root suite and again through the helper. That historical
94-selector workload is intentionally absent from clean main and is not a
Reference mode for this project.

Fast needs one exact full test listing before one full default test execution.
The listing must prove each guarded selector resolves exactly once and the
inventory contains clean main's 90 distinct names plus only the reviewed
candidate additions declared in the complete conclusion ledger. The one
captured full test transcript must then prove each declared name executed
successfully once. The separate package and manifest tests are not duplicates
of the root default suite and remain.

The existing Exhaustive branch sets
`HUM_CANONICAL_SEAL_EVIDENCE_TIER=exhaustive` and invokes exactly
`parser::tests::exhaustive_canonical_seal_pair_matrix_is_complete_and_nonzero`.
Its exact 14,226-pair transcript remains one producer artifact. The
Exhaustive dispatch and its existing `Invoke-ExactRustTest` execution path
must retain exactly equivalent behavior during Fast-selector simplification.
If either path, selector, environment, pair logic, or output logic changes,
one fresh Exhaustive producer run is required; otherwise the independent
reviewer verifies the existing exact transcript without rerunning it.

### CI boundary

`.github/workflows/ci.yml` runs Fast on both `windows-latest` and
`ubuntu-latest`. It runs Exhaustive once, on Ubuntu only, in the full lane.
Those two Fast platforms remain the final cross-platform evidence. The
workflow needs no change for the mapped architecture. It is audited but not
writable. If implementation discovers that a workflow edit is necessary, the
unit stops for BDFL re-scope rather than expanding its envelope.

### Audited paths that are not writable

The following likely boundaries were inspected and deliberately excluded
from both implementation units:

- `Cargo.toml`;
- `.github/workflows/ci.yml`;
- `src/ast.rs`, `src/parser.rs`, `src/check.rs`, `src/diagnostic.rs`,
  `src/diagnostics.rs`, `src/json.rs`, `src/graph.rs`, and `src/run.rs`;
- `src/resolve.rs`, `src/type_env.rs`, `src/type_check.rs`,
  `src/core_preview.rs`, `src/core_lower.rs`, `src/core_verify.rs`,
  `src/full_type_check.rs`, `src/effect_check.rs`,
  `src/ownership_check.rs`, `src/resource_report.rs`,
  `src/resource_check.rs`, `src/profile_check.rs`, and
  `src/ir_readiness.rs`;
- every other `src/*.rs` semantic producer or consumer;
- every fixture, example, schema, decision, governance file, README, and
  Work Order; and
- every document-check and release-check script.

The exclusion is load-bearing. Existing borrowed stage entries are sufficient
for the bounded session. A need to edit any excluded path proves that the
frozen architecture is incomplete and triggers the stop condition.

## Private reusable-session architecture

The implementation must preserve one product path, not create a second
evidence framework.

### Shared observation result

`src/main.rs` must extract a private, borrowed post-load observation boundary.
Its result contains exact stdout bytes, exact stderr bytes, and the exact exit
code. The normal CLI writes that result to the native channels. The private
corpus runner receives the same result in memory. There may not be parallel
rendering or exit-code logic in PowerShell and Rust.

The extraction preserves:

- command parsing and native argument rules;
- ordered input-path identity and source span text;
- diagnostic occurrence authority and diagnostic ordering;
- all callable, path, capability, stage-type, and readiness blockers;
- human and JSON bytes, including final newlines;
- stdout/stderr separation;
- exit 0, semantic failure exit 1, and CLI usage exit 2;
- `--timings` behavior outside parity snapshots; and
- all runtime and adapter behavior.

No public Hum command, option, schema field, crate API, or feature flag exposes
the session.

### Immutable source identity

A `ValidationSession` is private to one test case or one explicitly grouped
source identity. Its load key includes:

- the exact ordered input paths as presented to Hum;
- the exact source bytes used by parsing;
- semantic file order;
- command-independent parse/check configuration; and
- any environment or target fact that can change parse/check semantics.

The loader may be internally separated into read and parse/check steps inside
`src/main.rs` so the same captured bytes establish identity and feed the
parser. It may not reread a file and assume equivalence. The session retains
the exact bytes or an independently collision-safe equality witness using only
the standard library; no dependency is added.

The `Program`, diagnostics, occurrence set, and reanalysis ledger remain
semantically immutable after construction. Static observation cache keys
include the exact command, format, options, and source-session identity. A
cache hit returns the previously produced exact observation bytes and exit
code. It cannot skip a requested invariant validation.

### Forbidden reuse

There is no process-global or cross-test cache. A new session is mandatory
when any source byte, input order, environment-sensitive semantic input,
mutation, corruption, or test hook differs.

The following are always fresh and never cached:

- runtime execution state and `run` results;
- output, replay, file, file-locality, clock-like, or future external
  adapters;
- operator grant and deny decisions;
- filesystem contents observed after load;
- mutation and physical-corruption cases;
- diagnostic occurrence mutation or resealing cases;
- tests whose purpose is repeatability across fresh product processes; and
- any observation not proven semantically immutable.

Every corruption row runs in a fresh isolated fixture/session. A preceding row
cannot leave mutated compiler data, environment variables, adapters, output,
temporary files, or cache entries for the next row. Panic or failure cleanup
must restore process environment and remove temporary artifacts.

Equivalence, if ever claimed for a formerly forbidden reuse, requires a
separate reviewed corruption and fresh-process comparison. Convenience,
matching path text, or matching stage name is not proof.

### One framework and portability

The corpus runner is a test-only child module of the existing binary crate.
It uses the Rust test harness already executed by Fast and the same product
functions as `hum`. It is not a new executable, package, build system,
manifest, snapshot framework, daemon, service, or command.

Path construction uses `Path`/`PathBuf`; product-binary discovery is supplied
by the existing Fast build and works with `.exe` on Windows and no suffix on
Ubuntu. Tests cannot assume slash direction, drive letters, case folding,
locale, user profile, network access, or a repository outside
`CARGO_MANIFEST_DIR`.

## Frozen implementation envelope

There are exactly two possible dependency-coherent implementation units. They
are not sessions with sub-units. Each receives a separate explicit BDFL start
signal and an independent verdict. No unit starts merely because this proposal
is accepted.

### Unit 1 writable paths

Unit 1 may modify exactly:

| Path | Bounded purpose |
| --- | --- |
| `src/main.rs` | Extract the one shared private loaded-program observation boundary; declare the private session and test-only corpus modules; preserve normal CLI rendering. |
| `src/validation_session.rs` | New private session, immutable identity, eligible static observation cache, isolation rules, Reference/Optimized policy seam, and load-bearing counters/timers. |
| `src/validation_corpus.rs` | New test-only representative corpus, exact in-process assertions, product-CLI parity sentinels, mutation-isolation controls, complete conclusion ledger, and hierarchical metrics transcript. |
| `tools/check_all.ps1` | Build the product before the one root test suite; implement private Reference/Optimized evidence modes, candidate-manifest pre/post binding, canonical transcript capture, phase reconciliation, and one Fast test execution; consume runner metrics; remove only mapped duplicate subprocess observations; and retain all non-equivalent evidence. |
| `tools/test_exact_rust_selector.ps1` | Add Fast-only inventory/list/transcript verification and exact-selector conclusion-ledger support while leaving the Exhaustive producer behavior unchanged. |

No other path may be modified. In particular, Unit 1 may not edit a stage
report builder, parser, checker, AST, runtime, fixture, manifest, workflow,
schema, documentation, or accepted decision.

### Unit 2 writable paths

Only after independent acceptance and BDFL closure of Unit 1, Unit 2 may
modify exactly:

| Path | Bounded purpose |
| --- | --- |
| `src/validation_session.rs` | Extend only already accepted private metrics, Reference/Optimized policy behavior, or exact immutable observation-key coverage needed by migrated cases. |
| `src/validation_corpus.rs` | Migrate measured dominant repeated corpus observations into the accepted runner and retain the complete conclusion ledger, hierarchical phase ledger, and isolated runtime/corruption handling. |
| `tools/check_all.ps1` | Replace only mapped equivalent Hum subprocess groups, retain the bounded product sentinels, and enforce manifest-bound Reference/Optimized transcripts plus the final throughput/coverage ledger. |

Unit 2 may not change `src/main.rs`; its general observation boundary must have
been completed and accepted in Unit 1. A need to change it, a semantic stage,
the runtime, a fixture, a manifest, or CI is evidence that Unit 2 is not
bounded and must stop.

## Unit 1: real in-process vertical slice

Unit 1 produces actual code and measurable acceleration. A profiling-only
report, empty abstraction, unused cache, or test that still spawns Hum once
per stage/source pair does not satisfy the unit.

### Fixed representative corpus

The first slice loads these three existing source identities once each:

1. `examples/reference_surface.hum`, a broad successful static surface;
2. `fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum`, an ordered
   failing diagnostic-transport chain; and
3. `examples/probes/bounded_stdout.hum`, a real runtime and authority surface.

The successful source runs, through the shared in-process observation
boundary, at least:

- `check` human and JSON;
- `resolve`, `type-env`, and `type-check` JSON;
- `core-preview`, `core-lower`, and `core-verify` JSON;
- `full-type-check`, `effect-check`, and `ownership-check` JSON;
- `resource-check` and `profile-check` JSON; and
- `ir-readiness` JSON.

The failing source runs at least:

- `check` human and JSON;
- `resolve`, `full-type-check`, `effect-check`, `ownership-check`,
  `resource-check`, `profile-check`, and `ir-readiness` JSON; and
- the existing graph JSON observation.

The bounded-output source runs at least static `check` and `effect-check`
JSON observations from its one immutable load. Its allowed and default-denied
`run` observations reuse that immutable load but use fresh execution state,
fresh adapters, fresh grant policies, and fresh output buffers. The runtime
results themselves are not cached.

Every migrated assertion must be at least as exact as its current
`tools/check_all.ps1` assertion. JSON parsing alone is insufficient when the
old evidence pinned raw bytes, order, channel, count, or absence. Existing
assertions not represented in the fixed slice remain in PowerShell unchanged.

### Exact product parity sentinels

The one root Rust suite runs after Fast has built the exact default product
binary. The test-only corpus compares in-memory `CommandObservation` values
directly with six real product processes:

1. successful `check` human on `examples/reference_surface.hum`;
2. successful `ir-readiness --format=json` on that source;
3. failing `check` human on the Session AP blocker-chain source;
4. failing `ir-readiness --format=json` on that source;
5. allowed `run` on `examples/probes/bounded_stdout.hum`; and
6. default-denied `run` on that source.

For every sentinel, comparison is exact across stdout bytes, stderr bytes, and
exit code. The existing Windows forward-slash/backslash source-identity
sentinel also remains when running on Windows. These are seven or fewer
product invocations per platform for this slice, not one process per
observation.

The sentinels cover successful and failing static paths, human and JSON
formats, semantic exit 1, runtime output, runtime causal failure, and channel
separation. Usage exit 2 and every distinct native-argument behavior not
represented here remain in their existing real-CLI tests.

### Traceability and no evidence loss

Before deleting or replacing any old invocation, Unit 1 adds a checked
traceability ledger with:

- existing Fast label and source location;
- exact old command, working directory, environment, package, features, and
  harness flags;
- fixture and source-input order;
- old assertions and their required absences;
- new in-process test name and assertion location;
- static-cache eligibility or explicit non-cacheability;
- real-CLI sentinel coverage, if any; and
- reason the old subprocess is an exact duplicate.

The ledger is enforced by the existing Fast script or Rust test; it is not a
free-standing report. An unmapped old assertion remains. A count decrease
without one-to-one relationship evidence fails closed.

Unit 1 must prove:

- each of the three source identities was read, parsed, and checked once per
  session;
- multiple static observations used the same immutable artifact;
- at least one eligible repeated static observation was a real cache hit;
- changing one source byte creates a new identity and cannot hit the old
  cache;
- reversing a multi-file input order, where a multi-file isolation probe is
  used, creates a distinct identity;
- two corruption or source-mutation controls run in fresh sessions and remain
  order-independent;
- a failed or panicking control cannot contaminate the next case;
- runtime observations have zero runtime-result cache hits; and
- all parity sentinels match exactly.

### Unit 1 acceleration gate

Unit 1 is not required to close the final 2x Fast gate, but it must show real
acceleration on its fixed slice. The candidate's Reference and Optimized
policies execute a paired, instrumented slice comparison inside the required
Optimized Fast producer. Both sides use the same candidate bytes, fixed
three-source corpus, ordered observations, assertions, fixtures, conclusion
ledger, toolchain, platform, profile, features, and environment. Reference
forces fresh load, parse, initial check, and substantive eligible static
analysis at every migrated observation where clean main did so. Optimized
enables only the proposed immutable reuse and equivalent-command
deduplication.

- fewer source reads, parses, checks, and Hum process invocations than the
  paired Reference slice;
- fewer substantive static-analysis computations when an exact observation
  repeats;
- lower Optimized slice wall time than the paired Reference slice;
- no parity or coverage failure.

The implementer and independent reviewer each receive the complete paired
slice counts and timings in their own Optimized Fast transcript. This proves
only fixed-slice acceleration; it cannot be described as a 2x full-Fast result
or as project completion. If the slice has no material acceleration, or if
most cost still demands excluded stage-module edits, Unit 1 returns to the
BDFL and Unit 2 does not start.

## Unit 2: bounded dominant-corpus migration

Unit 2 begins only if Unit 1 has an independent `ACCEPT`, has been explicitly
BDFL-accepted, committed and published under separate authority, has green
required CI, and the BDFL then issues a separate Unit 2 signal.

The migration order follows measured cumulative cost, not file order or
convenience:

1. repeated immutable loading shared by the dominant `run` cases, while every
   runtime execution remains fresh;
2. `ir-readiness`;
3. `resource-check` and `ownership-check`;
4. `profile-check`, `effect-check`, and `full-type-check`; and
5. `check`.

Only groups needed to reach the final gate are migrated. Unit 2 stops adding
cases once the gate and coverage ledger pass. It does not rewrite all of
`check_all.ps1`, introduce a declarative framework, or move cheap unique CLI
tests merely for stylistic uniformity.

For each migrated group:

- one exact immutable source identity is loaded once;
- all eligible static observations use the accepted session boundary;
- repeated identical static observations use the accepted exact cache;
- runtime, native path, output channel, grant, file, replay, and corruption
  cases remain fresh;
- at least one real product sentinel remains for each distinct command
  behavior, format/channel contract, exit class, platform distinction, and
  adapter configuration represented by the group;
- every prior assertion is traced to an equal or stronger assertion; and
- Windows and Ubuntu execute the same portable Rust corpus, plus their
  genuinely platform-specific product sentinels.

If the final gate cannot be reached within the three Unit 2 paths, the unit
stops. It may not expose private stage reports, add a library, edit stage
semantics, weaken evidence, or request a third implementation unit inside
this Work Order.

## Deterministic candidate manifest and transcript binding

Every implementation and review producer is bound to exact candidate bytes,
not merely to a branch name, dirty-path summary, or implementer transcript.
The implementer freezes a deterministic content-addressed implementation
manifest before the terminal producer. The independent reviewer reconstructs
that manifest from the reviewed worktree and does not copy identifiers from
the implementer.

### Fixed manifest scope and status

The manifest scope order is fixed by this Work Order.

Unit 1 order is:

1. `src/main.rs`;
2. `src/validation_session.rs`;
3. `src/validation_corpus.rs`;
4. `tools/check_all.ps1`; and
5. `tools/test_exact_rust_selector.ps1`.

Unit 2 order is:

1. `src/validation_session.rs`;
2. `src/validation_corpus.rs`; and
3. `tools/check_all.ps1`.

Every scope path appears even when tracked-clean. The only permitted
dispositions are `tracked-clean`, `tracked-modified`, and `untracked-added`.
Deletion, rename, copy, symlink, submodule, unresolved merge, non-regular file,
mode drift, staged entry, or status outside the current unit's fixed envelope
fails closed. A new regular file has Git mode `100644`; every tracked path
retains its base Git mode. `HEAD` must equal the unit's accepted base, the real
index must equal `HEAD`, and porcelain status must correspond exactly to the
manifest dispositions before and after each producer.

### Canonical payload

The manifest is an external evidence artifact, never a checked-in path. Its
payload is UTF-8 without BOM, uses LF line endings, literal tabs as field
separators, lowercase hexadecimal, forward-slash repository-relative paths,
invariant decimal integers, and this exact record order:

```text
hum-implementation-manifest-v1
base<TAB><40-hex accepted-base commit>
unit<TAB><1-or-2>
scope_count<TAB><decimal>
path<TAB><0001-based ordinal><TAB><path><TAB><Git mode><TAB><disposition><TAB><raw byte length><TAB><raw SHA-256><TAB><raw Git blob OID><TAB><added lines><TAB><deleted lines>
...one path row for every fixed scope path...
diff_total<TAB><changed path count><TAB><added lines><TAB><deleted lines>
scoped_tree<TAB><Git tree OID>
complete_tree<TAB><Git tree OID>
```

There is exactly one terminal LF. The header is the only one-field record.
There is no quoting or escaping because the frozen ASCII paths contain no tab,
CR, or LF. Binary numstat, an absent path, a duplicate row, a reordered row,
or an unexpected field fails closed. Raw byte length and SHA-256 are computed
from the worktree file without newline conversion. The raw Git blob OID is
computed with non-writing `git hash-object --no-filters` over those same
bytes; `-w` is forbidden during identity computation. The OID must agree with
the recorded raw byte length and SHA-256 and must later equal the OID returned
when the same bytes are written only into the fresh external proof object
directory. Per-path and total numstat are computed with renames, text
conversion, and external diff disabled against the accepted base and the raw
candidate blobs. A tracked-clean disposition is still recorded from porcelain
status independently of its raw byte identity.

The manifest SHA-256 is computed over the complete payload bytes above; the
digest is reported alongside, not included recursively in, the payload.
Timestamp, hostname, username, locale-dependent text, temporary path, shell
enumeration order, branch name, and transcript data are forbidden from the
payload.

### Independent external tree construction

No proof command may write an object into the real repository object database.
Before construction, with inherited `GIT_OBJECT_DIRECTORY`,
`GIT_ALTERNATE_OBJECT_DIRECTORIES`, and `GIT_INDEX_FILE` absent, the actor
resolves and canonicalizes the absolute worktree, Git directory, and real
object directory. The real object directory is obtained from Git with
`git rev-parse --git-path objects`; `.git/objects` is never assumed. A relative
result is resolved against Git's reported context before use.

Every construction receives a newly absent external temporary root containing
one fresh object directory and two fresh indexes. Each proposed external path
is canonicalized from an existing parent before creation and canonicalized
again afterward. Symlinks, junctions, reparse points, unresolved parents, or
case-normalization ambiguity fail closed. The external root, object directory,
indexes, and artifacts must not equal, reside inside, or be an ancestor of the
worktree, repository, Git directory, real index, or real object directory.
Failure to prove every relationship stops the unit before an object-writing
command runs.

For the complete construction environment:

- `GIT_OBJECT_DIRECTORY` is the fresh absolute external object directory;
- `GIT_ALTERNATE_OBJECT_DIRECTORIES` contains only the resolved real object
  directory, which Git uses as a read-only accepted-base lookup;
- `GIT_INDEX_FILE` is one of the fresh absolute external indexes; and
- no proof command may target the real index or use the real object directory
  as its primary writable object directory.

Those exact object variables are set for every `git hash-object -w`,
`git write-tree`, or other proof command capable of writing an object. No
proof command may write through an alternate, update a ref, modify a worktree,
change configuration, or invoke object maintenance. Candidate blobs required
by either index are written with `git hash-object -w --no-filters` only after
the external object environment is active. Each returned OID must equal its
previously computed non-writing OID. Accepted-base commits, trees, and blobs
are read only through the real-object alternate.

For the scoped tree, start one external index empty, install only the fixed
ordered scope rows using each row's exact mode and externally stored raw blob
OID, and run `git write-tree` under the external object environment. For the
complete tree, seed the second external index with
`git read-tree <accepted-base>`, overlay exactly those same scope rows, and run
`git write-tree` under that environment. The generated scoped and complete
tree objects therefore exist only in the external object directory. Temporary
index entries must read back with the same mode and blob OID as every payload
row.

Immediately before and after each construction, the actor takes a read-only,
recursively complete inventory of the resolved real object directory. The
canonical inventory records every relative directory and every regular file
in ordinal path order; each file records raw byte length and SHA-256. This
includes loose objects, packfiles, pack indexes, reverse indexes,
multi-pack-indexes, commit-graphs, and every other file under the real object
directory. The before and after inventories must be byte-identical. Any added,
removed, renamed, or changed real-object-store entry invalidates the proof and
stops the unit without attempting to normalize or repair the real store.

The implementer and reviewer each independently construct the payload, its
SHA-256, every blob identity, the scoped tree, and the complete base-overlaid
tree once in PowerShell and once in Git Bash. Each actor/shell construction
uses a different fresh external object directory and fresh indexes. The two
shell reconstructions for that actor must be byte-identical, and all four
actor/shell results must match. Temporary path names are execution mechanics
and never enter the manifest payload. After verification, the external indexes
and object directory are removed. Manifest, transcript, and real-store
inventory evidence stays outside the repository. The real index is proved
unchanged and empty after every construction. Failure to externalize, verify,
or remove a proof-object directory or index fails closed.

### Producer transcript envelope

Immediately before every Reference or Optimized Fast producer, the actor
reconstructs and cross-shell-verifies the manifest. The producer records a
canonical external transcript envelope containing:

- pre- and post-producer manifest SHA-256, scoped tree, and complete tree;
- exact resolved executable plus ordered arguments and working directory;
- the fixed, ordinally sorted conclusion-affecting environment, evidence tier,
  and Reference or Optimized mode;
- Rust/Cargo/PowerShell versions, OS, architecture, CPU, and platform;
- package, manifest, features, default-feature state, profile, target
  directory, Cargo/cache policy, build readiness, and power conditions;
- before/after real-object-store inventory SHA-256 values, their equality
  result, and external proof-object cleanup result;
- UTC start and end instants plus monotonic elapsed time;
- process exit code; and
- raw stdout and stderr byte lengths and SHA-256 values.

Canonical transcript metadata uses the same UTF-8-without-BOM, LF, literal-tab,
invariant-integer rules as the manifest. Raw stdout and stderr are retained as
separate external byte artifacts and bound by their reported lengths and
hashes; neither channel is decoded and re-encoded to establish identity. The
SHA-256 of the complete canonical transcript metadata is reported separately.

Immediately after the producer, the actor recomputes the manifest payload,
digest, scoped tree, and complete tree independently in both shells. Any
pre/post byte, status, base, index, mode, tree, configuration, stdout/stderr,
or conclusion-ledger drift invalidates the producer. Proof artifacts remain
outside the repository, and temporary indexes are removed. The independent
reviewer must reconstruct the implementer's reported manifest and transcript
bindings from the candidate and raw artifacts; merely copying their hashes is
not verification.

## Evidence-execution simplification

Throughput gains cannot come from omitting a conclusion. They may come from
executing an exactly equivalent expensive conclusion once.

### Optimized Fast selector execution

The revised Optimized Fast flow is:

1. collect the complete guarded Fast selector inventory before execution;
2. run one exact default `cargo test -- --list`-equivalent listing operation;
3. prove all expected guarded selectors are present exactly once, prove the
   declared inventory is clean main's 90-name set plus only reviewed candidate
   additions, and reject missing, renamed, duplicated, ignored, or
   configuration-incompatible entries;
4. run one captured full default root `cargo test` in the same package,
   manifest, target, feature, profile, environment, and harness configuration;
5. prove each guarded selector appears once as a successful executed test in
   that transcript; and
6. credit it once without a standalone selected execution.

The C1R 111-row selector is not in the clean-main inventory and must not be
restored or rerun. Future compiler work must rederive review-sized corruption
authority; it may not recreate the rejected cross-product.

Separate `windows-drive-locality` package tests and effect-bakeoff manifest
tests remain because their package/manifest configurations differ. Focused
tests used during implementation do not earn terminal Fast credit.

### Exact-equivalence rule

Two commands are duplicates only when all conclusion-affecting fields match:

- executable and toolchain;
- repository commit and dirty implementation bytes;
- working directory;
- package and manifest;
- target and target directory where relevant;
- feature set and default-feature state;
- build/test profile;
- environment variables, especially evidence tier;
- test filter, ignored status, and harness flags;
- source/fixture bytes and ordered inputs;
- platform; and
- external adapter or authority configuration.

Every skipped duplicate records that tuple and points to the one retained
execution. It also records the stable conclusion identifier, producer,
positive and negative assertions, expected channel/exit relationship, and the
retained transcript location. The complete ordered conclusion ledger must be
identical before and after deduplication. A difference in any field makes the
command non-equivalent until independently proven otherwise.

An implementer or reviewer does not run a separate full `cargo test`
immediately before Fast when Fast will run the identical root configuration
and exact tests. They may run focused tests while developing or a probe for a
missing relationship. The terminal Fast command is the one full-suite
producer.

### Mandatory independent Fast producers

For each unit, the implementer freezes the candidate manifest and runs one
terminal Optimized Fast producer on those bytes. A fresh independent reviewer
then reconstructs the same manifest, inspects the complete implementation
diff, complete conclusion ledger, and implementer transcript, and independently
runs Optimized Fast on those unchanged bytes. Transcript inspection never
substitutes for the reviewer's required Fast run. This requirement follows the
repository's implementation-review policy and is distinct from the
pre-issuance document review, whose checks are document-only.

Both actors may target additional focused probes at missing relationships,
configuration differences, cache isolation, parity, or platform behavior.
Neither actor runs a redundant standalone full root `cargo test` immediately
before Fast. Inside Fast, an exact duplicate selector may be credited from the
single root-suite producer only under the exact-equivalence and complete-ledger
rules above.

If the reviewer changes any byte, authorship is contaminated and review stops.
If a probe differs in configuration, the reviewer records why it is not a
duplicate.

### Exhaustive evidence

Exhaustive remains one exact-byte producer transcript and one independent
verification. The reviewer checks selector identity, environment tier, seed,
F1 630 pairs, F2 4,950 pairs, F3/F4 8,646 pairs, total 14,226 pairs, exit
status, and transcript integrity.

No fresh Exhaustive producer is required when the parser, matrix, selector,
environment, output contract, `check_all.ps1` Exhaustive branch, and
`Invoke-ExactRustTest` Exhaustive path are byte-identical. If any of those
changes, one fresh producer run is mandatory. It still runs once, on Ubuntu,
and is not duplicated on Windows.

### Final cross-platform evidence

After each accepted implementation commit is separately authorized and
published, required CI runs Optimized Fast on both Ubuntu and Windows. Those
jobs are the final cross-platform conclusion. Local platform-specific probes
cannot replace either job.

No test, fixture, guarded selector, stage, human/JSON format, platform,
positive case, misuse case, diagnostic-order assertion, absence assertion,
runtime authority case, or physical-corruption relationship may disappear to
meet a time target.

## Reference and Optimized evidence modes

`Reference` and `Optimized` are private Fast evidence modes implemented only
inside the frozen validation paths. They are not Hum commands, public flags,
features, schemas, or developer convenience modes. Both execute the exact same
candidate manifest, ordered logical workload, selector inventory, packages,
manifests, source and fixture bytes, input order, tests, assertions, parity
sentinels, corruption controls, and complete conclusion ledger.

Reference disables the proposed session reuse, observation-result reuse, and
equivalent-command credit. It preserves the pre-optimization behavior on the
candidate bytes: each migrated observation performs the fresh load, parse,
initial check, substantive static analysis, product process, and guarded
exact-selector execution that it replaced. Optimized enables only the reviewed
reuse and deduplication. A conclusion must be produced in both modes even when
Optimized reaches it with fewer computations or processes. Historical C1R
bytes, its 94-selector inventory, and its rejected matrix are never a mode.

Within one actor's comparison, these conclusion-affecting facts are identical:

- candidate manifest payload, digest, scoped tree, and complete tree;
- corpus representation, selector inventory, conclusion ledger, fixtures, and
  order;
- host, OS, architecture, CPU, toolchain, PowerShell, profile, packages,
  manifests, features, default-feature state, environment, and evidence tier;
- dependency availability, target/cache policy, build readiness, and power
  conditions; and
- capture, phase-ledger, parity, and pass/fail rules.

Only the declared reuse/deduplication policy, the required actor-specific run
order, and fresh opaque temporary directory names may differ. Each full mode
starts with a newly absent, equivalent isolated Cargo target directory and
equivalent empty temporary root. Setup and cleanup occur inside the timed
interval. Registry and source caches use one recorded, already-ready policy;
network acquisition or a toolchain change invalidates the pair.

## Required metrics and hierarchical phase ledger

Every Fast producer emits one deterministic machine-readable metrics record
and a readable summary. At minimum it reports:

- total product Hum process invocation count;
- distinct source identity, source read, parse, and initial check counts;
- static observation request, cacheable request, non-cacheable request,
  substantive static-analysis computation, cache lookup, hit, miss, entry
  construction, and reuse counts;
- runtime request and execution counts, with zero runtime-result cache hits;
- guarded selector inventory, listing, root-execution, and credited-execution
  counts;
- real-CLI sentinel and parity totals for stdout, stderr, exit, human, JSON,
  diagnostic order, and isolated corruption/mutation controls;
- complete conclusion-ledger expected, Reference-produced, Optimized-produced,
  passed, and failed totals; and
- full Fast monotonic wall time plus the hierarchical phase ledger below.

The ledger has stable phase IDs, one parent ID or `root`, invocation counts,
monotonic start/end offsets, inclusive microseconds, exclusive microseconds,
and status. It contains, as applicable, all of:

1. dependency and toolchain readiness;
2. manifest precheck and mode/configuration binding;
3. build and prebuild;
4. root-suite listing;
5. root-suite execution;
6. special package and alternate-manifest tests;
7. guarded-selector verification and any Reference-only standalone execution;
8. text hygiene, public readiness, and release readiness;
9. remaining product-process corpus;
10. reusable-corpus construction;
11. source reads;
12. parse;
13. initial static check;
14. substantive static analysis;
15. cache lookup, hit, miss, entry construction, and reuse;
16. fresh runtime execution;
17. product parity and corruption isolation;
18. transcript finalization and post-producer manifest reconstruction;
19. cleanup; and
20. explicit measurement and orchestration overhead.

Children nested inside root-suite or corpus execution are not added a second
time at the top level. Overlapping intervals use interval unions; parallel
child durations are never arithmetically summed as if serial. At every timed
parent:

```text
parent inclusive = union(child inclusive) + parent exclusive
parent exclusive = attributed direct work + unattributed
```

The absolute reconciliation error must be no greater than the larger of 10ms
or 0.01% of that parent's inclusive time. Unattributed time must be at most 1%
of the parent and is always reported; it is never silently assigned to
analysis. A failed reconciliation, missing required phase, negative duration,
clock rollback, overlap error, or absent parent fails Fast.

The following count equations are load-bearing and fail closed:

```text
static requests = substantive computations + cache hits
static requests = cacheable requests + non-cacheable requests
cacheable requests = cache lookups = cache hits + cache misses
substantive computations = cache misses + non-cacheable requests
cache misses = cache entries constructed
runtime requests = runtime executions
runtime-result cache hits = 0
declared guarded selectors = unique listed selectors
credited guarded selectors = successful matching root executions
expected conclusions = produced conclusions = passed conclusions
failed conclusions = 0
```

For the successful reuse-eligible corpus, distinct loaded source identities,
source reads, parses, and initial checks also reconcile exactly in Optimized
mode; every documented exception in Reference mode has a stable conclusion or
observation ID. Every session reports its source identity and exactly which
observation keys were computed or reused.

Each paired result attributes Reference-minus-Optimized wall-time deltas to
compile/build, equivalent-command deduplication, source reads, parse/initial
check, substantive-analysis reuse, corpus/cache mechanics, transcript and
manifest overhead, or another specifically named measured phase. A generic
“miscellaneous speedup” is not an attribution. The phase and count ledgers
must make the claimed dominant cause load-bearing.

## Performance acceptance

Unit 1 uses only the fixed-slice paired comparison defined above. Its result
cannot satisfy the full Fast 2x gate.

Unit 2 requires two independent full-mode pairs on frozen Unit 2 bytes:

1. the implementer runs Reference and then Optimized; and
2. the independent reviewer reconstructs the same candidate and runs
   Optimized and then Reference.

Each actor uses one otherwise-idle local Windows host for their own pair,
fresh equivalent isolated target and temporary directories for every run, and
the identical recorded conditions above. The actor does not prewarm only one
mode. All started and completed runs, including failures or invalidated pairs,
are retained and reported; results may not be cherry-picked. If an external
event invalidates a run, the reason is recorded and that actor repeats the
entire two-mode pair under equivalent conditions while retaining the original
transcripts.

Both independent ratios must pass:

```text
implementer Optimized wall / implementer Reference wall <= 0.500000
reviewer Optimized wall / reviewer Reference wall <= 0.500000
```

Neither an average, the faster actor, an absolute historical ceiling, nor one
mode from each actor may substitute. The 44m45.4s C1R run remains motivation
only and produces no denominator.

Performance acceptance additionally requires:

- identical candidate identities and complete conclusion ledgers across all
  four valid pair runs;
- no reduction in coverage or assertion strength;
- all exact product parity, cache-isolation, mutation-isolation, and
  no-runtime-cache invariants green;
- the declared guarded selector inventory listed and credited exactly once in
  Optimized, with every Reference duplicate recorded;
- phase/count reconciliation and causal attribution green;
- no unaccounted skipped command;
- green local document checks included by Fast; and
- green final Ubuntu and Windows Fast CI.

A faster run with weaker evidence fails. A green Optimized run above half its
own paired Reference wall time does not complete Unit 2. Variance, runner
contention, dependency acquisition, or other invalidating conditions are
reported honestly; they are not deleted from the evidence or converted into
an unmeasured claim.

## Implementation and review state machine

This Work Order has the following one-way gates:

```text
proposed Work Order 11
  -> independent pre-issuance ACCEPT
  -> explicit BDFL acceptance of exact document bytes
  -> separately authorized document commit/publication and required CI
  -> separate explicit Unit 1 implementation signal
  -> Unit 1 implementation and manifest freeze
  -> implementer Optimized Fast on frozen Unit 1 bytes
  -> independent Unit 1 manifest reconstruction, Optimized Fast, and verdict
  -> explicit BDFL Unit 1 acceptance/commit/publication and required CI
  -> separate explicit Unit 2 signal, only if still needed
  -> Unit 2 implementation and manifest freeze
  -> implementer Reference-then-Optimized pair on frozen Unit 2 bytes
  -> independent Unit 2 reconstruction, Optimized-then-Reference pair, verdict
  -> explicit BDFL Unit 2 acceptance/commit/publication and required CI
  -> stop for BDFL
```

There is no automatic transition. A non-`ACCEPT` verdict returns to the BDFL
for a new instruction; it does not authorize an automatic correction,
envelope expansion, or extra unit. Commit and push remain separately
authorized actions.

Each implementer begins from a clean synchronized baseline, records exact
identity before editing, modifies only that unit's paths, keeps the index
empty until separately authorized, freezes the canonical implementation
manifest, and preserves unrelated user changes. Each reviewer verifies exact
diff scope, independently reconstructs the same candidate identities, and is
independent of both Work Order authorship and implementation authorship.

The independent implementation verdict is exactly one of:

- `ACCEPT`
- `ACCEPT WITH REQUIRED FIX`
- `REJECT`

It includes P0/P1/P2 findings and makes no edit.

## Explicit non-goals and stop conditions

This Work Order does not authorize:

- a C1 replacement, C1R2, C1R archive salvage, C2, or any later compiler
  increment;
- the rejected 111-by-34 evidence cross-product;
- a parser, checker, resolver, type, Core, effect, ownership, resource,
  profile, IR-readiness, runtime, or backend semantic change;
- a new H-code, schema version, public output field, CLI command, CLI flag,
  feature, dependency, crate, library target, manifest, build system, service,
  daemon, or evidence framework;
- a fixture rewrite to make caching easier;
- a global cache, persistent cache, cross-test cache, or cache keyed only by
  path text;
- caching runtime results, external observations, authority decisions,
  mutations, or corruption;
- weaker assertions, sampled corruption in place of complete accepted
  corruption evidence, skipped platforms, or removed formats;
- a measurement-only implementation unit;
- a third implementation unit or sub-sub-unit; or
- automatic resumption of compiler work after throughput closes.

Stop immediately and report to the BDFL if:

- a required edit lies outside the current exact envelope;
- source identity or cache isolation cannot be proved;
- shared product and in-process rendering cannot remain one code path;
- any existing conclusion lacks an equal or stronger trace;
- a real-CLI sentinel disagrees by one byte, channel, order, or exit code;
- a corruption case can observe another case's state;
- runtime or external state is cached;
- an actor's PowerShell/Git Bash manifest reconstructions differ, the reviewer
  cannot reproduce the candidate identity, or any producer has pre/post drift;
- Optimized Fast-selector credit cannot be shown exactly once under equivalent
  configuration, or a Reference duplicate is unreported;
- the Exhaustive path changes without a fresh exact producer;
- Unit 1 has no measurable acceleration;
- either independent Unit 2 Optimized/Reference wall-time ratio exceeds 0.5
  within the three-path envelope;
- a required phase, count equation, conclusion-ledger row, or timing
  reconciliation is absent or fails;
- Ubuntu or Windows evidence fails; or
- C1/C2 or language semantics enter the diff.

After the throughput project closes, stop for the BDFL. Future compiler work
must be re-derived from clean accepted source into review-sized
authority/producer/consumer units. Nothing in this Work Order ranks, defines,
or starts those units.

## Current planning disposition

The first bounded correction closed the original architectural P1 findings:
the independent reviewer must run Fast, and performance uses same-candidate,
workload-equivalent Reference/Optimized pairs rather than historical C1R.
The next fresh independent review rejected only real-object-store mutation in
the proof mechanics, the incorrect Hum timing flag, and the missing untracked
whitespace check. The BDFL then authorized this narrow second correction of
`WORKORDER_11.md` only.

The author runs only:

```powershell
git diff --check

$workOrderCheck = @(
    & git -c core.autocrlf=false -c core.safecrlf=false diff `
        --no-index --check -- NUL WORKORDER_11.md 2>&1
)
$workOrderCheckExit = $LASTEXITCODE

if ($workOrderCheckExit -ne 1 -or $workOrderCheck.Count -ne 0) {
    throw "WORKORDER_11.md no-index whitespace check failed"
}

.\tools\check_text_hygiene.ps1
.\tools\check_public_readiness.ps1
.\tools\check_release_readiness.ps1
```

Ordinary `git diff --check` remains required for tracked changes, but Git does
not include an untracked planning file in that comparison. The no-index check
therefore compares the raw `WORKORDER_11.md` bytes with Windows `NUL`.
Command-local `core.autocrlf=false` and `core.safecrlf=false` prevent newline
conversion and its non-error warning from obscuring the zero-output contract;
they do not modify repository or global configuration. A clean nonempty file
must return exactly exit 1 because the files differ and must emit zero output.
Any other exit, any output, an absent file, or a command failure fails closed.

No Cargo command, focused Rust selector, Fast, or Exhaustive is a planning
check. The document remains uncommitted and unstaged with an empty index. The
author reports its exact byte length, SHA-256, non-writing
`git hash-object --no-filters` blob OID, line count, and diff inventory.

One fresh independent pre-issuance architect reviewer then verifies:

- the archive facts and clean-main state;
- that the order attacks repeated parsing and substantive analysis rather
  than disproven process-start tuning;
- dependency closure of both exact writable envelopes;
- one shared product/in-process observation path;
- exact CLI bytes, channels, diagnostic order, and exit parity;
- cache identity, mutation isolation, and fresh runtime/adapters;
- no evidence weakening and exact-equivalence justification for every
  skipped duplicate;
- deterministic cross-shell candidate manifests, externally isolated proof
  objects, unchanged real-object-store inventories, scoped and complete trees,
  pre/post producer binding, and canonical transcript hashes;
- the mandatory independent reviewer Optimized Fast for each unit;
- same-candidate Reference/Optimized workload identity, opposite-order Unit 2
  pairs, both independent ratios at or below 0.5, and no historical absolute
  denominator;
- hierarchical phase/count reconciliation, complete conclusion ledgers, and
  causal delta attribution;
- final Ubuntu and Windows coverage;
- the two-unit maximum and all stop conditions; and
- continued prohibition of C1, archived C1R salvage, C2, and language
  semantics.

The reviewer runs document checks only, makes no edit, reports P0/P1/P2, and
returns exactly one verdict:

- `ACCEPT`
- `ACCEPT WITH REQUIRED FIX`
- `REJECT`

These exact candidate bytes embody only the BDFL-authorized narrow second
correction. They imply no third correction, review verdict, acceptance,
issuance, commit, publication, or implementation. This author performs no
self-review or further correction absent a new explicit BDFL instruction.
Even after a future `ACCEPT`, Work Order 11 remains unissued and uncommitted
until the BDFL accepts the exact reviewed bytes and separately authorizes
documentation commit/publication. Throughput implementation still requires
its own later signal.

## Current authorization gate

Work Order 11 is independently accepted, explicitly BDFL-accepted, published
at `8b1788d2c5325f95eb41d8b696d0446ba85fe112`, terminal-green in required
workflow `30501525217`, and active as the unique marked Work Order.

Unit 1 is the next possible implementation unit, but it remains unauthorized
pending a separate explicit BDFL signal. Unit 2 remains unauthorized. C1,
C1R, C2, archive salvage, and compiler work remain unauthorized.

Publication of this activation transition must use full CI because the active
Work Order path changes. The activation commit is substantive planning, not a
status-only transition; the production classifier is expected to select
`mode=full` with `reason=no_status_transition` or another fail-closed full
reason it actually reports.

Only after that exact activation commit is published and terminal-green may a
later exact `WORKORDER_11.md` status-only change become a candidate for
`mode=fast` with `reason=eligible_status_chain`. Eligibility remains
conditional on every existing trust-envelope and status-boundary requirement
and is not promised unconditionally.

No implementation, commit, push, repair, archive mutation, or later transition
is implied.
<!-- workorder-current-authorization-gate:end -->
