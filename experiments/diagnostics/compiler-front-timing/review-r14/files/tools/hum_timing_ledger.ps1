# Durable per-command timing ledger for the fixed validation profiles.
#
# Problem: hum-dev runs the whole profile inside one buffered pwsh process.
# When the outer capture deadline fires, the buffered console output is empty,
# so a slow-but-progressing command and a stalled one are indistinguishable.
# This module writes one JSON line per command START and END to a journal
# file with an immediate flush, so the records survive the outer timeout and
# the failed-artifact path. Every launch gets a unique id shared by its START
# and END records; readers match by id, never by label/phase (labels repeat).
# A START with no matching END means completion was unobserved: the process
# died before its finally block ran. That is distinct from failure.
#
# The journal never changes command behavior: child output, exit codes, JSON
# payloads, selector accounting, mutation failure checks, source restoration
# and process cleanup all flow through the existing wrappers untouched. The
# wrappers only add a START record before launch and an END record after.
#
# Coverage: the timed capture points are the complete native-launch
# inventory of the fixed profiles -
#   check_all.ps1: Invoke-Native, Read-NativeOutput, Read-NativeOutputWithExit,
#     Read-NativeChannelsWithExit, Read-NativeArgumentListWithExit,
#     Read-NativeBytesWithExit
#   test_exact_rust_selector.ps1: Invoke-ExactRustNativeCapture
#     (every cargo selector, list and run phases)
#   run_fast_evidence.ps1: Invoke-HumContainedRustNativeCapture
#     (the Unit C mutation containment path)
# A re-entrancy guard suppresses nested records: when a timed helper calls
# another timed launch function, only the outer launch is recorded.
#
# Standalone use: call sites guard on the existence of Start-HumTimedCommand
# (Get-Command), so scripts that import the launch functions without this
# module run untimed instead of failing. When the module is loaded but the
# ledger was never initialized (disabled), Start returns $null and the
# wrappers run unchanged with no journal file.

$ErrorActionPreference = 'Stop'

# Presence of this variable is the enablement signal. check_all.ps1
# dot-sources this file, which sets it; contexts that import individual
# functions without the module (e.g. the CI policy tests importing only
# Invoke-HumFixedProfile) see $null and skip timing silently.
$script:HumTimingEnabled = $true
$script:HumTimingLedgerPath = ''
$script:HumTimingGroup = ''
$script:HumTimingSession = ''
$script:HumTimingDepth = 0

function Get-HumTimingDefaultLedgerPath {
  return (Join-Path (Join-Path $PSScriptRoot '..') (Join-Path 'target' 'hum-timing-ledger.ndjson'))
}

function Write-HumTimingRecord {
  param(
    [Parameter(Mandatory = $true)]
    [System.Collections.Specialized.OrderedDictionary] $Record
  )
  if ([string]::IsNullOrEmpty($script:HumTimingLedgerPath)) { return }
  $Line = (New-Object PSCustomObject -Property $Record | ConvertTo-Json -Compress -Depth 6)
  # One open/write/close per record: nothing is buffered in the process, so a
  # record committed here is on disk even if the process is killed next.
  [IO.File]::AppendAllText($script:HumTimingLedgerPath, $Line + "`n", [Text.Encoding]::UTF8)
}

function Initialize-HumTimingLedger {
  param([string] $LedgerPath = '')
  if ([string]::IsNullOrWhiteSpace($LedgerPath)) { $LedgerPath = Get-HumTimingDefaultLedgerPath }
  $script:HumTimingLedgerPath = $LedgerPath
  $script:HumTimingGroup = ''
  $script:HumTimingSession = [guid]::NewGuid().ToString('N')
  $Parent = [IO.Path]::GetDirectoryName($LedgerPath)
  if (-not [string]::IsNullOrEmpty($Parent)) { [IO.Directory]::CreateDirectory($Parent) | Out-Null }
  # Truncate any stale journal (e.g. restored from the cargo cache) and open
  # the session. Readers group records by the session id, and CI uses a
  # per-invocation journal path, so a stale cached journal can never be
  # attributed to a new invocation.
  [IO.File]::WriteAllText(
    $LedgerPath,
    ((New-Object PSCustomObject -Property ([ordered]@{
      event          = 'session'
      session        = $script:HumTimingSession
      ledger_version = 2
      timestamp      = [DateTimeOffset]::UtcNow.ToString('o')
    }) | ConvertTo-Json -Compress) + "`n"),
    [Text.Encoding]::UTF8)
}

function Set-HumTimingGroup {
  param([string] $Group)
  $script:HumTimingGroup = $Group
}

function Start-HumTimedCommand {
  param(
    [string] $Kind = '',
    [string] $Label = '',
    [string] $Phase = '',
    [string] $Selector = '',
    [string] $Executable = '',
    [string[]] $Arguments = @()
  )
  if ([string]::IsNullOrEmpty($script:HumTimingLedgerPath)) { return $null }
  if ($script:HumTimingDepth -gt 0) { return $null }
  $Id = [guid]::NewGuid().ToString('N')
  $script:HumTimingDepth++
  $Scope = [pscustomobject] @{
    Id        = $Id
    Kind      = $Kind
    Label     = $Label
    Phase     = $Phase
    Selector  = $Selector
    Executable = $Executable
    Arguments = $Arguments
    Stopwatch = [Diagnostics.Stopwatch]::StartNew()
    Completed = $false
  }
  try {
    Write-HumTimingRecord ([ordered]@{
      event      = 'start'
      id         = $Id
      session    = $script:HumTimingSession
      group      = $script:HumTimingGroup
      kind       = $Kind
      label      = $Label
      phase      = $Phase
      selector   = $Selector
      executable = $Executable
      argv       = @($Arguments)
      pid        = $PID
      timestamp  = [DateTimeOffset]::UtcNow.ToString('o')
    })
  } catch {
    # The scope was opened but its START never hit the journal: release the
    # depth slot so a subsequent launch still records its own complete pair.
    $script:HumTimingDepth--
    throw
  }
  return $Scope
}

function Stop-HumTimedCommand {
  param(
    [Parameter(Mandatory = $true)]
    [AllowNull()]
    $Scope,
    [int] $ExitCode = -1,
    [string] $Outcome = 'error',
    [string] $ErrorMessage = ''
  )
  if ($null -eq $Scope) { return }
  if ($Scope.Completed) { return }
  $Scope.Completed = $true
  $script:HumTimingDepth--
  $Scope.Stopwatch.Stop()
  $Record = [ordered]@{
    event      = 'end'
    id         = $Scope.Id
    session    = $script:HumTimingSession
    group      = $script:HumTimingGroup
    kind       = $Scope.Kind
    label      = $Scope.Label
    phase      = $Scope.Phase
    selector   = $Scope.Selector
    executable = $Scope.Executable
    elapsed_ms = $Scope.Stopwatch.ElapsedMilliseconds
    exit_code  = $ExitCode
    outcome    = $Outcome
    timestamp  = [DateTimeOffset]::UtcNow.ToString('o')
  }
  if (-not [string]::IsNullOrEmpty($ErrorMessage)) { $Record['error'] = $ErrorMessage }
  Write-HumTimingRecord $Record
}
