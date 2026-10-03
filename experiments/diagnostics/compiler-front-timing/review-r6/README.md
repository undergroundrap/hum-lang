# Timing-observability candidate export (r6)

Review-transport bundle for the **uncommitted** R6 corrected production
timing-observability candidate (seven files). The candidate itself lives
only in its worktree; this directory is a byte-exact export for
independent review. The r2, r3, r4, and r5 bundles alongside are preserved
unchanged.

- Pinned production base: `a01f4dc242cd075d7aff91090280ce9c38bcc8a7` (PR #69 head; the candidate's base)
- Carrier commit (r5 review bundle): `38f83a1afe598f9e99d0fb727af55126f9379818`
- `files/` mirrors the repository layout and holds the seven candidate
  files with their exact candidate bytes.
- `candidate.diff` is the complete unified diff against the pinned base
  (new files shown as full-content additions; `git diff <base>` alone
  omits untracked files).
- `SHA256SUMS` lists the SHA-256 of each file under `files/` and of
  `candidate.diff`.
- `MANIFEST.txt` lists the pinned base, the seven git blob identities
  with byte counts, and the package SHA-256 (SHA-256 of `SHA256SUMS`).

## What R6 corrects (one bounded followup to R5's control 25)

R5's control 25 proved the shared-caller completion contract by running
the full slow suite twice more (all-pass inner run, sabotaged
assertion-failure run), tripling control-25 cost. R6 keeps the exact
same three proofs but with bounded fixtures instead of full-suite
executions:

1. Success fixture: runs a real native `exit 5` (the actual leak
   pattern), records the leaked `$LASTEXITCODE` to its scratch dir
   (asserted `5`, so the test is not vacuous), then executes the suite's
   OWN completion epilogue extracted verbatim at runtime — the real
   `Invoke-RepoScript` must not throw.
2. Setup-failure fixture: a deliberate `throw`; the caller must propagate
   it.
3. Assertion-failure fixture: `Failed=1` through the suite's own epilogue
   (`exit 1`; the success-path reset is unreachable); the caller must
   throw "failed with exit code 1".

Each fixture owns a scratch directory under the suite workdir, cleaned up
after its evidence is captured; the fixture scripts themselves are
temporary files in the tools dir, removed in the same finally. The R5
`$LASTEXITCODE` hygiene fix, control 9 journal isolation, scope balance,
diagnostics, coverage, and deadlines are unchanged. No production
wrappers or Rust transport were reopened.

## Verification

From the repository root at the pinned base:

```
D=experiments/diagnostics/compiler-front-timing/review-r6
(cd $D && sha256sum -c SHA256SUMS)
for f in $(grep '  files/' $D/SHA256SUMS | cut -d' ' -f3- | sed 's|^files/||'); do
  git hash-object $D/files/$f   # compare against MANIFEST.txt
done
sha256sum $D/SHA256SUMS        # compare against package sha256 in MANIFEST.txt
```

Nothing in this directory executes in CI; no workflow, test, or
production file outside it is touched by this commit.

## Retained correction-test results (Linux VM, 2026-10-03)

- `tools/test_timing_ledger.ps1`: 94 passed, 0 failed — unique START/END
  id pairing, duplicate-label interruption regression, actual executable
  and argv, success/expected-failure/wrapper-error/timeout outcomes,
  absent/disabled/enabled module configurations, nested-launch
  suppression, byte-identical outputs, selector credit, profile-init and
  retention seams, boundary interruption with descendant/channel
  quiescence, NDJSON schema, Bytes missing-executable setup failure plus
  retry, contained root-creation failure, contained journal END-write
  failure isolation (sabotaged mid-flight in a child process),
  adapter-seam default-path isolation (shadow capture, real default
  journal untouched, session/group/depth preserved), and the
  shared-caller completion contract via bounded fixtures (real
  `$LASTEXITCODE=5` leak proven non-vacuous, success succeeds, deliberate
  setup and assertion failures still fail through the real
  Invoke-RepoScript).
- `tools/test_ci_policy.ps1`: 436 assertions passed, no Full execution
  credit.
- `tools/check_text_hygiene.ps1`: passed for 659 files.
- `tools/check_public_readiness.ps1`: passed for 659 files.
- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- `cargo test -p hum-dev`: 25 + 6 passed, 0 failed — including the three
  timing-journal tests (absent probe, present probe, sanitized-environment
  parent) in `shell::hum_timing_journal_tests`.
- Pinned bodies: compiler-front/corpus AST-identical to base;
  `src/ir_verify.rs` zero diff; contained-capture contract literals
  intact.

## Measured control-suite duration and intentional wait budgets

Retained from the 2026-10-03 runs on the Linux VM (not rerun for this
bundle):

- `tools/test_timing_ledger.ps1`: 82 s for the 94/94 green run (control
  25 uses three bounded fixtures, ~2 s each, instead of two extra
  full-suite executions).
- Intentional waits inside the suite: control 12's 45 s
  `Invoke-HumBinaryCapture` boundary deadline (the production 3000 s
  deadline is exercised through the same machinery, not lengthened);
  control 24's 10 s fake-cargo sleep with a 120 s child-wait cap and a
  200 ms x up-to-150 sabotage poll; control 20's 2 s timeout probe.
- `tools/test_ci_policy.ps1`: ~38 s.
- Hygiene + public readiness: ~9 s combined.
- `cargo test -p hum-dev`: ~61 s total.
- `cargo fmt` / `cargo clippy`: seconds.

## Actual platform coverage

- All of the above ran on the Linux VM only.
- Windows process-launch, Job Object termination, timeout behavior, and
  artifact upload were not exercisable locally; CI remains the final
  Windows authority.
- The 3000 s production outer deadline was not exercised (controls use a
  45 s boundary deadline through the same `Invoke-HumBinaryCapture`
  machinery).
- GitHub artifact upload behavior was not exercised locally.
- Do not infer Windows CI residual time from Linux or warm local
  measurements.
