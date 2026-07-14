$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ClassifierPath = Join-Path $PSScriptRoot 'check_workorder_status_boundary.ps1'
. $ClassifierPath

$script:BoundaryTestCount = 0
$script:BoundaryRepositorySerial = 0

function Assert-BoundaryTest {
  param(
    [bool] $Condition,
    [string] $Message
  )

  if (-not $Condition) {
    throw $Message
  }
}

function Invoke-TestGit {
  param(
    [string] $RepoPath,
    [string[]] $Arguments,
    [string] $StdinText = ''
  )

  $Git = (Get-Command git -ErrorAction Stop).Source
  $PreviousPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    if ($StdinText -ne '') {
      $Output = $StdinText | & $Git -C $RepoPath @Arguments 2>&1
    } else {
      $Output = & $Git -C $RepoPath @Arguments 2>&1
    }
    $ExitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $PreviousPreference
  }
  if ($ExitCode -ne 0) {
    throw "test git $($Arguments -join ' ') failed: $(@($Output) -join "`n")"
  }
  return (@($Output | ForEach-Object { [string]$_ }) -join "`n").Trim()
}

function Write-TestText {
  param(
    [string] $Path,
    [string] $Text
  )

  $Parent = Split-Path -Parent $Path
  if (-not (Test-Path -LiteralPath $Parent)) {
    [void][System.IO.Directory]::CreateDirectory($Parent)
  }
  $Utf8 = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $Text, $Utf8)
}

function New-TestWorkOrderText {
  param(
    [string] $Status = ' baseline',
    [string] $Gate = "`nbaseline authorization`n",
    [string] $Mandate = "## Session AP mandate`nExecutable requirements stay frozen.`n",
    [string] $Tail = "`n"
  )

  return @(
    '# Test Work Order'
    ''
    'Date: 2026-07-14'
    "Status:$Status"
    'Owner: BDFL (Ocean).'
    ''
    $Mandate.TrimEnd("`r", "`n")
    ''
    '## Current authorization gate'
    $Gate.TrimEnd("`r", "`n")
    '<!-- workorder-current-authorization-gate:end -->'
    $Tail.TrimEnd("`r", "`n")
  ) -join "`n"
}

function New-TestRepository {
  param(
    [string] $Root,
    [switch] $WithoutWorkOrder
  )

  $script:BoundaryRepositorySerial += 1
  $Path = Join-Path $Root ("repo-{0:D3}" -f $script:BoundaryRepositorySerial)
  [void][System.IO.Directory]::CreateDirectory($Path)
  Invoke-TestGit $Path @('init', '--initial-branch=main', '--quiet') | Out-Null
  $EmptyHooks = Join-Path $Path '.git/empty-hooks'
  [void][System.IO.Directory]::CreateDirectory($EmptyHooks)
  Invoke-TestGit $Path @('config', 'core.hooksPath', $EmptyHooks) | Out-Null
  Invoke-TestGit $Path @('config', 'user.name', 'Hum Boundary Test') | Out-Null
  Invoke-TestGit $Path @('config', 'user.email', 'boundary@example.invalid') | Out-Null
  Invoke-TestGit $Path @('config', 'core.autocrlf', 'false') | Out-Null
  Invoke-TestGit $Path @('config', 'core.filemode', 'true') | Out-Null

  if (-not $WithoutWorkOrder) {
    Write-TestText (Join-Path $Path 'WORKORDER.md') (New-TestWorkOrderText)
  }
  Write-TestText (Join-Path $Path 'src/main.rs') "fn main() {}`n"
  Write-TestText (Join-Path $Path '.github/workflows/ci.yml') "name: ci`n"
  Write-TestText (Join-Path $Path 'tools/check_all.ps1') "Write-Host preflight`n"
  Write-TestText (Join-Path $Path 'fixtures/base.hum') "task base() -> Unit`n"
  Write-TestText (Join-Path $Path 'Cargo.toml') ('[package]' + "`n" + 'name = "boundary-test"' + "`n" + 'version = "0.0.0"' + "`n")
  Write-TestText (Join-Path $Path 'generated/output.txt') "baseline`n"
  Invoke-TestGit $Path @('add', '--all') | Out-Null
  Invoke-TestGit $Path @('commit', '--quiet', '-m', 'full anchor') | Out-Null
  $Anchor = Invoke-TestGit $Path @('rev-parse', 'HEAD')
  return [pscustomobject]@{
    Path = $Path
    Anchor = $Anchor
  }
}

function Commit-TestRepository {
  param(
    [object] $Repository,
    [string] $Message
  )

  Invoke-TestGit $Repository.Path @('add', '--all') | Out-Null
  Invoke-TestGit $Repository.Path @('commit', '--quiet', '-m', $Message) | Out-Null
  return Invoke-TestGit $Repository.Path @('rev-parse', 'HEAD')
}

function Add-TestStatusCommit {
  param(
    [object] $Repository,
    [string] $Status,
    [string] $Gate,
    [string] $Mandate = "## Session AP mandate`nExecutable requirements stay frozen.`n"
  )

  Write-TestText (Join-Path $Repository.Path 'WORKORDER.md') (
    New-TestWorkOrderText -Status $Status -Gate $Gate -Mandate $Mandate
  )
  return Commit-TestRepository $Repository 'status update'
}

function Copy-TestObject {
  param([object] $Value)

  return $Value | ConvertTo-Json -Depth 30 | ConvertFrom-Json
}

function New-TestStep {
  param(
    [string] $Name,
    [string] $Conclusion
  )

  return [pscustomobject]@{
    name = $Name
    status = 'completed'
    conclusion = $Conclusion
  }
}

function New-TestJob {
  param(
    [long] $Id,
    [long] $RunId,
    [int] $Attempt,
    [string] $Anchor,
    [string] $Platform
  )

  return [pscustomobject]@{
    id = $Id
    run_id = $RunId
    run_attempt = $Attempt
    name = "preflight ($Platform)"
    head_sha = $Anchor
    status = 'completed'
    conclusion = 'success'
    labels = @($Platform)
    steps = @(
      (New-TestStep 'Set up job' 'success')
      (New-TestStep 'Run Hum preflight' 'success')
      (New-TestStep 'Run status-only evidence' 'skipped')
    )
  }
}

function New-TestSnapshot {
  param(
    [string] $Anchor,
    [long] $RunId = 9001,
    [int] $Attempt = 1,
    [long] $UbuntuJobId = 9101,
    [long] $WindowsJobId = 9102
  )

  $Run = [pscustomobject]@{
    id = $RunId
    name = 'ci'
    path = '.github/workflows/ci.yml'
    head_branch = 'main'
    head_sha = $Anchor
    event = 'push'
    status = 'completed'
    conclusion = 'success'
    run_attempt = $Attempt
  }
  $Jobs = @(
    (New-TestJob $UbuntuJobId $RunId $Attempt $Anchor 'ubuntu-latest')
    (New-TestJob $WindowsJobId $RunId $Attempt $Anchor 'windows-latest')
  )
  return [pscustomobject]@{
    RunPages = @([pscustomobject]@{
      page_number = 1
      total_count = 1
      workflow_runs = @($Run)
    })
    JobPages = @([pscustomobject]@{
      page_number = 1
      total_count = 2
      jobs = @($Jobs)
    })
  }
}

function New-TestEvidencePair {
  param(
    [string] $Anchor,
    [int] $Attempt = 1
  )

  $First = New-TestSnapshot -Anchor $Anchor -Attempt $Attempt
  return [pscustomobject]@{
    First = $First
    Second = Copy-TestObject $First
    ThrowOn = @()
  }
}

function New-TestEvidenceProvider {
  param([object] $Pair)

  $State = $Pair
  return {
    param([string] $Candidate, [int] $SnapshotNumber)
    if (@($State.ThrowOn) -contains $SnapshotNumber) {
      throw "synthetic Actions failure $SnapshotNumber"
    }
    if ($SnapshotNumber -eq 1) {
      return $State.First
    }
    if ($SnapshotNumber -eq 2) {
      return $State.Second
    }
    throw "unexpected snapshot $SnapshotNumber for $Candidate"
  }.GetNewClosure()
}

function New-ValidPairFactory {
  param([string] $Anchor)

  return New-TestEvidencePair $Anchor
}

function New-MutatedPairFactory {
  param(
    [string] $Anchor,
    [scriptblock] $Mutation
  )

  $Pair = New-TestEvidencePair $Anchor
  & $Mutation $Pair
  return $Pair
}

function Invoke-BoundaryCase {
  param(
    [string] $Name,
    [object] $Repository,
    [string] $Base,
    [string] $Head,
    [object] $PairFactory,
    [string] $ExpectedMode,
    [string] $ExpectedReason,
    [string] $ExpectedAnchor = '',
    [string] $EventName = 'push',
    [string] $EventRef = 'refs/heads/main',
    [string] $WorkflowPath = '.github/workflows/ci.yml'
  )

  $EvidenceRows = New-Object System.Collections.Generic.List[string]
  for ($Execution = 1; $Execution -le 2; $Execution += 1) {
    $Pair = Copy-TestObject $PairFactory
    $Provider = New-TestEvidenceProvider $Pair
    $Arguments = @{
      RepoPath = $Repository.Path
      WorkflowPath = $WorkflowPath
      EventName = $EventName
      EventRef = $EventRef
      BaseCommit = $Base
      HeadCommit = $Head
      ActionsEvidenceProvider = $Provider
    }
    $Result = Invoke-WorkOrderStatusClassificationCore @Arguments
    Assert-BoundaryTest ($Result.Mode -ceq $ExpectedMode) "$Name execution $Execution returned mode $($Result.Mode) with reason $($Result.Reason)"
    Assert-BoundaryTest ($Result.Reason -ceq $ExpectedReason) "$Name execution $Execution returned reason $($Result.Reason)"
    Assert-BoundaryTest ($Result.Anchor -ceq $ExpectedAnchor) "$Name execution $Execution returned anchor $($Result.Anchor)"
    $EvidenceRows.Add((ConvertTo-WorkOrderBoundaryEvidence $Result))
  }

  Assert-BoundaryTest ($EvidenceRows[0] -ceq $EvidenceRows[1]) "$Name was not byte-identical across two fresh executions"
  $script:BoundaryTestCount += 1
  Write-Host "ok $($script:BoundaryTestCount) - $Name"
}

function Set-BothSnapshots {
  param(
    [object] $Pair,
    [scriptblock] $Mutation
  )

  & $Mutation $Pair.First
  & $Mutation $Pair.Second
}

function Assert-ProductionSeamIsClosed {
  $Parameters = @((Get-Command $ClassifierPath).Parameters.Keys)
  foreach ($Forbidden in @(
    'Anchor', 'RunId', 'RunAttempt', 'JobId', 'Success', 'Evidence',
    'EvidenceProvider', 'Snapshot', 'Response', 'Fixture', 'Cache', 'ResultPath'
  )) {
    Assert-BoundaryTest (-not ($Parameters -contains $Forbidden)) "production classifier exposes forbidden parameter $Forbidden"
  }
  foreach ($Required in @('Repository', 'WorkflowPath', 'EventName', 'EventRef', 'BaseCommit', 'HeadCommit')) {
    Assert-BoundaryTest ($Parameters -contains $Required) "production classifier is missing parameter $Required"
  }

  $Workflow = [System.IO.File]::ReadAllText((Join-Path $RepoRoot '.github/workflows/ci.yml'))
  foreach ($RequiredText in @(
    'contents: read',
    'actions: read',
    'cancel-in-progress: true',
    'fetch-depth: 0',
    'preflight (${{ matrix.os }})',
    'windows-latest',
    'ubuntu-latest',
    'Classify CI evidence lane',
    'Run Hum preflight',
    'Run status-only evidence'
  )) {
    Assert-BoundaryTest $Workflow.Contains($RequiredText) "workflow is missing $RequiredText"
  }
  Assert-BoundaryTest (-not $Workflow.Contains('paths-ignore')) 'workflow must not use paths-ignore'
  Assert-BoundaryTest ($Workflow.IndexOf('Classify CI evidence lane') -lt $Workflow.IndexOf('Cache Cargo artifacts')) 'classification must precede Cargo cache setup'

  $Classifier = [System.IO.File]::ReadAllText($ClassifierPath)
  foreach ($RequiredText in @('--no-replace-objects', 'refs/replace/', 'info/grafts')) {
    Assert-BoundaryTest $Classifier.Contains($RequiredText) "classifier is missing history-rewrite defense $RequiredText"
  }

  $CheckAll = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'tools/check_all.ps1'))
  Assert-BoundaryTest ([regex]::Matches($CheckAll, "test_workorder_status_boundary\.ps1").Count -eq 1) 'full preflight must invoke the boundary matrix exactly once'
  $script:BoundaryTestCount += 1
  Write-Host "ok $($script:BoundaryTestCount) - production evidence seam and workflow source contract"
}

$TempBase = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
$TestRoot = [System.IO.Path]::GetFullPath((Join-Path $TempBase ("hum-workorder-boundary-{0}" -f [guid]::NewGuid().ToString('N'))))
Assert-BoundaryTest $TestRoot.StartsWith($TempBase, [System.StringComparison]::OrdinalIgnoreCase) 'temporary test root escaped the system temp directory'
[void][System.IO.Directory]::CreateDirectory($TestRoot)

try {
  Assert-ProductionSeamIsClosed

  $Valid = New-TestRepository $TestRoot
  $ValidHead = Add-TestStatusCommit $Valid ' accepted and published' "`nnext session remains unauthorized`n"
  $ValidFactory = New-ValidPairFactory $Valid.Anchor
  $ValidCase = @{
    Name = 'one full anchor plus exact header and gate update is fast'
    Repository = $Valid
    Base = $Valid.Anchor
    Head = $ValidHead
    PairFactory = $ValidFactory
    ExpectedMode = 'fast'
    ExpectedReason = 'eligible_status_chain'
    ExpectedAnchor = $Valid.Anchor
  }
  Invoke-BoundaryCase @ValidCase

  $HeaderOnly = New-TestRepository $TestRoot
  $HeaderHead = Add-TestStatusCommit $HeaderOnly ' header-only update' "`nbaseline authorization`n"
  Invoke-BoundaryCase 'header interval alone is fast' $HeaderOnly $HeaderOnly.Anchor $HeaderHead (New-ValidPairFactory $HeaderOnly.Anchor) 'fast' 'eligible_status_chain' $HeaderOnly.Anchor

  $GateOnly = New-TestRepository $TestRoot
  $GateHead = Add-TestStatusCommit $GateOnly ' baseline' "`ngate-only update`n"
  Invoke-BoundaryCase 'current gate interval alone is fast' $GateOnly $GateOnly.Anchor $GateHead (New-ValidPairFactory $GateOnly.Anchor) 'fast' 'eligible_status_chain' $GateOnly.Anchor

  $Consecutive = New-TestRepository $TestRoot
  $ConsecutiveFirst = Add-TestStatusCommit $Consecutive ' first status' "`nfirst gate`n"
  $ConsecutiveSecond = Add-TestStatusCommit $Consecutive ' second status' "`nsecond gate`n"
  Invoke-BoundaryCase 'two consecutive status commits retain one anchor' $Consecutive $ConsecutiveFirst $ConsecutiveSecond (New-ValidPairFactory $Consecutive.Anchor) 'fast' 'eligible_status_chain' $Consecutive.Anchor
  Invoke-BoundaryCase 'rapid status push after canceled fast run remains fast' $Consecutive $ConsecutiveFirst $ConsecutiveSecond (New-ValidPairFactory $Consecutive.Anchor) 'fast' 'eligible_status_chain' $Consecutive.Anchor

  $RerunFactory = New-MutatedPairFactory $Valid.Anchor {
    param($Pair)
    foreach ($Snapshot in @($Pair.First, $Pair.Second)) {
      $Snapshot.RunPages[0].workflow_runs[0].run_attempt = 2
      foreach ($Job in @($Snapshot.JobPages[0].jobs)) {
        $Job.run_attempt = 2
      }
    }
  }
  Invoke-BoundaryCase 'one exact successful rerun attempt is eligible' $Valid $Valid.Anchor $ValidHead $RerunFactory 'fast' 'eligible_status_chain' $Valid.Anchor

  $EvidenceCases = @(
    [pscustomobject]@{ Name = 'zero workflow runs'; Reason = 'anchor_run_missing'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].total_count = 0; $Snapshot.RunPages[0].workflow_runs = @() }
    } },
    [pscustomobject]@{ Name = 'multiple workflow run IDs'; Reason = 'anchor_run_ambiguous'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot)
        $Other = Copy-TestObject $Snapshot.RunPages[0].workflow_runs[0]
        $Other.id = 9002
        $Snapshot.RunPages[0].total_count = 2
        $Snapshot.RunPages[0].workflow_runs = @($Snapshot.RunPages[0].workflow_runs[0], $Other)
      }
    } },
    [pscustomobject]@{ Name = 'incomplete run pagination'; Reason = 'run_pagination_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].total_count = 101 }
    } },
    [pscustomobject]@{ Name = 'complete multi-page run ambiguity'; Reason = 'anchor_run_ambiguous'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot)
        $Seed = $Snapshot.RunPages[0].workflow_runs[0]
        $FirstPage = @()
        for ($Index = 0; $Index -lt 100; $Index += 1) {
          $Run = Copy-TestObject $Seed
          $Run.id = 10000 + $Index
          $FirstPage += $Run
        }
        $LastRun = Copy-TestObject $Seed
        $LastRun.id = 10100
        $Snapshot.RunPages = @(
          [pscustomobject]@{ page_number = 1; total_count = 101; workflow_runs = @($FirstPage) },
          [pscustomobject]@{ page_number = 2; total_count = 101; workflow_runs = @($LastRun) }
        )
      }
    } },
    [pscustomobject]@{ Name = 'wrong workflow name'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].name = 'other' }
    } },
    [pscustomobject]@{ Name = 'missing returned workflow path'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].PSObject.Properties.Remove('path') }
    } },
    [pscustomobject]@{ Name = 'wrong path with right branch'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].path = '.github/workflows/other.yml' }
    } },
    [pscustomobject]@{ Name = 'right path with wrong branch'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].head_branch = 'release' }
    } },
    [pscustomobject]@{ Name = 'caller-derived composite cannot replace returned path'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].path = '.github/workflows/ci.yml@main' }
    } },
    [pscustomobject]@{ Name = 'wrong run event'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].event = 'workflow_dispatch' }
    } },
    [pscustomobject]@{ Name = 'wrong run head SHA'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].head_sha = ('a' * 40) }
    } },
    [pscustomobject]@{ Name = 'pending workflow run'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].status = 'in_progress'; $Snapshot.RunPages[0].workflow_runs[0].conclusion = $null }
    } },
    [pscustomobject]@{ Name = 'failed workflow run'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].conclusion = 'failure' }
    } },
    [pscustomobject]@{ Name = 'canceled workflow run'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].conclusion = 'cancelled' }
    } },
    [pscustomobject]@{ Name = 'missing run attempt'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].PSObject.Properties.Remove('run_attempt') }
    } },
    [pscustomobject]@{ Name = 'zero run attempt'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].run_attempt = 0 }
    } },
    [pscustomobject]@{ Name = 'missing Ubuntu job'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot) $Snapshot.JobPages[0].jobs = @($Snapshot.JobPages[0].jobs | Where-Object { $_.name -notlike '*ubuntu*' }); $Snapshot.JobPages[0].total_count = 1
      }
    } },
    [pscustomobject]@{ Name = 'duplicate Ubuntu job'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot) $Duplicate = Copy-TestObject $Snapshot.JobPages[0].jobs[0]; $Duplicate.id = 9199; $Snapshot.JobPages[0].jobs = @($Snapshot.JobPages[0].jobs) + @($Duplicate); $Snapshot.JobPages[0].total_count = 3
      }
    } },
    [pscustomobject]@{ Name = 'extra platform job'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot) $Extra = Copy-TestObject $Snapshot.JobPages[0].jobs[0]; $Extra.id = 9198; $Extra.name = 'preflight (macos-latest)'; $Extra.labels = @('macos-latest'); $Snapshot.JobPages[0].jobs = @($Snapshot.JobPages[0].jobs) + @($Extra); $Snapshot.JobPages[0].total_count = 3
      }
    } },
    [pscustomobject]@{ Name = 'pending platform job'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].status = 'in_progress'; $Snapshot.JobPages[0].jobs[0].conclusion = $null }
    } },
    [pscustomobject]@{ Name = 'skipped platform job'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].conclusion = 'skipped' }
    } },
    [pscustomobject]@{ Name = 'canceled platform job'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].conclusion = 'cancelled' }
    } },
    [pscustomobject]@{ Name = 'failed platform job'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].conclusion = 'failure' }
    } },
    [pscustomobject]@{ Name = 'wrong job SHA'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].head_sha = ('b' * 40) }
    } },
    [pscustomobject]@{ Name = 'job from another attempt'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].run_attempt = 2 }
    } },
    [pscustomobject]@{ Name = 'job from another run'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].run_id = 8000 }
    } },
    [pscustomobject]@{ Name = 'wrong platform label'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].jobs[0].labels = @('windows-latest') }
    } },
    [pscustomobject]@{ Name = 'missing full preflight step'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot) $Snapshot.JobPages[0].jobs[0].steps = @($Snapshot.JobPages[0].jobs[0].steps | Where-Object { $_.name -ne 'Run Hum preflight' })
      }
    } },
    [pscustomobject]@{ Name = 'duplicate full preflight step'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot) $Snapshot.JobPages[0].jobs[0].steps = @($Snapshot.JobPages[0].jobs[0].steps) + @(New-TestStep 'Run Hum preflight' 'success')
      }
    } },
    [pscustomobject]@{ Name = 'duplicate fast evidence step'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot) $Snapshot.JobPages[0].jobs[0].steps = @($Snapshot.JobPages[0].jobs[0].steps) + @(New-TestStep 'Run status-only evidence' 'skipped')
      }
    } },
    [pscustomobject]@{ Name = 'renamed fast step'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) ($Snapshot.JobPages[0].jobs[0].steps | Where-Object { $_.name -eq 'Run status-only evidence' }).name = 'Run quick evidence' }
    } },
    [pscustomobject]@{ Name = 'failed full preflight step'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) ($Snapshot.JobPages[0].jobs[0].steps | Where-Object { $_.name -eq 'Run Hum preflight' }).conclusion = 'failure' }
    } },
    [pscustomobject]@{ Name = 'successful fast step cannot anchor'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) ($Snapshot.JobPages[0].jobs[0].steps | Where-Object { $_.name -eq 'Run status-only evidence' }).conclusion = 'success' }
    } },
    [pscustomobject]@{ Name = 'pending fast step'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Step = $Snapshot.JobPages[0].jobs[0].steps | Where-Object { $_.name -eq 'Run status-only evidence' }; $Step.status = 'pending'; $Step.conclusion = $null }
    } },
    [pscustomobject]@{ Name = 'incomplete job pagination'; Reason = 'job_pagination_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.JobPages[0].total_count = 102 }
    } },
    [pscustomobject]@{ Name = 'Actions permission or transport failure'; Reason = 'actions_lookup_failed'; Mutate = {
      param($Pair) $Pair.ThrowOn = @(1)
    } },
    [pscustomobject]@{ Name = 'Actions authorization denial'; Reason = 'actions_lookup_failed'; Mutate = {
      param($Pair) $Pair.ThrowOn = @(1)
    } },
    [pscustomobject]@{ Name = 'Actions rate limit failure'; Reason = 'actions_lookup_failed'; Mutate = {
      param($Pair) $Pair.ThrowOn = @(1)
    } },
    [pscustomobject]@{ Name = 'Actions race on second snapshot'; Reason = 'actions_lookup_failed'; Mutate = {
      param($Pair) $Pair.ThrowOn = @(2)
    } },
    [pscustomobject]@{ Name = 'run changes between snapshots'; Reason = 'control_plane_changed'; Mutate = {
      param($Pair)
      $Pair.Second.RunPages[0].workflow_runs[0].id = 9002
      foreach ($Job in @($Pair.Second.JobPages[0].jobs)) { $Job.run_id = 9002 }
    } },
    [pscustomobject]@{ Name = 'attempt changes between snapshots'; Reason = 'control_plane_changed'; Mutate = {
      param($Pair)
      $Pair.Second.RunPages[0].workflow_runs[0].run_attempt = 2
      foreach ($Job in @($Pair.Second.JobPages[0].jobs)) { $Job.run_attempt = 2 }
    } },
    [pscustomobject]@{ Name = 'job identity changes between snapshots'; Reason = 'control_plane_changed'; Mutate = {
      param($Pair) $Pair.Second.JobPages[0].jobs[0].id = 9991
    } },
    [pscustomobject]@{ Name = 'run status changes between snapshots'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) $Pair.Second.RunPages[0].workflow_runs[0].status = 'in_progress'; $Pair.Second.RunPages[0].workflow_runs[0].conclusion = $null
    } },
    [pscustomobject]@{ Name = 'job conclusion changes between snapshots'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) $Pair.Second.JobPages[0].jobs[0].conclusion = 'failure'
    } },
    [pscustomobject]@{ Name = 'step conclusion changes between snapshots'; Reason = 'anchor_steps_invalid'; Mutate = {
      param($Pair) ($Pair.Second.JobPages[0].jobs[0].steps | Where-Object { $_.name -eq 'Run Hum preflight' }).conclusion = 'failure'
    } },
    [pscustomobject]@{ Name = 'unrelated step fact changes between snapshots'; Reason = 'control_plane_changed'; Mutate = {
      param($Pair) ($Pair.Second.JobPages[0].jobs[0].steps | Where-Object { $_.name -eq 'Set up job' }).conclusion = 'skipped'
    } },
    [pscustomobject]@{ Name = 'pagination changes between snapshots'; Reason = 'run_pagination_invalid'; Mutate = {
      param($Pair) $Pair.Second.RunPages[0].total_count = 101
    } }
  )

  foreach ($Case in $EvidenceCases) {
    $Mutation = $Case.Mutate
    $Factory = New-MutatedPairFactory $Valid.Anchor $Mutation
    Invoke-BoundaryCase $Case.Name $Valid $Valid.Anchor $ValidHead $Factory 'full' $Case.Reason
  }

  Invoke-BoundaryCase 'workflow dispatch is full' $Valid $Valid.Anchor $ValidHead $ValidFactory 'full' 'event_not_push' '' 'workflow_dispatch'
  Invoke-BoundaryCase 'tag push is full' $Valid $Valid.Anchor $ValidHead $ValidFactory 'full' 'event_not_main' '' 'push' 'refs/tags/v0.0.1'
  Invoke-BoundaryCase 'caller composite workflow identity is full' $Valid $Valid.Anchor $ValidHead $ValidFactory 'full' 'workflow_path_invalid' '' 'push' 'refs/heads/main' '.github/workflows/ci.yml@main'
  Invoke-BoundaryCase 'empty event range is full' $Valid $ValidHead $ValidHead $ValidFactory 'full' 'event_range_empty'
  Invoke-BoundaryCase 'zero event base is full' $Valid ('0' * 40) $ValidHead $ValidFactory 'full' 'event_base_invalid'
  Invoke-BoundaryCase 'invalid event base is full' $Valid 'not-a-commit' $ValidHead $ValidFactory 'full' 'event_base_invalid'
  Invoke-BoundaryCase 'unavailable event base is full' $Valid ('f' * 40) $ValidHead $ValidFactory 'full' 'event_base_invalid'
  $BlobBase = Invoke-TestGit $Valid.Path @('rev-parse', "$($Valid.Anchor):WORKORDER.md")
  Invoke-BoundaryCase 'non-commit event base is full' $Valid $BlobBase $ValidHead $ValidFactory 'full' 'event_base_invalid'
  Invoke-BoundaryCase 'invalid head is full' $Valid $Valid.Anchor 'not-a-head' $ValidFactory 'full' 'event_head_invalid'
  Invoke-BoundaryCase 'checkout and proposed head disagreement is full' $Valid $Valid.Anchor $Valid.Anchor $ValidFactory 'full' 'checkout_head_mismatch'

  $Unauthorized = New-TestRepository $TestRoot
  $UnauthorizedHead = Add-TestStatusCommit $Unauthorized ' changed status' "`nchanged gate`n" "## Session AP mandate`nExecutable requirements were weakened.`n"
  Invoke-BoundaryCase 'unauthorized mandate edit is full' $Unauthorized $Unauthorized.Anchor $UnauthorizedHead (New-ValidPairFactory $Unauthorized.Anchor) 'full' 'no_status_transition'

  foreach ($PathCase in @(
    [pscustomobject]@{ Name = 'Rust source'; Path = 'src/main.rs'; Text = ('fn main() { println!("changed"); }' + "`n") },
    [pscustomobject]@{ Name = 'fixture'; Path = 'fixtures/base.hum'; Text = "task changed() -> Unit`n" },
    [pscustomobject]@{ Name = 'Cargo'; Path = 'Cargo.toml'; Text = ('[package]' + "`n" + 'name = "changed"' + "`n" + 'version = "0.0.0"' + "`n") },
    [pscustomobject]@{ Name = 'tool'; Path = 'tools/check_all.ps1'; Text = "Write-Host changed`n" },
    [pscustomobject]@{ Name = 'workflow'; Path = '.github/workflows/ci.yml'; Text = "name: changed`n" },
    [pscustomobject]@{ Name = 'generated output'; Path = 'generated/output.txt'; Text = "changed`n" }
  )) {
    $Repo = New-TestRepository $TestRoot
    Write-TestText (Join-Path $Repo.Path $PathCase.Path) $PathCase.Text
    $Head = Commit-TestRepository $Repo "$($PathCase.Name) change"
    Invoke-BoundaryCase "$($PathCase.Name) change is full" $Repo $Repo.Anchor $Head (New-ValidPairFactory $Repo.Anchor) 'full' 'no_status_transition'
  }

  $Multiple = New-TestRepository $TestRoot
  Write-TestText (Join-Path $Multiple.Path 'WORKORDER.md') (New-TestWorkOrderText -Status ' status plus source')
  Write-TestText (Join-Path $Multiple.Path 'src/main.rs') ('fn main() { println!("changed"); }' + "`n")
  $MultipleHead = Commit-TestRepository $Multiple 'multiple paths'
  Invoke-BoundaryCase 'multiple-path change is full' $Multiple $Multiple.Anchor $MultipleHead (New-ValidPairFactory $Multiple.Anchor) 'full' 'no_status_transition'

  $ExecutableThenStatus = New-TestRepository $TestRoot
  Write-TestText (Join-Path $ExecutableThenStatus.Path 'src/main.rs') ('fn main() { println!("changed"); }' + "`n")
  $ExecutableCommit = Commit-TestRepository $ExecutableThenStatus 'executable change'
  $ExecutableStatus = Add-TestStatusCommit $ExecutableThenStatus ' after executable' "`nafter executable`n"
  Invoke-BoundaryCase 'status after unproven executable predecessor is full' $ExecutableThenStatus $ExecutableCommit $ExecutableStatus (New-ValidPairFactory $ExecutableThenStatus.Anchor) 'full' 'anchor_run_invalid'

  $ExecutablePredecessorCases = @(
    [pscustomobject]@{ Name = 'pending executable predecessor'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].status = 'in_progress'; $Snapshot.RunPages[0].workflow_runs[0].conclusion = $null }
    } },
    [pscustomobject]@{ Name = 'failed executable predecessor'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].conclusion = 'failure' }
    } },
    [pscustomobject]@{ Name = 'canceled executable predecessor'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].conclusion = 'cancelled' }
    } },
    [pscustomobject]@{ Name = 'skipped executable predecessor'; Reason = 'anchor_run_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].conclusion = 'skipped' }
    } },
    [pscustomobject]@{ Name = 'platform-incomplete executable predecessor'; Reason = 'anchor_jobs_invalid'; Mutate = {
      param($Pair) Set-BothSnapshots $Pair {
        param($Snapshot)
        $Snapshot.JobPages[0].jobs = @($Snapshot.JobPages[0].jobs | Where-Object { $_.name -notlike '*windows*' })
        $Snapshot.JobPages[0].total_count = 1
      }
    } }
  )
  foreach ($PredecessorCase in $ExecutablePredecessorCases) {
    $Mutation = $PredecessorCase.Mutate
    $Factory = New-MutatedPairFactory $ExecutableCommit $Mutation
    Invoke-BoundaryCase $PredecessorCase.Name $ExecutableThenStatus $ExecutableCommit $ExecutableStatus $Factory 'full' $PredecessorCase.Reason
  }

  $RevertedThenStatus = New-TestRepository $TestRoot
  Write-TestText (Join-Path $RevertedThenStatus.Path 'src/main.rs') ('fn main() { println!("changed"); }' + "`n")
  Commit-TestRepository $RevertedThenStatus 'executable change' | Out-Null
  Write-TestText (Join-Path $RevertedThenStatus.Path 'src/main.rs') "fn main() {}`n"
  $RevertCommit = Commit-TestRepository $RevertedThenStatus 'revert executable change'
  $RevertStatus = Add-TestStatusCommit $RevertedThenStatus ' after revert' "`nafter revert`n"
  Invoke-BoundaryCase 'executable change later reverted still cannot inherit old anchor' $RevertedThenStatus $RevertCommit $RevertStatus (New-ValidPairFactory $RevertedThenStatus.Anchor) 'full' 'anchor_run_invalid'

  $PendingExecutable = New-TestRepository $TestRoot
  Write-TestText (Join-Path $PendingExecutable.Path 'tools/check_all.ps1') "Write-Host changed`n"
  $PendingCommit = Commit-TestRepository $PendingExecutable 'pending executable'
  $PendingStatus = Add-TestStatusCommit $PendingExecutable ' cancels pending executable' "`ncancels pending executable`n"
  $PendingFactory = New-MutatedPairFactory $PendingCommit {
    param($Pair) Set-BothSnapshots $Pair { param($Snapshot) $Snapshot.RunPages[0].workflow_runs[0].status = 'in_progress'; $Snapshot.RunPages[0].workflow_runs[0].conclusion = $null }
  }
  Invoke-BoundaryCase 'rapid status push cannot inherit pending executable run' $PendingExecutable $PendingCommit $PendingStatus $PendingFactory 'full' 'anchor_run_invalid'

  $FastAsAnchorFactory = New-MutatedPairFactory $Valid.Anchor {
    param($Pair) Set-BothSnapshots $Pair {
      param($Snapshot)
      foreach ($Job in @($Snapshot.JobPages[0].jobs)) {
        ($Job.steps | Where-Object { $_.name -eq 'Run Hum preflight' }).conclusion = 'skipped'
        ($Job.steps | Where-Object { $_.name -eq 'Run status-only evidence' }).conclusion = 'success'
      }
    }
  }
  Invoke-BoundaryCase 'successful fast run cannot serve as full anchor' $Valid $Valid.Anchor $ValidHead $FastAsAnchorFactory 'full' 'anchor_steps_invalid'

  $ReplacedHistory = New-TestRepository $TestRoot
  Write-TestText (Join-Path $ReplacedHistory.Path 'src/main.rs') ('fn main() { println!("executable"); }' + "`n")
  $ExecutableHead = Commit-TestRepository $ReplacedHistory 'executable transition'
  $OriginalTransition = Invoke-TestGit $ReplacedHistory.Path @(
    '--no-replace-objects', 'diff-tree', '--no-commit-id', '--name-status', '-r',
    $ReplacedHistory.Anchor, $ExecutableHead, '--'
  )
  Assert-BoundaryTest ($OriginalTransition -ceq "M`tsrc/main.rs") 'replacement probe did not begin with exactly one executable transition'
  Write-TestText (Join-Path $ReplacedHistory.Path 'src/main.rs') "fn main() {}`n"
  Write-TestText (Join-Path $ReplacedHistory.Path 'WORKORDER.md') (
    New-TestWorkOrderText -Status ' replacement disguise' -Gate "`nreplacement disguise`n"
  )
  Invoke-TestGit $ReplacedHistory.Path @('add', '--all') | Out-Null
  $ReplacementTree = Invoke-TestGit $ReplacedHistory.Path @('write-tree')
  $ReplacementCommit = Invoke-TestGit $ReplacedHistory.Path @(
    'commit-tree', $ReplacementTree, '-p', $ReplacedHistory.Anchor, '-m', 'status-only replacement'
  )
  Invoke-TestGit $ReplacedHistory.Path @('replace', $ExecutableHead, $ReplacementCommit) | Out-Null
  Invoke-BoundaryCase (
    'replacement ref cannot mask an executable transition'
  ) $ReplacedHistory $ReplacedHistory.Anchor $ExecutableHead (
    New-ValidPairFactory $ReplacedHistory.Anchor
  ) 'full' 'history_rewrite_metadata_present'

  $GraftedHistory = New-TestRepository $TestRoot
  Write-TestText (Join-Path $GraftedHistory.Path 'src/main.rs') ('fn main() { println!("executable"); }' + "`n")
  Commit-TestRepository $GraftedHistory 'executable transition' | Out-Null
  Write-TestText (Join-Path $GraftedHistory.Path 'src/main.rs') "fn main() {}`n"
  Commit-TestRepository $GraftedHistory 'revert executable transition' | Out-Null
  $GraftedHead = Add-TestStatusCommit $GraftedHistory ' graft disguise' "`ngraft disguise`n"
  Write-TestText (Join-Path $GraftedHistory.Path '.git/info/grafts') (
    "$GraftedHead $($GraftedHistory.Anchor)`n"
  )
  Invoke-BoundaryCase (
    'graft metadata cannot conceal reverted executable history'
  ) $GraftedHistory $GraftedHistory.Anchor $GraftedHead (
    New-ValidPairFactory $GraftedHistory.Anchor
  ) 'full' 'history_rewrite_metadata_present'

  $Deletion = New-TestRepository $TestRoot
  Invoke-TestGit $Deletion.Path @('rm', '--quiet', 'WORKORDER.md') | Out-Null
  $DeletionHead = Commit-TestRepository $Deletion 'delete work order'
  Invoke-BoundaryCase 'Work Order deletion is full' $Deletion $Deletion.Anchor $DeletionHead (New-ValidPairFactory $Deletion.Anchor) 'full' 'no_status_transition'

  $Addition = New-TestRepository $TestRoot -WithoutWorkOrder
  Write-TestText (Join-Path $Addition.Path 'WORKORDER.md') (New-TestWorkOrderText)
  $AdditionHead = Commit-TestRepository $Addition 'add work order'
  Invoke-BoundaryCase 'Work Order addition is full' $Addition $Addition.Anchor $AdditionHead (New-ValidPairFactory $Addition.Anchor) 'full' 'no_status_transition'

  $Rename = New-TestRepository $TestRoot
  Invoke-TestGit $Rename.Path @('mv', 'WORKORDER.md', 'WORKORDER-renamed.md') | Out-Null
  $RenameHead = Commit-TestRepository $Rename 'rename work order'
  Invoke-BoundaryCase 'Work Order rename is full' $Rename $Rename.Anchor $RenameHead (New-ValidPairFactory $Rename.Anchor) 'full' 'no_status_transition'

  $Copy = New-TestRepository $TestRoot
  [System.IO.File]::Copy((Join-Path $Copy.Path 'WORKORDER.md'), (Join-Path $Copy.Path 'WORKORDER-copy.md'))
  Write-TestText (Join-Path $Copy.Path 'WORKORDER.md') (New-TestWorkOrderText -Status ' copied and changed')
  $CopyHead = Commit-TestRepository $Copy 'copy work order'
  Invoke-BoundaryCase 'Work Order copy is full' $Copy $Copy.Anchor $CopyHead (New-ValidPairFactory $Copy.Anchor) 'full' 'no_status_transition'

  $Mode = New-TestRepository $TestRoot
  Invoke-TestGit $Mode.Path @('update-index', '--chmod=+x', 'WORKORDER.md') | Out-Null
  Invoke-TestGit $Mode.Path @('commit', '--quiet', '-m', 'change mode') | Out-Null
  $ModeHead = Invoke-TestGit $Mode.Path @('rev-parse', 'HEAD')
  Invoke-BoundaryCase 'Work Order mode change is full' $Mode $Mode.Anchor $ModeHead (New-ValidPairFactory $Mode.Anchor) 'full' 'no_status_transition'

  foreach ($TypeCase in @(
    [pscustomobject]@{ Name = 'symlink'; Mode = '120000'; ObjectKind = 'blob' },
    [pscustomobject]@{ Name = 'submodule'; Mode = '160000'; ObjectKind = 'commit' }
  )) {
    $Repo = New-TestRepository $TestRoot
    if ($TypeCase.ObjectKind -eq 'blob') {
      Write-TestText (Join-Path $Repo.Path 'link-target.txt') "target`n"
      $ObjectId = Invoke-TestGit $Repo.Path @('hash-object', '-w', 'link-target.txt')
    } else {
      $ObjectId = $Repo.Anchor
    }
    Invoke-TestGit $Repo.Path @('rm', '--cached', '--quiet', 'WORKORDER.md') | Out-Null
    Invoke-TestGit $Repo.Path @('update-index', '--add', '--cacheinfo', "$($TypeCase.Mode),$ObjectId,WORKORDER.md") | Out-Null
    Invoke-TestGit $Repo.Path @('commit', '--quiet', '-m', "$($TypeCase.Name) replacement") | Out-Null
    $Head = Invoke-TestGit $Repo.Path @('rev-parse', 'HEAD')
    Invoke-BoundaryCase "Work Order $($TypeCase.Name) replacement is full" $Repo $Repo.Anchor $Head (New-ValidPairFactory $Repo.Anchor) 'full' 'no_status_transition'
  }

  $MalformedCases = @(
    [pscustomobject]@{ Name = 'missing status anchor'; Text = { (New-TestWorkOrderText -Status ' changed') -replace '(?m)^Status:', 'State:' }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'duplicate status anchor'; Text = { "Status: duplicate`n" + (New-TestWorkOrderText -Status ' changed') }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'moved owner anchor'; Text = { (New-TestWorkOrderText -Status ' changed') -replace "Owner: BDFL \(Ocean\)\.`n", '' -replace '(?m)^## Current authorization gate$', "Owner: BDFL (Ocean).`n## Current authorization gate" }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'reordered current gate anchor'; Text = { "## Current authorization gate`n" + ((New-TestWorkOrderText -Status ' changed') -replace '## Current authorization gate', '## Moved current gate') }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'altered gate heading'; Text = { (New-TestWorkOrderText -Status ' changed') -replace '## Current authorization gate', '## Current gate' }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'duplicate final marker'; Text = { (New-TestWorkOrderText -Status ' changed') + "<!-- workorder-current-authorization-gate:end -->`n" }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'content after final marker'; Text = { (New-TestWorkOrderText -Status ' changed') + "unauthorized tail`n" }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'conflict marker'; Text = { New-TestWorkOrderText -Status " changed`n<<<<<<< HEAD" }; Reason = 'no_status_transition' },
    [pscustomobject]@{ Name = 'trailing whitespace'; Text = { New-TestWorkOrderText -Status ' changed ' }; Reason = 'diff_hygiene_failed' }
  )
  foreach ($Malformed in $MalformedCases) {
    $Repo = New-TestRepository $TestRoot
    $Builder = $Malformed.Text
    Write-TestText (Join-Path $Repo.Path 'WORKORDER.md') (& $Builder)
    $Head = Commit-TestRepository $Repo $Malformed.Name
    Invoke-BoundaryCase "$($Malformed.Name) is full" $Repo $Repo.Anchor $Head (New-ValidPairFactory $Repo.Anchor) 'full' $Malformed.Reason
  }

  $Bom = New-TestRepository $TestRoot
  $BomText = New-TestWorkOrderText -Status ' changed'
  $Utf8Bytes = (New-Object System.Text.UTF8Encoding($false)).GetBytes($BomText)
  $BomBytes = New-Object byte[] ($Utf8Bytes.Length + 3)
  $BomBytes[0] = 0xEF; $BomBytes[1] = 0xBB; $BomBytes[2] = 0xBF
  [System.Array]::Copy($Utf8Bytes, 0, $BomBytes, 3, $Utf8Bytes.Length)
  [System.IO.File]::WriteAllBytes((Join-Path $Bom.Path 'WORKORDER.md'), $BomBytes)
  $BomHead = Commit-TestRepository $Bom 'BOM insertion'
  Invoke-BoundaryCase 'UTF-8 BOM insertion is full' $Bom $Bom.Anchor $BomHead (New-ValidPairFactory $Bom.Anchor) 'full' 'no_status_transition'

  $InvalidUtf8 = New-TestRepository $TestRoot
  [System.IO.File]::WriteAllBytes((Join-Path $InvalidUtf8.Path 'WORKORDER.md'), [byte[]](0xFF, 0xFE, 0x00))
  $InvalidUtf8Head = Commit-TestRepository $InvalidUtf8 'invalid UTF-8'
  Invoke-BoundaryCase 'malformed UTF-8 is full' $InvalidUtf8 $InvalidUtf8.Anchor $InvalidUtf8Head (New-ValidPairFactory $InvalidUtf8.Anchor) 'full' 'no_status_transition'

  $Merge = New-TestRepository $TestRoot
  $StatusParent = Add-TestStatusCommit $Merge ' before merge' "`nbefore merge`n"
  $Tree = Invoke-TestGit $Merge.Path @('rev-parse', "$StatusParent^{tree}")
  $MergeHead = Invoke-TestGit $Merge.Path @('commit-tree', $Tree, '-p', $StatusParent, '-p', $Merge.Anchor, '-m', 'merge')
  Invoke-TestGit $Merge.Path @('update-ref', 'refs/heads/main', $MergeHead) | Out-Null
  Invoke-BoundaryCase 'merge commit is full' $Merge $Merge.Anchor $MergeHead (New-ValidPairFactory $Merge.Anchor) 'full' 'event_range_not_linear'

  $MissingParent = New-TestRepository $TestRoot
  $Tree = Invoke-TestGit $MissingParent.Path @('rev-parse', 'HEAD^{tree}')
  $Missing = 'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee'
  $CommitText = "tree $Tree`nparent $Missing`nauthor Boundary Test <boundary@example.invalid> 1 +0000`ncommitter Boundary Test <boundary@example.invalid> 1 +0000`n`nmissing parent`n"
  $CommitObjectPath = Join-Path $MissingParent.Path 'missing-parent-commit.txt'
  Write-TestText $CommitObjectPath $CommitText
  $MissingParentHead = Invoke-TestGit $MissingParent.Path @('hash-object', '-t', 'commit', '-w', 'missing-parent-commit.txt')
  [System.IO.File]::Delete($CommitObjectPath)
  Invoke-TestGit $MissingParent.Path @('update-ref', 'refs/heads/main', $MissingParentHead) | Out-Null
  Invoke-BoundaryCase 'missing parent object is full' $MissingParent $MissingParent.Anchor $MissingParentHead (New-ValidPairFactory $MissingParent.Anchor) 'full' 'history_parent_unavailable'

  $Diverged = New-TestRepository $TestRoot
  $MainHead = Add-TestStatusCommit $Diverged ' main status' "`nmain status`n"
  $SideTree = Invoke-TestGit $Diverged.Path @('rev-parse', "$($Diverged.Anchor)^{tree}")
  $SideHead = Invoke-TestGit $Diverged.Path @('commit-tree', $SideTree, '-p', $Diverged.Anchor, '-m', 'side')
  Invoke-BoundaryCase 'non-ancestor base is full' $Diverged $SideHead $MainHead (New-ValidPairFactory $Diverged.Anchor) 'full' 'event_range_not_linear'

  $Reversed = New-TestRepository $TestRoot
  $ReversedDescendant = Add-TestStatusCommit $Reversed ' descendant' "`ndescendant`n"
  Invoke-TestGit $Reversed.Path @('update-ref', 'refs/heads/main', $Reversed.Anchor) | Out-Null
  Invoke-BoundaryCase 'reversed comparison range is full' $Reversed $ReversedDescendant $Reversed.Anchor (New-ValidPairFactory $Reversed.Anchor) 'full' 'event_range_not_linear'

  Write-Host "All $script:BoundaryTestCount Work Order status-boundary classifier cases passed twice deterministically."
} finally {
  $ResolvedTestRoot = [System.IO.Path]::GetFullPath($TestRoot)
  if (-not $ResolvedTestRoot.StartsWith($TempBase, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'refusing to remove a temporary path outside the system temp directory'
  }
  if (Test-Path -LiteralPath $ResolvedTestRoot) {
    foreach ($File in [System.IO.Directory]::EnumerateFiles($ResolvedTestRoot, '*', [System.IO.SearchOption]::AllDirectories)) {
      [System.IO.File]::SetAttributes($File, [System.IO.FileAttributes]::Normal)
    }
    [System.IO.Directory]::Delete($ResolvedTestRoot, $true)
  }
}
