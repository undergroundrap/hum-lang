# Timing Observability Candidate R10

Review-only harness/package correction. Seven candidate files frozen
from R9; only tools/test_timing_ledger.ps1 changes.

## Mechanical extraction

Control 27 extracts production source at runtime via AST (no handwritten
copies):
- Fast-tier guard: `IfStatementAst` from check_all.ps1 matching
  HumTimingEnabled + HUM_TIMING_JOURNAL + Initialize-HumProfileTimingJournal
- Save-HumPreflightDiagnostics: `FunctionDefinitionAst` from the ci.yml
  "Run Hum preflight" step body (unindented from YAML)
- Invoke-HumPreflightCapture: same, via AST
- Journal finalizer: the `$Quiescent = $false` block via regex

If extraction fails, the control reports the precise obstacle (which
pattern missed) rather than substituting helper-level evidence.

## Genuine timeout

The child script initializes the ledger, starts a timed command via
`Start-HumTimedCommand`, then sleeps 60s. The 15s capture deadline kills
the tree. The flushed journal retains a genuine START without END
(no `Stop-HumTimedCommand` ever runs). The capture's existing fields
(`JobQuiescenceObserved`, `FinalActiveProcessCount`, etc.) prove
authenticated quiescence.

## Observations (not regex)

- Extracted `Save-HumPreflightDiagnostics` creates real early diagnostics
  (diagnostic_status.txt); a preexisting marker file survives staging
- The staged journal in DiagnosticDirectory contains the orphan START
- Removing the retention call leaves DiagnosticDirectory empty (same
  observation fails)
- Misdirecting to another directory leaves the observed DiagnosticDirectory
  empty (same observation fails)

## Diff provenance

candidate.diff is `git diff FETCH_HEAD` where FETCH_HEAD is 827eec28
(the correction baseline), not a01f4dc2. Verified by:
1. `git apply --check` against an owned clean `git archive` of 827eec28
2. Applying the diff reconstructs all eight blob SHA-1 identities
