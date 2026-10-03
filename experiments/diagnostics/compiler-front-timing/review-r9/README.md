# Timing Observability Candidate R9

Review-only correction implementing Codex's repeated-rejection
satisfiability audit.

## Baseline

- Correction baseline: 827eec28 (PR #69 published head)
- Worktree HEAD: 94990036 (same tree 942e06dc)
- R8 carrier: 5c1f5c67

## Changes from R8

### 1. Capture-native quiescence proof (ci.yml)

R8 reconstructed quiescence from `Get-Process -Id $Capture.Pid`. R9
consumes the current authenticated capture's existing proof fields:
`JobQuiescenceObserved`, `FinalActiveProcessCount -eq 0`,
`StdoutCompletionObserved`, `StderrCompletionObserved`,
`PrimaryExitObserved`. No new process model or schema. Missing/unproven
capture preserves the original failure and reports the retention gap.

### 2. Control 27: real production path

- Executes the actual Fast-tier guard (not a text check)
- Launches a real timed child via `Invoke-HumBinaryCapture`
- Uses the real capture's quiescence fields for the staging decision
- Observes DiagnosticDirectory for the staged journal
- Proves removal (no call -> empty dir) and misdirection (wrong dir ->
  upload dir empty) fail the observation

### 3. Policy scoping

Assertions scoped to the actual "Run Hum preflight" step block via regex.
Canonical closure (Unit B hash pin in check_all.ps1) preserved and updated.
Get-Process text-presence criterion removed.

## Verification (exact bundle bytes)

- tools/test_timing_ledger.ps1: 115 passed, 0 failed (~85s)
- tools/test_ci_policy.ps1: 445 assertions (~38s)
- YAML parses; public readiness 659 files; Linux only, no Full campaign
