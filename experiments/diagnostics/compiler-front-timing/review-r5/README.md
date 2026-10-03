# Timing-observability candidate export (r5)

Review-transport bundle for the **uncommitted** R5 corrected production
timing-observability candidate (seven files). The candidate itself lives
only in its worktree; this directory is a byte-exact export for
independent review. The r2, r3, and r4 bundles alongside are preserved
unchanged.

- Pinned production base: `a01f4dc242cd075d7aff91090280ce9c38bcc8a7` (PR #69 head; the candidate's base)
- Carrier commit (r4 review bundle): `3daed1a005599e37f58cc5b3f2a9ffcb85a4ec6b`
- `files/` mirrors the repository layout and holds the seven candidate
  files with their exact candidate bytes.
- `candidate.diff` is the complete unified diff against the pinned base
  (new files shown as full-content additions; `git diff <base>` alone
  omits untracked files).
- `SHA256SUMS` lists the SHA-256 of each file under `files/` and of
  `candidate.diff`.
- `MANIFEST.txt` lists the pinned base, the seven git blob identities
  with byte counts, and the package SHA-256 (SHA-256 of `SHA256SUMS`).

## What R5 corrects (one bounded correction of R4, Codex accept-with-required-fixes)

1. Shared-caller completion contract: the 84 assertions passed, but the
   real `Invoke-RepoScript` caller then threw "timing ledger controls
   failed with exit code 5" — expected native failures (exit 3/5 probes)
   leaked `$LASTEXITCODE`. The suite now resets `$global:LASTEXITCODE = 0`
   on its success path (genuine failures already `exit 1` above, so the
   reset is unreachable on failure). The generic caller is untouched.
   New control 25 proves through the actual shared caller (extracted via
   AST from check_all.ps1, never re-implemented): the all-pass suite
   succeeds, a deliberate setup failure still fails, and a deliberate
   assertion failure still fails with exit code 1.
2. Default parent-journal preservation: control 9's unset-env probe
   initialized and deleted the real default journal
   (`target/hum-timing-ledger.ndjson`) that an incoming local profile can
   be using. The probe now captures the adapter's argument with a shadow
   (proving empty-path deferral to the module default) and asserts the
   real default file is neither created, deleted, nor modified; the real
   initializer is restored afterwards. Both incoming configurations are
   verified (configured `HUM_TIMING_JOURNAL`, unset env), the parent
   environment is left exactly as found, and the next command is proven
   to retain the restored session/group with balanced depth.

The accepted R4 scope-balance, cleanup, and Rust environment corrections
are preserved. No deadline increase, coverage removal, framework, N9, or
WO31 work.

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

## Retained correction-test results (Linux VM, 2026-10-02/03)

- `tools/test_timing_ledger.ps1`: 93 passed, 0 failed — unique START/END
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
  shared-caller completion contract (all-pass succeeds; deliberate setup
  and assertion failures still fail through the real Invoke-RepoScript).
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

Retained from the 2026-10-02/03 runs on the Linux VM (not rerun for this
bundle):

- `tools/test_timing_ledger.ps1`: 224 s for the 93/93 green run (control
  25 runs the suite twice more through the shared caller: one all-pass,
  one deliberate-assertion-failure).
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
