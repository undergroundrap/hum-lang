param(
  [ValidateSet('powershell', 'pwsh')][string] $ShellContract = 'pwsh',
  [string] $ScratchRoot = '',
  [ValidateSet('', 'preflight', 'success', 'exit23', 'empty', 'interleaved', 'unicode',
    'early-marker', 'duplicate-marker', 'nonzero-marker', 'timeout', 'descendant',
    'descendant-long', 'descendant-short', 'inherited-parent', 'redirected-parent',
    'earliest-parent', 'quiescent-parent')]
  [string] $SyntheticChild = ''
)

$ErrorActionPreference = 'Stop'
$SuccessMarker = 'All Hum preflight checks passed.'
$script:HumCaptureTestPath = $MyInvocation.MyCommand.Path

function Write-ExactBytes {
  param([System.IO.Stream] $Stream, [byte[]] $Bytes)
  $Stream.Write($Bytes, 0, $Bytes.Length)
  $Stream.Flush()
}

function Write-ExactAscii {
  param([System.IO.Stream] $Stream, [string] $Text)
  Write-ExactBytes $Stream ([System.Text.Encoding]::ASCII.GetBytes($Text))
}

function Start-SyntheticDescendant {
  param([bool] $Redirect, [string] $Mode)
  $Self = $script:HumCaptureTestPath
  $Shell = (Get-Process -Id $PID).Path
  $Process = New-Object System.Diagnostics.Process
  $Process.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
  $Process.StartInfo.FileName = $Shell
  $Process.StartInfo.Arguments = "-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$Self`" -SyntheticChild $Mode"
  $Process.StartInfo.UseShellExecute = $false
  $Process.StartInfo.CreateNoWindow = $true
  if ($Redirect) {
    $Process.StartInfo.RedirectStandardOutput = $true
    $Process.StartInfo.RedirectStandardError = $true
  }
  if (-not $Process.Start()) { throw 'descendant did not launch' }
  $Process
}

if ($SyntheticChild -ne '') {
  $Stdout = [Console]::OpenStandardOutput()
  $Stderr = [Console]::OpenStandardError()
  switch ($SyntheticChild) {
    'preflight' {
      Write-ExactAscii $Stdout "$((Get-Process -Id $PID).Path)`n$($PSVersionTable.PSVersion.ToString())`n"
      exit 0
    }
    'success' {
      Write-ExactAscii $Stdout "CAPTURE_STDOUT`n$SuccessMarker`n"
      Write-ExactAscii $Stderr "CAPTURE_STDERR`n"
      exit 0
    }
    'exit23' {
      Write-ExactAscii $Stdout "EXIT23_STDOUT`n"
      Write-ExactAscii $Stderr "EXIT23_STDERR`n"
      exit 23
    }
    'empty' { exit 0 }
    'interleaved' {
      for ($Index = 0; $Index -lt 2048; $Index++) {
        $OutBytes = [Text.Encoding]::ASCII.GetBytes(("O{0:d5}:{1}`n" -f $Index, ('o' * 48)))
        $ErrBytes = [Text.Encoding]::ASCII.GetBytes(("E{0:d5}:{1}`n" -f $Index, ('e' * 48)))
        $Stdout.Write($OutBytes, 0, $OutBytes.Length)
        $Stderr.Write($ErrBytes, 0, $ErrBytes.Length)
      }
      $Stdout.Flush()
      $Stderr.Flush()
      exit 0
    }
    'unicode' {
      Write-ExactBytes $Stdout ([byte[]] (0x75, 0x74, 0x66, 0x38, 0x3d, 0xe2, 0x98, 0x83, 0x0d, 0x0a, 0x6c, 0x66, 0x0a))
      Write-ExactBytes $Stderr ([byte[]] (0x65, 0x72, 0x72, 0x3d, 0xf0, 0x9f, 0x8c, 0x8a, 0x0a, 0x63, 0x72, 0x0d))
      exit 0
    }
    'early-marker' {
      Write-ExactAscii $Stdout "$SuccessMarker`nlater assertion`n"
      exit 0
    }
    'duplicate-marker' {
      Write-ExactAscii $Stdout "$SuccessMarker`n$SuccessMarker`n"
      exit 0
    }
    'nonzero-marker' {
      Write-ExactAscii $Stdout "$SuccessMarker`n"
      exit 9
    }
    'descendant' {
      Write-ExactAscii $Stdout "descendant_alive=$PID`n"
      Write-ExactAscii $Stderr "descendant_partial_stderr`n"
      Start-Sleep -Seconds 60
      exit 0
    }
    'descendant-long' {
      Start-Sleep -Seconds 12
      exit 0
    }
    'descendant-short' {
      Start-Sleep -Milliseconds 250
      exit 0
    }
    'inherited-parent' {
      $Process = Start-SyntheticDescendant $false 'descendant-long'
      Write-ExactAscii $Stdout "inherited_parent_pid=$PID`ninherited_descendant_pid=$($Process.Id)`ninherited_parent_stdout`n"
      Write-ExactAscii $Stderr "inherited_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'redirected-parent' {
      $Process = Start-SyntheticDescendant $true 'descendant-long'
      Write-ExactAscii $Stdout "redirected_parent_pid=$PID`nredirected_descendant_pid=$($Process.Id)`nredirected_parent_stdout`n"
      Write-ExactAscii $Stderr "redirected_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'earliest-parent' {
      $Process = Start-SyntheticDescendant $false 'descendant-long'
      Write-ExactAscii $Stdout "earliest_parent_pid=$PID`nearliest_descendant_pid=$($Process.Id)`nearliest_parent_stdout`n"
      Write-ExactAscii $Stderr "earliest_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'quiescent-parent' {
      $Process = Start-SyntheticDescendant $false 'descendant-short'
      Write-ExactAscii $Stdout "quiescent_parent_pid=$PID`nquiescent_descendant_pid=$($Process.Id)`nquiescent_parent_stdout`n"
      Write-ExactAscii $Stderr "quiescent_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'timeout' {
      $Self = $MyInvocation.MyCommand.Path
      $Shell = (Get-Process -Id $PID).Path
      $Process = New-Object System.Diagnostics.Process
      $Process.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
      $Process.StartInfo.FileName = $Shell
      $Process.StartInfo.Arguments = "-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$Self`" -SyntheticChild descendant"
      $Process.StartInfo.UseShellExecute = $false
      $Process.StartInfo.CreateNoWindow = $true
      if (-not $Process.Start()) { throw 'descendant did not launch' }
      Write-ExactAscii $Stdout "parent_alive=$PID`ndescendant_pid=$($Process.Id)`nparent_partial_stdout`n"
      Write-ExactAscii $Stderr "parent_partial_stderr`n"
      $Process.WaitForExit()
      exit $Process.ExitCode
    }
  }
}

$RequestedScratchRoot = $ScratchRoot
. (Join-Path $PSScriptRoot 'run_fast_evidence.ps1')
$ScratchRoot = $RequestedScratchRoot

function Assert-True {
  param([bool] $Condition, [string] $Message)
  if (-not $Condition) { throw $Message }
}

function Read-Bytes {
  param([string] $Path)
  [System.IO.File]::ReadAllBytes($Path)
}

function Assert-Bytes {
  param([byte[]] $Actual, [byte[]] $Expected, [string] $Message)
  Assert-True ($Actual.Length -eq $Expected.Length) "$Message length"
  for ($Index = 0; $Index -lt $Actual.Length; $Index++) {
    Assert-True ($Actual[$Index] -eq $Expected[$Index]) "$Message byte $Index"
  }
}

function Assert-CaptureRejected {
  param([string] $CaptureDirectory, [string] $Message)
  $Rejected = $false
  try { $null = Read-HumCaptureRecord $CaptureDirectory } catch { $Rejected = $true }
  Assert-True $Rejected $Message
}

function Assert-LaunchedCapture {
  param([object] $Capture)
  try {
    Assert-HumCaptureComplete $Capture
  } catch {
    $Diagnostic = Get-HumCaptureFailureDiagnostic $Capture
    [Console]::Error.WriteLine($Diagnostic)
    [Console]::Error.Flush()
    throw
  }
}

function Assert-PrelaunchDiagnostic {
  param([object] $Capture, [string] $Diagnostic)
  $LaunchIdentity = Get-HumFileIdentity $Capture.LaunchErrorPath
  $CaptureIdentity = Get-HumFileIdentity $Capture.CaptureErrorPath
  $Expected = @(
    'hum_capture_failure_v1'
    "case_name=$($Capture.CaseName)"
    "capture_directory=$($Capture.CaptureDirectory)"
    "containment_kind=$($Capture.ContainmentKind)"
    "process_creation_attempted=$($Capture.ProcessCreationAttempted.ToString().ToLowerInvariant())"
    "process_creation_succeeded=$($Capture.ProcessCreationSucceeded.ToString().ToLowerInvariant())"
    "launch_succeeded=$($Capture.Launched.ToString().ToLowerInvariant())"
    "pid=$(if ($null -eq $Capture.Pid) { 'null' } else { $Capture.Pid })"
    "completion_count=$($Capture.CompletionCount)"
    "exit=$(if ($null -eq $Capture.ExitCode) { 'null' } else { $Capture.ExitCode })"
    "deadline_disposition=$($Capture.DeadlineDisposition)"
    "timed_out=$($Capture.TimedOut.ToString().ToLowerInvariant())"
    "termination_disposition=$($Capture.TerminationDisposition)"
    "termination_count=$($Capture.TerminationCount)"
    "launch_error_bytes=$($LaunchIdentity.Bytes)"
    "launch_error_sha256=$($LaunchIdentity.Sha256)"
    "launch_error_base64=$([Convert]::ToBase64String([byte[]] [System.IO.File]::ReadAllBytes($Capture.LaunchErrorPath)))"
    "capture_error_bytes=$($CaptureIdentity.Bytes)"
    "capture_error_sha256=$($CaptureIdentity.Sha256)"
    "capture_error_base64=$([Convert]::ToBase64String([byte[]] [System.IO.File]::ReadAllBytes($Capture.CaptureErrorPath)))"
  ) -join "`n"
  Assert-True ($Diagnostic -ceq $Expected) 'prelaunch diagnostic does not bind exact retained evidence'
  Assert-True (-not $Diagnostic.Contains('stdout_base64=') -and -not $Diagnostic.Contains('stderr_base64=')) 'prelaunch diagnostic exposed child streams'
}

function Test-PrelaunchDiagnostic {
  param([object] $Capture, [string] $Diagnostic)
  try { Assert-PrelaunchDiagnostic $Capture $Diagnostic; $true } catch { $false }
}

function Assert-WindowsContainmentLifecycle {
  param([object] $Capture, [string] $Message)
  Assert-True ($Capture.ContainmentKind -ceq 'windows_job') "$Message containment kind"
  Assert-True ($Capture.JobCreationAttempted -and $Capture.JobCreationSucceeded) "$Message Job creation"
  Assert-True $Capture.JobKillOnCloseConfigured "$Message kill-on-close"
  Assert-True ($Capture.ProcessCreationAttempted -and $Capture.ProcessCreationSucceeded) "$Message process creation"
  Assert-True $Capture.ProcessCreatedSuspended "$Message suspended creation"
  Assert-True ($Capture.JobAssignmentAttempted -and $Capture.JobAssignmentSucceeded) "$Message Job assignment"
  Assert-True ($Capture.ResumeAttempted -and $Capture.ResumeSucceeded) "$Message resume"
  Assert-True ($Capture.PrimaryExitObserved -and $Capture.StdoutCompletionObserved -and $Capture.StderrCompletionObserved) "$Message terminal channels"
  Assert-True ($Capture.JobQuiescenceObserved -and $Capture.FinalActiveProcessCount -eq 0) "$Message Job quiescence"
}

function Get-WitnessPid {
  param([string] $Text, [string] $Name)
  $Match = [regex]::Match($Text, [regex]::Escape($Name) + '=([0-9]+)')
  Assert-True $Match.Success "missing PID witness: $Name"
  [int] $Match.Groups[1].Value
}

function New-ContainmentProjection {
  [pscustomobject]@{
    CreateSuspended = $true
    KillOnClose = $true
    Assignment = $true
    AssignmentOrdinal = 1
    ResumeOrdinal = 2
    JobQuiescence = $true
    StreamsUseRemainingDeadline = $true
    SingleAbsoluteDeadline = $true
    TerminationCount = 1
    DescendantAbsence = $true
    PersistedFacts = $true
  }
}

function Test-ContainmentProjection {
  param([object] $Projection)
  $Projection.CreateSuspended -and $Projection.KillOnClose -and
    $Projection.Assignment -and
    $Projection.AssignmentOrdinal -lt $Projection.ResumeOrdinal -and
    $Projection.JobQuiescence -and $Projection.StreamsUseRemainingDeadline -and
    $Projection.SingleAbsoluteDeadline -and $Projection.TerminationCount -eq 1 -and
    $Projection.DescendantAbsence -and $Projection.PersistedFacts
}

function Assert-ContainmentWeakeningRejected {
  param([string] $Property, [object] $Value)
  $Projection = New-ContainmentProjection
  $Projection.$Property = $Value
  Assert-True (-not (Test-ContainmentProjection $Projection)) "containment weakening accepted: $Property"
}

function Assert-RunnerSourceContract {
  param([string] $Source)
  foreach ($Literal in @(
    'private const UInt32 CreateSuspendedFlag = 0x00000004;',
    'ProcThreadAttributeHandleList',
    'ConfigureKillOnClose($JobHandle)', 'CreateSuspended(',
    "'process_creation_succeeded.txt'", 'Assign(',
    "'job_assignment_succeeded.txt'", 'Resume(',
    "'resume_succeeded.txt'", 'ActiveProcessCount',
    'Get-HumRemainingMilliseconds', "'termination_count.txt'",
    'Read-HumCaptureRecord $Result.CaptureDirectory',
    "'case_name.txt'", 'Get-HumCaptureFailureDiagnostic $Retained',
    'launch_error_base64=', 'capture_error_base64=',
    'if ($State.TerminationCount -ne 0) { throw ''termination requested more than once'' }',
    'if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent)',
    '-not $JobQuiescent -or $FinalActive -ne 0',
    '$Remaining = Get-HumRemainingMilliseconds $Timer $DeadlineTicks'
  )) { Assert-True ($Source.Contains($Literal)) "runner source contract missing: $Literal" }
  $Create = $Source.IndexOf('$NativeProcess = [HumFastJobNative]::CreateSuspended', [StringComparison]::Ordinal)
  $PersistCreate = $Source.IndexOf("'process_creation_succeeded.txt'", $Create, [StringComparison]::Ordinal)
  $Assign = $Source.IndexOf('[HumFastJobNative]::Assign(', $PersistCreate, [StringComparison]::Ordinal)
  $PersistAssign = $Source.IndexOf("'job_assignment_succeeded.txt'", $Assign, [StringComparison]::Ordinal)
  $Resume = $Source.IndexOf('[HumFastJobNative]::Resume(', $PersistAssign, [StringComparison]::Ordinal)
  $PersistResume = $Source.IndexOf("'resume_succeeded.txt'", $Resume, [StringComparison]::Ordinal)
  Assert-True ($Create -ge 0 -and $Create -lt $PersistCreate -and $PersistCreate -lt $Assign -and
    $Assign -lt $PersistAssign -and $PersistAssign -lt $Resume -and $Resume -lt $PersistResume) 'create/assign/resume source order'
  Assert-True (-not $Source.Contains('Task]::WaitAll')) 'unbounded Task.WaitAll returned'
  Assert-True (-not $Source.Contains('.WaitForExit()')) 'unbounded WaitForExit returned'
  Assert-True (-not $Source.Contains('$StdoutTask.Wait()')) 'unbounded stream wait returned'
  Assert-True (-not $Source.Contains('$Timer.ElapsedTicks + $DeadlineTicks')) 'execution deadline can restart'
}

function Test-RunnerSourceContract {
  param([string] $Source)
  try { Assert-RunnerSourceContract $Source; $true } catch { $false }
}

function Assert-SourceWeakeningRejected {
  param([string] $Source, [string] $Old, [string] $New, [string] $Name)
  $Mutated = $Source.Replace($Old, $New)
  Assert-True ($Mutated -cne $Source) "source mutation did not initialize: $Name"
  $Tokens = $null
  $Errors = $null
  [Management.Automation.Language.Parser]::ParseInput($Mutated, [ref] $Tokens, [ref] $Errors) | Out-Null
  Assert-True ($Errors.Count -eq 0) "source mutation failed to parse: $Name"
  Assert-True (-not (Test-RunnerSourceContract $Mutated)) "source weakening accepted: $Name"
}

function Copy-CaptureForMutation {
  param([string] $Source, [string] $Destination)
  Copy-Item -LiteralPath $Source -Destination $Destination -Recurse
  $Destination
}

function Rewrite-Manifest {
  param([string] $CaptureDirectory)
  Remove-Item -LiteralPath (Join-Path $CaptureDirectory 'manifest.txt')
  Write-HumCaptureManifest $CaptureDirectory
}

function Get-GitConfigurationIdentity {
  $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
  $Paths = @(
    (Join-Path $RepoRoot '.git\config'),
    (Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)) '.gitconfig'),
    (Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)) '.config\git\config')
  )
  foreach ($Variable in @('GIT_CONFIG_GLOBAL', 'GIT_CONFIG_SYSTEM')) {
    $Value = [Environment]::GetEnvironmentVariable($Variable)
    if (-not [string]::IsNullOrEmpty($Value)) { $Paths += $Value }
  }
  if ($env:OS -eq 'Windows_NT') { $Paths += (Join-Path $env:ProgramFiles 'Git\etc\gitconfig') }
  else { $Paths += '/etc/gitconfig' }
  @($Paths | Sort-Object -Unique | ForEach-Object {
    $Resolved = [System.IO.Path]::GetFullPath($_)
    if (Test-Path -LiteralPath $Resolved -PathType Leaf) {
      "$Resolved=$((Get-HumFileIdentity $Resolved).Sha256)"
    } else {
      "$Resolved=<absent>"
    }
  }) -join "`n"
}

if ($ScratchRoot -eq '') {
  $ScratchRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("hum-fast-capture-test-" + [Guid]::NewGuid().ToString('N'))
}
Assert-True (-not (Test-Path -LiteralPath $ScratchRoot)) 'scratch root must be absent before test'
$BeforeEnvironment = @(Get-ChildItem Env: | Sort-Object Name | ForEach-Object { "$($_.Name)=$($_.Value)" }) -join "`n"
$BeforeDirectory = (Get-Location).Path
$BeforeConfig = Get-GitConfigurationIdentity
$RunnerSource = [System.IO.File]::ReadAllText((Join-Path $PSScriptRoot 'run_fast_evidence.ps1'))
Assert-RunnerSourceContract $RunnerSource
Assert-SourceWeakeningRejected $RunnerSource `
  'private const UInt32 CreateSuspendedFlag = 0x00000004;' `
  'private const UInt32 CreateSuspendedFlag = 0x00000000;' 'CREATE_SUSPENDED removal'
Assert-SourceWeakeningRejected $RunnerSource `
  '[HumFastJobNative]::Assign($JobHandle, $NativeProcess.ProcessHandle)' `
  '[HumFastJobNative]::Resume($NativeProcess.ThreadHandle)' 'resume before assignment'
Assert-SourceWeakeningRejected $RunnerSource `
  '[HumFastJobNative]::Assign($JobHandle, $NativeProcess.ProcessHandle)' `
  '$null = $JobHandle' 'Job assignment omission'
Assert-SourceWeakeningRejected $RunnerSource `
  '[HumFastJobNative]::ConfigureKillOnClose($JobHandle)' `
  '$null = $JobHandle' 'kill-on-close omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent)' `
  'if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $true)' 'Job quiescence omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'Start-Sleep -Milliseconds $Remaining' '$StdoutTask.Wait()' 'unbounded stream wait'
Assert-SourceWeakeningRejected $RunnerSource `
  '$Remaining = Get-HumRemainingMilliseconds $Timer $DeadlineTicks' `
  '$Remaining = Get-HumRemainingMilliseconds $Timer ($Timer.ElapsedTicks + $DeadlineTicks)' 'deadline restart'
Assert-SourceWeakeningRejected $RunnerSource `
  'if ($State.TerminationCount -ne 0) { throw ''termination requested more than once'' }' `
  '$null = $State.TerminationCount' 'double termination'
Assert-SourceWeakeningRejected $RunnerSource `
  '-not $JobQuiescent -or $FinalActive -ne 0' '$false' 'descendant absence omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'Read-HumCaptureRecord $Result.CaptureDirectory' '$Result' 'in-memory fact trust'
Assert-True (Test-ContainmentProjection (New-ContainmentProjection)) 'honest containment projection rejected'
Assert-ContainmentWeakeningRejected 'CreateSuspended' $false
Assert-ContainmentWeakeningRejected 'KillOnClose' $false
Assert-ContainmentWeakeningRejected 'Assignment' $false
Assert-ContainmentWeakeningRejected 'AssignmentOrdinal' 3
Assert-ContainmentWeakeningRejected 'JobQuiescence' $false
Assert-ContainmentWeakeningRejected 'StreamsUseRemainingDeadline' $false
Assert-ContainmentWeakeningRejected 'SingleAbsoluteDeadline' $false
Assert-ContainmentWeakeningRejected 'TerminationCount' 2
Assert-ContainmentWeakeningRejected 'DescendantAbsence' $false
Assert-ContainmentWeakeningRejected 'PersistedFacts' $false
$CreatedPids = New-Object System.Collections.Generic.List[int]
$ValidCaptures = New-Object System.Collections.Generic.List[object]

try {
  $Shell = if ($ShellContract -eq 'powershell') {
    "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"
  } else {
    (Get-Command pwsh -ErrorAction Stop).Source
  }
  $Self = $MyInvocation.MyCommand.Path
  $BaseArguments = @('-NoLogo', '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $Self)

  $Preflight = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'preflight')) $BeforeDirectory (Join-Path $ScratchRoot 'preflight') 30 -CaseName 'preflight'
  $Preflight = Assert-LaunchedCapture $Preflight
  Assert-True ($Preflight.ExitCode -eq 0 -and $Preflight.StderrBytes -eq 0) 'shell preflight failed'
  $PreflightLines = @([Text.Encoding]::UTF8.GetString((Read-Bytes $Preflight.StdoutPath)) -split "\r?\n" | Where-Object { $_ -ne '' })
  Assert-True ($PreflightLines.Count -eq 2) 'shell preflight output shape'
  Assert-True ([System.IO.Path]::GetFullPath($PreflightLines[0]) -eq [System.IO.Path]::GetFullPath($Shell)) 'fresh child resolved a different shell'

  $Success = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'success')) $BeforeDirectory (Join-Path $ScratchRoot 'success') 30 -CaseName 'success'
  $Success = Assert-LaunchedCapture $Success
  $Success = Read-HumCaptureRecord $Success.CaptureDirectory
  Assert-True ($Success.ExitCode -eq 0 -and $Success.CompletionCount -eq 1) 'success terminal metadata'
  Assert-Bytes (Read-Bytes $Success.StdoutPath) ([Text.Encoding]::ASCII.GetBytes("CAPTURE_STDOUT`n$SuccessMarker`n")) 'success stdout'
  Assert-Bytes (Read-Bytes $Success.StderrPath) ([Text.Encoding]::ASCII.GetBytes("CAPTURE_STDERR`n")) 'success stderr'
  Assert-True ($Success.SuccessMarkerCount -eq 1 -and $Success.TerminalStdoutLine -ceq $SuccessMarker) 'success terminal marker'

  $Exit23 = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'exit23')) $BeforeDirectory (Join-Path $ScratchRoot 'exit23') 30 -CaseName 'exit23'
  $Exit23 = Assert-LaunchedCapture $Exit23
  Assert-True ($Exit23.ExitCode -eq 23) 'exit 23 must be retained'
  Assert-Bytes (Read-Bytes $Exit23.StdoutPath) ([Text.Encoding]::ASCII.GetBytes("EXIT23_STDOUT`n")) 'exit23 stdout'
  Assert-Bytes (Read-Bytes $Exit23.StderrPath) ([Text.Encoding]::ASCII.GetBytes("EXIT23_STDERR`n")) 'exit23 stderr'

  $Empty = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'empty')) $BeforeDirectory (Join-Path $ScratchRoot 'empty') 30 -CaseName 'empty'
  $Empty = Assert-LaunchedCapture $Empty
  Assert-True ($Empty.Launched -and $Empty.ExitCode -eq 0 -and $Empty.StdoutBytes -eq 0 -and $Empty.StderrBytes -eq 0) 'launched empty-stream child'

  $Missing = Invoke-HumBinaryCapture (Join-Path $ScratchRoot 'missing-executable.exe') @('--unused') $BeforeDirectory (Join-Path $ScratchRoot 'prelaunch') 30 -CaseName 'known-missing-executable'
  $Missing = Read-HumCaptureRecord $Missing.CaptureDirectory
  Assert-True (-not $Missing.Launched -and $null -eq $Missing.ExitCode -and $Missing.CompletionCount -eq 0) 'prelaunch failure state'
  Assert-True ($Missing.StdoutBytes -eq 0 -and $Missing.StderrBytes -eq 0) 'prelaunch durable empty streams'
  Assert-True ((Get-HumFileIdentity $Missing.LaunchErrorPath).Bytes -gt 0) 'prelaunch error must be retained'
  $MissingDiagnostic = Get-HumCaptureFailureDiagnostic $Missing
  Assert-PrelaunchDiagnostic $Missing $MissingDiagnostic
  $MissingDiagnosticMutation = $MissingDiagnostic -replace '(?m)^launch_error_base64=.*$', 'launch_error_base64='
  Assert-True ($MissingDiagnosticMutation -cne $MissingDiagnostic) 'prelaunch diagnostic mutation did not initialize'
  Assert-True (-not (Test-PrelaunchDiagnostic $Missing $MissingDiagnosticMutation)) 'launch-error diagnostic omission accepted'

  $Interleaved = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'interleaved')) $BeforeDirectory (Join-Path $ScratchRoot 'interleaved') 60 -CaseName 'interleaved'
  $Interleaved = Assert-LaunchedCapture $Interleaved
  $ExpectedOut = New-Object System.Text.StringBuilder
  $ExpectedErr = New-Object System.Text.StringBuilder
  for ($Index = 0; $Index -lt 2048; $Index++) {
    $null = $ExpectedOut.Append(("O{0:d5}:{1}`n" -f $Index, ('o' * 48)))
    $null = $ExpectedErr.Append(("E{0:d5}:{1}`n" -f $Index, ('e' * 48)))
  }
  Assert-Bytes (Read-Bytes $Interleaved.StdoutPath) ([Text.Encoding]::ASCII.GetBytes($ExpectedOut.ToString())) 'interleaved stdout order'
  Assert-Bytes (Read-Bytes $Interleaved.StderrPath) ([Text.Encoding]::ASCII.GetBytes($ExpectedErr.ToString())) 'interleaved stderr order'

  $Unicode = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'unicode')) $BeforeDirectory (Join-Path $ScratchRoot 'unicode') 30 -CaseName 'unicode'
  $Unicode = Assert-LaunchedCapture $Unicode
  Assert-Bytes (Read-Bytes $Unicode.StdoutPath) ([byte[]] (0x75, 0x74, 0x66, 0x38, 0x3d, 0xe2, 0x98, 0x83, 0x0d, 0x0a, 0x6c, 0x66, 0x0a)) 'Unicode stdout'
  Assert-Bytes (Read-Bytes $Unicode.StderrPath) ([byte[]] (0x65, 0x72, 0x72, 0x3d, 0xf0, 0x9f, 0x8c, 0x8a, 0x0a, 0x63, 0x72, 0x0d)) 'Unicode stderr'

  $Early = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'early-marker')) $BeforeDirectory (Join-Path $ScratchRoot 'early-marker') 30 -CaseName 'early-marker')
  Assert-True ($Early.SuccessMarkerCount -eq 1 -and $Early.TerminalStdoutLine -ceq 'later assertion') 'early marker must not be terminal success'
  $Duplicate = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'duplicate-marker')) $BeforeDirectory (Join-Path $ScratchRoot 'duplicate-marker') 30 -CaseName 'duplicate-marker')
  Assert-True ($Duplicate.SuccessMarkerCount -eq 2) 'duplicate marker must be visible'
  $Nonzero = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'nonzero-marker')) $BeforeDirectory (Join-Path $ScratchRoot 'nonzero-marker') 30 -CaseName 'nonzero-marker')
  Assert-True ($Nonzero.ExitCode -eq 9 -and $Nonzero.SuccessMarkerCount -eq 1 -and $Nonzero.TerminalStdoutLine -ceq $SuccessMarker) 'nonzero marker case'

  $Timeout = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'timeout')) $BeforeDirectory (Join-Path $ScratchRoot 'timeout') 2 -CaseName 'timeout')
  Assert-True ($Timeout.TimedOut -and $Timeout.KillAttemptCount -eq 1 -and $Timeout.TerminationDisposition -ceq 'tree_termination_confirmed') 'timeout tree termination'
  $TimeoutOut = [Text.Encoding]::UTF8.GetString((Read-Bytes $Timeout.StdoutPath))
  $TimeoutErr = [Text.Encoding]::UTF8.GetString((Read-Bytes $Timeout.StderrPath))
  Assert-True ($TimeoutOut -match 'parent_alive=([0-9]+)' -and $TimeoutOut -match 'descendant_pid=([0-9]+)' -and $TimeoutOut.Contains('parent_partial_stdout')) 'timeout PID and stdout witnesses'
  $ParentPid = [int] ([regex]::Match($TimeoutOut, 'parent_alive=([0-9]+)').Groups[1].Value)
  $DescendantPid = [int] ([regex]::Match($TimeoutOut, 'descendant_pid=([0-9]+)').Groups[1].Value)
  Assert-True ($TimeoutErr.Contains('parent_partial_stderr')) 'timeout stderr witness'
  Assert-True ($null -eq (Get-Process -Id $ParentPid -ErrorAction SilentlyContinue)) 'timed-out parent survived'
  Assert-True ($null -eq (Get-Process -Id $DescendantPid -ErrorAction SilentlyContinue)) 'timed-out descendant survived'

  $WindowsCaptures = @()
  $SetupCaptures = @()
  if ($env:OS -eq 'Windows_NT') {
    $InheritedWall = [Diagnostics.Stopwatch]::StartNew()
    $Inherited = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'inherited-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'inherited-parent') 1 2 -CaseName 'inherited-parent')
    $InheritedWall.Stop()
    Assert-WindowsContainmentLifecycle $Inherited 'inherited-pipe'
    Assert-True ($Inherited.TimedOut -and $Inherited.TerminationCount -eq 1 -and $Inherited.FinalDescendantTree -ceq 'terminated_quiescent') 'inherited-pipe timeout disposition'
    $InheritedText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Inherited.StdoutPath))
    $InheritedDescendant = Get-WitnessPid $InheritedText 'inherited_descendant_pid'
    $InheritedExpected = [Text.Encoding]::ASCII.GetBytes("inherited_parent_pid=$($Inherited.Pid)`ninherited_descendant_pid=$InheritedDescendant`ninherited_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Inherited.StdoutPath) $InheritedExpected 'inherited-pipe stdout'
    Assert-Bytes (Read-Bytes $Inherited.StderrPath) ([Text.Encoding]::ASCII.GetBytes("inherited_parent_stderr`n")) 'inherited-pipe stderr'
    Assert-True ($InheritedWall.Elapsed.TotalSeconds -lt 6.0) 'inherited-pipe capture waited for natural descendant exit'
    Assert-True ($null -eq (Get-Process -Id $InheritedDescendant -ErrorAction SilentlyContinue)) 'inherited-pipe descendant survived'

    $RedirectedWall = [Diagnostics.Stopwatch]::StartNew()
    $Redirected = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'redirected-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'redirected-parent') 1 2 -CaseName 'redirected-parent')
    $RedirectedWall.Stop()
    Assert-WindowsContainmentLifecycle $Redirected 'redirected-descendant'
    Assert-True ($Redirected.TimedOut -and $Redirected.TerminationCount -eq 1) 'redirected descendant escaped Job deadline'
    $RedirectedText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Redirected.StdoutPath))
    $RedirectedDescendant = Get-WitnessPid $RedirectedText 'redirected_descendant_pid'
    $RedirectedExpected = [Text.Encoding]::ASCII.GetBytes("redirected_parent_pid=$($Redirected.Pid)`nredirected_descendant_pid=$RedirectedDescendant`nredirected_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Redirected.StdoutPath) $RedirectedExpected 'redirected descendant stdout'
    Assert-Bytes (Read-Bytes $Redirected.StderrPath) ([Text.Encoding]::ASCII.GetBytes("redirected_parent_stderr`n")) 'redirected descendant stderr'
    Assert-True ($RedirectedWall.Elapsed.TotalSeconds -lt 6.0) 'redirected capture waited for natural descendant exit'
    Assert-True ($null -eq (Get-Process -Id $RedirectedDescendant -ErrorAction SilentlyContinue)) 'redirected descendant survived'

    $Earliest = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'earliest-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'earliest-parent') 1 2 -CaseName 'earliest-parent')
    Assert-WindowsContainmentLifecycle $Earliest 'earliest-descendant'
    Assert-True ($Earliest.TimedOut -and $Earliest.TerminationCount -eq 1) 'earliest descendant did not remain contained'
    $EarliestText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Earliest.StdoutPath))
    $EarliestDescendant = Get-WitnessPid $EarliestText 'earliest_descendant_pid'
    $EarliestExpected = [Text.Encoding]::ASCII.GetBytes("earliest_parent_pid=$($Earliest.Pid)`nearliest_descendant_pid=$EarliestDescendant`nearliest_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Earliest.StdoutPath) $EarliestExpected 'earliest descendant stdout'
    Assert-Bytes (Read-Bytes $Earliest.StderrPath) ([Text.Encoding]::ASCII.GetBytes("earliest_parent_stderr`n")) 'earliest descendant stderr'
    Assert-True ($null -eq (Get-Process -Id $EarliestDescendant -ErrorAction SilentlyContinue)) 'earliest descendant survived'

    $Quiescent = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'quiescent-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'quiescent-parent') 5 2 -CaseName 'quiescent-parent')
    Assert-WindowsContainmentLifecycle $Quiescent 'ordinary-quiescent'
    Assert-True (-not $Quiescent.TimedOut -and $Quiescent.TerminationCount -eq 0 -and $Quiescent.DeadlineDisposition -ceq 'completed_before_deadline') 'ordinary quiescence disposition'
    $QuiescentText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Quiescent.StdoutPath))
    $QuiescentDescendant = Get-WitnessPid $QuiescentText 'quiescent_descendant_pid'
    $QuiescentExpected = [Text.Encoding]::ASCII.GetBytes("quiescent_parent_pid=$($Quiescent.Pid)`nquiescent_descendant_pid=$QuiescentDescendant`nquiescent_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Quiescent.StdoutPath) $QuiescentExpected 'ordinary quiescence stdout'
    Assert-Bytes (Read-Bytes $Quiescent.StderrPath) ([Text.Encoding]::ASCII.GetBytes("quiescent_parent_stderr`n")) 'ordinary quiescence stderr'
    Assert-True ($null -eq (Get-Process -Id $QuiescentDescendant -ErrorAction SilentlyContinue)) 'ordinary short descendant survived'

    foreach ($FailureName in @('job_create', 'job_configure', 'process_create', 'assignment', 'resume')) {
      $Failure = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'empty')) $BeforeDirectory (Join-Path $ScratchRoot "setup-$FailureName") 10 2 $FailureName -CaseName "setup-$FailureName"
      $Failure = Read-HumCaptureRecord $Failure.CaptureDirectory
      Assert-True ($Failure.CaptureErrorBytes -gt 0 -or -not $Failure.ProcessCreationSucceeded) "setup failure not retained: $FailureName"
      if ($FailureName -in @('job_create', 'job_configure', 'process_create')) {
        Assert-True (-not $Failure.Launched -and $null -eq $Failure.Pid) "prelaunch setup failure launched: $FailureName"
      } else {
        Assert-True ($Failure.Launched -and $Failure.ProcessCreatedSuspended -and $Failure.TerminationCount -eq 1) "post-create setup failure not contained: $FailureName"
        Assert-True ($null -eq (Get-Process -Id $Failure.Pid -ErrorAction SilentlyContinue)) "setup-failure child survived: $FailureName"
      }
      $SetupCaptures += $Failure
    }
    Assert-True (-not $SetupCaptures[3].JobAssignmentSucceeded -and -not $SetupCaptures[3].ResumeAttempted) 'assignment-failure lifecycle'
    Assert-True ($SetupCaptures[4].JobAssignmentSucceeded -and $SetupCaptures[4].ResumeAttempted -and -not $SetupCaptures[4].ResumeSucceeded) 'resume-failure lifecycle'
    $WindowsCaptures = @($Inherited, $Redirected, $Earliest, $Quiescent)
  }

  $MissingFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-missing')
  Remove-Item -LiteralPath (Join-Path $MissingFact 'completion_count.txt')
  Assert-CaptureRejected $MissingFact 'missing completion fact accepted'

  $MalformedFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-malformed')
  Set-HumDurableText (Join-Path $MalformedFact 'completion_count.txt') 'malformed'
  Rewrite-Manifest $MalformedFact
  Assert-CaptureRejected $MalformedFact 'malformed completion fact accepted'

  $DuplicateFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-duplicate')
  Set-HumDurableBytes (Join-Path $DuplicateFact 'completion_count.txt') ([Text.Encoding]::UTF8.GetBytes("1`n1`n"))
  Rewrite-Manifest $DuplicateFact
  Assert-CaptureRejected $DuplicateFact 'duplicate completion fact accepted'

  $InconsistentFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-inconsistent')
  Set-HumDurableText (Join-Path $InconsistentFact 'completion_count.txt') '2'
  Rewrite-Manifest $InconsistentFact
  Assert-CaptureRejected $InconsistentFact 'inconsistent completion fact accepted'

  $HashMismatch = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-hash')
  Set-HumDurableBytes (Join-Path $HashMismatch 'stdout.bin') ([Text.Encoding]::ASCII.GetBytes("changed`n"))
  Assert-CaptureRejected $HashMismatch 'hash mismatch accepted'

  $Reordered = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-reordered')
  $ManifestLines = [System.Collections.Generic.List[string]] (Get-Content -LiteralPath (Join-Path $Reordered 'manifest.txt'))
  $Swap = $ManifestLines[1]
  $ManifestLines[1] = $ManifestLines[2]
  $ManifestLines[2] = $Swap
  Set-HumDurableText (Join-Path $Reordered 'manifest.txt') ($ManifestLines -join "`n")
  Assert-CaptureRejected $Reordered 'reordered manifest accepted'

  $TerminalMismatch = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-terminal')
  Set-HumDurableBytes (Join-Path $TerminalMismatch 'terminal_stdout_line.bin') ([Text.Encoding]::UTF8.GetBytes('forged terminal'))
  Rewrite-Manifest $TerminalMismatch
  Assert-CaptureRejected $TerminalMismatch 'terminal mismatch accepted'

  $QuiescenceMismatch = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-quiescence')
  Set-HumDurableText (Join-Path $QuiescenceMismatch 'job_quiescence_observed.txt') '0'
  Rewrite-Manifest $QuiescenceMismatch
  Assert-CaptureRejected $QuiescenceMismatch 'quiescence corruption accepted'

  $ContainmentOmission = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-containment-omission')
  Remove-Item -LiteralPath (Join-Path $ContainmentOmission 'job_assignment_succeeded.txt')
  Assert-CaptureRejected $ContainmentOmission 'containment record omission accepted'

  $ContainmentInconsistent = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-containment-inconsistent')
  Set-HumDurableText (Join-Path $ContainmentInconsistent 'job_assignment_succeeded.txt') '0'
  Rewrite-Manifest $ContainmentInconsistent
  Assert-CaptureRejected $ContainmentInconsistent 'inconsistent assigned/resumed state accepted'

  $MemoryTrust = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-memory-trust')
  Assert-True $Success.ResumeSucceeded 'initialized in-memory witness missing'
  Set-HumDurableText (Join-Path $MemoryTrust 'resume_succeeded.txt') '0'
  Rewrite-Manifest $MemoryTrust
  Assert-CaptureRejected $MemoryTrust 'in-memory containment fact overrode disk'

  if ($env:OS -eq 'Windows_NT') {
    $DoubleTermination = Copy-CaptureForMutation $Inherited.CaptureDirectory (Join-Path $ScratchRoot 'mutation-double-termination')
    Set-HumDurableText (Join-Path $DoubleTermination 'termination_count.txt') '2'
    Set-HumDurableText (Join-Path $DoubleTermination 'kill_attempt_count.txt') '2'
    Rewrite-Manifest $DoubleTermination
    Assert-CaptureRejected $DoubleTermination 'double termination accepted'

    $ActiveDescendant = Copy-CaptureForMutation $Inherited.CaptureDirectory (Join-Path $ScratchRoot 'mutation-descendant-absence')
    Set-HumDurableText (Join-Path $ActiveDescendant 'final_active_process_count.txt') '1'
    Rewrite-Manifest $ActiveDescendant
    Assert-CaptureRejected $ActiveDescendant 'live descendant witness accepted'
  }

  foreach ($Capture in @($Preflight, $Success, $Exit23, $Empty, $Missing, $Interleaved, $Unicode, $Early, $Duplicate, $Nonzero, $Timeout) + $WindowsCaptures + $SetupCaptures) {
    $null = Read-HumCaptureRecord $Capture.CaptureDirectory
    $ValidCaptures.Add($Capture)
    if ($null -ne $Capture.Pid) { $CreatedPids.Add([int] $Capture.Pid) }
  }
  foreach ($CreatedPid in @($CreatedPids | Sort-Object -Unique)) {
    Assert-True ($null -eq (Get-Process -Id $CreatedPid -ErrorAction SilentlyContinue)) "test-created process survived: $CreatedPid"
  }
} finally {
  if (Test-Path -LiteralPath $ScratchRoot) { Remove-Item -LiteralPath $ScratchRoot -Recurse -Force }
}

Assert-True (-not (Test-Path -LiteralPath $ScratchRoot)) 'scratch root cleanup'
Assert-True ((Get-Location).Path -eq $BeforeDirectory) 'current directory changed'
Assert-True ((@(Get-ChildItem Env: | Sort-Object Name | ForEach-Object { "$($_.Name)=$($_.Value)" }) -join "`n") -eq $BeforeEnvironment) 'parent environment changed'
$AfterConfig = Get-GitConfigurationIdentity
Assert-True ($AfterConfig -eq $BeforeConfig) 'local/global/system Git configuration changed'
Write-Output "Fast evidence capture tests passed for $ShellContract."
