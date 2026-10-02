<#
.SYNOPSIS
  Synthetic validation of the compiler-front timing kit.
  Uses ONLY short non-Hum synthetic runner checks (pwsh children, temp git repo):
    Phase 1: pinned helper mechanics — dot-source guard, dual-channel output,
             Assert-HumCaptureComplete, timeout + descendant cleanup.
    Phase 2: kit dot-source guard (no execution); budget + boundary helpers.
    Phase 3: plan inventory (28 commands, kinds, exact argv arrays).
    Phase 4: classification — exact selection (zero/ignored/duplicate rejected),
             run-result and mutation-run-result from actual result text,
             run-launch gating predicate.
    Phase 5: ACTUAL finalization path — Complete-KitAbortRecords and
             Write-KitSummary with recorded commands:
             normal completion, early capture failure (abort sweep),
             capture-error bytes, identity drift, frozen-hash mismatch.
  No Hum/Cargo execution, no builds, no repo writes.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = '/home/hatch/workspace/hum-irverify-compat'
$KitPath = '/home/hatch/workspace/measure-compiler-front-kit/measure-compiler-front-kit.ps1'
$Pwsh = (Get-Process -Id $PID).Path
$WorkRoot = Join-Path ([IO.Path]::GetTempPath()) ('kit-harness-validate-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $WorkRoot | Out-Null
$Failures = New-Object 'System.Collections.Generic.List[string]'

function Assert-True([bool] $Cond, [string] $Name) {
  if ($Cond) { Write-Host "ok - $Name" } else { $Failures.Add($Name); Write-Host "FAIL - $Name" }
}

function New-SynthRecord {
  param($Seq, $Family, $State, $Outcome, $Elapsed)
  [ordered]@{
    seq = $Seq; family = $Family; label = "synth $Seq"; kind = 'control'; file = 'x'
    argv = @('a'); selector = ''; mutation = $null; note = ''
    state = $State; outcome = $Outcome; reason = 'synth'
    elapsed_ms = $Elapsed; exit_code = 0; timed_out = $false
  }
}

try {
  # ---------- Phase 1: pinned helper mechanics ----------
  . (Join-Path $RepoRoot 'tools/run_fast_evidence.ps1')
  Assert-True ((Get-Command Invoke-HumBinaryCapture -ErrorAction SilentlyContinue) -ne $null) 'helper dot-source exposes Invoke-HumBinaryCapture'

  $CapDir = Join-Path $WorkRoot 'cap-channels'
  $Cap = Invoke-HumBinaryCapture $Pwsh @('-NoProfile', '-NonInteractive', '-Command',
    "[Console]::Out.WriteLine('CHAN-OUT-1'); [Console]::Error.WriteLine('CHAN-ERR-1')") `
    $WorkRoot $CapDir 30 5 -CaseName 'kit-validate-channels'
  $OutText = [IO.File]::ReadAllText((Join-Path $CapDir 'stdout.bin'))
  $ErrText = [IO.File]::ReadAllText((Join-Path $CapDir 'stderr.bin'))
  Assert-True ($Cap.ExitCode -eq 0 -and -not $Cap.TimedOut) 'channels: exit 0, no timeout'
  Assert-True ($OutText -match 'CHAN-OUT-1') 'channels: stdout captured'
  Assert-True ($ErrText -match 'CHAN-ERR-1') 'channels: stderr captured'
  $AssertOk = $true
  try { $null = Assert-HumCaptureComplete $Cap } catch { $AssertOk = $false }
  Assert-True $AssertOk 'channels: Assert-HumCaptureComplete authenticates clean capture'

  $Marker = 'kit-synth-marker-9d2c'
  $Before = [int](& pgrep -f $Marker | Measure-Object | Select-Object -ExpandProperty Count)
  $Grandchild = "Start-Sleep -Seconds 90 # $Marker"
  $ChildCmd = "Start-Process -FilePath '$Pwsh' -ArgumentList @('-NoProfile','-NonInteractive','-Command','$Grandchild') -NoNewWindow; Start-Sleep -Seconds 90"
  $CapDir2 = Join-Path $WorkRoot 'cap-timeout'
  $Cap2 = Invoke-HumBinaryCapture $Pwsh @('-NoProfile', '-NonInteractive', '-Command', $ChildCmd) `
    $WorkRoot $CapDir2 6 5 -CaseName 'kit-validate-timeout'
  Assert-True ($Cap2.TimedOut -eq $true) 'timeout: TimedOut=true'
  Assert-True ($Cap2.DeadlineDisposition -ceq 'deadline_expired') 'timeout: deadline_expired'
  Assert-True ($Cap2.JobQuiescenceObserved -and $Cap2.FinalActiveProcessCount -eq 0) 'timeout: descendant quiescence authenticated, final_active=0'
  Start-Sleep -Seconds 2
  $After = [int](& pgrep -f $Marker | Measure-Object | Select-Object -ExpandProperty Count)
  Assert-True ($After -eq $Before) "timeout: no stray grandchild (before=$Before after=$After)"

  # Absolute-path control: bare tool names fail under the Windows native
  # capture path; the resolved absolute path must capture cleanly.
  $AbsGit = (Get-Command git -ErrorAction Stop).Source
  Assert-True ([IO.Path]::IsPathRooted($AbsGit)) 'absolute-path control: git resolves to a rooted path'
  $GitCapDir = Join-Path $WorkRoot 'cap-git-abs'
  $GitCap = Invoke-HumBinaryCapture $AbsGit @('--version') $WorkRoot $GitCapDir 30 5 -CaseName 'kit-validate-git-abs'
  $GitAssertOk = $true; try { $null = Assert-HumCaptureComplete $GitCap } catch { $GitAssertOk = $false }
  $GitOut = [IO.File]::ReadAllText((Join-Path $GitCapDir 'stdout.bin'))
  Assert-True ($GitAssertOk -and $GitCap.ExitCode -eq 0 -and $GitOut -match '^git version') 'absolute-path control: git --version captured and authenticated via absolute path'

  # ---------- Phase 2: kit dot-source guard; budget + boundary helpers ----------
  . $KitPath
  $Now = [DateTime]::UtcNow
  $r1 = Test-KitLaunchBudget $Now ($Now.AddSeconds(1000)) 60 5 180
  Assert-True ($r1.Launch -and $r1.TimeoutSeconds -eq 180) 'budget: 935s usable -> launch, clamped to max 180'
  $r2 = Test-KitLaunchBudget $Now ($Now.AddSeconds(100)) 60 5 180
  Assert-True ($r2.Launch -and $r2.TimeoutSeconds -eq 35) 'budget: 35s usable -> launch, clamped to 35 (reserve+grace charged)'
  $r3 = Test-KitLaunchBudget $Now ($Now.AddSeconds(70)) 60 5 180
  Assert-True (-not $r3.Launch) 'budget: 5s usable -> not-run (skip accounting)'
  Assert-True ((Test-KitMutationBudget $Now ($Now.AddSeconds(300)) 60 5) -eq $true) 'mutation budget: 235s usable -> proceed'
  Assert-True ((Test-KitMutationBudget $Now ($Now.AddSeconds(150)) 60 5) -eq $false) 'mutation budget: 85s usable -> skip'
  Assert-True ((Test-PathInside '/a/b/c' '/a/b') -eq $true) 'boundary: child inside parent'
  Assert-True ((Test-PathInside '/a/b' '/a/b') -eq $true) 'boundary: equal paths inside'
  Assert-True ((Test-PathInside '/a/bc' '/a/b') -eq $false) 'boundary: prefix sibling not inside'
  Assert-True ((Test-PathInside '/x' '/a/b') -eq $false) 'boundary: outside not inside'

  # ---------- Phase 3: plan inventory ----------
  $Plan = New-KitPlan
  Assert-True ($Plan.Count -eq 28) ("plan inventory: 28 entries (got $($Plan.Count))")
  $Kinds = @($Plan | Group-Object { $_['kind'] } | ForEach-Object { "$($_.Name)=$($_.Count)" } | Sort-Object) -join ','
  Assert-True ($Kinds -ceq 'control=10,list=4,mut-list=2,mut-run=2,probe=6,run=4') "plan kinds: $Kinds"
  $Probe0 = @($Plan | Where-Object { $_['kind'] -ceq 'probe' })[0]
  Assert-True ((($Probe0['argv'] -join ' ') -ceq 'backend-probe --format human examples/core/minimal_add.hum')) 'plan: exact argv arrays retained'

  # ---------- Phase 4: classification ----------
  $Sel = 'parser::tests::alpha'
  $SelOk = $true; try { Assert-ExactSelection "$Sel`: test" $Sel 't' } catch { $SelOk = $false }
  Assert-True $SelOk 'selection: exact single accepted'
  $SelZero = $false; try { Assert-ExactSelection '' $Sel 't' } catch { $SelZero = $true }
  Assert-True $SelZero 'selection: zero rejected'
  $SelDup = $false; try { Assert-ExactSelection "$Sel`: test`nother::x: test" $Sel 't' } catch { $SelDup = $true }
  Assert-True $SelDup 'selection: duplicate rejected'
  $SelIgnored = $false; try { Assert-ExactSelection 'other::x: test' $Sel 't' } catch { $SelIgnored = $true }
  Assert-True $SelIgnored 'selection: non-matching single (ignored) rejected'

  $RunOkText = "test $Sel ... ok`ntest result: ok. 1 passed; 0 failed; 0 ignored;"
  Assert-True ((Test-KitRunResult $RunOkText $Sel 0) -ceq 'ok') 'run result: named ok + 1 passed -> ok'
  Assert-True ((Test-KitRunResult 'some other output' $Sel 0) -ceq 'unexpected_exit') 'run result: exit 0 without named pass -> unexpected_exit'
  Assert-True ((Test-KitRunResult "error[E0308]: mismatched types`ncould not compile" $Sel 101) -ceq 'compile_failure') 'run result: compilation failure distinguished'
  Assert-True ((Test-KitRunResult $RunOkText $Sel 1) -ceq 'unexpected_exit') 'run result: nonzero exit without compile error -> unexpected_exit'

  $MutOkText = "test $Sel ... FAILED`ntest result: FAILED. 0 passed; 1 failed;"
  Assert-True ((Test-KitMutationRunResult $MutOkText $Sel 101) -ceq 'expected_failure') 'mutation result: exactly the intended FAILED -> expected_failure'
  Assert-True ((Test-KitMutationRunResult "test $Sel ... FAILED`ntest other::y ... FAILED" $Sel 101) -ceq 'unexpected_exit') 'mutation result: two FAILED lines -> unexpected_exit'
  Assert-True ((Test-KitMutationRunResult 'test other::y ... FAILED' $Sel 101) -ceq 'unexpected_exit') 'mutation result: wrong selector FAILED -> unexpected_exit'
  Assert-True ((Test-KitMutationRunResult 'error: could not compile' $Sel 101) -ceq 'compile_failure') 'mutation result: compilation failure distinguished'
  Assert-True ((Test-KitMutationRunResult $MutOkText $Sel 0) -ceq 'unexpected_exit') 'mutation result: exit 0 -> unexpected_exit'

  Assert-True ((Test-KitRunLaunchAllowed 's' @{}) -eq $false) 'launch gate: unknown selector not launched'
  Assert-True ((Test-KitRunLaunchAllowed 's' @{'s' = 'ok'}) -eq $true) 'launch gate: clean listing launches run'
  Assert-True ((Test-KitRunLaunchAllowed 's' @{'s' = 'bad'}) -eq $false) 'launch gate: failed listing blocks run'

  # Restore proof gate: staleness is prevented structurally (the kit resets
  # the proof on every launch); the predicate rejects absent/non-quiescent proof.
  $QGood = [ordered]@{ job_quiescent = $true; final_active = 0 }
  $QActive = [ordered]@{ job_quiescent = $false; final_active = 3 }
  Assert-True ((Test-KitRestoreProof $false $null) -eq $true) 'restore proof: no launch attempted -> restore allowed'
  Assert-True ((Test-KitRestoreProof $true $QGood) -eq $true) 'restore proof: current child quiescent -> restore allowed'
  Assert-True ((Test-KitRestoreProof $true $null) -eq $false) 'restore proof: helper threw (absent proof) -> retain + report'
  Assert-True ((Test-KitRestoreProof $true $QActive) -eq $false) 'restore proof: child still active -> retain + report'

  # ---------- Phase 5: actual finalization path ----------
  $tmpRepo = Join-Path $WorkRoot 'fake-repo'
  New-Item -ItemType Directory -Force -Path $tmpRepo | Out-Null
  & git init -q $tmpRepo; $global:LASTEXITCODE = 0
  & git -C $tmpRepo -c user.name=t -c user.email=t@t commit -q --allow-empty -m one; $global:LASTEXITCODE = 0
  & git -C $tmpRepo -c user.name=t -c user.email=t@t commit -q --allow-empty -m two; $global:LASTEXITCODE = 0
  $fakeHead = ((& git -C $tmpRepo rev-parse HEAD) | Out-String).Trim(); $global:LASTEXITCODE = 0
  $fakeBase = ((& git -C $tmpRepo rev-parse 'HEAD^') | Out-String).Trim(); $global:LASTEXITCODE = 0
  $SavedHead = $script:PinnedHead; $SavedBase = $script:PinnedBase

  $tmpOut = Join-Path $WorkRoot 'kit-out'
  New-Item -ItemType Directory -Force -Path $tmpOut | Out-Null
  $script:NdjsonPath = Join-Path $tmpOut 'timing.ndjson'
  $frozenFile = Join-Path $tmpOut 'frozen.bin'
  [IO.File]::WriteAllText($frozenFile, 'fake-binary-bytes')
  $frozenHash = Get-FileSha256 $frozenFile

  # 5a. normal completion through Write-KitSummary
  $script:Plan = New-KitPlan
  $script:Records = @(
    (New-SynthRecord 1 'F0' 'measured' 'ok' 120),
    (New-SynthRecord 2 'F0' 'measured' 'ok' 130),
    (New-SynthRecord 11 'F1' 'measured' 'ok' 5000),
    (New-SynthRecord 12 'F1' 'incomplete' $null 180000),
    (New-SynthRecord 19 'F2' 'measured' 'expected_failure' 9000),
    (New-SynthRecord 20 'F2' 'error' $null $null),
    (New-SynthRecord 23 'F3' 'not-run' $null $null)
  )
  $script:CargoVersion = 'cargo 1.99.0 (synth)'; $script:RustcVersion = 'rustc 1.99.0 (synth)'
  $script:KitError = $null
  $script:PinnedHead = $fakeHead; $script:PinnedBase = $fakeBase
  $ValGit = (Get-Command git -ErrorAction Stop).Source
  Write-KitSummary -OutDir $tmpOut -RepoRoot $tmpRepo -FrozenPath $frozenFile -FrozenSha256 $frozenHash `
    -KitStartUtc ([DateTime]::UtcNow.AddMinutes(-5)) -AbsoluteBudgetSeconds 900 -GitPath $ValGit
  $sum = Get-Content (Join-Path $tmpOut 'summary.json') -Raw | ConvertFrom-Json
  Assert-True ($sum.family_stats.F0.mean_ms -eq 125) 'summary: F0 mean over clean samples only'
  Assert-True ($sum.family_stats.F1.mean_ms -eq 5000 -and $sum.family_stats.F1.measured_clean -eq 1 -and $sum.family_stats.F1.incomplete -eq 1) 'summary: incomplete timing excluded from mean'
  Assert-True ($sum.family_stats.F2.measured_clean -eq 1 -and $sum.family_stats.F2.error -eq 1) 'summary: F2 counts'
  Assert-True ($null -eq $sum.kit_error) 'summary: no kit_error on clean verification'
  Assert-True ($sum.git_after.clean -eq $true -and $sum.git_after.head -ceq $fakeHead) 'summary: final git verification recorded'
  Assert-True ($sum.provenance.hum_frozen_verified -eq $true) 'summary: frozen hash verified'
  Assert-True ($sum.inventory.Count -eq 7) 'summary: inventory retained'

  # 5b. identity drift fails the kit (fresh outdir: git capture tags are single-use)
  $tmpOutB = Join-Path $WorkRoot 'kit-out-b'
  New-Item -ItemType Directory -Force -Path $tmpOutB | Out-Null
  $script:KitError = $null
  $script:PinnedHead = '0000000000000000000000000000000000000000'
  Write-KitSummary -OutDir $tmpOutB -RepoRoot $tmpRepo -FrozenPath $frozenFile -FrozenSha256 $frozenHash `
    -KitStartUtc ([DateTime]::UtcNow.AddMinutes(-5)) -AbsoluteBudgetSeconds 900 -GitPath $ValGit
  Assert-True ($script:KitError -match 'final verification failed' -and $script:KitError -match 'head drift') 'summary: head drift FAILS the kit'
  $script:PinnedHead = $fakeHead

  # 5c. frozen hash mismatch fails the kit (fresh outdir)
  $tmpOutC = Join-Path $WorkRoot 'kit-out-c'
  New-Item -ItemType Directory -Force -Path $tmpOutC | Out-Null
  $script:KitError = $null
  Write-KitSummary -OutDir $tmpOutC -RepoRoot $tmpRepo -FrozenPath $frozenFile -FrozenSha256 'deadbeef' `
    -KitStartUtc ([DateTime]::UtcNow.AddMinutes(-5)) -AbsoluteBudgetSeconds 900 -GitPath $ValGit
  Assert-True ($script:KitError -match 'final verification failed' -and $script:KitError -match 'frozen binary hash') 'summary: frozen hash mismatch FAILS the kit'

  # 5d. early capture failure: abort sweep finalizes every planned ID
  $script:Plan = New-KitPlan
  $script:Records = @(
    (New-SynthRecord 1 'F0' 'measured' 'ok' 100),
    (New-SynthRecord 2 'F0' 'error' $null $null)
  )
  $script:KitError = 'synthetic boom'
  Complete-KitAbortRecords -Reason 'aborted'
  $Seqs = @($script:Records | ForEach-Object { $_['seq'] } | Sort-Object)
  $AllOnce = ($Seqs.Count -eq 28) -and (($Seqs | Select-Object -Unique).Count -eq 28) -and ($Seqs[0] -eq 1) -and ($Seqs[27] -eq 28)
  Assert-True $AllOnce 'abort sweep: exactly one terminal state per planned ID (28)'
  $AbortedTail = @($script:Records | Where-Object { $_['seq'] -ge 3 } | Where-Object { $_['state'] -ceq 'not-run' -and $_['reason'] -ceq 'aborted' }).Count
  Assert-True ($AbortedTail -eq 26) 'abort sweep: unstarted commands are definitive not-run/aborted'
  $FailedKept = @($script:Records | Where-Object { $_['seq'] -eq 2 })[0]
  Assert-True ($FailedKept['state'] -ceq 'error') 'abort sweep: current failed command keeps its terminal state'

  $script:PinnedHead = $SavedHead; $script:PinnedBase = $SavedBase
} finally {
  Remove-Item -LiteralPath $WorkRoot -Recurse -Force -ErrorAction SilentlyContinue
}

if ($Failures.Count -gt 0) { throw "harness validation failed: $($Failures -join '; ')" }
Write-Host 'harness validation: all synthetic checks passed'
