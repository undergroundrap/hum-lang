# Timing Observability Candidate R14

Review-only control-27 correction on the R13 carrier (ACCEPT WITH REQUIRED FIX).
Seven candidate files frozen at R12/R13; only tools/test_timing_ledger.ps1 changes.

## Required corrections (R13 verdict)

1. Guard observation: the unset-path test now observes whether the actual
   initializer is CALLED through the extracted guard. A call-observing wrapper
   records invocation: explicit path must call the initializer; unset must not
   call it. The corruption removes the env condition from the extracted guard,
   which must then call the initializer even when unset (failing the same
   observation). The genuine initializer is restored in finally; parent and
   default journals are preserved.

2. Transport: the raw log bytes are now committed with a .txt suffix
   (the repo ignores *.log, so the .log names could not be committed).
   SHA256SUMS names the committed files; every entry is verified in the
   committed tree and via published readback.

## What control 27 proves (138 assertions, 0 failures)

- Timeout invocation's own authenticated capture; quiescence from its fields
- Source-owned Read-NativeChannelsWithExit in both children
- Real failure finalizer executed with timeout data; orphan START staged
- Real success-finalizer branch executed with success data; journal cleaned
- Three corruption controls via the source-owned finalizer; null capture
  prevents staging
- Guard call observation: explicit path calls initializer, unset does not,
  removed env condition calls when unset (observation fails as expected)

## Evidence

- test_timing_ledger-stdout-stderr.txt: R14 run on final bytes
  (138 passed, 0 failed, true exit code 0)
- test_ci_policy-stdout-stderr.txt: retained R13 run
  (445 assertions, true exit code 0; policy-relevant files unchanged)

Review-only. Preserves R2-R13. Stops for independent review.
