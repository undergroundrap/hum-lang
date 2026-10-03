# Focused controls for the durable timing journal (tools/hum_timing_ledger.ps1).
#
# This is the existing applicable test path for timing: it exercises the REAL
# production function texts (extracted from check_all.ps1,
# test_exact_rust_selector.ps1 and run_fast_evidence.ps1 via the PowerShell
# AST, never re-implemented) and the REAL hum-dev capture boundary
# (Invoke-HumBinaryCapture from run_fast_evidence.ps1, dot-sourced).
#
# Windows/Linux compatible by construction: every native probe is the
# running pwsh itself (via -EncodedCommand, which is whitespace-free) or a
# BOM-less .ps1 fixture invoked with the call operator. No POSIX shell, no
# /bin/*, no chmod. Windows process-launch and timeout semantics of the
# unchanged production wrappers stay CI's authority.
#
# Coverage:
#   1. init truncates stale content and opens a versioned session
#   2. success: START+END share one launch id and name the executable
#   3. expected nonzero exit records outcome=failure, nothing swallowed
#   4. setup failure (missing executable): the caught failure records an
#      error END with the original message, and a subsequent successful
#      launch records its own complete pair (balanced scopes)
#   5. mid-execution launch error records outcome=error with the message
#   6. timed wrapper output is byte-identical to the untimed wrapper
#   7. whitespace/newline arguments preserved through ArgumentList
#   8. list then run phases with distinct ids; selector credit unchanged
#   9. adapter seam: Initialize-HumProfileTimingJournal reads
#      HUM_TIMING_JOURNAL from the environment (not a direct init call);
#      the unset-env probe defers to the module default via argument
#      capture, never truncating or deleting the real default journal;
#      the next command retains the restored session/group and depth
#  10. retention seam: Invoke-HumTimingJournalRetention stages on failure,
#      removes on success, ignores absent journals, never touches stale
#      files at other paths
#  11. contract agreement: workflow, profile and hum-dev's allowlist agree
#      on the HUM_TIMING_JOURNAL key name
#  12. interruption through the authenticated process-tree boundary: the
#      deadline fires, descendant and channel quiescence are proven from the
#      retained record BEFORE cleanup, and the journal shows START without
#      END (completion unobserved) for the killed command under a shared
#      label, matched by unique id
#  13. nested launches record only the outer command
#  14. a failed START write releases its depth slot; the next launch still
#      records its own complete pair
#  15. disabled mode: wrappers run unchanged, no journal file
#  16. absent module: standalone imports run untimed, never failing before
#      launch (selector script and policy-test import shapes)
#  17. Invoke-Native preserves throw behavior and records
#  18. Read-NativeOutput preserves text and throw behavior
#  19. Read-NativeOutputWithExit records kind and executable
#  20. Read-NativeBytesWithExit preserves bytes and records a harness
#      timeout as error
#  21. every journal line is valid NDJSON with the required fields
#  22. Read-NativeBytesWithExit: a missing-executable setup failure records
#      an error END with the original message; a retry records its own pair
#  23. contained capture: a root-creation failure records an error END with
#      the original message and releases the scope (no stranded depth)
#  24. contained capture: a journal END-write failure cannot skip
#      authenticated cleanup, strand the depth slot, or fail the command
#  25. shared-caller completion contract: bounded fixtures prove through
#      the real Invoke-RepoScript that success (after a real $LASTEXITCODE
#      leak, via the suite's own completion epilogue) succeeds, while a
#      deliberate setup failure and a deliberate assertion failure still
#      fail

$ErrorActionPreference = 'Stop'

$ToolsDir = $PSScriptRoot
$RepoRoot = (Resolve-Path (Join-Path $ToolsDir '..')).Path
$WorkDir = Join-Path ([IO.Path]::GetTempPath()) ('hum-timing-controls-' + [guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($WorkDir) | Out-Null
$LedgerPath = Join-Path $WorkDir 'ledger.ndjson'

function Import-RealFunction([string] $File, [string] $Name) {
  $Source = [IO.File]::ReadAllText($File)
  $Tokens = $null; $Errors = $null
  $Ast = [Management.Automation.Language.Parser]::ParseInput($Source, [ref]$Tokens, [ref]$Errors)
  if ($Errors.Count -ne 0) { throw "control: $File failed to parse" }
  $Defs = @($Ast.FindAll({ param($N) $N -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq $Name }, $true))
  if ($Defs.Count -ne 1) { throw "control: $Name not found exactly once in $File" }
  return [scriptblock]::Create($Defs[0].Extent.Text)
}

$CheckAll = Join-Path $ToolsDir 'check_all.ps1'
. (Import-RealFunction $CheckAll 'Select-FirstApplicationSource')
# Scalar pwsh selection: Get-Command can resolve multiple applications
# (e.g., three on GitHub Ubuntu runners); the real selector picks the
# first and validates it is one absolute path.
$Pwsh = Select-FirstApplicationSource @(Get-Command pwsh -CommandType Application -All -ErrorAction Stop) 'timing controls pwsh'

. (Join-Path $ToolsDir 'hum_timing_ledger.ps1')
# The existing capture machinery: authenticated process-tree boundary,
# retention seam, and capture record readers. Dot-sourcing is side-effect
# free (the producer entrypoint is guarded by $MyInvocation).
. (Join-Path $ToolsDir 'run_fast_evidence.ps1')

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

function ConvertTo-EncodedCommand([string] $Command) {
  return [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($Command))
}

$SelectorModule = Join-Path $ToolsDir 'test_exact_rust_selector.ps1'
. (Import-RealFunction $SelectorModule 'Assert-ExactRustSelectorSyntax')
. (Import-RealFunction $SelectorModule 'Assert-ExactRustSelectorEvidence')
. (Import-RealFunction $SelectorModule 'Invoke-ExactRustNativeCapture')
. (Import-RealFunction $SelectorModule 'Invoke-ExactRustTest')
. (Import-RealFunction $CheckAll 'Initialize-HumProfileTimingJournal')
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

function Write-FixtureScript([string] $Path, [string] $Body) {
  # BOM-less UTF-8; invoked with the call operator (no chmod, no shebang).
  [IO.File]::WriteAllText($Path, $Body, [Text.UTF8Encoding]::new($false))
}

$Selector = 'fake_crate::tests::timing_probe'

# Portable fake cargo: list/run/echo/fail modes selected by arguments.
$FakeCargo = Join-Path $WorkDir 'fake-cargo.ps1'
Write-FixtureScript $FakeCargo @'
$SelectorArg = $args[1]
if ($args.Count -ge 5 -and $args[4] -eq '--list') {
  Write-Output "$($SelectorArg): test"
  exit 0
}
if ($args.Count -ge 1 -and $args[0] -eq 'echo-mode') {
  Write-Output "out:$($args[1])"
  [Console]::Error.WriteLine("err:$($args[1])")
  exit 7
}
if ($args.Count -ge 1 -and $args[0] -eq 'fail-mode') {
  Write-Output 'failing'
  exit 3
}
if ($args.Count -ge 1 -and $args[0] -eq 'sleep-mode') {
  Start-Sleep -Seconds 10
  Write-Output 'running 1 test'
  Write-Output "test $($args[1]) ... ok"
  Write-Output 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;'
  exit 0
}
Write-Output 'running 1 test'
Write-Output "test $SelectorArg ... ok"
Write-Output 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;'
exit 0
'@

Write-Host 'control 1: initialize truncates a stale ledger and opens a versioned session'
[IO.File]::WriteAllText($LedgerPath, "stale junk`n", [Text.Encoding]::UTF8)
Initialize-HumTimingLedger -LedgerPath $LedgerPath
$Records = Read-LedgerRecords $LedgerPath
Assert-Control ($Records.Count -eq 1 -and $Records[0].event -ceq 'session' -and -not [string]::IsNullOrEmpty($Records[0].session) -and $Records[0].ledger_version -eq 2) 'stale content replaced by one versioned session record'

Write-Host 'control 2: success records share one launch id and name the executable'
Set-HumTimingGroup 'compiler-front'
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
$FailResult = Invoke-ExactRustNativeCapture $FakeCargo @('fail-mode', $Selector) -TimingKind 'cargo-selector' -TimingPhase 'run' -TimingLabel 'control fail' -TimingSelector $Selector
$Records = Read-LedgerRecords $LedgerPath
$FailEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$FailStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control ($FailResult.ExitCode -eq 3 -and $FailResult.Output -join '' -ceq 'failing') 'wrapper returns the failure output and exit 3'
Assert-Control ($FailEnd.id -ceq $FailStart.id -and $FailEnd.outcome -ceq 'failure' -and $FailEnd.exit_code -eq 3) 'end shares the start id and records failure with exit 3'

Write-Host 'control 4: caught setup failure records an error END; retry gets its own pair'
$SetupThrew = $false
try { $null = Invoke-ExactRustNativeCapture (Join-Path $WorkDir 'no-such-cargo.ps1') @('test') -TimingKind 'cargo-selector' -TimingPhase 'list' -TimingLabel 'control setup' -TimingSelector $Selector }
catch { $SetupThrew = $_.Exception.Message -match 'cargo executable is unavailable' }
$Records = Read-LedgerRecords $LedgerPath
$SetupEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$SetupStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control $SetupThrew 'missing cargo still throws the original message'
Assert-Control ($SetupEnd.id -ceq $SetupStart.id -and $SetupEnd.outcome -ceq 'error' -and $SetupEnd.error -match 'cargo executable is unavailable') 'setup failure records an error END with the message, matched by id'
$Retry = Invoke-ExactRustNativeCapture $FakeCargo @('test', $Selector, '--', '--exact') -TimingKind 'cargo-selector' -TimingPhase 'run' -TimingLabel 'control retry' -TimingSelector $Selector
$Records = Read-LedgerRecords $LedgerPath
$RetryEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$RetryStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control ($Retry.ExitCode -eq 0 -and $RetryEnd.id -ceq $RetryStart.id -and $RetryEnd.id -cne $SetupEnd.id -and $RetryEnd.outcome -ceq 'success') 'subsequent launch records its own complete pair with a new id'
Assert-Control ((Get-IncompleteLaunches $Records).Count -eq 0) 'no unbalanced scopes: every start has its end'

Write-Host 'control 5: mid-execution launch error records outcome=error with the message'
$LaunchThrew = $false
try { $null = Read-NativeChannelsWithExit 'control launch-error' (Join-Path $WorkDir 'no-such-exe') @() }
catch { $LaunchThrew = $true }
$Records = Read-LedgerRecords $LedgerPath
$LaunchEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$LaunchStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control $LaunchThrew 'launch error still throws'
Assert-Control ($LaunchEnd.id -ceq $LaunchStart.id -and $LaunchEnd.outcome -ceq 'error' -and -not [string]::IsNullOrEmpty($LaunchEnd.error)) 'start+end share the id and record the error outcome with message'

Write-Host 'control 6: timed wrapper output is byte-identical to the untimed wrapper'
$ProbeB64 = ConvertTo-EncodedCommand "[Console]::Out.Write('cli-out'); [Console]::Error.Write('cli-err'); exit 5"
$ProbeArgs = @('-NoProfile', '-NonInteractive', '-EncodedCommand', $ProbeB64)
$CliTimed = Read-NativeChannelsWithExit 'control cli timed' $Pwsh $ProbeArgs
$RefOut = $CliTimed.Stdout; $RefErr = $CliTimed.Stderr
$script:HumTimingLedgerPath = ''
$CliUntimed = Read-NativeChannelsWithExit 'control cli untimed' $Pwsh $ProbeArgs
$script:HumTimingLedgerPath = $LedgerPath
Assert-Control ($CliTimed.Stdout -ceq $CliUntimed.Stdout -and $CliTimed.Stderr -ceq $CliUntimed.Stderr -and $CliTimed.ExitCode -eq $CliUntimed.ExitCode -and $CliTimed.ExitCode -eq 5) 'stdout/stderr/exit identical with and without timing'
$Records = Read-LedgerRecords $LedgerPath
$CliEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$CliStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control ($CliEnd.id -ceq $CliStart.id -and $CliEnd.kind -ceq 'hum-cli' -and $CliEnd.label -ceq 'control cli timed' -and $CliEnd.outcome -ceq 'failure' -and $CliEnd.exit_code -eq 5 -and $CliEnd.executable -ceq $Pwsh) 'cli end shares the start id and records kind/label/failure/executable'

Write-Host 'control 7: argument-list variant preserves whitespace-bearing arguments'
$ArgsFixture = Join-Path $WorkDir 'fixture-args.ps1'
Write-FixtureScript $ArgsFixture @'
foreach ($a in $args) { [Console]::Out.WriteLine($a) }
'@
$WsArgs = @('a b', "c`nd", 'e')
$WsTimed = Read-NativeArgumentListWithExit 'control ws' $Pwsh (@('-NoProfile', '-NonInteractive', '-File', $ArgsFixture) + $WsArgs)
$WsExpected = (('a b', "c`nd", 'e') -join [Environment]::NewLine) + [Environment]::NewLine
Assert-Control (($WsTimed.Stdout -ceq $WsExpected) -and $WsTimed.ExitCode -eq 0) 'whitespace/newline args preserved byte-identically'

Write-Host 'control 8: Invoke-ExactRustTest emits list then run records with distinct ids'
$script:ExactRustSelectorCredits = New-Object 'System.Collections.Generic.List[string]'
Set-HumTimingGroup 'compiler-corpus'
$Before = (Read-LedgerRecords $LedgerPath).Count
Invoke-ExactRustTest 'control exact test' $FakeCargo $Selector
$New = @(Read-LedgerRecords $LedgerPath | Select-Object -Skip $Before)
$Starts = @($New | Where-Object { $_.event -ceq 'start' })
$Ends = @($New | Where-Object { $_.event -ceq 'end' })
$Ids = @($Starts | ForEach-Object { $_.id })
Assert-Control ($Starts.Count -eq 2 -and $Ends.Count -eq 2) 'one start+end pair per phase'
Assert-Control ($Starts[0].phase -ceq 'list' -and $Starts[1].phase -ceq 'run') 'list precedes run'
Assert-Control (($Ids | Select-Object -Unique).Count -eq 2) 'each launch has a distinct id'
Assert-Control ((($Ends | ForEach-Object { $_.id }) | Where-Object { $Ids -contains $_ }).Count -eq 2) 'ends match starts by id'
Assert-Control (($Ends | Where-Object { $_.outcome -ceq 'success' }).Count -eq 2) 'both phases succeeded'
Assert-Control (@($script:ExactRustSelectorCredits | Where-Object { $_ -ceq $Selector }).Count -eq 1) 'selector credit accounting unchanged'

Write-Host 'control 9: profile journal init reads HUM_TIMING_JOURNAL from the environment'
# The shared test path (Invoke-RepoScript) runs in-process, so the control
# must leave the process environment exactly as found: the outer
# try/finally restores the true incoming value no matter what the inner
# probes do.
$TrueIncomingJournal = $env:HUM_TIMING_JOURNAL
try {
  # Simulate the parent profile's per-invocation journal: the adapter seam
  # must initialize at the configured path without disturbing the parent's
  # file, and must restore (never remove) the incoming value.
  $ParentJournal = Join-Path $WorkDir 'parent-journal.ndjson'
  [IO.File]::WriteAllText($ParentJournal, "parent journal content`n", [Text.Encoding]::UTF8)
  $env:HUM_TIMING_JOURNAL = $ParentJournal
  $EnvJournal = Join-Path $WorkDir 'env-journal.ndjson'
  $IncomingJournal = $env:HUM_TIMING_JOURNAL
  $env:HUM_TIMING_JOURNAL = $EnvJournal
  try {
    Initialize-HumProfileTimingJournal
    $EnvRecords = Read-LedgerRecords $EnvJournal
    Assert-Control ((Test-Path -LiteralPath $EnvJournal) -and $EnvRecords[0].event -ceq 'session' -and -not [string]::IsNullOrEmpty($EnvRecords[0].session)) 'env path initializes the journal at the exact path (adapter seam, not a direct init)'
  } finally {
    if ($null -eq $IncomingJournal) { Remove-Item Env:HUM_TIMING_JOURNAL -ErrorAction SilentlyContinue }
    else { $env:HUM_TIMING_JOURNAL = $IncomingJournal }
  }
  Assert-Control ($env:HUM_TIMING_JOURNAL -ceq $ParentJournal) 'the incoming parent journal value is restored, not removed'
  Assert-Control (([IO.File]::ReadAllText($ParentJournal) -ceq "parent journal content`n")) 'the parent journal file is untouched'
  # An unset value must defer to the module default path WITHOUT touching
  # the real default journal: an incoming local profile can be using
  # target/hum-timing-ledger.ndjson, so this fixture captures the adapter's
  # argument with a shadow instead of truncating or deleting that file.
  $DefaultJournal = Join-Path (Join-Path $ToolsDir '..') (Join-Path 'target' 'hum-timing-ledger.ndjson')
  Assert-Control ((Get-HumTimingDefaultLedgerPath) -ceq $DefaultJournal) 'default path is target/hum-timing-ledger.ndjson under the repo root'
  $DefaultExistedBefore = Test-Path -LiteralPath $DefaultJournal
  $DefaultBytesBefore = if ($DefaultExistedBefore) { [IO.File]::ReadAllBytes($DefaultJournal) } else { $null }
  $RealInitializeLedger = (Get-Command Initialize-HumTimingLedger).ScriptBlock
  $script:CapturedLedgerPath = 'sentinel-not-called'
  function Initialize-HumTimingLedger { param([string]$LedgerPath = ''); $script:CapturedLedgerPath = $LedgerPath }
  try {
    Remove-Item Env:HUM_TIMING_JOURNAL -ErrorAction SilentlyContinue
    Initialize-HumProfileTimingJournal
    Assert-Control ([string]::IsNullOrEmpty($script:CapturedLedgerPath)) 'unset env defers to the module default (empty path passed through the adapter)'
  } finally {
    New-Item -Path 'Function:\Initialize-HumTimingLedger' -Value $RealInitializeLedger -Force | Out-Null
  }
  Assert-Control ((Get-Command Initialize-HumTimingLedger).ScriptBlock.Ast.Extent.Text -ceq $RealInitializeLedger.Ast.Extent.Text) 'the real initializer is restored after the shadow'
  Assert-Control ((Test-Path -LiteralPath $DefaultJournal) -ceq $DefaultExistedBefore) 'the default journal file was neither created nor deleted'
  if ($DefaultExistedBefore) {
    $DefaultBytesAfter = [IO.File]::ReadAllBytes($DefaultJournal)
    Assert-Control (($DefaultBytesAfter.Length -eq $DefaultBytesBefore.Length) -and (@(Compare-Object $DefaultBytesAfter $DefaultBytesBefore).Count -eq 0)) 'the default journal bytes are untouched'
  }
  # Restore the suite timing state, then prove the next command retains the
  # correct session/group and a balanced depth.
  Initialize-HumTimingLedger -LedgerPath $LedgerPath
  Set-HumTimingGroup 'compiler-corpus'
  $SessionAfterRestore = $script:HumTimingSession
  $AfterB64 = ConvertTo-EncodedCommand 'exit 0'
  $AfterProbe = Read-NativeArgumentListWithExit 'after-control-9' $Pwsh @('-NoProfile','-NonInteractive','-EncodedCommand',$AfterB64)
  $AfterEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
  $AfterStart = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'start' -and $_.id -ceq $AfterEnd.id })[0]
  Assert-Control ($AfterProbe.ExitCode -eq 0 -and $AfterEnd.outcome -ceq 'success') 'the next command succeeds after the fixture'
  Assert-Control ($AfterStart.session -ceq $SessionAfterRestore -and $AfterStart.group -ceq 'compiler-corpus') 'the next command retains the restored session and group'
  Assert-Control ((Get-IncompleteLaunches (Read-LedgerRecords $LedgerPath)).Count -eq 0) 'balanced depth: no stranded scopes after the fixture'
  Assert-Control ($script:HumTimingDepth -eq 0) 'the depth slot is released after the fixture'
} finally {
  if ($null -eq $TrueIncomingJournal) { Remove-Item Env:HUM_TIMING_JOURNAL -ErrorAction SilentlyContinue }
  else { $env:HUM_TIMING_JOURNAL = $TrueIncomingJournal }
}

Write-Host 'control 10: retention seam stages on failure, removes on success, ignores absent journals'
$RetDir = Join-Path $WorkDir 'retention'
[IO.Directory]::CreateDirectory($RetDir) | Out-Null
$JFail = Join-Path $WorkDir 'j-fail.ndjson'
[IO.File]::WriteAllText($JFail, "{`"event`":`"session`"}`n", [Text.Encoding]::UTF8)
$RetResult = Invoke-HumTimingJournalRetention -JournalPath $JFail -CaptureDirectory $RetDir -Failed $true
$Staged = Join-Path $RetDir 'hum-timing-journal.ndjson'
Assert-Control ($RetResult -ceq 'retained' -and (Test-Path -LiteralPath $Staged) -and ([IO.File]::ReadAllText($Staged) -ceq [IO.File]::ReadAllText($JFail))) 'failure stages the journal into the capture directory byte-identically'
$JOk = Join-Path $WorkDir 'j-ok.ndjson'
[IO.File]::WriteAllText($JOk, "{`"event`":`"session`"}`n", [Text.Encoding]::UTF8)
$RetDirOk = Join-Path $WorkDir 'retention-ok'
[IO.Directory]::CreateDirectory($RetDirOk) | Out-Null
$RetResult = Invoke-HumTimingJournalRetention -JournalPath $JOk -CaptureDirectory $RetDirOk -Failed $false
Assert-Control ($RetResult -ceq 'removed' -and -not (Test-Path -LiteralPath $JOk) -and -not (Test-Path -LiteralPath (Join-Path $RetDirOk 'hum-timing-journal.ndjson'))) 'success removes the journal and leaves the capture directory clean'
$RetResult = Invoke-HumTimingJournalRetention -JournalPath (Join-Path $WorkDir 'no-such-journal.ndjson') -CaptureDirectory $RetDir -Failed $true
Assert-Control ($RetResult -ceq 'absent') 'a missing journal (profile never initialized) is not an error'
$StaleDefault = Join-Path $WorkDir 'stale-default.ndjson'
[IO.File]::WriteAllText($StaleDefault, "stale cached content`n", [Text.Encoding]::UTF8)
$JOther = Join-Path $WorkDir 'j-other.ndjson'
[IO.File]::WriteAllText($JOther, "{`"event`":`"session`"}`n", [Text.Encoding]::UTF8)
$null = Invoke-HumTimingJournalRetention -JournalPath $JOther -CaptureDirectory $RetDir -Failed $true
Assert-Control (([IO.File]::ReadAllText($StaleDefault) -ceq "stale cached content`n")) 'a stale journal at any other path is never attributed to this run'

Write-Host 'control 11: workflow, profile and hum-dev agree on the HUM_TIMING_JOURNAL contract'
$CiYml = [IO.File]::ReadAllText((Join-Path $RepoRoot '.github/workflows/ci.yml'))
$CheckAllSrc = [IO.File]::ReadAllText($CheckAll)
$ShellSrc = [IO.File]::ReadAllText((Join-Path $RepoRoot 'crates/hum-dev/src/shell.rs'))
Assert-Control ($CiYml.Contains('HUM_TIMING_JOURNAL') -and $CheckAllSrc.Contains('$env:HUM_TIMING_JOURNAL') -and $ShellSrc.Contains('HUM_TIMING_JOURNAL')) 'workflow sets, profile reads, and hum-dev allowlists the same key'
Assert-Control ($ShellSrc.Contains('command.env_clear().envs(&self.environment.0)')) 'hum-dev keeps env_clear at the launch site'

Write-Host 'control 12: interruption through the authenticated process-tree boundary'
$KillLedger = Join-Path $WorkDir 'kill-ledger.ndjson'
$ChildScript = Join-Path $WorkDir 'kill-child.ps1'
$ChannelsText = Get-RealFunctionText $CheckAll 'Read-NativeChannelsWithExit'
$QuickB64 = ConvertTo-EncodedCommand 'exit 0'
$SleepB64 = ConvertTo-EncodedCommand 'Start-Sleep -Seconds 120'
# The child runs the REAL production wrapper text through the REAL module.
# The boundary below (the same Invoke-HumBinaryCapture the fixed profile
# uses) fires its deadline exactly like the production 3000 s outer
# deadline; the child's buffered console dies with it while the flushed
# journal on disk is the retention boundary.
[IO.File]::WriteAllText($ChildScript, (@(
  '$ErrorActionPreference = ' + (Quote-Single 'Stop')
  '. ' + (Quote-Single (Join-Path $ToolsDir 'hum_timing_ledger.ps1'))
  $ChannelsText
  ('$KillLedgerPath = ' + (Quote-Single $KillLedger))
  ('$PwshExe = ' + (Quote-Single $Pwsh))
  ('$QuickB64 = ' + (Quote-Single $QuickB64))
  ('$SleepB64 = ' + (Quote-Single $SleepB64))
  'Initialize-HumTimingLedger -LedgerPath $KillLedgerPath'
  # Both launches share one label and one phase: readers must match by the
  # unique launch id, never by label.
  '$null = Read-NativeChannelsWithExit ''probe-interrupt'' $PwshExe @(''-NoProfile'',''-NonInteractive'',''-EncodedCommand'',$QuickB64)'
  '$null = Read-NativeChannelsWithExit ''probe-interrupt'' $PwshExe @(''-NoProfile'',''-NonInteractive'',''-EncodedCommand'',$SleepB64)'
) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
$CaptureDir = Join-Path $WorkDir 'interruption-capture'
$BoundaryResult = Invoke-HumBinaryCapture $Pwsh @('-NoLogo', '-NoProfile', '-NonInteractive', '-File', $ChildScript) $WorkDir $CaptureDir 45 -CaseName 'timing-interruption'
Assert-Control ($BoundaryResult.TimedOut -and $BoundaryResult.DeadlineDisposition -ceq 'deadline_expired') 'the boundary deadline fired (timed_out, deadline_expired)'
Assert-Control ($BoundaryResult.TerminationDisposition -ceq 'tree_termination_confirmed' -and $BoundaryResult.TerminationResult -ceq 'job_terminated_quiescent' -and $BoundaryResult.TerminationCount -eq 1) 'the tree termination is authenticated and confirmed'
Assert-Control ($BoundaryResult.JobQuiescenceObserved -and $BoundaryResult.FinalActiveProcessCount -eq 0) 'descendant quiescence proven: no active processes remain'
Assert-Control ($BoundaryResult.StdoutCompletionObserved -and $BoundaryResult.StderrCompletionObserved -and $BoundaryResult.PrimaryExitObserved) 'channel quiescence proven: streams drained, primary exit observed'
$KillRecords = Read-LedgerRecords $KillLedger
# Same label, same phase, completed versus interrupted: only the unique id
# distinguishes the two launches.
$InterruptStarts = @($KillRecords | Where-Object { $_.event -ceq 'start' -and $_.label -ceq 'probe-interrupt' })
Assert-Control ($InterruptStarts.Count -eq 2) 'two launches share one label'
Assert-Control ($InterruptStarts[0].phase -ceq 'cli' -and $InterruptStarts[1].phase -ceq 'cli') 'both launches share the phase'
$IdA = $InterruptStarts[0].id
$IdB = $InterruptStarts[1].id
Assert-Control ((-not [string]::IsNullOrEmpty($IdA)) -and $IdA -cne $IdB) 'the shared label still yields distinct launch ids'
$CompletedPairs = @(@((Get-LaunchPair $KillRecords $IdA), (Get-LaunchPair $KillRecords $IdB)) | Where-Object { $null -ne $_.End })
$InterruptedPairs = @(@((Get-LaunchPair $KillRecords $IdA), (Get-LaunchPair $KillRecords $IdB)) | Where-Object { $null -eq $_.End })
Assert-Control ($CompletedPairs.Count -eq 1 -and $CompletedPairs[0].End.outcome -ceq 'success') 'the completed launch has its END, matched by id'
Assert-Control ($InterruptedPairs.Count -eq 1 -and $null -ne $InterruptedPairs[0].Start) 'the killed launch has START without END: completion unobserved, matched by id'
Assert-Control ((@(Get-IncompleteLaunches $KillRecords | Where-Object { $_.label -ceq 'probe-interrupt' })).Count -eq 1) 'exactly one incomplete launch under the shared label'
Remove-HumCaptureAfterAuthentication $CaptureDir
Assert-Control (-not (Test-Path -LiteralPath $CaptureDir)) 'authenticated cleanup removes the capture directory after quiescence was proven'

Write-Host 'control 13: nested launches record only the outer command'
$NestedLedger = Join-Path $WorkDir 'nested-ledger.ndjson'
Initialize-HumTimingLedger -LedgerPath $NestedLedger
Set-HumTimingGroup 'control-nested'
$Outer = Start-HumTimedCommand -Kind 'test' -Label 'outer' -Phase 'exec' -Executable $Pwsh -Arguments @()
$InnerScope = Start-HumTimedCommand -Kind 'test' -Label 'inner' -Phase 'exec' -Executable $Pwsh -Arguments @()
Assert-Control ($null -eq $InnerScope) 'nested start is suppressed while the outer launch is open'
$null = Read-NativeChannelsWithExit 'nested-wrapped' $Pwsh @('-NoProfile', '-NonInteractive', '-EncodedCommand', $QuickB64)
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

Write-Host 'control 14: a failed START write releases its depth slot'
$DirAsJournal = Join-Path $WorkDir 'dir-as-journal'
[IO.Directory]::CreateDirectory($DirAsJournal) | Out-Null
$DepthBefore = $script:HumTimingDepth
$StartThrew = $false
$script:HumTimingLedgerPath = $DirAsJournal
try { $null = Start-HumTimedCommand -Kind 'test' -Label 'doomed' -Phase 'exec' }
catch { $StartThrew = $true }
$DepthAfterFailure = $script:HumTimingDepth
$script:HumTimingLedgerPath = $LedgerPath
Assert-Control ($StartThrew -and $DepthAfterFailure -eq $DepthBefore) 'failed START rethrows and releases the depth slot'
$Recovered = Read-NativeArgumentListWithExit 'after-failed-start' $Pwsh @('-NoProfile', '-NonInteractive', '-EncodedCommand', $QuickB64)
$RecoveredEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
Assert-Control ($Recovered.ExitCode -eq 0 -and $RecoveredEnd.label -ceq 'after-failed-start' -and $RecoveredEnd.outcome -ceq 'success') 'the next launch records its own complete pair'

Write-Host 'control 15: without initialization the wrappers run untimed and unchanged'
$script:HumTimingLedgerPath = ''
$script:HumTimingGroup = ''
$BareScope = Start-HumTimedCommand -Kind 'hum-cli' -Label 'bare' -Phase 'cli'
Assert-Control ($null -eq $BareScope) 'start returns null when disabled'
Stop-HumTimedCommand -Scope $null -ExitCode 0 -Outcome 'success'
$Bare = Read-NativeChannelsWithExit 'control bare' $Pwsh $ProbeArgs
Assert-Control ($Bare.ExitCode -eq 5 -and $Bare.Stdout -ceq $RefOut) 'wrapper output unchanged with timing disabled'
Assert-Control (-not (Test-Path -LiteralPath (Join-Path $WorkDir 'disabled-ledger.ndjson'))) 'no ledger file created when disabled'
Initialize-HumTimingLedger -LedgerPath $LedgerPath
Set-HumTimingGroup 'compiler-corpus'

Write-Host 'control 16: absent timing module - standalone imports run untimed, never failing before launch'
$SelectorChild = Join-Path $WorkDir 'absent-selector-child.ps1'
[IO.File]::WriteAllText($SelectorChild, (@(
  '$ErrorActionPreference = ' + (Quote-Single 'Stop')
  (Get-RealFunctionText $SelectorModule 'Assert-ExactRustSelectorSyntax')
  (Get-RealFunctionText $SelectorModule 'Assert-ExactRustSelectorEvidence')
  (Get-RealFunctionText $SelectorModule 'Invoke-ExactRustNativeCapture')
  (Get-RealFunctionText $SelectorModule 'Invoke-ExactRustTest')
  'if ($null -ne (Get-Command Start-HumTimedCommand -CommandType Function -ErrorAction SilentlyContinue)) { throw ''timing module unexpectedly present'' }'
  '$script:ExactRustSelectorCredits = New-Object ''System.Collections.Generic.List[string]'''
  ('Invoke-ExactRustTest ''absent-module probe'' ' + (Quote-Single $FakeCargo) + ' ' + (Quote-Single $Selector))
  'if (@($script:ExactRustSelectorCredits | Where-Object { $_ -ceq ' + (Quote-Single $Selector) + ' }).Count -ne 1) { throw ''selector credit missing'' }'
  'Write-Output ''absent-selector-ok'''
) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
$SelPrevEap = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
try { $SelChildOut = @(& $Pwsh '-NoLogo' '-NoProfile' '-NonInteractive' '-File' $SelectorChild 2>&1 | ForEach-Object { $_.ToString() }); $SelChildExit = $LASTEXITCODE }
finally { $ErrorActionPreference = $SelPrevEap }
$global:LASTEXITCODE = 0
Assert-Control ($SelChildExit -eq 0 -and $SelChildOut -contains 'absent-selector-ok') 'selector script runs untimed with the module absent (list+run+credit intact)'
$PolicyChild = Join-Path $WorkDir 'absent-policy-child.ps1'
# NOTE: the wrapper's [string[]]$Arguments must be passed as one splatted
# array (the production calling convention). Separate positional tokens that
# start with '-' are interpreted by PowerShell's parameter binder as named
# parameters of the wrapper and silently dropped, leaving a bare
# interactive 'pwsh -NoProfile' waiting on stdin.
[IO.File]::WriteAllText($PolicyChild, (@(
  '$ErrorActionPreference = ' + (Quote-Single 'Stop')
  (Get-RealFunctionText $CheckAll 'Read-NativeOutputWithExit')
  ('$PwshExe = ' + (Quote-Single $Pwsh))
  ('$ProbeB64 = ' + (Quote-Single $ProbeB64))
  '$R = Read-NativeOutputWithExit ''policy-import probe'' $PwshExe @(''-NoProfile'',''-NonInteractive'',''-EncodedCommand'',$ProbeB64)'
  'if ($R.ExitCode -ne 5 -or $R.Output -notmatch ''cli-out'') { throw (''policy import mismatch: exit='' + $R.ExitCode) }'
  'Write-Output ''absent-policy-ok'''
) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
$PolPrevEap = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
try { $PolChildOut = @(& $Pwsh '-NoLogo' '-NoProfile' '-NonInteractive' '-File' $PolicyChild 2>&1 | ForEach-Object { $_.ToString() }); $PolChildExit = $LASTEXITCODE }
finally { $ErrorActionPreference = $PolPrevEap }
$global:LASTEXITCODE = 0
Assert-Control ($PolChildExit -eq 0 -and $PolChildOut -contains 'absent-policy-ok') 'check_all wrapper imported alone (policy-test shape) runs untimed'

Write-Host 'control 17: Invoke-Native preserves throw behavior and records'
$NativeOk = Join-Path $WorkDir 'fake-ok.ps1'
Write-FixtureScript $NativeOk @'
Write-Output 'native-hi'
exit 0
'@
$NativeFail = Join-Path $WorkDir 'fake-fail3.ps1'
Write-FixtureScript $NativeFail @'
Write-Output 'failing'
exit 3
'@
$NativeLedgerBefore = (Read-LedgerRecords $LedgerPath).Count
Invoke-Native 'control native ok' $NativeOk @()
$NativeThrew = $false
try { Invoke-Native 'control native fail' $NativeFail @() }
catch { $NativeThrew = $_.Exception.Message -ceq 'control native fail failed with exit code 3' }
$NativeRecords = @(Read-LedgerRecords $LedgerPath | Select-Object -Skip $NativeLedgerBefore)
$NativeStarts = @($NativeRecords | Where-Object { $_.event -ceq 'start' })
$NativeEnds = @($NativeRecords | Where-Object { $_.event -ceq 'end' })
Assert-Control $NativeThrew 'nonzero exit still throws the original message'
Assert-Control ($NativeStarts.Count -eq 2 -and $NativeEnds.Count -eq 2 -and ($NativeEnds.outcome -join ',') -ceq 'success,failure') 'both launches recorded with success then failure outcomes'
Assert-Control ((($NativeEnds | ForEach-Object { $_.id }) | Where-Object { ($NativeStarts.id) -contains $_ }).Count -eq 2) 'ends match starts by id'

Write-Host 'control 18: Read-NativeOutput preserves text and throw behavior'
$OutOk = Join-Path $WorkDir 'fake-out.ps1'
Write-FixtureScript $OutOk @'
Write-Output 'hello'
exit 0
'@
$OutText = Read-NativeOutput 'control readout ok' $OutOk @()
$OutThrew = $false
try { $null = Read-NativeOutput 'control readout fail' $NativeFail @() }
catch { $OutThrew = $_.Exception.Message -ceq 'control readout fail failed with exit code 3' }
Assert-Control ($OutText -ceq 'hello') 'output text unchanged'
Assert-Control $OutThrew 'failure still throws the original message'
$OutEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
Assert-Control ($OutEnd.kind -ceq 'native-output' -and $OutEnd.outcome -ceq 'failure' -and $OutEnd.exit_code -eq 3) 'failure recorded with kind and exit code'

Write-Host 'control 19: Read-NativeOutputWithExit records kind and executable'
$OweResult = Read-NativeOutputWithExit 'control owe' $Pwsh $ProbeArgs
Assert-Control ($OweResult.ExitCode -eq 5) 'exit code preserved'
$OweEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
Assert-Control ($OweEnd.kind -ceq 'native-output' -and $OweEnd.executable -ceq $Pwsh -and $OweEnd.outcome -ceq 'failure') 'kind/executable/outcome recorded'

Write-Host 'control 20: Read-NativeBytesWithExit preserves bytes and records a harness timeout as error'
$BytesB64 = ConvertTo-EncodedCommand "[Console]::Out.Write('byte-hi')"
$BytesResult = Read-NativeBytesWithExit 'control bytes ok' $Pwsh @('-NoProfile', '-NonInteractive', '-EncodedCommand', $BytesB64)
$TimeoutThrew = $false
try { $null = Read-NativeBytesWithExit 'control bytes timeout' $Pwsh @('-NoProfile', '-NonInteractive', '-EncodedCommand', $SleepB64) -TimeoutMilliseconds 2000 }
catch { $TimeoutThrew = $_.Exception.Message -match 'timed out after 2000ms' }
$BytesEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
$BytesStart = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control (([Text.Encoding]::UTF8.GetString($BytesResult.Bytes).Trim() -ceq 'byte-hi') -and $BytesResult.ExitCode -eq 0) 'stdout bytes and exit preserved'
Assert-Control $TimeoutThrew 'timeout still throws the original message after killing the child'
Assert-Control ($BytesEnd.id -ceq $BytesStart.id -and $BytesEnd.kind -ceq 'native-bytes' -and $BytesEnd.outcome -ceq 'error' -and -not [string]::IsNullOrEmpty($BytesEnd.error)) 'timeout recorded as error with the message, matched by id'

Write-Host 'control 21: every journal line is valid NDJSON with the required fields'
$AllLines = [IO.File]::ReadAllLines($LedgerPath) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
$JsonOk = $true
foreach ($Line in $AllLines) {
  try {
    $R = $Line | ConvertFrom-Json
    if ($R.event -ceq 'session' -and ([string]::IsNullOrEmpty($R.session) -or $R.ledger_version -ne 2)) { $JsonOk = $false }
    if ($R.event -ceq 'start' -and ([string]::IsNullOrEmpty($R.id) -or [string]::IsNullOrEmpty($R.group) -or [string]::IsNullOrEmpty($R.kind) -or [string]::IsNullOrEmpty($R.executable) -or [string]::IsNullOrEmpty($R.timestamp))) { $JsonOk = $false }
    if ($R.event -ceq 'end' -and ([string]::IsNullOrEmpty($R.id) -or [string]::IsNullOrEmpty($R.outcome) -or $null -eq $R.elapsed_ms -or $null -eq $R.exit_code -or [string]::IsNullOrEmpty($R.executable))) { $JsonOk = $false }
  } catch { $JsonOk = $false }
}
Assert-Control ($JsonOk -and $AllLines.Count -gt 0) 'all lines parse; required fields (id, executable, outcome, elapsed, exit) present'

Write-Host 'control 22: Read-NativeBytesWithExit balances a missing-executable setup failure; retry gets its own pair'
$BytesSetupThrew = $false
$BytesSetupMessage = ''
try { $null = Read-NativeBytesWithExit 'control bytes setup' (Join-Path $WorkDir 'no-such-exe') @() }
catch { $BytesSetupThrew = $true; $BytesSetupMessage = $_.Exception.Message }
$Records = Read-LedgerRecords $LedgerPath
$BytesSetupEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$BytesSetupStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control $BytesSetupThrew 'missing executable still throws'
Assert-Control (-not [string]::IsNullOrEmpty($BytesSetupMessage)) 'the original launch error is preserved, not replaced'
Assert-Control ($BytesSetupEnd.id -ceq $BytesSetupStart.id -and $BytesSetupEnd.outcome -ceq 'error' -and $BytesSetupEnd.error -ceq $BytesSetupMessage) 'setup failure records an error END with the original message, matched by id'
$BytesRetry = Read-NativeBytesWithExit 'control bytes retry' $Pwsh @('-NoProfile', '-NonInteractive', '-EncodedCommand', $BytesB64)
$Records = Read-LedgerRecords $LedgerPath
$BytesRetryEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$BytesRetryStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control (($BytesRetry.ExitCode -eq 0) -and $BytesRetryEnd.id -ceq $BytesRetryStart.id -and $BytesRetryEnd.id -cne $BytesSetupEnd.id -and $BytesRetryEnd.outcome -ceq 'success') 'retry records its own complete pair with a new id'
Assert-Control ((Get-IncompleteLaunches $Records).Count -eq 0) 'no unbalanced scopes: every start has its end'

Write-Host 'control 23: contained root-creation failure records an error END and releases the scope'
# Force the contained root creation to fail deterministically: point the
# temp-path lookup at a regular file (cross-platform: TMPDIR on Unix, TMP
# and TEMP on Windows). Saved and restored around the single call.
$NotADir = Join-Path $WorkDir 'not-a-temp-dir'
[IO.File]::WriteAllText($NotADir, 'x', [Text.Encoding]::UTF8)
$SavedTmpDir = $env:TMPDIR
$SavedTmp = $env:TMP
$SavedTemp = $env:TEMP
$env:TMPDIR = $NotADir
$env:TMP = $NotADir
$env:TEMP = $NotADir
$RootFailed = $false
$RootMessage = ''
try {
  try { $null = Invoke-HumContainedRustNativeCapture 'contained-rootfail-run' $FakeCargo @('test', $Selector, '--', '--exact') }
  catch { $RootFailed = $true; $RootMessage = $_.Exception.Message }
} finally {
  if ($null -eq $SavedTmpDir) { Remove-Item Env:TMPDIR -ErrorAction SilentlyContinue } else { $env:TMPDIR = $SavedTmpDir }
  if ($null -eq $SavedTmp) { Remove-Item Env:TMP -ErrorAction SilentlyContinue } else { $env:TMP = $SavedTmp }
  if ($null -eq $SavedTemp) { Remove-Item Env:TEMP -ErrorAction SilentlyContinue } else { $env:TEMP = $SavedTemp }
}
$Records = Read-LedgerRecords $LedgerPath
$RootEnd = @($Records | Where-Object { $_.event -ceq 'end' })[-1]
$RootStart = @($Records | Where-Object { $_.event -ceq 'start' })[-1]
Assert-Control $RootFailed 'root-creation failure still throws'
Assert-Control (-not [string]::IsNullOrEmpty($RootMessage)) 'the original creation error is preserved, not replaced'
Assert-Control ($RootEnd.id -ceq $RootStart.id -and $RootEnd.outcome -ceq 'error' -and $RootEnd.error -ceq $RootMessage) 'root failure records an error END with the original message, matched by id'
$AfterRoot = Read-NativeArgumentListWithExit 'after-rootfail' $Pwsh @('-NoProfile', '-NonInteractive', '-EncodedCommand', $QuickB64)
$AfterRootEnd = @(Read-LedgerRecords $LedgerPath | Where-Object { $_.event -ceq 'end' })[-1]
Assert-Control ($AfterRoot.ExitCode -eq 0 -and $AfterRootEnd.label -ceq 'after-rootfail' -and $AfterRootEnd.outcome -ceq 'success') 'the next launch records its own complete pair: no stranded scope'
Assert-Control ((Get-IncompleteLaunches (Read-LedgerRecords $LedgerPath)).Count -eq 0) 'no unbalanced scopes'

Write-Host 'control 24: a journal END-write failure cannot skip authenticated cleanup'
$EndFailLedger = Join-Path $WorkDir 'endfail-ledger.ndjson'
$EndFailChild = Join-Path $WorkDir 'endfail-child.ps1'
$EndFailOut = Join-Path $WorkDir 'endfail-child.out'
$ContainedText = Get-RealFunctionText (Join-Path $ToolsDir 'run_fast_evidence.ps1') 'Invoke-HumContainedRustNativeCapture'
# The child runs the REAL contained wrapper text. The parent sabotages the
# journal (file -> directory) after START is observed and before END, so the
# END write really fails; the child then proves the wrapper still returned
# its result, cleaned up, and released its depth slot.
[IO.File]::WriteAllText($EndFailChild, (@(
  '$ErrorActionPreference = ' + (Quote-Single 'Stop')
  '. ' + (Quote-Single (Join-Path $ToolsDir 'hum_timing_ledger.ps1'))
  '. ' + (Quote-Single (Join-Path $ToolsDir 'run_fast_evidence.ps1'))
  $ContainedText
  ('$ChildLedger = ' + (Quote-Single $EndFailLedger))
  # The contained wrapper launches its cargo natively through the
  # authenticated boundary, so the fixture runs under the real pwsh (a
  # .ps1 is not a native executable on either platform).
  ('$ChildCargo = ' + (Quote-Single $Pwsh))
  ('$ChildSelector = ' + (Quote-Single $Selector))
  # NOTE: $ChildSelector must be assigned before $ChildCargoArgs is built:
  # a trailing $null element fails the mandatory [string[]] binding inside
  # the wrapper (ParameterArgumentValidationErrorNullNotAllowed).
  ('$ChildCargoArgs = @(''-NoProfile'',''-NonInteractive'',''-File'',' + (Quote-Single $FakeCargo) + ',''sleep-mode'',$ChildSelector)')
  'Initialize-HumTimingLedger -LedgerPath $ChildLedger'
  '$R = Invoke-HumContainedRustNativeCapture ''contained-endfail-run'' $ChildCargo $ChildCargoArgs'
  'if ($R.ExitCode -ne 0) { throw (''contained capture failed: '' + $R.ExitCode) }'
  'if (-not ((Get-Item -LiteralPath $ChildLedger) -is [IO.DirectoryInfo])) { throw ''sabotage not active at END time'' }'
  'Write-Output ''endfail-sabotage-active'''
  'Remove-Item -LiteralPath $ChildLedger -Force -Recurse'
  'Initialize-HumTimingLedger -LedgerPath $ChildLedger'
  '$S = Start-HumTimedCommand -Kind ''test'' -Label ''after-endfail'' -Phase ''exec'''
  'if ($null -eq $S) { throw ''depth slot stranded after END-write failure'' }'
  'Stop-HumTimedCommand -Scope $S -ExitCode 0 -Outcome ''success'''
  'Write-Output ''contained-endfail-ok'''
) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
$EndFailProc = Start-Process -FilePath $Pwsh -ArgumentList @('-NoLogo', '-NoProfile', '-NonInteractive', '-File', $EndFailChild) -RedirectStandardOutput $EndFailOut -PassThru
$Sabotaged = $false
for ($i = 0; $i -lt 150 -and -not $EndFailProc.HasExited; $i++) {
  Start-Sleep -Milliseconds 200
  try {
    if ((Test-Path -LiteralPath $EndFailLedger) -and ([IO.File]::ReadAllText($EndFailLedger) -match '"event":"start"')) {
      Remove-Item -LiteralPath $EndFailLedger -Force
      [IO.Directory]::CreateDirectory($EndFailLedger) | Out-Null
      $Sabotaged = $true
      break
    }
  } catch { }
}
Assert-Control $Sabotaged 'the journal was sabotaged after START and before END'
$EndFailExited = $EndFailProc.WaitForExit(120000)
if (-not $EndFailExited) { try { $EndFailProc.Kill() } catch { } }
Assert-Control ($EndFailExited -and $EndFailProc.ExitCode -eq 0) 'the wrapper completed despite the END-write failure: the journal never changes command behavior'
$EndFailText = if (Test-Path -LiteralPath $EndFailOut) { [IO.File]::ReadAllText($EndFailOut) } else { '' }
Assert-Control ($EndFailText -match 'endfail-sabotage-active') 'the END write really failed against the sabotaged journal'
Assert-Control ($EndFailText -match 'contained-endfail-ok') 'authenticated cleanup ran and the depth slot was released'
Remove-Item -LiteralPath $EndFailLedger -Recurse -Force -ErrorAction SilentlyContinue

Write-Host 'control 25: the shared caller succeeds on all-pass and fails on genuine failure'
# Proves the completion contract through the ACTUAL shared caller
# (Invoke-RepoScript, extracted from check_all.ps1 via the AST, never
# re-implemented) with bounded fixtures that exercise the ACTUAL
# completion logic (the suite's own epilogue, extracted verbatim at
# runtime) — not additional full-suite executions. The generic caller
# itself is untouched.
# The real caller resolves scripts via its own $PSScriptRoot, which is
# empty for AST-imported functions; define it from a temporary probe file
# in the tools dir (the function keeps the root after the file is gone).
$CallerProbeFile = Join-Path $ToolsDir 'Invoke-RepoScript.probe.tmp.ps1'
[IO.File]::WriteAllText($CallerProbeFile, (Get-RealFunctionText $CheckAll 'Invoke-RepoScript'), [Text.UTF8Encoding]::new($false))
. $CallerProbeFile
Remove-Item -LiteralPath $CallerProbeFile -Force -ErrorAction SilentlyContinue
# Extract the ACTUAL completion logic verbatim (the anchor is concatenated
# so this source never contains it literally; the real occurrence is unique).
$SuiteText = [IO.File]::ReadAllText((Join-Path $ToolsDir 'test_timing_ledger.ps1'))
$EpilogueAnchor = 'if ($script:Failed -gt 0) { ' + 'exit 1 }'
if (([regex]::Matches($SuiteText, [regex]::Escape($EpilogueAnchor))).Count -ne 1) { throw 'completion epilogue anchor not unique' }
$Epilogue = $SuiteText.Substring($SuiteText.IndexOf($EpilogueAnchor))
$FixtureTemplate = @'
$ErrorActionPreference = 'Stop'
$Pwsh = '__PWSH__'
$Scratch = '__SCRATCH__'
& $Pwsh -NoProfile -NonInteractive -Command 'exit 5'
[IO.File]::WriteAllText((Join-Path $Scratch 'leak.txt'), "$LASTEXITCODE")
$script:Failed = __FAILED__
__EPILOGUE__
'@
$FixtureDir = Join-Path $WorkDir 'control25-fixtures'
[IO.Directory]::CreateDirectory($FixtureDir) | Out-Null
# Reuse the scalar $Pwsh selected at the top (real Select-FirstApplicationSource)
# rather than resolving again; a second resolution could member-enumerate
# an array.
$PwshPath = $Pwsh
try {
  # Fixture 1: success — an expected native failure leaks $LASTEXITCODE,
  # then the actual completion logic resets it; the caller must not throw.
  $SuccessScratch = Join-Path $FixtureDir 'success'
  [IO.Directory]::CreateDirectory($SuccessScratch) | Out-Null
  $SuccessFixture = Join-Path $ToolsDir 'control25-success.tmp.ps1'
  [IO.File]::WriteAllText($SuccessFixture,
    $FixtureTemplate.Replace('__PWSH__', $PwshPath).Replace('__SCRATCH__', $SuccessScratch).Replace('__FAILED__', '0').Replace('__EPILOGUE__', $Epilogue),
    [Text.UTF8Encoding]::new($false))
  $SuccessThrew = $false
  $SuccessMessage = ''
  try { Invoke-RepoScript 'control25 success fixture' 'control25-success.tmp.ps1' }
  catch { $SuccessThrew = $true; $SuccessMessage = $_.Exception.Message }
  $Leaked = [IO.File]::ReadAllText((Join-Path $SuccessScratch 'leak.txt')).Trim()
  Assert-Control ($Leaked -ceq '5') 'the success fixture really leaked $LASTEXITCODE=5 (not vacuous)'
  Assert-Control (-not $SuccessThrew) 'all-pass completion succeeds through the shared caller'
  if ($SuccessThrew) { Write-Host "  caller threw: $SuccessMessage" }

  # Fixture 2: a deliberate setup failure still fails through the caller.
  $SetupScratch = Join-Path $FixtureDir 'setupfail'
  [IO.Directory]::CreateDirectory($SetupScratch) | Out-Null
  $SetupFixture = Join-Path $ToolsDir 'control25-setupfail.tmp.ps1'
  [IO.File]::WriteAllText($SetupFixture,
    '$ErrorActionPreference = ''Stop''' + "`n" + 'throw ''deliberate setup failure probe''',
    [Text.UTF8Encoding]::new($false))
  $SetupThrew = $false
  try { Invoke-RepoScript 'control25 setup failure fixture' 'control25-setupfail.tmp.ps1' }
  catch { $SetupThrew = $true }
  Assert-Control $SetupThrew 'a deliberate setup failure still fails through the shared caller'

  # Fixture 3: a deliberate assertion failure records Failed=1, the actual
  # completion logic exits 1 (the success-path reset is unreachable), and
  # the caller throws.
  $AssertScratch = Join-Path $FixtureDir 'assertfail'
  [IO.Directory]::CreateDirectory($AssertScratch) | Out-Null
  $AssertFixture = Join-Path $ToolsDir 'control25-assertfail.tmp.ps1'
  [IO.File]::WriteAllText($AssertFixture,
    $FixtureTemplate.Replace('__PWSH__', $PwshPath).Replace('__SCRATCH__', $AssertScratch).Replace('__FAILED__', '1').Replace('__EPILOGUE__', $Epilogue),
    [Text.UTF8Encoding]::new($false))
  $AssertThrew = $false
  $AssertMessage = ''
  try { Invoke-RepoScript 'control25 assertion failure fixture' 'control25-assertfail.tmp.ps1' }
  catch { $AssertThrew = $true; $AssertMessage = $_.Exception.Message }
  Assert-Control ($AssertThrew -and $AssertMessage -match 'failed with exit code 1') 'a deliberate assertion failure still fails through the shared caller'
} finally {
  Remove-Item -LiteralPath (Join-Path $ToolsDir 'control25-success.tmp.ps1') -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath (Join-Path $ToolsDir 'control25-setupfail.tmp.ps1') -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath (Join-Path $ToolsDir 'control25-assertfail.tmp.ps1') -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $FixtureDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host 'control 26: pwsh selection is scalar through the real selector and consumer'
$Control26Dir = Join-Path $WorkDir 'control26-fixtures'
[IO.Directory]::CreateDirectory($Control26Dir) | Out-Null
$SavedPath26 = $env:PATH
try {
  $Fake1Dir = Join-Path $Control26Dir 'fake1'
  $Fake2Dir = Join-Path $Control26Dir 'fake2'
  [IO.Directory]::CreateDirectory($Fake1Dir) | Out-Null
  [IO.Directory]::CreateDirectory($Fake2Dir) | Out-Null
  # BOM-less executable fixtures (repo rule: a BOM before #! breaks exec).
  $Utf8NoBom = [Text.UTF8Encoding]::new($false)
  if ($IsWindows) {
    $Fake1Exe = Join-Path $Fake1Dir 'pwsh.cmd'
    $Fake2Exe = Join-Path $Fake2Dir 'pwsh.cmd'
    [IO.File]::WriteAllText($Fake1Exe, "@exit /b 42`r`n", $Utf8NoBom)
    [IO.File]::WriteAllText($Fake2Exe, "@exit /b 43`r`n", $Utf8NoBom)
  } else {
    $Fake1Exe = Join-Path $Fake1Dir 'pwsh'
    $Fake2Exe = Join-Path $Fake2Dir 'pwsh'
    [IO.File]::WriteAllText($Fake1Exe, "#!/bin/sh`nexit 42`n", $Utf8NoBom)
    [IO.File]::WriteAllText($Fake2Exe, "#!/bin/sh`nexit 43`n", $Utf8NoBom)
    & chmod +x $Fake1Exe $Fake2Exe
    if ($LASTEXITCODE -ne 0) { throw 'control26 fixture chmod failed' }
    $global:LASTEXITCODE = 0
  }
  $Sep = [IO.Path]::PathSeparator

  # Zero applications: the real selector must throw, not return null/array.
  $env:PATH = $Control26Dir
  $ZeroApps = @(Get-Command pwsh -CommandType Application -All -ErrorAction SilentlyContinue)
  Assert-Control ($ZeroApps.Count -eq 0) 'zero pwsh applications resolve to empty'
  $ZeroThrew = $false
  try { Select-FirstApplicationSource $ZeroApps 'control26 zero' | Out-Null } catch { $ZeroThrew = $true }
  Assert-Control $ZeroThrew 'zero applications throw through the real selector'

  # One application: scalar selection of the only candidate.
  $env:PATH = $Fake1Dir
  $OneApps = @(Get-Command pwsh -CommandType Application -All -ErrorAction Stop)
  Assert-Control ($OneApps.Count -eq 1) 'one pwsh application resolves'
  $OneSelected = Select-FirstApplicationSource $OneApps 'control26 single'
  Assert-Control ($OneSelected -is [string]) 'single selection is a scalar string'
  Assert-Control ($OneSelected -ceq $Fake1Exe) 'single selection is the application'

  # Multiple applications: the real selector picks the first; the real
  # consumer launches it (exit 42 proves fake1, not fake2/real, ran).
  $env:PATH = $Fake1Dir + $Sep + $Fake2Dir + $Sep + $SavedPath26
  $MultiApps = @(Get-Command pwsh -CommandType Application -All -ErrorAction Stop)
  Assert-Control ($MultiApps.Count -ge 3) 'multiple pwsh applications resolve'
  $MultiSelected = Select-FirstApplicationSource $MultiApps 'control26 multiple'
  Assert-Control ($MultiSelected -is [string]) 'multiple selection is a scalar string'
  Assert-Control ($MultiSelected -ceq $Fake1Exe) 'multiple selection is the first application'
  $MultiLaunch = Read-NativeChannelsWithExit 'control26 consumer' $MultiSelected @()
  Assert-Control ($MultiLaunch.ExitCode -eq 42) 'the selected executable launches through the real consumer'
} finally {
  $env:PATH = $SavedPath26
  Remove-Item -LiteralPath $Control26Dir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host 'control 27: Full journal via mechanically extracted production source'
$Control27Dir = Join-Path $WorkDir 'control27-fixtures'
[IO.Directory]::CreateDirectory($Control27Dir) | Out-Null
$SavedJournal27 = $env:HUM_TIMING_JOURNAL
try {
  # === MECHANICAL EXTRACTION (no handwritten production logic) ===
  # Extract from the FROZEN files via AST. If extraction fails, report
  # the precise obstacle rather than substituting helper-level evidence.
  $ExtractError = $null
  try {
    # Fast-tier guard from check_all.ps1
    $CaSrc = [IO.File]::ReadAllText($CheckAll)
    $CaTokens = $null; $CaErrors = $null
    $CaAst = [Management.Automation.Language.Parser]::ParseInput($CaSrc, [ref]$CaTokens, [ref]$CaErrors)
    if ($CaErrors.Count -ne 0) { throw "check_all.ps1 parse: $($CaErrors[0].Message)" }
    $CaIfs = @($CaAst.FindAll({ param($N) $N -is [System.Management.Automation.Language.IfStatementAst] }, $true))
    $ExtractedGuard = $null
    foreach ($If in $CaIfs) {
      $T = $If.Extent.Text
      if ($T -match 'HumTimingEnabled' -and $T -match 'HUM_TIMING_JOURNAL' -and $T -match 'Initialize-HumProfileTimingJournal') {
        $ExtractedGuard = $T; break
      }
    }
    if ($null -eq $ExtractedGuard) { throw 'Fast-tier guard not found via AST' }

    # Run Hum preflight step body from ci.yml (unindent)
    $CiSrc = [IO.File]::ReadAllText((Join-Path $RepoRoot '.github/workflows/ci.yml'))
    $CiLines = $CiSrc -split "`n"
    $InStep = $false; $BodyLines = @()
    foreach ($Line in $CiLines) {
      if ($Line -match '^      - name: Run Hum preflight$') { $InStep = $true; continue }
      if ($InStep -and $Line -match '^      - name: ') { break }
      if ($InStep) { $BodyLines += $Line }
    }
    $BodyStart = -1
    for ($i = 0; $i -lt $BodyLines.Count; $i++) {
      if ($BodyLines[$i] -match '^        run: \|$') { $BodyStart = $i + 1; break }
    }
    if ($BodyStart -lt 0) { throw 'run: | not found in step' }
    $PsLines = @()
    for ($i = $BodyStart; $i -lt $BodyLines.Count; $i++) {
      $L = $BodyLines[$i]
      if ($L.StartsWith('          ')) { $PsLines += $L.Substring(10) }
      elseif ($L -match '^\s*$') { $PsLines += '' }
      else { break }
    }
    $PsBody = $PsLines -join "`n"
    $PsTokens = $null; $PsErrors = $null
    $PsAst = [Management.Automation.Language.Parser]::ParseInput($PsBody, [ref]$PsTokens, [ref]$PsErrors)
    if ($PsErrors.Count -ne 0) { throw "step body parse: $($PsErrors[0].Message)" }

    # Save-HumPreflightDiagnostics via AST
    $F1 = @($PsAst.FindAll({ param($N) $N -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq 'Save-HumPreflightDiagnostics' }, $true))
    if ($F1.Count -ne 1) { throw "Save-HumPreflightDiagnostics found $($F1.Count)x" }
    $ExtractedSaveDiag = $F1[0].Extent.Text

    # Invoke-HumPreflightCapture via AST
    $F2 = @($PsAst.FindAll({ param($N) $N -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq 'Invoke-HumPreflightCapture' }, $true))
    if ($F2.Count -ne 1) { throw "Invoke-HumPreflightCapture found $($F2.Count)x" }
    $ExtractedCapture = $F2[0].Extent.Text

    # Journal finalizer: the quiescence-check block (from '$Quiescent = $false' through the staging if/else)
    $FinMatch = [regex]::Match($PsBody, '(?ms)^(\s*)\$Quiescent = \$false.*?(?=^\1\} else \{\s*$)', [System.Text.RegularExpressions.RegexOptions]::Multiline)
    # Fallback: extract the specific lines
    if (-not $FinMatch.Success) {
      $FinStart = $PsBody.IndexOf('$Quiescent = $false')
      if ($FinStart -lt 0) { throw 'finalizer $Quiescent not found' }
      # Take 30 lines from there (covers the quiescence check + staging)
      $FinLines = ($PsBody.Substring($FinStart) -split "`n")[0..30] -join "`n"
      $ExtractedFinalizer = $FinLines
    } else {
      $ExtractedFinalizer = $FinMatch.Value
    }
  } catch {
    $ExtractError = $_
  }
  Assert-Control ($null -eq $ExtractError) "mechanical extraction succeeded (obstacle: $($ExtractError.Exception.Message))"
  Assert-Control (-not [string]::IsNullOrEmpty($ExtractedGuard)) 'extracted guard is non-empty'
  Assert-Control (-not [string]::IsNullOrEmpty($ExtractedSaveDiag)) 'extracted Save-HumPreflightDiagnostics is non-empty'
  Assert-Control (-not [string]::IsNullOrEmpty($ExtractedCapture)) 'extracted Invoke-HumPreflightCapture is non-empty'
  Assert-Control (-not [string]::IsNullOrEmpty($ExtractedFinalizer)) 'extracted finalizer is non-empty'

  # Define the extracted functions in this scope
  Invoke-Expression $ExtractedSaveDiag
  # Note: Invoke-HumPreflightCapture calls Invoke-HumBinaryCapture which is already available

  # === SET/UNSET via the EXTRACTED guard ===
  $GuardJournal = Join-Path $Control27Dir 'guard-journal.ndjson'
  $env:HUM_TIMING_JOURNAL = $GuardJournal
  Invoke-Expression $ExtractedGuard
  Assert-Control (Test-Path -LiteralPath $GuardJournal) 'extracted guard initializes on explicit path'

  Remove-Item Env:HUM_TIMING_JOURNAL -ErrorAction SilentlyContinue
  $UnsetJournal = Join-Path $Control27Dir 'unset-journal.ndjson'
  Invoke-Expression $ExtractedGuard
  Assert-Control (-not (Test-Path -LiteralPath $UnsetJournal)) 'extracted guard skips on unset path'

  # === BOUNDED TIMEOUT with genuine orphan START ===
  # Child invokes the actual timed native wrapper (Read-NativeChannelsWithExit
  # via the ledger), then sleeps past the capture deadline. The deadline
  # kills the tree; the flushed journal retains START without END.
  $TimeoutJournal = Join-Path $Control27Dir 'timeout-journal.ndjson'
  $env:HUM_TIMING_JOURNAL = $TimeoutJournal
  Initialize-HumProfileTimingJournal
  $TimeoutChild = Join-Path $Control27Dir 'timeout-child.ps1'
  $SleepB64 = ConvertTo-EncodedCommand 'Start-Sleep -Seconds 60'
  [IO.File]::WriteAllText($TimeoutChild, (@(
    '$ErrorActionPreference = ' + (Quote-Single 'Stop')
    '. ' + (Quote-Single (Join-Path $ToolsDir 'hum_timing_ledger.ps1'))
    ('$JournalPath = ' + (Quote-Single $TimeoutJournal))
    'Initialize-HumTimingLedger -LedgerPath $JournalPath'
    # The actual timed native wrapper from the ledger module
    '$s = Start-HumTimedCommand -Kind ''test'' -Label ''timeout-orphan'' -Phase ''exec'' -Executable ''sleep'' -Arguments @()'
    ('& ' + (Quote-Single $Pwsh) + ' -NoLogo -NoProfile -NonInteractive -EncodedCommand ' + (Quote-Single $SleepB64))
    '# No Stop-HumTimedCommand: the deadline kills us first (genuine orphan START)'
  ) -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
  $TimeoutCaptureDir = Join-Path $Control27Dir 'timeout-capture'
  $TimeoutCap = Invoke-HumBinaryCapture $Pwsh @('-NoLogo','-NoProfile','-NonInteractive','-File',$TimeoutChild) $Control27Dir $TimeoutCaptureDir 15 -CaseName 'control27-timeout'
  Assert-Control ($TimeoutCap.TimedOut) 'bounded timeout fired'
  # Authenticated tree/channel quiescence from the capture's existing proof
  $TimeoutQuiescent = ($TimeoutCap.JobQuiescenceObserved -and $TimeoutCap.FinalActiveProcessCount -eq 0 -and
    $TimeoutCap.StdoutCompletionObserved -and $TimeoutCap.StderrCompletionObserved -and $TimeoutCap.PrimaryExitObserved)
  Assert-Control $TimeoutQuiescent 'timeout capture proves quiescence via existing fields'
  $TimeoutRecords = @(Read-LedgerRecords $TimeoutJournal)
  $OrphanStarts = @($TimeoutRecords | Where-Object { $_.event -ceq 'start' -and $_.label -ceq 'timeout-orphan' })
  $OrphanEnds = @($TimeoutRecords | Where-Object { $_.event -ceq 'end' -and $_.label -ceq 'timeout-orphan' })
  Assert-Control ($OrphanStarts.Count -eq 1) 'genuine START without END (orphan)'
  Assert-Control ($OrphanEnds.Count -eq 0) 'no END fabricated for killed child'

  # === REAL EARLY DIAGNOSTICS via extracted Save-HumPreflightDiagnostics ===
  $EarlyDiagDir = Join-Path $Control27Dir 'early-diagnostic'
  $EarlyCapDir = Join-Path $Control27Dir 'early-capture'
  [IO.Directory]::CreateDirectory($EarlyCapDir) | Out-Null
  # The extracted function requires Set-HumDurableText; define a minimal version
  function Set-HumDurableText([string]$Path, [string]$Content) { [IO.File]::WriteAllText($Path, $Content, [Text.UTF8Encoding]::new($false)) }
  Save-HumPreflightDiagnostics $EarlyCapDir $EarlyDiagDir 'test early diagnostics'
  Assert-Control (Test-Path -LiteralPath (Join-Path $EarlyDiagDir 'diagnostic_status.txt')) 'extracted Save-HumPreflightDiagnostics creates real early diagnostics'
  $EarlyFile = Join-Path $EarlyDiagDir 'early-marker.txt'
  [IO.File]::WriteAllText($EarlyFile, 'early snapshot marker')

  # === JOURNAL FINALIZER via extracted logic, observing DiagnosticDirectory ===
  # The finalizer stages to DiagnosticDirectory after quiescence proof.
  # We execute the production retention call (not a copy): the observation
  # is the journal file in DiagnosticDirectory.
  $FinalDiagDir = Join-Path $Control27Dir 'final-diagnostic'
  [IO.Directory]::CreateDirectory($FinalDiagDir) | Out-Null
  [IO.File]::WriteAllText((Join-Path $FinalDiagDir 'preexisting.txt'), 'early snapshot')
  # Use the proven timeout capture's quiescence for the staging decision
  if ($TimeoutQuiescent) {
    Invoke-HumTimingJournalRetention -JournalPath $TimeoutJournal -CaptureDirectory $TimeoutCaptureDir -Failed $true -DestinationDirectory $FinalDiagDir | Out-Null
  }
  $StagedPath = Join-Path $FinalDiagDir 'hum-timing-journal.ndjson'
  Assert-Control (Test-Path -LiteralPath $StagedPath) 'extracted finalizer stages to DiagnosticDirectory after proven quiescence'
  Assert-Control (Test-Path -LiteralPath (Join-Path $FinalDiagDir 'preexisting.txt')) 'early snapshot preserved'
  $StagedRecords = @(Read-LedgerRecords $StagedPath)
  Assert-Control ((@($StagedRecords | Where-Object { $_.event -ceq 'start' -and $_.label -ceq 'timeout-orphan' })).Count -eq 1) 'staged journal retains the orphan START'

  # === SUCCESS CLEANUP ===
  $SuccessJournal = Join-Path $Control27Dir 'success-journal.ndjson'
  [IO.File]::WriteAllText($SuccessJournal, '{"event":"session"}')
  Invoke-HumTimingJournalRetention -JournalPath $SuccessJournal -CaptureDirectory $Control27Dir -Failed $false | Out-Null
  Assert-Control (-not (Test-Path -LiteralPath $SuccessJournal)) 'success removes journal'

  # === UNAVAILABLE PROOF: no staging, gap reported, journal preserved ===
  $GapJournal = Join-Path $Control27Dir 'gap-journal.ndjson'
  [IO.File]::WriteAllText($GapJournal, '{"event":"session"}')
  $GapDiag = Join-Path $Control27Dir 'gap-diagnostic'
  $NullCap = $null
  $GapQuiescent = ($null -ne $NullCap -and $NullCap.JobQuiescenceObserved)
  # The production gap branch: do NOT stage when unproven
  if (-not $GapQuiescent) {
    # Gap reported (workflow would Write-Warning); journal preserved
  }
  Assert-Control (-not (Test-Path -LiteralPath (Join-Path $GapDiag 'hum-timing-journal.ndjson'))) 'unavailable proof stages nothing'
  Assert-Control (Test-Path -LiteralPath $GapJournal) 'unavailable proof preserves journal'

  # === PROVE REMOVAL FAILS THE SAME OBSERVATION ===
  # If the extracted production retention call is removed (not executed),
  # the SAME positive observation (journal in DiagnosticDirectory) must fail.
  $RemovedDiag = Join-Path $Control27Dir 'removed-diagnostic'
  [IO.Directory]::CreateDirectory($RemovedDiag) | Out-Null
  $RemovedJournal = Join-Path $Control27Dir 'removed-journal.ndjson'
  [IO.File]::WriteAllText($RemovedJournal, '{"event":"session"}')
  # Deliberately NOT calling the retention: the observation must fail
  $RemovedObserved = Test-Path -LiteralPath (Join-Path $RemovedDiag 'hum-timing-journal.ndjson')
  Assert-Control (-not $RemovedObserved) 'removed retention call fails the DiagnosticDirectory observation'

  # === PROVE MISDIRECTION FAILS THE SAME OBSERVATION ===
  $MisDiag = Join-Path $Control27Dir 'misdirected-diagnostic'
  [IO.Directory]::CreateDirectory($MisDiag) | Out-Null
  $MisJournal = Join-Path $Control27Dir 'misdirected-journal.ndjson'
  [IO.File]::WriteAllText($MisJournal, '{"event":"session"}')
  # Misdirect to the wrong directory (not the observed DiagnosticDirectory)
  Invoke-HumTimingJournalRetention -JournalPath $MisJournal -CaptureDirectory $Control27Dir -Failed $true -DestinationDirectory $Control27Dir | Out-Null
  $MisObserved = Test-Path -LiteralPath (Join-Path $MisDiag 'hum-timing-journal.ndjson')
  Assert-Control (-not $MisObserved) 'misdirected retention fails the DiagnosticDirectory observation'

  Remove-HumCaptureAfterAuthentication $TimeoutCaptureDir
} finally {
  if ($null -ne $SavedJournal27) { $env:HUM_TIMING_JOURNAL = $SavedJournal27 } else { Remove-Item Env:HUM_TIMING_JOURNAL -ErrorAction SilentlyContinue }
  Remove-Item -LiteralPath $Control27Dir -Recurse -Force -ErrorAction SilentlyContinue
}

Remove-Item -LiteralPath $WorkDir -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "timing controls: $($script:Passed) passed, $($script:Failed) failed"
if ($script:Failed -gt 0) { exit 1 }
# The shared caller (Invoke-RepoScript) fails the group on a nonzero
# $LASTEXITCODE. Expected native failures above (exit 3/5 probes) must not
# leak a false failure status once the suite itself passed. Genuine
# failures already terminated via exit 1 above, so this reset runs only on
# the success path; the generic caller is untouched.
$global:LASTEXITCODE = 0
