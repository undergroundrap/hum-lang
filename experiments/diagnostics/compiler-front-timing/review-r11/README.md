# Timing Observability Candidate R11

Review-only harness correction. Seven candidate files frozen at R10;
only tools/test_timing_ledger.ps1 changes.

## What R11 fixes (Codex-reproduced root cause)

R10's test_timing_ledger.ps1 defined a test-only `Set-HumDurableText` override
that omitted the final LF the genuine production writer appends. The override
shadowed the writer for every capture manifest, so `Assert-HumCaptureComplete`
threw "final LF required" and no valid `$Capture` object could be obtained.

R11 removes the override (not repaired, not worked around). The genuine
production writer from dot-sourced run_fast_evidence.ps1 stays active.
Byte proof retained: genuine emits `61 62 63 0a`; the override emitted
`61 62 63`.

## Mechanical extraction (no handwritten copies)

Control 27 extracts production source at runtime via AST:
- Fast-tier guard: `IfStatementAst` from check_all.ps1
- Save-HumPreflightDiagnostics: `FunctionDefinitionAst` from ci.yml
- Invoke-HumPreflightCapture: `FunctionDefinitionAst` from ci.yml
- Journal finalizer: the `$Quiescent = $false` block (second `} else {`,
  6-space YAML-stripped indent, extended to its closing brace)

## What control 27 now proves (126 assertions, 0 failures)

- Extracted guard initializes on explicit path; skips on unset
- Real Invoke-HumPreflightCapture CALLED (timeout child: threw or returned
  timed-out capture); early diagnostics via extracted producer
- Genuine START without END in the timeout journal (no fabrication)
- Extracted finalizer EXECUTED with real capture data; journal staged to
  DiagnosticDirectory; early snapshot preserved; orphan START retained
- Poisoned capture owner, removed retention, misdirected destination all fail
  the SAME DiagnosticDirectory observation
- Null capture proves nothing (unavailable-proof)

## Regression (R11)

- Pins the genuine writer's definition before capture; asserts unchanged after
  (no replacement writer leaked into scope)
- Asserts the produced manifest ends with LF (framing survived)
- Asserts Assert-HumCaptureComplete passes on the real capture (authentication)

## R10 claims withdrawn

See WITHDRAWAL-R10.md. R10's harness/apply claims are withdrawn; its report is
preserved unmodified.

Review-only. Preserves R2-R10. Stops for independent review.
