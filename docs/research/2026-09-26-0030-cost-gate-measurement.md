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
  cluster) is preserved, not trimmed. It is the visible trace of shared-VM
  contention; the alternating check/full-type-check design means a
  transient load spike lands on both commands rather than biasing one.
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
checked**. The 16.713 s median measures parse + resolve + report on a
blocked corpus — not the cost of checking 306 statements. Comparing it
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
identifies three duplicate-name errors in `examples/control_flow.hum`
and, in `examples/session_server.hum`, three duplicates plus one
immutable mutation target and one unresolved token. The retained Linux
artifacts enumerate only the count (`resolver_errors=8`); the individual
eight are Codex-reported, not independently re-verified here. They are
pre-existing source conditions in independently-authored example files —
they are **not** labeled intentional misuse fixtures; no source evidence
supports that label.

Exit status of the untimed run: not separately recorded in the retained
files. The five timed runs of the identical command all exited 1; the
untimed run's report shows the same blocked status and error count.

## On idle samples and contention

The retained script captures no load/idle data — there are no
before/after idle samples in the artifacts, and none are claimed. The
contention assessment rests on two facts: (1) the alternating-run
protocol, which interleaves check and full-type-check so transient load
cannot systematically bias one command; and (2) the preserved wordfreq
outlier (1.564 s), which shows the VM was not perfectly quiet. Those
support a contention *assessment*; they do not continuously prove
nothing else ran. No stronger claim is made.

## Why programs must be checked separately, not combined

One `hum` invocation builds one resolver scope: `load_program`
(`src/main.rs:3424`) folds every path argument into a single `Program`,
and `collect_inputs` (`:3552`) gathers all files under a directory
argument. Independently-authored example files share names — e.g.
`task add` is defined in both `examples/core/add.hum` and
`examples/core/minimal_add.hum` — so combining them into one invocation
manufactures duplicate-name resolver errors that then block *all* body
checking in that scope. That is exactly the `examples/` outcome. A
combined directory of independently-authored programs is therefore not a
valid body-checking corpus; per-program separate invocations are
required. (`hum full-type-check` accepts multiple `<file-or-dir>...`
arguments, but they still share the one scope.)

## Existing positive consumers

- `examples/tools/wordfreq.hum`: full-type-check exit 0 on all timed
  runs (Linux and Windows) — reaches body checking.
- `examples/probes/causal_failures.hum`: Session W runs
  `hum run examples/probes/causal_failures.hum` with `--entry same_root`
  / `outer_value` / `root_value` and asserts success output
  (`tools/check_all.ps1:4534`–`:4549`). `hum run`'s preflight runs
  resolve → type-check → full-type-check and refuses before execution on
  static failure, so these run successes are existing positive evidence
  that the file passes the resolver gate in its own scope.

## Proposed supplemental measurement (ONE, bounded)

To give the gate a second genuine body-checking data point alongside
wordfreq, run the existing protocol unchanged — 1 warm-up of each
command, then 5 timed runs alternating check / full-type-check, release
binary, uncontended VM — on exactly:

```
./target/release/hum check examples/probes/causal_failures.hum
./target/release/hum full-type-check examples/probes/causal_failures.hum
```

Rationale (coverage, not favorable timing):

- It is an existing input; no manufactured corpus.
- Existing positive evidence (Session W run successes) that it passes
  the resolver gate and reaches body checking in its own scope.
- It carries contract sections (`needs:`/`ensures:`) with causal-failure
  predicates — it exercises the predicate-analysis stage that a plain
  190-line program stresses least, which is precisely the stage whose
  cost Option B adds to `hum check`.
- Single file, single scope: no cross-file duplicate hazard. It must be
  run as its own invocation, not combined with other programs into one
  resolver scope (see above).

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

(a) accept the `causal_failures.hum` supplemental measurement above as
    the second leg (recommended — genuine body-checking cost, existing
    input, no corpus surgery);
(b) rule the gate on the wordfreq leg alone, striking the `examples/`
    leg as unsatisfiable in its current form;
(c) direct otherwise.

Until that ruling lands, Option B implementation stays gated and no
further benchmark campaign is authorized.
