# Timing Observability Candidate R16

Review-only control-27 correction on the R15 carrier (ACCEPT WITH REQUIRED FIX).
Seven candidate files frozen at R12/R13/R14/R15; only tools/test_timing_ledger.ps1 changes.

## Required correction (R15 verdict)

Order-sensitive byte preservation: both default-journal byte checks
(controls 9 and 27) used Compare-Object, which is order-insensitive on
byte arrays — a same-length permutation would wrongly pass. Replaced with
a SHA-256 order-sensitive comparison (Test-HumBytesOrderEqual). Added a
bounded oracle control proving identical bytes pass and a same-length
permutation fails.

The R14 isolation correction (non-writing recorder, function restoration)
is preserved unchanged, as are all success/timeout/finalizer controls.

## Evidence attribution

- test_timing_ledger-stdout-stderr.txt: R16 run on final bytes in isolated
  checkout (142 passed, 0 failed, true exit code 0) — builder-lane run.
- test_ci_policy-stdout-stderr.txt: retained R13 run
  (445 assertions, true exit code 0; policy-relevant files unchanged).
- Present and absent default-journal preservation: independently verified
  by Codex (per R15 verdict); the absent case is additionally covered by
  the existence assertions in this run.

## What control 27 proves (142 assertions, 0 failures)

- Timeout invocation's own authenticated capture; quiescence from its fields
- Source-owned Read-NativeChannelsWithExit in both children
- Real failure finalizer executed with timeout data; orphan START staged
- Real success-finalizer branch executed with success data; journal cleaned
- Three corruption controls via the source-owned finalizer; null capture
  prevents staging
- Guard call observation via non-writing recorder; default journal untouched
  (order-sensitive); initializer identity restored
- Oracle: the preservation comparison itself is proven order-sensitive

Review-only. Preserves R2-R15. Stops for independent review.
