$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ClassifierPath = Join-Path $PSScriptRoot 'check_workorder_status_boundary.ps1'
. $ClassifierPath

$script:BoundaryTestCount = 0
$script:BoundaryRepositorySerial = 0
$script:ExpectedPublishedBoundaryTestCount = 123
$script:ExpectedBoundaryTestCount = 151
$script:BoundaryCaseNames = New-Object System.Collections.Generic.List[string]
$script:UnitACaseResults = New-Object System.Collections.Generic.List[object]
$script:BoundaryActiveWorkOrderPath = 'WORKORDER_10.md'
$script:BoundaryInactiveWorkOrderPath = 'WORKORDER.md'
$script:BoundaryCanonicalActivePath = 'workorders/active/WORKORDER_21.md'
$script:BoundaryCanonicalClosedPath = 'workorders/closed/WORKORDER_20.md'

function Assert-BoundaryTest {
  param(
    [bool] $Condition,
    [string] $Message
  )

  if (-not $Condition) {
    throw $Message
  }
}

function Register-BoundaryCase {
  param([string] $Name)

  Assert-BoundaryTest (-not $script:BoundaryCaseNames.Contains($Name)) "duplicate boundary case name $Name"
  $script:BoundaryCaseNames.Add($Name)
  $script:BoundaryTestCount += 1
}

function Get-OrdinalUniqueCount {
  param([string[]] $Values)

  $Unique = New-Object 'System.Collections.Generic.HashSet[string]' (
    [System.StringComparer]::Ordinal
  )
  foreach ($Value in $Values) {
    [void]$Unique.Add($Value)
  }
  return $Unique.Count
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

function Write-TestBytes {
  param(
    [string] $Path,
    [byte[]] $Bytes
  )

  $Parent = Split-Path -Parent $Path
  if (-not (Test-Path -LiteralPath $Parent)) {
    [void][System.IO.Directory]::CreateDirectory($Parent)
  }
  [System.IO.File]::WriteAllBytes($Path, $Bytes)
}

function Get-TestRawMarkerLineCount {
  param([byte[]] $Bytes)

  [byte[]]$MarkerBytes = [System.Text.Encoding]::ASCII.GetBytes(
    $script:WorkOrderBoundaryActiveMarker
  )
  $Count = 0
  for ($Start = 0; $Start -le ($Bytes.Length - $MarkerBytes.Length); $Start += 1) {
    if ($Start -gt 0 -and $Bytes[$Start - 1] -ne 0x0A) {
      continue
    }
    $Matches = $true
    for ($Offset = 0; $Offset -lt $MarkerBytes.Length; $Offset += 1) {
      if ($Bytes[$Start + $Offset] -ne $MarkerBytes[$Offset]) {
        $Matches = $false
        break
      }
    }
    if (-not $Matches) {
      continue
    }
    $After = $Start + $MarkerBytes.Length
    if ($After -eq $Bytes.Length -or $Bytes[$After] -eq 0x0A) {
      $Count += 1
    }
  }
  return $Count
}

function Get-TestCommitBlobBytes {
  param(
    [object] $Repository,
    [string] $Commit,
    [string] $Path
  )

  $ObjectId = Invoke-TestGit $Repository.Path @('rev-parse', "$Commit`:$Path")
  return Invoke-BoundaryGitBytes $Repository.Path @('cat-file', 'blob', $ObjectId)
}

function New-TestWorkOrderText {
  param(
    [string] $Status = ' baseline',
    [string] $Gate = "`nbaseline authorization`n",
    [string] $Mandate = "## Session AP mandate`nExecutable requirements stay frozen.`n",
    [string] $Tail = "`n",
    [switch] $Inactive
  )

  return @(
    '# Test Work Order'
    ''
    'Date: 2026-07-14'
    if (-not $Inactive) {
      '<!-- hum-active-workorder:v1 -->'
    }
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
    Write-TestText (Join-Path $Path $script:BoundaryInactiveWorkOrderPath) (
      New-TestWorkOrderText -Inactive
    )
    Write-TestText (Join-Path $Path $script:BoundaryActiveWorkOrderPath) (New-TestWorkOrderText)
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
    [string] $Mandate = "## Session AP mandate`nExecutable requirements stay frozen.`n",
    [string] $WorkOrderPath = $script:BoundaryActiveWorkOrderPath
  )

  Write-TestText (Join-Path $Repository.Path $WorkOrderPath) (
    New-TestWorkOrderText -Status $Status -Gate $Gate -Mandate $Mandate
  )
  return Commit-TestRepository $Repository 'status update'
}

function New-CanonicalTestRepository {
  param(
    [string] $Root,
    [switch] $WithAdjacentPath
  )

  $Repository = New-TestRepository $Root -WithoutWorkOrder
  Write-TestText (Join-Path $Repository.Path $script:BoundaryCanonicalClosedPath) (
    New-TestWorkOrderText -Inactive
  )
  Write-TestText (Join-Path $Repository.Path $script:BoundaryCanonicalActivePath) (
    New-TestWorkOrderText
  )
  if ($WithAdjacentPath) {
    Write-TestText (Join-Path $Repository.Path 'workorders/active/WORKORDERING.md') (
      New-TestWorkOrderText -Status ' adjacent non-candidate' -Inactive
    )
  }
  $Repository.Anchor = Commit-TestRepository $Repository 'canonical full anchor'
  return $Repository
}

function New-UnitALegacyRepository {
  param([string] $Root)

  $Repository = New-TestRepository $Root -WithoutWorkOrder
  Write-TestText (Join-Path $Repository.Path 'WORKORDER_20.md') (
    New-TestWorkOrderText -Inactive
  )
  Write-TestText (Join-Path $Repository.Path 'WORKORDER_21.md') (
    New-TestWorkOrderText
  )
  $Repository.Anchor = Commit-TestRepository $Repository 'unit a legacy full anchor'
  return $Repository
}

function Assert-ResolvedActivePath {
  param(
    [object] $Repository,
    [string] $Commit,
    [string] $ExpectedPath,
    [string] $ControlName
  )

  $Resolved = Resolve-ActiveWorkOrderBlob $Repository.Path $Commit
  Assert-BoundaryTest ($Resolved.Path -ceq $ExpectedPath) "$ControlName resolved $($Resolved.Path), expected $ExpectedPath"
  Assert-BoundaryTest ($Resolved.ObjectId -cmatch '^[0-9a-f]{40}$') "$ControlName did not return an exact blob OID"
  Assert-BoundaryTest ($Resolved.Bytes.Length -gt 0) "$ControlName returned empty bytes"
  Assert-BoundaryTest ($Resolved.Text.Contains($script:WorkOrderBoundaryActiveMarker)) "$ControlName did not return marked active text"
}

function Move-TestPathExact {
  param(
    [object] $Repository,
    [string] $Source,
    [string] $Destination,
    [switch] $CaseOnly
  )

  if ($CaseOnly) {
    $Temporary = "$Source-unit-a-case-move"
    Invoke-TestGit $Repository.Path @('mv', $Source, $Temporary) | Out-Null
    Invoke-TestGit $Repository.Path @('mv', $Temporary, $Destination) | Out-Null
  } else {
    Invoke-TestGit $Repository.Path @('mv', $Source, $Destination) | Out-Null
  }
}

function Invoke-TestGitWithoutOutput {
  param(
    [string] $RepoPath,
    [string[]] $Arguments
  )

  $Git = (Get-Command git -ErrorAction Stop).Source
  $PreviousPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $Output = @(& $Git -C $RepoPath @Arguments 2>&1)
    $ExitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $PreviousPreference
  }
  Assert-BoundaryTest ($ExitCode -eq 0) "test git $($Arguments -join ' ') failed"
  Assert-BoundaryTest ($Output.Count -eq 0) "test git $($Arguments -join ' ') emitted output"
}

function Test-TestByteArrayEqual {
  param([byte[]] $First, [byte[]] $Second)

  if ($First.Length -ne $Second.Length) { return $false }
  for ($Index = 0; $Index -lt $First.Length; $Index += 1) {
    if ($First[$Index] -ne $Second[$Index]) { return $false }
  }
  return $true
}

function Get-TestA19EntryState {
  param(
    [object] $Repository,
    [string] $ObjectId,
    [string] $Commit = '',
    [AllowNull()][byte[]] $InventoryBytes = $null
  )

  Assert-BoundaryTest ($ObjectId -cmatch '^[0-9a-f]{40}$') 'A19 expected blob is invalid'
  $Arguments = if ($Commit -ceq '') {
    @('ls-files', '--stage', '-z')
  } else {
    @('ls-tree', '-r', '-z', '--full-tree', $Commit)
  }
  $ExpectedMetadata = if ($Commit -ceq '') {
    "100644 $ObjectId 0"
  } else {
    "100644 blob $ObjectId"
  }
  $MetadataPattern = if ($Commit -ceq '') {
    '^(?:100644|100755|120000|160000) [0-9a-f]{40} [0-3]$'
  } else {
    '^(?:100644 blob|100755 blob|120000 blob|040000 tree|160000 commit) [0-9a-f]{40}$'
  }
  [byte[]]$ExpectedPath = [System.Text.Encoding]::ASCII.GetBytes(
    'workorders\active\WORKORDER_21.md'
  )
  [byte[]]$CanonicalPath = [System.Text.Encoding]::ASCII.GetBytes(
    $script:BoundaryCanonicalActivePath
  )
  [byte[]]$Bytes = if ($null -eq $InventoryBytes) { Invoke-BoundaryGitBytes $Repository.Path $Arguments } else { $InventoryBytes }
  Assert-BoundaryTest (
    $Bytes.Length -gt 0 -and $Bytes[$Bytes.Length - 1] -eq 0
  ) 'A19 raw Git inventory is not NUL terminated'

  $TargetCount = 0
  $CanonicalCount = 0
  $TargetMetadataValid = $true
  $InventoryMetadataValid = $true
  $Start = 0
  for ($Index = 0; $Index -lt $Bytes.Length; $Index += 1) {
    if ($Bytes[$Index] -ne 0) { continue }
    Assert-BoundaryTest ($Index -gt $Start) 'A19 raw Git inventory has an empty record'
    [byte[]]$Record = $Bytes[$Start..($Index - 1)]
    $Tab = [System.Array]::IndexOf($Record, [byte]9)
    Assert-BoundaryTest (
      $Tab -gt 0 -and $Tab -lt ($Record.Length - 1)
    ) 'A19 raw Git inventory record is malformed'
    [byte[]]$MetadataBytes = $Record[0..($Tab - 1)]
    foreach ($Byte in $MetadataBytes) {
      Assert-BoundaryTest ($Byte -le 0x7F) 'A19 raw Git metadata is not ASCII'
    }
    $Metadata = [System.Text.Encoding]::ASCII.GetString($MetadataBytes)
    if ($Metadata -cnotmatch $MetadataPattern) { $InventoryMetadataValid = $false }
    [byte[]]$PathBytes = $Record[($Tab + 1)..($Record.Length - 1)]
    if (Test-TestByteArrayEqual $PathBytes $ExpectedPath) {
      $TargetCount += 1
      if ($Metadata -cne $ExpectedMetadata) { $TargetMetadataValid = $false }
    }
    if (Test-TestByteArrayEqual $PathBytes $CanonicalPath) {
      $CanonicalCount += 1
    }
    $Start = $Index + 1
  }
  Assert-BoundaryTest ($Start -eq $Bytes.Length) 'A19 raw Git inventory has trailing bytes'
  return [pscustomobject]@{
    IsValid = $InventoryMetadataValid -and $TargetCount -eq 1 -and $TargetMetadataValid -and $CanonicalCount -eq 0
    InventoryMetadataValid = $InventoryMetadataValid
    TargetCount = $TargetCount
    TargetMetadataValid = $TargetMetadataValid
    CanonicalCount = $CanonicalCount
  }
}

function Assert-TestA19MetadataGrammar {
  $ObjectId = ('1' * 40) -join ''; $TargetPath = 'workorders\active\WORKORDER_21.md'
  $Check = {
    param($Shape, [string] $OrdinaryMetadata, [string] $OrdinaryPath)
    $Text = "$($Shape.Target)`t$TargetPath`0$OrdinaryMetadata`t$OrdinaryPath`0"
    try {
      $State = Get-TestA19EntryState -Repository ([pscustomobject]@{ Path = '' }) `
        -ObjectId $ObjectId -Commit ([string]$Shape.Commit) `
        -InventoryBytes ([System.Text.Encoding]::UTF8.GetBytes($Text))
      return $State.IsValid -and $State.TargetCount -eq 1 -and $State.CanonicalCount -eq 0
    } catch { return $false }
  }
  foreach ($Shape in @(
    [pscustomobject]@{ Commit = ''; Target = "100644 $ObjectId 0"; Valid = @(
      "100644 $ObjectId 0", "100755 $ObjectId 1", "120000 $ObjectId 2", "160000 $ObjectId 3"
    ); Invalid = @(
      'malformed metadata', "10064x $ObjectId 0", "100644 $((('g' * 40) -join '')) 0", "100644 $ObjectId 4", "100644 $ObjectId", "100644 $ObjectId 0 extra", "100644é $ObjectId 0",
      "777777 $ObjectId 0", "040000 $ObjectId 0", "100664 $ObjectId 0"
    ) },
    [pscustomobject]@{ Commit = 'synthetic'; Target = "100644 blob $ObjectId"; Valid = @(
      "100644 blob $ObjectId", "100755 blob $ObjectId", "120000 blob $ObjectId", "040000 tree $ObjectId", "160000 commit $ObjectId"
    ); Invalid = @(
      'malformed metadata', "10064x blob $ObjectId", "100644 blob $((('g' * 40) -join ''))", "100644 mystery $ObjectId", "100644 $ObjectId", "100644 blob $ObjectId extra", "100644é blob $ObjectId",
      "100644 tree $ObjectId", "040000 blob $ObjectId", "160000 blob $ObjectId", "100644 commit $ObjectId", "100644 tag $ObjectId", "777777 blob $ObjectId"
    ) }
  )) {
    foreach ($Metadata in @($Shape.Valid)) {
      Assert-BoundaryTest (& $Check $Shape $Metadata 'ordinary.txt') 'A19 valid ordinary metadata changed target counts'
    }
    foreach ($Metadata in $Shape.Invalid) {
      Assert-BoundaryTest (-not (& $Check $Shape $Metadata 'ordinary.txt')) 'A19 malformed ordinary metadata was accepted'
    }
    Assert-BoundaryTest (-not (& $Check $Shape ([string]$Shape.Valid[0]) '')) 'A19 empty ordinary path was accepted'
  }
}
function Assert-TestA19Entry {
  param(
    [object] $Repository,
    [string] $ObjectId,
    [string] $Commit = ''
  )

  $State = Get-TestA19EntryState $Repository $ObjectId $Commit
  Assert-BoundaryTest $State.IsValid (
    "A19 literal entry authentication failed: target=$($State.TargetCount); " +
    "metadata=$($State.TargetMetadataValid); canonical=$($State.CanonicalCount)"
  )
  return $true
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
      (New-TestStep 'Generate evidence summary' 'success')
      (New-TestStep 'Upload evidence summary' 'success')
      (New-TestStep 'Upload hum-dev executable' 'success')
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
    [string] $WorkflowPath = '.github/workflows/ci.yml',
    [scriptblock] $ValidateResult
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
    if ($null -ne $ValidateResult) {
      & $ValidateResult $Result $Execution
    }
    $EvidenceRows.Add((ConvertTo-WorkOrderBoundaryEvidence $Result))
  }

  Assert-BoundaryTest ($EvidenceRows[0] -ceq $EvidenceRows[1]) "$Name was not byte-identical across two fresh executions"
  Register-BoundaryCase $Name
  Write-Host "ok $($script:BoundaryTestCount) - $Name => mode=$ExpectedMode;reason=$ExpectedReason"
}

function Invoke-UnitABoundaryCase {
  param(
    [string] $Name,
    [object] $Repository,
    [string] $Base,
    [string] $Head,
    [object] $PairFactory,
    [string] $ExpectedMode,
    [string] $ExpectedReason,
    [string] $ExpectedAnchor = '',
    [string[]] $ExpectedTransitions = @()
  )

  $Mode = $ExpectedMode
  $Anchor = $ExpectedAnchor
  $Transitions = @($ExpectedTransitions)
  $Assert = {
    param([bool] $Condition, [string] $Message)
    if (-not $Condition) {
      throw $Message
    }
  }
  $Validator = {
    param($Result, $Execution)
    if ($Mode -ceq 'full') {
      & $Assert ($Result.Anchor -ceq '') "$Name execution $Execution returned a full-lane anchor"
      & $Assert ($Result.RunId -eq 0) "$Name execution $Execution returned a full-lane run ID"
      & $Assert ($Result.RunAttempt -eq 0) "$Name execution $Execution returned a full-lane run attempt"
      & $Assert ($Result.UbuntuJobId -eq 0) "$Name execution $Execution returned a full-lane Ubuntu job ID"
      & $Assert ($Result.WindowsJobId -eq 0) "$Name execution $Execution returned a full-lane Windows job ID"
      & $Assert (@($Result.Transitions).Count -eq 0) "$Name execution $Execution returned full-lane transitions"
      return
    }
    & $Assert ($Result.Anchor -ceq $Anchor) "$Name execution $Execution returned the wrong fast anchor"
    & $Assert ($Result.RunId -eq 9001) "$Name execution $Execution returned the wrong fast run ID"
    & $Assert ($Result.RunAttempt -eq 1) "$Name execution $Execution returned the wrong fast run attempt"
    & $Assert ($Result.UbuntuJobId -eq 9101) "$Name execution $Execution returned the wrong Ubuntu job ID"
    & $Assert ($Result.WindowsJobId -eq 9102) "$Name execution $Execution returned the wrong Windows job ID"
    & $Assert (
      (@($Result.Transitions) -join "`n") -ceq ($Transitions -join "`n")
    ) "$Name execution $Execution returned the wrong ordered transition binding"
  }.GetNewClosure()

  Invoke-BoundaryCase -Name $Name -Repository $Repository -Base $Base -Head $Head `
    -PairFactory $PairFactory -ExpectedMode $ExpectedMode -ExpectedReason $ExpectedReason `
    -ExpectedAnchor $ExpectedAnchor -ValidateResult $Validator
  $script:UnitACaseResults.Add([pscustomobject]@{
    Name = $Name
    Mode = $ExpectedMode
    Reason = $ExpectedReason
  })
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
    'EvidenceProvider', 'Snapshot', 'Response', 'Fixture', 'Cache', 'ResultPath',
    'WorkOrder', 'WorkOrderPath', 'ActiveWorkOrder', 'ActivePath'
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
    'Run status-only evidence',
    'Generate evidence summary',
    'Upload evidence summary',
    'Upload hum-dev executable',
    'actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a',
    'actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c'
  )) {
    Assert-BoundaryTest $Workflow.Contains($RequiredText) "workflow is missing $RequiredText"
  }
  Assert-BoundaryTest (-not $Workflow.Contains('paths-ignore')) 'workflow must not use paths-ignore'
  Assert-BoundaryTest ($Workflow.IndexOf('Classify CI evidence lane') -lt $Workflow.IndexOf('Cache Cargo artifacts')) 'classification must precede Cargo cache setup'
  foreach ($RequiredTerminalStep in @('Generate evidence summary','Upload evidence summary','Upload hum-dev executable')) {
    $Snapshot = New-TestSnapshot ('a' * 40)
    ($Snapshot.JobPages[0].jobs[0].steps | Where-Object { $_.name -ceq $RequiredTerminalStep }).conclusion = 'failure'
    $Rejected = $false
    try { ConvertTo-ControlPlaneSnapshot $Snapshot ('a' * 40) | Out-Null } catch { $Rejected = $_.Exception.Message -ceq 'workorder-boundary:anchor_steps_invalid' }
    Assert-BoundaryTest $Rejected "terminal full step $RequiredTerminalStep did not fail closed"
  }

  $Classifier = [System.IO.File]::ReadAllText($ClassifierPath)
  foreach ($RequiredText in @(
    '--no-replace-objects',
    'refs/replace/',
    'info/grafts',
    'Resolve-ActiveWorkOrderBlob',
    '<!-- hum-active-workorder:v1 -->',
    '^WORKORDER(?:_[1-9][0-9]*)?\.md$',
    '^workorders/active/WORKORDER_[1-9][0-9]*\.md$',
    '^workorders/closed/WORKORDER_[1-9][0-9]*\.md$',
    '^(?:WORKORDER(?![A-Za-z]).*|workorders(?:[/\\]|$))',
    '(?:^|[/\\])WORKORDER(?![A-Za-z]).*$',
    "'ls-tree', '-r', '-z', '--full-tree'"
  )) {
    Assert-BoundaryTest $Classifier.Contains($RequiredText) "classifier is missing history-rewrite defense $RequiredText"
  }

  $ScopeAssignment = "`$script:WorkOrderBoundaryTopologyScopePattern = '^(?:WORKORDER(?![A-Za-z]).*|workorders(?:[/\\]|`$))'"
  $ScopeUse = '$IsWorkOrderLike = $IsWithinWorkOrderTopology -and [regex]::IsMatch('
  Assert-BoundaryTest ([regex]::Matches($Classifier, [regex]::Escape($ScopeAssignment)).Count -eq 1) 'classifier topology-scope owner is missing or duplicated'
  Assert-BoundaryTest ([regex]::Matches($Classifier, [regex]::Escape($ScopeUse)).Count -eq 1) 'classifier topology-scope consumer is missing or duplicated'
  foreach ($ScopeCorruption in @(
    $Classifier.Replace($ScopeAssignment, ''),
    $Classifier.Replace($ScopeAssignment, "`$script:WorkOrderBoundaryTopologyScopePattern = '(?:^|[/\\])WORKORDER(?![A-Za-z]).*`$'")
  )) {
    Assert-BoundaryTest ($ScopeCorruption -cne $Classifier) 'classifier topology-scope corruption did not initialize'
    $ScopeRejected = (
      [regex]::Matches($ScopeCorruption, [regex]::Escape($ScopeAssignment)).Count -ne 1 -or
      [regex]::Matches($ScopeCorruption, [regex]::Escape($ScopeUse)).Count -ne 1
    )
    Assert-BoundaryTest $ScopeRejected 'classifier topology-scope corruption did not fail closed'
  }

  $ExactUnitBStatus = Test-StatusOnlyTransition $RepoRoot '4252af3310663785e7ce6bf30e3a32b0b0177373' '8478f23a9d1a0446002f1ca76d5951e9d39f45e1'
  Assert-BoundaryTest $ExactUnitBStatus.IsValid 'exact Unit B publication-status transition was not recognized'

  $CheckAll = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'tools/check_all.ps1'))
  Assert-BoundaryTest ([regex]::Matches($CheckAll, "test_workorder_status_boundary\.ps1").Count -eq 1) 'full preflight must invoke the boundary matrix exactly once'
  Register-BoundaryCase 'production evidence seam and workflow source contract'
  Write-Host "ok $($script:BoundaryTestCount) - production evidence seam and workflow source contract"
}

function Assert-HistoricalAmendmentIsFull {
  $Parent = '450a8b4bec36b2a92253df207a21b5e62e853e5d'
  $Amendment = '505ce3095ca1d5ab6ada1eb375b8a0ca347812af'
  $Evidence = New-Object System.Collections.Generic.List[string]
  for ($Execution = 1; $Execution -le 2; $Execution += 1) {
    $Transition = Test-StatusOnlyTransition $RepoRoot $Parent $Amendment
    Assert-BoundaryTest (-not $Transition.IsValid) "505ce30 execution $Execution became status-only eligible"
    $Evidence.Add("$($Transition.IsValid):$($Transition.Reason)")
  }
  Assert-BoundaryTest ($Evidence[0] -ceq $Evidence[1]) '505ce30 ineligibility was not deterministic'
  Register-BoundaryCase 'exact 505ce30 mandate amendment remains ineligible'
  Write-Host "ok $($script:BoundaryTestCount) - exact 505ce30 mandate amendment remains ineligible => mode=full;reason=no_status_transition"
}

$TempBase = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
$TestRoot = [System.IO.Path]::GetFullPath((Join-Path $TempBase ("hum-workorder-boundary-{0}" -f [guid]::NewGuid().ToString('N'))))
Assert-BoundaryTest $TestRoot.StartsWith($TempBase, [System.StringComparison]::OrdinalIgnoreCase) 'temporary test root escaped the system temp directory'
[void][System.IO.Directory]::CreateDirectory($TestRoot)

try {
  Assert-ProductionSeamIsClosed
  Assert-HistoricalAmendmentIsFull

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
  $BlobBase = Invoke-TestGit $Valid.Path @('rev-parse', "$($Valid.Anchor):$script:BoundaryActiveWorkOrderPath")
  Invoke-BoundaryCase 'non-commit event base is full' $Valid $BlobBase $ValidHead $ValidFactory 'full' 'event_base_invalid'
  Invoke-BoundaryCase 'invalid head is full' $Valid $Valid.Anchor 'not-a-head' $ValidFactory 'full' 'event_head_invalid'
  Invoke-BoundaryCase 'checkout and proposed head disagreement is full' $Valid $Valid.Anchor $Valid.Anchor $ValidFactory 'full' 'checkout_head_mismatch'

  $Unauthorized = New-TestRepository $TestRoot
  $UnauthorizedHead = Add-TestStatusCommit $Unauthorized ' changed status' "`nchanged gate`n" "## Session AP mandate`nExecutable requirements were weakened.`n"
  Invoke-BoundaryCase 'current-amendment-shaped out-of-region edit is full' $Unauthorized $Unauthorized.Anchor $UnauthorizedHead (New-ValidPairFactory $Unauthorized.Anchor) 'full' 'no_status_transition'

  $InactiveWorkOrder = New-TestRepository $TestRoot
  Write-TestText (Join-Path $InactiveWorkOrder.Path $script:BoundaryInactiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' inactive changed' -Inactive
  )
  $InactiveWorkOrderHead = Commit-TestRepository $InactiveWorkOrder 'inactive work order change'
  Invoke-BoundaryCase 'inactive Work Order change is full' $InactiveWorkOrder $InactiveWorkOrder.Anchor $InactiveWorkOrderHead (New-ValidPairFactory $InactiveWorkOrder.Anchor) 'full' 'no_status_transition'

  $StatusPlusWorkOrder = New-TestRepository $TestRoot
  Write-TestText (Join-Path $StatusPlusWorkOrder.Path $script:BoundaryActiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' active status changed'
  )
  Write-TestText (Join-Path $StatusPlusWorkOrder.Path $script:BoundaryInactiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' inactive status changed' -Inactive
  )
  $StatusPlusWorkOrderHead = Commit-TestRepository $StatusPlusWorkOrder 'status plus another work order'
  Invoke-BoundaryCase 'status plus another Work Order is full' $StatusPlusWorkOrder $StatusPlusWorkOrder.Anchor $StatusPlusWorkOrderHead (New-ValidPairFactory $StatusPlusWorkOrder.Anchor) 'full' 'no_status_transition'

  $MissingActiveMarker = New-TestRepository $TestRoot
  $MissingMarkerText = (New-TestWorkOrderText -Status ' missing marker') -replace (
    '(?m)^' + [regex]::Escape($script:WorkOrderBoundaryActiveMarker) + "`n"
  ), ''
  Write-TestText (Join-Path $MissingActiveMarker.Path $script:BoundaryActiveWorkOrderPath) $MissingMarkerText
  $MissingActiveMarkerHead = Commit-TestRepository $MissingActiveMarker 'remove active marker'
  Invoke-BoundaryCase 'missing active marker is full' $MissingActiveMarker $MissingActiveMarker.Anchor $MissingActiveMarkerHead (New-ValidPairFactory $MissingActiveMarker.Anchor) 'full' 'no_status_transition'

  $AddedActiveMarker = New-TestRepository $TestRoot
  Write-TestText (Join-Path $AddedActiveMarker.Path $script:BoundaryActiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' markerless parent' -Inactive
  )
  $MarkerlessParent = Commit-TestRepository $AddedActiveMarker 'remove marker before probe'
  Write-TestText (Join-Path $AddedActiveMarker.Path $script:BoundaryActiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' marker added'
  )
  $AddedActiveMarkerHead = Commit-TestRepository $AddedActiveMarker 'add active marker'
  Invoke-BoundaryCase 'active marker addition is full' $AddedActiveMarker $MarkerlessParent $AddedActiveMarkerHead (New-ValidPairFactory $MarkerlessParent) 'full' 'no_status_transition'

  $DuplicateActiveMarker = New-TestRepository $TestRoot
  $DuplicateMarkerText = (New-TestWorkOrderText -Status ' duplicate marker') -replace (
    '(?m)^' + [regex]::Escape($script:WorkOrderBoundaryActiveMarker) + '$'
  ), "$script:WorkOrderBoundaryActiveMarker`n$script:WorkOrderBoundaryActiveMarker"
  Write-TestText (Join-Path $DuplicateActiveMarker.Path $script:BoundaryActiveWorkOrderPath) $DuplicateMarkerText
  $DuplicateActiveMarkerHead = Commit-TestRepository $DuplicateActiveMarker 'duplicate active marker'
  Invoke-BoundaryCase 'duplicate active marker is full' $DuplicateActiveMarker $DuplicateActiveMarker.Anchor $DuplicateActiveMarkerHead (New-ValidPairFactory $DuplicateActiveMarker.Anchor) 'full' 'no_status_transition'

  $MovedActiveMarker = New-TestRepository $TestRoot
  $MovedMarkerText = (New-TestWorkOrderText) -replace (
    '(?m)^' + [regex]::Escape($script:WorkOrderBoundaryActiveMarker) + "`n"
  ), ''
  $MovedMarkerText = $MovedMarkerText -replace (
    '(?m)^Status:.*$'
  ), "Status: marker moved`n$script:WorkOrderBoundaryActiveMarker"
  Write-TestText (Join-Path $MovedActiveMarker.Path $script:BoundaryActiveWorkOrderPath) $MovedMarkerText
  $MovedActiveMarkerHead = Commit-TestRepository $MovedActiveMarker 'move marker into mutable status region'
  Invoke-BoundaryCase 'active marker moved into mutable region is full' $MovedActiveMarker $MovedActiveMarker.Anchor $MovedActiveMarkerHead (New-ValidPairFactory $MovedActiveMarker.Anchor) 'full' 'no_status_transition'

  $SubstitutedActiveMarker = New-TestRepository $TestRoot
  $SubstitutedMarkerText = (New-TestWorkOrderText -Status ' marker substituted').Replace(
    $script:WorkOrderBoundaryActiveMarker,
    '<!-- hum-active-workorder:v2 -->'
  )
  Write-TestText (Join-Path $SubstitutedActiveMarker.Path $script:BoundaryActiveWorkOrderPath) $SubstitutedMarkerText
  $SubstitutedActiveMarkerHead = Commit-TestRepository $SubstitutedActiveMarker 'substitute active marker'
  Invoke-BoundaryCase 'substituted active marker is full' $SubstitutedActiveMarker $SubstitutedActiveMarker.Anchor $SubstitutedActiveMarkerHead (New-ValidPairFactory $SubstitutedActiveMarker.Anchor) 'full' 'no_status_transition'

  $TransferredMarker = New-TestRepository $TestRoot
  Write-TestText (Join-Path $TransferredMarker.Path $script:BoundaryInactiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' transferred marker copy'
  )
  $TransferredMarkerHead = Commit-TestRepository $TransferredMarker 'copy marker to another work order'
  Invoke-BoundaryCase 'marker transfer creating multiple active Work Orders is full' $TransferredMarker $TransferredMarker.Anchor $TransferredMarkerHead (New-ValidPairFactory $TransferredMarker.Anchor) 'full' 'no_status_transition'

  $ActivePathDisagreement = New-TestRepository $TestRoot
  Write-TestText (Join-Path $ActivePathDisagreement.Path $script:BoundaryActiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' former active' -Inactive
  )
  Write-TestText (Join-Path $ActivePathDisagreement.Path $script:BoundaryInactiveWorkOrderPath) (
    New-TestWorkOrderText -Status ' new active'
  )
  $ActivePathDisagreementHead = Commit-TestRepository $ActivePathDisagreement 'move active identity between work orders'
  Invoke-BoundaryCase 'parent and child active-path disagreement is full' $ActivePathDisagreement $ActivePathDisagreement.Anchor $ActivePathDisagreementHead (New-ValidPairFactory $ActivePathDisagreement.Anchor) 'full' 'no_status_transition'

  foreach ($InvalidWorkOrderPath in @(
    'WORKORDER_latest.md',
    'WORKORDER_11.md.bak',
    'WORKORDER_11.MD',
    'workorder_11.md',
    'WORKORDER11.md',
    'WORKORDER-11.md'
  )) {
    $MalformedWorkOrderPath = New-TestRepository $TestRoot
    Write-TestText (Join-Path $MalformedWorkOrderPath.Path $InvalidWorkOrderPath) (
      New-TestWorkOrderText -Status ' invalid candidate path'
    )
    $MalformedWorkOrderAnchor = Commit-TestRepository $MalformedWorkOrderPath "add malformed Work Order path $InvalidWorkOrderPath"
    $MalformedWorkOrderHead = Add-TestStatusCommit $MalformedWorkOrderPath ' valid status beside malformed path' "`nnext session remains unauthorized`n"
    Invoke-BoundaryCase "malformed Work Order path $InvalidWorkOrderPath is full" $MalformedWorkOrderPath $MalformedWorkOrderAnchor $MalformedWorkOrderHead (New-ValidPairFactory $MalformedWorkOrderAnchor) 'full' 'no_status_transition'
  }

  $AdjacentUnrelatedPath = New-TestRepository $TestRoot
  Write-TestText (Join-Path $AdjacentUnrelatedPath.Path 'WORKORDERING.md') (
    New-TestWorkOrderText -Status ' unrelated adjacent name' -Inactive
  )
  Write-TestText (Join-Path $AdjacentUnrelatedPath.Path 'crates/hum-dev/src/workorder.rs') "pub fn unrelated_module() {}`n"
  Write-TestText (Join-Path $AdjacentUnrelatedPath.Path 'src/nested/WORKORDER_99.md') "unrelated nested source module`n"
  $AdjacentUnrelatedAnchor = Commit-TestRepository $AdjacentUnrelatedPath 'add unrelated adjacent path'
  $AdjacentUnrelatedHead = Add-TestStatusCommit $AdjacentUnrelatedPath ' valid status beside unrelated path' "`nnext session remains unauthorized`n"
  Invoke-BoundaryCase 'unrelated nested workorder-like source paths remain fast' $AdjacentUnrelatedPath $AdjacentUnrelatedAnchor $AdjacentUnrelatedHead (New-ValidPairFactory $AdjacentUnrelatedAnchor) 'fast' 'eligible_status_chain' $AdjacentUnrelatedAnchor

  foreach ($PathCase in @(
    [pscustomobject]@{ Name = 'Rust source'; Path = 'src/main.rs'; Text = ('fn main() { println!("changed"); }' + "`n") },
    [pscustomobject]@{ Name = 'fixture'; Path = 'fixtures/base.hum'; Text = "task changed() -> Unit`n" },
    [pscustomobject]@{ Name = 'Cargo'; Path = 'Cargo.toml'; Text = ('[package]' + "`n" + 'name = "changed"' + "`n" + 'version = "0.0.0"' + "`n") },
    [pscustomobject]@{ Name = 'tool'; Path = 'tools/check_all.ps1'; Text = "Write-Host changed`n" },
    [pscustomobject]@{ Name = 'workflow'; Path = '.github/workflows/ci.yml'; Text = "name: changed`n" },
    [pscustomobject]@{ Name = 'schema'; Path = 'schemas/example.json'; Text = "{}`n" },
    [pscustomobject]@{ Name = 'generated output'; Path = 'generated/output.txt'; Text = "changed`n" }
  )) {
    $Repo = New-TestRepository $TestRoot
    Write-TestText (Join-Path $Repo.Path $PathCase.Path) $PathCase.Text
    $Head = Commit-TestRepository $Repo "$($PathCase.Name) change"
    Invoke-BoundaryCase "$($PathCase.Name) change is full" $Repo $Repo.Anchor $Head (New-ValidPairFactory $Repo.Anchor) 'full' 'no_status_transition'
  }

  $Multiple = New-TestRepository $TestRoot
  Write-TestText (Join-Path $Multiple.Path $script:BoundaryActiveWorkOrderPath) (New-TestWorkOrderText -Status ' status plus source')
  Write-TestText (Join-Path $Multiple.Path 'src/main.rs') ('fn main() { println!("changed"); }' + "`n")
  $MultipleHead = Commit-TestRepository $Multiple 'multiple paths'
  Invoke-BoundaryCase 'status plus code change is full' $Multiple $Multiple.Anchor $MultipleHead (New-ValidPairFactory $Multiple.Anchor) 'full' 'no_status_transition'

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
  Write-TestText (Join-Path $ReplacedHistory.Path $script:BoundaryActiveWorkOrderPath) (
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
  Invoke-TestGit $Deletion.Path @('rm', '--quiet', $script:BoundaryActiveWorkOrderPath) | Out-Null
  $DeletionHead = Commit-TestRepository $Deletion 'delete work order'
  Invoke-BoundaryCase 'Work Order deletion is full' $Deletion $Deletion.Anchor $DeletionHead (New-ValidPairFactory $Deletion.Anchor) 'full' 'no_status_transition'

  $Addition = New-TestRepository $TestRoot -WithoutWorkOrder
  Write-TestText (Join-Path $Addition.Path $script:BoundaryActiveWorkOrderPath) (New-TestWorkOrderText)
  $AdditionHead = Commit-TestRepository $Addition 'add work order'
  Invoke-BoundaryCase 'Work Order addition is full' $Addition $Addition.Anchor $AdditionHead (New-ValidPairFactory $Addition.Anchor) 'full' 'no_status_transition'

  $Rename = New-TestRepository $TestRoot
  Invoke-TestGit $Rename.Path @('mv', $script:BoundaryActiveWorkOrderPath, 'WORKORDER_11.md') | Out-Null
  $RenameHead = Commit-TestRepository $Rename 'rename work order'
  Invoke-BoundaryCase 'Work Order rename is full' $Rename $Rename.Anchor $RenameHead (New-ValidPairFactory $Rename.Anchor) 'full' 'no_status_transition'

  $Copy = New-TestRepository $TestRoot
  [System.IO.File]::Copy(
    (Join-Path $Copy.Path $script:BoundaryActiveWorkOrderPath),
    (Join-Path $Copy.Path 'WORKORDER_11.md')
  )
  Write-TestText (Join-Path $Copy.Path $script:BoundaryActiveWorkOrderPath) (New-TestWorkOrderText -Status ' copied and changed')
  $CopyHead = Commit-TestRepository $Copy 'copy work order'
  Invoke-BoundaryCase 'Work Order copy is full' $Copy $Copy.Anchor $CopyHead (New-ValidPairFactory $Copy.Anchor) 'full' 'no_status_transition'

  $Mode = New-TestRepository $TestRoot
  Invoke-TestGit $Mode.Path @('update-index', '--chmod=+x', $script:BoundaryActiveWorkOrderPath) | Out-Null
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
    Invoke-TestGit $Repo.Path @('rm', '--cached', '--quiet', $script:BoundaryActiveWorkOrderPath) | Out-Null
    Invoke-TestGit $Repo.Path @(
      'update-index', '--add', '--cacheinfo',
      "$($TypeCase.Mode),$ObjectId,$script:BoundaryActiveWorkOrderPath"
    ) | Out-Null
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
    Write-TestText (Join-Path $Repo.Path $script:BoundaryActiveWorkOrderPath) (& $Builder)
    $Head = Commit-TestRepository $Repo $Malformed.Name
    Invoke-BoundaryCase "$($Malformed.Name) is full" $Repo $Repo.Anchor $Head (New-ValidPairFactory $Repo.Anchor) 'full' $Malformed.Reason
  }

  $Bom = New-TestRepository $TestRoot
  $BomText = New-TestWorkOrderText -Status ' changed'
  $Utf8Bytes = (New-Object System.Text.UTF8Encoding($false)).GetBytes($BomText)
  $BomBytes = New-Object byte[] ($Utf8Bytes.Length + 3)
  $BomBytes[0] = 0xEF; $BomBytes[1] = 0xBB; $BomBytes[2] = 0xBF
  [System.Array]::Copy($Utf8Bytes, 0, $BomBytes, 3, $Utf8Bytes.Length)
  [System.IO.File]::WriteAllBytes((Join-Path $Bom.Path $script:BoundaryActiveWorkOrderPath), $BomBytes)
  $BomHead = Commit-TestRepository $Bom 'BOM insertion'
  Invoke-BoundaryCase 'UTF-8 BOM insertion is full' $Bom $Bom.Anchor $BomHead (New-ValidPairFactory $Bom.Anchor) 'full' 'no_status_transition'

  $InvalidUtf8 = New-TestRepository $TestRoot
  [System.IO.File]::WriteAllBytes(
    (Join-Path $InvalidUtf8.Path $script:BoundaryActiveWorkOrderPath),
    [byte[]](0xFF, 0xFE, 0x00)
  )
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

  Assert-BoundaryTest (
    $script:BoundaryTestCount -eq $script:ExpectedPublishedBoundaryTestCount
  ) "published Work Order status-boundary inventory changed from 123 to $script:BoundaryTestCount"
  $PublishedBoundaryNames = @($script:BoundaryCaseNames | ForEach-Object { $_ })
  Assert-BoundaryTest (
    (Get-OrdinalUniqueCount $PublishedBoundaryNames) -eq 123
  ) 'published Work Order status-boundary inventory contains duplicate names'

  $A01 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A01 $A01.Anchor $script:BoundaryCanonicalActivePath 'A01 parent control'
  $A01Head = Add-TestStatusCommit -Repository $A01 -Status ' canonical header update' `
    -Gate "`nbaseline authorization`n" -WorkOrderPath $script:BoundaryCanonicalActivePath
  Assert-ResolvedActivePath $A01 $A01Head $script:BoundaryCanonicalActivePath 'A01 child control'
  Invoke-UnitABoundaryCase -Name 'canonical_nested_header_only_fast' -Repository $A01 `
    -Base $A01.Anchor -Head $A01Head -PairFactory (New-ValidPairFactory $A01.Anchor) `
    -ExpectedMode 'fast' -ExpectedReason 'eligible_status_chain' -ExpectedAnchor $A01.Anchor `
    -ExpectedTransitions @("$($A01.Anchor)>$A01Head")

  $A02 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A02 $A02.Anchor $script:BoundaryCanonicalActivePath 'A02 parent control'
  $A02Head = Add-TestStatusCommit -Repository $A02 -Status ' baseline' `
    -Gate "`ncanonical gate update`n" -WorkOrderPath $script:BoundaryCanonicalActivePath
  Assert-ResolvedActivePath $A02 $A02Head $script:BoundaryCanonicalActivePath 'A02 child control'
  Invoke-UnitABoundaryCase -Name 'canonical_nested_gate_only_fast' -Repository $A02 `
    -Base $A02.Anchor -Head $A02Head -PairFactory (New-ValidPairFactory $A02.Anchor) `
    -ExpectedMode 'fast' -ExpectedReason 'eligible_status_chain' -ExpectedAnchor $A02.Anchor `
    -ExpectedTransitions @("$($A02.Anchor)>$A02Head")

  $A03 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A03 $A03.Anchor $script:BoundaryCanonicalActivePath 'A03 anchor control'
  $A03First = Add-TestStatusCommit -Repository $A03 -Status ' canonical first status' `
    -Gate "`nbaseline authorization`n" -WorkOrderPath $script:BoundaryCanonicalActivePath
  $A03Second = Add-TestStatusCommit -Repository $A03 -Status ' canonical first status' `
    -Gate "`ncanonical second gate`n" -WorkOrderPath $script:BoundaryCanonicalActivePath
  Assert-ResolvedActivePath $A03 $A03First $script:BoundaryCanonicalActivePath 'A03 first child control'
  Assert-ResolvedActivePath $A03 $A03Second $script:BoundaryCanonicalActivePath 'A03 second child control'
  Invoke-UnitABoundaryCase -Name 'canonical_nested_two_commit_chain_fast' -Repository $A03 `
    -Base $A03First -Head $A03Second -PairFactory (New-ValidPairFactory $A03.Anchor) `
    -ExpectedMode 'fast' -ExpectedReason 'eligible_status_chain' -ExpectedAnchor $A03.Anchor `
    -ExpectedTransitions @("$($A03.Anchor)>$A03First", "$A03First>$A03Second")

  $A04 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A04 $A04.Anchor $script:BoundaryCanonicalActivePath 'A04 parent control'
  Write-TestText (Join-Path $A04.Path $script:BoundaryCanonicalActivePath) (
    New-TestWorkOrderText -Mandate "## Session AP mandate`nCanonical immutable requirements changed.`n"
  )
  $A04Head = Commit-TestRepository $A04 'canonical immutable change'
  Assert-ResolvedActivePath $A04 $A04Head $script:BoundaryCanonicalActivePath 'A04 child control'
  Invoke-UnitABoundaryCase -Name 'canonical_non_status_full' -Repository $A04 -Base $A04.Anchor `
    -Head $A04Head -PairFactory (New-ValidPairFactory $A04.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A05 = New-UnitALegacyRepository $TestRoot
  Assert-ResolvedActivePath $A05 $A05.Anchor 'WORKORDER_21.md' 'A05 legacy parent control'
  [void][System.IO.Directory]::CreateDirectory((Join-Path $A05.Path 'workorders/active'))
  [void][System.IO.Directory]::CreateDirectory((Join-Path $A05.Path 'workorders/closed'))
  Move-TestPathExact $A05 'WORKORDER_20.md' $script:BoundaryCanonicalClosedPath
  Move-TestPathExact $A05 'WORKORDER_21.md' $script:BoundaryCanonicalActivePath
  $A05Head = Commit-TestRepository $A05 'legacy to canonical migration'
  Assert-ResolvedActivePath $A05 $A05Head $script:BoundaryCanonicalActivePath 'A05 canonical child control'
  Invoke-UnitABoundaryCase -Name 'legacy_to_canonical_migration_full' -Repository $A05 `
    -Base $A05.Anchor -Head $A05Head -PairFactory (New-ValidPairFactory $A05.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A06 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A06 $A06.Anchor $script:BoundaryCanonicalActivePath 'A06 predecessor control'
  Move-TestPathExact $A06 $script:BoundaryCanonicalActivePath 'workorders/closed/WORKORDER_21.md'
  $A06Closed = [System.IO.File]::ReadAllText((Join-Path $A06.Path 'workorders/closed/WORKORDER_21.md'))
  Write-TestText (Join-Path $A06.Path 'workorders/closed/WORKORDER_21.md') (
    $A06Closed.Replace("$script:WorkOrderBoundaryActiveMarker`n", '')
  )
  Write-TestText (Join-Path $A06.Path 'workorders/active/WORKORDER_22.md') (
    New-TestWorkOrderText -Status ' successor issued'
  )
  $A06Head = Commit-TestRepository $A06 'canonical successor issuance'
  Assert-ResolvedActivePath $A06 $A06Head 'workorders/active/WORKORDER_22.md' 'A06 successor control'
  Invoke-UnitABoundaryCase -Name 'canonical_successor_issuance_full' -Repository $A06 `
    -Base $A06.Anchor -Head $A06Head -PairFactory (New-ValidPairFactory $A06.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A07 = New-CanonicalTestRepository $TestRoot -WithAdjacentPath
  Assert-ResolvedActivePath $A07 $A07.Anchor $script:BoundaryCanonicalActivePath 'A07 parent control'
  $A07Head = Add-TestStatusCommit -Repository $A07 -Status ' adjacent path ignored' `
    -Gate "`nadjacent path remains unrelated`n" -WorkOrderPath $script:BoundaryCanonicalActivePath
  Assert-ResolvedActivePath $A07 $A07Head $script:BoundaryCanonicalActivePath 'A07 child control'
  Invoke-UnitABoundaryCase -Name 'canonical_adjacent_workordering_ignored' -Repository $A07 `
    -Base $A07.Anchor -Head $A07Head -PairFactory (New-ValidPairFactory $A07.Anchor) `
    -ExpectedMode 'fast' -ExpectedReason 'eligible_status_chain' -ExpectedAnchor $A07.Anchor `
    -ExpectedTransitions @("$($A07.Anchor)>$A07Head")

  $A08 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A08 $A08.Anchor $script:BoundaryCanonicalActivePath 'A08 honest control'
  Write-TestText (Join-Path $A08.Path $script:BoundaryCanonicalActivePath) (New-TestWorkOrderText -Inactive)
  Write-TestText (Join-Path $A08.Path $script:BoundaryCanonicalClosedPath) (New-TestWorkOrderText)
  $A08Head = Commit-TestRepository $A08 'move marker into closed work order'
  Invoke-UnitABoundaryCase -Name 'closed_marker_rejected' -Repository $A08 -Base $A08.Anchor `
    -Head $A08Head -PairFactory (New-ValidPairFactory $A08.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A09 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A09 $A09.Anchor $script:BoundaryCanonicalActivePath 'A09 honest control'
  Invoke-TestGit $A09.Path @('rm', '--quiet', $script:BoundaryCanonicalActivePath) | Out-Null
  [void][System.IO.Directory]::CreateDirectory((Join-Path $A09.Path 'workorders/active'))
  [System.IO.File]::Copy(
    (Join-Path $A09.Path $script:BoundaryCanonicalClosedPath),
    (Join-Path $A09.Path 'workorders/active/WORKORDER_20.md')
  )
  $A09Head = Commit-TestRepository $A09 'copy closed work order into active'
  Invoke-UnitABoundaryCase -Name 'closed_copy_cannot_become_active' -Repository $A09 `
    -Base $A09.Anchor -Head $A09Head -PairFactory (New-ValidPairFactory $A09.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A10 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A10 $A10.Anchor $script:BoundaryCanonicalActivePath 'A10 honest control'
  Write-TestText (Join-Path $A10.Path 'workorders/active/WORKORDER_22.md') (New-TestWorkOrderText -Inactive)
  $A10Head = Commit-TestRepository $A10 'add second active candidate'
  Invoke-UnitABoundaryCase -Name 'two_active_candidates_rejected' -Repository $A10 `
    -Base $A10.Anchor -Head $A10Head -PairFactory (New-ValidPairFactory $A10.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A11 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A11 $A11.Anchor $script:BoundaryCanonicalActivePath 'A11 State 1 control'
  Assert-BoundaryTest (
    (Invoke-TestGit $A11.Path @('ls-tree', '-r', '--name-only', $A11.Anchor, '--', 'hidden.bin')) -ceq ''
  ) 'A11 State 1 unexpectedly contains hidden.bin'
  $A11ActiveObject = Invoke-TestGit $A11.Path @(
    'rev-parse', "$($A11.Anchor):$script:BoundaryCanonicalActivePath"
  )

  [byte[]]$A11HiddenBytes = @([byte]0x00, [byte]0x41, [byte]0x0A) +
    [System.Text.Encoding]::ASCII.GetBytes($script:WorkOrderBoundaryActiveMarker) +
    @([byte]0x0A, [byte]0x42, [byte]0x00)
  Write-TestBytes (Join-Path $A11.Path 'hidden.bin') $A11HiddenBytes
  $A11CorruptionAnchor = Commit-TestRepository $A11 'add binary marker corruption anchor'
  $A11State2Diff = Invoke-TestGit $A11.Path @(
    'diff-tree', '--no-commit-id', '--name-status', '-r', '--no-renames',
    $A11.Anchor, $A11CorruptionAnchor, '--'
  )
  Assert-BoundaryTest ($A11State2Diff -ceq "A`thidden.bin") 'A11 State 2 did not add only hidden.bin'
  $A11HiddenEntry = Invoke-TestGit $A11.Path @('ls-tree', $A11CorruptionAnchor, '--', 'hidden.bin')
  Assert-BoundaryTest (
    $A11HiddenEntry -cmatch '^100644 blob [0-9a-f]{40}\thidden\.bin$'
  ) 'A11 State 2 hidden.bin is not one regular 100644 blob'
  $A11HiddenAnchorBytes = Get-TestCommitBlobBytes $A11 $A11CorruptionAnchor 'hidden.bin'
  Assert-BoundaryTest ($A11HiddenAnchorBytes -contains 0x00) 'A11 State 2 hidden.bin lacks a NUL byte'
  Assert-BoundaryTest (
    (Get-TestRawMarkerLineCount $A11HiddenAnchorBytes) -eq 1
  ) 'A11 State 2 hidden.bin does not contain exactly one standalone marker line'
  Assert-BoundaryTest (
    (Invoke-TestGit $A11.Path @('rev-parse', "$A11CorruptionAnchor`:$script:BoundaryCanonicalActivePath")) -ceq $A11ActiveObject
  ) 'A11 State 2 changed the legitimate active Work Order'
  $A11AdjacentChild = Add-TestStatusCommit -Repository $A11 -Status ' binary marker adjacent child' `
    -Gate "`nbaseline authorization`n" -WorkOrderPath $script:BoundaryCanonicalActivePath
  $A11HiddenChildBytes = Get-TestCommitBlobBytes $A11 $A11AdjacentChild 'hidden.bin'
  Assert-BoundaryTest (
    [Convert]::ToBase64String($A11HiddenAnchorBytes) -ceq [Convert]::ToBase64String($A11HiddenChildBytes)
  ) 'A11 State 3 did not retain hidden.bin byte-identically'
  $A11MeasuredDiff = Invoke-TestGit $A11.Path @(
    'diff-tree', '--no-commit-id', '--name-status', '-r', '--no-renames',
    $A11CorruptionAnchor, $A11AdjacentChild, '--'
  )
  Assert-BoundaryTest (
    $A11MeasuredDiff -ceq "M`t$script:BoundaryCanonicalActivePath"
  ) "A11 State 3 measured diff was $A11MeasuredDiff"
  Invoke-UnitABoundaryCase -Name 'duplicate_repository_marker_rejected' -Repository $A11 `
    -Base $A11CorruptionAnchor -Head $A11AdjacentChild `
    -PairFactory (New-ValidPairFactory $A11CorruptionAnchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A12 = New-UnitALegacyRepository $TestRoot
  Assert-ResolvedActivePath $A12 $A12.Anchor 'WORKORDER_21.md' 'A12 honest control'
  Write-TestText (Join-Path $A12.Path 'workorders/active/WORKORDER_22.md') (New-TestWorkOrderText -Inactive)
  $A12Head = Commit-TestRepository $A12 'mix legacy and canonical layouts'
  Invoke-UnitABoundaryCase -Name 'mixed_layout_ambiguity_rejected' -Repository $A12 `
    -Base $A12.Anchor -Head $A12Head -PairFactory (New-ValidPairFactory $A12.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A13 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A13 $A13.Anchor $script:BoundaryCanonicalActivePath 'A13 honest control'
  Move-TestPathExact $A13 $script:BoundaryCanonicalActivePath 'workorders/active/WORKORDER.md'
  $A13Head = Commit-TestRepository $A13 'canonical unnumbered active path'
  Invoke-UnitABoundaryCase -Name 'canonical_unnumbered_active_rejected' -Repository $A13 `
    -Base $A13.Anchor -Head $A13Head -PairFactory (New-ValidPairFactory $A13.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A14 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A14 $A14.Anchor $script:BoundaryCanonicalActivePath 'A14 honest control'
  Move-TestPathExact $A14 $script:BoundaryCanonicalActivePath 'workorders/active/WORKORDER_021.md'
  $A14Head = Commit-TestRepository $A14 'canonical leading-zero active path'
  Invoke-UnitABoundaryCase -Name 'canonical_leading_zero_rejected' -Repository $A14 `
    -Base $A14.Anchor -Head $A14Head -PairFactory (New-ValidPairFactory $A14.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A15 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A15 $A15.Anchor $script:BoundaryCanonicalActivePath 'A15 honest control'
  Move-TestPathExact $A15 'workorders/active' 'workorders/Active' -CaseOnly
  $A15Head = Commit-TestRepository $A15 'canonical active directory case'
  Invoke-UnitABoundaryCase -Name 'active_directory_case_rejected' -Repository $A15 `
    -Base $A15.Anchor -Head $A15Head -PairFactory (New-ValidPairFactory $A15.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A16 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A16 $A16.Anchor $script:BoundaryCanonicalActivePath 'A16 honest control'
  Move-TestPathExact $A16 'workorders/closed' 'workorders/Closed' -CaseOnly
  $A16Head = Commit-TestRepository $A16 'canonical closed directory case'
  Invoke-UnitABoundaryCase -Name 'closed_directory_case_rejected' -Repository $A16 `
    -Base $A16.Anchor -Head $A16Head -PairFactory (New-ValidPairFactory $A16.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A17 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A17 $A17.Anchor $script:BoundaryCanonicalActivePath 'A17 honest control'
  Move-TestPathExact $A17 $script:BoundaryCanonicalActivePath 'workorders/active/WORKORDER_21.MD' -CaseOnly
  $A17Head = Commit-TestRepository $A17 'canonical extension case'
  Invoke-UnitABoundaryCase -Name 'extension_case_rejected' -Repository $A17 `
    -Base $A17.Anchor -Head $A17Head -PairFactory (New-ValidPairFactory $A17.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A18 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A18 $A18.Anchor $script:BoundaryCanonicalActivePath 'A18 honest control'
  Move-TestPathExact $A18 $script:BoundaryCanonicalActivePath 'workorders/active/WORKORDER_21.md.bak'
  $A18Head = Commit-TestRepository $A18 'canonical suffix path'
  Invoke-UnitABoundaryCase -Name 'canonical_suffix_rejected' -Repository $A18 `
    -Base $A18.Anchor -Head $A18Head -PairFactory (New-ValidPairFactory $A18.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  Assert-TestA19MetadataGrammar
  $A19LiteralPath = 'workorders\active\WORKORDER_21.md'
  if ($env:OS -ceq 'Windows_NT') {
    $A19Protected = New-CanonicalTestRepository $TestRoot
    Assert-ResolvedActivePath $A19Protected $A19Protected.Anchor `
      $script:BoundaryCanonicalActivePath 'A19 protected-path control'
    Invoke-TestGit $A19Protected.Path @('config', '--local', 'core.protectNTFS', 'true') | Out-Null
    $A19ProtectedObject = Invoke-TestGit $A19Protected.Path @(
      'rev-parse', "$($A19Protected.Anchor):$script:BoundaryCanonicalActivePath"
    )
    Invoke-TestGit $A19Protected.Path @(
      'update-index', '--force-remove', '--', $script:BoundaryCanonicalActivePath
    ) | Out-Null
    $A19ProtectedClean = $true
    try {
      Invoke-TestGitWithoutOutput $A19Protected.Path @(
        'update-index', '--add', '--cacheinfo', "100644,$A19ProtectedObject,$A19LiteralPath"
      )
    } catch { $A19ProtectedClean = $false }
    $A19ProtectedState = Get-TestA19EntryState $A19Protected $A19ProtectedObject
    if ($A19ProtectedState.IsValid) {
      Assert-BoundaryTest $A19ProtectedClean `
        'A19 protected-path control accepted the entry with unexpected diagnostics'
    } else {
      Assert-BoundaryTest (
        $A19ProtectedState.TargetCount -eq 0 -and $A19ProtectedState.CanonicalCount -eq 0
      ) 'A19 protected-path control did not expose a deletion-only index'
    }
  }

  $A19 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A19 $A19.Anchor $script:BoundaryCanonicalActivePath 'A19 honest control'
  Invoke-TestGitWithoutOutput $A19.Path @('config', '--local', 'core.protectNTFS', 'false')
  $A19Object = Invoke-TestGit $A19.Path @('rev-parse', "$($A19.Anchor):$script:BoundaryCanonicalActivePath")
  Invoke-TestGit $A19.Path @('update-index', '--force-remove', '--', $script:BoundaryCanonicalActivePath) | Out-Null
  Invoke-TestGitWithoutOutput $A19.Path @(
    'update-index', '--add', '--cacheinfo', "100644,$A19Object,$A19LiteralPath"
  )
  $A19IndexWitness = Assert-TestA19Entry $A19 $A19Object
  Invoke-TestGit $A19.Path @('commit', '--quiet', '-m', 'literal backslash work order path') | Out-Null
  $A19Head = Invoke-TestGit $A19.Path @('rev-parse', 'HEAD')
  $A19TreeWitness = Assert-TestA19Entry $A19 $A19Object $A19Head
  Assert-BoundaryTest (
    $A19IndexWitness -and $A19TreeWitness
  ) 'A19 classifier credit requires authenticated index and tree entries'
  Invoke-UnitABoundaryCase -Name 'backslash_separator_rejected' -Repository $A19 `
    -Base $A19.Anchor -Head $A19Head -PairFactory (New-ValidPairFactory $A19.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A20 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A20 $A20.Anchor $script:BoundaryCanonicalActivePath 'A20 honest control'
  [void][System.IO.Directory]::CreateDirectory((Join-Path $A20.Path 'workorders/pending'))
  Move-TestPathExact $A20 $script:BoundaryCanonicalActivePath 'workorders/pending/WORKORDER_21.md'
  $A20Head = Commit-TestRepository $A20 'unrecognized work order directory'
  Invoke-UnitABoundaryCase -Name 'unrecognized_directory_rejected' -Repository $A20 `
    -Base $A20.Anchor -Head $A20Head -PairFactory (New-ValidPairFactory $A20.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A21 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A21 $A21.Anchor $script:BoundaryCanonicalActivePath 'A21 honest control'
  Write-TestText (Join-Path $A21.Path 'link-target.txt') "target`n"
  $A21Object = Invoke-TestGit $A21.Path @('hash-object', '-w', 'link-target.txt')
  Invoke-TestGit $A21.Path @('rm', '--cached', '--quiet', $script:BoundaryCanonicalActivePath) | Out-Null
  Invoke-TestGit $A21.Path @('update-index', '--add', '--cacheinfo', "120000,$A21Object,$script:BoundaryCanonicalActivePath") | Out-Null
  Invoke-TestGit $A21.Path @('commit', '--quiet', '-m', 'canonical active symlink replacement') | Out-Null
  $A21Head = Invoke-TestGit $A21.Path @('rev-parse', 'HEAD')
  Invoke-UnitABoundaryCase -Name 'active_symlink_rejected' -Repository $A21 `
    -Base $A21.Anchor -Head $A21Head -PairFactory (New-ValidPairFactory $A21.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A22 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A22 $A22.Anchor $script:BoundaryCanonicalActivePath 'A22 honest control'
  Invoke-TestGit $A22.Path @('rm', '--cached', '--quiet', $script:BoundaryCanonicalActivePath) | Out-Null
  Invoke-TestGit $A22.Path @('update-index', '--add', '--cacheinfo', "160000,$($A22.Anchor),$script:BoundaryCanonicalActivePath") | Out-Null
  Invoke-TestGit $A22.Path @('commit', '--quiet', '-m', 'canonical active submodule replacement') | Out-Null
  $A22Head = Invoke-TestGit $A22.Path @('rev-parse', 'HEAD')
  Invoke-UnitABoundaryCase -Name 'active_submodule_rejected' -Repository $A22 `
    -Base $A22.Anchor -Head $A22Head -PairFactory (New-ValidPairFactory $A22.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A23 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A23 $A23.Anchor $script:BoundaryCanonicalActivePath 'A23 honest control'
  Invoke-TestGit $A23.Path @('rm', '--quiet', $script:BoundaryCanonicalActivePath) | Out-Null
  $A23Head = Commit-TestRepository $A23 'delete canonical active without successor'
  Invoke-UnitABoundaryCase -Name 'active_deletion_without_successor_full' -Repository $A23 `
    -Base $A23.Anchor -Head $A23Head -PairFactory (New-ValidPairFactory $A23.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A24 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A24 $A24.Anchor $script:BoundaryCanonicalActivePath 'A24 honest control'
  Move-TestPathExact $A24 $script:BoundaryCanonicalActivePath 'workorders/active/WORKORDER_22.md'
  $A24Head = Commit-TestRepository $A24 'bare canonical active rename'
  Assert-ResolvedActivePath $A24 $A24Head 'workorders/active/WORKORDER_22.md' 'A24 renamed child control'
  Invoke-UnitABoundaryCase -Name 'active_rename_without_successor_full' -Repository $A24 `
    -Base $A24.Anchor -Head $A24Head -PairFactory (New-ValidPairFactory $A24.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A25 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A25 $A25.Anchor $script:BoundaryCanonicalActivePath 'A25 honest control'
  Write-TestText (Join-Path $A25.Path $script:BoundaryCanonicalActivePath) (
    New-TestWorkOrderText -Status ' active status changed'
  )
  $A25Closed = [System.IO.File]::ReadAllText((Join-Path $A25.Path $script:BoundaryCanonicalClosedPath))
  Write-TestText (Join-Path $A25.Path $script:BoundaryCanonicalClosedPath) (
    $A25Closed.Replace('Executable requirements stay frozen.', 'Executable requirements changed.')
  )
  $A25Head = Commit-TestRepository $A25 'status plus closed edit'
  Invoke-UnitABoundaryCase -Name 'status_plus_closed_edit_full' -Repository $A25 `
    -Base $A25.Anchor -Head $A25Head -PairFactory (New-ValidPairFactory $A25.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A26 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A26 $A26.Anchor $script:BoundaryCanonicalActivePath 'A26 honest control'
  Write-TestText (Join-Path $A26.Path $script:BoundaryCanonicalActivePath) (
    New-TestWorkOrderText -Status ' active status changed'
  )
  Move-TestPathExact $A26 $script:BoundaryCanonicalClosedPath 'workorders/closed/WORKORDER_22.md'
  $A26Head = Commit-TestRepository $A26 'status plus closed work order move'
  Invoke-UnitABoundaryCase -Name 'status_plus_workorder_move_full' -Repository $A26 `
    -Base $A26.Anchor -Head $A26Head -PairFactory (New-ValidPairFactory $A26.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A27 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A27 $A27.Anchor $script:BoundaryCanonicalActivePath 'A27 honest control'
  Write-TestText (Join-Path $A27.Path $script:BoundaryCanonicalActivePath) (New-TestWorkOrderText -Inactive)
  $A27Head = Commit-TestRepository $A27 'remove canonical active marker'
  Invoke-UnitABoundaryCase -Name 'canonical_active_marker_removed_full' -Repository $A27 `
    -Base $A27.Anchor -Head $A27Head -PairFactory (New-ValidPairFactory $A27.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $A28 = New-CanonicalTestRepository $TestRoot
  Assert-ResolvedActivePath $A28 $A28.Anchor $script:BoundaryCanonicalActivePath 'A28 honest control'
  Move-TestPathExact $A28 $script:BoundaryCanonicalActivePath 'workorders/closed/WORKORDER_21.md'
  Write-TestText (Join-Path $A28.Path 'workorders/active/WORKORDER_22.md') (
    New-TestWorkOrderText -Status ' successor with retained predecessor marker'
  )
  $A28Head = Commit-TestRepository $A28 'retain predecessor marker during successor issuance'
  Invoke-UnitABoundaryCase -Name 'successor_retained_predecessor_marker_rejected' -Repository $A28 `
    -Base $A28.Anchor -Head $A28Head -PairFactory (New-ValidPairFactory $A28.Anchor) `
    -ExpectedMode 'full' -ExpectedReason 'no_status_transition'

  $ExpectedUnitANames = @(
    'canonical_nested_header_only_fast',
    'canonical_nested_gate_only_fast',
    'canonical_nested_two_commit_chain_fast',
    'canonical_non_status_full',
    'legacy_to_canonical_migration_full',
    'canonical_successor_issuance_full',
    'canonical_adjacent_workordering_ignored',
    'closed_marker_rejected',
    'closed_copy_cannot_become_active',
    'two_active_candidates_rejected',
    'duplicate_repository_marker_rejected',
    'mixed_layout_ambiguity_rejected',
    'canonical_unnumbered_active_rejected',
    'canonical_leading_zero_rejected',
    'active_directory_case_rejected',
    'closed_directory_case_rejected',
    'extension_case_rejected',
    'canonical_suffix_rejected',
    'backslash_separator_rejected',
    'unrecognized_directory_rejected',
    'active_symlink_rejected',
    'active_submodule_rejected',
    'active_deletion_without_successor_full',
    'active_rename_without_successor_full',
    'status_plus_closed_edit_full',
    'status_plus_workorder_move_full',
    'canonical_active_marker_removed_full',
    'successor_retained_predecessor_marker_rejected'
  )
  $ActualUnitANames = @($script:UnitACaseResults | ForEach-Object { $_.Name })
  Assert-BoundaryTest (
    ($ActualUnitANames -join "`n") -ceq ($ExpectedUnitANames -join "`n")
  ) 'Unit A case names or direct invocation order changed'
  Assert-BoundaryTest (
    (Get-OrdinalUniqueCount $ActualUnitANames) -eq 28
  ) 'Unit A case names are not exactly 28 unique values'
  Assert-BoundaryTest (
    @($script:UnitACaseResults | Where-Object { $_.Mode -ceq 'fast' -and $_.Reason -ceq 'eligible_status_chain' }).Count -eq 4
  ) 'Unit A fast case count is not exactly four'
  Assert-BoundaryTest (
    @($script:UnitACaseResults | Where-Object { $_.Mode -ceq 'full' -and $_.Reason -ceq 'no_status_transition' }).Count -eq 24
  ) 'Unit A full case count is not exactly twenty-four'
  Assert-BoundaryTest (
    (Get-OrdinalUniqueCount @($script:BoundaryCaseNames)) -eq $script:ExpectedBoundaryTestCount
  ) 'final Work Order status-boundary names are not unique'
  Assert-BoundaryTest (
    (@($script:BoundaryCaseNames | Select-Object -First 123) -join "`n") -ceq ($PublishedBoundaryNames -join "`n")
  ) 'published Work Order status-boundary names were renamed or reordered'

  Assert-BoundaryTest ($script:BoundaryTestCount -gt 0) 'Work Order status-boundary selector executed zero cases'
  Assert-BoundaryTest (
    $script:BoundaryTestCount -eq $script:ExpectedBoundaryTestCount
  ) "Work Order status-boundary selector executed $script:BoundaryTestCount cases; expected $script:ExpectedBoundaryTestCount"
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
