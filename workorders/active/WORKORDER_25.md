# Hum Work Order 25: Evidence Infrastructure Acceleration and Portable Developer Workflow

Date: 2026-08-26
<!-- hum-active-workorder:v1 -->
Status: UNITS A-B IMPLEMENTED, INDEPENDENTLY ACCEPTED, PUBLISHED, SYNCHRONIZED, AND TERMINAL-GREEN; UNIT B CLOSED TO FURTHER IMPLEMENTATION; THIS EXACT UNIT B PUBLICATION-STATUS CANDIDATE AWAITS FRESH INDEPENDENT REVIEW. UNITS C-E UNAUTHORIZED.

WO25 Unit A remains implemented, published, status-recorded, synchronized,
terminal-green, complete, and closed. Unit B's satisfiability amendment is
commit `659d5232e2b564d68f45806200b8eae8aa5ef802`. The independently accepted
Unit B implementation is commit
`ef7df6ba6d3727f45c8171f6c15ee8e9eb4c7337`, subject
`ci(evidence): consume authenticated status summaries`, with canonical
manifest `c48e35604411c34040c83e40ef78df779b4d387f8bd74356b10ff035310d192c`
and exact accounting `+1621/-92`, raw and whitespace-insensitive. Its exact
14-path inventory is:

- `.github/workflows/ci.yml`;
- `crates/hum-dev/src/command.rs`;
- `crates/hum-dev/src/main.rs`;
- `crates/hum-dev/src/status.rs`;
- `crates/hum-dev/src/summary.rs`;
- `crates/hum-dev/src/workorder.rs`;
- `crates/hum-dev/tests/cli.rs`;
- `docs/HUM_EVIDENCE_SUMMARY_SCHEMA.md`;
- `fixtures/evidence/job_summary_ubuntu.v1.json`;
- `fixtures/evidence/job_summary_windows.v1.json`;
- `fixtures/evidence/summary_corruption_cases.v1.json`;
- `tools/check_all.ps1`;
- `tools/check_workorder_status_boundary.ps1`; and
- `tools/test_workorder_status_boundary.ps1`.

Unit B implements the accepted shared production status orchestration,
authenticated cross-platform summary production and consumption, uploaded
platform-specific `hum-dev` executable transport, the test-only controlled
transport seam, immutable Work Order projection, separate bootstrap and
summary cleanup ownership, selectors 125-128, I05-I07, the exact ordered
128-selector ledger, and the required fail-closed corruption and fallback
evidence. It establishes status review only through the accepted bounded
transport contract. It does not complete Units C-E, arbitrary trust discovery,
broader workflow migration, PowerShell retirement, generalized evidence reuse,
or later infrastructure work.

Unit B's first publication workflow `33261749582` was terminal-red at Ubuntu
job `99124773973`: a direct exact-selector invocation violated the guarded
selector policy. It earns no terminal publication-success or artifact credit.
Repair commit `e2c8d164caad80bda3e9111827b1f329454f22f3`, subject
`fix(ci): guard unit b mutation selectors`, routed I05-I07 through the guarded
selector owner without weakening the direct-invocation ban.

Workflow `33263458233` was terminal-red at Ubuntu job `99129226731`: the
workflow-route audit still expected the retired one-line preflight route. It
earns no terminal publication-success or artifact credit. Repair commit
`4626e8b2a6e0ce59c0f632d8abd2d67cc6cff105`, subject
`fix(ci): authenticate full preflight route`, made the instrumented route and
its 29 load-bearing corruptions authoritative.

Workflow `33266958071` was terminal-red at Ubuntu job `99138597344`: brittle
source-order substring ownership used a removed helper as its end marker and
produced a negative substring length. It earns no terminal
publication-success or artifact credit. Repair commit
`4252af3310663785e7ce6bf30e3a32b0b0177373`, subject
`fix(ci): bind dispatcher ownership structurally`, replaced source-order
slicing with AST ownership and established exactly 33 dispatcher controls,
including malformed-source rejection.

Terminal publication evidence is `ci` workflow `33271143652`, attempt `1`,
event `push`, exact SHA `4252af3310663785e7ce6bf30e3a32b0b0177373`.
Ubuntu job `99149715047` succeeded in `13m38s`; Windows job `99149715128`
succeeded in `42m13s`. Both selected exactly:

```text
mode=full
reason=no_status_transition
```

Both platforms passed all 151 ordered classifier cases twice
deterministically; Fast-capture evidence; the 33-control dispatcher matrix and
malformed-source rejection; the 29-control workflow-route matrix; selectors
119-128; I01-I07 with restoration; the exact 128-selector ledger; workspace,
receipt, summary, executable-transport, cleanup, hygiene, public-readiness,
alpha-claims, and release-readiness evidence; and exactly one terminal
full-preflight success marker. Ubuntu Exhaustive passed all `14,226` pairs
with seed `0x48554D5F5345414C`; Windows skipped only the duplicate Exhaustive
producer. Both platforms correctly skipped status-only evidence.

The four exact 14-day evidence artifacts bind producer SHA
`4252af3310663785e7ce6bf30e3a32b0b0177373`, run `33271143652`, attempt `1`,
their numeric producer job, and platform:

- Ubuntu summary artifact ID `9720297281`, name
  `hum-evidence-summary-v1-33271143652-1-99149715047-ubuntu`, GitHub SHA-256
  `1831438e9876c14ac39106daf420ceee745635a03bf9095c3743b94250d4932e`;
- Ubuntu executable artifact ID `9720297632`, name
  `hum-dev-executable-transport-v1-33271143652-1-99149715047-ubuntu-167bb6204431d9b199a8d38495dc7992750d78a1197c10c8e935f81c81d78bb0`,
  GitHub SHA-256
  `8455610c34f8820871d4fb83a7bf4430c7e1f214c6a65eb0681a929607a10c42`;
- Windows summary artifact ID `9720642673`, name
  `hum-evidence-summary-v1-33271143652-1-99149715128-windows`, GitHub SHA-256
  `45ea9e69819ba9c603a877beb6fb2b6b771954099626443a749f785277af4c88`;
  and
- Windows executable artifact ID `9720642963`, name
  `hum-dev-executable-transport-v1-33271143652-1-99149715128-windows-60b553169021ad74868e59d225f4b9f29e0fd1187aacd744a6c11e27768b9417`,
  GitHub SHA-256
  `baac7ef86464d6a52ec0af331bf20771a026d2faa50e7f60a9eaaa59165bfdc8`.

Each applicable Unit B implementation or repair commit used a separately
authorized `--no-verify` only after the external global `commit-msg` hook
failed because `sed`, `grep`, or `cat` was unavailable. The repository-owned
portable `hum-dev` validator and the hook's sole substantive Conventional
Commit rule accepted every exact subject. No compiler, test, security,
formatting, publication, or push validation was bypassed. Every publication
push was normal, non-force, and used no bypass.

Owner: BDFL (Ocean).

## Mission and present authorization

The mission is to preserve Hum's existing proof strength while reducing the
time, fragility, and manual ceremony required to implement, review, commit, and
publish new programs.

This issuance package closes terminal WO24 and proposes five bounded WO25
implementation units. It authorizes only fresh independent pre-issuance review
of the uncommitted two-path topology package. It does not authorize any WO25
implementation, staging, commit, push, CI, hook edit, cleanup, or successor
work.

WO25 changes infrastructure ownership, not Hum language meaning. Hum source
remains authoritative. Compiler semantics, typed facts, diagnostics,
selectors, mutations, fail-closed behavior, and required Windows/Linux
publication evidence remain at least as strong as they are at issuance.

## Authenticated WO24 terminal lifecycle and issuance baseline

This package starts from synchronized published state:

- `HEAD`, local `main`, cached `origin/main`, and authenticated live
  `origin/main`: `57349868aa40e625ce5b95d7ad578da2b68d9154`;
- ahead/behind: `0/0`;
- tree: `e9f36db31eb093821dd838ec29ed8018cc112e24`;
- subject: `docs(workorder): record unit a publication`;
- published active WO24 mode/blob:
  `100644 be166df6a4d765f0927dc26dfb9537a964c8253b`;
- worktree and index clean, with no untracked file;
- topology: one active, fifteen closed, zero root Work Orders;
- sole standalone marker: active WO24 line 4;
- eight stashes, eleven local branches under `refs/heads/archive` (including
  one local-only unrelated branch), ten cached/live publication archive refs,
  all other unrelated refs, persistent configuration, and seven historical
  target proof directories unchanged.

WO24 implementation is authenticated by commit
`9c729d08ae98327221d43c2153184d5e087eb514`, parent
`70eae19247682d8fa5c7ed10471c7e6e96bf4395`, tree
`cfbed4c3f1e6a6501ba18dbf2b478013cc4b8a7b`, and subject
`feat(program): run canonical hello world natively`. Full publication workflow
`33024468941`, attempt `1`, tested that exact commit. Ubuntu job `98362446915`
and Windows job `98362447053` both succeeded. The exact 118-selector ledger,
B01-B15, M01-M13, N01-N08, exact 13-byte `Hello, world!`
interpreter/native parity, H0634/H0635 controls, backend/readiness evidence,
Ubuntu Exhaustive `14,226` pairs, and required terminal full-preflight markers
all passed.

WO24 publication status is authenticated by commit
`57349868aa40e625ce5b95d7ad578da2b68d9154`, parent
`9c729d08ae98327221d43c2153184d5e087eb514`, tree
`e9f36db31eb093821dd838ec29ed8018cc112e24`, subject
`docs(workorder): record unit a publication`, and sole path/blob
`100644 be166df6a4d765f0927dc26dfb9537a964c8253b`. Its exact statistics are
`+109/-56`. Fast status workflow `33032320161`, attempt `1`, tested the exact
status commit. Ubuntu job `98387391337` and Windows job `98387391488` both
succeeded with exact `eligible_status_chain` binding to the implementation
anchor. Both passed 151 ordered classifier records twice deterministically,
565-file hygiene/readiness, alpha claims, and release readiness `0.0.1`, while
Cargo preparation, full preflight, and Exhaustive were skipped.

The combined issuance package performs exactly these topology changes:

1. change only WO24's Status body and current authorization gate to record its
   terminal CLOSED lifecycle;
2. remove exactly WO24's standalone active-marker line;
3. move the resulting immutable record to
   `workorders/closed/WORKORDER_24.md`; and
4. create this file with the sole standalone marker at line 4.

Every WO24 byte outside the Status body, current authorization gate, and
removed marker must remain identical to published blob
`be166df6a4d765f0927dc26dfb9537a964c8253b`. Replacing the two mutable regions
with their published forms and reinserting the line-4 marker must reconstruct
that blob exactly. The resulting topology must be one active, sixteen closed,
and zero root Work Orders.

## Evidence that remains valuable

WO25 preserves these durable obligations as product evidence, not ceremony:

- parser, type, effect, ownership, resource, and capability contracts;
- typed verified artifacts and exact producer/consumer authority;
- stable diagnostics, cause ownership, blame, and precedence;
- exact ordered, case-sensitive selectors that are proven to select;
- initialized mutation ownership and exact restoration;
- interpreter/native parity and source-derived output;
- backend, target, and readiness evidence;
- fail-closed behavior and no fallback;
- cross-platform publication on required Windows-MSVC and Linux-GNU jobs;
- explicit nonclaims and bounded public claims; and
- versioned machine-readable schemas and evidence identities.

No unit may trade one of these obligations for speed. Acceleration comes from
eliminating duplicate execution, re-parsing, shell ambiguity, and unauthenticated
manual reconstruction.

## Process debt to remove

The current path carries concrete infrastructure debt:

- `tools/check_all.ps1` is an approximately 500 KB PowerShell integration
  monolith (`578,356` bytes at issuance);
- mandatory Windows PowerShell 5.1 execution remains load-bearing;
- PowerShell 7/5.1 `PSModulePath` contamination can change behavior;
- disposable wrappers and quoting fail before useful evidence runs;
- PATH, shell, credential, and dubious-ownership failures recur;
- the machine-global commit hook depends substantively on `sed`, `grep`, and
  `cat`;
- candidate manifests and handoff digests are serialized manually and can
  disagree;
- status reviews repeatedly mine GitHub full logs;
- unchanged status evidence is rerun or reparsed;
- aggregate assertions can hide the one failing predicate;
- temporary proof and capture artifacts lack one typed cleanup owner;
- one-shot stop rules have been applied to harmless disposable preflight
  parsing as if it were a state-changing action;
- status-only reviews take many minutes; and
- a full local Fast currently takes approximately 25-27 minutes.

The machine-global hook is physically owned by the Windows machine-conventions
surface under `C:/Users/ocean/Documents/Codex/git-hooks`, not by this
repository. WO25 owns the portable rule and its tests only. Replacing the
machine-global hook body requires a separate BDFL-authorized machine-owner
change after the repository implementation is accepted; no Hum commit may
pretend that external file is in its path envelope.

## Frozen principles

1. Do not weaken compiler semantics, safety checks, mutations, selectors,
   platform requirements, or fail-closed behavior.
2. Move evidence into typed, repository-owned tooling.
3. Hum source remains authoritative; evidence graphs and summaries are
   derived.
4. Deterministic tools remain inside the trusted correctness boundary; AI does
   not.
5. Every retained ceremony must reject misuse, constrain authority, prove an
   obligation, justify an optimization, improve blame, or preserve necessary
   audit evidence.
6. Disposable helper construction may be corrected before a real
   state-changing action launches.
7. One-shot rules apply to actual Fast runs, commits, pushes, destructive
   cleanup, and CI reruns, not harmless preflight parsing.
8. Every aggregate assertion must report its individual failing predicate.
9. Temporary resources have one initialized owner. Normal completion, handled
   failure, timeout, and controlled termination require verified cleanup;
   process crashes and uncatchable kill, host-failure, or power-loss cases
   require authenticated stale-resource discovery and safe reconciliation by
   the next invocation rather than an impossible same-process guarantee.
10. No status-only review may rerun compiler suites already bound to an
    authenticated terminal-green CI anchor.

## Canonical architecture selection

Repository inspection establishes one root package and language executable,
`hum`, plus an internal crate layout under `crates/`. The language binary is
already large and carries Cranelift and platform authority. Developer-process
orchestration does not belong in that CLI.

WO25 therefore selects exactly one canonical repository-owned developer and
evidence executable:

```text
binary: hum-dev
package: crates/hum-dev
```

`hum-dev` owns typed developer commands, evidence profiles, candidate identity,
canonical summary serialization, commit-message validation, process capture,
and cleanup ownership. It is not a second compiler, language command, semantic
graph owner, test framework, package manager, or AI agent. `src/evidence.rs`
continues to describe Hum language evidence items and must not become a
developer runner.

The command model is fixed as:

```text
hum-dev evidence focused
hum-dev evidence status
hum-dev evidence full
hum-dev evidence exhaustive
hum-dev evidence summarize
hum-dev commit-message check
hum-dev candidate identity
hum-dev cleanup verify
hum-dev workorder status-facts
```

Commands use typed arguments and dispositions. Unknown profiles, fields,
stages, schema versions, and evidence states fail closed. During migration,
typed commands may invoke one explicitly named legacy boundary, but the
PowerShell wrapper may not decide compiler meaning or synthesize success.

## Versioned evidence summary ownership

`hum-dev` and `docs/HUM_EVIDENCE_SUMMARY_SCHEMA.md` jointly own
`hum.evidence_summary.v1`. One canonical serializer emits UTF-8 without BOM,
LF line endings, deterministic field and collection order, and a final LF.
The summary binds at least:

- schema and policy versions plus generator identity;
- exact commit SHA, parent where relevant, tree, candidate manifest, changed
  path modes/OIDs, raw and whitespace-insensitive accounting;
- platform, target, toolchain, compiler identity, dependency closure, and
  active profile;
- workflow name, event, run ID, attempt, numeric job ID, and checkout SHA;
- classifier mode, reason, authenticated anchor, and transitions;
- ordered selector ledger, case-sensitive uniqueness, selector stream hash,
  and exact suite counts;
- mutation IDs, individual results, restoration identities, and mutation
  stream hash;
- readiness, hygiene inventory, alpha claims, release readiness, and explicit
  nonclaims;
- the selected profile's exact expected stage set plus every skipped and
  executed stage in canonical order;
- terminal disposition, process exit, start/end monotonic timing, and stdout,
  stderr, and normalized-event stream SHA-256 hashes; and
- owned temporary-resource identity, residue class, and cleanup disposition.

Human prose and Work Order facts are projections from these fields. The
summary never becomes authority for Hum semantics, does not replace source or
typed artifacts, and cannot grant a commit, push, rerun, or optimization.

`hum.evidence_summary.v1` has one normative stage-closure invariant. The
selected profile and policy version determine one exact, canonically ordered
expected-stage set. Overall success requires exact set equality and exactly
one authenticated terminal disposition for every required stage. Every
permitted skip names a reason from the selected policy's closed skip-reason
catalog and authenticates the predicate that permits it. Missing, duplicate,
unknown, partial, stale, or unexpected stages and missing, duplicate, unknown,
or false skip reasons fail closed. Every stage record in one invocation binds
the same commit, policy, configuration digest, tool and compiler identities,
toolchain, workflow run/attempt where applicable, and evidence invocation ID.
The summary is only a derived projection: a green projection can never
override red, missing, or incomplete authenticated underlying stage evidence.

## Performance goals and deterministic comparison

Performance targets are obligations to measure, not permission to omit a
correctness predicate when a target is missed:

- commit-message verification: under 1 second;
- candidate/status identity verification: under 5 seconds;
- focused warm feedback: target under 60 seconds;
- status-only local verification: target under 2 minutes initially and under
  30 seconds eventually;
- status-only independent review: target under 2 minutes;
- no unnecessary GitHub full-log download for status review;
- full local evidence materially faster than the current 25-27 minutes, with a
  comparison target of 15 minutes or less on the issuance baseline machine;
  and
- Exhaustive remains an explicit CI, nightly, release, or milestone boundary,
  not routine documentation evidence.

Only a unit that changes a measured command's production path builds a timing
baseline. It records one cold and three warm samples for the authenticated
predecessor executable/commit and one cold and three warm samples for the
candidate executable/commit. Fixture inputs, machine, OS, toolchain, policy,
configuration, target, dependency closure, and power/performance mode are held
equal; the two intentionally different commit/executable identities are
recorded. The warm median is the comparison value; every sample remains
visible. Ordinary status invocations and reviews consume the accepted bound
baseline and never rebuild it. A routine status review must remain capable of
completing in under two minutes.

For commit-message, candidate, focused, full, and exhaustive commands, timing
starts immediately before the operating system is asked to create the exact
`hum-dev` process and stops only after process exit, stream capture, result
parsing, and owned controlled-path cleanup complete. This includes `hum-dev`
startup, repository and fixture discovery, producer work, serialization,
parsing, and cleanup. Compilation is excluded because the exact executable is
prebuilt and content-authenticated; when a unit changes compilation cost, that
build is a separately named timed command with the same boundaries.

For status review, timing starts immediately before the first authenticated
executable-artifact metadata query. It includes exact artifact resolution, the
pinned download step, archive-digest and one-file-shape authentication,
executable extraction and SHA-256 verification, platform pre-execution checks,
`hum-dev` launch, summary metadata queries and downloads, complete summary
authentication and two-platform comparison, status projection, stdout/stderr
capture, process exit, and all bootstrap- and `hum-dev`-owned cleanup. It stops
only after exact absence and no-survivor predicates have completed. It excludes
only the earlier terminal full producer and that producer's build and uploads.
The initial status-review requirement remains under two minutes, and Unit B
records one cold and three warm predecessor samples plus one cold and three warm
candidate samples under the frozen comparison method above. A timing miss is
red and may not skip or weaken evidence.

Each timing record identifies the exact executable, command/profile and
arguments, start/stop events, included/excluded phases, monotonic timer and
frequency, machine/CPU/memory, OS, target, toolchain and executable hashes,
repository commit/tree/worktree/index state, configuration and policy, cache
state, dependency closure, artifact size where applicable, exit/disposition,
stream hashes, and timeout. A timeout or failed correctness disposition is a
failed sample, is never discarded or replaced, and cannot support a
performance claim. A missed target is reported as a miss and returns for
review; it may never be made green by skipping correctness evidence.

## Unit A: typed portable evidence foundation

Unit A is the smallest independently useful spine. It adds `hum-dev`, makes the
existing substantive commit-message rule repository-owned and portable, binds
candidate/status identity without shell text hashing, creates the canonical
summary and cleanup types, and exposes typed focused/status/full/exhaustive
profiles. Legacy profile execution remains authoritative behind one named
adapter until later equivalence gates pass.

The canonical commit-message rule applies to the first-line subject only. It
accepts the present exact exemptions (`Merge `, `Revert `, `fixup! `, and
`squash! `) or a scoped Conventional Commit subject with an allowed type,
lowercase `[a-z0-9._-]+` scope, optional `!`, `: `, and a nonempty summary. The
subject value itself contains no CR, LF, or NUL. A normal LF-delimited message
may contain a multi-line body after that subject. The hook adapter reads the
message file as UTF-8 bytes, extracts the first physical line without
flattening it or concatenating body text, and sends that exact subject to the
same portable owner used by direct checks. Invalid UTF-8, BOM, NUL, an empty
subject, a direct subject input containing CR/LF, CR corruption in the file's
subject, missing scope, unknown type, or empty summary fails with one
predicate-level diagnostic. The permanent corpus proves the portable checker
and existing substantive hook rule agree exactly, including that a valid
subject with a valid multi-line body passes. The global hook remains unchanged
until its separate owner is authorized.

Candidate identity is computed from Git objects and raw bytes, never a
shell-piped diff. It reports exact commit/parent/tree/ref state, path mode and
OID, SHA-256, index/worktree state, and raw/whitespace-insensitive accounting
without writing an object or configuration.

### Unit A exact path envelope

| Path | Max + | Max - | Purpose |
| --- | ---: | ---: | --- |
| `Cargo.toml` | 24 | 4 | workspace/path-crate registration only |
| `Cargo.lock` | 24 | 4 | local package identities only |
| `src/sha256.rs` | 10 | 220 | thin compatibility re-export after shared extraction |
| `crates/hum-sha256/Cargo.toml` | 12 | 0 | one internal std-only hash crate |
| `crates/hum-sha256/src/lib.rs` | 225 | 0 | move existing SHA-256 authority with equivalence tests |
| `crates/hum-dev/Cargo.toml` | 20 | 0 | canonical executable package |
| `crates/hum-dev/src/main.rs` | 180 | 0 | process entry and terminal disposition |
| `crates/hum-dev/src/command.rs` | 160 | 0 | typed command/profile parsing |
| `crates/hum-dev/src/commit_message.rs` | 200 | 0 | portable canonical rule and blame |
| `crates/hum-dev/src/identity.rs` | 260 | 0 | Git/candidate/manifest identity |
| `crates/hum-dev/src/summary.rs` | 280 | 0 | v1 types and canonical serialization |
| `crates/hum-dev/src/cleanup.rs` | 200 | 0 | owned temporary-resource guard |
| `crates/hum-dev/tests/cli.rs` | 360 | 0 | process, portability, corruption, and timing evidence |
| `fixtures/evidence/commit_message_cases.v1.txt` | 80 | 0 | exact accept/reject corpus |
| `fixtures/evidence/status_candidate.v1.json` | 100 | 0 | candidate identity fixture |
| `docs/HUM_EVIDENCE_SUMMARY_SCHEMA.md` | 220 | 0 | schema, authority, fields, and nonclaims |
| `tools/check_all.ps1` | 90 | 30 | integrated equivalence and selector entry only |
| **Unit A total** | **2,445** | **258** | **17 non-borrowable paths** |

| Unit A category | Paths | Max + | Max - |
| --- | ---: | ---: | ---: |
| Manifests and lock | 4 | 80 | 8 |
| Shared hash Rust | 2 | 235 | 220 |
| `hum-dev` Rust and tests | 7 | 1,640 | 0 |
| Permanent fixtures | 2 | 180 | 0 |
| Schema documentation | 1 | 220 | 0 |
| PowerShell integration | 1 | 90 | 30 |
| **Unit A category total** | **17** | **2,445** | **258** |

No third-party dependency is authorized. Unit A may add only the two listed
repository path packages. Existing `src/sha256.rs` vectors and callers must be
byte- and behavior-equivalent through the shared internal crate before its old
implementation is removed.

## Unit B: fast status and publication evidence

Unit B has one repository-owned transport contract with no implicit owner. It
separates terminal full-anchor production, fast bootstrap transport, and typed
status consumption. No one of those layers may substitute for another.

Only a terminal full evidence-anchor job produces a summary. After all required
underlying full evidence for that platform reaches its authenticated terminal
disposition, the exact ordered full-lane steps are
`generate_evidence_summary` (`Generate evidence summary`),
`upload_evidence_summary` (`Upload evidence summary`), and
`upload_hum_dev_executable` (`Upload hum-dev executable`). The producer is
`hum-dev evidence summarize --output <owned-path>`. Through `hum-dev`'s
metadata adapter it resolves the current numeric job ID exactly once from
authenticated current-run/current-attempt metadata and rejects zero or multiple
matches. A fast or status-only job never generates or uploads a replacement
summary or executable artifact.

The summary upload contains exactly one regular file named
`hum-evidence-summary.v1.json`: one canonical `hum.evidence_summary.v1` JSON
object encoded as UTF-8 without BOM, with LF/final-LF framing. Its exact
artifact-name grammar remains:

```text
hum-evidence-summary-v1-<run_id>-<run_attempt>-<numeric_job_id>-<platform>
```

`run_id`, `run_attempt`, and `numeric_job_id` are positive canonical decimal
integers with no sign or leading zero; `platform` is exactly `ubuntu` or
`windows`. Retention is exactly 14 days. The archive may contain no executable,
manifest sidecar, directory entry, link, reparse point, or extra file.

Each terminal full Ubuntu job also publishes one separate artifact archive
containing exactly one regular executable named `hum-dev`; each terminal full
Windows job publishes the same contract with exactly one regular executable
named `hum-dev.exe`. The exact executable artifact-name grammar is:

```text
hum-dev-executable-transport-v1-<run_id>-<run_attempt>-<numeric_job_id>-<platform>-<lowercase_sha256>
```

The three numeric fields and platform use the canonical values above, and
`lowercase_sha256` is exactly 64 lowercase hexadecimal characters over the
unmodified executable bytes. The grammar is identical on both platforms except
for the explicit platform value. Retention is exactly 14 days. This artifact is
transport/bootstrap data only: it is not evidence authority, cache authority,
a release asset, package distribution, installer, or substitute for source or
summary facts. Fast jobs never publish it.

The current PowerShell classifier remains only the temporary fail-closed lane
selector until its later authorized migration. It may return the exact
terminal-green full anchor, run ID, attempt, Ubuntu job ID, and Windows job ID
that it already authenticates, but it may not generate, parse, authenticate,
compare, or accept `hum.evidence_summary.v1`. Multiple consecutive recognized
status commits may retain that same authenticated full anchor while all required
summary and executable artifacts remain valid. This changes no classifier
trust, topology, transition, or fail-closed rule.

After fast classification selects one exact terminal-green full anchor, the
exact ordered fast bootstrap steps are
`start_status_review_and_resolve_hum_dev` (`Resolve status executable`),
`download_hum_dev_executable` (`Download hum-dev executable`), and
`authenticate_and_run_status_only_evidence` (`Run status-only evidence`). The
bootstrap resolves exactly one artifact ID for only the current platform's
exact executable artifact, downloads that artifact by ID from the selected
repository/run into a newly initialized owned directory with the pinned
download action and no implicit current-run or name-pattern fallback, and keeps
the raw archive unexpanded until its own checks run.

Before any process creation, the thin workflow bootstrap authenticates the
repository, workflow name/path, event, anchor commit and tree, run ID, attempt,
numeric producing job ID, producing job status/conclusion, platform and target,
artifact ID/name/size/GitHub SHA-256 digest, the exact artifact-name grammar,
the archive's exact one-file shape, absence of directory/absolute/parent or
separator traversal, regular-file/no-link/no-reparse properties, exact
executable filename, and the lowercase executable SHA-256 embedded in the
artifact name. It independently hashes the raw archive against the GitHub
digest and the extracted executable against the name-bound SHA-256, requiring
exact equality. These checks use only fixed-workflow platform primitives and
may not interpret summary fields, synthesize an evidence disposition, use full
logs, trust ambient PATH, reuse a cache or local copy, or fall back.

The Windows and Ubuntu branches remain explicit. Windows launches only after
the exact regular-file/no-reparse checks and performs executable cleanup after
process exit, stream completion, handle disposal, and child/process quiescence.
Because GitHub's archived artifact transport does not preserve POSIX executable
mode bits, Ubuntu sets execute permission only on the already shape-, digest-,
and byte-authenticated owned file by an already-available platform primitive,
then proves the required executable bit, regular-file/no-link identity, and
unchanged SHA-256 before launch. No ambient `chmod` lookup is authority. Only
after every pre-execution predicate passes may the executable launch.

The exact semantic consumer is the authenticated bootstrapped executable
running `hum-dev evidence status`. That command retains ownership of GitHub
run, job, and summary-artifact metadata queries; exact `gh run download`
summary byte transport into its initialized owned directory; canonical parsing;
complete field authentication; Ubuntu/Windows agreement; status projection;
terminal disposition; and local summary cleanup. It binds and reports the exact
GitHub CLI executable path, version, and SHA-256. Before consuming evidence it
authenticates both platform summaries. Its running platform-specific executable
SHA-256, toolchain, and target must equal only the corresponding producer fields
in the authenticated summary for the current platform. Each platform-specific
producer executable SHA-256, toolchain, and target is authenticated against its
corresponding platform summary. The running executable's platform-neutral source
commit, tree, and Cargo/dependency-closure identities must equal the corresponding
platform-neutral fields in both authenticated summaries. The two summaries must
agree exactly on every field the schema defines as platform-neutral, including
source commit, tree, Cargo/dependency closure, run ID, attempt, and every other
shared anchor identity. No single platform executable, toolchain, or target is
required to equal both platform-specific values.

The bootstrap's pre-execution transport check and `hum-dev`'s self/summary
cross-check are distinct load-bearing predicates; neither substitutes for the
other. Summary artifacts continue to bind the full-anchor producer executable
identity. They need not and cannot self-assert artifact IDs or GitHub digests
created only after serialization; the consumer authenticates those later
transport facts directly from GitHub metadata before field consumption. Shell
and PowerShell own no schema meaning or evidence acceptance, and GitHub Actions
owns only the authenticated remote transport and expiry.

GitHub owns remote expiry at 14 days, and no remote delete action is authorized.
The workflow bootstrap owns only its downloaded executable archive/extraction
directory and removes it after `hum-dev` exits on normal and handled-failure
paths, proving exact absence and no process, stream, or handle survivor.
`hum-dev`'s initialized cleanup guard separately owns downloaded summary
archives and extracted JSON and retains the existing fail-closed cleanup
contract.

Missing, expired, ambiguous, duplicate, stale, extra-file, wrong-platform,
wrong-job/run/attempt, wrong repository/workflow/event, malformed-name,
path-traversal, link/reparse, size/digest/hash mismatch, same-length corruption,
terminal-red/incomplete, mixed-binding, cleanup failure, or unavailable
executable or summary artifacts fail closed before projection. Artifact expiry
or transport failure never triggers Cargo/rustc, cache or local-copy reuse,
logs, a new workflow dispatch, a rerun, inferred success, or PowerShell semantic
fallback. The terminal report names every individual failed predicate and
requires separate authority for any later full producer.

`hum-dev workorder status-facts` produces a deterministic proposed projection
for the recognized mutable Work Order regions. Human review remains required
for public claims and authorization meaning. An apply mode may write only
after the exact target, base blob, mutable spans, and requested output are
explicit; every other byte is immutable and independently reconstructible.

Status-only local validation recognizes only the canonical status transition.
It runs no Cargo, Rust selector, compiler, interpreter, native, Fast, or
Exhaustive stage when all changed bytes are inside recognized mutable Work
Order regions and the authenticated summary chain is terminal-green.

Unit B permanently preserves selectors 125-128 and I05-I07 ownership, the
frozen 124-selector prefix, the complete ordered 128-selector ledger, all 151
classifier cases twice deterministically, the complete summary corruption
matrix and S01-S06, immutable Work Order reconstruction, and the no-full-log
and no-Cargo/compiler/Fast sentinels. Focused evidence additionally covers both
immutable action pins; both artifact-name grammars; exact one-file shape;
platform/name/hash substitution; cross-run, attempt, job, and platform swaps;
wrong GitHub digest; same-length executable corruption; absent, expired, and
duplicate artifacts; pre-execution rejection; running-self-hash/summary
disagreement; cleanup on every controlled terminal path; Windows post-exit
handle/process quiescence; Ubuntu regular-file/executable-bit handling; forbidden
cache, local, log, build, and PowerShell-owner fallbacks; and consecutive status
chains consuming one still-valid full anchor. Every aggregate failure emits
each individual predicate. None of these checks belongs to Unit C shell
portability or Unit D generalized cache/reuse authority: permitted same-anchor
artifact consumption is authenticated transport of the exact full producer.

### Unit B exact path envelope

Each listed ceiling applies independently to both raw and
whitespace-insensitive accounting and is non-borrowable by another path or
category.

| Path | Max + (raw/WS) | Max - (raw/WS) | Purpose |
| --- | ---: | ---: | --- |
| `.github/workflows/ci.yml` | 340 | 100 | full-only summary/executable uploads plus fixed fast download, pre-execution authentication, launch, timing, and cleanup bootstrap |
| `crates/hum-dev/src/main.rs` | 80 | 20 | status and Work Order command routing |
| `crates/hum-dev/src/command.rs` | 100 | 20 | typed status arguments |
| `crates/hum-dev/src/summary.rs` | 360 | 80 | full-anchor producer/executable identities, authenticated CI fields, canonical parsing, and agreement checks |
| `crates/hum-dev/src/status.rs` | 700 | 0 | authenticated metadata transport, self/summary cross-check, status projection, and summary cleanup |
| `crates/hum-dev/src/workorder.rs` | 300 | 0 | immutable-region projection and status facts |
| `crates/hum-dev/tests/cli.rs` | 320 | 30 | process, bootstrap boundary, no-fallback, cleanup, and no-log-download evidence |
| `fixtures/evidence/job_summary_ubuntu.v1.json` | 120 | 0 | canonical Ubuntu summary |
| `fixtures/evidence/job_summary_windows.v1.json` | 120 | 0 | canonical Windows summary |
| `fixtures/evidence/summary_corruption_cases.v1.json` | 420 | 0 | complete summary, executable transport, substitution, and cleanup corruption matrix |
| `docs/HUM_EVIDENCE_SUMMARY_SCHEMA.md` | 300 | 60 | full-anchor production, producer identity, artifact, consumer, ownership, and status projection contract |
| `tools/check_workorder_status_boundary.ps1` | 220 | 100 | temporary lane/anchor selector and exact terminal full-step authentication only |
| `tools/test_workorder_status_boundary.ps1` | 280 | 100 | unchanged 151-case trust corpus, terminal-step obligations, and same-anchor chain controls |
| `tools/check_all.ps1` | 200 | 80 | integrated 128-ledger, mutation, artifact, fallback, cleanup, and timing evidence |
| **Unit B total** | **3,860** | **590** | **14 non-borrowable paths** |

| Unit B category | Paths | Max + | Max - |
| --- | ---: | ---: | ---: |
| `hum-dev` Rust and tests | 6 | 1,860 | 150 |
| Permanent fixtures | 3 | 660 | 0 |
| Schema documentation | 1 | 300 | 60 |
| Workflow | 1 | 340 | 100 |
| PowerShell integration/tests | 3 | 700 | 280 |
| **Unit B category total** | **14** | **3,860** | **590** |

## Unit C: shell and hook portability

The canonical commit-message rule remains in `hum-dev`; repository tests prove
byte-for-byte verdict/help equivalence with the accepted legacy corpus.
Substantive validation must not depend on `sed`, `grep`, or `cat`. A separately
authorized machine-conventions change may later make the global hook a thin
launcher for the accepted binary. That external edit is not a WO25 repo path,
commit, or acceptance claim.

PowerShell 7 is the only temporary scripting runtime for new Hum orchestration.
Mandatory PS5.1 execution may retire only after every load-bearing contract
formerly attributed to it is enumerated and old/new equivalence passes on
Windows and Ubuntu. Remaining PowerShell becomes thin, declarative, and unable
to classify compiler semantics. `PSModulePath`, PATH, quoting, credential, and
dubious-ownership inputs are explicit environment facts rather than ambient
success assumptions.

### Unit C exact path envelope

| Path | Max + | Max - | Purpose |
| --- | ---: | ---: | --- |
| `crates/hum-dev/src/main.rs` | 40 | 10 | portable runner entry |
| `crates/hum-dev/src/command.rs` | 81 | 20 | explicit environment options |
| `crates/hum-dev/src/commit_message.rs` | 100 | 30 | legacy equivalence and portable help |
| `crates/hum-dev/src/shell.rs` | 352 | 0 | process launch, environment, and stream ownership |
| `crates/hum-dev/tests/cli.rs` | 160 | 20 | Windows/Linux shell corruption tests |
| `tools/check_all.ps1` | 95 | 160 | delegate migrated orchestration |
| `tools/run_fast_evidence.ps1` | 60 | 220 | thin PS7 adapter |
| `tools/test_fast_evidence_capture.ps1` | 100 | 260 | portable equivalence tests |
| `tools/check_workorder_status_boundary.ps1` | 60 | 100 | thin status adapter |
| `tools/test_workorder_status_boundary.ps1` | 80 | 160 | PS7-only compatibility matrix |
| `.github/workflows/ci.yml` | 60 | 80 | invoke canonical portable path |
| `docs/TESTING_STRATEGY.md` | 80 | 40 | supported shells and evidence ownership |
| `CONTRIBUTING.md` | 40 | 20 | portable commit-message workflow |
| **Unit C total** | **1,308** | **1,120** | **13 non-borrowable paths** |

| Unit C category | Paths | Max + | Max - |
| --- | ---: | ---: | ---: |
| `hum-dev` Rust and tests | 5 | 733 | 80 |
| PowerShell wrappers/tests | 5 | 395 | 900 |
| Workflow | 1 | 60 | 80 |
| Documentation | 2 | 120 | 60 |
| **Unit C category total** | **13** | **1,308** | **1,120** |

The `shell.rs` exceptions are solely for the proven readable implementations
of fixed Windows system-command identity and the native authenticated
`ProgramFiles` known-folder resolver. Native-system `cmd.exe` is selected from
the OS-derived system directory and may legitimately be Windows-hard-linked
into WinSxS. Substitutable `pwsh`, `link.exe`, `vswhere`, Git, Cargo, and other
executables retain their stricter substitution and link rules. Rustfmt expands
the preserved compact `shell.rs` to 352 lines; the shortest tested readable
manual refactor was 339 lines. The 352-line ceiling authorizes canonical
rustfmt output instead of line-count code-golf. Its unused capacity remains
non-borrowable and authorizes no new feature, behavior, dependency, path,
evidence weakening, documentation expansion, or later-unit work. The
`tools/check_all.ps1` increase is solely for the already-demonstrated Unit C
environment, receipt, corruption, closed-child framing, and diagnostic
precedence evidence. Neither increase is borrowable or grants unrelated
production, feature, test, documentation, workflow, or later-unit budget.
The current `command.rs` candidate is `+56/-3`; the smallest tested readable
complete Rust bootstrap closure is `+71/-3`. Its 80-line ceiling leaves nine
explicitly non-borrowable additions beyond that measured minimum to avoid
another line-exact amendment cycle. The subsequently required readable,
rustfmt-clean direct-toolchain layout classifier is `+81/-3`, exactly one line
beyond that ceiling; the 81-line ceiling authorizes only this proven pressure.
This closure is limited to native profile
resolution, authenticated rustup settings and toolchain, direct Cargo and
`rustc` identities, explicit configuration roots, ordered `PATH` construction,
and effective child Cargo identity. Unused capacity cannot be borrowed and
grants no feature, dependency, unrelated refactor, new path, compiler behavior,
or Unit D-E authority.

## Unit D: change-aware execution and cleanup

One deterministic impact map relates changed paths and semantic fingerprints
to required selectors, mutations, and evidence profiles. It fails closed on an
unknown path, unknown policy, ambiguous fingerprint, changed load-bearing
predicate, missing dependency, or incomplete closure.

The canonical reuse key has these twelve and only these dimensions; no
consumer may add an ambient or undeclared dimension:

1. source/candidate identity: commit, tree, path set, modes, OIDs, and raw
   source hashes;
2. complete dependency closure, including lockfile and repository path crates;
3. evidence-policy identity and version;
4. load-bearing configuration and environment digest;
5. exact ordered selector ledger and hash;
6. exact ordered mutation ledger and hash;
7. `hum-dev`, compiler, and generator executable identities and hashes;
8. Rust/Cargo and external transport toolchain identities;
9. OS, architecture, platform, and compilation target;
10. semantic-input, typed-artifact, and impact-fingerprint graph digest;
11. schema and canonical-serializer identity/version; and
12. exact command, profile, arguments, and expected-stage-set identity.

Reuse requires equality in all twelve dimensions. Changing any one dimension
invalidates reuse even if the displayed aggregate digest is unchanged. Each
dimension has one permanent load-bearing corruption case in
`fixtures/evidence/cache_key_cases.v1.json`; a case passes only when an
instrumented producer actually recomputes in the test harness or the production
consumer explicitly rejects and requests fresh authorization. A changed label,
displayed digest, or cache-miss message without observed recomputation or
rejection earns no credit. Evidence is never reusable after an owned input
changes, and a key field outside this canonical list is a schema defect rather
than hidden cache authority.

All temporary directories, subprocesses, captures, manifests, and proof outputs
have one initialized owner before launch. Normal completion and handled failure
must reap children and remove owned paths before terminal reporting. Timeout or
controlled termination must terminate and reap the owned child tree, attempt
owned-path cleanup, and fail with every survivor named. A catchable panic uses
the same guard, but a process crash may prevent in-process cleanup. Uncatchable
kill, host failure, and power loss have no same-run cleanup guarantee.

The next `hum-dev` invocation runs stale-resource discovery before creating new
resources. `hum-dev cleanup verify` distinguishes current-invocation residue,
authenticated historical residue carrying the repository/tool/owner token and
creation identity, foreign paths, and unrecoverable ambiguity. It may safely
reconcile only authenticated owned historical residue, reports and leaves
foreign paths untouched, and fails closed without deletion on ambiguous or
unrecoverable identity. Cleanup reports every predicate and survivor and never
broadens deletion authority.

The seven historical `target/hum-math-obligations-*` proof directories are an
inventory outside automatic cleanup. Their deletion remains unauthorized
unless a later prompt names each exact path, recovery implication, and
approval. Stashes and archives remain outside every cleanup command.

### Unit D exact path envelope

| Path | Max + | Max - | Purpose |
| --- | ---: | ---: | --- |
| `crates/hum-dev/src/main.rs` | 40 | 10 | impact/cache command routing |
| `crates/hum-dev/src/command.rs` | 60 | 20 | typed impact and reuse arguments |
| `crates/hum-dev/src/summary.rs` | 80 | 30 | reuse-key and performance fields |
| `crates/hum-dev/src/cleanup.rs` | 160 | 40 | controlled cleanup and stale-resource reconciliation |
| `crates/hum-dev/src/impact.rs` | 360 | 0 | deterministic fail-closed impact map |
| `crates/hum-dev/src/cache.rs` | 340 | 0 | content-addressed evidence reuse |
| `crates/hum-dev/tests/cli.rs` | 260 | 30 | stale-key, unknown-path, and survivor evidence |
| `fixtures/evidence/impact_map_cases.v1.json` | 160 | 0 | changed-path/fingerprint corpus |
| `fixtures/evidence/cache_key_cases.v1.json` | 160 | 0 | complete twelve-dimension cache corruption corpus |
| `tools/check_all.ps1` | 120 | 180 | consume typed impact decisions |
| `docs/HUM_EVIDENCE_SUMMARY_SCHEMA.md` | 100 | 30 | reuse and timing identity |
| `docs/TESTING_STRATEGY.md` | 80 | 30 | profile selection and cache non-authority |
| **Unit D total** | **1,920** | **370** | **12 non-borrowable paths** |

| Unit D category | Paths | Max + | Max - |
| --- | ---: | ---: | ---: |
| `hum-dev` Rust and tests | 7 | 1,300 | 130 |
| Permanent fixtures | 2 | 320 | 0 |
| PowerShell integration | 1 | 120 | 180 |
| Documentation | 2 | 180 | 60 |
| **Unit D category total** | **12** | **1,920** | **370** |

## Unit E: migration and retirement

`tools/check_all.ps1` shrinks incrementally at producer, validator, consumer,
or runner boundaries. Each migrated boundary runs old and new against the same
authenticated candidate and proves equivalent stage order, selector/mutation
identity, exit/disposition, diagnostics, streams, summaries, and cleanup. No
large rewrite receives credit for approximate parity.

PowerShell authority is removed only after fresh independent acceptance and
terminal required cross-platform CI for the exact migrated boundary. Historical
diagnostics, selectors, mutations, schemas, and externally load-bearing
evidence identities remain stable or receive an explicit versioned migration.

The terminal deliverable is one ownership map in
`hum.evidence_summary.v1` schema documentation and executable tests. Every
gate has exactly one owner: Hum compiler producer, `hum-dev`, thin wrapper,
workflow, GitHub artifact metadata, reviewer, or BDFL. No gate may have two
semantic owners or no owner.

### Unit E exact path envelope

| Path | Max + | Max - | Purpose |
| --- | ---: | ---: | --- |
| `crates/hum-dev/src/main.rs` | 40 | 10 | terminal migrated routing |
| `crates/hum-dev/src/command.rs` | 60 | 20 | retired-command diagnostics |
| `crates/hum-dev/src/summary.rs` | 80 | 20 | final ownership map projection |
| `crates/hum-dev/src/ownership.rs` | 260 | 0 | exact gate ownership registry |
| `crates/hum-dev/tests/cli.rs` | 180 | 30 | no-remaining-consumer proof |
| `tools/check_all.ps1` | 160 | 1,800 | incremental authority removal |
| `tools/run_fast_evidence.ps1` | 40 | 1,000 | retire migrated capture authority |
| `tools/test_fast_evidence_capture.ps1` | 80 | 1,000 | retain only adapter contract tests |
| `tools/check_workorder_status_boundary.ps1` | 60 | 400 | retire migrated status authority |
| `tools/test_workorder_status_boundary.ps1` | 80 | 700 | retain compatibility boundary tests |
| `tools/test_exact_rust_selector.ps1` | 40 | 200 | delegate exact selector ownership |
| `.github/workflows/ci.yml` | 40 | 80 | final typed profile invocation |
| `docs/TESTING_STRATEGY.md` | 120 | 100 | final execution/ownership model |
| `docs/ARCHITECTURE.md` | 80 | 40 | evidence-spine ownership and nonclaims |
| **Unit E total** | **1,320** | **5,400** | **14 non-borrowable paths** |

| Unit E category | Paths | Max + | Max - |
| --- | ---: | ---: | ---: |
| `hum-dev` Rust and tests | 5 | 620 | 80 |
| PowerShell migration | 6 | 460 | 5,100 |
| Workflow | 1 | 40 | 80 |
| Documentation | 2 | 200 | 140 |
| **Unit E category total** | **14** | **1,320** | **5,400** |

The five unit tables contain 70 authorized path occurrences with aggregate
telemetry `+10,853/-7,738`. This sum is not a cross-unit borrowing pool. Every
unit and category ceiling is independently non-borrowable, and a path may be
edited in a later unit only where it is listed again.

## Exact selector and mutation additions

The published 118-selector root ledger remains byte-for-byte ordered and
case-sensitive. WO25 adds exactly these 18 `hum-dev` selectors in this order;
the integrated combined ledger becomes 136 unique selectors:

1. `commit_message::tests::canonical_rule_is_portable_and_exact`
2. `identity::tests::candidate_identity_binds_commit_tree_index_and_paths`
3. `summary::tests::evidence_summary_v1_is_canonical_and_hash_bound`
4. `cleanup::tests::owned_resources_close_on_every_controlled_terminal_path`
5. `command::tests::evidence_profiles_are_typed_and_fail_closed`
6. `cli::legacy_equivalence_preserves_exit_stages_and_stream_hashes`
7. `status::tests::job_summary_binds_run_attempt_job_sha_tree_and_platform`
8. `status::tests::status_review_consumes_summaries_without_full_logs`
9. `workorder::tests::status_facts_touch_only_authenticated_mutable_regions`
10. `summary::tests::cross_platform_status_agreement_is_exact`
11. `shell::tests::pwsh7_adapter_is_thin_declarative_and_environment_bound`
12. `commit_message::tests::legacy_hook_corpus_matches_portable_rule`
13. `cli::preflight_repairs_stop_before_state_changing_launch`
14. `impact::tests::impact_map_is_deterministic_complete_and_fail_closed`
15. `cache::tests::reuse_key_binds_every_declared_dimension`
16. `cleanup::tests::stale_owned_residue_is_discovered_and_safely_reconciled`
17. `ownership::tests::every_gate_has_one_load_bearing_owner`
18. `cli::retired_powershell_authority_has_no_remaining_consumer`

Each selector must select exactly once through the integrated route. Deletion,
duplication, reordering, casing change, zero-match filtering, or fabricated
replacement fails the ledger.

WO25 adds exactly twelve initialized infrastructure mutations. Each mutation
must compile, alter one authenticated predicate, fail at its named selector or
disposition, expose the individual predicate, and restore exact bytes:

| ID | Initialized weakening | Required escaped disposition |
| --- | --- | --- |
| I01 | accept an unscoped or unknown Conventional Commit type | invalid commit message passes the permanent corpus |
| I02 | omit commit/tree/index/path binding from candidate identity | a foreign or dirty candidate authenticates |
| I03 | reorder or omit a canonical summary field | summary bytes/hash remain falsely accepted |
| I04 | disable initialized cleanup on a handled failure | authenticated current-run residue survives a controlled disposition |
| I05 | omit run, attempt, numeric job, SHA, tree, or platform binding | a cross-run or cross-job summary substitution authenticates |
| I06 | consume full logs when compact summaries are present | the no-log-download sentinel records forbidden access |
| I07 | allow status-only evidence to invoke Cargo/compiler/Fast | a forbidden stage appears without rejection |
| I08 | collapse predicate failures into one aggregate Boolean | the exact failing predicate disappears from output |
| I09 | omit a changed semantic fingerprint from the impact map | a load-bearing change reuses the smaller profile |
| I10 | omit any canonical reuse-key dimension | a K01-K12 changed input reuses evidence without recomputation or rejection |
| I11 | bypass controlled cleanup or stale-resource reconciliation | controlled residue survives, or authenticated historical residue is not safely reconciled |
| I12 | retire a PowerShell owner without old/new equivalence | an unowned or double-owned gate reaches publication |

I01-I04 belong to Unit A, I05-I07 to Unit B, I08 to Unit C, I09-I11 to Unit
D, and I12 to Unit E. No mutation earns credit from compilation failure,
panic, missing fixture, broad-suite failure, no-op replacement, or incomplete
restoration.

## Corruption controls and performance fixtures

Permanent summary corruption covers schema, policy, generator, commit, parent,
tree, candidate manifest, mode/OID, platform, target, compiler, dependency
closure, workflow, event, run, attempt, numeric job, checkout SHA, anchor,
transitions, selector order/hash/count, mutation order/hash/result, suite count,
readiness, hygiene inventory, claims, release version, expected/skipped stage,
terminal disposition, timing, stdout hash, stderr hash, event-stream hash, and
cleanup disposition. Single-field, same-length substitution and cross-platform
swaps must reject independently.

Unit B executable-bootstrap corruption additionally covers immutable
upload/download pin identity, artifact grammar and cardinality, artifact ID and
GitHub digest, archive shape and entry path, regular/link/reparse type,
platform/executable name, run/attempt/job substitution, embedded executable
SHA-256, same-length executable byte corruption, producer-summary executable
identity, running self-hash, Ubuntu execute permission, Windows post-exit handle
quiescence, and bootstrap-owned cleanup. Each corruption must reject before
process creation when its predicate is pre-execution; no later summary failure
may receive credit for a missing bootstrap rejection.

The summary corruption corpus has these non-degenerate stage-closure rows:

| ID | Corruption | Required result |
| --- | --- | --- |
| S01 | delete one required stage while leaving overall success | reject missing stage |
| S02 | duplicate one terminal stage record byte-for-byte | reject duplicate stage |
| S03 | add a plausible but profile-unknown terminal stage | reject unexpected stage |
| S04 | mark a required stage skipped with a missing or false reason/predicate | reject invalid skip |
| S05 | splice a stage with another commit, policy, configuration, toolchain, attempt, or invocation binding | reject mixed binding |
| S06 | retain overall success over one red, partial, stale, or incomplete underlying stage | reject false terminal success |

The cache-key corpus enumerates the complete reuse-key mutation set. Every row
holds the other eleven dimensions fixed, changes the named owned input, and
uses an instrumented producer sentinel to prove actual recomputation or an
explicit rejection disposition:

| ID | Sole changed dimension | Required observation |
| --- | --- | --- |
| K01 | source/candidate commit, tree, path/mode/OID, or raw hash | recompute or reject candidate mismatch |
| K02 | dependency closure, lockfile, or path-crate identity | recompute or reject closure mismatch |
| K03 | evidence-policy identity/version | recompute or reject policy mismatch |
| K04 | load-bearing configuration/environment digest | recompute or reject configuration mismatch |
| K05 | selector name, order, membership, or ledger hash | recompute or reject selector mismatch |
| K06 | mutation name, order, membership, or ledger hash | recompute or reject mutation mismatch |
| K07 | `hum-dev`, compiler, or generator executable hash | recompute or reject tool mismatch |
| K08 | Rust, Cargo, or transport-toolchain identity | recompute or reject toolchain mismatch |
| K09 | OS, architecture, platform, or target | recompute or reject platform mismatch |
| K10 | semantic input, typed artifact, or impact fingerprint | recompute or reject semantic-input mismatch |
| K11 | schema or canonical-serializer identity/version | recompute or reject schema mismatch |
| K12 | command, profile, arguments, or expected-stage-set identity | recompute or reject command/profile mismatch |

The exact performance fixtures are:

- `fixtures/evidence/commit_message_cases.v1.txt`;
- `fixtures/evidence/status_candidate.v1.json`;
- `fixtures/evidence/job_summary_ubuntu.v1.json`;
- `fixtures/evidence/job_summary_windows.v1.json`;
- `fixtures/evidence/summary_corruption_cases.v1.json`;
- `fixtures/evidence/impact_map_cases.v1.json`; and
- `fixtures/evidence/cache_key_cases.v1.json`.

Generated temporary variants live only under an owned OS temporary directory.
No capture, benchmark, proof, credential, PATH, trust, or configuration artifact
may remain after a normal, handled-failure, timeout, or controlled-termination
test disposition. Crash or uncatchable-interruption fixtures must prove that
the next invocation discovers, authenticates, reports, and safely reconciles
owned historical residue while leaving foreign or ambiguous paths untouched.

## Dependency, schema, compatibility, and migration policy

- No unit authorizes a third-party Cargo dependency. A network library, parser,
  serializer, regex engine, async runtime, or GitHub SDK requires a substantive
  Work Order amendment. Unit B's sole workflow-dependency exception is exactly
  these two official actions at these literal immutable full commit SHAs:

  ```text
  actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a
  actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c
  ```

  `actions/upload-artifact` is authorized only in the fixed
  `upload_evidence_summary` and `upload_hum_dev_executable` steps. Both use
  exact one-file paths, archived transport, `if-no-files-found: error`,
  `retention-days: 14`, `overwrite: false`, and no hidden-file inclusion.
  `actions/download-artifact` is authorized only in the fixed
  `download_hum_dev_executable` fast bootstrap step, by the one authenticated
  artifact ID, repository, and selected anchor run ID, with the GitHub token,
  `skip-decompress: true`, `digest-mismatch: error`, and the exact initialized
  owned path. Name, pattern, multi-artifact, implicit current-run, and extracted
  fallback forms are forbidden.

  Read-only official GitHub release, tag-ref, and commit inspection on
  2026-08-28 authenticated `actions/upload-artifact` release/tag `v7.0.1`
  directly to verified commit
  `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`, and
  `actions/download-artifact` release/tag `v8.0.1` directly to verified commit
  `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c`. The Work Order freezes only
  those SHAs; the release tags are provenance, not workflow authority. Unit B
  focused evidence and independent review must reauthenticate the literal pins.
  No tag, mutable ref, action use outside those exact steps, other action or
  dependency, Cargo dependency, SDK, installer, cache, or network library is
  authorized, and WO25 neither installs nor invokes either action locally.

  The summary consumer uses the existing authenticated GitHub CLI only through
  the exact metadata and `gh run download` adapter above; its executable path,
  version, and SHA-256 are invocation-bound, absence or drift fails closed, and
  WO25 neither installs nor modifies it. The bootstrap uses only already
  available fixed-workflow platform primitives for its metadata and archive
  trust checks. Neither action, GitHub CLI, workflow, nor PowerShell owns schema
  meaning or evidence disposition.
- `hum-sha256` is the sole shared hash owner. Extraction must preserve every
  existing vector, API result, and caller behavior.
- `hum.evidence_summary.v1` is owned by `hum-dev`; Hum compiler types and source
  remain the semantic producers. Workflow and PowerShell may transport but not
  reinterpret it.
- V1 is additive during Units A-D. Removing or renaming a load-bearing field,
  changing canonical bytes, or widening an enum requires a schema-version
  decision and migration fixtures.
- Legacy PowerShell remains authoritative at an unmigrated boundary. Typed code
  becomes authoritative only after same-candidate old/new equivalence,
  independent acceptance, an exact commit, and terminal Windows/Linux CI.
- No cache result is authority. A reused result is accepted only as the same
  evidence under a complete key; otherwise the required producer runs.
- The machine-global hook, user Git configuration, credentials, trust entries,
  PATH, and shell installation are external machine state and cannot be edited
  by a WO25 repository unit.

## Windows and Linux obligations

Every implementation unit must pass focused Windows evidence before review and
terminal required publication CI on `x86_64-pc-windows-msvc` and
`x86_64-unknown-linux-gnu` after separately authorized publication. The same
typed command, schema, field order, selector ledger, mutation dispositions, and
summary semantics apply on both.

Windows evidence covers native process creation, path/argument quoting, UTF-8,
CRLF input normalization without output drift, controlled Ctrl-C child-tree
cleanup, crash-residue discovery, and PowerShell 7 adapter isolation. Linux
evidence covers POSIX process exit and controlled signal cleanup, crash-residue
discovery, LF, executable resolution, and identical canonical summary bytes for
platform-neutral fields. Platform-specific fields must be explicit and cannot
be normalized away.

macOS, non-MSVC Windows, non-GNU Linux, cross-compilation, containers, and
remote developer hosts remain unexercised and unsupported by WO25.

## Per-unit implementation and review evidence

Every unit runs `cargo fmt --all -- --check`, workspace all-target checking,
workspace all-target warnings-denied Clippy, the exact applicable `hum-dev`
selectors, initialized mutations, diff/accounting, hygiene, public readiness,
alpha claims, and release readiness. These commands are implementation-time
requirements only; they are not authorized during this planning package.

- Unit A proves selectors 1-6, I01-I04, every shared SHA-256 vector, the exact
  commit-message corpus including invalid empty/CR/LF/NUL subjects and a valid
  multi-line body, candidate identity corruption, legacy adapter
  exit/stage/stream equivalence, controlled cleanup, and the
  under-1-second/under-5-second timing gates. Because it changes validation
  tooling, a fresh reviewer may consume one exact Fast allowance only after
  focused evidence is green.
- Unit B proves selectors 7-10 at exact ledger positions 125-128, the
  byte-identical frozen 124-selector prefix and complete 128-selector ledger,
  I05-I07, every summary corruption row, exact S01-S06 stage closure,
  Ubuntu/Windows agreement, immutable-region Work Order reconstruction, and
  sentinels that fail on any full-log download or Cargo/compiler/Fast stage in
  status mode. It also proves both immutable action pins; exact summary and
  executable artifact grammars; one-file/no-link/no-reparse/no-traversal shape;
  platform, name, hash, run, attempt, job, digest, and same-length executable
  substitutions; absent/expired/duplicate artifacts; pre-execution rejection;
  running-self-hash/summary disagreement; distinct bootstrap and summary
  cleanup ownership; Windows post-exit handle/process quiescence; Ubuntu
  regular-file and executable-bit handling; cache/local/log/build/PowerShell
  fallback rejection; and consecutive recognized status commits consuming one
  still-valid full anchor. Missing, stale, partial, malformed, duplicate,
  cross-boundary, mixed-binding, cleanup-failed, or unauthenticated artifacts
  fail before projection, with every individual predicate emitted. The complete
  151-case classifier suite runs twice deterministically without changing
  classifier trust. One reviewer-adjudicated Fast is allowed only after all
  focused evidence is green.
- Unit C proves selectors 11-13 and I08 through Windows PS5.1/PowerShell 7 and
  Ubuntu PowerShell 7 legacy-equivalence matrices. The new path itself supports
  only PowerShell 7. PATH, `PSModulePath`, quoting, missing-shell, invalid UTF-8,
  credential, dubious-ownership, and prelaunch-repair cases remain distinct.
  One reviewer-adjudicated Fast is allowed after the matrices pass.
- Unit D proves selectors 14-16 and I09-I11 over every impact case, every
  K01-K12 cache-key dimension, each controlled cleanup disposition, authenticated
  historical reconciliation, foreign-path preservation, and ambiguous-residue
  rejection. One exact Fast producer may run after focused green; a same-key
  consumer must reuse its authenticated result without launching a second Fast,
  while every changed key must demand a new authorization rather than running
  automatically.
- Unit E proves selectors 17-18 and I12, the complete final ownership map, no
  remaining semantic consumer of retired PowerShell authority, and exact
  old/new evidence on one reviewer-adjudicated Fast. Deletion volume earns no
  equivalence credit by itself.

An aggregate failure prints every individual failing predicate before its
terminal disposition. Review independently corrupts at least one commit/tree
identity, one platform/job identity, one selector or mutation identity, every
declared reuse-key dimension through K01-K12, and one controlled and one stale
cleanup path per applicable unit; a fixture that would stay green under its
named regression is defective.

## Review, commit, and publication gates

The exact implementation commit subjects are:

| Unit | Frozen implementation subject |
| --- | --- |
| A | `build(evidence): add portable evidence spine` |
| B | `ci(evidence): consume authenticated status summaries` |
| C | `build(tooling): make orchestration shell-portable` |
| D | `perf(evidence): add change-aware execution` |
| E | `refactor(evidence): retire migrated powershell authority` |

Each unit follows the full gate independently:

1. fresh explicit BDFL authorization for that unit only;
2. one implementer edits only its exact envelope and leaves a complete
   unstaged candidate with empty index;
3. fresh independent review of code, fixtures, corruption controls, paths,
   budgets, performance baseline/comparison, platform branches, and old/new
   equivalence;
4. only unqualified `ACCEPT` may recommend the frozen local commit;
5. BDFL separately authorizes that exact commit and later one normal,
   non-force push;
6. required Ubuntu and Windows jobs reach terminal-green on the exact SHA;
7. a separately authorized status record may use subject
   `docs(workorder): record unit <letter> publication`, with the literal
   lowercase unit letter substituted; and
8. the next unit remains unauthorized until its own fresh signal.

Implementation commits classify full. Status-only documentation commits may
classify fast only through the authenticated summary chain and recognized
mutable regions. Exhaustive runs only when the exact unit changes its parser,
matrix, selector, environment, output contract, or execution path, or when an
explicit milestone/release gate requires it. It is never rerun merely to review
documentation.

No unit may dispatch, rerun, cancel, or repair CI without fresh authorization.
A terminal-red required job stops the unit at publication without an automatic
repair, second push, or next-unit start.

## Stop rules

Stop without expansion on any unexpected path, category or row budget breach,
new dependency, schema ambiguity, lost selector/mutation, zero-match selector,
cross-platform disagreement, false cache hit, hidden predicate, leftover
temporary resource, historical-artifact touch, semantic weakening, summary
substitution, non-equivalent migration, unexpected stage, red Fast/CI, or
repository-state drift.

A disposable helper or preflight parse may be corrected while no actual Fast,
commit, push, destructive cleanup, or CI rerun has launched. Once one of those
stateful or expensive one-shot actions begins, its existing stop rule applies
and no retry is implied.

A performance miss never authorizes skipped correctness. Report the exact miss
and stop for architect/BDFL disposition. No stop grants an amendment, repair,
commit, push, rerun, cleanup, or later unit.

## Explicit exclusions

WO25 authorizes no:

- new Hum language syntax or semantics;
- another canonical program;
- arbitrary-program lowering;
- package management;
- standard library or Nectar implementation;
- cost-intelligence scoring implementation;
- macOS support;
- AOT or optimization work;
- another backend or LLVM;
- self-hosting;
- release, tag, installer, or version work;
- deletion of historical artifacts; or
- unrelated research-document publication.

It also authorizes no machine-global hook edit, credential/trust/PATH change,
stash/archive operation, historical target cleanup, workflow dispatch, CI
rerun, or external announcement.

## Queued later advisories

Four advisories remain queued and unimplemented:

1. agent-native language comparison and Zero research;
2. cost-intelligence and performance-rating framework;
3. assurance-status taxonomy for declared, statically checked, runtime
   checked, tested, measured, and externally proven claims; and
4. construct-generic compiler growth so future programs require less
   program-specific Rust.

Exactly advisories 2, 3, and 4 constrain future planning after the evidence
spine is independently accepted. Cost intelligence must consume the accepted
typed evidence instead of inventing a second score authority; public claim
planning must preserve the assurance-status distinctions; and future program
planning must prefer reusable construct-level compiler growth over a new
program-specific Rust seam. Advisory 1 remains research input only until a
separate evidence review and accepted decision gives it normative weight.
None of the four is implementation authority under WO25.

## Planning-package validation

Authorship and fresh independent pre-issuance review run only:

- `git diff --check`;
- raw and whitespace-insensitive diff accounting for the sole modified WO25
  path;
- exact changed-region/path authentication and reconstruction proving every
  byte outside the mutable Status/current-gate bodies and exact amended Unit B
  clauses remains identical to base blob
  `c592785c4187dff35d15830f02e2bc08b931f9da`;
- exact WO25 mode, non-writing Git blob identity, SHA-256, byte size,
  regular-file kind, UTF-8/no-BOM/LF/final-LF framing, exact-one marker at line
  4, and one-active/sixteen-closed/zero-root topology;
- internal row/category/overall arithmetic, complete 14-path uniqueness and
  order, category membership, non-borrowable raw/whitespace ceilings, and
  aggregate five-unit telemetry;
- exact selector/mutation/S01-S06 inventories, frozen 124 prefix and complete
  128 ledger, summary/executable artifact grammars, full/fast workflow step
  uniqueness and order, dependency pins, platform obligations, cleanup owners,
  performance boundary, exclusions, gates, and stop anchors;
- read-only official GitHub release/tag/commit provenance for the two frozen
  action SHAs;
- the repository-required 151-case classifier planning suite twice
  deterministically, including same-anchor consecutive status classification;
- text hygiene and public readiness for the candidate;
- alpha claims; and
- release readiness `0.0.1`.

Planning authorship/review runs no Cargo, Rust selector, compiler, interpreter,
native execution, Fast, full preflight, Exhaustive, CI, workflow dispatch,
artifact action, artifact upload/download, archive code, or stash operation.

## Current authorization gate

WO25 Unit A remains complete and closed. Unit B is implemented, independently
accepted, published, terminal-green, and closed to further implementation. The
sole next action is fresh independent architect-review of this exact Unit B
publication-status candidate. The author issues no verdict.

Only an unqualified `ACCEPT` may recommend, but does not execute, a separately
authorized local documentation commit with exact subject:

```text
docs(workorder): record unit b publication
```

Review acceptance alone authorizes no staging, status commit, push, CI, Fast,
Exhaustive, implementation, historical cleanup, hook edit, or later unit.
Unit C is next in sequence but remains unauthorized. Units D-E, successor or
language work, package/stdlib/Nectar work, optimization, another backend,
release/tag work, stashes, archives, and historical-artifact operations remain
unauthorized.

<!-- workorder-current-authorization-gate:end -->
