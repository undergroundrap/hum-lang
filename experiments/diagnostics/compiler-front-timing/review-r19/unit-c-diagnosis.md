# Read-only runtime reconciliation — PR #69 run 37177745673 Windows 3000s timeout

## Observations (from retained evidence)

1. **887 balanced command pairs, no orphan START.** Every timed command
   that started also ended. The timeout did not interrupt a timed command
   mid-flight.

2. **WO22–WO24 total: 2,092,345 ms** (~34.9 minutes of timed execution).

3. **Last timed END: WO25 I07 restoration** (restoring
   `crates/hum-dev/src/status.rs` after the I07 mutation probe).

4. **98-second gap** between the last timed END and process termination,
   with zero timed ledger activity.

5. **Windows PowerShell 5.1** (`powershell.exe`) was present in the
   process tree at termination.

## Trace (pinned source: tools/check_all.ps1:1718)

After the WO25 mutation matrix completes, `check_all.ps1` runs the
**Unit C shell-equivalence** validation (Windows-only gate):

```powershell
if($env:OS -eq 'Windows_NT'){
  ...
  foreach($Contract in @(
    @('powershell',"$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"),
    @('pwsh',[string]$Pwsh[0].Source))){
    ...
    $Process=New-Object Diagnostics.Process
    ...
    $Process.WaitForExit()   # <-- untimed blocking wait
```

Each contract launches a child PowerShell that executes the **entire**
`test_fast_evidence_capture.ps1` suite (~87s on Linux; longer on Windows
runners), then blocks on `$Process.WaitForExit()`.

**This wait is not wrapped in the timing ledger.** No START is emitted
before the child launches; no END is emitted after it completes. The
98-second gap is this untimed window.

## Inference (labeled)

- **Inference**: The 98s gap was the Unit C PowerShell 5.1 child running
  the capture suite untimed. The ledger shows no activity because none
  was recorded — not because the machine was idle.
- **Inference**: The 3000s deadline fired during this untimed child
  execution. The deadline mechanism only observes timed commands; it
  cannot see (or account for) wall-clock spent in untimed subprocess waits.
- **Observation**: This is consistent with "887 balanced pairs" — the
  ledger is internally consistent; the gap is outside its observation
  window.

## Recommendation (smallest coverage-preserving approach)

Wrap the Unit C shell-equivalence child invocations in the timing ledger
(emit START before `$Process.Start()`, END after `$Process.WaitForExit()`
completes). This:

- Preserves all existing coverage (same child, same assertions, same
  shell-equivalence semantics).
- Closes the untimed gap: the deadline mechanism can then observe and
  account for this wall-clock.
- Requires no deadline increase, no coverage cuts, and no new process model.

Alternative (not recommended): split the 3000s budget to reserve a
Unit C allowance. This adds a second deadline mechanism and complicates
the timeout contract for no coverage benefit.

## Explicitly out of scope

No implementation performed. No deadline changed. No coverage cut.
No new process model. No Full campaign. This diagnosis is read-only.
