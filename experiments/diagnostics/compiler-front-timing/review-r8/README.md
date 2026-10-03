# Timing Observability Candidate R8

Review-only consolidated correction to R7 (review/timing-observability-r2).

## Correction baseline vs worktree HEAD

- Correction baseline: 827eec28 (PR #69 published head)
- Worktree HEAD: 94990036 (local R6 commit; same tree 942e06dc as baseline)
- The candidate.diff is computed against 827eec28 (via FETCH_HEAD).

## What changed from R7

### 1. Quiescence proof required (ci.yml)

Full's failure finally previously assumed quiescence. Now it requires
proof: the current capture must have a child Pid AND `Get-Process -Id`
must return null (process gone). A timeout flag alone is not proof;
quiescence does not require Full success. If proof is unavailable (e.g.,
capture authentication failed, no Pid), a warning reports the gap and the
journal is NOT staged — the original failure is preserved, not masked.

### 2. Control 27 rewritten (test_timing_ledger.ps1)

Tests the actual Full path, not just helpers:
- Extracts the real Fast-tier guard from check_all.ps1 (not retyped)
- Uses real Start/Stop-HumTimedCommand (no fabricated START records)
- Verifies the workflow text contains quiescence checks
- Covers unavailable quiescence (null capture -> no staging, journal preserved)
- Proves wiring removal (stripped workflow lacks retention) and
  misdirection (must stage to DiagnosticDirectory, not CaptureDirectory) fail

### 3. Missing test_ci_policy.ps1 included

R7 omitted tools/test_ci_policy.ps1 from the payload. R8 includes it with
7 assertions covering the Full journal wiring.

## Verification (from exact bundle bytes)

- tools/test_timing_ledger.ps1: 122 passed, 0 failed (~87s)
- tools/test_ci_policy.ps1: 443 assertions (~39s)
- YAML parses; public readiness 659 files

## Files

- `files/`: 8 exact candidate files (byte-identical to worktree)
- `candidate.diff`: complete diff against 827eec28
- `MANIFEST.txt`: blob SHA-1 identities and byte counts
- `SHA256SUMS`: SHA-256 of all payloads
- `results.txt`: test counts from exact bytes
