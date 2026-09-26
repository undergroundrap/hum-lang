# Decision 0030 cost-gate measurement — release-build evidence (2026-09-26)

Status: **evidence recorded; gate NOT cleared.** This snapshot preserves the
Builder's retained measurements with full provenance, separates
successful-check evidence from blocked-corpus evidence, and proposes one
bounded supplemental measurement. It authorizes nothing and implements
nothing.

## What the gate requires

Decision 0030 §10: implementation of Option B does not start until a clean
measurement exists — uncontended release build, `hum check` vs
`hum full-type-check` on `examples/tools/wordfreq.hum` and on `examples/`.
At ≥2× the current `hum check` cost, fall back to Option A now and revisit
B after incremental checking lands.

## Provenance

- Binary: `./target/release/hum`, 12039808 bytes, built from main at
  `500e6ea` ("WO30 Item 3"). Tree was clean apart from the untracked
  measurement artifacts themselves (`costgate_measure.sh`,
  `costgate_timing.log`, later `costgate_out.txt` and the two
  `full_type_check_examples_*` dumps) — no source modifications.
- Script: `costgate_measure.sh` (retained at
  `~/workspace/hum-0030-costgate/costgate_measure.sh`). Wall-clock via
  `date +%s.%N` (no GNU time / hyperfine on the VM). Protocol per target:
  1 warm-up run of each command, then 5 timed runs alternating
  check / full-type-check. Stdout/stderr discarded during timing; the
  hum exit code captured per run.
- Raw logs retained: `costgate_timing.log` (all 20 timed runs),
  `costgate_out.txt` (full script transcript including binary/tree
  provenance line), `full_type_check_examples_stdout.txt` (97049 bytes)
  and `full_type_check_examples_stderr.txt` (13162 bytes) from the one
  untimed `hum full-type-check examples/` clarification run.
- Medians and ratios below were recomputed from the raw log by the
  Researcher and match the reported values exactly.

## Results — successful-check evidence

All 20 timed runs preserved, including the outlier.

| target | cmd | runs (s) | median (s) | hum exit |
|---|---|---|---|---|
| `examples/tools/wordfreq.hum` | check | 1.041, 1.022, **1.564**, 1.014, 1.044 | 1.041 | 0 ×5 |
| `examples/tools/wordfreq.hum` | full-type-check | 1.708, 1.645, 1.782, 1.752, 1.773 | 1.752 | 0 ×5 |
| `examples/` | check | 13.539, 12.824, 13.771, 13.676, 13.222 | 13.539 | 0 ×5 |
| `examples/` | full-type-check | 17.128, 16.112, 17.196, 16.713, 16.669 | 16.713 | 1 ×5 |

- wordfreq ratio (full-type-check / check): **1.6830** — below the 2×
  fallback threshold. Both commands exit 0 on all runs: this is genuine
  body-checking cost on a real 190-line program.
- The wordfreq check outlier (run 3, 1.564 s against a 1.014–1.044 s
  cluster) is preserved, not trimmed. Its cause is unknown. The
  alternating check/full-type-check design reduces ordering bias; it
  does not guarantee equal exposure to transient load.
- Windows corroboration (reported by Codex, existing binary): check
  1.2707471 s, full-type-check 2.0490012 s, ratio **1.6124**, both exit 0.
  Same shape as Linux: comfortably below 2×.

## Results — blocked-corpus evidence (NOT body-checking cost)

The `examples/` full-type-check runs are rejection-path timings. The
untimed clarification run reports:

```
status: blocked_by_resolver_errors
summary: files=33 items=137 body_items=97 statements=306 checked_statements=0
  accepted_statements=0 rejected_statements=0 unchecked_statements=0
  unsupported_statements=306 blocking_issues=314 source_errors=0
  resolver_errors=8 type_errors=0 core_verify_errors=0 execution_ready=0 ir_ready=0
```

Every item in the report carries `[blocked_by_prior_errors]` /
`not_checked_blocked_by_prior_errors_v0`; **zero of 306 statements were
checked**. The 16.713 s median is a blocked-path timing with zero
statement checking — not the cost of checking 306 statements.
`build_report_with` (`src/full_type_check.rs:492`) computes the
type-check summary, callable analysis, task return types, task
signatures, typed-failure analysis, field-place types, predicate
analysis, and the core-verify handoff *before* the blocked test
(`:524`), so the timing covers those analyses plus the blocked
collection/report. Comparing it
against `hum check`'s 13.539 s (exit 0, its own stages run to completion)
as if both measured the same work would be a category error. Both
platforms show the same blocked shape (Windows: resolver_errors=8,
checked_statements=0).

Stage gating, from source (`src/full_type_check.rs`): line 524 sets
`blocked` when `type_check_summary.resolver_errors > 0` (among other
error classes), which propagates the per-item blocked markers; line
2585–2586 renders the `blocked_by_resolver_errors` status. This is the
intended fail-closed behavior, not a measurement artifact.

The eight resolver errors: Codex's existing-binary resolve query
locates them within `examples/control_flow.hum` (three duplicate-name
errors) and `examples/session_server.hum` (three duplicates, one
immutable mutation target, one unresolved token). The retained Linux
artifacts enumerate only the count (`resolver_errors=8`); the individual
eight are Codex-reported, not independently re-verified here. They are
pre-existing source conditions in the example files — they are **not**
labeled intentional misuse fixtures; no source evidence supports that
label.

Correction to the earlier revision of this snapshot: `src/resolve.rs`
(`:976–991`) creates a separate file scope per file, so one `Program`
is not one lexical scope. The two `task add` definitions (in
`examples/core/add.hum` and `examples/core/minimal_add.hum`) live in
separate file scopes and are not evidence of cross-file duplicate
errors. The aggregate resolver-error count gates the whole report,
blocking body checking report-wide even though the errors sit in two
of the 33 files.

Exit status of the untimed run: not separately recorded in the retained
files. The five timed runs of the identical command all exited 1; the
untimed run's report shows the same blocked status and error count.

## On idle samples and contention

The retained artifacts contain no load or idle observations — there are
no before/after idle samples to describe. Whether the runs were quiet is
Builder-reported (via Codex), not retained evidence; this snapshot
treats it as reported, not observed. The alternating check /
full-type-check protocol reduces ordering bias but does not guarantee
equal exposure to transient load. The wordfreq check outlier (run 3,
1.564 s) is preserved with its cause stated as unknown — it is not
discarded merely for being slower, and no monitoring requirement is
invented around it.

## Why programs must be checked separately, not combined

One `hum` invocation builds one `Program` from all path arguments
(`load_program`, `src/main.rs:3424`; `collect_inputs`, `:3552`), and
the full-type-check block test applies to the whole report
(`src/full_type_check.rs:524`): any file's resolver errors mark *every*
item `[blocked_by_prior_errors]`, so one blocked file poisons body
checking for all files in the invocation. That is the `examples/`
outcome — errors in 2 of 33 files yield checked_statements=0
report-wide. Per-program separate invocations are therefore required
for genuine per-program body-checking measurements, even though file
scopes are separate within one invocation.

## Existing positive consumers

- `examples/tools/wordfreq.hum`: full-type-check exit 0 on all timed
  runs (Linux and Windows) — reaches body checking.
- `examples/probes/word_count.hum`: exercised by
  `Invoke-HumLanguageProgramChecks` (`tools/check_all.ps1:1862`–`:1877`)
  — `resolve --format json` (asserted valid JSON), `test-skeletons`
  (exit 0 asserted), and `hum run --entry count_hum_literal`
  (exit 0, empty stderr, stdout `2` asserted) — plus run-corpus entries
  at `:3984` and `:4135` asserting output `2`. `hum run`'s preflight
  runs resolve → type-check → full-type-check and refuses before
  execution on static failure, so these run successes are existing
  positive evidence that the file passes the resolver gate in its own
  scope. Its `ensures:` clauses use `list_count`
  (`examples/probes/word_count.hum:8`, `:32`).

## Proposed supplemental measurement (ONE, bounded)

This is a proposal, not permission to measure: no measurement has been
run and none is authorized by this snapshot.

To give the gate a second genuine body-checking data point alongside
wordfreq, the proposal runs the existing protocol unchanged — 1 warm-up
of each command, then 5 timed runs alternating check / full-type-check,
release binary, uncontended VM — on exactly:

```
./target/release/hum check examples/probes/word_count.hum
./target/release/hum full-type-check examples/probes/word_count.hum
```

Rationale (coverage, not favorable timing):

- It is an existing input; no manufactured corpus.
- Existing positive evidence (the check_all `hum run` successes listed
  above) that it passes the resolver gate and reaches body checking in
  its own scope.
- Its actual `ensures:` clauses use `list_count` — it exercises the
  predicate-analysis stage that a plain 190-line program stresses
  least, which is precisely the stage whose cost Option B adds to
  `hum check`. (Correction: the earlier revision wrongly claimed
  `causal_failures.hum` carries `needs:`/`ensures:` contracts; it has
  none — it contains typed-failure propagation.)
- Single file: it must be run as its own invocation, not combined with
  other programs into one report (see above).

Preserve all samples including outliers, exactly as the existing 20
runs were preserved. No example repairs and no checker weakening to
manufacture a passing corpus — if the file does not pass the gate when
measured, that is evidence, reported as-is.

## BDFL clarification required — gate NOT cleared

The gate as written names `examples/` as a leg, but `examples/` cannot
satisfy that leg as a body-checking measurement: its full-type-check
run is blocked before statement checking on both platforms. This is not
a failure of the measurement — it is a property of the corpus. The
snapshot does not redefine the gate and does not call it cleared.

The wordfreq leg is clean (1.6830 Linux, 1.6124 Windows — both below
2×). For the corpus leg, the BDFL should rule one of:

(a) accept the `word_count.hum` supplemental measurement above as
    the second leg (recommended — genuine body-checking cost, existing
    input, no corpus surgery);
(b) rule the gate on the wordfreq leg alone, striking the `examples/`
    leg as unsatisfiable in its current form;
(c) direct otherwise.

Replacing `examples/` with another individual program requires BDFL
approval and loses corpus-scale coverage: one program cannot stand in
for a 33-file corpus, whatever its ratio shows.

Until that ruling lands, Option B implementation stays gated and no
further benchmark campaign is authorized.
