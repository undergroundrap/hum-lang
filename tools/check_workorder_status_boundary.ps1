[CmdletBinding()]
param(
  [string] $Repository,
  [string] $WorkflowPath,
  [string] $EventName,
  [string] $EventRef,
  [string] $BaseCommit,
  [string] $HeadCommit
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:WorkOrderBoundaryDotSourced = $MyInvocation.InvocationName -eq '.'
$script:WorkOrderBoundaryWorkflow = '.github/workflows/ci.yml'
$script:WorkOrderBoundaryApiRoot = 'https://api.github.com'
$script:WorkOrderBoundaryApiVersion = '2026-03-10'
$script:WorkOrderBoundaryPageSize = 100

function Throw-WorkOrderBoundaryFailure {
  param([string] $Reason)

  throw [System.InvalidOperationException]::new("workorder-boundary:$Reason")
}

function New-WorkOrderBoundaryResult {
  param(
    [ValidateSet('fast', 'full')]
    [string] $Mode,
    [string] $Reason,
    [string] $Anchor = '',
    [long] $RunId = 0,
    [int] $RunAttempt = 0,
    [long] $UbuntuJobId = 0,
    [long] $WindowsJobId = 0,
    [string[]] $Transitions = @()
  )

  return [pscustomobject][ordered]@{
    Mode = $Mode
    Reason = $Reason
    Anchor = $Anchor
    RunId = $RunId
    RunAttempt = $RunAttempt
    UbuntuJobId = $UbuntuJobId
    WindowsJobId = $WindowsJobId
    Transitions = @($Transitions)
  }
}

function ConvertTo-WorkOrderBoundaryEvidence {
  param([object] $Result)

  $Transitions = @($Result.Transitions) -join ','
  return @(
    "mode=$($Result.Mode)"
    "reason=$($Result.Reason)"
    "anchor=$($Result.Anchor)"
    "run_id=$($Result.RunId)"
    "run_attempt=$($Result.RunAttempt)"
    "ubuntu_job_id=$($Result.UbuntuJobId)"
    "windows_job_id=$($Result.WindowsJobId)"
    "transitions=$Transitions"
  ) -join ';'
}

function Get-RequiredProperty {
  param(
    [object] $Object,
    [string] $Name,
    [string] $Reason
  )

  if ($null -eq $Object -or -not ($Object.PSObject.Properties.Name -contains $Name)) {
    Throw-WorkOrderBoundaryFailure $Reason
  }
  return $Object.$Name
}

function ConvertTo-ExactLong {
  param(
    [object] $Value,
    [string] $Reason,
    [switch] $Positive
  )

  $Parsed = [long]0
  if ($null -eq $Value -or -not [long]::TryParse([string]$Value, [ref]$Parsed)) {
    Throw-WorkOrderBoundaryFailure $Reason
  }
  if ($Positive -and $Parsed -le 0) {
    Throw-WorkOrderBoundaryFailure $Reason
  }
  return $Parsed
}

function Resolve-BoundaryGit {
  $Command = Get-Command git -ErrorAction SilentlyContinue
  if ($null -eq $Command) {
    Throw-WorkOrderBoundaryFailure 'git_unavailable'
  }
  return $Command.Source
}

function Invoke-BoundaryGit {
  param(
    [string] $RepoPath,
    [string[]] $Arguments
  )

  $Git = Resolve-BoundaryGit
  $PreviousPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $Output = & $Git --no-replace-objects -C $RepoPath @Arguments 2>&1
    $ExitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $PreviousPreference
  }

  return [pscustomobject]@{
    ExitCode = $ExitCode
    Lines = @($Output | ForEach-Object { [string]$_ })
    Text = (@($Output | ForEach-Object { [string]$_ }) -join "`n")
  }
}

function Invoke-BoundaryGitBytes {
  param(
    [string] $RepoPath,
    [string[]] $Arguments
  )

  foreach ($Argument in $Arguments) {
    if ($Argument -notmatch '^[A-Za-z0-9_./:{}^=+-]+$') {
      Throw-WorkOrderBoundaryFailure 'git_byte_argument_invalid'
    }
  }

  $StartInfo = New-Object System.Diagnostics.ProcessStartInfo
  $StartInfo.FileName = Resolve-BoundaryGit
  $StartInfo.WorkingDirectory = $RepoPath
  $StartInfo.Arguments = (@('--no-replace-objects') + $Arguments) -join ' '
  $StartInfo.UseShellExecute = $false
  $StartInfo.CreateNoWindow = $true
  $StartInfo.RedirectStandardOutput = $true
  $StartInfo.RedirectStandardError = $true

  $Process = New-Object System.Diagnostics.Process
  $Process.StartInfo = $StartInfo
  if (-not $Process.Start()) {
    Throw-WorkOrderBoundaryFailure 'git_byte_read_failed'
  }

  $ErrorTask = $Process.StandardError.ReadToEndAsync()
  $Bytes = New-Object System.IO.MemoryStream
  try {
    $Process.StandardOutput.BaseStream.CopyTo($Bytes)
    $Process.WaitForExit()
    $ErrorText = $ErrorTask.Result
    if ($Process.ExitCode -ne 0) {
      Throw-WorkOrderBoundaryFailure 'git_byte_read_failed'
    }
    return $Bytes.ToArray()
  } finally {
    $Bytes.Dispose()
    $Process.Dispose()
  }
}

function Assert-NoHistoryRewriteMetadata {
  param([string] $RepoPath)

  if (-not [string]::IsNullOrEmpty(
    [System.Environment]::GetEnvironmentVariable('GIT_REPLACE_REF_BASE', 'Process')
  )) {
    Throw-WorkOrderBoundaryFailure 'history_rewrite_metadata_present'
  }

  $ReplaceRefs = Invoke-BoundaryGit $RepoPath @(
    'for-each-ref', '--format=%(refname)', 'refs/replace/'
  )
  if ($ReplaceRefs.ExitCode -ne 0) {
    Throw-WorkOrderBoundaryFailure 'history_rewrite_metadata_unavailable'
  }
  if ($ReplaceRefs.Lines.Count -ne 0) {
    Throw-WorkOrderBoundaryFailure 'history_rewrite_metadata_present'
  }

  $GraftsPath = Invoke-BoundaryGit $RepoPath @(
    'rev-parse', '--path-format=absolute', '--git-path', 'info/grafts'
  )
  if ($GraftsPath.ExitCode -ne 0 -or $GraftsPath.Lines.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure 'history_rewrite_metadata_unavailable'
  }
  if (Test-Path -LiteralPath $GraftsPath.Lines[0]) {
    Throw-WorkOrderBoundaryFailure 'history_rewrite_metadata_present'
  }
}

function Resolve-ExactCommit {
  param(
    [string] $RepoPath,
    [string] $Commit,
    [string] $Reason
  )

  if ($Commit -notmatch '^[0-9a-fA-F]{40}$' -or $Commit -eq ('0' * 40)) {
    Throw-WorkOrderBoundaryFailure $Reason
  }

  $Type = Invoke-BoundaryGit $RepoPath @('cat-file', '-t', $Commit)
  if ($Type.ExitCode -ne 0 -or $Type.Lines.Count -ne 1 -or $Type.Lines[0] -ne 'commit') {
    Throw-WorkOrderBoundaryFailure $Reason
  }

  $Resolved = Invoke-BoundaryGit $RepoPath @('rev-parse', '--verify', "$Commit^{commit}")
  if ($Resolved.ExitCode -ne 0 -or $Resolved.Lines.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure $Reason
  }
  if ($Resolved.Lines[0].ToLowerInvariant() -ne $Commit.ToLowerInvariant()) {
    Throw-WorkOrderBoundaryFailure $Reason
  }
  return $Resolved.Lines[0].ToLowerInvariant()
}

function Get-CommitParents {
  param(
    [string] $RepoPath,
    [string] $Commit
  )

  $Result = Invoke-BoundaryGit $RepoPath @('rev-list', '--parents', '-n', '1', $Commit)
  if ($Result.ExitCode -ne 0 -or $Result.Lines.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure 'history_parent_unavailable'
  }
  $Parts = @($Result.Lines[0] -split '\s+' | Where-Object { $_ -ne '' })
  if ($Parts.Count -lt 1 -or $Parts[0].ToLowerInvariant() -ne $Commit.ToLowerInvariant()) {
    Throw-WorkOrderBoundaryFailure 'history_parent_unavailable'
  }
  return @($Parts | Select-Object -Skip 1 | ForEach-Object { $_.ToLowerInvariant() })
}

function Get-WorkOrderBlob {
  param(
    [string] $RepoPath,
    [string] $Commit
  )

  $Tree = Invoke-BoundaryGit $RepoPath @('ls-tree', $Commit, '--', 'WORKORDER.md')
  if ($Tree.ExitCode -ne 0 -or $Tree.Lines.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure 'workorder_object_invalid'
  }

  $Pattern = '^100644 blob (?<oid>[0-9a-f]{40})\tWORKORDER\.md$'
  if ($Tree.Lines[0] -notmatch $Pattern) {
    Throw-WorkOrderBoundaryFailure 'workorder_object_invalid'
  }

  $Bytes = Invoke-BoundaryGitBytes $RepoPath @('cat-file', 'blob', $Matches.oid)
  if ($Bytes.Length -ge 3 -and $Bytes[0] -eq 0xEF -and $Bytes[1] -eq 0xBB -and $Bytes[2] -eq 0xBF) {
    Throw-WorkOrderBoundaryFailure 'workorder_encoding_invalid'
  }

  try {
    $Utf8 = New-Object System.Text.UTF8Encoding($false, $true)
    $Text = $Utf8.GetString($Bytes)
  } catch {
    Throw-WorkOrderBoundaryFailure 'workorder_encoding_invalid'
  }

  return [pscustomobject]@{
    ObjectId = $Matches.oid
    Bytes = $Bytes
    Text = $Text
  }
}

function Get-UniqueRegexMatch {
  param(
    [string] $Text,
    [string] $Pattern
  )

  $Matches = [System.Text.RegularExpressions.Regex]::Matches($Text, $Pattern)
  if ($Matches.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure 'status_region_invalid'
  }
  return $Matches[0]
}

function Get-WorkOrderStatusProjection {
  param([string] $Text)

  if ([regex]::IsMatch($Text, '(?m)^(<<<<<<<|=======|>>>>>>>)')) {
    Throw-WorkOrderBoundaryFailure 'workorder_conflict_marker'
  }

  $Status = Get-UniqueRegexMatch $Text '(?m)^Status:'
  $Owner = Get-UniqueRegexMatch $Text '(?m)^Owner: BDFL \(Ocean\)\.\r?$'
  $Gate = Get-UniqueRegexMatch $Text '(?m)^## Current authorization gate\r?$'
  $Marker = Get-UniqueRegexMatch $Text '(?m)^<!-- workorder-current-authorization-gate:end -->\r?$'

  $HeaderStart = $Status.Index + 'Status:'.Length
  $HeaderEnd = $Owner.Index
  $GateHeadingLength = $Gate.Value.TrimEnd("`r").Length
  $GateStart = $Gate.Index + $GateHeadingLength
  $GateEnd = $Marker.Index
  if ($HeaderStart -ge $HeaderEnd -or $HeaderEnd -ge $Gate.Index -or $GateStart -ge $GateEnd) {
    Throw-WorkOrderBoundaryFailure 'status_region_invalid'
  }

  $HeaderBody = $Text.Substring($HeaderStart, $HeaderEnd - $HeaderStart)
  $GateBody = $Text.Substring($GateStart, $GateEnd - $GateStart)
  $BeforeHeader = $Text.Substring(0, $HeaderStart)
  $Between = $Text.Substring($HeaderEnd, $GateStart - $HeaderEnd)
  $AfterGate = $Text.Substring($GateEnd)
  $Normalized = $BeforeHeader + '<hum-status-header-body>' + $Between + '<hum-current-gate-body>' + $AfterGate

  return [pscustomobject]@{
    HeaderBody = $HeaderBody
    GateBody = $GateBody
    Normalized = $Normalized
  }
}

function Test-StatusOnlyTransition {
  param(
    [string] $RepoPath,
    [string] $Parent,
    [string] $Child
  )

  try {
    $Parents = @(Get-CommitParents $RepoPath $Child)
    if ($Parents.Count -ne 1 -or $Parents[0] -ne $Parent) {
      return [pscustomobject]@{ IsValid = $false; Reason = 'transition_not_linear' }
    }

    $Diff = Invoke-BoundaryGit $RepoPath @(
      'diff-tree', '--no-commit-id', '--raw', '-r', '--no-renames', $Parent, $Child, '--'
    )
    if ($Diff.ExitCode -ne 0 -or $Diff.Lines.Count -ne 1) {
      return [pscustomobject]@{ IsValid = $false; Reason = 'transition_path_invalid' }
    }

    $RawPattern = '^:100644 100644 [0-9a-f]{40} [0-9a-f]{40} M\tWORKORDER\.md$'
    if ($Diff.Lines[0] -notmatch $RawPattern) {
      return [pscustomobject]@{ IsValid = $false; Reason = 'transition_path_invalid' }
    }

    $ParentBlob = Get-WorkOrderBlob $RepoPath $Parent
    $ChildBlob = Get-WorkOrderBlob $RepoPath $Child
    $ParentProjection = Get-WorkOrderStatusProjection $ParentBlob.Text
    $ChildProjection = Get-WorkOrderStatusProjection $ChildBlob.Text
    if ($ParentProjection.Normalized -cne $ChildProjection.Normalized) {
      return [pscustomobject]@{ IsValid = $false; Reason = 'status_remainder_changed' }
    }
    if (
      $ParentProjection.HeaderBody -ceq $ChildProjection.HeaderBody -and
      $ParentProjection.GateBody -ceq $ChildProjection.GateBody
    ) {
      return [pscustomobject]@{ IsValid = $false; Reason = 'status_body_unchanged' }
    }

    return [pscustomobject]@{
      IsValid = $true
      Reason = 'status_transition_valid'
      Identity = "$Parent>$Child"
    }
  } catch {
    return [pscustomobject]@{ IsValid = $false; Reason = 'status_transition_invalid' }
  }
}

function Get-LinearEventRange {
  param(
    [string] $RepoPath,
    [string] $Base,
    [string] $Head
  )

  if ($Base -eq $Head) {
    Throw-WorkOrderBoundaryFailure 'event_range_empty'
  }

  $Current = $Head
  $Commits = New-Object System.Collections.Generic.List[string]
  $Commits.Add($Head)
  for ($Depth = 0; $Depth -lt 10000; $Depth += 1) {
    if ($Current -eq $Base) {
      return @($Commits | ForEach-Object { $_ })
    }
    $Parents = @(Get-CommitParents $RepoPath $Current)
    if ($Parents.Count -ne 1) {
      Throw-WorkOrderBoundaryFailure 'event_range_not_linear'
    }
    $Current = Resolve-ExactCommit $RepoPath $Parents[0] 'event_range_parent_missing'
    $Commits.Add($Current)
  }

  Throw-WorkOrderBoundaryFailure 'event_range_not_first_parent'
}

function Test-DiffHygiene {
  param(
    [string] $RepoPath,
    [string] $From,
    [string] $To
  )

  $Result = Invoke-BoundaryGit $RepoPath @('diff', '--check', $From, $To, '--')
  if ($Result.ExitCode -ne 0 -or $Result.Lines.Count -ne 0) {
    Throw-WorkOrderBoundaryFailure 'diff_hygiene_failed'
  }
}

function Get-CompletePageCollection {
  param(
    [object[]] $Pages,
    [string] $CollectionProperty,
    [string] $Reason
  )

  if ($Pages.Count -lt 1) {
    Throw-WorkOrderBoundaryFailure $Reason
  }

  $ExpectedTotal = $null
  $Items = New-Object System.Collections.Generic.List[object]
  for ($Index = 0; $Index -lt $Pages.Count; $Index += 1) {
    $Page = $Pages[$Index]
    $PageNumber = ConvertTo-ExactLong (Get-RequiredProperty $Page 'page_number' $Reason) $Reason -Positive
    if ($PageNumber -ne ($Index + 1)) {
      Throw-WorkOrderBoundaryFailure $Reason
    }
    $Total = ConvertTo-ExactLong (Get-RequiredProperty $Page 'total_count' $Reason) $Reason
    if ($Total -lt 0) {
      Throw-WorkOrderBoundaryFailure $Reason
    }
    if ($null -eq $ExpectedTotal) {
      $ExpectedTotal = $Total
    } elseif ($ExpectedTotal -ne $Total) {
      Throw-WorkOrderBoundaryFailure $Reason
    }

    $PageItems = @(Get-RequiredProperty $Page $CollectionProperty $Reason)
    if ($PageItems.Count -gt $script:WorkOrderBoundaryPageSize) {
      Throw-WorkOrderBoundaryFailure $Reason
    }
    foreach ($Item in $PageItems) {
      $Items.Add($Item)
    }
  }

  $ExpectedPages = [Math]::Max(1, [int][Math]::Ceiling($ExpectedTotal / [double]$script:WorkOrderBoundaryPageSize))
  if ($Pages.Count -ne $ExpectedPages -or $Items.Count -ne $ExpectedTotal) {
    Throw-WorkOrderBoundaryFailure $Reason
  }
  for ($Index = 0; $Index -lt ($Pages.Count - 1); $Index += 1) {
    $PageItems = @(Get-RequiredProperty $Pages[$Index] $CollectionProperty $Reason)
    if ($PageItems.Count -ne $script:WorkOrderBoundaryPageSize) {
      Throw-WorkOrderBoundaryFailure $Reason
    }
  }

  return [pscustomobject]@{
    Total = $ExpectedTotal
    PageCount = $Pages.Count
    Items = @($Items | ForEach-Object { $_ })
  }
}

function Get-ValidatedRunIdentity {
  param(
    [object[]] $RunPages,
    [string] $Candidate
  )

  $Collection = Get-CompletePageCollection $RunPages 'workflow_runs' 'run_pagination_invalid'
  $Runs = @($Collection.Items)
  if ($Runs.Count -eq 0) {
    Throw-WorkOrderBoundaryFailure 'anchor_run_missing'
  }

  $RunIds = @($Runs | ForEach-Object {
    ConvertTo-ExactLong (Get-RequiredProperty $_ 'id' 'anchor_run_invalid') 'anchor_run_invalid' -Positive
  } | Sort-Object -Unique)
  if ($RunIds.Count -ne 1 -or $Runs.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure 'anchor_run_ambiguous'
  }

  $Run = $Runs[0]
  $RunId = $RunIds[0]
  $AttemptValue = Get-RequiredProperty $Run 'run_attempt' 'anchor_run_invalid'
  $AttemptLong = ConvertTo-ExactLong $AttemptValue 'anchor_run_invalid' -Positive
  if ($AttemptLong -gt [int]::MaxValue) {
    Throw-WorkOrderBoundaryFailure 'anchor_run_invalid'
  }
  $Attempt = [int]$AttemptLong

  $Expected = [ordered]@{
    name = 'ci'
    path = $script:WorkOrderBoundaryWorkflow
    head_branch = 'main'
    head_sha = $Candidate
    event = 'push'
    status = 'completed'
    conclusion = 'success'
  }
  foreach ($Name in $Expected.Keys) {
    $Value = [string](Get-RequiredProperty $Run $Name 'anchor_run_invalid')
    if ($Value -cne [string]$Expected[$Name]) {
      Throw-WorkOrderBoundaryFailure 'anchor_run_invalid'
    }
  }

  $Composite = ([string]$Run.path) + '@' + ([string]$Run.head_branch)
  if ($Composite -cne '.github/workflows/ci.yml@main') {
    Throw-WorkOrderBoundaryFailure 'anchor_run_invalid'
  }

  return [pscustomobject][ordered]@{
    RunId = $RunId
    Attempt = $Attempt
    WorkflowName = [string]$Run.name
    WorkflowPath = [string]$Run.path
    HeadBranch = [string]$Run.head_branch
    HeadSha = [string]$Run.head_sha
    Event = [string]$Run.event
    Status = [string]$Run.status
    Conclusion = [string]$Run.conclusion
    RunPageCount = $Collection.PageCount
    RunRecordCount = $Collection.Total
  }
}

function Get-ValidatedStepIdentity {
  param(
    [object] $Job,
    [string] $Name,
    [string] $ExpectedConclusion
  )

  $Steps = @(Get-RequiredProperty $Job 'steps' 'anchor_steps_invalid')
  $Matches = @($Steps | Where-Object {
    (Get-RequiredProperty $_ 'name' 'anchor_steps_invalid') -ceq $Name
  })
  if ($Matches.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure 'anchor_steps_invalid'
  }
  $Step = $Matches[0]
  if (
    [string](Get-RequiredProperty $Step 'status' 'anchor_steps_invalid') -cne 'completed' -or
    [string](Get-RequiredProperty $Step 'conclusion' 'anchor_steps_invalid') -cne $ExpectedConclusion
  ) {
    Throw-WorkOrderBoundaryFailure 'anchor_steps_invalid'
  }
  return [pscustomobject][ordered]@{
    Name = $Name
    Status = 'completed'
    Conclusion = $ExpectedConclusion
  }
}

function Get-ValidatedJobIdentity {
  param(
    [object] $Job,
    [object] $Run,
    [string] $Candidate,
    [string] $ExpectedName,
    [string] $ExpectedPlatform
  )

  $JobId = ConvertTo-ExactLong (Get-RequiredProperty $Job 'id' 'anchor_jobs_invalid') 'anchor_jobs_invalid' -Positive
  $JobRunId = ConvertTo-ExactLong (Get-RequiredProperty $Job 'run_id' 'anchor_jobs_invalid') 'anchor_jobs_invalid' -Positive
  $JobAttempt = ConvertTo-ExactLong (Get-RequiredProperty $Job 'run_attempt' 'anchor_jobs_invalid') 'anchor_jobs_invalid' -Positive
  if ($JobRunId -ne $Run.RunId -or $JobAttempt -ne $Run.Attempt) {
    Throw-WorkOrderBoundaryFailure 'anchor_jobs_invalid'
  }
  if (
    [string](Get-RequiredProperty $Job 'name' 'anchor_jobs_invalid') -cne $ExpectedName -or
    [string](Get-RequiredProperty $Job 'head_sha' 'anchor_jobs_invalid') -cne $Candidate -or
    [string](Get-RequiredProperty $Job 'status' 'anchor_jobs_invalid') -cne 'completed' -or
    [string](Get-RequiredProperty $Job 'conclusion' 'anchor_jobs_invalid') -cne 'success'
  ) {
    Throw-WorkOrderBoundaryFailure 'anchor_jobs_invalid'
  }

  $Labels = @(Get-RequiredProperty $Job 'labels' 'anchor_jobs_invalid' | ForEach-Object { [string]$_ })
  if ($Labels.Count -ne 1 -or $Labels[0] -cne $ExpectedPlatform) {
    Throw-WorkOrderBoundaryFailure 'anchor_jobs_invalid'
  }

  $AllSteps = @(
    @(Get-RequiredProperty $Job 'steps' 'anchor_steps_invalid') | ForEach-Object {
      $StepName = [string](Get-RequiredProperty $_ 'name' 'anchor_steps_invalid')
      $StepStatus = [string](Get-RequiredProperty $_ 'status' 'anchor_steps_invalid')
      $StepConclusion = [string](Get-RequiredProperty $_ 'conclusion' 'anchor_steps_invalid')
      if ([string]::IsNullOrEmpty($StepName) -or [string]::IsNullOrEmpty($StepStatus) -or [string]::IsNullOrEmpty($StepConclusion)) {
        Throw-WorkOrderBoundaryFailure 'anchor_steps_invalid'
      }
      [pscustomobject][ordered]@{
        Name = $StepName
        Status = $StepStatus
        Conclusion = $StepConclusion
      }
    }
  )
  $FullStep = Get-ValidatedStepIdentity $Job 'Run Hum preflight' 'success'
  $FastStep = Get-ValidatedStepIdentity $Job 'Run status-only evidence' 'skipped'
  return [pscustomobject][ordered]@{
    JobId = $JobId
    RunId = $JobRunId
    RunAttempt = [int]$JobAttempt
    Name = $ExpectedName
    Platform = $ExpectedPlatform
    HeadSha = $Candidate
    Status = 'completed'
    Conclusion = 'success'
    Steps = @($AllSteps)
    FullStep = $FullStep
    FastStep = $FastStep
  }
}

function ConvertTo-ControlPlaneSnapshot {
  param(
    [object] $Snapshot,
    [string] $Candidate
  )

  $RunPages = @(Get-RequiredProperty $Snapshot 'RunPages' 'actions_snapshot_invalid')
  $JobPages = @(Get-RequiredProperty $Snapshot 'JobPages' 'actions_snapshot_invalid')
  $Run = Get-ValidatedRunIdentity $RunPages $Candidate
  $JobCollection = Get-CompletePageCollection $JobPages 'jobs' 'job_pagination_invalid'
  $Jobs = @($JobCollection.Items)
  if ($Jobs.Count -ne 2) {
    Throw-WorkOrderBoundaryFailure 'anchor_jobs_invalid'
  }

  $UbuntuMatches = @($Jobs | Where-Object {
    (Get-RequiredProperty $_ 'name' 'anchor_jobs_invalid') -ceq 'preflight (ubuntu-latest)'
  })
  $WindowsMatches = @($Jobs | Where-Object {
    (Get-RequiredProperty $_ 'name' 'anchor_jobs_invalid') -ceq 'preflight (windows-latest)'
  })
  if ($UbuntuMatches.Count -ne 1 -or $WindowsMatches.Count -ne 1) {
    Throw-WorkOrderBoundaryFailure 'anchor_jobs_invalid'
  }

  $Ubuntu = Get-ValidatedJobIdentity $UbuntuMatches[0] $Run $Candidate 'preflight (ubuntu-latest)' 'ubuntu-latest'
  $Windows = Get-ValidatedJobIdentity $WindowsMatches[0] $Run $Candidate 'preflight (windows-latest)' 'windows-latest'
  if ($Ubuntu.JobId -eq $Windows.JobId) {
    Throw-WorkOrderBoundaryFailure 'anchor_jobs_invalid'
  }

  return [pscustomobject][ordered]@{
    Run = $Run
    Ubuntu = $Ubuntu
    Windows = $Windows
    JobPageCount = $JobCollection.PageCount
    JobRecordCount = $JobCollection.Total
  }
}

function ConvertTo-ControlPlaneIdentity {
  param([object] $Snapshot)

  $Identity = [pscustomobject][ordered]@{
    run_id = $Snapshot.Run.RunId
    run_attempt = $Snapshot.Run.Attempt
    workflow_name = $Snapshot.Run.WorkflowName
    workflow_path = $Snapshot.Run.WorkflowPath
    head_branch = $Snapshot.Run.HeadBranch
    head_sha = $Snapshot.Run.HeadSha
    event = $Snapshot.Run.Event
    run_status = $Snapshot.Run.Status
    run_conclusion = $Snapshot.Run.Conclusion
    run_page_count = $Snapshot.Run.RunPageCount
    run_record_count = $Snapshot.Run.RunRecordCount
    ubuntu_job_id = $Snapshot.Ubuntu.JobId
    ubuntu_platform = $Snapshot.Ubuntu.Platform
    ubuntu_status = $Snapshot.Ubuntu.Status
    ubuntu_conclusion = $Snapshot.Ubuntu.Conclusion
    ubuntu_steps = @($Snapshot.Ubuntu.Steps)
    ubuntu_full_status = $Snapshot.Ubuntu.FullStep.Status
    ubuntu_full_conclusion = $Snapshot.Ubuntu.FullStep.Conclusion
    ubuntu_fast_status = $Snapshot.Ubuntu.FastStep.Status
    ubuntu_fast_conclusion = $Snapshot.Ubuntu.FastStep.Conclusion
    windows_job_id = $Snapshot.Windows.JobId
    windows_platform = $Snapshot.Windows.Platform
    windows_status = $Snapshot.Windows.Status
    windows_conclusion = $Snapshot.Windows.Conclusion
    windows_steps = @($Snapshot.Windows.Steps)
    windows_full_status = $Snapshot.Windows.FullStep.Status
    windows_full_conclusion = $Snapshot.Windows.FullStep.Conclusion
    windows_fast_status = $Snapshot.Windows.FastStep.Status
    windows_fast_conclusion = $Snapshot.Windows.FastStep.Conclusion
    job_page_count = $Snapshot.JobPageCount
    job_record_count = $Snapshot.JobRecordCount
  }
  return $Identity | ConvertTo-Json -Compress -Depth 8
}

function Invoke-GitHubApiGet {
  param(
    [string] $Uri,
    [hashtable] $Headers
  )

  try {
    return Invoke-RestMethod -Method Get -Uri $Uri -Headers $Headers -ErrorAction Stop
  } catch {
    Throw-WorkOrderBoundaryFailure 'actions_api_failure'
  }
}

function Get-GitHubPages {
  param(
    [string] $BaseUri,
    [string] $CollectionProperty,
    [hashtable] $Headers
  )

  $Pages = New-Object System.Collections.Generic.List[object]
  $ExpectedPages = $null
  for ($PageNumber = 1; $PageNumber -le 1000; $PageNumber += 1) {
    $Separator = if ($BaseUri.Contains('?')) { '&' } else { '?' }
    $Uri = "$BaseUri${Separator}per_page=$($script:WorkOrderBoundaryPageSize)&page=$PageNumber"
    $Response = Invoke-GitHubApiGet $Uri $Headers
    $Total = ConvertTo-ExactLong (Get-RequiredProperty $Response 'total_count' 'actions_api_schema_invalid') 'actions_api_schema_invalid'
    if ($Total -lt 0) {
      Throw-WorkOrderBoundaryFailure 'actions_api_schema_invalid'
    }
    $Items = @(Get-RequiredProperty $Response $CollectionProperty 'actions_api_schema_invalid')
    $Page = [ordered]@{
      page_number = $PageNumber
      total_count = $Total
    }
    $Page[$CollectionProperty] = @($Items)
    $Pages.Add([pscustomobject]$Page)

    $CurrentExpectedPages = [Math]::Max(1, [int][Math]::Ceiling($Total / [double]$script:WorkOrderBoundaryPageSize))
    if ($null -eq $ExpectedPages) {
      $ExpectedPages = $CurrentExpectedPages
    } elseif ($ExpectedPages -ne $CurrentExpectedPages) {
      Throw-WorkOrderBoundaryFailure 'actions_api_pagination_changed'
    }
    if ($PageNumber -eq $ExpectedPages) {
      return @($Pages | ForEach-Object { $_ })
    }
  }

  Throw-WorkOrderBoundaryFailure 'actions_api_pagination_incomplete'
}

function Get-ProductionActionsSnapshot {
  param(
    [string] $RepositoryIdentity,
    [string] $Candidate
  )

  if ($RepositoryIdentity -notmatch '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') {
    Throw-WorkOrderBoundaryFailure 'repository_identity_invalid'
  }
  if ([string]::IsNullOrWhiteSpace($env:GITHUB_TOKEN)) {
    Throw-WorkOrderBoundaryFailure 'actions_token_missing'
  }

  $Headers = @{
    Accept = 'application/vnd.github+json'
    Authorization = "Bearer $($env:GITHUB_TOKEN)"
    'X-GitHub-Api-Version' = $script:WorkOrderBoundaryApiVersion
    'User-Agent' = 'hum-ci-status-boundary'
  }
  $RunsUri = "$($script:WorkOrderBoundaryApiRoot)/repos/$RepositoryIdentity/actions/workflows/ci.yml/runs?head_sha=$Candidate&branch=main&event=push"
  $RunPages = @(Get-GitHubPages $RunsUri 'workflow_runs' $Headers)
  $Run = Get-ValidatedRunIdentity $RunPages $Candidate
  $JobsUri = "$($script:WorkOrderBoundaryApiRoot)/repos/$RepositoryIdentity/actions/runs/$($Run.RunId)/attempts/$($Run.Attempt)/jobs"
  $JobPages = @(Get-GitHubPages $JobsUri 'jobs' $Headers)
  return [pscustomobject]@{
    RunPages = @($RunPages)
    JobPages = @($JobPages)
  }
}

function Invoke-WorkOrderStatusClassificationCore {
  [CmdletBinding()]
  param(
    [string] $RepoPath,
    [string] $WorkflowPath,
    [string] $EventName,
    [string] $EventRef,
    [string] $BaseCommit,
    [string] $HeadCommit,
    [scriptblock] $ActionsEvidenceProvider
  )

  try {
    if ($WorkflowPath -cne $script:WorkOrderBoundaryWorkflow) {
      Throw-WorkOrderBoundaryFailure 'workflow_path_invalid'
    }
    if ($EventName -cne 'push') {
      Throw-WorkOrderBoundaryFailure 'event_not_push'
    }
    if ($EventRef -cne 'refs/heads/main') {
      Throw-WorkOrderBoundaryFailure 'event_not_main'
    }
    if ($null -eq $ActionsEvidenceProvider) {
      Throw-WorkOrderBoundaryFailure 'actions_provider_missing'
    }

    $ResolvedRepo = (Resolve-Path -LiteralPath $RepoPath).Path
    Assert-NoHistoryRewriteMetadata $ResolvedRepo
    $Base = Resolve-ExactCommit $ResolvedRepo $BaseCommit 'event_base_invalid'
    $Head = Resolve-ExactCommit $ResolvedRepo $HeadCommit 'event_head_invalid'
    $CheckoutHead = Invoke-BoundaryGit $ResolvedRepo @('rev-parse', '--verify', 'HEAD')
    if ($CheckoutHead.ExitCode -ne 0 -or $CheckoutHead.Lines.Count -ne 1 -or $CheckoutHead.Lines[0].ToLowerInvariant() -ne $Head) {
      Throw-WorkOrderBoundaryFailure 'checkout_head_mismatch'
    }

    $EventRange = @(Get-LinearEventRange $ResolvedRepo $Base $Head)
    $Current = $Head
    $ReverseTransitions = New-Object System.Collections.Generic.List[object]
    for ($Depth = 0; $Depth -lt 10000; $Depth += 1) {
      $Parents = @(Get-CommitParents $ResolvedRepo $Current)
      if ($Parents.Count -ne 1) {
        break
      }
      $Parent = Resolve-ExactCommit $ResolvedRepo $Parents[0] 'history_parent_unavailable'
      $Transition = Test-StatusOnlyTransition $ResolvedRepo $Parent $Current
      if (-not $Transition.IsValid) {
        break
      }
      $ReverseTransitions.Add([pscustomobject]@{
        Parent = $Parent
        Child = $Current
        Identity = $Transition.Identity
      })
      $Current = $Parent
    }

    if ($ReverseTransitions.Count -lt 1) {
      Throw-WorkOrderBoundaryFailure 'no_status_transition'
    }
    $Anchor = $Current
    $SuffixCommits = New-Object System.Collections.Generic.List[string]
    $SuffixCommits.Add($Anchor)
    for ($Index = $ReverseTransitions.Count - 1; $Index -ge 0; $Index -= 1) {
      $SuffixCommits.Add($ReverseTransitions[$Index].Child)
    }
    if (-not ($SuffixCommits -contains $Base)) {
      Throw-WorkOrderBoundaryFailure 'event_base_outside_status_suffix'
    }

    Test-DiffHygiene $ResolvedRepo $Anchor $Head
    Test-DiffHygiene $ResolvedRepo $Base $Head

    try {
      $FirstRaw = & $ActionsEvidenceProvider $Anchor 1
    } catch {
      Throw-WorkOrderBoundaryFailure 'actions_lookup_failed'
    }
    $First = ConvertTo-ControlPlaneSnapshot $FirstRaw $Anchor

    try {
      $SecondRaw = & $ActionsEvidenceProvider $Anchor 2
    } catch {
      Throw-WorkOrderBoundaryFailure 'actions_lookup_failed'
    }
    $Second = ConvertTo-ControlPlaneSnapshot $SecondRaw $Anchor
    if ((ConvertTo-ControlPlaneIdentity $First) -cne (ConvertTo-ControlPlaneIdentity $Second)) {
      Throw-WorkOrderBoundaryFailure 'control_plane_changed'
    }

    $OrderedTransitions = New-Object System.Collections.Generic.List[string]
    for ($Index = $ReverseTransitions.Count - 1; $Index -ge 0; $Index -= 1) {
      $OrderedTransitions.Add($ReverseTransitions[$Index].Identity)
    }
    $FastResult = @{
      Mode = 'fast'
      Reason = 'eligible_status_chain'
      Anchor = $Anchor
      RunId = $First.Run.RunId
      RunAttempt = $First.Run.Attempt
      UbuntuJobId = $First.Ubuntu.JobId
      WindowsJobId = $First.Windows.JobId
      Transitions = @($OrderedTransitions | ForEach-Object { $_ })
    }
    return New-WorkOrderBoundaryResult @FastResult
  } catch {
    $Reason = 'classifier_exception'
    if ($_.Exception.Message -match '^workorder-boundary:(?<reason>[a-z0-9_]+)$') {
      $Reason = $Matches.reason
    } else {
      Write-Verbose "status-boundary classifier exception: $($_.Exception.Message) $($_.ScriptStackTrace)"
    }
    return New-WorkOrderBoundaryResult -Mode 'full' -Reason $Reason
  }
}

function Invoke-ProductionWorkOrderStatusClassification {
  param(
    [string] $RepositoryIdentity,
    [string] $Workflow,
    [string] $Event,
    [string] $Ref,
    [string] $Base,
    [string] $Head
  )

  $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
  $script:WorkOrderBoundaryProductionRepository = $RepositoryIdentity
  $Provider = {
    param([string] $Candidate, [int] $SnapshotNumber)
    return Get-ProductionActionsSnapshot $script:WorkOrderBoundaryProductionRepository $Candidate
  }
  $CoreArguments = @{
    RepoPath = $RepoRoot
    WorkflowPath = $Workflow
    EventName = $Event
    EventRef = $Ref
    BaseCommit = $Base
    HeadCommit = $Head
    ActionsEvidenceProvider = $Provider
  }
  return Invoke-WorkOrderStatusClassificationCore @CoreArguments
}

if (-not $script:WorkOrderBoundaryDotSourced) {
  $ProductionArguments = @{
    RepositoryIdentity = $Repository
    Workflow = $WorkflowPath
    Event = $EventName
    Ref = $EventRef
    Base = $BaseCommit
    Head = $HeadCommit
  }
  $Result = Invoke-ProductionWorkOrderStatusClassification @ProductionArguments

  Write-Host "CI evidence classification: $(ConvertTo-WorkOrderBoundaryEvidence $Result)"
  Write-Output $Result.Mode
}
