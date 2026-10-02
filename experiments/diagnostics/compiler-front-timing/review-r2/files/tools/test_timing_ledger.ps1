# Focused controls for the durable timing journal (tools/hum_timing_ledger.ps1).
#
# Exercises the REAL production function texts (extracted from check_all.ps1,
# test_exact_rust_selector.ps1 and run_fast_evidence.ps1 via the PowerShell
# AST, never re-implemented) against synthetic native commands and proves:
#   1. success: START+END records sharing one unique launch id, with the
#      actual executable, argv, group, kind, phase, elapsed and exit code
#   2. expected failure: exit 3 records outcome=failure and the wrapper still
#      returns the failure (nothing is swallowed or retried)
#   3. wrapper-throw path: a mid-execution launch error records outcome=error
#      with the message; a pre-launch validation failure throws before any
#      record (the journal times executions, not config errors)
#   4. interruption retention through the real wrapped command: a child
#      process killed mid-command (its buffered console output lost, exactly
#      like hum-dev under the outer capture deadline) leaves its flushed
#      START on disk with no END; the id-based reader flags completion as
#      unobserved, and a repeated label cannot hide the unfinished command
#   5. unchanged outputs: stdout/stderr/exit bytes through the timed wrappers
#      are identical to the untimed reference invocations
#   6. list/run phase separation and selector credit through
#      Invoke-ExactRustTest
#   7. module configurations: absent module (standalone import) runs untimed
#      instead of failing before launch; loaded-but-disabled runs unchanged
#      with no journal file; enabled records
#   8. nested launches: only the outer launch is recorded (no double entries)
#   9. every journal line is valid NDJSON with the required fields
#
# Platform-neutral by construction: the interruption child and the probes use
# the running pwsh itself as the native executable; nothing here depends on
# POSIX shell scripts except the fake-cargo fixtures, which are plain files
# with a shebang (the Linux/macOS control host). Windows process-launch and
# timeout semantics of the unchanged production wrappers stay CI's authority.

$ErrorActionPreference = 'Stop'

$ToolsDir = $PSScriptRoot
$WorkDir = Join-Path ([IO.Path]::GetTempPath()) ('hum-timing-controls-' + [guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($WorkDir) | Out-Null
$LedgerPath = Join-Path $WorkDir 'ledger.ndjson'
$Pwsh = (Get-Command pwsh -CommandType Application -ErrorAction Stop).Source

. (Join-Path $ToolsDir 'hum_timing_ledger.ps1')

function Import-RealFunction([string] $File, [string] $Name) {
  $Source = [IO.File]::ReadAllText($File)
  $Tokens = $null; $Errors = $null
  $Ast = [Management.Automation.Language.Parser]::ParseInput($Source, [ref]$Tokens, [ref]$Errors)
  if ($Errors.Count -ne 0) { throw "control: $File failed to parse" }
  $Defs = @($Ast.FindAll({ param($N) $N -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq $Name }, $true))
  if ($Defs.Count -ne 1) { throw "control: $Name not found exactly once in $File" }
  return [scriptblock]::Create($Defs[0].Extent.Text)
}

function Get-RealFunctionText([string] $File, [string] $Name) {
  $Source = [IO.File]::ReadAllText($File)
  $Tokens = $null; $Errors = $null
  $Ast = [Management.Automation.Language.Parser]::ParseInput($Source, [ref]$Tokens, [ref]$Errors)
  if ($Errors.Count -ne 0) { throw "control: $File failed to parse" }
  $Defs = @($Ast.FindAll({ param($N) $N -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq $Name }, $true))
  if ($Defs.Count -ne 1) { throw "control: $Name not found exactly once in $File" }
  return $Defs[0].Extent.Text
}

function Quote-Single([string] $Text) { return "'" + ($Text -replace "'", "''") + "'" }

$CheckAll = Join-Path $ToolsDir 'check_all.ps1'
$SelectorModule = Join-Path $ToolsDir 'test_exact_rust_selector.ps1'
. (Import-RealFunction $SelectorModule 'Assert-ExactRustSelectorSyntax')
. (Import-RealFunction $SelectorModule 'Assert-ExactRustSelectorEvidence')
. (Import-RealFunction $SelectorModule 'Invoke-ExactRustNativeCapture')
. (Import-RealFunction $SelectorModule 'Invoke-ExactRustTest')
. (Import-RealFunction $CheckAll 'Invoke-Native')
. (Import-RealFunction $CheckAll 'Read-NativeOutput')
. (Import-RealFunction $CheckAll 'Read-NativeOutputWithExit')
. (Import-RealFunction $CheckAll 'Read-NativeChannelsWithExit')
. (Import-RealFunction $CheckAll 'Read-NativeArgumentListWithExit')
. (Import-RealFunction $CheckAll 'Read-NativeBytesWithExit')

$script:Passed = 0
$script:Failed = 0
function Assert-Control([bool] $Condition, [string] $Label) {
  if ($Condition) { $script:Passed++; Write-Host "  ok: $Label" }
  else { $script:Failed++; Write-Host "  FAIL: $Label" }
}

function Read-LedgerRecords([string] $Path) {
  return @([IO.File]::ReadAllLines($Path) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object { $_ | ConvertFrom-Json })
}

# Reader semantics: match END to START by the unique launch id. A START with
# no END of the same id means completion was unobserved (interrupted).
function Get-IncompleteLaunches([object[]] $Records) {
  $EndIds = @($Records | Where-Object { $_.event -ceq 'end' } | ForEach-Object { $_.id })
  return @($Records | Where-Object { $_.event -ceq 'start' -and $EndIds -notcontains $_.id })
}

function Get-LaunchPair([object[]] $Records, [string] $Id) {
  return @{
    Start = @($Records | Where-Object { $_.event -ceq 'start' -and $_.id -ceq $Id })[0]
    End   = @($Records | Where-Object { $_.event -ceq 'end' -and $_.id -ceq $Id })[0]
  }
}

function Write-FakeCargo([string] $Path, [string] $Body) {
  # BOM-less UTF-8: a BOM before the shebang breaks kernel script parsing.
  [IO.File]::WriteAllText($Path, $Body, [Text.UTF8Encoding]::new($false))
  & chmod +x $Path
  if ($LASTEXITCODE -ne 0) { throw 'control: chmod failed' }
  $global:LASTEXITCODE = 0
}

$Selector = 'fake_crate::tests::timing_probe'

Write-Host 'control 1: initialize truncates a stale ledger and opens a session'
[IO.File]::WriteAllText($LedgerPath, "stale junk`n", [Text.Encoding]::UTF8)
Initialize-HumTimingLedger -LedgerPath $LedgerPath
$Records = Read-LedgerRecords $LedgerPath
Assert-Control ($Records.Count -eq 1 -and $Records[0].event -ceq 'session' -and -not [string]::IsNullOrEmpty($Records[0].session)) 'stale content replaced by one session record'

Write-Host 'control 2: success records share one launch id and name the executable'
Set-HumTimingGroup 'compiler-front'
$FakeCargo = Join-Path $WorkDir 'fake-cargo-ok.sh'
Write-FakeCargo $FakeCargo "#!/bin/sh`necho `"cargo-stdout:$2`"`necho `"cargo-stderr:$2`" >&2`nexit 0`n"
$Result = Invoke-ExactRustNativeCapture $FakeCargo @('test', $Selector, '--', '--exact') -TimingKind 'cargo-selector' -TimingPhase 'run' -TimingLabel 'control label' -TimingSelector $Selector
$Records = Read-LedgerRecords $LedgerPath
$Start = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
$End = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
Assert-Control ($Result.ExitCode -eq 0) 'wrapper still returns exit 0'
Assert-Control (-not [string]::IsNullOrEmpty($Start.id) -and $End.id -ceq $Start.id) 'start and end share one unique launch id'
Assert-Control ($Start.executable -ceq $FakeCargo -and $End.executable -ceq $FakeCargo) 'both records name the actual executable'
Assert-Control ($Start.kind -ceq 'cargo-selector' -and $Start.phase -ceq 'run' -and $Start.group -ceq 'compiler-front') 'start has kind/phase/group'
Assert-Control ($Start.label -ceq 'control label' -and $Start.selector -ceq $Selector) 'start has label/selector'
Assert-Control ($Start.argv -join ' ' -ceq "test $Selector -- --exact") 'start carries argv'
Assert-Control ($End.outcome -ceq 'success' -and $End.exit_code -eq 0 -and $End.elapsed_ms -ge 0) 'end has success outcome, exit 0, elapsed'

Write-Host 'control 3: expected failure records outcome=failure without swallowing'
$FakeFail = Join-Path $WorkDir 'fake-cargo-fail.sh'
Write-FakeCargo $FakeFail "#!/bin/sh`necho `"failing`"`nexit 3`n"
$FailResult = Invoke-ExactRustNativeCapture $FakeFail @('test', $Selector) -TimingKind 'cargo-selector' -TimingPhase 'run' -TimingLabel 'control fail' -TimingSelector $Selector
$Records = Read-LedgerRecords $LedgerPath
$FailEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$FailStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control ($FailResult.ExitCode -eq 3 -and $FailResult.Output -join '' -ceq 'failing') 'wrapper returns the failure output and exit 3'
Assert-Control ($FailEnd.id -ceq $FailStart.id -and $FailEnd.outcome -ceq 'failure' -and $FailEnd.exit_code -eq 3) 'end shares the start id and records failure with exit 3'

Write-Host 'control 4: pre-launch validation failure throws before any record (no execution to time)'
$CountBefore = (Read-LedgerRecords $LedgerPath).Count
$Threw = $false
try { $null = Invoke-ExactRustNativeCapture (Join-Path $WorkDir 'no-such-cargo') @('test') -TimingKind 'cargo-selector' -TimingPhase 'list' -TimingLabel 'control throw' -TimingSelector $Selector }
catch { $Threw = $_.Exception.Message -match 'cargo executable is unavailable' }
$CountAfter = (Read-LedgerRecords $LedgerPath).Count
Assert-Control $Threw 'missing cargo still throws the original message'
Assert-Control ($CountAfter -eq $CountBefore) 'validation failure adds no records: the journal times executions, not config errors'

Write-Host 'control 4b: launch error mid-execution records outcome=error with the message'
$LaunchThrew = $false
try { $null = Read-NativeChannelsWithExit 'control launch-error' (Join-Path $WorkDir 'no-such-exe') @() }
catch { $LaunchThrew = $true }
$Records = Read-LedgerRecords $LedgerPath
$LaunchEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$LaunchStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control $LaunchThrew 'launch error still throws'
Assert-Control ($LaunchEnd.id -ceq $LaunchStart.id -and $LaunchEnd.outcome -ceq 'error' -and -not [string]::IsNullOrEmpty($LaunchEnd.error)) 'start+end share the id and record the error outcome with message'

Write-Host 'control 5: timed cargo wrapper output is byte-identical to the untimed reference'
$FakeEcho = Join-Path $WorkDir 'fake-cargo-echo.sh'
Write-FakeCargo $FakeEcho "#!/bin/sh`necho `"out:$2`"`necho `"err:$2`" >&2`nexit 7`n"
$Timed = Invoke-ExactRustNativeCapture $FakeEcho @('test', $Selector, '--', '--exact') -TimingKind 'cargo-selector' -TimingPhase 'run' -TimingLabel 'control bytes' -TimingSelector $Selector
$PrevEap = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
try { $RefLines = @(& $FakeEcho 'test' $Selector '--' '--exact' 2>&1 | ForEach-Object { $_.ToString() }); $RefExit = $LASTEXITCODE }
finally { $ErrorActionPreference = $PrevEap }
Assert-Control ((($Timed.Output | Sort-Object) -join "`n") -ceq (($RefLines | Sort-Object) -join "`n") -and $Timed.ExitCode -eq $RefExit -and $RefExit -eq 7) 'stdout/stderr/exit identical through the timed wrapper (stderr/stdout merge order is OS-scheduled, compared as multisets)'

Write-Host 'control 6: hum-cli wrapper output is byte-identical to the reference'
$CliScript = Join-Path $WorkDir 'cli-probe.sh'
Write-FakeCargo $CliScript "#!/bin/sh`necho `"cli-out`"`necho `"cli-err`" >&2`nexit 5`n"
$CliTimed = Read-NativeChannelsWithExit 'control cli' $CliScript @()
$RefInfo = New-Object System.Diagnostics.ProcessStartInfo
$RefInfo.FileName = $CliScript; $RefInfo.UseShellExecute = $false; $RefInfo.RedirectStandardOutput = $true; $RefInfo.RedirectStandardError = $true
$RefInfo.StandardOutputEncoding = [System.Text.Encoding]::UTF8; $RefInfo.StandardErrorEncoding = [System.Text.Encoding]::UTF8
$RefProc = [System.Diagnostics.Process]::Start($RefInfo)
$RefOut = $RefProc.StandardOutput.ReadToEnd(); $RefErr = $RefProc.StandardError.ReadToEnd(); $RefProc.WaitForExit()
Assert-Control ($CliTimed.Stdout -ceq $RefOut -and $CliTimed.Stderr -ceq $RefErr -and $CliTimed.ExitCode -eq $RefProc.ExitCode -and $CliTimed.ExitCode -eq 5) 'cli stdout/stderr/exit identical'
$Records = Read-LedgerRecords $LedgerPath
$CliEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$CliStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control ($CliEnd.id -ceq $CliStart.id -and $CliEnd.kind -ceq 'hum-cli' -and $CliEnd.label -ceq 'control cli' -and $CliEnd.outcome -ceq 'failure' -and $CliEnd.exit_code -eq 5 -and $CliEnd.executable -ceq $CliScript) 'cli end shares the start id and records kind/label/failure/executable'

Write-Host 'control 7: argument-list variant preserves whitespace-bearing arguments'
$WsArgs = @('a b', "c`nd", 'e')
$WsTimed = Read-NativeArgumentListWithExit 'control ws' '/bin/echo' $WsArgs
Assert-Control (($WsTimed.Stdout -ceq "a b c`nd e`n") -and $WsTimed.ExitCode -eq 0) 'whitespace/newline args preserved byte-identically'

Write-Host 'control 8: Invoke-ExactRustTest emits list then run records with distinct ids'
$script:ExactRustSelectorCredits = New-Object 'System.Collections.Generic.List[string]'
$FakeTestCargo = Join-Path $WorkDir 'fake-cargo-test.sh'
Write-FakeCargo $FakeTestCargo ("#!/bin/sh`nSEL=`$2`nif [ `"`$5`" = `"--list`" ]; then`n  echo `"`${SEL}: test`"`n  exit 0`nfi`necho `"running 1 test`"`necho `"test `${SEL} ... ok`"`necho `"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;`"`nexit 0`n")
Set-HumTimingGroup 'compiler-corpus'
$Before = (Read-LedgerRecords $LedgerPath).Count
Invoke-ExactRustTest 'control exact test' $FakeTestCargo $Selector
$New = @(Read-LedgerRecords $LedgerPath | Select-Object -Skip $Before)
$Starts = @($New | Where-Object { $_.event -ceq 'start' })
$Ends = @($New | Where-Object { $_.event -ceq 'end' })
$Ids = @($Starts | ForEach-Object { $_.id })
Assert-Control ($Starts.Count -eq 2 -and $Ends.Count -eq 2) 'one start+end pair per phase'
Assert-Control ($Starts[0].phase -ceq 'list' -and $Starts[1].phase -ceq 'run') 'list precedes run'
Assert-Control (($Ids | Select-Object -Unique).Count -eq 2 -and ($Ends | ForEach-Object { $_.id } | Select-Object -Unique).Count -eq 2) 'each launch has a distinct id'
Assert-Control ((($Ends | ForEach-Object { $_.id }) | Where-Object { $Ids -contains $_ }).Count -eq 2) 'ends match starts by id'
Assert-Control (($Starts | ForEach-Object { $_.selector } | Select-Object -Unique) -eq $Selector) 'both records name the selector'
Assert-Control (($Ends | Where-Object { $_.outcome -ceq 'success' }).Count -eq 2) 'both phases succeeded'
Assert-Control (@($script:ExactRustSelectorCredits | Where-Object { $_ -ceq $Selector }).Count -eq 1) 'selector credit accounting unchanged'

Write-Host 'control 9: real wrapped commands - completion writes END, kill leaves START without END'
$KillLedger = Join-Path $WorkDir 'kill-ledger.ndjson'
$ChildScript = Join-Path $WorkDir 'kill-child.ps1'
$QuickExitScript = Join-Path $WorkDir 'quick-exit.ps1'
$QuickOkScript = Join-Path $WorkDir 'quick-ok.ps1'
$LongSleepScript = Join-Path $WorkDir 'long-sleep.ps1'
# The Channels wrapper rejects whitespace-bearing arguments, so the child
# drives pwsh via -File scripts (platform-neutral: no shell involved).
[IO.File]::WriteAllText($QuickExitScript, "exit 41`n", [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText($QuickOkScript, "exit 0`n", [Text.UTF8Encoding]::new($false))
[IO.File]::WriteAllText($LongSleepScript, "Start-Sleep -Seconds 120`n", [Text.UTF8Encoding]::new($false))
$ModulePath = Join-Path $ToolsDir 'hum_timing_ledger.ps1'
$ChannelsText = Get-RealFunctionText $CheckAll 'Read-NativeChannelsWithExit'
# The child runs the REAL production wrapper text against the REAL module.
# Its buffered console output dies with it (like hum-dev under the outer
# capture deadline); the flushed journal file is the retention boundary.
[IO.File]::WriteAllText($ChildScript, (@(
  '$ErrorActionPreference = ' + (Quote-Single 'Stop')
  '. ' + (Quote-Single $ModulePath)
  $ChannelsText
  ('$KillLedger = ' + (Quote-Single $KillLedger))
  ('$PwshExe = ' + (Quote-Single $Pwsh))
  ('$QuickExit = ' + (Quote-Single $QuickExitScript))
  ('$QuickOk = ' + (Quote-Single $QuickOkScript))
  ('$LongSleep = ' + (Quote-Single $LongSleepScript))
  'Initialize-HumTimingLedger -LedgerPath $KillLedger'
  '$null = Read-NativeChannelsWithExit ''probe-quick'' $PwshExe @(''-NoProfile'',''-NonInteractive'',''-File'',$QuickExit)'
  '$null = Read-NativeChannelsWithExit ''dup-label'' $PwshExe @(''-NoProfile'',''-NonInteractive'',''-File'',$QuickOk)'
  '$null = Read-NativeChannelsWithExit ''dup-label'' $PwshExe @(''-NoProfile'',''-NonInteractive'',''-File'',$LongSleep)'
) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
$Child = Start-Process -FilePath $Pwsh -ArgumentList '-NoLogo','-NoProfile','-NonInteractive','-File',$ChildScript -PassThru
# Bounded: wait for the third START (both completions plus the interrupted
# launch) to hit disk, then kill. The START is flushed before launch, so
# waiting for it proves the record survived the kill that follows.
$Deadline = [DateTime]::UtcNow.AddSeconds(90)
do {
  Start-Sleep -Milliseconds 200
  $ProbeRecords = @()
  if (Test-Path -LiteralPath $KillLedger) { $ProbeRecords = Read-LedgerRecords $KillLedger }
} while ((@($ProbeRecords | Where-Object { $_.event -ceq 'start' }).Count -lt 3) -and [DateTime]::UtcNow -lt $Deadline)
$ThirdStartSeen = (@($ProbeRecords | Where-Object { $_.event -ceq 'start' }).Count -ge 3)
if (-not $Child.HasExited) { Stop-Process -Id $Child.Id -Force }
$Child.WaitForExit(15000) | Out-Null
$ChildDied = $Child.HasExited
$KillRecords = @()
if (Test-Path -LiteralPath $KillLedger) { $KillRecords = Read-LedgerRecords $KillLedger }
Assert-Control ($ThirdStartSeen -and $ChildDied) 'interrupted launch start flushed to disk before the kill; child is dead'
$QuickPair = Get-LaunchPair $KillRecords (@($KillRecords | Where-Object { $_.event -ceq 'start' -and $_.label -ceq 'probe-quick' })[0].id)
Assert-Control ($null -ne $QuickPair.Start -and $null -ne $QuickPair.End -and $QuickPair.End.outcome -ceq 'failure' -and $QuickPair.End.exit_code -eq 41 -and $QuickPair.End.executable -ceq $Pwsh) 'normal completion writes END with the exit code and executable, matched by id'
$DupStarts = @($KillRecords | Where-Object { $_.event -ceq 'start' -and $_.label -ceq 'dup-label' })
$DupEndIds = @($KillRecords | Where-Object { $_.event -ceq 'end' -and $_.label -ceq 'dup-label' } | ForEach-Object { $_.id })
$DupComplete = @($DupStarts | Where-Object { $DupEndIds -contains $_.id })
$DupIncomplete = @(Get-IncompleteLaunches $KillRecords | Where-Object { $_.label -ceq 'dup-label' })
Assert-Control ($DupStarts.Count -eq 2 -and ($DupStarts.id | Select-Object -Unique).Count -eq 2) 'repeated label gets two distinct launch ids'
Assert-Control ($DupComplete.Count -eq 1 -and $DupIncomplete.Count -eq 1) 'id matching pairs the finished repeat and flags the unfinished one: no END means completion unobserved'

Write-Host 'control 10: nested launches record only the outer command'
$NestedLedger = Join-Path $WorkDir 'nested-ledger.ndjson'
Initialize-HumTimingLedger -LedgerPath $NestedLedger
Set-HumTimingGroup 'control-nested'
$Outer = Start-HumTimedCommand -Kind 'test' -Label 'outer' -Phase 'exec' -Executable $CliScript -Arguments @()
$InnerScope = Start-HumTimedCommand -Kind 'test' -Label 'inner' -Phase 'exec' -Executable $CliScript -Arguments @()
Assert-Control ($null -eq $InnerScope) 'nested start is suppressed while the outer launch is open'
$null = Read-NativeChannelsWithExit 'nested-wrapped' $CliScript @()
Stop-HumTimedCommand -Scope $Outer -ExitCode 0 -Outcome 'success'
$After = Start-HumTimedCommand -Kind 'test' -Label 'after' -Phase 'exec'
Assert-Control ($null -ne $After) 'a new launch records again after the outer scope closes'
Stop-HumTimedCommand -Scope $After -ExitCode 0 -Outcome 'success'
$NestedRecords = Read-LedgerRecords $NestedLedger
$NestedStarts = @($NestedRecords | Where-Object { $_.event -ceq 'start' })
$NestedEnds = @($NestedRecords | Where-Object { $_.event -ceq 'end' })
Assert-Control ($NestedStarts.Count -eq 2 -and ($NestedStarts.label -join ',') -ceq 'outer,after') 'only outer and after launches recorded; the nested wrapper added none'
Assert-Control ($NestedEnds.Count -eq 2 -and (Get-IncompleteLaunches $NestedRecords).Count -eq 0) 'both recorded launches completed'
Initialize-HumTimingLedger -LedgerPath $LedgerPath
Set-HumTimingGroup 'compiler-corpus'

Write-Host 'control 11: without initialization the wrappers run untimed and unchanged'
$script:HumTimingLedgerPath = ''
$script:HumTimingGroup = ''
$BareScope = Start-HumTimedCommand -Kind 'hum-cli' -Label 'bare' -Phase 'cli'
Assert-Control ($null -eq $BareScope) 'start returns null when disabled'
Stop-HumTimedCommand -Scope $null -ExitCode 0 -Outcome 'success'
$Bare = Read-NativeChannelsWithExit 'control bare' $CliScript @()
Assert-Control ($Bare.ExitCode -eq 5 -and $Bare.Stdout -ceq $RefOut) 'wrapper output unchanged with timing disabled'
Assert-Control (-not (Test-Path -LiteralPath (Join-Path $WorkDir 'disabled-ledger.ndjson'))) 'no ledger file created when disabled'
Initialize-HumTimingLedger -LedgerPath $LedgerPath
Set-HumTimingGroup 'compiler-corpus'

Write-Host 'control 12: absent timing module - standalone imports run untimed, never failing before launch'
$SelectorChild = Join-Path $WorkDir 'absent-selector-child.ps1'
[IO.File]::WriteAllText($SelectorChild, (@(
  '$ErrorActionPreference = ' + (Quote-Single 'Stop')
  (Get-RealFunctionText $SelectorModule 'Assert-ExactRustSelectorSyntax')
  (Get-RealFunctionText $SelectorModule 'Assert-ExactRustSelectorEvidence')
  (Get-RealFunctionText $SelectorModule 'Invoke-ExactRustNativeCapture')
  (Get-RealFunctionText $SelectorModule 'Invoke-ExactRustTest')
  'if ($null -ne (Get-Command Start-HumTimedCommand -CommandType Function -ErrorAction SilentlyContinue)) { throw ''timing module unexpectedly present'' }'
  '$script:ExactRustSelectorCredits = New-Object ''System.Collections.Generic.List[string]'''
  ('Invoke-ExactRustTest ''absent-module probe'' ' + (Quote-Single $FakeTestCargo) + ' ' + (Quote-Single $Selector))
  'if (@($script:ExactRustSelectorCredits | Where-Object { $_ -ceq ' + (Quote-Single $Selector) + ' }).Count -ne 1) { throw ''selector credit missing'' }'
  'Write-Output ''absent-selector-ok'''
) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
$SelPrevEap = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
try { $SelChildOut = @(& $Pwsh '-NoLogo' '-NoProfile' '-NonInteractive' '-File' $SelectorChild 2>&1 | ForEach-Object { $_.ToString() }); $SelChildExit = $LASTEXITCODE }
finally { $ErrorActionPreference = $SelPrevEap }
$global:LASTEXITCODE = 0
Assert-Control ($SelChildExit -eq 0 -and $SelChildOut -contains 'absent-selector-ok') 'selector script runs untimed with the module absent (list+run+credit intact)'
# Mirrors test_ci_policy.ps1: Read-NativeOutputWithExit imported alone, no module.
$PolicyChild = Join-Path $WorkDir 'absent-policy-child.ps1'
[IO.File]::WriteAllText($PolicyChild, (@(
  '$ErrorActionPreference = ' + (Quote-Single 'Stop')
  (Get-RealFunctionText $CheckAll 'Read-NativeOutputWithExit')
  ('$R = Read-NativeOutputWithExit ''policy-import probe'' ' + (Quote-Single $CliScript) + ' @()')
  'if ($R.ExitCode -ne 5 -or $R.Output -notmatch ''cli-out'') { throw (''policy import mismatch: exit='' + $R.ExitCode) }'
  'Write-Output ''absent-policy-ok'''
) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
$PolPrevEap = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
try { $PolChildOut = @(& $Pwsh '-NoLogo' '-NoProfile' '-NonInteractive' '-File' $PolicyChild 2>&1 | ForEach-Object { $_.ToString() }); $PolChildExit = $LASTEXITCODE }
finally { $ErrorActionPreference = $PolPrevEap }
$global:LASTEXITCODE = 0
Assert-Control ($PolChildExit -eq 0 -and $PolChildOut -contains 'absent-policy-ok') 'check_all wrapper imported alone (policy-test shape) runs untimed'

Write-Host 'control 13: Invoke-Native preserves throw behavior and records'
$NativeLedgerBefore = (Read-LedgerRecords $LedgerPath).Count
Invoke-Native 'control native ok' '/bin/echo' @('native-hi')
$NativeThrew = $false
try { Invoke-Native 'control native fail' $FakeFail @('test', $Selector) }
catch { $NativeThrew = $_.Exception.Message -ceq "control native fail failed with exit code 3" }
$NativeRecords = @(Read-LedgerRecords $LedgerPath | Select-Object -Skip $NativeLedgerBefore)
$NativeStarts = @($NativeRecords | Where-Object { $_.event -ceq 'start' })
$NativeEnds = @($NativeRecords | Where-Object { $_.event -ceq 'end' })
Assert-Control $NativeThrew 'nonzero exit still throws the original message'
Assert-Control ($NativeStarts.Count -eq 2 -and $NativeEnds.Count -eq 2 -and ($NativeEnds.outcome -join ',') -ceq 'success,failure') 'both launches recorded with success then failure outcomes'
Assert-Control (($NativeStarts.executable | Select-Object -Unique | Sort-Object) -join '|' -ceq '/bin/echo|' + $FakeFail) 'executables recorded'
Assert-Control ((($NativeEnds | ForEach-Object { $_.id }) | Where-Object { ($NativeStarts.id) -contains $_ }).Count -eq 2) 'ends match starts by id'

Write-Host 'control 14: Read-NativeOutput preserves text and throw behavior'
$OutText = Read-NativeOutput 'control readout ok' '/bin/echo' @('hello')
$OutThrew = $false
try { $null = Read-NativeOutput 'control readout fail' $FakeFail @('x') }
catch { $OutThrew = $_.Exception.Message -ceq 'control readout fail failed with exit code 3' }
Assert-Control ($OutText -ceq 'hello') 'output text unchanged'
Assert-Control $OutThrew 'failure still throws the original message'
$OutEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
Assert-Control ($OutEnd.kind -ceq 'native-output' -and $OutEnd.outcome -ceq 'failure' -and $OutEnd.exit_code -eq 3) 'failure recorded with kind and exit code'

Write-Host 'control 15: Read-NativeOutputWithExit records kind and executable'
$OweResult = Read-NativeOutputWithExit 'control owe' $CliScript @()
Assert-Control ($OweResult.ExitCode -eq 5) 'exit code preserved'
$OweEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
Assert-Control ($OweEnd.kind -ceq 'native-output' -and $OweEnd.executable -ceq $CliScript -and $OweEnd.outcome -ceq 'failure') 'kind/executable/outcome recorded'

Write-Host 'control 16: Read-NativeBytesWithExit preserves bytes and records a harness timeout as error'
$BytesResult = Read-NativeBytesWithExit 'control bytes ok' '/bin/echo' @('byte-hi')
$TimeoutThrew = $false
try { $null = Read-NativeBytesWithExit 'control bytes timeout' $Pwsh @('-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 30') -TimeoutMilliseconds 2000 }
catch { $TimeoutThrew = $_.Exception.Message -match 'timed out after 2000ms' }
$BytesEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
$BytesStart = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control (([Text.Encoding]::UTF8.GetString($BytesResult.Bytes).Trim() -ceq 'byte-hi') -and $BytesResult.ExitCode -eq 0) 'stdout bytes and exit preserved'
Assert-Control $TimeoutThrew 'timeout still throws the original message after killing the child'
Assert-Control ($BytesEnd.id -ceq $BytesStart.id -and $BytesEnd.kind -ceq 'native-bytes' -and $BytesEnd.outcome -ceq 'error' -and -not [string]::IsNullOrEmpty($BytesEnd.error)) 'timeout recorded as error with the message, matched by id'

Write-Host 'control 17: every journal line is valid NDJSON with the required fields'
$AllLines = [IO.File]::ReadAllLines($LedgerPath) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
$JsonOk = $true
foreach ($Line in $AllLines) {
  try {
    $R = $Line | ConvertFrom-Json
    if ($R.event -ceq 'session' -and [string]::IsNullOrEmpty($R.session)) { $JsonOk = $false }
    if ($R.event -ceq 'start' -and ([string]::IsNullOrEmpty($R.id) -or [string]::IsNullOrEmpty($R.group) -or [string]::IsNullOrEmpty($R.kind) -or [string]::IsNullOrEmpty($R.executable) -or [string]::IsNullOrEmpty($R.timestamp))) { $JsonOk = $false }
    if ($R.event -ceq 'end' -and ([string]::IsNullOrEmpty($R.id) -or [string]::IsNullOrEmpty($R.outcome) -or $null -eq $R.elapsed_ms -or $null -eq $R.exit_code -or [string]::IsNullOrEmpty($R.executable))) { $JsonOk = $false }
  } catch { $JsonOk = $false }
}
Assert-Control ($JsonOk -and $AllLines.Count -gt 0) 'all lines parse; required fields (id, executable, outcome, elapsed, exit) present'

Remove-Item -LiteralPath $WorkDir -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "timing controls: $($script:Passed) passed, $($script:Failed) failed"
if ($script:Failed -gt 0) { exit 1 }
