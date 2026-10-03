# Timing Observability Candidate R12

Review-only control-27 correction on the R11 carrier. Seven candidate files
frozen at R11; only tools/test_timing_ledger.ps1 changes.

## What R12 corrects (review of R11)

R11 borrowed a separate success child's capture for the finalizer's quiescence
proof and hardcoded `$QProven = $true`. R12 uses the authenticated capture
returned by the actual TIMEOUT invocation, keeping its capture directory,
child-written journal, and actual early DiagnosticDirectory together as one
invocation. A throw alone proves nothing; only the capture's own fields do.

The timeout child now invokes the source-owned `Read-NativeChannelsWithExit`
(extracted via AST from check_all.ps1) instead of manually calling
`Start-HumTimedCommand`.

## What control 27 now proves (128 assertions, 0 failures)

- Timeout invocation returns an authenticated capture; its own fields prove
  quiescence (JobQuiescenceObserved, FinalActiveProcessCount=0, channel
  completions, PrimaryExitObserved)
- Genuine orphan START without END via the source-owned wrapper
- Real finalizer EXECUTED with the timeout invocation's data; journal staged
  to the invocation's DiagnosticDirectory; orphan START retained
- Success cleanup uses its own corresponding success invocation
- Three corruption controls exercise the SAME source-owned finalizer:
  invalidated proof, removed journal, misdirected destination — each fails
  the same staging observation
- Missing capture proof prevents staging (null capture through the finalizer)

## Regression (carried from R11)

Genuine `Set-HumDurableText` pinned; manifest LF framing and capture
authentication verified.

Review-only. Preserves R2-R11. Stops for independent review.
