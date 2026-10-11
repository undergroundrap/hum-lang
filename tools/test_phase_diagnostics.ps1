# Focused fixture for phase diagnostic markers.
# Tests the Write-HumPhaseMarker and Invoke-HumPhase functions from check_all.ps1
# without running the full evidence suite.
#
# Covers:
# - Completed phases with real ordered timings
# - Phase records available before producer completion (start marker written first)
# - Interruption between phase start/end retains previous completion and marks current incomplete
# - Absent diagnostics directory (PhaseDiagnostics empty) is a no-op
# - Malformed phase names are rejected
# - Sink failure does not mask the phase action

$ErrorActionPreference = 'Stop'

# Dot-source the functions from check_all.ps1 without running the main body.
# We extract the function definitions by line numbers.
$CheckAllPath = Join-Path $PSScriptRoot 'check_all.ps1'
$Lines = [IO.File]::ReadAllLines($CheckAllPath)

# Find function boundaries (Write-HumPhaseMarker: line 20 to 68, Invoke-HumPhase: line 69 to 82)
# Use 1-based line numbers from grep, convert to 0-based indices
$MarkerStart = 20 - 1; $MarkerEnd = 68 - 1  # inclusive
$PhaseStart = 69 - 1; $PhaseEnd = 82 - 1    # inclusive

$MarkerLines = $Lines[$MarkerStart..$MarkerEnd] -join "`n"
$PhaseLines = $Lines[$PhaseStart..$PhaseEnd] -join "`n"

# Set up the PhaseDiagnostics variable in script scope
$PhaseDiagnostics = ''

Invoke-Expression $MarkerLines
Invoke-Expression $PhaseLines

$TestRoot = Join-Path ([IO.Path]::GetTempPath()) ('hum-phase-test-' + [Guid]::NewGuid().ToString('N'))
$null = [IO.Directory]::CreateDirectory($TestRoot)

try {
  $Passed = 0
  $Failed = 0

  function Assert-True([bool]$Condition, [string]$Message) {
    if ($Condition) { $script:Passed++; Write-Host "PASS: $Message" }
    else { $script:Failed++; Write-Host "FAIL: $Message" }
  }

  # Test 1: Completed phases with ordered timings
  Write-Host "`n=== Test 1: Completed phases ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases1'
  Invoke-HumPhase 'alpha' { Start-Sleep -Milliseconds 50 }
  Invoke-HumPhase 'beta' { Start-Sleep -Milliseconds 50 }

  $AlphaStart = Join-Path $PhaseDiagnostics 'phase-alpha-start.txt'
  $AlphaEnd = Join-Path $PhaseDiagnostics 'phase-alpha-end.txt'
  $BetaStart = Join-Path $PhaseDiagnostics 'phase-beta-start.txt'
  $BetaEnd = Join-Path $PhaseDiagnostics 'phase-beta-end.txt'

  Assert-True (Test-Path $AlphaStart) 'alpha start marker exists'
  Assert-True (Test-Path $AlphaEnd) 'alpha end marker exists'
  Assert-True (Test-Path $BetaStart) 'beta start marker exists'
  Assert-True (Test-Path $BetaEnd) 'beta end marker exists'

  # Verify ordering via ticks: alpha start < alpha end < beta start < beta end
  function Get-Ticks([string]$Path) {
    $Content = [IO.File]::ReadAllText($Path)
    if ($Content -match 'ticks=(\d+)') { return [int64]$Matches[1] }
    throw "no ticks in $Path"
  }
  $AS = Get-Ticks $AlphaStart; $AE = Get-Ticks $AlphaEnd
  $BS = Get-Ticks $BetaStart; $BE = Get-Ticks $BetaEnd
  Assert-True ($AS -lt $AE) 'alpha start before alpha end'
  Assert-True ($AE -lt $BS) 'alpha end before beta start'
  Assert-True ($BS -lt $BE) 'beta start before beta end'

  # Test 2: Interruption retains previous completion, marks current incomplete
  Write-Host "`n=== Test 2: Interruption ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases2'
  Invoke-HumPhase 'first' { Start-Sleep -Milliseconds 10 }
  try {
    Invoke-HumPhase 'second' { throw 'simulated phase failure' }
  } catch { $Caught = $_.Exception.Message }
  Assert-True ($Caught -eq 'simulated phase failure') 'exception propagated'

  $FirstEnd = Join-Path $PhaseDiagnostics 'phase-first-end.txt'
  $SecondStart = Join-Path $PhaseDiagnostics 'phase-second-start.txt'
  $SecondEnd = Join-Path $PhaseDiagnostics 'phase-second-end.txt'
  Assert-True (Test-Path $FirstEnd) 'first phase completed before interruption'
  Assert-True (Test-Path $SecondStart) 'second phase start recorded'
  Assert-True (Test-Path $SecondEnd) 'second phase end recorded even on failure'

  # Test 3: Empty PhaseDiagnostics is a no-op (no exception)
  Write-Host "`n=== Test 3: Empty diagnostics (no-op) ==="
  $PhaseDiagnostics = ''
  try {
    Invoke-HumPhase 'noop' { $script:NoopRan = $true }
    Assert-True $script:NoopRan 'phase action ran with empty diagnostics'
    Assert-True $true 'no exception with empty PhaseDiagnostics'
  } catch {
    Assert-True $false "unexpected exception: $($_.Exception.Message)"
  }

  # Test 4: Invalid phase names rejected
  Write-Host "`n=== Test 4: Invalid phase names ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases4'
  try { Write-HumPhaseMarker '../evil' 'start'; Assert-True $false 'path traversal accepted' }
  catch { Assert-True ($_.Exception.Message -like '*invalid phase name*') 'path traversal rejected' }
  try { Write-HumPhaseMarker 'ok-name_123' 'start'; Assert-True $true 'valid name accepted' }
  catch { Assert-True $false "valid name rejected: $($_.Exception.Message)" }

  # Test 5: Marker content format
  Write-Host "`n=== Test 5: Marker content ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases5'
  Write-HumPhaseMarker 'content' 'start'
  $MarkerPath = Join-Path $PhaseDiagnostics 'phase-content-start.txt'
  $Content = [IO.File]::ReadAllText($MarkerPath)
  Assert-True ($Content -match 'phase=content') 'marker has phase name'
  Assert-True ($Content -match 'event=start') 'marker has event'
  Assert-True ($Content -match 'utc=\d{4}-\d{2}-\d{2}T') 'marker has UTC timestamp'
  Assert-True ($Content -match 'ticks=\d+') 'marker has ticks'
  Assert-True ($Content -match 'frequency=\d+') 'marker has frequency'

  # Test 6: Outcome distinction (completed vs failed)
  Write-Host "`n=== Test 6: Outcome distinction ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases6'
  Invoke-HumPhase 'okphase' { $true | Out-Null }
  try { Invoke-HumPhase 'failphase' { throw 'test failure' } } catch { }
  $OkEnd = [IO.File]::ReadAllText((Join-Path $PhaseDiagnostics 'phase-okphase-end.txt'))
  $FailEnd = [IO.File]::ReadAllText((Join-Path $PhaseDiagnostics 'phase-failphase-end.txt'))
  Assert-True ($OkEnd -match 'outcome=completed') 'completed phase marked completed'
  Assert-True ($FailEnd -match 'outcome=failed') 'failed phase marked failed'

  # Test 7: Parent binding GUID in markers
  Write-Host "`n=== Test 7: Parent binding ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases7'
  $null = [IO.Directory]::CreateDirectory($PhaseDiagnostics)
  $TestGuid = 'abc123testguid456'
  [IO.File]::WriteAllText((Join-Path $PhaseDiagnostics 'run-binding.txt'), "binding_guid=$TestGuid`n", [Text.Encoding]::UTF8)
  Write-HumPhaseMarker 'bound' 'start'
  $BoundContent = [IO.File]::ReadAllText((Join-Path $PhaseDiagnostics 'phase-bound-start.txt'))
  Assert-True ($BoundContent -match "binding=$TestGuid") 'marker carries parent binding GUID'

  # Test 8: Outcome required on end (no empty outcome)
  Write-Host "`n=== Test 8: Outcome required ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases8'
  try { Write-HumPhaseMarker 'nooutcome' 'end' ''; Assert-True $false 'empty outcome accepted' }
  catch { Assert-True ($_.Exception.Message -like '*outcome must be*') 'empty outcome rejected' }

  # === Test 9: R5 actual-consumer regression fixture ===
  # Selects the ACTUAL workflow preflight code from the repository source via AST
  # (not line slices, not a model): the parent diagnostic setup try/catch, the
  # actual capture caller (Invoke-HumPreflightCapture) and the actual diagnostic
  # consumer (Save-HumPreflightDiagnostics) from the Run Hum preflight run block
  # of .github/workflows/ci.yml, plus the actual producer functions
  # (Write-HumPhaseMarker, Invoke-HumPhase) from tools/check_all.ps1.
  # Every real environment/native/FS/clock/capture accessor in the selected
  # source is replaced with ordinal owned facts/sinks BEFORE execution; the
  # selection is rejected if any real accessor remains. Selector identities are
  # pinned to the R4-reviewed bytes and the fixture fails on drift.
  # The 24 legacy assertions above are retained producer-only controls (labeled
  # as such in the receipt); the assertions below execute the actual consumer.
  Write-Host "`n=== Test 9: R5 actual-consumer regression fixture ==="
  $NewPassed = 0; $NewFailed = 0; $NewAssertions = 0
  $NewSelection = [Collections.Generic.List[object]]::new()
  $NewCases = [Collections.Generic.List[object]]::new()
  function Test-Require([bool]$Ok, [string]$Message) {
    $script:NewAssertions++
    if ($Ok) { $script:NewPassed++; Write-Host "PASS(new): $Message" }
    else { $script:NewFailed++; Write-Host "FAIL(new): $Message" }
  }
  try {
    # --- 9a: locate and extract the actual preflight run block from ci.yml ---
    $WorkflowPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../.github/workflows/ci.yml'))
    Test-Require (Test-Path -LiteralPath $WorkflowPath) "workflow source exists at $WorkflowPath"
    $WorkflowLines = [IO.File]::ReadAllLines($WorkflowPath)
    $RunBlockIndex = -1
    $RunCandidates = [Collections.Generic.List[int]]::new()
    for ($i = 0; $i -lt $WorkflowLines.Count; $i++) {
      if ($WorkflowLines[$i] -cmatch '^        run: \|[ \t]*$') { $RunCandidates.Add($i) }
    }
    Test-Require ($RunCandidates.Count -gt 0) 'found run blocks in ci.yml'
    foreach ($Candidate in $RunCandidates) {
      $ProbeLines = [Collections.Generic.List[string]]::new()
      for ($j = $Candidate + 1; $j -lt $WorkflowLines.Count; $j++) {
        $ProbeLine = $WorkflowLines[$j]
        if ($ProbeLine.Trim() -eq '') { $ProbeLines.Add(''); continue }
        if (($ProbeLine.Length - $ProbeLine.TrimStart(' ').Length) -le 8) { break }
        $ProbeLines.Add($ProbeLine)
      }
      if (($ProbeLines -join "`n").Contains('function Save-HumPreflightDiagnostics')) { $RunBlockIndex = $Candidate; break }
    }
    Test-Require ($RunBlockIndex -ge 0) 'located the Run Hum preflight run block in ci.yml'
    $PreflightLines = [Collections.Generic.List[string]]::new()
    for ($j = $RunBlockIndex + 1; $j -lt $WorkflowLines.Count; $j++) {
      $Line = $WorkflowLines[$j]
      if ($Line.Trim() -eq '') { $PreflightLines.Add(''); continue }
      $Indent = $Line.Length - $Line.TrimStart(' ').Length
      if ($Indent -le 8) { break }
      if ($Line.Length -lt 10) { Test-Require $false "run block line $j shorter than content indent"; break }
      $PreflightLines.Add($Line.Substring(10))
    }
    $PreflightScript = $PreflightLines -join "`n"
    Test-Require ($PreflightScript.Contains('function Invoke-HumPreflightCapture')) 'run block contains the actual capture caller'
    $WTokens = $null; $WErrors = $null
    $WAst = [System.Management.Automation.Language.Parser]::ParseInput($PreflightScript, [ref]$WTokens, [ref]$WErrors)
    Test-Require ($WErrors.Count -eq 0) 'preflight run block parses with zero AST errors'
    $CheckAllText = [IO.File]::ReadAllText($CheckAllPath)
    $CTokens = $null; $CErrors = $null
    $CAst = [System.Management.Automation.Language.Parser]::ParseInput($CheckAllText, [ref]$CTokens, [ref]$CErrors)
    Test-Require ($CErrors.Count -eq 0) 'check_all.ps1 parses with zero AST errors'

    # --- 9b: unique AST selection with R4-pinned selector identities ---
    function Select-ActualSource([object]$Tree, [string]$Name, [string]$SourceLabel, [string]$ExpectedSha256) {
      $Found = @($Tree.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq $Name }, $true))
      if ($Found.Count -ne 1) { Test-Require $false "selector '$Name' in $SourceLabel is not unique (found $($Found.Count))"; return $null }
      $Ext = $Found[0].Extent
      $Sha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($Ext.Text))).Replace('-','')).ToLowerInvariant()
      $script:NewSelection.Add([pscustomobject]@{ name=$Name; source=$SourceLabel; start_utf16=$Ext.StartOffset; end_utf16=$Ext.EndOffset; line=$Ext.StartLineNumber; sha256=$Sha })
      Test-Require ($Sha -ceq $ExpectedSha256) "selector '$Name' matches R4-reviewed bytes"
      return $Ext.Text
    }
    $ActualMarker = Select-ActualSource $CAst 'Write-HumPhaseMarker' 'tools/check_all.ps1' 'e601afa49bbeb0f6d43bd3164fe4100a0aa17feac2b75f008d8d639a8d093ca6'
    $ActualPhase = Select-ActualSource $CAst 'Invoke-HumPhase' 'tools/check_all.ps1' '4f11c16095dd105da5986dd0ff98a482500a37dfca4cc49097daee35c3cf0c60'
    $ActualSave = Select-ActualSource $WAst 'Save-HumPreflightDiagnostics' 'ci.yml run block' 'fd32618e07be9e1d9fc0efca3eb9ce9493c5d4f8174902b98339553e6bc2b207'
    $ActualCaller = Select-ActualSource $WAst 'Invoke-HumPreflightCapture' 'ci.yml run block' '8ba6ec9ca424a00279b645e7e0f2aaad9dc624ea148d49ca5cf6a8d2f6c97258'
    $ParentNodes = @($WAst.EndBlock.Statements | Where-Object {
      $_ -is [System.Management.Automation.Language.TryStatementAst] -and
      $_.Body.Extent.Text.Contains('run-binding.txt') -and
      $_.Body.Extent.Text.Contains('$PhaseDiagnosticsAvailable = $true')
    })
    Test-Require ($ParentNodes.Count -eq 1) "parent setup try selector unique (found $($ParentNodes.Count))"
    $ParentTryText = $null
    if ($ParentNodes.Count -eq 1) {
      Test-Require ($ParentNodes[0].CatchClauses.Count -ge 1) 'parent setup guard has a catch clause'
      Test-Require ($ParentNodes[0].CatchClauses[0].Extent.Text.Contains('setup_failed_unavailable')) 'parent setup catch reports unavailable'
      $ParentTryText = $ParentNodes[0].Extent.Text
      $ParentSha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($ParentTryText))).Replace('-','')).ToLowerInvariant()
      $script:NewSelection.Add([pscustomobject]@{ name='parent diagnostic setup TryStatementAst'; source='ci.yml run block'; start_utf16=$ParentNodes[0].Extent.StartOffset; end_utf16=$ParentNodes[0].Extent.EndOffset; line=$ParentNodes[0].Extent.StartLineNumber; sha256=$ParentSha })
      Test-Require ($ParentSha -ceq 'd8ebac332d0c95fe4984b1623b0c3789ba9f92c260a79ee2fb29be19684ead62') 'parent setup selector matches R4-reviewed bytes'
    }

    # --- 9c: static proofs on the selected (pre-substitution) source ---
    $SavePre = $ActualSave; $CallerPre = $ActualCaller
    Test-Require (([regex]::Matches($CallerPre, 'Invoke-HumBinaryCapture')).Count -eq 1) 'actual capture call present exactly once in selected caller'
    Test-Require (([regex]::Matches($CallerPre, 'Assert-HumCaptureComplete')).Count -eq 1) 'capture completion/authentication present exactly once in selected caller'
    Test-Require (([regex]::Matches($CallerPre, 'Save-HumPreflightDiagnostics')).Count -eq 2) 'caller wires consumer on both success and failure paths'
    Test-Require ($CallerPre.Contains("Save-HumPreflightDiagnostics `$CaptureDirectory `$DiagnosticDirectory `$PhaseDiagnosticDirectory 'capture_succeeded'")) 'normal-success consumer wiring present in selected caller'
    Test-Require (([regex]::Matches($SavePre, '\$script:HumCaptureFiles')).Count -eq 1) 'capture-v3 exact inventory reference unchanged in selected consumer'

    # --- 9d: replace every real accessor with ordinal owned facts/sinks ---
    $ActualMarker = $ActualMarker.Replace('Test-Path','Test-OwnedPath').Replace('[IO.Directory]::CreateDirectory($PhaseDiagnostics)','(New-OwnedDirectory $PhaseDiagnostics)').Replace("[DateTime]::UtcNow.ToString('o')","'2026-10-09T00:00:00.0000000Z'").Replace('[Diagnostics.Stopwatch]::GetTimestamp()','(Get-OwnedTicks)').Replace('[Diagnostics.Stopwatch]::Frequency','1000').Replace('[IO.File]::ReadAllText($BindingPath)','(Read-OwnedText $BindingPath)').Replace('Join-Path','Join-OwnedPath').Replace('[IO.File]::WriteAllText($TempPath, $Content, [Text.Encoding]::UTF8)','Write-OwnedText $TempPath $Content').Replace('[IO.File]::Move($TempPath, $Path, $true)','Move-OwnedText $TempPath $Path')
    $ActualSave = $ActualSave.Replace('Test-Path','Test-OwnedPath').Replace('Get-Item','Get-OwnedItem').Replace('Get-ChildItem','Get-OwnedChildren').Replace('Join-Path','Join-OwnedPath').Replace('Get-HumFileIdentity','Get-OwnedIdentity').Replace('[IO.File]::ReadAllText($PhaseFile.FullName)','(Read-OwnedText $PhaseFile.FullName)').Replace('[IO.File]::ReadAllText($BindingFile)','(Read-OwnedText $BindingFile)').Replace('[IO.File]::WriteAllText("$Dest.invalid_reason.txt", "reason=$InvalidReason`n", [Text.Encoding]::UTF8)','Write-OwnedText ("$Dest.invalid_reason.txt") ("reason=$InvalidReason`n")').Replace('Set-HumDurableText','Write-OwnedText').Replace('[IO.Directory]::CreateDirectory($DiagnosticDirectory)','(New-OwnedDirectory $DiagnosticDirectory)').Replace('[IO.File]::Copy($Source, $Destination, $false)','Copy-OwnedText $Source $Destination').Replace('[IO.File]::Copy($PhaseFile.FullName, $Dest, $false)','Copy-OwnedText $PhaseFile.FullName $Dest').Replace('[IO.File]::ReadAllBytes($Destination)','([Text.Encoding]::UTF8.GetBytes($script:OwnedFiles[$Destination]))')
    $ActualCaller = $ActualCaller.Replace('Invoke-HumBinaryCapture','Invoke-OwnedCapture').Replace('Assert-HumCaptureComplete','Assert-OwnedCaptureComplete')
    $ActualParent = '$PhaseDiagnosticsAvailable = $false' + "`n" + $ParentTryText
    $ActualParent = $ActualParent.Replace("[Guid]::NewGuid().ToString('N')","'ownedparent123'").Replace('[IO.Directory]::CreateDirectory($PhaseDiagnosticDirectory)','(New-OwnedDirectory $PhaseDiagnosticDirectory)').Replace('${env:GITHUB_RUN_ID}','$OwnedRunId').Replace("[DateTime]::UtcNow.ToString('o')","'2026-10-09T00:00:00.0000000Z'").Replace('[IO.File]::WriteAllText((Join-Path $PhaseDiagnosticDirectory ''run-binding.txt''), $BindingContent, [Text.Encoding]::UTF8)','Write-OwnedText (Join-OwnedPath $PhaseDiagnosticDirectory ''run-binding.txt'') $BindingContent')
    $RealPattern = '\$(?:env:|\{env:)|\[(?:System\.)?Environment\]|GetEnvironment|\[(?:System\.)?IO\.(?:File|Directory|Path)\]::|\[DateTime\]::UtcNow|\[Diagnostics\.Stopwatch\]::|(?<![A-Za-z-])(?:Test-Path|Get-Item|Get-ChildItem|Join-Path|Get-HumFileIdentity|Invoke-HumBinaryCapture|Assert-HumCaptureComplete|Add-Type|Invoke-Native)\b'
    $SelectedAll = $ActualMarker + "`n" + $ActualPhase + "`n" + $ActualSave + "`n" + $ActualCaller + "`n" + $ActualParent
    Test-Require (-not [regex]::IsMatch($SelectedAll, $RealPattern)) 'selected projection retains zero real accessors'
    # R6 P1: Safety gate is FATAL. A nonfatal Test-Require must never permit
    # dynamic compilation/execution. Abort before ScriptBlock.Create if any
    # real accessor remains.
    if ([regex]::IsMatch($SelectedAll, $RealPattern)) {
      throw 'FATAL: selected projection retains real accessors; refusing dynamic execution'
    }
    # R6 P2: Register functions separately from running parent setup.
    # $SelectedAll includes $ActualParent executable code; dot-sourcing it
    # would run parent setup immediately with no case path bound.
    $SelectedFunctions = $ActualMarker + "`n" + $ActualPhase + "`n" + $ActualSave + "`n" + $ActualCaller
    if ([regex]::IsMatch($SelectedFunctions, $RealPattern)) {
      throw 'FATAL: selected functions retain real accessors; refusing dynamic execution'
    }

    # --- 9e: owned facts/sinks ---
    $script:OwnedFiles = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
    $script:OwnedDirectories = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $script:OwnedHooks = [Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
    $script:OwnedMode = ''; $script:OwnedTicks = 0; $script:CaptureExit = 0; $script:CaptureThrow = $false
    $OwnedRunId = 'owned-run'
    function Register-OwnedHook([string]$Name) { if (-not $script:OwnedHooks.ContainsKey($Name)) { $script:OwnedHooks[$Name] = 0 }; $script:OwnedHooks[$Name]++ }
    function Get-OwnedHookCount([string]$Name) { if ($script:OwnedHooks.ContainsKey($Name)) { return $script:OwnedHooks[$Name] }; return 0 }
    function Test-OwnedPath { param([string]$LiteralPath) Register-OwnedHook 'exists'; return $script:OwnedDirectories.Contains($LiteralPath) -or $script:OwnedFiles.ContainsKey($LiteralPath) }
    function Join-OwnedPath { param([string]$Parent, [string]$Leaf) Register-OwnedHook 'join'; return "$Parent/$Leaf" }
    function New-OwnedDirectory([string]$Path) { Register-OwnedHook 'mkdir'; if ($script:OwnedMode -eq 'parent-create-fail') { throw 'owned_directory_failure' }; $null = $script:OwnedDirectories.Add($Path) }
    function Write-OwnedText([string]$Path, [string]$Content) { Register-OwnedHook 'write'; if ($script:OwnedMode -in @('parent-write-fail','consumer-write-fail')) { throw 'owned_write_failure' }; $script:OwnedFiles[$Path] = $Content }
    function Move-OwnedText([string]$Source, [string]$Destination) { Register-OwnedHook 'move'; $script:OwnedFiles[$Destination] = $script:OwnedFiles[$Source]; $null = $script:OwnedFiles.Remove($Source) }
    function Read-OwnedText([string]$Path) { Register-OwnedHook 'read'; return $script:OwnedFiles[$Path] }
    function Copy-OwnedText([string]$Source, [string]$Destination) { Register-OwnedHook 'copy'; if ($script:OwnedMode -eq 'consumer-copy-fail') { throw 'owned_copy_failure' }; $script:OwnedFiles[$Destination] = $script:OwnedFiles[$Source] }
    function Get-OwnedTicks { Register-OwnedHook 'clock'; $script:OwnedTicks += 100; return $script:OwnedTicks }
    function Get-OwnedChildren { param([string]$LiteralPath, [switch]$File) Register-OwnedHook 'children'; foreach ($Key in @($script:OwnedFiles.Keys)) { if ($Key.StartsWith($LiteralPath + '/', [StringComparison]::Ordinal)) { [pscustomobject]@{ Name = $Key.Substring($LiteralPath.Length + 1); FullName = $Key } } } }
    function Get-OwnedItem { param([string]$LiteralPath, [switch]$Force) Register-OwnedHook 'item'; return [pscustomobject]@{ PSIsContainer = $script:OwnedDirectories.Contains($LiteralPath); Attributes = 0 } }
    function Get-OwnedIdentity([string]$Path) { Register-OwnedHook 'identity'; $B = [Text.Encoding]::UTF8.GetBytes($script:OwnedFiles[$Path]); return [pscustomobject]@{ Bytes = $B.Length; Sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($B)) } }
    function Invoke-OwnedCapture { param($Executable, $Arguments, $WorkingDirectory, $CaptureDirectory, $DeadlineSeconds, $CaseName) Register-OwnedHook 'capture'; if ($script:CaptureThrow) { throw 'owned_capture_refusal' }; return [pscustomobject]@{ ExitCode = $script:CaptureExit; TimedOut = $false; StdoutSha256 = 'owned-stdout'; StderrSha256 = 'owned-stderr' } }
    function Assert-OwnedCaptureComplete { param($Capture) Register-OwnedHook 'capture_authentication'; return $Capture }
    # R6 P2: Register only the function definitions here. Parent setup runs
    # per-case in 9f with $PhaseDiagnosticDirectory bound to the case path.
    . ([scriptblock]::Create($SelectedFunctions))

    # --- 9f: actual parent setup cases (success / create-fail / write-fail) ---
    foreach ($CaseMode in @('', 'parent-create-fail', 'parent-write-fail')) {
      $script:OwnedMode = $CaseMode
      $SetupDir = '/owned/setup-' + $NewCases.Count
      # R6 P2: Bind the actual parent diagnostic variable to this case's
      # fresh path BEFORE running parent setup, so parent-to-caller path
      # linkage is genuinely tested.
      $PhaseDiagnosticDirectory = $SetupDir
      $CaptureHooksBefore = Get-OwnedHookCount 'capture'
      $SetupPublic = @(. ([scriptblock]::Create($ActualParent)) 6>&1 3>&1)
      $SetupAvailable = $PhaseDiagnosticsAvailable
      # R6 P2: On success, assert the parent binding was produced at the case
      # path. On failure modes the binding is absent by design (setup failed).
      if ($CaseMode -eq '') {
        Test-Require (Test-OwnedPath (Join-OwnedPath $SetupDir 'run-binding.txt')) 'parent binding produced at case path'
      }
      $script:OwnedMode = ''; $script:CaptureExit = 0
      $ReportDir = '/owned/setup-output-' + $NewCases.Count
      $CaptureOut = @(Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' $ReportDir $SetupDir 3000 6>&1 3>&1)
      $CaptureResult = $CaptureOut[-1]
      Test-Require (($CaptureResult.ExitCode -eq 0) -and ((Get-OwnedHookCount 'capture') -gt $CaptureHooksBefore)) 'validation boundary runs after parent setup'
      Test-Require ($SetupAvailable -eq ($CaseMode -eq '')) 'parent availability flag correct'
      if ($CaseMode -ne '') { Test-Require ([bool]($SetupPublic -match 'phase_diagnostics=setup_failed_unavailable')) 'parent failure public cause present' }
      else { Test-Require ([bool]($SetupPublic -match 'phase_diagnostics=available')) 'parent success public label present' }
      $NewCases.Add([pscustomobject]@{ name = 'actual parent setup ' + $(if ($CaseMode -eq '') { 'success' } else { $CaseMode }) })
    }

    # --- 9g: actual producer creates fresh facts; actual caller selects actual consumer ---
    function New-OwnedProduce([string]$Path, [string]$Kind) {
      $script:OwnedMode = ''; $script:PhaseDiagnostics = $Path
      $null = $script:OwnedDirectories.Add($Path)
      $script:OwnedFiles["$Path/run-binding.txt"] = "binding_guid=ownedparent123`n"
      if ($Kind -eq 'start-only') { Write-HumPhaseMarker 'probe' 'start'; return }
      if ($Kind -eq 'failed') { try { Invoke-HumPhase 'probe' { throw 'owned_phase_failed' } } catch { Test-Require ($_.Exception.Message -eq 'owned_phase_failed') 'producer refusal changed' }; return }
      $Out = @(Invoke-HumPhase 'probe' { 'owned_phase_output' })
      Test-Require ($Out[0] -ceq 'owned_phase_output') 'producer output changed'
    }
    $Controls = @(
      @{ Name='normal complete'; Kind='normal'; Exit=0; Mode=''; Expected='phase_timing=probe' },
      @{ Name='failed complete'; Kind='failed'; Exit=7; Mode=''; Expected='phase_timing=probe' },
      @{ Name='interrupted start-only'; Kind='start-only'; Exit=7; Mode=''; Expected='phase_incomplete=probe;last_event=start' },
      @{ Name='absent diagnostics'; Kind='absent'; Exit=0; Mode=''; Expected='phase_diagnostics=directory_missing' },
      @{ Name='foreign binding'; Kind='normal'; Exit=0; Mode=''; Expected='reason=binding_mismatch_foreign_run' },
      @{ Name='missing outcome'; Kind='normal'; Exit=0; Mode=''; Expected='reason=missing_or_invalid_outcome' },
      @{ Name='zero frequency'; Kind='normal'; Exit=0; Mode=''; Expected='reason=frequency_not_usable' },
      @{ Name='reversed ticks'; Kind='normal'; Exit=0; Mode=''; Expected='reason=ticks_reversed' },
      @{ Name='parent fact mutation'; Kind='normal'; Exit=0; Mode=''; Expected='reason=binding_mismatch_foreign_run' },
      @{ Name='consumer write failure on success'; Kind='normal'; Exit=0; Mode='consumer-write-fail'; Expected='owned_write_failure' },
      @{ Name='consumer copy failure on failure'; Kind='failed'; Exit=7; Mode='consumer-copy-fail'; Expected='owned_copy_failure' }
    )
    foreach ($Control in $Controls) {
      $InputPath = '/owned/records-' + $NewCases.Count
      $OutputPath = '/owned/report-' + $NewCases.Count
      if ($Control.Kind -ne 'absent') { New-OwnedProduce $InputPath $Control.Kind }
      switch ($Control.Name) {
        'foreign binding' { $script:OwnedFiles["$InputPath/phase-probe-end.txt"] = $script:OwnedFiles["$InputPath/phase-probe-end.txt"].Replace('binding=ownedparent123','binding=foreignrun') }
        'missing outcome' { $script:OwnedFiles["$InputPath/phase-probe-end.txt"] = $script:OwnedFiles["$InputPath/phase-probe-end.txt"].Replace("outcome=completed`n",'') }
        'zero frequency' { foreach ($Ev in @('start','end')) { $P = "$InputPath/phase-probe-$Ev.txt"; $script:OwnedFiles[$P] = $script:OwnedFiles[$P].Replace('frequency=1000','frequency=0') } }
        'reversed ticks' {
          $StartTicks = [regex]::Match($script:OwnedFiles["$InputPath/phase-probe-start.txt"], 'ticks=(\d+)').Groups[1].Value
          $script:OwnedFiles["$InputPath/phase-probe-end.txt"] = [regex]::Replace($script:OwnedFiles["$InputPath/phase-probe-end.txt"], 'ticks=\d+', ('ticks=' + ([int64]$StartTicks - 1)))
        }
        'parent fact mutation' { $script:OwnedFiles["$InputPath/run-binding.txt"] = "binding_guid=changedparent`n" }
      }
      $KeysBefore = @($script:OwnedFiles.Keys)
      $script:OwnedMode = $Control.Mode; $script:CaptureExit = $Control.Exit; $ReadsBefore = Get-OwnedHookCount 'read'
      $Public = @(Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' $OutputPath $InputPath 3000 6>&1 3>&1)
      $CaptureResult = $Public[-1]
      $Messages = if ($Public.Count -gt 1) { @($Public[0..($Public.Count - 2)] | ForEach-Object { $_.ToString() }) } else { @() }
      $Reads = (Get-OwnedHookCount 'read') - $ReadsBefore
      Test-Require ($CaptureResult.ExitCode -eq $Control.Exit) "original capture exit preserved ($($Control.Name))"
      Test-Require (($CaptureResult.StdoutSha256 -ceq 'owned-stdout') -and ($CaptureResult.StderrSha256 -ceq 'owned-stderr')) "capture streams unchanged ($($Control.Name))"
      Test-Require ([bool]($Messages -match [regex]::Escape($Control.Expected))) "public outcome present ($($Control.Name))"
      if ($Control.Name -in @('foreign binding','missing outcome','zero frequency','reversed ticks','parent fact mutation')) {
        Test-Require (-not [bool]($Messages -match '^phase_timing=')) "corruption not promoted to valid timing ($($Control.Name))"
      }
      if ($Control.Kind -in @('normal','failed') -and $Control.Mode -ne 'consumer-write-fail') { Test-Require ($Reads -ge 2) "consumer reads parent binding + records ($($Control.Name))" }
      $AddedKeys = @($script:OwnedFiles.Keys | Where-Object { $KeysBefore -notcontains $_ })
      Test-Require ((@($AddedKeys | Where-Object { -not $_.StartsWith($OutputPath + '/', [StringComparison]::Ordinal) })).Count -eq 0) "consumer writes stay inside report directory ($($Control.Name))"
      $Status = if ($script:OwnedFiles.ContainsKey("$OutputPath/diagnostic_status.txt")) { $script:OwnedFiles["$OutputPath/diagnostic_status.txt"] } else { 'unavailable' }
      if ($Control.Mode -eq '') { Test-Require ($Status.Contains('not_success_evidence')) "diagnostic status grants no success evidence ($($Control.Name))" }
      $script:OwnedMode = ''; $script:CaptureExit = 0
      $NewCases.Add([pscustomobject]@{ name = $Control.Name })
    }

    # --- 9h: original capture refusal is preserved ---
    $script:OwnedMode = ''; $script:CaptureThrow = $true
    $RefusalInput = '/owned/refusal-input'; New-OwnedProduce $RefusalInput 'normal'
    $Refusal = ''
    try { Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' '/owned/refusal-output' $RefusalInput 3000 6>$null 3>$null } catch { $Refusal = $_.Exception.Message }
    $script:CaptureThrow = $false
    Test-Require ($Refusal -ceq 'owned_capture_refusal') 'original capture refusal preserved'
    $NewCases.Add([pscustomobject]@{ name = 'original capture refusal preserved' })

    # --- 9i: negative controls — guard/wiring removal must be detected ---
    # Note: TryStatementAst.Body.Extent.Text includes the surrounding braces, so
    # strip them to obtain the bare guard-stripped statement list.
    $MutantParentBody = $ParentNodes[0].Body.Extent.Text.Trim()
    Test-Require ($MutantParentBody.StartsWith('{') -and $MutantParentBody.EndsWith('}')) 'mutant body brace shape as expected'
    $MutantParentBody = $MutantParentBody.Substring(1, $MutantParentBody.Length - 2)
    $MutantParentBody = $MutantParentBody.Replace("[Guid]::NewGuid().ToString('N')","'ownedparent123'").Replace('[IO.Directory]::CreateDirectory($PhaseDiagnosticDirectory)','(New-OwnedDirectory $PhaseDiagnosticDirectory)').Replace('${env:GITHUB_RUN_ID}','$OwnedRunId').Replace("[DateTime]::UtcNow.ToString('o')","'2026-10-09T00:00:00.0000000Z'").Replace('[IO.File]::WriteAllText((Join-Path $PhaseDiagnosticDirectory ''run-binding.txt''), $BindingContent, [Text.Encoding]::UTF8)','Write-OwnedText (Join-OwnedPath $PhaseDiagnosticDirectory ''run-binding.txt'') $BindingContent')
    $script:OwnedMode = 'parent-write-fail'
    $MutantThrew = $false
    # R6 P2: Bind the parent path for the mutant too, so the guard-stripped
    # body executes against a real case path.
    $PhaseDiagnosticDirectory = '/owned/mutant-parent'
    try { . ([scriptblock]::Create('$PhaseDiagnosticsAvailable = $false' + "`n" + $MutantParentBody)) 6>$null 3>$null } catch { $MutantThrew = $true }
    $script:OwnedMode = ''
    Test-Require $MutantThrew 'parent guard removal is detected (guard-stripped mutant throws)'
    $SuccessTryNodes = @($WAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.TryStatementAst] -and $n.Extent.Text.Contains('Save-HumPreflightDiagnostics $CaptureDirectory $DiagnosticDirectory $PhaseDiagnosticDirectory ''capture_succeeded''') }, $true))
    Test-Require ($SuccessTryNodes.Count -eq 1) 'normal-success consumer try selector unique'
    $MutantCaller = $ActualCaller.Replace($SuccessTryNodes[0].Extent.Text, '')
    Test-Require ($MutantCaller -cne $ActualCaller) 'consumer-wiring mutant constructed (success-branch try/catch removed)'
    . ([scriptblock]::Create($MutantCaller))
    $MutantInput = '/owned/mutant-input'; New-OwnedProduce $MutantInput 'normal'
    $MutantOut = @(Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' '/owned/mutant-output' $MutantInput 3000 6>$null 3>$null)
    $MutantMessages = @($MutantOut | ForEach-Object { $_.ToString() })
    Test-Require (-not [bool]($MutantMessages -match 'phase_timing=probe')) 'consumer-wiring removal is detected (no timing without consumer)'
    Test-Require (-not [bool]($MutantMessages -match 'phase_diagnostic_retained=')) 'consumer-wiring removal retains nothing'
    . ([scriptblock]::Create($ActualCaller))

    Write-Host "`nR5 selector identities (UTF-16 offsets within parsed source):"
    foreach ($S in $NewSelection) { Write-Host ("  {0} [{1}] start={2} end={3} line={4} sha256={5}" -f $S.name, $S.source, $S.start_utf16, $S.end_utf16, $S.line, $S.sha256) }
    Write-Host ("R5 distinct execution cases: {0}" -f $NewCases.Count)
  } catch {
    $script:OwnedMode = ''; $script:CaptureThrow = $false
    Test-Require $false "R5 actual-consumer section aborted: $($_.Exception.Message)"
  }

  # === Test 10: Live phase-progress observer (actual-loop, owned) ===
  # Selects ACTUAL Update-HumPhaseProgress + ACTUAL while-loop + ACTUAL caller
  # via pinned AST selectors with explicit UTF16 offsets. ALL real accessors
  # replaced with owned facts/sinks; FATAL gates before each dynamic execution.
  # Executes the real loop on BOTH platform routes with owned observation/wait.
  Write-Host "`n=== Test 10: Live phase-progress observer (actual-loop) ==="
  $ObsPassed = 0; $ObsFailed = 0; $ObsAssertions = 0; $ObsCases = [Collections.Generic.List[object]]::new()
  function Test-ObsRequire([bool]$Ok, [string]$Message) {
    $script:ObsAssertions++
    if ($Ok) { $script:ObsPassed++; Write-Host "PASS(obs): $Message" }
    else { $script:ObsFailed++; Write-Host "FAIL(obs): $Message" }
  }
  function Throw-ObsFatal([string]$Message) { throw "FATAL: $Message; refusing dynamic execution" }
  try {
    $EvidencePath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'run_fast_evidence.ps1'))
    $EvidenceText = [IO.File]::ReadAllText($EvidencePath)
    $ETokens = $null; $EErrors = $null
    $EAst = [System.Management.Automation.Language.Parser]::ParseInput($EvidenceText, [ref]$ETokens, [ref]$EErrors)
    if ($EErrors.Count -ne 0) { Throw-ObsFatal 'run_fast_evidence.ps1 has AST errors' }
    Test-ObsRequire ($EErrors.Count -eq 0) 'run_fast_evidence.ps1 parses with zero AST errors'

    # --- 10a: unique pinned selections with explicit offsets ---
    $ObsSel = @($EAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq 'Update-HumPhaseProgress' }, $true))
    if ($ObsSel.Count -ne 1) { Throw-ObsFatal "observer selector nonunique (found $($ObsSel.Count))" }
    $ActualObs = $ObsSel[0].Extent.Text
    $ObsSha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($ActualObs))).Replace('-','')).ToLowerInvariant()
    # Pinned expected hash for R3 observer (with unavailable-cause reporting)
    $ExpectedObsSha = 'd82e2a28c9e26fe4ac9c9888bf93cb7aff6ed6becac9c4958ae7a2d86d2dc034'
    if ($ObsSha -cne $ExpectedObsSha) { Throw-ObsFatal "observer hash mismatch: expected $ExpectedObsSha, got $ObsSha" }
    Write-Host ("  observer [run_fast_evidence.ps1] UTF16 [{0},{1}) line={2} sha256={3}" -f $ObsSel[0].Extent.StartOffset, $ObsSel[0].Extent.EndOffset, $ObsSel[0].Extent.StartLineNumber, $ObsSha)
    Test-ObsRequire ($ObsSha -ceq $ExpectedObsSha) 'observer matches pinned R3 identity'

    $CapSel = @($EAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq 'Invoke-HumBinaryCapture' }, $true))
    if ($CapSel.Count -ne 1) { Throw-ObsFatal 'capture selector nonunique' }
    $LoopSel = @($CapSel[0].FindAll({ param($n) $n -is [System.Management.Automation.Language.WhileStatementAst] }, $true))
    if ($LoopSel.Count -lt 1) { Throw-ObsFatal 'no while loop found in capture' }
    # Select the main loop (the one containing the observer invocation) - must be unique
    $MatchingLoops = @($LoopSel | Where-Object { $_.Extent.Text.Contains('Update-HumPhaseProgress') })
    if ($MatchingLoops.Count -ne 1) { Throw-ObsFatal "matching loop count=$($MatchingLoops.Count), expected exactly 1" }
    $MainLoop = $MatchingLoops[0]
    $ActualLoop = $MainLoop.Extent.Text
    $LoopSha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($ActualLoop))).Replace('-','')).ToLowerInvariant()
    $ExpectedLoopSha = 'fa6c26b775a6c6c93aac88915489dc06d210d00219dbb69eac60ca6b61bddc3b'
    if ($LoopSha -cne $ExpectedLoopSha) { Throw-ObsFatal "loop hash mismatch: expected $ExpectedLoopSha, got $LoopSha" }
    Write-Host ("  loop [run_fast_evidence.ps1] UTF16 [{0},{1}) line={2} sha256={3}" -f $MainLoop.Extent.StartOffset, $MainLoop.Extent.EndOffset, $MainLoop.Extent.StartLineNumber, $LoopSha)
    Test-ObsRequire ($LoopSha -ceq $ExpectedLoopSha) 'loop matches pinned R3 identity'

    # Static: caller wiring
    $CapText = $CapSel[0].Extent.Text
    $InvCount = ([regex]::Matches($CapText, 'Update-HumPhaseProgress \$PhaseDiagnosticDirectory \$PhaseObserverState')).Count
    if ($InvCount -ne 1) { Throw-ObsFatal "observer invocation count=$InvCount, expected exactly 1" }
    Test-ObsRequire ($InvCount -eq 1) 'loop observer invocation present exactly once'
    Test-ObsRequire ($CapText.Contains('observer unavailable')) 'caller-side nonfatal boundary present'

    # --- 10b: owned accessor replacements ---
    # Observer: replace FS/console accessors
    $ActualObsOwned = $ActualObs.Replace('Test-Path','Test-OwnedObsPath').Replace('Get-ChildItem','Get-OwnedObsChildren').Replace('[IO.File]::ReadAllText($BindingPath)','(Read-OwnedObsText $BindingPath)').Replace('[IO.File]::ReadAllText($File.FullName)','(Read-OwnedObsText $File.FullName)').Replace("Join-Path `$PhaseDiagnosticDirectory 'run-binding.txt'","(Join-OwnedObsPath `$PhaseDiagnosticDirectory 'run-binding.txt')").Replace('Write-Host','Write-OwnedObsHost')
    # Loop: replace observation/wait/clock/native/console accessors
    $ActualLoopOwned = $ActualLoop.Replace('Update-HumCaptureObservation','Update-OwnedCaptureObservation').Replace('Update-HumPhaseProgress','Update-HumPhaseProgress').Replace('Get-HumRemainingMilliseconds','Get-OwnedRemainingMilliseconds').Replace('[HumFastJobNative]::Wait($NativeProcess.ProcessHandle, [UInt32] $Remaining)','(Invoke-OwnedNativeWait $Remaining)').Replace('Start-Sleep','Start-OwnedSleep').Replace('Write-Host','Write-OwnedObsHost').Replace("[Environment]::GetEnvironmentVariable('VCToolsInstallDir')","'owned-vc-root'").Replace('[IO.Path]::IsPathRooted($Root)','$true').Replace('[IO.Path]::GetFullPath($Root)','$Root').Replace('Complete-HumAuthenticatedVctip','Complete-OwnedVctip')

    # Fatal zero-accessor gates
    $RealMarkers = @('[IO.File]::', '[IO.Directory]::', '[IO.Path]::', '[Environment]::', '$env:', 'HumFastJobNative]::')
    $RealCmds = @('Test-Path', 'Get-ChildItem', 'Join-Path', 'Write-Host', 'Start-Sleep', 'Update-HumCaptureObservation', 'Get-HumRemainingMilliseconds', 'Complete-HumAuthenticatedVctip')
    function Test-ZeroAccessors([string]$Text, [string]$Label) {
      $Stripped = $Text
      # Remove owned replacements before checking
      foreach ($O in @('Test-OwnedObsPath','Get-OwnedObsChildren','Join-OwnedObsPath','Write-OwnedObsHost','Read-OwnedObsText','Update-OwnedCaptureObservation','Get-OwnedRemainingMilliseconds','Invoke-OwnedNativeWait','Start-OwnedSleep','Complete-OwnedVctip')) {
        $Stripped = $Stripped.Replace($O, '')
      }
      foreach ($M in $RealMarkers) { if ($Stripped.Contains($M)) { Throw-ObsFatal "$Label retains $M" } }
      foreach ($C in $RealCmds) {
        # Word-boundary check via regex
        if ([regex]::IsMatch($Stripped, "(?<![A-Za-z-])$C(?![A-Za-z-])")) { Throw-ObsFatal "$Label retains $C" }
      }
    }
    Test-ZeroAccessors $ActualObsOwned 'observer'
    Test-ZeroAccessors $ActualLoopOwned 'loop'
    Test-ObsRequire $true 'zero real accessors in observer and loop (fatal gates passed)'

    # --- 10c: owned facts/sinks ---
    $script:ObsFiles = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
    $script:ObsDirs = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $script:ObsHooks = [Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
    $script:ObsHostOut = [Collections.Generic.List[pscustomobject]]::new()
    $script:ObsReadThrow = $false; $script:ObsPathThrow = $false
    $script:ObsThrowObserver = $false
    $script:LoopTurns = 0
    $script:LoopMaxTurns = 10
    function Register-ObsHook([string]$Name) { if (-not $script:ObsHooks.ContainsKey($Name)) { $script:ObsHooks[$Name] = 0 }; $script:ObsHooks[$Name]++ }
    function Get-ObsHookCount([string]$Name) { if ($script:ObsHooks.ContainsKey($Name)) { return $script:ObsHooks[$Name] }; return 0 }
    function Test-OwnedObsPath { param([string]$LiteralPath) if ($script:ObsPathThrow) { Register-ObsHook 'obs_path_refusal'; throw 'owned_discovery_refusal' }; Register-ObsHook 'obs_exists'; return $script:ObsDirs.Contains($LiteralPath) -or $script:ObsFiles.ContainsKey($LiteralPath) }
    function Join-OwnedObsPath { param([string]$Parent, [string]$Leaf) Register-ObsHook 'obs_join'; return "$Parent/$Leaf" }
    function Get-OwnedObsChildren { param([string]$LiteralPath, [switch]$File) Register-ObsHook 'obs_children'; foreach ($Key in @($script:ObsFiles.Keys)) { if ($Key.StartsWith($LiteralPath + '/', [StringComparison]::Ordinal)) { [pscustomobject]@{ Name = $Key.Substring($LiteralPath.Length + 1); FullName = $Key } } } }
    function Read-OwnedObsText([string]$Path) { Register-ObsHook 'obs_read'; if ($script:ObsReadThrow) { throw 'owned_obs_read_refusal' }
      # Fact corruption: return corrupted data while preserving raw bytes
      if ($script:ObsCorruptPath -ceq $Path) { Register-ObsHook 'obs_corrupt'; return $script:ObsCorruptValue }
      # Read-return delay: return empty until delay turns reached (for late-progress test)
      if (($script:ObsDelayPath -ceq $Path) -and ($script:LoopTurns -lt $script:ObsDelayUntil)) { Register-ObsHook 'obs_delayed_read'; return '' }
      if ($script:ObsDelayPath -ceq $Path) { Register-ObsHook 'obs_delayed_read' }
      return $script:ObsFiles[$Path] }
    function Write-OwnedObsHost([string]$Message) { Register-ObsHook 'obs_host'
      # Record turn and capture-active state at emission for time-sensitive assertions
      $null = $script:ObsHostOut.Add([pscustomobject]@{ Message = $Message; Turn = $script:LoopTurns; Active = -not $script:LoopState.PrimaryExited })
    }
    function Update-OwnedCaptureObservation { param($State, $Win, $NP, $MP, $JH, $ST, $ET, $CD) Register-ObsHook 'obs_observe'; $script:LoopTurns++; $Timer.ElapsedTicks += 1000
      # Script-controlled completion: set State flags when turns reached
      if ($script:LoopTurns -ge $script:LoopCompleteAfter) { $State.PrimaryExited = $true; $State.StdoutCompleted = $true; $State.StderrCompleted = $true; $State.JobQuiescent = $true }
    }
    function Get-OwnedRemainingMilliseconds { param($T, $D) Register-ObsHook 'obs_remaining'; if ($script:LoopTurns -ge $script:LoopMaxTurns) { return 0 }; return 50 }
    function Invoke-OwnedNativeWait { param($Ms) Register-ObsHook 'obs_native_wait'; }
    function Start-OwnedSleep { param([int]$Milliseconds) Register-ObsHook 'obs_managed_wait'; }
    function Complete-OwnedVctip { param($S, $NP, $JH, $CD, $R) Register-ObsHook 'obs_vctip'; }
    # Register actual observer with owned FS (fatal guard before compile)
    Test-ZeroAccessors $ActualObsOwned 'observer-precompile'
    . ([scriptblock]::Create($ActualObsOwned))

    function Reset-ObsCase {
      $script:ObsFiles.Clear(); $script:ObsDirs.Clear(); $script:ObsHostOut.Clear()
      $script:ObsReadThrow = $false; $script:ObsPathThrow = $false
      $script:ObsCorruptPath = ''; $script:ObsCorruptValue = ''; $script:ObsDelayPath = ''; $script:ObsDelayUntil = 0
      $script:ObsHooks.Clear(); $script:LoopTurns = 0; $script:LoopState = $null
    }
    function New-ObsBinding([string]$Dir, [string]$Guid) {
      $null = $script:ObsDirs.Add($Dir)
      $script:ObsFiles["$Dir/run-binding.txt"] = "binding_guid=$Guid`n"
    }
    function New-ObsMarker([string]$Dir, [string]$Name, [string]$Content) {
      $script:ObsFiles["$Dir/$Name"] = $Content
    }
    function Invoke-OwnedLoop([bool]$Windows, [string]$PhaseDir, [int]$CompleteAfter) {
      $script:LoopCompleteAfter = $CompleteAfter
      $script:LoopTurns = 0
      # Owned loop state (script scope for the loop text)
      $State = @{ PrimaryExited = $false; StdoutCompleted = $false; StderrCompleted = $false; JobQuiescent = $false; Assigned = $false; ExitCode = 0; FinalActive = 0 }
      $script:LoopState = $State
      $WindowsPlatform = $Windows
      $NativeProcess = $null; $ManagedProcess = $null; $JobHandle = $null; $StdoutTask = $null; $StderrTask = $null
      $CaptureDirectory = '/owned/capture'
      $PhaseDiagnosticDirectory = $PhaseDir
      $PhaseObserverState = @{ Binding = ''; Reported = @{} }
      $Timer = [pscustomobject]@{ ElapsedTicks = [int64]0 }
      $Frequency = [int64]1000
      $DeadlineTicks = [int64]3000000
      $DeadlineDisposition = 'pending'
      $FinalTree = ''
      $VctipGraceDeadline = $null; $TerminationDeadline = 0; $GraceTicks = 3000; $TerminationDisposition = ''
      Test-ZeroAccessors $ActualLoopOwned 'loop-precompile'
      . ([scriptblock]::Create($ActualLoopOwned))
      return [pscustomobject]@{ Disposition = $DeadlineDisposition; Turns = $script:LoopTurns; Tree = $FinalTree }
    }
    function Get-ObsMessages { ($script:ObsHostOut | ForEach-Object { $_.Message }) -join '|' }
    function Get-ObsEvent([string]$Pattern) { $script:ObsHostOut | Where-Object { $_.Message -match $Pattern } | Select-Object -First 1 }

    # --- 10d: actual loop executions ---
    # Case L1: Windows route, start before completion
    Reset-ObsCase; New-ObsBinding '/obs/l1' 'guidl1'
    New-ObsMarker '/obs/l1' 'phase-alpha-start.txt' "phase=alpha`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl1`n"
    $R1 = Invoke-OwnedLoop $true '/obs/l1' 3
    $J1 = Get-ObsMessages
    $E1 = Get-ObsEvent 'phase_progress: started alpha'
    Test-ObsRequire (($null -ne $E1) -and $E1.Active -and ($E1.Turn -lt $R1.Turns) -and ($R1.Disposition -eq 'completed_before_deadline')) 'L1: Windows route reports start while capture active (turn < completion)'
    Test-ObsRequire ((Get-ObsHookCount 'obs_native_wait') -gt 0) 'L1: Windows route uses owned native wait'
    $ObsCases.Add('loop-windows-start-before-completion')

    # Case L2: Non-Windows route, start before completion
    Reset-ObsCase; New-ObsBinding '/obs/l2' 'guidl2'
    New-ObsMarker '/obs/l2' 'phase-beta-start.txt' "phase=beta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl2`n"
    $R2 = Invoke-OwnedLoop $false '/obs/l2' 3
    $J2 = Get-ObsMessages
    $E2 = Get-ObsEvent 'phase_progress: started beta'
    Test-ObsRequire (($null -ne $E2) -and $E2.Active -and ($E2.Turn -lt $R2.Turns) -and ($R2.Disposition -eq 'completed_before_deadline')) 'L2: managed route reports start while capture active (turn < completion)'
    Test-ObsRequire ((Get-ObsHookCount 'obs_managed_wait') -gt 0) 'L2: managed route uses owned sleep wait'
    $ObsCases.Add('loop-managed-start-before-completion')

    # Case L3: chronology — start+end together, start first
    Reset-ObsCase; New-ObsBinding '/obs/l3' 'guidl3'
    New-ObsMarker '/obs/l3' 'phase-gamma-start.txt' "phase=gamma`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl3`n"
    New-ObsMarker '/obs/l3' 'phase-gamma-end.txt' "phase=gamma`nevent=end`nutc=2026-10-10T00:00:01Z`nticks=1100`nfrequency=1000`nbinding=guidl3`noutcome=completed`n"
    $R3 = Invoke-OwnedLoop $true '/obs/l3' 2
    $J3 = Get-ObsMessages
    $SI = $J3.IndexOf('started gamma'); $EI = $J3.IndexOf('gamma completed')
    Test-ObsRequire (($SI -ge 0) -and ($EI -gt $SI)) 'L3: loop emits start before end when both present'
    $ObsCases.Add('loop-chronology')

    # Case L4: corrupt read-return preserves input, public invalid, no completion
    Reset-ObsCase; New-ObsBinding '/obs/l4' 'guidl4'
    $RawInput = "phase=delta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl4`n"
    New-ObsMarker '/obs/l4' 'phase-delta-start.txt' $RawInput
    # Corrupt the authoritative READ RETURN (not the input): simulate refusal
    $script:ObsReadThrow = $true
    $R4 = Invoke-OwnedLoop $true '/obs/l4' 2
    $J4 = Get-ObsMessages
    # Input preserved (not modified by observer)
    Test-ObsRequire ($script:ObsFiles['/obs/l4/phase-delta-start.txt'] -ceq $RawInput) 'L4: corrupt read preserves raw owned input'
    Test-ObsRequire ($J4 -match 'observer unavailable \(parent_read_failed\)') 'L4: read refusal yields public unavailable cause'
    Test-ObsRequire (-not ($J4 -match 'delta (completed|failed)')) 'L4: no false completion on read failure'
    Test-ObsRequire ($R4.Disposition -eq 'completed_before_deadline') 'L4: loop completes normally despite observer refusal'
    $ObsCases.Add('loop-read-refusal-unavailable')

    # Case L5: throwing observer → caller boundary reports unavailable once, loop continues
    Reset-ObsCase; New-ObsBinding '/obs/l5' 'guidl5'
    New-ObsMarker '/obs/l5' 'phase-eps-start.txt' "phase=eps`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl5`n"
    # Temporarily redefine observer to throw (owned redefinition, restored after)
    function Update-HumPhaseProgress { param([string]$PhaseDiagnosticDirectory, [hashtable]$ObserverState) throw 'owned_observer_refusal' }
    $R5 = Invoke-OwnedLoop $true '/obs/l5' 2
    # Restore observer by dot-sourcing in CURRENT scope (not via helper - scope matters)
    Test-ZeroAccessors $ActualObsOwned 'observer-restore'
    . ([scriptblock]::Create($ActualObsOwned))
    $J5 = Get-ObsMessages
    Test-ObsRequire (($J5 -match 'phase_progress: observer unavailable') -and ($R5.Disposition -eq 'completed_before_deadline')) 'L5: throwing observer isolated, loop continues to completion'
    # Only once
    Test-ObsRequire (([regex]::Matches($J5, 'observer unavailable')).Count -eq 1) 'L5: unavailable reported exactly once'
    $ObsCases.Add('loop-throwing-observer-isolated')

    # Case L6: hookup removal kills progress, loop continues (negative control)
    Reset-ObsCase; New-ObsBinding '/obs/l6' 'guidl6'
    New-ObsMarker '/obs/l6' 'phase-zeta-start.txt' "phase=zeta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl6`n"
    $MutantLoop = $ActualLoopOwned.Replace('Update-HumPhaseProgress $PhaseDiagnosticDirectory $PhaseObserverState', '# observer invocation removed')
    Test-ZeroAccessors $MutantLoop 'mutant-loop'
    $script:LoopCompleteAfter = 2; $script:LoopTurns = 0
    $State = @{ PrimaryExited = $false; StdoutCompleted = $false; StderrCompleted = $false; JobQuiescent = $false; Assigned = $false; ExitCode = 0; FinalActive = 0 }
    $WindowsPlatform = $true
    $NativeProcess = $null; $ManagedProcess = $null; $JobHandle = $null; $StdoutTask = $null; $StderrTask = $null
    $CaptureDirectory = '/owned/capture'
    $PhaseDiagnosticDirectory = '/obs/l6'
    $PhaseObserverState = @{ Binding = ''; Reported = @{} }
    $Timer = [pscustomobject]@{ ElapsedTicks = [int64]0 }
    $Frequency = [int64]1000; $DeadlineTicks = [int64]3000000; $DeadlineDisposition = 'pending'; $FinalTree = ''
    $VctipGraceDeadline = $null; $TerminationDeadline = 0; $GraceTicks = 3000; $TerminationDisposition = ''
    Test-ZeroAccessors $MutantLoop 'mutant-precompile'
    . ([scriptblock]::Create($MutantLoop))
    $J6 = Get-ObsMessages
    Test-ObsRequire ((-not ($J6 -match 'phase_progress:')) -and ($DeadlineDisposition -eq 'completed_before_deadline')) 'L6: hookup removal kills progress, loop still completes'
    $ObsCases.Add('loop-hookup-removal-negative')

    # Case L7: absent observer → zero FS hooks, loop completes
    Reset-ObsCase
    $R7 = Invoke-OwnedLoop $true '' 2
    Test-ObsRequire (((Get-ObsHookCount 'obs_exists') -eq 0) -and ((Get-ObsHookCount 'obs_children') -eq 0) -and ($R7.Disposition -eq 'completed_before_deadline')) 'L7: absent observer does zero FS work'
    $ObsCases.Add('loop-absent-observer-quiet')

    # Case L8: deadline path — remaining returns 0
    Reset-ObsCase; New-ObsBinding '/obs/l8' 'guidl8'
    $script:LoopMaxTurns = 0  # Force immediate deadline
    $R8 = Invoke-OwnedLoop $true '/obs/l8' 999
    Test-ObsRequire ($R8.Turns -eq 1) 'L8: deadline breaks loop after one observation turn'
    $script:LoopMaxTurns = 10
    $ObsCases.Add('loop-deadline')

    # Case L9: authoritative parent-binding READ RETURN corruption
    # Corrupt the read return (not the raw bytes); assert public invalid + preserved input
    Reset-ObsCase; New-ObsBinding '/obs/l9' 'guidl9'
    $RawBinding = $script:ObsFiles['/obs/l9/run-binding.txt']
    New-ObsMarker '/obs/l9' 'phase-theta-start.txt' "phase=theta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl9`n"
    $script:ObsCorruptPath = '/obs/l9/run-binding.txt'
    $script:ObsCorruptValue = "binding_guid=CORRUPTED`n"
    $R9 = Invoke-OwnedLoop $true '/obs/l9' 2
    $J9 = Get-ObsMessages
    Test-ObsRequire ($script:ObsFiles['/obs/l9/run-binding.txt'] -ceq $RawBinding) 'L9: corruption preserves raw owned bytes'
    Test-ObsRequire ((Get-ObsHookCount 'obs_corrupt') -gt 0) 'L9: nonzero corruption hooks (read return corrupted)'
    Test-ObsRequire (($J9 -match 'theta invalid \(binding_mismatch_foreign\)') -and -not ($J9 -match 'theta (completed|failed)')) 'L9: corrupt binding yields public invalid, no false completion'
    $script:ObsCorruptPath = ''
    $ObsCases.Add('loop-binding-corruption')

    # Case L10: authoritative marker-phase READ RETURN corruption
    Reset-ObsCase; New-ObsBinding '/obs/l10' 'guidl10'
    $RawMarker = "phase=iota`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl10`n"
    New-ObsMarker '/obs/l10' 'phase-iota-start.txt' $RawMarker
    $script:ObsCorruptPath = '/obs/l10/phase-iota-start.txt'
    $script:ObsCorruptValue = "phase=WRONG`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl10`n"
    $R10 = Invoke-OwnedLoop $true '/obs/l10' 2
    $J10 = Get-ObsMessages
    Test-ObsRequire ($script:ObsFiles['/obs/l10/phase-iota-start.txt'] -ceq $RawMarker) 'L10: corruption preserves raw marker bytes'
    Test-ObsRequire ((Get-ObsHookCount 'obs_corrupt') -gt 0) 'L10: nonzero corruption hooks'
    Test-ObsRequire (($J10 -match 'iota invalid \(name_event_mismatch\)') -and -not ($J10 -match 'started iota')) 'L10: corrupt phase yields public invalid, no progress'
    $script:ObsCorruptPath = ''
    $ObsCases.Add('loop-marker-corruption')

    # Case L11: late-progress sensitivity — actual read-return delay until completion
    # The start marker's READ RETURN is delayed until the final completion turn.
    # Raw bytes are present from turn 1; only the authoritative return is delayed.
    # The emitted late event must FAIL the early-progress predicate (Active=false).
    Reset-ObsCase; New-ObsBinding '/obs/l11' 'guidl11'
    $RawKappa = "phase=kappa`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl11`n"
    New-ObsMarker '/obs/l11' 'phase-kappa-start.txt' $RawKappa
    $script:ObsDelayPath = '/obs/l11/phase-kappa-start.txt'
    $script:ObsDelayUntil = 3  # Delay until turn 3 (completion turn)
    $R11 = Invoke-OwnedLoop $true '/obs/l11' 3
    $E11 = Get-ObsEvent 'phase_progress: started kappa'
    Test-ObsRequire ($script:ObsFiles['/obs/l11/phase-kappa-start.txt'] -ceq $RawKappa) 'L11: delay preserves raw owned bytes'
    Test-ObsRequire ((Get-ObsHookCount 'obs_delayed_read') -gt 0) 'L11: nonzero delayed-reader hooks'
    Test-ObsRequire ((Get-ObsHookCount 'obs_host') -gt 0) 'L11: nonzero public-sink hooks'
    # The late event exists but must FAIL the early-progress predicate
    $EarlyPredicate = ($null -ne $E11) -and $E11.Active -and ($E11.Turn -lt $R11.Turns)
    Test-ObsRequire (-not $EarlyPredicate) 'L11: late event fails early-progress predicate (not while active)'
    # Confirm the event was late: turn == completion turn, Active=false
    if ($null -ne $E11) {
      Test-ObsRequire ((-not $E11.Active) -and ($E11.Turn -eq $R11.Turns)) 'L11: late event has Active=false at completion turn'
    } else {
      Test-ObsRequire $false 'L11: expected late event was emitted'
    }
    $script:ObsDelayPath = ''
    $ObsCases.Add('loop-late-progress-sensitive')

    # Case L12: discovery refusal (Test-Path throws) yields unavailable cause
    # Uses owned $script:ObsPathThrow flag (no provider commands)
    Reset-ObsCase; New-ObsBinding '/obs/l12' 'guidl12'
    $script:ObsPathThrow = $true
    $R12 = Invoke-OwnedLoop $true '/obs/l12' 2
    $J12 = Get-ObsMessages
    Test-ObsRequire ((Get-ObsHookCount 'obs_path_refusal') -gt 0) 'L12: discovery refusal hooks nonzero'
    Test-ObsRequire ($J12 -match 'observer unavailable \(directory_discovery_failed\)') 'L12: discovery refusal yields public unavailable cause'
    Test-ObsRequire ($R12.Disposition -eq 'completed_before_deadline') 'L12: loop completes normally despite discovery refusal'
    $script:ObsPathThrow = $false
    $ObsCases.Add('loop-discovery-refusal')


    Write-Host ("`nObs selector identities:")
    Write-Host ("  observer UTF16 [{0},{1}) sha256={2}" -f $ObsSel[0].Extent.StartOffset, $ObsSel[0].Extent.EndOffset, $ObsSha)
    Write-Host ("  loop UTF16 [{0},{1}) sha256={2}" -f $MainLoop.Extent.StartOffset, $MainLoop.Extent.EndOffset, $LoopSha)
    Write-Host ("Obs distinct execution cases: {0}" -f $ObsCases.Count)
  } catch {
    $script:ObsReadThrow = $false; $script:ObsPathThrow = $false; $script:LoopMaxTurns = 10
    Test-ObsRequire $false "observer test aborted: $($_.Exception.Message)"
  }

  Write-Host "`n=== Results (legacy producer-only marker tests, retained): $Passed passed, $Failed failed ==="
  Write-Host "=== Results (R5 actual-consumer fixture): $NewAssertions assertions across $($NewCases.Count) distinct execution cases: $NewPassed passed, $NewFailed failed ==="
  Write-Host "=== Results (Test 10 live observer): $ObsAssertions assertions: $ObsPassed passed, $ObsFailed failed ==="
  if (($Failed + $NewFailed + $ObsFailed) -gt 0) { exit 1 }
  exit 0
} finally {
  if (Test-Path $TestRoot) { Remove-Item -Recurse -Force $TestRoot }
}
