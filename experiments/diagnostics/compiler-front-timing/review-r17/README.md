# Timing Observability Candidate R17

Review-only control-27 correction on the R16 carrier.
Seven candidate files frozen at R12–R16; only tools/test_timing_ledger.ps1 changes.

## Required correction (R16 verdict)

Non-enumerating before-snapshots: both default-journal before snapshots
(controls 9 and 27) used the if-expression form, which enumerates an
existing empty byte array into $null — conflating present-empty with
missing. Both now use Get-HumJournalByteSnapshot (direct assignment inside
the branch; unary comma on return), shared by both controls. The
order-sensitive helper is unchanged; missing-file/null stays distinct from
present-empty.

Added a bounded owned-fixture regression exercising the snapshot path used
by both controls: existing empty file snapshots as present-empty byte[0]
(not null), missing file as null, present-empty equals itself, and
present-empty stays distinct from missing/null.

The R16 digest helper, permutation oracle, R14 isolation correction, and
all success/timeout/finalizer controls are preserved unchanged.

## Evidence attribution

- test_timing_ledger-stdout-stderr.txt: R17 run on final bytes in isolated
  checkout (146 passed, 0 failed, true exit code 0) — builder-lane run.
- test_ci_policy-stdout-stderr.txt: retained R13 run
  (445 assertions, true exit code 0; policy-relevant files unchanged).
- Present and absent default-journal preservation: independently verified
  by Codex (per R15 verdict).

## What control 27 proves (146 assertions, 0 failures)

- Timeout invocation's own authenticated capture; quiescence from its fields
- Source-owned Read-NativeChannelsWithExit in both children
- Real failure finalizer executed with timeout data; orphan START staged
- Real success-finalizer branch executed with success data; journal cleaned
- Three corruption controls via the source-owned finalizer; null capture
  prevents staging
- Guard call observation via non-writing recorder; default journal untouched
  (order-sensitive, non-enumerating snapshots); initializer identity restored
- Oracles: the preservation comparison is order-sensitive; the snapshot
  path preserves present-empty distinct from missing

Review-only. Preserves R2-R16. Stops for independent review.
