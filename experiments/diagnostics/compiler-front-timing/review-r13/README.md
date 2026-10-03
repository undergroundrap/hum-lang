# Timing Observability Candidate R13

Review-only control-27 correction on the R12 carrier (ACCEPT WITH REQUIRED FIX).
Seven candidate files frozen at R12; only tools/test_timing_ledger.ps1 changes.

## Required corrections (R12 verdict)

1. Genuine success evidence: the success child now uses whitespace-free
   arguments (base64-encoded inner command) with source-owned
   Read-NativeChannelsWithExit. Requires authenticated exit 0, no timeout,
   expected stdout/stderr, and matching START/END with success outcome.
   Executes the source-owned Full success-finalizer branch (extracted via AST
   from ci.yml) with the success invocation's own data.

2. Retention-call corruption: keeps a valid journal and disables the extracted
   Invoke-HumTimingJournalRetention CALL within the source-owned finalizer flow.
   R12 removed the journal instead, proving a different condition.

3. Packaging: SHA256SUMS now includes the real retained raw log bytes;
   every manifest entry verified.

## What control 27 proves (136 assertions, 0 failures)

- Timeout invocation's own authenticated capture; quiescence from its fields
- Source-owned Read-NativeChannelsWithExit in both children
- Real failure finalizer executed with timeout data; orphan START staged
- Real success-finalizer branch executed with success data; journal cleaned
- Three corruption controls via the source-owned finalizer; null capture
  prevents staging

Review-only. Preserves R2-R12. Stops for independent review.
