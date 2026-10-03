# Timing-observability candidate export (r4)

Review-transport bundle for the **uncommitted** R4 corrected production
timing-observability candidate (seven files). The candidate itself lives
only in its worktree; this directory is a byte-exact export for
independent review. The r2 and r3 bundles alongside are preserved
unchanged.

- Pinned production base: `a01f4dc242cd075d7aff91090280ce9c38bcc8a7` (PR #69 head; the candidate's base)
- Carrier commit (r3 review bundle): `c97e4f36ea4c4ab71d8e8d1da46bd78074cb8548`
- `files/` mirrors the repository layout and holds the seven candidate
  files with their exact candidate bytes.
- `candidate.diff` is the complete unified diff against the pinned base
  (new files shown as full-content additions; `git diff <base>` alone
  omits untracked files).
- `SHA256SUMS` lists the SHA-256 of each file under `files/` and of
  `candidate.diff`.
- `MANIFEST.txt` lists the pinned base, the seven git blob identities
  with byte counts, and the package SHA-256 (SHA-256 of `SHA256SUMS`).

## What R4 corrects (one consolidated bounded correction of R3)

1. `Read-NativeBytesWithExit` scope: every post-START setup/launch step
   (start-info construction, process creation, `Start()`, stdin close,
   buffer allocation) now runs inside the scope-balancing try; a setup
   failure records an error END with the original message instead of
   stranding the scope, and the outer finally still disposes whatever was
   created (null-guarded). Missing-executable to successful-retry coverage
   through the real wrapper.
2. Contained wrapper (`Invoke-HumContainedRustNativeCapture`): root
   creation moved inside the try (it stranded the scope before it); the
   journal END write in the finally is isolated so an END-write failure
   cannot skip `Remove-HumCaptureAfterAuthentication`. No depth-counter
   reset. Root-creation-failure and END-write-failure coverage added.
3. `test_timing_ledger.ps1` runs once through the shared
   `Invoke-RepoScript` path (hygiene group) used by fixed profiles and
   Full; control 9 saves/restores the incoming `HUM_TIMING_JOURNAL`
   (never removes what it did not set) and leaves the process environment
   exactly as found; control 12 restored same-label/same-phase
   completed-versus-interrupted coverage, matched by unique id.
4. Rust test: the ambient-absence assumption is gone. Absent/present
   inputs go through the real `from_process` in child processes
   (`current_exe --exact --nocapture`) with controlled environments;
   no global environment mutation, no parallel-test races. Unrelated-key
   rejection, the pwsh transport echo, and the `env_clear` source contract
   are unchanged.

Workload bodies, coverage, ordering, deadlines, capture authentication
and existing behavior are otherwise preserved.

## Verification

From the repository root at the pinned base:

```
D=experiments/diagnostics/compiler-front-timing/review-r4
(cd $D && sha256sum -c SHA256SUMS)
for f in $(grep '  files/' $D/SHA256SUMS | cut -d' ' -f3- | sed 's|^files/||'); do
  git hash-object $D/files/$f   # compare against MANIFEST.txt
done
sha256sum $D/SHA256SUMS        # compare against package sha256 in MANIFEST.txt
```

Nothing in this directory executes in CI; no workflow, test, or
production file outside it is touched by this commit.

## Retained correction-test results (Linux VM, 2026-10-02)

- `tools/test_timing_ledger.ps1`: 84 passed, 0 failed — unique START/END
  id pairing, duplicate-label interruption regression, actual executable
  and argv, success/expected-failure/wrapper-error/timeout outcomes,
  absent/disabled/enabled module configurations, nested-launch
  suppression, byte-identical outputs, selector credit, profile-init and
  retention seams, boundary interruption with descendant/channel
  quiescence, NDJSON schema, Bytes missing-executable setup failure plus
  retry, contained root-creation failure, contained journal END-write
  failure isolation (sabotaged mid-flight in a child process).
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

Retained from the 2026-10-02 runs on the Linux VM (not rerun for this
bundle):

- `tools/test_timing_ledger.ps1`: 66-80 s across runs; the final 84/84
  green run took ~80 s.
- Intentional waits inside the suite: control 12's 45 s
  `Invoke-HumBinaryCapture` boundary deadline (the production 3000 s
  deadline is exercised through the same machinery, not lengthened);
  control 24's 10 s fake-cargo sleep with a 120 s child-wait cap and a
  200 ms x up-to-150 sabotage poll; control 20's 2 s timeout probe.
- `tools/test_ci_policy.ps1`: ~33-35 s.
- Hygiene + public readiness: ~8 s combined.
- `cargo test -p hum-dev`: ~52-55 s total.
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
