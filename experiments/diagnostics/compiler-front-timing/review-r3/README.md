# Timing-observability candidate export (r3)

Review-transport bundle for the **uncommitted** corrected production
timing-observability candidate (seven files). The candidate itself lives
only in its worktree; this directory is a byte-exact export for
independent review. The r2 bundle alongside is preserved unchanged.

- Pinned base: `a01f4dc242cd075d7aff91090280ce9c38bcc8a7` (PR #69 head; the candidate's base)
- `files/` mirrors the repository layout and holds the seven candidate
  files with their exact candidate bytes.
- `candidate.diff` is the complete unified diff against the pinned base
  (new files shown as full-content additions).
- `SHA256SUMS` lists the SHA-256 of each file under `files/` and of
  `candidate.diff`.
- `MANIFEST.txt` lists the pinned base, the seven git blob identities
  with byte counts, and the package SHA-256 (SHA-256 of `SHA256SUMS`).

## Verification

From the repository root at the pinned base:

```
D=experiments/diagnostics/compiler-front-timing/review-r3
(cd $D && sha256sum -c SHA256SUMS)
for f in $(grep '  files/' $D/SHA256SUMS | cut -d' ' -f3- | sed 's|^files/||'); do
  git hash-object $D/files/$f   # compare against MANIFEST.txt
done
sha256sum $D/SHA256SUMS        # compare against package sha256 in MANIFEST.txt
```

Nothing in this directory executes in CI; no workflow, test, or
production file outside it is touched by this commit.

## Existing results (Linux VM)

- `tools/test_timing_ledger.ps1`: 64 passed, 0 failed — unique
  START/END id pairing, duplicate-label interruption regression, actual
  executable and argv, success/expected-failure/wrapper-error/timeout
  outcomes, absent/disabled/enabled module configurations,
  nested-launch suppression, byte-identical outputs, selector credit,
  profile-init and retention seams, boundary interruption with
  descendant/channel quiescence, NDJSON schema.
- `tools/test_ci_policy.ps1`: 436 assertions passed, no Full execution
  credit.
- `tools/check_text_hygiene.ps1`: passed for 659 files.
- `tools/check_public_readiness.ps1`: passed for 659 files.
- `cargo fmt --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- hum-dev shell tests: 6/6, including the new HUM_TIMING_JOURNAL
  transport test (allowlist acceptance, unrelated-key rejection,
  child-visible value through env_clear).
- Pinned bodies: compiler-front/corpus AST-identical to base;
  `src/ir_verify.rs` zero diff; contained-capture contract literals
  intact.

## Platform limits

- Windows process-launch and timeout behavior could not be exercised
  on the Linux VM; CI remains the final Windows authority.
- The 3000 s production outer deadline was not exercised (controls use
  a 45 s boundary deadline through the same Invoke-HumBinaryCapture
  machinery).
- GitHub artifact upload behavior was not exercised locally.
- Do not infer Windows CI residual time from Linux or warm local
  measurements.
