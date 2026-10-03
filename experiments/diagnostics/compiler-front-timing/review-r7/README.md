# Timing Observability Candidate R7

Review-only correction to the R6 candidate (review/timing-observability-r2).

## What changed from R6

Two defects from the R6 validation diagnosis (run 37101821311):

### 1. Multi-pwsh scalar resolution (tools/test_timing_ledger.ps1)

`Get-Command pwsh` can return multiple applications (three on GitHub
Ubuntu runners). The R6 code read `.Source` directly, which member-
enumerates to an array on multi-result — the array was then passed as a
single executable argument.

Fixed by importing the real `Select-FirstApplicationSource` from
check_all.ps1 (before first use) and using it at both sites (:72, :718).
Control 26 adds nondegenerate one/multiple-application coverage through
the real selector and a real executable consumer (fake pwsh fixtures with
distinct exit codes prove the first is selected and launched).

### 2. Full timing journal wiring

- `.github/workflows/ci.yml`: `full_preflight` now sets an invocation-
  owned `HUM_TIMING_JOURNAL` and retains it on failure into the
  DiagnosticDirectory (the actual upload directory — Full uploads
  DiagnosticDirectory, not CaptureDirectory). Handles an early snapshot
  (directory already exists); preserves the original failure if retention
  fails; removes the journal on success.
- `tools/check_all.ps1`: Fast-tier initializes the journal only when an
  explicit path is supplied (`$env:HUM_TIMING_JOURNAL` non-empty);
  unset preserves local no-journal behavior.
- `tools/run_fast_evidence.ps1`: `Invoke-HumTimingJournalRetention` gains
  an optional `-DestinationDirectory` (narrow, backward-compatible).

## Verification

- tools/test_timing_ledger.ps1: 119 passed, 0 failed (was 94)
- tools/test_ci_policy.ps1: 441 assertions (was 436)
- YAML parses; public readiness 659 files; no Full campaign run locally.

## Files

- `files/`: the 7 exact candidate files (byte-identical to worktree)
- `candidate.diff`: complete diff against pinned base a01f4dc2
- `MANIFEST.txt`: blob SHA-1 identities and byte counts
- `SHA256SUMS`: SHA-256 of all payloads
- `results.txt`: test counts
