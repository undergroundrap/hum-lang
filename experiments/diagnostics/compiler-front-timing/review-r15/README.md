# Timing Observability Candidate R15

Review-only control-27 correction on the R14 carrier (ACCEPT WITH REQUIRED FIX).
Seven candidate files frozen at R12/R13/R14; only tools/test_timing_ledger.ps1 changes.

## Required correction (R14 verdict)

Default-journal preservation: R14's call observer forwarded the unset
corruption probe into the genuine initializer, overwriting
target/hum-timing-ledger.ndjson; restoring the function and environment
does not restore file contents.

R15 keeps genuine initialization for the fixture-owned explicit path.
Unset and condition-removed probes use a narrowly scoped, non-writing
ledger-initializer argument recorder through the real guard and genuine
profile adapter (the control-9 seam). Call sensitivity is preserved:
the recorder captures invocation without writing. Replacements are
restored in finally and their identities verified via AST comparison.
Existing-default bytes and absent-default existence are proven unchanged
using owned disposable fixtures; the real default journal is never
written.

## What control 27 proves (140 assertions, 0 failures)

- Timeout invocation's own authenticated capture; quiescence from its fields
- Source-owned Read-NativeChannelsWithExit in both children
- Real failure finalizer executed with timeout data; orphan START staged
- Real success-finalizer branch executed with success data; journal cleaned
- Three corruption controls via the source-owned finalizer; null capture
  prevents staging
- Guard call observation: explicit path initializes genuinely; unset does
  not call; removed env condition calls (observation fails as expected)
  via non-writing recorder; default journal untouched; initializer
  identity restored

## Evidence

- test_timing_ledger-stdout-stderr.txt: R15 run on final bytes in isolated
  checkout (140 passed, 0 failed, true exit code 0)
- test_ci_policy-stdout-stderr.txt: retained R13 run
  (445 assertions, true exit code 0; policy-relevant files unchanged)

Review-only. Preserves R2-R14. Stops for independent review.
