$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$GitRepoRoot = $RepoRoot.Replace([System.IO.Path]::DirectorySeparatorChar, '/')

function Resolve-Tool {
  param(
    [string] $Name,
    [string] $FallbackRelativeToHome,
    [string] $MissingMessage
  )

  $Command = Get-Command $Name -ErrorAction SilentlyContinue
  if ($null -ne $Command) {
    return $Command.Source
  }

  if ($FallbackRelativeToHome -ne '') {
    $ProfileRoot = [Environment]::GetFolderPath('UserProfile')
    $Candidate = Join-Path $ProfileRoot $FallbackRelativeToHome
    if (Test-Path -LiteralPath $Candidate) {
      return $Candidate
    }
  }

  throw $MissingMessage
}

$Cargo = Resolve-Tool 'cargo' '.cargo\bin\cargo.exe' 'cargo was not found on PATH or in the standard user Cargo install directory'
$Git = Resolve-Tool 'git' '' 'git was not found on PATH'
. (Join-Path $PSScriptRoot 'test_exact_rust_selector.ps1')

function Invoke-Native {
  param(
    [string] $Label,
    [string] $FilePath,
    [string[]] $Arguments
  )

  Write-Host "==> $Label"
  & $FilePath @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "$Label failed with exit code $LASTEXITCODE"
  }
}

function Read-NativeOutput {
  param(
    [string] $Label,
    [string] $FilePath,
    [string[]] $Arguments
  )

  Write-Host "==> $Label"
  $Output = & $FilePath @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "$Label failed with exit code $LASTEXITCODE"
  }

  return ($Output -join "`n")
}
function Read-NativeOutputWithExit {
  param(
    [string] $Label,
    [string] $FilePath,
    [string[]] $Arguments
  )

  Write-Host "==> $Label"
  $PreviousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $Output = & $FilePath @Arguments 2>&1
    $ExitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $PreviousErrorActionPreference
  }
  return [pscustomobject] @{
    Output = ($Output -join "`n")
    ExitCode = $ExitCode
  }
}

function Read-NativeChannelsWithExit {
  param(
    [string] $Label,
    [string] $FilePath,
    [string[]] $Arguments
  )

  Write-Host "==> $Label"
  if (@($Arguments | Where-Object { $_ -match '\s' }).Count -gt 0) {
    throw "$Label channel capture accepts only whitespace-free smoke-test arguments"
  }
  $StartInfo = New-Object System.Diagnostics.ProcessStartInfo
  $StartInfo.FileName = $FilePath
  $StartInfo.Arguments = ($Arguments -join ' ')
  $StartInfo.UseShellExecute = $false
  $StartInfo.CreateNoWindow = $true
  $StartInfo.RedirectStandardOutput = $true
  $StartInfo.RedirectStandardError = $true
  $Process = New-Object System.Diagnostics.Process
  $Process.StartInfo = $StartInfo
  if (-not $Process.Start()) {
    throw "$Label could not start"
  }
  $StdoutTask = $Process.StandardOutput.ReadToEndAsync()
  $StderrTask = $Process.StandardError.ReadToEndAsync()
  $Process.WaitForExit()
  return [pscustomobject] @{
    Stdout = $StdoutTask.Result
    Stderr = $StderrTask.Result
    ExitCode = $Process.ExitCode
  }
}

function Invoke-RepoScript {
  param(
    [string] $Label,
    [string] $RelativePath
  )

  Write-Host "==> $Label"
  $global:LASTEXITCODE = 0
  & (Join-Path $PSScriptRoot $RelativePath)
  if ($LASTEXITCODE -ne 0) {
    throw "$Label failed with exit code $LASTEXITCODE"
  }
}

function Assert-Json {
  param(
    [string] $Label,
    [string] $Text
  )

  Write-Host "==> parse $Label"
  $Text | ConvertFrom-Json | Out-Null
}

function Get-GraphItems {
  param([object[]] $Items)

  foreach ($Item in @($Items)) {
    if ($null -eq $Item) {
      continue
    }

    $Item
    if ($null -ne $Item.items) {
      Get-GraphItems @($Item.items)
    }
  }
}

function Assert-ReferenceEvidenceCoverage {
  param([object] $Graph)

  Write-Host '==> reference fixture evidence coverage smoke'
  $EvidenceObligations = @()
  foreach ($File in @($Graph.files)) {
    foreach ($Item in Get-GraphItems @($File.items)) {
      if ($Item.kind -eq 'task' -and $null -ne $Item.evidence_obligations) {
        $EvidenceObligations += @($Item.evidence_obligations)
      }
    }
  }

  if ($EvidenceObligations.Count -eq 0) {
    throw 'reference fixture graph JSON has no evidence obligations'
  }

  $Unlinked = @($EvidenceObligations | Where-Object {
    $_.verification_status -ne 'linked' -or $null -eq $_.linked_evidence -or @($_.linked_evidence).Count -eq 0
  })
  if ($Unlinked.Count -gt 0) {
    $Details = ($Unlinked | ForEach-Object {
      "  $($_.id) covers '$($_.covers)' with status '$($_.verification_status)'"
    }) -join "`n"
    throw "reference fixture has unlinked evidence obligations:`n$Details"
  }
}

function Assert-TextMateSnapshot {
  param([string] $Generated)

  Write-Host '==> generated TextMate grammar matches snapshot'
  if (-not $Generated.EndsWith("`n")) {
    $Generated += "`n"
  }

  $SnapshotPath = Join-Path (Join-Path (Join-Path $RepoRoot 'editors') 'textmate') 'hum.tmLanguage.json'
  $Snapshot = [System.IO.File]::ReadAllText($SnapshotPath)
  if ($Snapshot -ne $Generated) {
    throw 'editors/textmate/hum.tmLanguage.json is stale; run tools/update_textmate_grammar.ps1'
  }
}

function Assert-ReadmeHumExamplesMatch {
  Write-Host '==> README Hum examples match fixtures'
  $ReadmePath = Join-Path $RepoRoot 'README.md'
  $Readme = [System.IO.File]::ReadAllText($ReadmePath)
  $Pattern = '<!-- hum-example:start (?<path>[^\r\n]+) -->\s*```hum\r?\n(?<code>.*?)\r?\n```\s*<!-- hum-example:end -->'
  $Matches = [System.Text.RegularExpressions.Regex]::Matches($Readme, $Pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)
  if ($Matches.Count -lt 2) {
    throw 'README.md must contain at least two checked hum-example blocks'
  }

  foreach ($Match in $Matches) {
    $Relative = $Match.Groups['path'].Value.Trim()
    $FixturePath = Join-Path $RepoRoot ($Relative -replace '/', [System.IO.Path]::DirectorySeparatorChar)
    if (-not (Test-Path -LiteralPath $FixturePath)) {
      throw "README hum-example fixture is missing: $Relative"
    }
    $Fixture = [System.IO.File]::ReadAllText($FixturePath).Replace(([string][char]13 + [string][char]10), [string][char]10)
    $Code = $Match.Groups['code'].Value.Replace(([string][char]13 + [string][char]10), [string][char]10)
    if (-not $Fixture.Contains($Code)) {
      throw "README hum-example block does not match fixture text: $Relative"
    }
  }
}

function Assert-SessionASurfaceRules {
  Write-Host '==> Session A source-surface rules'
  $HumRoots = @((Join-Path $RepoRoot 'examples'), (Join-Path $RepoRoot 'fixtures'))
  foreach ($Root in $HumRoots) {
    foreach ($File in Get-ChildItem -LiteralPath $Root -Recurse -Filter '*.hum') {
      $Text = [System.IO.File]::ReadAllText($File.FullName)
      if ([regex]::IsMatch($Text, '(?m)^\s*task [a-z]+ [a-z]+')) {
        throw "spaced task name remains in $($File.FullName)"
      }
      if ([regex]::IsMatch($Text, '(?m)^\s*(store [a-z]+ [a-z]+|app [A-Z][A-Za-z]*|[^\r\n]*\((?!borrow |change |consume )[a-z]+ [a-z]+:)')) {
        throw "spaced or noncanonical declaration name remains in $($File.FullName)"
      }
      if ($Text.Contains('Number')) {
        throw "Number type usage remains in $($File.FullName)"
      }
    }
  }

  foreach ($Relative in @('README.md', 'SPEC.md')) {
    $Path = Join-Path $RepoRoot $Relative
    $Text = [System.IO.File]::ReadAllText($Path)
    if ($Text.Contains('Number')) {
      throw "Number type usage remains in $Relative"
    }
  }
}

Push-Location $RepoRoot
try {
  Invoke-RepoScript 'Work Order status-boundary classifier tests' 'test_workorder_status_boundary.ps1'
  $CheckAllSource = [System.IO.File]::ReadAllText((Join-Path $PSScriptRoot 'check_all.ps1'))
  $ExactFlag = '--' + 'exact'
  if ($CheckAllSource.Contains($ExactFlag)) { throw 'exact Rust tests must use the guarded selector helper' }
  Invoke-ExactRustSelectorSelfTests $Cargo
  Reset-ExactRustSelectorCredits
  Invoke-Native 'cargo fmt --check' $Cargo @('fmt', '--check')
  Invoke-Native 'cargo test' $Cargo @('test')
  Invoke-ExactRustTest 'canonical diagnostic registry/projection test' $Cargo 'diagnostic_catalog::tests::canonical_registry_and_checked_projections_are_valid'
  Invoke-Native 'Windows drive locality adapter tests' $Cargo @('test', '-p', 'windows-drive-locality')
  Invoke-Native 'effect bake-off corpus harness tests' $Cargo @('test', '--manifest-path', 'experiments/effect-bakeoff/Cargo.toml', '--target-dir', 'target/effect-bakeoff')
  Invoke-Native 'cargo clippy' $Cargo @('clippy', '--all-targets', '--', '-D', 'warnings')
  Invoke-Native 'Windows drive locality adapter clippy' $Cargo @('clippy', '-p', 'windows-drive-locality', '--all-targets', '--', '-D', 'warnings')
  Invoke-Native 'cargo build' $Cargo @('build')

  $MainSource = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'src/main.rs'))
  if (-not $MainSource.StartsWith('#![forbid(unsafe_code)]')) { throw 'Session AC main crate must retain forbid(unsafe_code)' }
  $LocalitySource = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'crates/windows-drive-locality/src/lib.rs'))
  if (-not $LocalitySource.Contains('#![deny(unsafe_op_in_unsafe_fn)]')) { throw 'Session AC locality adapter must deny unsafe_op_in_unsafe_fn' }
  foreach ($Symbol in @('GetDriveTypeW', 'QueryDosDeviceW', 'CreateFileW', 'DeviceIoControl', 'GetStorageDependencyInformation', 'CloseHandle')) {
    if ([regex]::Matches($LocalitySource, "\b$Symbol\s*\(").Count -ne 2) { throw "Session AC locality adapter foreign-symbol allowlist drifted for $Symbol" }
  }
  $Ioctls = @([regex]::Matches($LocalitySource, '\bIOCTL_[A-Z0-9_]+\b') | ForEach-Object { $_.Value } | Sort-Object -Unique)
  if ($Ioctls.Count -ne 2 -or $Ioctls[0] -ne 'IOCTL_STORAGE_QUERY_PROPERTY' -or $Ioctls[1] -ne 'IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS') { throw 'Session AC locality adapter IOCTL allowlist drifted' }
  if ([regex]::Matches($LocalitySource, '\bunsafe\s*\{').Count -ne 6 -or [regex]::Matches($LocalitySource, 'unsafe\s+extern').Count -ne 2) { throw 'Session AC locality adapter unsafe-block inventory drifted' }
  foreach ($Forbidden in @('std::fs', 'File::open', 'OpenOptions', 'canonicalize(', 'metadata(', 'read_to_', 'std::process', 'std::env', 'Command::', 'RegOpenKey', 'WMI', 'CoCreateInstance', 'SetupDi', 'LoadLibrary', 'GetProcAddress', 'WinHttp', 'WinSock', 'VendorIdOffset', 'ProductIdOffset')) {
    if ($LocalitySource.Contains($Forbidden)) { throw "Session AC locality adapter contains forbidden host surface: $Forbidden" }
  }

  $HumName = if ($env:OS -eq 'Windows_NT') { 'hum.exe' } else { 'hum' }
  $Hum = Join-Path (Join-Path (Join-Path $RepoRoot 'target') 'debug') $HumName

  Write-Host '==> Increment 10A canonical syntax and string-aware scope matrix'
  foreach ($EvidenceTest in @(
    'parser::tests::string_braces_and_escaped_quotes_do_not_close_items',
    'parser::tests::quote_escape_and_brace_direction_sabotage_changes_scope_facts',
    'parser::tests::retained_block_relationship_corruption_fails_closed',
    'parser::tests::genuine_unclosed_item_still_owns_h0004',
    'parser::tests::canonical_expression_tree_is_left_associative_and_precedence_aware',
    'parser::tests::canonical_expression_corruption_fails_closed',
    'parser::tests::non_ascii_unsupported_expression_is_utf8_safe',
    'parser::tests::signed_int_is_one_structural_literal_node',
    'core_body::tests::retained_parser_facts_survive_section_text_sabotage',
    'core_body::tests::retained_parser_fact_mutation_is_observable',
    'core_body::tests::parser_owned_core_kinds_preserve_established_preview_pairs'
  )) {
    Invoke-ExactRustTest "Increment 10A evidence $EvidenceTest" $Cargo $EvidenceTest
  }
  Invoke-ExactRustTest 'Increment 10B.1a.1.1 source/owner matrix: 7 fields, 21 pairs, named sabotage' $Cargo 'parser::tests::source_owner_authority_kernel_is_complete_and_load_bearing'
  Invoke-ExactRustTest 'Replacement F1 occurrence/common matrix: 36 fields, 216 singles, 630 pairs, 16 cross-occurrence cases, named sabotage' $Cargo 'parser::tests::occurrence_authority_and_common_node_topology_are_complete_and_load_bearing'
  $FoundationNonAscii = Join-Path ([System.IO.Path]::GetTempPath()) "hum-10a-nonascii-$([guid]::NewGuid().ToString('N')).hum"
  try {
    $FoundationNonAsciiSource = "module temp.nonascii`n`ntask non_ascii() -> Text {`n  does:`n    return caf$([char]0x00E9)`n}`n"
    [System.IO.File]::WriteAllText($FoundationNonAscii, $FoundationNonAsciiSource, (New-Object System.Text.UTF8Encoding($false)))
    foreach ($Format in @('human', 'json')) {
      $Args = @('check')
      if ($Format -eq 'json') { $Args += '--format=json' }
      $Args += $FoundationNonAscii
      $Result = Read-NativeChannelsWithExit "Increment 10A UTF-8 unsupported expression $Format" $Hum $Args
      $Combined = $Result.Stdout + $Result.Stderr
      if ($Result.ExitCode -ne 1) { throw "Increment 10A non-ASCII unsupported expression $Format exited $($Result.ExitCode), expected 1" }
      if ([regex]::Matches($Combined, 'H0009').Count -ne 1) { throw "Increment 10A non-ASCII unsupported expression $Format did not produce exactly one H0009: $Combined" }
      if ($Combined.Contains('panicked') -or $Combined.Contains('runtime trap')) { throw "Increment 10A non-ASCII unsupported expression reached a panic or runtime trap in $Format" }
      if ($Format -eq 'json') { Assert-Json 'Increment 10A UTF-8 unsupported expression JSON' $Result.Stdout }
    }
    $NonAsciiRun = Read-NativeChannelsWithExit 'Increment 10A UTF-8 unsupported expression runtime preflight' $Hum @('run', $FoundationNonAscii, '--entry', 'non_ascii')
    $NonAsciiCombined = $NonAsciiRun.Stdout + $NonAsciiRun.Stderr
    if ($NonAsciiRun.ExitCode -ne 1 -or [regex]::Matches($NonAsciiCombined, 'H0009').Count -ne 1 -or $NonAsciiRun.Stdout -ne '' -or $NonAsciiCombined.Contains('panicked') -or $NonAsciiCombined.Contains('runtime trap')) { throw 'Increment 10A non-ASCII unsupported expression reached runtime instead of source preflight' }
  } finally {
    Remove-Item -LiteralPath $FoundationNonAscii -ErrorAction SilentlyContinue
  }
  $FoundationTextBraces = 'fixtures/foundation/pre_ar_text_braces_pass.hum'
  $FoundationStages = @(
    @{ Name = 'check'; Args = @('check') },
    @{ Name = 'resolve'; Args = @('resolve') },
    @{ Name = 'type-env'; Args = @('type-env') },
    @{ Name = 'type-check'; Args = @('type-check') },
    @{ Name = 'full-type-check'; Args = @('full-type-check') },
    @{ Name = 'effect-check'; Args = @('effect-check') },
    @{ Name = 'ownership-check'; Args = @('ownership-check') },
    @{ Name = 'resource-check'; Args = @('resource-check') },
    @{ Name = 'profile-check'; Args = @('profile-check') },
    @{ Name = 'core-preview'; Args = @('core-preview') },
    @{ Name = 'core-lower'; Args = @('core-lower') },
    @{ Name = 'core-verify'; Args = @('core-verify') },
    @{ Name = 'ir-readiness'; Args = @('ir-readiness') }
  )
  foreach ($Stage in $FoundationStages) {
    foreach ($Format in @('human', 'json')) {
      $Args = @($Stage.Args)
      if ($Format -eq 'json') { $Args += '--format=json' }
      $Args += $FoundationTextBraces
      $Result = Read-NativeChannelsWithExit "Increment 10A $($Stage.Name) $Format" $Hum $Args
      if ($Result.ExitCode -notin @(0, 1)) { throw "Increment 10A $($Stage.Name) $Format exited unexpectedly" }
      $Combined = $Result.Stdout + $Result.Stderr
      if ($Combined.Contains('H0001') -or $Combined.Contains('H0003') -or $Combined.Contains('H0004')) { throw "Increment 10A Text braces changed parser diagnostic ownership in $($Stage.Name) $Format" }
      if ($Format -eq 'json') { Assert-Json "Increment 10A $($Stage.Name) JSON" $Result.Stdout }
    }
  }
  $FoundationGraph = Read-NativeChannelsWithExit 'Increment 10A graph JSON first fresh run' $Hum @('graph', $FoundationTextBraces)
  $FoundationGraphRepeat = Read-NativeChannelsWithExit 'Increment 10A graph JSON second fresh run' $Hum @('graph', $FoundationTextBraces)
  foreach ($Observed in @($FoundationGraph, $FoundationGraphRepeat)) {
    if ($Observed.ExitCode -ne 0 -or $Observed.Stderr -ne '' -or ($Observed.Stdout + $Observed.Stderr).Contains('H0001') -or ($Observed.Stdout + $Observed.Stderr).Contains('H0003') -or ($Observed.Stdout + $Observed.Stderr).Contains('H0004')) { throw 'Increment 10A Text braces changed parser diagnostic ownership in graph JSON' }
    Assert-Json 'Increment 10A graph JSON' $Observed.Stdout
  }
  if ($FoundationGraph.Stdout -cne $FoundationGraphRepeat.Stdout -or $FoundationGraph.Stderr -cne $FoundationGraphRepeat.Stderr -or $FoundationGraph.ExitCode -ne $FoundationGraphRepeat.ExitCode) { throw 'Increment 10A graph observations are not byte-identical across fresh runs' }
  $FoundationGraphReport = $FoundationGraph.Stdout | ConvertFrom-Json
  $ExpectedFileId = 'file:fixtures/foundation/pre_ar_text_braces_pass.hum'
  $ExpectedTaskId = 'item:fixtures/foundation/pre_ar_text_braces_pass.hum:3:1:task-text_braces'
  $ExpectedDoesId = 'section:fixtures/foundation/pre_ar_text_braces_pass.hum:15:3:does'
  $ExpectedExpressionId = 'line:fixtures/foundation/pre_ar_text_braces_pass.hum:16:5'
  $GraphFile = @($FoundationGraphReport.files)
  if ($GraphFile.Count -ne 1 -or $GraphFile[0].id -ne $ExpectedFileId -or $GraphFile[0].path -ne $FoundationTextBraces) { throw 'Increment 10A graph file identity is not the exact stable source identity' }
  $GraphTask = @($GraphFile[0].items | Where-Object { $_.kind -eq 'task' -and $_.name -eq 'text_braces' })
  if ($GraphTask.Count -ne 1 -or $GraphTask[0].id -ne $ExpectedTaskId -or $GraphTask[0].span.line -ne 3 -or $GraphTask[0].span.column -ne 1) { throw 'Increment 10A graph task identity or source endpoint drifted' }
  $GraphSymbol = @($GraphFile[0].symbols | Where-Object { $_.id -eq $ExpectedTaskId })
  if ($GraphSymbol.Count -ne 1 -or $GraphSymbol[0].kind -ne 'task' -or $GraphSymbol[0].name -ne 'text_braces') { throw 'Increment 10A graph symbol does not join the exact task identity' }
  $DoesFold = @($GraphFile[0].folding_ranges | Where-Object { $_.id -eq $ExpectedDoesId })
  if ($DoesFold.Count -ne 1 -or $DoesFold[0].kind -ne 'section' -or $DoesFold[0].name -ne 'does' -or $DoesFold[0].owner.id -ne $ExpectedTaskId -or $DoesFold[0].owner.kind -ne 'task' -or $DoesFold[0].start_line -ne 15 -or $DoesFold[0].end_line -ne 16) { throw 'Increment 10A graph block relationship does not join the exact does section to its task' }
  $DoesSection = @($GraphTask[0].sections | Where-Object { $_.id -eq $ExpectedDoesId })
  if ($DoesSection.Count -ne 1 -or $DoesSection[0].name -ne 'does' -or $DoesSection[0].lines -ne 1) { throw 'Increment 10A graph does block projection is incomplete' }
  $ReturnExpression = @($DoesSection[0].line_items)
  if ($ReturnExpression.Count -ne 1 -or $ReturnExpression[0].id -ne $ExpectedExpressionId -or $ReturnExpression[0].text -ne 'return "}{"' -or $ReturnExpression[0].span.file -ne $FoundationTextBraces -or $ReturnExpression[0].span.line -ne 16 -or $ReturnExpression[0].span.column -ne 5 -or -not $ReturnExpression[0].meaningful) { throw 'Increment 10A graph expression identity, range, or block endpoint drifted' }
  $GraphObligation = @($GraphTask[0].test_obligations | Where-Object { $_.source_section -eq 'ensures' })
  $GraphPlace = @($FoundationGraphReport.predicate_place_facts | Where-Object { $_.task -eq 'text_braces' -and $_.section -eq 'ensures' })
  if ($GraphObligation.Count -ne 1 -or $GraphObligation[0].predicate_recognition_status -ne 'recognized_typed_executable_predicate_v2' -or $GraphPlace.Count -ne 1 -or $GraphPlace[0].text -ne 'result' -or $GraphPlace[0].definition_id -ne 'predicate-result:fixtures/foundation/pre_ar_text_braces_pass.hum:3:1:text_braces') { throw 'Increment 10A graph expression relationship does not retain the exact task, predicate, and result endpoints' }
  $FoundationRuntime = Read-NativeChannelsWithExit 'Increment 10A exact Text runtime first fresh run' $Hum @('run', $FoundationTextBraces)
  $FoundationRuntimeRepeat = Read-NativeChannelsWithExit 'Increment 10A exact Text runtime second fresh run' $Hum @('run', $FoundationTextBraces)
  if ($FoundationRuntime.Stdout -cne $FoundationRuntimeRepeat.Stdout -or $FoundationRuntime.Stderr -cne $FoundationRuntimeRepeat.Stderr -or $FoundationRuntime.ExitCode -ne $FoundationRuntimeRepeat.ExitCode) { throw 'Increment 10A runtime observations are not byte-identical across fresh runs' }
  $FoundationRuntimeStdout = $FoundationRuntime.Stdout.Replace([Environment]::NewLine, [string][char]10)
  $FoundationRuntimeExpected = '}{' + [char]10
  if ($FoundationRuntime.ExitCode -ne 0 -or $FoundationRuntime.Stderr -ne '' -or $FoundationRuntimeStdout -cne $FoundationRuntimeExpected -or $FoundationRuntimeRepeat.ExitCode -ne 0 -or $FoundationRuntimeRepeat.Stderr -ne '') { throw 'Increment 10A runtime did not observe exact Text brace bytes' }
  $FoundationUnclosed = 'fixtures/foundation/pre_ar_real_unclosed_block_fail.hum'
  foreach ($Format in @('human', 'json')) {
    $Args = @('check')
    if ($Format -eq 'json') { $Args += '--format=json' }
    $Args += $FoundationUnclosed
    $Result = Read-NativeChannelsWithExit "Increment 10A genuine H0004 $Format" $Hum $Args
    $Combined = $Result.Stdout + $Result.Stderr
    if ($Result.ExitCode -ne 1 -or [regex]::Matches($Combined, 'H0004').Count -ne 1 -or $Combined.Contains('H0001') -or $Combined.Contains('H0003')) { throw "Increment 10A genuine unclosed item must produce exactly H0004 in $Format" }
    if ($Format -eq 'json') { Assert-Json 'Increment 10A genuine H0004 JSON' $Result.Stdout }
  }
  $CoreBodySource = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'src/core_body.rs'))
  foreach ($ForbiddenRecognizer in @('trim_end_matches(''{'')', 'strip_suffix(''{'')', 'has_binary_operator', 'fn classify_line')) {
    if ($CoreBodySource.Contains($ForbiddenRecognizer)) { throw "Increment 10A core_body retained a competing source recognizer: $ForbiddenRecognizer" }
  }

  $VersionJson = Read-NativeOutput 'version JSON' $Hum @('version', '--format', 'json')
  Assert-Json 'version JSON' $VersionJson
  if (-not $VersionJson.Contains('"core_lower": "hum.core_lower.v0"')) { throw 'version JSON is missing hum.core_lower.v0 schema' }
  if (-not $VersionJson.Contains('"core_verify": "hum.core_verify.v0"')) { throw 'version JSON is missing hum.core_verify.v0 schema' }
  if (-not $VersionJson.Contains('"full_type_check": "hum.full_type_check.v0"')) { throw 'version JSON is missing hum.full_type_check.v0 schema' }
  if (-not $VersionJson.Contains('"effect_check": "hum.effect_check.v0"')) { throw 'version JSON is missing hum.effect_check.v0 schema' }
  if (-not $VersionJson.Contains('"ownership_check": "hum.ownership_check.v0"')) { throw 'version JSON is missing hum.ownership_check.v0 schema' }
  if (-not $VersionJson.Contains('"resource_check": "hum.resource_check.v0"')) { throw 'version JSON is missing hum.resource_check.v0 schema' }
  if (-not $VersionJson.Contains('"profile_check": "hum.profile_check.v0"')) { throw 'version JSON is missing hum.profile_check.v0 schema' }

  $ExplainJson = Read-NativeOutput 'diagnostic explain JSON' $Hum @('explain', 'H0201', '--format', 'json')
  Assert-Json 'diagnostic explain JSON' $ExplainJson

  $DiagnosticsJson = Read-NativeOutput 'diagnostic catalog JSON' $Hum @('diagnostics', '--format', 'json')
  Assert-Json 'diagnostic catalog JSON' $DiagnosticsJson
  $DiagnosticsCatalog = $DiagnosticsJson | ConvertFrom-Json
  $DiagnosticCodes = @($DiagnosticsCatalog.diagnostics | ForEach-Object { $_.code })
  if ($DiagnosticsCatalog.count -ne 87 -or $DiagnosticCodes.Count -ne 87 -or @($DiagnosticCodes | Sort-Object -Unique).Count -ne 87) { throw 'canonical diagnostic catalog must expose exactly 87 unique active codes' }
  $DiagnosticCatalogSource = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'src/diagnostic_catalog.rs'))
  foreach ($RegistryEvidence in @('family_interval_failures_are_independent', 'exact_code_identity_and_ownership_failures_are_independent', 'retired_allocations_are_append_only_and_semantically_frozen', 'public_catalog_projection_rejects_every_semantic_field_mismatch', 'catalog_detail_coverage_rejects_missing_duplicate_and_unknown_rows', 'checked_documents_reject_unknown_contradictory_and_missing_projections')) {
    if (-not $DiagnosticCatalogSource.Contains($RegistryEvidence)) { throw "canonical diagnostic registry evidence was removed: $RegistryEvidence" }
  }

  Write-Host '==> Session AO registered cause identity and prior-blocker matrix'
  foreach ($EvidenceTest in @(
    'diagnostic_catalog::tests::cause_registry_rejects_every_identity_and_owner_mutation',
    'diagnostic::tests::occurrence_identity_rejects_every_registered_field_mutation',
    'diagnostic::tests::canonical_identity_rejects_valid_prefix_and_route_substitutions',
    'diagnostic::tests::collector_rejects_duplicate_emission_and_exact_prior_substitution',
    'typed_failure::tests::program_analysis_preserves_distinct_same_cause_occurrences_repeatably',
    'typed_failure::tests::missing_failure_declaration_has_one_effect_owned_occurrence',
    'full_type_check::tests::ao_full_type_consumes_exact_typed_failure_occurrences_and_defers_h0907',
    'effect_check::tests::ao_effect_owns_h0907_once_and_consumes_full_type_prior_exactly',
    'callable::tests::ao_callable_occurrences_keep_relationship_identity_and_precedence',
    'callable::tests::ao_independent_same_line_causes_keep_distinct_occurrences'
  )) {
    Invoke-ExactRustTest "Session AO evidence $EvidenceTest" $Cargo $EvidenceTest
  }

  $AoTypedPath = 'fixtures/diagnostics/session_ao_typed_failure_prior_blocker_fail.hum'
  $AoTypedHuman = Read-NativeOutputWithExit 'Session AO H0901 full type human' $Hum @('full-type-check', $AoTypedPath)
  $AoTypedJson = Read-NativeOutputWithExit 'Session AO H0901 full type JSON' $Hum @('full-type-check', '--format', 'json', $AoTypedPath)
  Assert-Json 'Session AO H0901 full type JSON' $AoTypedJson.Output
  if ($AoTypedHuman.ExitCode -ne 1 -or [regex]::Matches($AoTypedHuman.Output, 'diagnostic=H0901').Count -ne 1) { throw 'Session AO H0901 must have one full-type owner' }
  $AoTypedReport = $AoTypedJson.Output | ConvertFrom-Json
  $AoTypedRows = @($AoTypedReport.typed_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0901' })
  if ($AoTypedJson.ExitCode -ne 1 -or $AoTypedRows.Count -ne 1) { throw 'Session AO H0901 JSON must project one canonical occurrence' }
  $AoTypedJsonRepeat = Read-NativeOutputWithExit 'Session AO H0901 repeat JSON' $Hum @('full-type-check', '--format', 'json', $AoTypedPath)
  if ($AoTypedJson.Output -ne $AoTypedJsonRepeat.Output -or $AoTypedJson.ExitCode -ne $AoTypedJsonRepeat.ExitCode) { throw 'Session AO H0901 public bytes must be repeat-identical' }
  $AoTypedRun = Read-NativeChannelsWithExit 'Session AO H0901 runtime preflight' $Hum @('run', $AoTypedPath, '--entry', 'caller')
  if ($AoTypedRun.ExitCode -ne 2 -or $AoTypedRun.Stdout -ne '' -or [regex]::Matches($AoTypedRun.Stderr, 'error\[H0901\]').Count -ne 1 -or $AoTypedRun.Stderr.Contains('runtime trap')) { throw 'Session AO H0901 runtime must consume one canonical occurrence without executing' }

  $AoH0907Path = 'fixtures/effect_check/session_w_missing_fails_when_fail.hum'
  $AoH0907Full = Read-NativeOutputWithExit 'Session AO H0907 full-type deferral' $Hum @('full-type-check', '--format', 'json', $AoH0907Path)
  if ($AoH0907Full.ExitCode -ne 0 -or $AoH0907Full.Output.Contains('"diagnostic_code": "H0907"') -or -not $AoH0907Full.Output.Contains('accepted_typed_failure_deferred_to_effect_v0')) { throw 'Session AO H0907 must defer exactly through full type' }
  $AoH0907Effect = Read-NativeOutputWithExit 'Session AO H0907 effect owner' $Hum @('effect-check', '--format', 'json', $AoH0907Path)
  Assert-Json 'Session AO H0907 effect owner JSON' $AoH0907Effect.Output
  $AoH0907Report = $AoH0907Effect.Output | ConvertFrom-Json
  $AoH0907Rows = @($AoH0907Report.effect_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0907' })
  if ($AoH0907Effect.ExitCode -ne 1 -or $AoH0907Rows.Count -ne 1) { throw 'Session AO H0907 must be owned once by effect checking' }
  $AoH0907Run = Read-NativeChannelsWithExit 'Session AO H0907 runtime preflight' $Hum @('run', $AoH0907Path, '--entry', 'caller')
  if ($AoH0907Run.ExitCode -ne 2 -or $AoH0907Run.Stdout -ne '' -or [regex]::Matches($AoH0907Run.Stderr, 'error\[H0907\]').Count -ne 1 -or $AoH0907Run.Stderr.Contains('runtime trap')) { throw 'Session AO H0907 runtime must consume the effect-owned occurrence without executing' }

  $AoCallablePrior = Read-NativeOutputWithExit 'Session AO typed-failure precedence over callable' $Hum @('full-type-check', '--format', 'json', 'fixtures/diagnostics/session_ao_callable_prior_blocker_fail.hum')
  if ($AoCallablePrior.ExitCode -ne 1 -or [regex]::Matches($AoCallablePrior.Output, '"diagnostic_code": "H0901"').Count -ne 1 -or $AoCallablePrior.Output.Contains('H1401') -or $AoCallablePrior.Output.Contains('H1402')) { throw 'Session AO registered H090 precedence must suppress only the related callable cause' }

  $AoAdjacent = Read-NativeOutputWithExit 'Session AO adjacent distinct callable causes' $Hum @('full-type-check', '--format', 'json', 'fixtures/diagnostics/session_ao_adjacent_distinct_causes_fail.hum')
  Assert-Json 'Session AO adjacent distinct callable causes JSON' $AoAdjacent.Output
  $AoAdjacentFacts = @(($AoAdjacent.Output | ConvertFrom-Json).callable_facts.diagnostic_facts)
  if ($AoAdjacent.ExitCode -ne 1 -or $AoAdjacentFacts.Count -ne 3 -or @($AoAdjacentFacts | Where-Object { $_.code -eq 'H1401' }).Count -ne 2 -or @($AoAdjacentFacts | Where-Object { $_.code -eq 'H1402' }).Count -ne 1 -or @($AoAdjacentFacts.id | Sort-Object -Unique).Count -ne 3) { throw 'Session AO independent same-line causes must remain three distinct canonical occurrences' }

  $AoSameCode = Read-NativeOutputWithExit 'Session AO same-code distinct H0901 occurrences' $Hum @('full-type-check', '--format', 'json', 'fixtures/diagnostics/session_ao_same_code_distinct_occurrences_fail.hum')
  Assert-Json 'Session AO same-code distinct H0901 occurrences JSON' $AoSameCode.Output
  $AoSameRows = @(($AoSameCode.Output | ConvertFrom-Json).typed_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0901' })
  if ($AoSameCode.ExitCode -ne 1 -or $AoSameRows.Count -ne 2 -or $AoSameRows[0].call_span.line -eq $AoSameRows[1].call_span.line) { throw 'Session AO same code and cause must preserve two source occurrences' }

  foreach ($PublicOutput in @($AoTypedHuman.Output, $AoTypedJson.Output, $AoH0907Full.Output, $AoH0907Effect.Output, $AoCallablePrior.Output, $AoAdjacent.Output, $AoSameCode.Output)) {
    foreach ($PrivateField in @('occurrence_id', 'cause_key', 'semantic_owner', 'owning_stage', 'semantic_origin', 'relationship_route')) {
      if ($PublicOutput.Contains($PrivateField)) { throw "Session AO internal field leaked publicly: $PrivateField" }
    }
  }

  Write-Host '==> Session AQ runtime and top-level occurrence composition matrix'
  foreach ($EvidenceTest in @(
    'diagnostic::tests::session_aq_collector_order_duplicate_owner_and_replacement_matrix',
    'diagnostic_catalog::tests::session_aq_every_code_cause_owner_stage_and_precedence_has_validation',
    'tests::post_aq_real_command_app_reanalysis_composes_exactly',
    'tests::post_aq_runtime_uses_main_composed_occurrence_authority',
    'tests::session_aq_real_load_app_old_to_new_and_old_to_absence',
    'tests::session_aq_real_load_app_removal_mutation_matrix',
    'tests::session_aq_real_app_positions_prior_and_insertion_order_fail_closed',
    'tests::session_aq_capability_is_insert_only_and_exact',
    'run::tests::session_aq_static_runtime_causes_are_consumed_once_before_adapters',
    'run::tests::session_aq_execution_time_use_after_move_survivor_is_internal_invariant',
    'run::tests::session_aq_reachable_second_ownership_occurrence_is_consumed_exactly',
    'run::tests::session_aq_behavioral_legacy_classifier_witness_fails_wrong_occurrence',
    'run::tests::session_aq_runtime_producer_substitutions_fail_closed',
    'ownership_check::tests::runtime_h0801_blocker_carries_structured_producer_facts',
    'run::tests::session_aq_same_code_and_app_scope_occurrences_remain_exact',
    'run::tests::callable_preflight_rejects_before_output_adapter',
    'run::tests::reachable_predicate_preflight_precedes_output_adapter',
    'run::tests::divide_zero_fails_needs_with_caller_blame',
    'run::tests::wrong_add_fixture_fails_ensures_with_task_blame'
  )) {
    Invoke-ExactRustTest "Session AQ evidence $EvidenceTest" $Cargo $EvidenceTest
  }

  $AqTypedPath = 'fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum'
  $AqTypedRun = Read-NativeChannelsWithExit 'Session AQ shared typed-failure runtime' $Hum @('run', '--entry', 'typed_failure_probe', $AqTypedPath)
  if ($AqTypedRun.ExitCode -ne 2 -or $AqTypedRun.Stdout -ne '' -or [regex]::Matches($AqTypedRun.Stderr, 'error\[H0901\]').Count -ne 1 -or $AqTypedRun.Stderr.Contains('runtime trap')) { throw 'Session AQ typed-failure runtime must consume one static occurrence before adapters' }
  $AqTypedRepeat = Read-NativeChannelsWithExit 'Session AQ shared typed-failure runtime repeat' $Hum @('run', '--entry', 'typed_failure_probe', $AqTypedPath)
  if ($AqTypedRun.ExitCode -ne $AqTypedRepeat.ExitCode -or $AqTypedRun.Stdout -ne $AqTypedRepeat.Stdout -or $AqTypedRun.Stderr -ne $AqTypedRepeat.Stderr) { throw 'Session AQ typed-failure runtime bytes must be identical across fresh executions' }

  $AqOwnershipPath = 'fixtures/diagnostics/session_aq_static_runtime_shared_ownership_fail.hum'
  $AqOwnershipRun = Read-NativeChannelsWithExit 'Session AQ shared ownership runtime' $Hum @('run', '--entry', 'ownership_probe', $AqOwnershipPath)
  if ($AqOwnershipRun.ExitCode -ne 2 -or $AqOwnershipRun.Stdout -ne '' -or [regex]::Matches($AqOwnershipRun.Stderr, 'error\[H0801\]').Count -ne 1 -or [regex]::Matches($AqOwnershipRun.Stderr, 'runtime trap: H0801').Count -ne 1) { throw 'Session AQ ownership runtime must preserve one shared occurrence and its existing runtime channel' }
  $AqOwnershipExpected = @(
    "$AqOwnershipPath`:12:5: error[H0801]: value ``value`` was used after it was moved",
    "  help: Fix task ``ownership_probe``: ``value`` moved at $AqOwnershipPath`:11:5; use it before that move or create a fresh owned value.",
    'runtime trap: H0801 use after move'
  ) -join "`n"
  if ($AqOwnershipRun.Stderr.Replace(([string][char]13 + [string][char]10), [string][char]10).TrimEnd() -ne $AqOwnershipExpected) { throw 'Session AQ H0801 sealed runtime projection changed' }
  $AqOwnershipRepeat = Read-NativeChannelsWithExit 'Session AQ shared ownership runtime repeat' $Hum @('run', '--entry', 'ownership_probe', $AqOwnershipPath)
  if ($AqOwnershipRun.ExitCode -ne $AqOwnershipRepeat.ExitCode -or $AqOwnershipRun.Stdout -ne $AqOwnershipRepeat.Stdout -or $AqOwnershipRun.Stderr -ne $AqOwnershipRepeat.Stderr) { throw 'Session AQ ownership runtime bytes must be identical across fresh executions' }

  $AqSecondOwnershipPath = 'fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum'
  $AqSecondOwnershipRun = Read-NativeChannelsWithExit 'Session AQ exact reachable second ownership occurrence' $Hum @('run', '--allow', 'stdout.write', $AqSecondOwnershipPath)
  if ($AqSecondOwnershipRun.ExitCode -ne 2 -or $AqSecondOwnershipRun.Stdout -ne '' -or [regex]::Matches($AqSecondOwnershipRun.Stderr, 'error\[H0801\]').Count -ne 1 -or [regex]::Matches($AqSecondOwnershipRun.Stderr, 'runtime trap: H0801').Count -ne 1 -or -not $AqSecondOwnershipRun.Stderr.Contains("$AqSecondOwnershipPath`:40:7")) { throw 'Session AQ runtime must consume only the exact reachable second ownership occurrence before output' }
  $AqSecondOwnershipExpected = @(
    "$AqSecondOwnershipPath`:40:7: error[H0801]: value ``second`` was used after it was moved",
    "  help: Fix task ``reachable_second``: ``second`` moved at $AqSecondOwnershipPath`:38:7; use it before that move or create a fresh owned value.",
    'runtime trap: H0801 use after move'
  ) -join "`n"
  if ($AqSecondOwnershipRun.Stderr.Replace(([string][char]13 + [string][char]10), [string][char]10).TrimEnd() -ne $AqSecondOwnershipExpected) { throw 'Session AQ reachable H0801 projection or runtime channel changed' }
  $AqSecondOwnershipRepeat = Read-NativeChannelsWithExit 'Session AQ exact reachable second ownership occurrence repeat' $Hum @('run', '--allow', 'stdout.write', $AqSecondOwnershipPath)
  if ($AqSecondOwnershipRun.ExitCode -ne $AqSecondOwnershipRepeat.ExitCode -or $AqSecondOwnershipRun.Stdout -ne $AqSecondOwnershipRepeat.Stdout -or $AqSecondOwnershipRun.Stderr -ne $AqSecondOwnershipRepeat.Stderr) { throw 'Session AQ reachable second ownership bytes must be identical across fresh executions' }

  $AqSameCodePath = 'fixtures/diagnostics/session_aq_same_code_distinct_occurrences_fail.hum'
  $AqSameCode = Read-NativeOutputWithExit 'Session AQ same-code occurrence JSON' $Hum @('full-type-check', '--format', 'json', $AqSameCodePath)
  Assert-Json 'Session AQ same-code occurrence JSON' $AqSameCode.Output
  $AqSameRows = @(($AqSameCode.Output | ConvertFrom-Json).typed_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0901' })
  if ($AqSameCode.ExitCode -ne 1 -or $AqSameRows.Count -ne 2 -or $AqSameRows[0].call_span.line -ge $AqSameRows[1].call_span.line) { throw 'Session AQ same-code occurrences must remain distinct in semantic source order' }
  $AqSameCodeRepeat = Read-NativeOutputWithExit 'Session AQ same-code occurrence JSON repeat' $Hum @('full-type-check', '--format', 'json', $AqSameCodePath)
  if ($AqSameCode.ExitCode -ne $AqSameCodeRepeat.ExitCode -or $AqSameCode.Output -ne $AqSameCodeRepeat.Output) { throw 'Session AQ same-code JSON must be byte-identical across fresh executions' }

  $AqAppPath = 'fixtures/diagnostics/session_aq_app_scope_reanalysis_fail.hum'
  $AqApp = Read-NativeChannelsWithExit 'Session AQ app capability scope' $Hum @('run', $AqAppPath)
  if ($AqApp.ExitCode -ne 1 -or $AqApp.Stdout -ne '' -or [regex]::Matches($AqApp.Stderr, 'error\[H0621\]').Count -ne 1) { throw 'Session AQ app scope must preserve one canonical capability diagnostic' }
  $AqAppRepeat = Read-NativeChannelsWithExit 'Session AQ app capability scope repeat' $Hum @('run', $AqAppPath)
  if ($AqApp.ExitCode -ne $AqAppRepeat.ExitCode -or $AqApp.Stdout -ne $AqAppRepeat.Stdout -or $AqApp.Stderr -ne $AqAppRepeat.Stderr) { throw 'Session AQ app scope output must be byte-identical across fresh executions' }

  $AqFixturePaths = @(
    'fixtures/diagnostics/session_aq_app_scope_reanalysis_fail.hum',
    'fixtures/diagnostics/session_aq_reachable_second_ownership_occurrence_fail.hum',
    'fixtures/diagnostics/session_aq_same_code_distinct_occurrences_fail.hum',
    'fixtures/diagnostics/session_aq_static_runtime_shared_cause_fail.hum',
    'fixtures/diagnostics/session_aq_static_runtime_shared_ownership_fail.hum'
  )
  $AqSurfaceMatrix = @(
    @{ Name = 'check human'; Args = @('check'); Json = $false },
    @{ Name = 'check JSON'; Args = @('check', '--format=json'); Json = $true },
    @{ Name = 'resolve human'; Args = @('resolve'); Json = $false },
    @{ Name = 'resolve JSON'; Args = @('resolve', '--format=json'); Json = $true },
    @{ Name = 'type-env human'; Args = @('type-env'); Json = $false },
    @{ Name = 'type-env JSON'; Args = @('type-env', '--format=json'); Json = $true },
    @{ Name = 'type-check human'; Args = @('type-check'); Json = $false },
    @{ Name = 'type-check JSON'; Args = @('type-check', '--format=json'); Json = $true },
    @{ Name = 'full-type-check human'; Args = @('full-type-check'); Json = $false },
    @{ Name = 'full-type-check JSON'; Args = @('full-type-check', '--format=json'); Json = $true },
    @{ Name = 'effect-check human'; Args = @('effect-check'); Json = $false },
    @{ Name = 'effect-check JSON'; Args = @('effect-check', '--format=json'); Json = $true },
    @{ Name = 'ownership-check human'; Args = @('ownership-check'); Json = $false },
    @{ Name = 'ownership-check JSON'; Args = @('ownership-check', '--format=json'); Json = $true },
    @{ Name = 'resource-check human'; Args = @('resource-check'); Json = $false },
    @{ Name = 'resource-check JSON'; Args = @('resource-check', '--format=json'); Json = $true },
    @{ Name = 'profile-check human'; Args = @('profile-check'); Json = $false },
    @{ Name = 'profile-check JSON'; Args = @('profile-check', '--format=json'); Json = $true },
    @{ Name = 'core-preview human'; Args = @('core-preview'); Json = $false },
    @{ Name = 'core-preview JSON'; Args = @('core-preview', '--format=json'); Json = $true },
    @{ Name = 'core-lower human'; Args = @('core-lower'); Json = $false },
    @{ Name = 'core-lower JSON'; Args = @('core-lower', '--format=json'); Json = $true },
    @{ Name = 'core-verify human'; Args = @('core-verify'); Json = $false },
    @{ Name = 'core-verify JSON'; Args = @('core-verify', '--format=json'); Json = $true },
    @{ Name = 'graph'; Args = @('graph'); Json = $true },
    @{ Name = 'ir-readiness human'; Args = @('ir-readiness'); Json = $false },
    @{ Name = 'ir-readiness JSON'; Args = @('ir-readiness', '--format=json'); Json = $true }
  )
  foreach ($AqFixture in $AqFixturePaths) {
    foreach ($AqSurface in $AqSurfaceMatrix) {
      $AqArgs = @($AqSurface.Args) + @($AqFixture)
      $AqFirst = Read-NativeChannelsWithExit "Session AQ $($AqSurface.Name) $AqFixture" $Hum $AqArgs
      $AqSecond = Read-NativeChannelsWithExit "Session AQ repeat $($AqSurface.Name) $AqFixture" $Hum $AqArgs
      if ($AqFirst.ExitCode -notin @(0, 1) -or $AqFirst.ExitCode -ne $AqSecond.ExitCode -or $AqFirst.Stdout -cne $AqSecond.Stdout -or $AqFirst.Stderr -cne $AqSecond.Stderr) { throw "Session AQ cross-surface output changed or failed: $($AqSurface.Name) $AqFixture" }
      if ($AqSurface.Json) { Assert-Json "Session AQ $($AqSurface.Name) $AqFixture" $AqFirst.Stdout }
      $AqCombinedOutput = $AqFirst.Stdout + $AqFirst.Stderr
      foreach ($PrivateField in @('occurrence_id', 'cause_key', 'owning_stage', 'relationship_route')) {
        if ($AqCombinedOutput.Contains($PrivateField)) { throw "Session AQ cross-surface internal field leaked: $PrivateField in $($AqSurface.Name) $AqFixture" }
      }
    }
  }

  foreach ($PublicOutput in @($AqTypedRun.Stderr, $AqOwnershipRun.Stderr, $AqSameCode.Output, $AqApp.Stderr)) {
    foreach ($PrivateField in @('occurrence_id', 'cause_key', 'semantic_owner', 'owning_stage', 'semantic_origin', 'relationship_route')) {
      if ($PublicOutput.Contains($PrivateField)) { throw "Session AQ internal field leaked publicly: $PrivateField" }
    }
  }

  function Get-AqRustProductionSource([string]$Path) {
    $Source = [System.IO.File]::ReadAllText((Resolve-Path $Path))
    $TerminalTestModules = [regex]::Matches($Source, '(?m)^#\[cfg\(test\)\]\r?\nmod tests\s*\{')
    if ($TerminalTestModules.Count -gt 1) { throw "Session AQ source extraction found ambiguous terminal test modules in $Path" }
    if ($TerminalTestModules.Count -eq 0) { return $Source }
    return $Source.Substring(0, $TerminalTestModules[0].Index)
  }
  $AqMainProduction = Get-AqRustProductionSource 'src/main.rs'
  $AqRunProduction = Get-AqRustProductionSource 'src/run.rs'
  $AqAppProduction = Get-AqRustProductionSource 'src/app_entry.rs'
  $AqCapabilityProduction = Get-AqRustProductionSource 'src/capability_root.rs'
  $AqDiagnosticProduction = Get-AqRustProductionSource 'src/diagnostic.rs'
  $AqOwnershipProduction = Get-AqRustProductionSource 'src/ownership_check.rs'
  if (-not $AqRunProduction.Contains('run_program_with_occurrences_and_file_adapters') -or -not $AqRunProduction.Contains('fn location(span: &Span)') -or $AqRunProduction.Contains('session_aq_execution_time_use_after_move_survivor_is_internal_invariant')) { throw 'Session AQ production-source extraction truncated at a field- or method-level cfg(test) attribute or retained the terminal test module' }
  foreach ($ForbiddenRuntimeH0801 in @(
    '\bDiagnosticCode\s*::\s*USE_AFTER_MOVE\b',
    '\bUSE_AFTER_MOVE\b',
    'DiagnosticCauseKey\s*::\s*producer_owned\s*\(\s*110\s*\)',
    'value_used_after_move_v0',
    '"H0801"'
  )) {
    if ([regex]::IsMatch($AqRunProduction, $ForbiddenRuntimeH0801)) { throw "Session AQ runtime production directly reconstructs or selects H0801: $ForbiddenRuntimeH0801" }
  }
  foreach ($ForbiddenClassifier in @('is_app_entry_diagnostic', 'is_app_entry_code', 'is_capability_diagnostic', 'is_capability_code', 'diagnostics.retain(|diagnostic|', 'message.starts_with("H', 'consume_first_owned_origin', 'consume_first_owned_origin_prefix', 'consume_matching_projection', 'consume_matching_projection_any_owner', 'consume_first_owned_stage')) {
    if ($AqMainProduction.Contains($ForbiddenClassifier) -or $AqRunProduction.Contains($ForbiddenClassifier) -or $AqAppProduction.Contains($ForbiddenClassifier) -or $AqCapabilityProduction.Contains($ForbiddenClassifier) -or $AqDiagnosticProduction.Contains($ForbiddenClassifier) -or $AqOwnershipProduction.Contains($ForbiddenClassifier)) { throw "Session AQ superseded production classifier remains: $ForbiddenClassifier" }
  }
  foreach ($RequiredTransport in @('remove_reanalyzed_projection', 'remove_loaded_app_projections', 'insert_capability_projections', 'ReanalysisProducer', 'runtime_diagnostic_occurrences', 'run_program_with_occurrences_and_adapters', 'consume_exact', 'emit_exact_occurrences', 'runtime_use_after_move_blockers', 'OwnershipRuntimeBlocker')) {
    if (-not ($AqMainProduction + $AqRunProduction + $AqDiagnosticProduction + $AqOwnershipProduction).Contains($RequiredTransport)) { throw "Session AQ canonical transport evidence is missing: $RequiredTransport" }
  }
  $AqCapabilityInsertionSignature = [regex]::Match($AqMainProduction, '(?s)fn insert_capability_projections\((.*?)\)\s*->')
  if (-not $AqCapabilityInsertionSignature.Success -or $AqCapabilityInsertionSignature.Groups[1].Value.Contains('ReanalyzableProjection') -or $AqCapabilityInsertionSignature.Groups[1].Value.Contains('BTreeMap')) { throw 'Session AQ capability insertion must remain structurally unable to create an authoritative-old ledger entry' }
  if ([regex]::Matches($AqMainProduction, 'producer:\s*ReanalysisProducer::AppEntry').Count -ne 1 -or [regex]::Matches($AqMainProduction, 'producer:\s*ReanalysisProducer::CapabilityRoot').Count -ne 0) { throw 'Session AQ production ledger must classify only real per-file app authorities as removable old projections' }
  $AqRawProduction = @()
  foreach ($SourceFile in Get-ChildItem -Path 'src' -Filter '*.rs') {
    if ($SourceFile.Name -eq 'diagnostic_catalog.rs') { continue }
    $Production = Get-AqRustProductionSource $SourceFile.FullName
    if ([regex]::IsMatch($Production, '"H\d{4}"')) { $AqRawProduction += $SourceFile.Name }
  }
  $AqAllowedPresentation = @('callable.rs', 'explain.rs', 'full_type_check.rs')
  foreach ($RawFile in $AqRawProduction) {
    if ($AqAllowedPresentation -notcontains $RawFile) { throw "Session AQ found a raw production H-code outside the registry/presentation allowlist: $RawFile" }
  }
  if (-not $AqRunProduction.Contains('DiagnosticOccurrenceCollector::from_authority') -or $AqRunProduction.Contains('DiagnosticOccurrence::registered') -or $AqRunProduction.Contains('DiagnosticOccurrence::producer_diagnostic')) { throw 'Session AQ runtime must consume authority and must not mint static/shared occurrences' }

  Write-Host '==> Session AP static cause registry and prior-blocker propagation matrix'
  foreach ($EvidenceTest in @(
    'diagnostic_catalog::tests::session_ap_static_emitters_have_one_registered_default_owner',
    'diagnostic::tests::exact_precedence_rejects_every_relationship_and_occurrence_substitution',
    'diagnostic::tests::code_only_and_public_identical_alternative_causes_fail_closed',
    'diagnostic::tests::registered_ap_rules_dispatch_only_to_their_named_consumers',
    'diagnostic::tests::occurrence_set_rejects_every_prior_blocker_projection_corruption',
    'diagnostic::tests::every_static_stage_projection_rejects_independent_corruption',
    'diagnostic::tests::static_prior_blocker_chain_preserves_exact_type_occurrence',
    'diagnostic::tests::same_line_type_causes_keep_distinct_semantic_origins',
    'diagnostic::tests::parser_and_resolver_causes_remain_distinct_through_type_environment',
    'diagnostic::tests::path_and_authority_precedence_preserve_only_fundamental_static_causes',
    'diagnostic::tests::effect_and_ownership_blockers_keep_one_owner_through_later_gates',
    'diagnostic::tests::ownership_precedence_requires_exact_resolver_call_occurrences',
    'app_entry::tests::app_occurrence_identity_is_structural_and_display_name_independent',
    'parser::tests::parser_occurrence_identity_uses_only_producer_owned_file_and_event_facts',
    'check::tests::check_occurrence_identity_does_not_depend_on_display_path',
    'predicate::tests::typed_predicate_owner_is_independent_of_rendered_reason',
    'writable_field_alias::tests::typed_alias_cause_is_sealed_and_rendering_independent',
    'typed_failure::tests::statement_analysis_cannot_mint_occurrence_before_resolver_binding',
    'typed_failure::tests::typed_failure_cause_enum_is_closed_and_registry_exact',
    'typed_failure::tests::exact_call_spans_and_identifier_ownership_fail_closed',
    'typed_failure::tests::resolver_call_binding_rejects_missing_duplicate_ambiguous_and_wrong_identity',
    'parser::tests::parser_body_syntax_owns_repeated_sibling_and_nested_calls',
    'resolve::tests::parser_precedence_is_consumed_for_a_genuine_shared_item_node',
    'resolve::tests::resolver_mints_distinct_repeated_sibling_and_nested_call_occurrences',
    'resolve::tests::resolver_call_identity_ignores_retained_section_text_after_parsing',
    'resolve::tests::later_resolver_call_identity_ignores_earlier_retained_statement_references',
    'type_check::tests::resolver_precedence_is_consumed_for_a_genuine_blocked_type_relationship',
    'capability_root::tests::capability_occurrence_identity_uses_structural_items_not_display_names',
    'capability_root::tests::call_occurrence_policy_ids_are_unique_and_repeatable',
    'ownership_check::tests::nested_use_after_move_binds_the_innermost_resolver_call',
    'core_verify::tests::core_transport_rejects_projection_regenerated_or_corrupted_downstream',
    'graph::tests::graph_rejects_corrupt_occurrence_projection_before_serialization'
  )) {
    Invoke-ExactRustTest "Session AP evidence $EvidenceTest" $Cargo $EvidenceTest
  }
  $ExactRustSelectorCredits = @(Get-ExactRustSelectorCredits)
  $UniqueExactRustSelectorCredits = @($ExactRustSelectorCredits | Sort-Object -Unique)
  if ($ExactRustSelectorCredits.Count -ne 75 -or $UniqueExactRustSelectorCredits.Count -ne 75) { throw "exact Rust selector inventory must credit 75 unique tests, credited $($ExactRustSelectorCredits.Count) invocations and $($UniqueExactRustSelectorCredits.Count) unique tests" }
  if ($ExactRustSelectorCredits -notcontains 'typed_failure::tests::exact_call_spans_and_identifier_ownership_fail_closed') { throw 'exact Rust selector inventory lost the typed-failure call-identity boundary test' }

  $ApForbiddenFallbacks = @(Get-ChildItem -Path 'src' -Filter '*.rs' | Where-Object { $_.Name -ne 'diagnostic_catalog.rs' } | Select-String -Pattern 'default_emitter_cause|registered_default|from_diagnostics|validate_owned_diagnostics')
  if ($ApForbiddenFallbacks.Count -ne 0) { throw 'Session AP production source must not reconstruct occurrences from codes or public diagnostics' }
  $ApCatalogSource = Get-Content -Raw 'src/diagnostic_catalog.rs'
  if (-not [regex]::IsMatch($ApCatalogSource, '#\[cfg\(test\)\]\s+pub\(crate\) fn default_emitter_cause')) { throw 'Session AP emitter-default lookup must remain test-only registry-baseline evidence' }
  $ApTypedFailureSource = Get-Content -Raw 'src/typed_failure.rs'
  if ([regex]::IsMatch($ApTypedFailureSource, 'diagnostic_cause_for_reason|diagnostic_cause\s*\(|registered_reason|default_emitter_cause')) { throw 'Session AP typed-failure producer must carry an opaque cause key rather than select from codes, reasons, or defaults' }
  if (-not $ApTypedFailureSource.Contains('fact.cause') -or -not $ApTypedFailureSource.Contains('cause.key()') -or -not $ApTypedFailureSource.Contains('diagnostic_cause_for_key(cause_key)')) { throw 'Session AP typed-failure producer-owned cause transport evidence is missing' }
  $ApResolverProductionSource = [regex]::Replace((Get-Content -Raw 'src/resolve.rs'), '(?s)#\[cfg\(test\)\].*$', '')
  $ApParserProductionSource = [regex]::Replace((Get-Content -Raw 'src/parser.rs'), '(?s)#\[cfg\(test\)\]\s*mod tests\s*\{.*$', '')
  $ApCapabilityProductionSource = [regex]::Replace((Get-Content -Raw 'src/capability_root.rs'), '(?s)#\[cfg\(test\)\].*$', '')
  $ApOwnershipProductionSource = [regex]::Replace((Get-Content -Raw 'src/ownership_check.rs'), '(?s)#\[cfg\(test\)\]\s*mod tests\s*\{.*$', '')
  $ApTypedProductionSource = [regex]::Replace($ApTypedFailureSource, '(?s)#\[cfg\(test\)\].*$', '')
  if ($ApResolverProductionSource.Contains('calls_in_expression') -or $ApCapabilityProductionSource.Contains('calls_in_expression')) { throw 'Session AP resolver/capability producers must not rescan source text to mint executable call identity' }
  if ($ApParserProductionSource.Contains('scan_raw_executable_calls') -or $ApParserProductionSource.Contains('scan_identifier_tokens') -or $ApParserProductionSource.Contains('executable_calls_in_statement')) { throw 'Session AP must remove the late retained-text executable-call scanner rather than rename its resolver entry point' }
  if (-not $ApResolverProductionSource.Contains('executable_call_nodes(body_syntax)') -or $ApResolverProductionSource.Contains('executable_call_nodes(sections)')) { throw 'Session AP resolver call occurrences must come only from the parser-produced body syntax tree' }
  if ($ApResolverProductionSource.Contains('semantic_internal_reference_identity') -or -not $ApResolverProductionSource.Contains('resolver_call_reference_identity(') -or -not $ApResolverProductionSource.Contains('unresolved_call_target_identity(')) { throw 'Session AP private resolver-call identity must derive from owner, target status, and parser-owned call position rather than the global public reference serial' }
  $ApCallReferenceIdentity = [regex]::Match($ApResolverProductionSource, '(?s)fn resolver_call_reference_identity\(.*?\n\}')
  if (-not $ApCallReferenceIdentity.Success -or $ApCallReferenceIdentity.Value.Contains('reference_serial') -or $ApCallReferenceIdentity.Value.Contains('ResolveReference')) { throw 'Session AP private call-reference identity helper must not consume ordinary resolver reference allocation state' }
  $ApCallTraversal = [regex]::Match($ApParserProductionSource, '(?s)pub\(crate\) fn executable_call_nodes\(.*?\n\}\r?\n\r?\nfn collect_executable_calls')
  if (-not $ApCallTraversal.Success -or $ApCallTraversal.Value.Contains('SectionLine') -or $ApCallTraversal.Value.Contains('.text')) { throw 'Session AP executable-call traversal must consume structured ParsedBodyStatement nodes without retained SectionLine text' }
  if ([regex]::Matches($ApParserProductionSource, 'parser_owned_top_level_call_ranges\(').Count -ne 2) { throw 'Session AP parser-owned call recognition must run only during initial expression production' }
  if ($ApOwnershipProductionSource.Contains('call_span_for_identifier_use') -or $ApOwnershipProductionSource.Contains('strip_prefix("resolver_call_occurrence=")')) { throw 'Session AP ownership precedence must carry and compare opaque resolver call identities directly' }
  if ($ApTypedProductionSource.Contains('exact_call_span == fact.call_span') -or $ApCapabilityProductionSource.Contains('exact_call_span == route.primary_span')) { throw 'Session AP semantic producers must not select resolver calls by public display spans' }
  if (-not $ApTypedProductionSource.Contains('fact.resolver_call.as_ref()') -or -not $ApOwnershipProductionSource.Contains('dominant.resolver_call_occurrence()')) { throw 'Session AP exact resolver-call transport evidence is missing' }
  $ApPredicateProductionSource = [regex]::Replace((Get-Content -Raw 'src/predicate.rs'), '(?s)#\[cfg\(test\)\].*$', '')
  if ($ApPredicateProductionSource.Contains('diagnostic_cause_for_reason') -or [regex]::IsMatch($ApPredicateProductionSource, '(?:self\.reason|issue\.reason|\breason)\s*==\s*"')) { throw 'Session AP predicate ownership must come from its closed typed cause, not rendered reason text' }
  if (-not $ApPredicateProductionSource.Contains('PredicateDiagnosticOwner::Predicate(cause)') -or -not $ApPredicateProductionSource.Contains('PredicateDiagnosticOwner::PathBoundary')) { throw 'Session AP predicate typed-owner transport evidence is missing' }
  $ApMainProductionSource = [regex]::Replace((Get-Content -Raw 'src/main.rs'), '(?s)#\[cfg\(test\)\].*$', '')
  foreach ($ForbiddenMainCollector in @(
    'diagnostic_occurrences.retain_codes',
    'app_entry::diagnostic_occurrence_set(&program)',
    'capability_root::diagnostic_occurrence_set(&program)',
    'core_preview::diagnostic_occurrence_set_from_source',
    'core_lower::validate_diagnostic_projection_from_source',
    'core_verify::validate_diagnostic_projection_from_source',
    'ir_readiness::validate_diagnostic_projection_from_source'
  )) {
    if ($ApMainProductionSource.Contains($ForbiddenMainCollector)) { throw "Session AP top-level main boundary contains unauthorized occurrence collection/validation: $ForbiddenMainCollector" }
  }
  if (-not $ApMainProductionSource.Contains('profile_check::diagnostic_transport_from_source') -or -not $ApMainProductionSource.Contains('graph::validate_diagnostic_occurrence_projection') -or -not $ApMainProductionSource.Contains('runtime_diagnostic_occurrences')) { throw 'Session AQ main must retain graph validation and add only the canonical runtime occurrence transport over load_program authority' }
  $ApCoreVerifyProductionSource = [regex]::Replace((Get-Content -Raw 'src/core_verify.rs'), '(?s)#\[cfg\(test\)\].*$', '')
  $ApIrProductionSource = [regex]::Replace((Get-Content -Raw 'src/ir_readiness.rs'), '(?s)#\[cfg\(test\)\].*$', '')
  if (-not $ApCoreVerifyProductionSource.Contains('.validate_against("core_lower", &preview_authority)') -or -not $ApIrProductionSource.Contains('.validate_against("ir_readiness", diagnostic_occurrences)')) { throw 'Session AP Core/IR occurrence validation must remain inside its authorized stage modules' }

  $ApParser = 'fixtures/diagnostics/session_ap_parser_resolver_precedence_fail.hum'
  $ApParserResolve = Read-NativeOutputWithExit 'Session AP parser/resolver owner' $Hum @('resolve', $ApParser)
  if ($ApParserResolve.ExitCode -ne 1 -or [regex]::Matches($ApParserResolve.Output, 'H0001').Count -ne 1 -or [regex]::Matches($ApParserResolve.Output, 'H0601').Count -ne 1) { throw 'Session AP parser/resolver fixture must retain its two independently owned source facts at resolve' }
  foreach ($Stage in @('type-check', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'profile-check')) {
    foreach ($Format in @('human', 'json')) {
      $Args = @($Stage, $ApParser)
      if ($Format -eq 'json') { $Args = @($Stage, '--format', 'json', $ApParser) }
      $Projection = Read-NativeOutputWithExit "Session AP parser blocker $Stage $Format" $Hum $Args
      if ($Projection.ExitCode -ne 1 -or $Projection.Output.Contains('H0601')) { throw "Session AP parser cause must own the downstream public blocker at $Stage $Format" }
      if ($Format -eq 'json') {
        Assert-Json "Session AP parser blocker $Stage JSON" $Projection.Output
        if (-not $Projection.Output.Contains('blocked_by_')) { throw "Session AP parser JSON must preserve downstream blocker status at $Stage" }
      } elseif (-not $Projection.Output.Contains('H0001')) {
        throw "Session AP parser human output must retain H0001 at $Stage"
      }
    }
  }

  $ApSameLine = 'fixtures/diagnostics/session_ap_same_line_independent_causes_fail.hum'
  foreach ($Format in @('human', 'json')) {
    $Args = @('type-check', $ApSameLine)
    if ($Format -eq 'json') { $Args = @('type-check', '--format', 'json', $ApSameLine) }
    $Projection = Read-NativeOutputWithExit "Session AP same-line H0605 $Format" $Hum $Args
    if ($Projection.ExitCode -ne 1 -or [regex]::Matches($Projection.Output, 'H0605').Count -ne 2 -or -not $Projection.Output.Contains('MissingLeft') -or -not $Projection.Output.Contains('MissingRight')) { throw "Session AP same-line H0605 causes must remain distinct in $Format" }
    if ($Format -eq 'json') {
      Assert-Json 'Session AP same-line H0605 JSON' $Projection.Output
      if (($Projection.Output | ConvertFrom-Json).summary.type_errors -ne 2) { throw 'Session AP same-line JSON must retain two type errors' }
    }
  }

  foreach ($Boundary in @(
    @{ Name = 'Path/predicate'; File = 'fixtures/diagnostics/session_ap_path_predicate_precedence_fail.hum'; Owner = 'H0630'; Suppressed = 'H0704' },
    @{ Name = 'authority/ownership'; File = 'fixtures/diagnostics/session_ap_authority_ownership_precedence_fail.hum'; Owner = 'H0618'; Suppressed = 'H0801' }
  )) {
    foreach ($Stage in @('resolve', 'type-check', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'profile-check', 'ir-readiness')) {
      foreach ($Format in @('human', 'json')) {
        $Args = @($Stage, $Boundary.File)
        if ($Format -eq 'json') { $Args = @($Stage, '--format', 'json', $Boundary.File) }
        $Projection = Read-NativeOutputWithExit "Session AP $($Boundary.Name) $Stage $Format" $Hum $Args
        if ($Projection.ExitCode -ne 1 -or $Projection.Output.Contains($Boundary.Suppressed)) { throw "Session AP $($Boundary.Name) precedence disagrees at $Stage $Format" }
        if ($Format -eq 'json') {
          Assert-Json "Session AP $($Boundary.Name) $Stage JSON" $Projection.Output
        } elseif (-not $Projection.Output.Contains($Boundary.Owner)) {
          throw "Session AP $($Boundary.Name) human output lost $($Boundary.Owner) at $Stage"
        }
      }
    }
  }

  $ApTypeChain = 'fixtures/diagnostics/session_ap_prior_blocker_chain_fail.hum'
  foreach ($Stage in @('type-check', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'profile-check')) {
    foreach ($Format in @('human', 'json')) {
      $Args = @($Stage, $ApTypeChain)
      if ($Format -eq 'json') { $Args = @($Stage, '--format', 'json', $ApTypeChain) }
      $Projection = Read-NativeOutputWithExit "Session AP H0605 blocker chain $Stage $Format" $Hum $Args
      if ($Projection.ExitCode -ne 1) { throw "Session AP H0605 prior blocker must keep $Stage blocked in $Format" }
      if ($Format -eq 'json') { Assert-Json "Session AP H0605 blocker chain $Stage JSON" $Projection.Output }
      foreach ($PrivateField in @('occurrence_id', 'cause_key', 'semantic_owner', 'owning_stage', 'semantic_origin', 'relationship_route')) {
        if ($Projection.Output.Contains($PrivateField)) { throw "Session AP internal field leaked publicly: $PrivateField" }
      }
    }
  }

  $ApEffectOwnershipFixture = 'fixtures/diagnostics/session_ap_effect_ownership_precedence_fail.hum'
  $ApEffectOwnership = Read-NativeOutputWithExit 'Session AP effect/ownership owner' $Hum @('effect-check', $ApEffectOwnershipFixture)
  if ($ApEffectOwnership.ExitCode -ne 1 -or [regex]::Matches($ApEffectOwnership.Output, 'H0907').Count -ne 1 -or $ApEffectOwnership.Output.Contains('H0801')) { throw 'Session AP H0907 must own the effect/ownership combined cause' }
  $ApOwnership = 'fixtures/diagnostics/session_ap_ownership_resource_profile_chain_fail.hum'
  $ApOwnershipOwner = Read-NativeOutputWithExit 'Session AP ownership owner' $Hum @('ownership-check', $ApOwnership)
  if ($ApOwnershipOwner.ExitCode -ne 1 -or [regex]::Matches($ApOwnershipOwner.Output, 'H0801').Count -ne 1) { throw 'Session AP ownership cause must be owned once' }
  foreach ($Stage in @('resource-check', 'profile-check')) {
    $Projection = Read-NativeOutputWithExit "Session AP ownership prior blocker $Stage" $Hum @($Stage, $ApOwnership)
    if ($Projection.ExitCode -ne 1 -or -not $Projection.Output.Contains('blocked_by_')) { throw "Session AP ownership blocker must propagate through $Stage" }
  }

  foreach ($GraphFixture in @($ApParser, $ApSameLine, $ApTypeChain, $ApEffectOwnershipFixture, $ApOwnership)) {
    $Projection = Read-NativeOutputWithExit "Session AP authoritative graph projection $GraphFixture" $Hum @('graph', $GraphFixture)
    Assert-Json "Session AP authoritative graph projection $GraphFixture" $Projection.Output
    if ($Projection.Output.Contains('occurrence_id') -or $Projection.Output.Contains('cause_key')) { throw 'Session AP graph projection leaked private diagnostic identity' }
  }

  foreach ($Format in @('human', 'json')) {
    $Args = @('ir-readiness', $ApEffectOwnershipFixture)
    if ($Format -eq 'json') { $Args = @('ir-readiness', '--format', 'json', $ApEffectOwnershipFixture) }
    $Projection = Read-NativeOutputWithExit "Session AP authoritative IR projection $Format" $Hum $Args
    if ($Projection.ExitCode -ne 0 -or $Projection.Output.Contains('diagnostic invariant failure') -or -not $Projection.Output.Contains('effect_errors_v0') -or -not $Projection.Output.Contains('blocked_by_effect_check_errors')) { throw "Session AP authoritative IR projection disagrees for the effect/ownership fixture in $Format" }
    if ($Format -eq 'json') { Assert-Json 'Session AP authoritative IR projection JSON' $Projection.Output }
    foreach ($PrivateField in @('occurrence_id', 'cause_key', 'semantic_owner', 'owning_stage', 'semantic_origin', 'relationship_route')) {
      if ($Projection.Output.Contains($PrivateField)) { throw "Session AP authoritative IR projection leaked private diagnostic identity: $PrivateField" }
    }
  }

  $CapabilitiesJson = Read-NativeOutput 'capabilities JSON' $Hum @('capabilities', '--format', 'json')
  Assert-Json 'capabilities JSON' $CapabilitiesJson
  if (-not $CapabilitiesJson.Contains('"schema": "hum.capabilities.v0"')) { throw 'capabilities JSON is missing hum.capabilities.v0 schema' }
  if (-not $CapabilitiesJson.Contains('"editor_capabilities"')) { throw 'capabilities JSON is missing editor_capabilities' }
  if (-not $CapabilitiesJson.Contains('"document_symbols"')) { throw 'capabilities JSON is missing document_symbols capability' }
  if (-not $CapabilitiesJson.Contains('"semantic_token_legend"')) { throw 'capabilities JSON is missing semantic_token_legend capability' }
  if (-not $CapabilitiesJson.Contains('"lsp_capabilities"')) { throw 'capabilities JSON is missing lsp_capabilities schema' }
  if (-not $CapabilitiesJson.Contains('"evidence_report"')) { throw 'capabilities JSON is missing evidence_report schema' }
  if (-not $CapabilitiesJson.Contains('"math_obligations_report"')) { throw 'capabilities JSON is missing math_obligations_report schema' }
  if (-not $CapabilitiesJson.Contains('"math_obligation"')) { throw 'capabilities JSON is missing math_obligation schema' }
  if (-not $CapabilitiesJson.Contains('"resource_report"')) { throw 'capabilities JSON is missing resource_report schema' }
  if (-not $CapabilitiesJson.Contains('"core_preview"')) { throw 'capabilities JSON is missing core_preview schema' }
  if (-not $CapabilitiesJson.Contains('"core_lower"')) { throw 'capabilities JSON is missing core_lower schema' }
  if (-not $CapabilitiesJson.Contains('"core_lower_json"')) { throw 'capabilities JSON is missing core_lower_json command' }
  if (-not $CapabilitiesJson.Contains('"core_verify"')) { throw 'capabilities JSON is missing core_verify schema' }
  if (-not $CapabilitiesJson.Contains('"core_verify_json"')) { throw 'capabilities JSON is missing core_verify_json command' }
  if (-not $CapabilitiesJson.Contains('"resolve_report"')) { throw 'capabilities JSON is missing resolve_report schema' }
  if (-not $CapabilitiesJson.Contains('"resolve_json"')) { throw 'capabilities JSON is missing resolve_json command' }
  if (-not $CapabilitiesJson.Contains('"type_env"')) { throw 'capabilities JSON is missing type_env schema' }
  if (-not $CapabilitiesJson.Contains('"type_env_json"')) { throw 'capabilities JSON is missing type_env_json command' }
  if (-not $CapabilitiesJson.Contains('"type_check"')) { throw 'capabilities JSON is missing type_check schema' }
  if (-not $CapabilitiesJson.Contains('"type_check_json"')) { throw 'capabilities JSON is missing type_check_json command' }
  if (-not $CapabilitiesJson.Contains('"full_type_check"')) { throw 'capabilities JSON is missing full_type_check schema' }
  if (-not $CapabilitiesJson.Contains('"full_type_check_json"')) { throw 'capabilities JSON is missing full_type_check_json command' }
  if (-not $CapabilitiesJson.Contains('"effect_check"')) { throw 'capabilities JSON is missing effect_check schema' }
  if (-not $CapabilitiesJson.Contains('"effect_check_json"')) { throw 'capabilities JSON is missing effect_check_json command' }
  if (-not $CapabilitiesJson.Contains('"ownership_check"')) { throw 'capabilities JSON is missing ownership_check schema' }
  if (-not $CapabilitiesJson.Contains('"ownership_check_json"')) { throw 'capabilities JSON is missing ownership_check_json command' }
  if (-not $CapabilitiesJson.Contains('"resource_check"')) { throw 'capabilities JSON is missing resource_check schema' }
  if (-not $CapabilitiesJson.Contains('"resource_check_json"')) { throw 'capabilities JSON is missing resource_check_json command' }
  if (-not $CapabilitiesJson.Contains('"profile_check"')) { throw 'capabilities JSON is missing profile_check schema' }
  if (-not $CapabilitiesJson.Contains('"profile_check_json"')) { throw 'capabilities JSON is missing profile_check_json command' }
  if (-not $CapabilitiesJson.Contains('"ir_readiness"')) { throw 'capabilities JSON is missing ir_readiness schema' }
  if (-not $CapabilitiesJson.Contains('"core_contract"')) { throw 'capabilities JSON is missing core_contract schema' }
  if (-not $CapabilitiesJson.Contains('"ir_contract"')) { throw 'capabilities JSON is missing ir_contract schema' }
  if (-not $CapabilitiesJson.Contains('"backend_contract"')) { throw 'capabilities JSON is missing backend_contract schema' }
  if (-not $CapabilitiesJson.Contains('"runtime_profiles"')) { throw 'capabilities JSON is missing runtime_profiles schema' }
  if (-not $CapabilitiesJson.Contains('"runtime_profile"')) { throw 'capabilities JSON is missing runtime_profile schema' }
  if (-not $CapabilitiesJson.Contains('"runtime_profiles_json"')) { throw 'capabilities JSON is missing runtime_profiles_json command' }
  if (-not $CapabilitiesJson.Contains('"state_model"')) { throw 'capabilities JSON is missing state_model schema' }
  if (-not $CapabilitiesJson.Contains('"state_permission"')) { throw 'capabilities JSON is missing state_permission schema' }
  if (-not $CapabilitiesJson.Contains('"state_model_json"')) { throw 'capabilities JSON is missing state_model_json command' }
  if (-not $CapabilitiesJson.Contains('"doctor"')) { throw 'capabilities JSON is missing doctor schema' }
  if (-not $CapabilitiesJson.Contains('"target_facts"')) { throw 'capabilities JSON is missing target_facts schema' }
  if (-not $CapabilitiesJson.Contains('"target_fact_record"')) { throw 'capabilities JSON is missing target_fact_record schema' }

  $CoreContractJson = Read-NativeOutput 'Core contract JSON' $Hum @('core-contract', '--format', 'json')
  Assert-Json 'Core contract JSON' $CoreContractJson
  if (-not $CoreContractJson.Contains('"schema": "hum.core_contract.v0"')) { throw 'Core contract JSON is missing hum.core_contract.v0 schema' }
  if (-not $CoreContractJson.Contains('"lowers_from_schema": "hum.semantic_graph.v0"')) { throw 'Core contract JSON is missing semantic graph source schema' }
  if (-not $CoreContractJson.Contains('"lowers_to_schema": "hum.ir_contract.v0"')) { throw 'Core contract JSON is missing Hum IR target schema' }
  if (-not $CoreContractJson.Contains('"name": "statements"')) { throw 'Core contract JSON is missing statements catalog' }
  if (-not $CoreContractJson.Contains('"output"')) { throw 'Core contract JSON is missing Session Z output effect' }
  if (-not $CoreContractJson.Contains('"set_place"')) { throw 'Core contract JSON is missing set_place statement' }
  if (-not $CoreContractJson.Contains('"id": "body_grammar"')) { throw 'Core contract JSON is missing body_grammar gate' }
  if (-not $CoreContractJson.Contains('"status": "partial_v0"')) { throw 'Core contract JSON is missing partial_v0 body grammar status' }
  if (-not $CoreContractJson.Contains('"id": "core_preview"')) { throw 'Core contract JSON is missing core_preview gate' }
  if (-not $CoreContractJson.Contains('"status": "preview_v0"')) { throw 'Core contract JSON is missing preview_v0 status' }
  if (-not $CoreContractJson.Contains('"id": "core_lowering"')) { throw 'Core contract JSON is missing core_lowering gate' }
  if (-not $CoreContractJson.Contains('"status": "unverified_core_artifact_v0"')) { throw 'Core contract JSON is missing unverified core artifact gate status' }
  if (-not $CoreContractJson.Contains('"id": "type_check"')) { throw 'Core contract JSON is missing type_check gate' }
  if (-not $CoreContractJson.Contains('"status": "declaration_and_trivial_return_check_available"')) { throw 'Core contract JSON is missing narrow type-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "full_type_check"')) { throw 'Core contract JSON is missing full_type_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_body_type_gate_available_v0"')) { throw 'Core contract JSON is missing full type-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "effect_check"')) { throw 'Core contract JSON is missing effect_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_effect_gate_available_v0"')) { throw 'Core contract JSON is missing effect-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "ownership_check"')) { throw 'Core contract JSON is missing ownership_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_ownership_gate_available_v0"')) { throw 'Core contract JSON is missing ownership-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "allocation_resource_check"')) { throw 'Core contract JSON is missing allocation_resource_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_resource_gate_available_v0"')) { throw 'Core contract JSON is missing resource-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "profile_check"')) { throw 'Core contract JSON is missing profile_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_profile_gate_available_v0"')) { throw 'Core contract JSON is missing profile-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "core_verify"')) { throw 'Core contract JSON is missing core_verify gate' }
  if (-not $CoreContractJson.Contains('"status": "verified_non_executing_core_artifact_v0"')) { throw 'Core contract JSON is missing core verify gate status' }
  if (-not $CoreContractJson.Contains('"no executable semantics"')) { throw 'Core contract JSON must keep V0 non-execution claim' }

  $IrContractJson = Read-NativeOutput 'IR contract JSON' $Hum @('ir-contract', '--format', 'json')
  Assert-Json 'IR contract JSON' $IrContractJson
  if (-not $IrContractJson.Contains('"schema": "hum.ir_contract.v0"')) { throw 'IR contract JSON is missing hum.ir_contract.v0 schema' }
  if (-not $IrContractJson.Contains('"semantic_owner": "hum_ir"')) { throw 'IR contract JSON is missing semantic owner' }
  if (-not $IrContractJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'IR contract JSON is missing Core Hum contract schema' }
  if (-not $IrContractJson.Contains('"id": "core_hum"')) { throw 'IR contract JSON is missing core_hum layer' }
  if (-not $IrContractJson.Contains('"id": "hum_ir"')) { throw 'IR contract JSON is missing hum_ir layer' }
  if (-not $IrContractJson.Contains('"typed_failure_edges"')) { throw 'IR contract JSON is missing typed failure facts' }
  if (-not $IrContractJson.Contains('"core_verify"')) { throw 'IR contract JSON is missing core_verify pass' }
  if (-not $IrContractJson.Contains('"full_type_check"')) { throw 'IR contract JSON is missing full_type_check pass' }
  if (-not $IrContractJson.Contains('"effect_check"')) { throw 'IR contract JSON is missing effect_check pass' }
  if (-not $IrContractJson.Contains('"ir_verify"')) { throw 'IR contract JSON is missing ir_verify pass' }
  if (-not $IrContractJson.Contains('"no IR emission for source files"')) { throw 'IR contract JSON must keep V0 non-emission claim' }

  $BackendContractJson = Read-NativeOutput 'backend contract JSON' $Hum @('backend-contract', '--format', 'json')
  Assert-Json 'backend contract JSON' $BackendContractJson
  if (-not $BackendContractJson.Contains('"schema": "hum.backend_contract.v0"')) { throw 'backend contract JSON is missing hum.backend_contract.v0 schema' }
  if (-not $BackendContractJson.Contains('"semantic_owner": "hum_ir"')) { throw 'backend contract JSON is missing semantic owner' }
  if (-not $BackendContractJson.Contains('"semantic_owner_schema": "hum.ir_contract.v0"')) { throw 'backend contract JSON is missing Hum IR owner schema' }
  if (-not $BackendContractJson.Contains('"id": "interpreter"')) { throw 'backend contract JSON is missing interpreter stage' }
  if (-not $BackendContractJson.Contains('"id": "cranelift"')) { throw 'backend contract JSON is missing cranelift stage' }
  if (-not $BackendContractJson.Contains('"id": "llvm"')) { throw 'backend contract JSON is missing llvm stage' }
  if (-not $BackendContractJson.Contains('"no code execution"')) { throw 'backend contract JSON must keep V0 non-execution claim' }

  $RuntimeProfilesJson = Read-NativeOutput 'runtime profiles JSON' $Hum @('profiles', '--format', 'json')
  Assert-Json 'runtime profiles JSON' $RuntimeProfilesJson
  if (-not $RuntimeProfilesJson.Contains('"schema": "hum.runtime_profiles.v0"')) { throw 'runtime profiles JSON is missing hum.runtime_profiles.v0 schema' }
  if (-not $RuntimeProfilesJson.Contains('"profile_schema": "hum.runtime_profile.v0"')) { throw 'runtime profiles JSON is missing hum.runtime_profile.v0 schema link' }
  if (-not $RuntimeProfilesJson.Contains('"mode": "contract_only_no_profile_enforcement"')) { throw 'runtime profiles JSON must stay contract-only in V0' }
  if (-not $RuntimeProfilesJson.Contains('"id": "agent_tool_sandbox"')) { throw 'runtime profiles JSON is missing agent_tool_sandbox profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "footprint_constrained"')) { throw 'runtime profiles JSON is missing footprint_constrained profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "hard_realtime"')) { throw 'runtime profiles JSON is missing hard_realtime profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "windows_service"')) { throw 'runtime profiles JSON is missing windows_service profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "driver_candidate"')) { throw 'runtime profiles JSON is missing driver_candidate profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "engine_hot_path"')) { throw 'runtime profiles JSON is missing engine_hot_path profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "medical_class_c"')) { throw 'runtime profiles JSON is missing medical_class_c profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "automotive_asil_d"')) { throw 'runtime profiles JSON is missing automotive_asil_d profile' }
  if (-not $RuntimeProfilesJson.Contains('"denied_capability_families"')) { throw 'runtime profiles JSON is missing denied capability policy' }
  if (-not $RuntimeProfilesJson.Contains('"os.network"')) { throw 'runtime profiles JSON is missing network capability policy' }
  if (-not $RuntimeProfilesJson.Contains('"no profile syntax enforcement"')) { throw 'runtime profiles JSON must keep V0 enforcement non-claim' }
  if (-not $RuntimeProfilesJson.Contains('"no certification claim"')) { throw 'runtime profiles JSON must not claim certification' }

  $StateModelJson = Read-NativeOutput 'state model JSON' $Hum @('state-model', '--format', 'json')
  Assert-Json 'state model JSON' $StateModelJson
  if (-not $StateModelJson.Contains('"schema": "hum.state_model.v0"')) { throw 'state model JSON is missing hum.state_model.v0 schema' }
  if (-not $StateModelJson.Contains('"permission_schema": "hum.state_permission.v0"')) { throw 'state model JSON is missing hum.state_permission.v0 schema link' }
  if (-not $StateModelJson.Contains('"mode": "contract_only_partial_declared_mutation_check"')) { throw 'state model JSON must keep V0 contract-only mode' }
  if (-not $StateModelJson.Contains('"id": "immutable_value"')) { throw 'state model JSON is missing immutable_value kind' }
  if (-not $StateModelJson.Contains('"id": "mutable_local"')) { throw 'state model JSON is missing mutable_local kind' }
  if (-not $StateModelJson.Contains('"id": "linear_resource"')) { throw 'state model JSON is missing linear_resource kind' }
  if (-not $StateModelJson.Contains('"id": "borrow_check"')) { throw 'state model JSON is missing borrow_check gate' }
  if (-not $StateModelJson.Contains('"no borrow checker implementation"')) { throw 'state model JSON must not claim borrow checking' }
  if (-not $StateModelJson.Contains('"no concurrency or memory-order model"')) { throw 'state model JSON must not claim concurrency checking' }

  $LspCapabilitiesJson = Read-NativeOutput 'LSP capabilities JSON' $Hum @('lsp', '--capabilities', '--format', 'json')
  Assert-Json 'LSP capabilities JSON' $LspCapabilitiesJson
  if (-not $LspCapabilitiesJson.Contains('"schema": "hum.lsp_capabilities.v0"')) { throw 'LSP capabilities JSON is missing hum.lsp_capabilities.v0 schema' }
  if (-not $LspCapabilitiesJson.Contains('"json_rpc_server": false')) { throw 'LSP capabilities JSON should say server mode is not implemented' }
  if (-not $LspCapabilitiesJson.Contains('"textDocument/publishDiagnostics"')) { throw 'LSP capabilities JSON is missing diagnostics method' }
  if (-not $LspCapabilitiesJson.Contains('"textDocument/documentSymbol"')) { throw 'LSP capabilities JSON is missing documentSymbol method' }

  $DoctorJson = Read-NativeOutput 'doctor JSON' $Hum @('doctor', '--format', 'json')
  Assert-Json 'doctor JSON' $DoctorJson
  if (-not $DoctorJson.Contains('"schema": "hum.doctor.v0"')) { throw 'doctor JSON is missing hum.doctor.v0 schema' }
  if (-not $DoctorJson.Contains('"summary"')) { throw 'doctor JSON is missing summary' }
  if (-not $DoctorJson.Contains('"status": "pass"')) { throw 'doctor JSON should pass from the repo root' }
  if (-not $DoctorJson.Contains('"text_hygiene_policy"')) { throw 'doctor JSON is missing text_hygiene_policy check' }
  if (-not $DoctorJson.Contains('"public_readiness_policy"')) { throw 'doctor JSON is missing public_readiness_policy check' }
  if (-not $DoctorJson.Contains('"clean_checkout_smoke"')) { throw 'doctor JSON is missing clean_checkout_smoke check' }
  if (-not $DoctorJson.Contains('"tag_readiness"')) { throw 'doctor JSON is missing tag_readiness check' }

  $TargetFactsJson = Read-NativeOutput 'target facts JSON' $Hum @('target-facts', '--format', 'json')
  Assert-Json 'target facts JSON' $TargetFactsJson
  if (-not $TargetFactsJson.Contains('"schema": "hum.target_facts.v0"')) { throw 'target facts JSON is missing hum.target_facts.v0 schema' }
  if (-not $TargetFactsJson.Contains('"record_schema": "hum.target_fact_record.v0"')) { throw 'target facts JSON is missing target fact record schema' }
  if (-not $TargetFactsJson.Contains('"mode": "contract_only_no_host_probe"')) { throw 'target facts JSON must stay non-probing in V0' }
  if (-not $TargetFactsJson.Contains('"default_policy": "unknown_fails_closed"')) { throw 'target facts JSON is missing fail-closed policy' }
  if (-not $TargetFactsJson.Contains('"field_catalog"')) { throw 'target facts JSON is missing field catalog' }
  if (-not $TargetFactsJson.Contains('"capability_families"')) { throw 'target facts JSON is missing capability families' }
  if (-not $TargetFactsJson.Contains('"family": "os.clock"')) { throw 'target facts JSON is missing clock capability family' }
  if (-not $TargetFactsJson.Contains('"family": "os.stdio"')) { throw 'target facts JSON is missing reserved stdio capability family' }
  if ([regex]::Matches($TargetFactsJson, '"family": "os\.stdio"').Count -ne 5 -or [regex]::Matches($TargetFactsJson, '"availability": "reserved_mapping_only"').Count -ne 4) { throw 'target facts JSON must reserve os.stdio in the catalog and all four fixtures without availability claims' }
  if (-not $TargetFactsJson.Contains('"availability": "host_import_profile_gated"')) { throw 'target facts JSON is missing WASI clock/random availability' }
  if (-not $TargetFactsJson.Contains('"id": "windows-x86_64-msvc"')) { throw 'target facts JSON is missing Windows fixture' }
  if (-not $TargetFactsJson.Contains('"id": "wasm32-wasi-preview1"')) { throw 'target facts JSON is missing WASI fixture' }
  if (-not $TargetFactsJson.Contains('"no host capability probing"')) { throw 'target facts JSON must reject host probing claims' }

  $TargetFactFixtureDir = Join-Path (Join-Path $RepoRoot 'fixtures') 'target_facts'
  $TargetFactFixtures = @(Get-ChildItem -LiteralPath $TargetFactFixtureDir -Filter '*.json' -File)
  if ($TargetFactFixtures.Count -lt 4) { throw 'target fact fixture directory must contain at least four JSON fixtures' }
  foreach ($Fixture in $TargetFactFixtures) {
    $FixtureText = [System.IO.File]::ReadAllText($Fixture.FullName)
    $FixtureJson = $FixtureText | ConvertFrom-Json
    if ($FixtureJson.schema -ne 'hum.target_fact_record.v0') { throw "target fact fixture $($Fixture.Name) has wrong schema" }
    if ($FixtureJson.status -ne 'fixture') { throw "target fact fixture $($Fixture.Name) must have fixture status" }
    if ($FixtureJson.absence_policy -ne 'unknown_or_absent_capabilities_fail_closed') { throw "target fact fixture $($Fixture.Name) must fail closed" }
    if ($null -eq $FixtureJson.facts.triple) { throw "target fact fixture $($Fixture.Name) is missing facts.triple" }
    if ($null -eq $FixtureJson.capabilities -or @($FixtureJson.capabilities).Count -eq 0) { throw "target fact fixture $($Fixture.Name) is missing capabilities" }
    $Stdio = @($FixtureJson.capabilities | Where-Object { $_.family -eq 'os.stdio' })
    if ($Stdio.Count -ne 1 -or $Stdio[0].availability -ne 'reserved_mapping_only') { throw "target fact fixture $($Fixture.Name) must reserve os.stdio without claiming availability" }
    if ($null -eq $FixtureJson.non_claims -or @($FixtureJson.non_claims).Count -eq 0) { throw "target fact fixture $($Fixture.Name) is missing non_claims" }
  }

  Invoke-Native 'hum check examples' $Hum @('check', 'examples')

  $RunAdd = Read-NativeOutput 'run core add' $Hum @('run', 'examples/core/add.hum', '--entry', 'add', '--args', '2', '3')
  if ($RunAdd.Trim() -ne '5') { throw "hum run add expected 5, got `$RunAdd" }

  $RunDivideZero = Read-NativeOutputWithExit 'run core divide needs violation' $Hum @('run', 'examples/core/divide.hum', '--entry', 'divide', '--args', '10', '0')
  if ($RunDivideZero.ExitCode -ne 1) { throw "hum run divide zero expected exit 1, got $($RunDivideZero.ExitCode)" }
  if (-not $RunDivideZero.Output.Contains('caller did not satisfy needs: b != 0')) { throw "hum run divide zero expected caller needs blame, got $($RunDivideZero.Output)" }
  if (-not $RunDivideZero.Output.Contains('examples/core/divide.hum:12:')) { throw "hum run divide zero expected source span for needs predicate, got $($RunDivideZero.Output)" }

  $RunCountCompleted = Read-NativeOutput 'run core count_completed' $Hum @('run', 'examples/core/count_completed.hum', '--entry', 'count_completed', '--args', '[{done:true},{done:false},{done:true}]')
  if ($RunCountCompleted.Trim() -ne '2') { throw "hum run count_completed expected 2, got `$RunCountCompleted" }

  $RunWrongAdd = Read-NativeOutputWithExit 'run wrong add contract violation' $Hum @('run', 'fixtures/run/wrong_add_contract.hum', '--entry', 'add', '--args', '2', '3')
  if ($RunWrongAdd.ExitCode -ne 1) { throw "hum run wrong add expected exit 1, got $($RunWrongAdd.ExitCode)" }
  if (-not $RunWrongAdd.Output.Contains('task `add` did not satisfy ensures: result == a + b')) { throw "hum run wrong add expected task ensures blame, got $($RunWrongAdd.Output)" }

  $RunWordCount = Read-NativeOutput 'run probe word_count' $Hum @('run', 'examples/probes/word_count.hum', '--entry', 'count_hum_literal')
  if ($RunWordCount.Trim() -ne '2') { throw "hum run word_count expected 2, got `$RunWordCount" }

  $RunTaskListFlow = Read-NativeOutput 'run probe task_list_flow' $Hum @('run', 'examples/probes/task_list_flow.hum', '--entry', 'task_list_demo')
  if ($RunTaskListFlow.Trim() -ne '1') { throw "hum run task_list_flow expected 1, got `$RunTaskListFlow" }

  $RunTransactionOnce = Read-NativeOutput 'run probe transaction_once' $Hum @('run', 'examples/probes/transaction_once.hum', '--entry', 'transfer', '--args', '10')
  if ($RunTransactionOnce.Trim() -ne 'ok') { throw "hum run transaction_once expected ok, got `$RunTransactionOnce" }

  $RunSessionJBorrow = Read-NativeOutput 'run Session J borrow fixture' $Hum @('run', 'fixtures/ownership_check/session_j_borrow_pass.hum', '--entry', 'echo', '--args', '7')
  if ($RunSessionJBorrow.Trim() -ne '7') { throw "Session J borrow run expected 7, got `$RunSessionJBorrow" }

  $RunSessionJChange = Read-NativeOutput 'run Session J change fixture' $Hum @('run', 'fixtures/ownership_check/session_j_change_pass.hum', '--entry', 'increment', '--args', '7')
  if ($RunSessionJChange.Trim() -ne '8') { throw "Session J change run expected 8, got `$RunSessionJChange" }

  $RunSessionJConsume = Read-NativeOutput 'run Session J consume fixture' $Hum @('run', 'fixtures/ownership_check/session_j_consume_pass.hum', '--entry', 'consume_demo')
  if ($RunSessionJConsume.Trim() -ne '7') { throw "Session J consume run expected 7, got `$RunSessionJConsume" }

  $RunSessionJUseAfterMove = Read-NativeOutputWithExit 'run Session J use-after-move misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_j_use_after_move_fail.hum', '--entry', 'use_after_move')
  if ($RunSessionJUseAfterMove.ExitCode -ne 2) { throw "Session J use-after-move run expected exit 2, got $($RunSessionJUseAfterMove.ExitCode)" }
  if (-not $RunSessionJUseAfterMove.Output.Contains('H0801')) { throw "Session J use-after-move run expected H0801, got $($RunSessionJUseAfterMove.Output)" }
  $RunSessionJUseAfterMoveExpected = @(
    'fixtures/ownership_check/session_j_use_after_move_fail.hum:18:5: error[H0801]: value `value` was used after it was moved',
    '  help: Fix task `use_after_move`: `value` moved at fixtures/ownership_check/session_j_use_after_move_fail.hum:17:5; use it before that move or create a fresh owned value.',
    'runtime trap: H0801 use after move'
  ) -join "`n"
  if ($RunSessionJUseAfterMove.Output.Replace(([string][char]13 + [string][char]10), [string][char]10).TrimEnd() -ne $RunSessionJUseAfterMoveExpected) { throw 'Session J H0801 runtime bytes changed' }
  if (-not $RunSessionJUseAfterMove.Output.Contains('help:')) { throw "Session J use-after-move run expected blame help, got $($RunSessionJUseAfterMove.Output)" }

  $RunSessionJBorrowWrite = Read-NativeOutputWithExit 'run Session J borrowed-write misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_j_borrow_write_fail.hum', '--entry', 'write_borrow', '--args', '7')
  if ($RunSessionJBorrowWrite.ExitCode -ne 2) { throw "Session J borrowed-write run expected exit 2, got $($RunSessionJBorrowWrite.ExitCode)" }
  if (-not $RunSessionJBorrowWrite.Output.Contains('H0802')) { throw "Session J borrowed-write run expected H0802, got $($RunSessionJBorrowWrite.Output)" }
  if (-not $RunSessionJBorrowWrite.Output.Contains('help:')) { throw "Session J borrowed-write run expected blame help, got $($RunSessionJBorrowWrite.Output)" }

  $RunSessionJDoubleConsume = Read-NativeOutputWithExit 'run Session J double-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_j_double_consume_fail.hum', '--entry', 'double_consume')
  if ($RunSessionJDoubleConsume.ExitCode -ne 2) { throw "Session J double-consume run expected exit 2, got $($RunSessionJDoubleConsume.ExitCode)" }
  if (-not $RunSessionJDoubleConsume.Output.Contains('H0801')) { throw "Session J double-consume run expected H0801, got $($RunSessionJDoubleConsume.Output)" }
  if (-not $RunSessionJDoubleConsume.Output.Contains('help:')) { throw "Session J double-consume run expected blame help, got $($RunSessionJDoubleConsume.Output)" }

  $RunSessionKMissingConsume = Read-NativeOutputWithExit 'run Session K missing-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_k_missing_consume_fail.hum', '--entry', 'transfer_missing', '--args', '0')
  if ($RunSessionKMissingConsume.ExitCode -ne 2) { throw "Session K missing-consume run expected exit 2, got $($RunSessionKMissingConsume.ExitCode)" }
  if (-not $RunSessionKMissingConsume.Output.Contains('H0803')) { throw "Session K missing-consume run expected H0803, got $($RunSessionKMissingConsume.Output)" }
  if (-not $RunSessionKMissingConsume.Output.Contains('help:')) { throw "Session K missing-consume run expected blame help, got $($RunSessionKMissingConsume.Output)" }

  $RunSessionKDoubleConsume = Read-NativeOutputWithExit 'run Session K double-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_k_double_consume_fail.hum', '--entry', 'transfer_double_consume')
  if ($RunSessionKDoubleConsume.ExitCode -ne 2) { throw "Session K double-consume run expected exit 2, got $($RunSessionKDoubleConsume.ExitCode)" }
  if (-not $RunSessionKDoubleConsume.Output.Contains('H0804')) { throw "Session K double-consume run expected H0804, got $($RunSessionKDoubleConsume.Output)" }
  if (-not $RunSessionKDoubleConsume.Output.Contains('commit')) { throw "Session K double-consume run expected prior commit blame, got $($RunSessionKDoubleConsume.Output)" }

  $RunSessionKBranchConsume = Read-NativeOutputWithExit 'run Session K branch-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_k_branch_consume_fail.hum', '--entry', 'branch_consume', '--args', '1')
  if ($RunSessionKBranchConsume.ExitCode -ne 2) { throw "Session K branch-consume run expected exit 2, got $($RunSessionKBranchConsume.ExitCode)" }
  if (-not $RunSessionKBranchConsume.Output.Contains('H0803')) { throw "Session K branch-consume run expected H0803, got $($RunSessionKBranchConsume.Output)" }
  if (-not $RunSessionKBranchConsume.Output.Contains('help:')) { throw "Session K branch-consume run expected blame help, got $($RunSessionKBranchConsume.Output)" }

  $RunSessionLParameterView = Read-NativeOutput 'run Session L parameter returned-view fixture' $Hum @('run', 'fixtures/ownership_check/session_l_return_parameter_view_pass.hum', '--entry', 'echo_view', '--args', 'hello hum')
  if ($RunSessionLParameterView.Trim() -ne 'hello hum') { throw "Session L parameter returned-view run expected hello hum, got `$RunSessionLParameterView" }

  $RunSessionLLocalView = Read-NativeOutputWithExit 'run Session L local returned-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_l_return_view_local_fail.hum', '--entry', 'local_view')
  if ($RunSessionLLocalView.ExitCode -ne 2) { throw "Session L local returned-view run expected exit 2, got $($RunSessionLLocalView.ExitCode)" }
  if (-not $RunSessionLLocalView.Output.Contains('H0805')) { throw "Session L local returned-view run expected H0805, got $($RunSessionLLocalView.Output)" }
  if (-not $RunSessionLLocalView.Output.Contains('help:')) { throw "Session L local returned-view run expected blame help, got $($RunSessionLLocalView.Output)" }

  $RunSessionNFirstWord = Read-NativeOutput 'run Session N first_word derived returned-view fixture' $Hum @('run', 'examples/probes/first_word.hum', '--entry', 'first_word', '--args', 'hum language')
  if ($RunSessionNFirstWord.Trim() -ne 'hum') { throw "Session N first_word run expected hum, got `$RunSessionNFirstWord" }

  $RunSessionNLocalSlice = Read-NativeOutputWithExit 'run Session N local sub-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_n_return_view_local_slice_fail.hum', '--entry', 'local_first_word')
  if ($RunSessionNLocalSlice.ExitCode -ne 2) { throw "Session N local sub-view run expected exit 2, got $($RunSessionNLocalSlice.ExitCode)" }
  if (-not $RunSessionNLocalSlice.Output.Contains('H0805')) { throw "Session N local sub-view run expected H0805, got $($RunSessionNLocalSlice.Output)" }
  if (-not $RunSessionNLocalSlice.Output.Contains('help:')) { throw "Session N local sub-view run expected blame help, got $($RunSessionNLocalSlice.Output)" }

  $RunSessionNLostProvenance = Read-NativeOutputWithExit 'run Session N lost-provenance misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_n_return_view_lost_provenance_fail.hum', '--entry', 'lost_first_word', '--args', 'hum language')
  if ($RunSessionNLostProvenance.ExitCode -ne 2) { throw "Session N lost-provenance run expected exit 2, got $($RunSessionNLostProvenance.ExitCode)" }
  if (-not $RunSessionNLostProvenance.Output.Contains('H0805')) { throw "Session N lost-provenance run expected H0805, got $($RunSessionNLostProvenance.Output)" }
  if (-not $RunSessionNLostProvenance.Output.Contains('non-closed derivation chains remain rejected')) { throw "Session N lost-provenance run expected non-closed help, got $($RunSessionNLostProvenance.Output)" }
  $RunSessionOSwap = Read-NativeOutput 'run Session O swap_xy field-place fixture' $Hum @('run', 'examples/probes/field_places.hum', '--entry', 'swap_xy', '--args', '{x:1,y:2}')
  if ($RunSessionOSwap.Trim() -ne '{x: 2, y: 1}') { throw "Session O swap_xy run expected {x: 2, y: 1}, got `$RunSessionOSwap" }

  $RunSessionOComplete = Read-NativeOutput 'run Session O complete_item field-place fixture' $Hum @('run', 'fixtures/run/session_o_complete_item_field_place.hum', '--entry', 'complete_item_demo')
  if ($RunSessionOComplete.Trim() -ne '{done: true, title: hum}') { throw "Session O complete_item run expected done true with preserved title, got $($RunSessionOComplete)" }

  $RunSessionOBorrowField = Read-NativeOutputWithExit 'run Session O borrowed field-write misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_o_field_write_borrow_fail.hum', '--entry', 'write_borrowed_field', '--args', '{x:1,y:2}')
  if ($RunSessionOBorrowField.ExitCode -ne 2) { throw "Session O borrowed field-write run expected exit 2, got $($RunSessionOBorrowField.ExitCode)" }
  if (-not $RunSessionOBorrowField.Output.Contains('H0802')) { throw "Session O borrowed field-write run expected H0802, got $($RunSessionOBorrowField.Output)" }
  if (-not $RunSessionOBorrowField.Output.Contains('point.x')) { throw "Session O borrowed field-write run expected point.x blame, got $($RunSessionOBorrowField.Output)" }
  if (-not $RunSessionOBorrowField.Output.Contains('help:')) { throw "Session O borrowed field-write run expected blame help, got $($RunSessionOBorrowField.Output)" }

  $RunSessionPBuilder = Read-NativeOutput 'run Session P builder fixture' $Hum @('run', 'examples/probes/list_builder.hum', '--entry', 'builder_demo')
  if ($RunSessionPBuilder.Trim() -ne '[parse, check, run]') { throw "Session P builder run expected [parse, check, run], got `$RunSessionPBuilder" }

  $RunSessionPBuilderContract = Read-NativeOutput 'run Session P builder contract fixture' $Hum @('run', 'examples/probes/list_builder.hum', '--entry', 'builder_contract_demo')
  if ($RunSessionPBuilderContract.Trim() -ne '3') { throw "Session P builder contract run expected 3, got `$RunSessionPBuilderContract" }

  $RunSessionPIterConflict = Read-NativeOutputWithExit 'run Session P iteration-conflict misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_p_append_during_iteration_fail.hum', '--entry', 'append_during_iteration')
  if ($RunSessionPIterConflict.ExitCode -ne 2) { throw "Session P iteration-conflict run expected exit 2, got $($RunSessionPIterConflict.ExitCode)" }
  if (-not $RunSessionPIterConflict.Output.Contains('H0806')) { throw "Session P iteration-conflict run expected H0806, got $($RunSessionPIterConflict.Output)" }
  if (-not $RunSessionPIterConflict.Output.Contains('list_append')) { throw "Session P iteration-conflict run expected list_append blame, got $($RunSessionPIterConflict.Output)" }
  if (-not $RunSessionPIterConflict.Output.Contains('for each')) { throw "Session P iteration-conflict run expected loop blame, got $($RunSessionPIterConflict.Output)" }
  if (-not $RunSessionPIterConflict.Output.Contains('help:')) { throw "Session P iteration-conflict run expected help, got $($RunSessionPIterConflict.Output)" }

  $RunSessionPAddAfterFinish = Read-NativeOutputWithExit 'run Session P add-after-finish misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_p_add_after_finish_fail.hum', '--entry', 'add_after_finish')
  if ($RunSessionPAddAfterFinish.ExitCode -ne 2) { throw "Session P add-after-finish run expected exit 2, got $($RunSessionPAddAfterFinish.ExitCode)" }
  if (-not $RunSessionPAddAfterFinish.Output.Contains('H0801')) { throw "Session P add-after-finish run expected H0801, got $($RunSessionPAddAfterFinish.Output)" }
  if (-not $RunSessionPAddAfterFinish.Output.Contains('help:')) { throw "Session P add-after-finish run expected help, got $($RunSessionPAddAfterFinish.Output)" }

  $RunSessionRDistinctFieldView = Read-NativeOutput 'run Session R distinct-field view fixture' $Hum @('run', 'examples/probes/field_views.hum', '--entry', 'distinct_field_view_survives', '--args', '{left:false,right:false}')
  if ($RunSessionRDistinctFieldView.Trim() -ne 'false') { throw "Session R distinct-field view run expected false, got $RunSessionRDistinctFieldView" }

  $RunSessionRCopyVsView = Read-NativeOutput 'run Session R copy-vs-view fixture' $Hum @('run', 'examples/probes/field_views.hum', '--entry', 'copy_survives_field_write', '--args', '{left:false,right:false}')
  if ($RunSessionRCopyVsView.Trim() -ne 'false') { throw "Session R copy-vs-view run expected false, got $RunSessionRCopyVsView" }

  $RunSessionRStalePoint = Read-NativeOutputWithExit 'run Session R stale point field-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_r_stale_point_field_view_fail.hum', '--entry', 'stale_point_field_view', '--args', '{x:1,y:2}')
  if ($RunSessionRStalePoint.ExitCode -ne 2) { throw "Session R stale point field-view run expected exit 2, got $($RunSessionRStalePoint.ExitCode)" }
  if (-not $RunSessionRStalePoint.Output.Contains('H0807')) { throw "Session R stale point field-view run expected H0807, got $($RunSessionRStalePoint.Output)" }
  if (-not $RunSessionRStalePoint.Output.Contains('x_view')) { throw "Session R stale point field-view run expected x_view blame, got $($RunSessionRStalePoint.Output)" }
  if (-not $RunSessionRStalePoint.Output.Contains('point.x')) { throw "Session R stale point field-view run expected point.x write blame, got $($RunSessionRStalePoint.Output)" }
  if (-not $RunSessionRStalePoint.Output.Contains('re-borrow after the write')) { throw "Session R stale point field-view run expected repair help, got $($RunSessionRStalePoint.Output)" }

  $RunSessionRStaleItem = Read-NativeOutputWithExit 'run Session R stale item field-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_r_stale_item_field_view_fail.hum', '--entry', 'stale_item_done_view', '--args', '{done:false,pinned:false}')
  if ($RunSessionRStaleItem.ExitCode -ne 2) { throw "Session R stale item field-view run expected exit 2, got $($RunSessionRStaleItem.ExitCode)" }
  if (-not $RunSessionRStaleItem.Output.Contains('H0807')) { throw "Session R stale item field-view run expected H0807, got $($RunSessionRStaleItem.Output)" }
  if (-not $RunSessionRStaleItem.Output.Contains('done_view')) { throw "Session R stale item field-view run expected done_view blame, got $($RunSessionRStaleItem.Output)" }
  if (-not $RunSessionRStaleItem.Output.Contains('item.done')) { throw "Session R stale item field-view run expected item.done write blame, got $($RunSessionRStaleItem.Output)" }

  $RunSessionSElementView = Read-NativeOutput 'run Session S element-view fixture' $Hum @('run', 'examples/probes/element_views.hum', '--entry', 'element_view_before_growth')
  if ($RunSessionSElementView.Trim() -ne 'parse') { throw "Session S element-view run expected parse, got $RunSessionSElementView" }

  $RunSessionSCopyVsView = Read-NativeOutput 'run Session S element copy-vs-view fixture' $Hum @('run', 'examples/probes/element_views.hum', '--entry', 'copied_element_survives_growth')
  if ($RunSessionSCopyVsView.Trim() -ne 'parse') { throw "Session S copied element run expected parse, got $RunSessionSCopyVsView" }

  $RunSessionSStaleElement = Read-NativeOutputWithExit 'run Session S stale element-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_s_stale_element_view_fail.hum', '--entry', 'stale_element_view_after_append')
  if ($RunSessionSStaleElement.ExitCode -ne 2) { throw "Session S stale element-view run expected exit 2, got $($RunSessionSStaleElement.ExitCode)" }
  if (-not $RunSessionSStaleElement.Output.Contains('H0807')) { throw "Session S stale element-view run expected H0807, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('first_view')) { throw "Session S stale element-view run expected first_view blame, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('items[0]')) { throw "Session S stale element-view run expected items[0] source, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('list_append grew items')) { throw "Session S stale element-view run expected append site, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('re-borrow after the append')) { throw "Session S stale element-view run expected repair help, got $($RunSessionSStaleElement.Output)" }

  $RunSessionSOverlap = Read-NativeOutputWithExit 'run Session S H0806/H0807 overlap fixture' $Hum @('run', 'fixtures/ownership_check/session_s_append_iteration_view_overlap_fail.hum', '--entry', 'append_during_iteration_with_element_view')
  if ($RunSessionSOverlap.ExitCode -ne 2) { throw "Session S overlap run expected exit 2, got $($RunSessionSOverlap.ExitCode)" }
  if (-not $RunSessionSOverlap.Output.Contains('H0806')) { throw "Session S overlap run expected H0806, got $($RunSessionSOverlap.Output)" }
  if ($RunSessionSOverlap.Output.Contains('H0807')) { throw "Session S overlap run must not also report H0807, got $($RunSessionSOverlap.Output)" }
  if (-not $RunSessionSOverlap.Output.Contains('list_append')) { throw "Session S overlap run expected list_append blame, got $($RunSessionSOverlap.Output)" }
  if (-not $RunSessionSOverlap.Output.Contains('for each')) { throw "Session S overlap run expected loop blame, got $($RunSessionSOverlap.Output)" }

  $RunSessionTWrongSwap = Read-NativeOutputWithExit 'run Session T wrong-swap old() sabotage fixture' $Hum @('run', 'fixtures/run/session_t_wrong_swap_contract.hum', '--entry', 'wrong_swap_demo')
  if ($RunSessionTWrongSwap.ExitCode -ne 1) { throw "Session T wrong-swap run expected exit 1, got $($RunSessionTWrongSwap.ExitCode)" }
  if (-not $RunSessionTWrongSwap.Output.Contains('H0703')) { throw "Session T wrong-swap run expected H0703, got $($RunSessionTWrongSwap.Output)" }
  if (-not $RunSessionTWrongSwap.Output.Contains('result.x == old(point.y)')) { throw "Session T wrong-swap run expected old() contract text, got $($RunSessionTWrongSwap.Output)" }

  $RunSessionTOldInNeeds = Read-NativeOutputWithExit 'run Session AF old-in-needs semantic boundary fixture' $Hum @('run', 'fixtures/run/session_t_old_in_needs_prose.hum', '--entry', 'old_in_needs_demo')
  if ($RunSessionTOldInNeeds.ExitCode -ne 2) { throw "Session AF old-in-needs run expected exit 2, got $($RunSessionTOldInNeeds.ExitCode)" }
  if (-not $RunSessionTOldInNeeds.Output.Contains('H0704')) { throw "Session AF old-in-needs run expected H0704, got $($RunSessionTOldInNeeds.Output)" }

  $RunSessionTBuilderLen = Read-NativeOutputWithExit 'run Session T builder list_len contract fixture' $Hum @('run', 'examples/probes/list_builder.hum', '--entry', 'builder_demo')
  if ($RunSessionTBuilderLen.ExitCode -ne 0) { throw "Session T builder list_len run expected exit 0, got $($RunSessionTBuilderLen.ExitCode)" }
  if ($RunSessionTBuilderLen.Output.Contains('list_len(result) == 3') -and $RunSessionTBuilderLen.Output.Contains('H0701')) {
    if ($RunSessionTBuilderLen.Output -match 'H0701[^\r\n]*list_len') { throw "Session T list_len contract must be checked, not prose: $($RunSessionTBuilderLen.Output)" }
  }

  foreach ($Positive in @(
    @{ File = 'examples/probes/word_count.hum'; Entry = 'count_hum_literal'; Expected = '2' },
    @{ File = 'examples/probes/list_builder.hum'; Entry = 'builder_demo'; Expected = '[parse, check, run]' },
    @{ File = 'examples/probes/element_views.hum'; Entry = 'element_view_before_growth'; Expected = 'parse' }
  )) {
    $RunPredicateV2 = Read-NativeOutputWithExit "run Session AF positive $($Positive.Entry)" $Hum @('run', $Positive.File, '--entry', $Positive.Entry)
    if ($RunPredicateV2.ExitCode -ne 0 -or $RunPredicateV2.Output.Trim() -ne $Positive.Expected) { throw "Session AF positive $($Positive.Entry) expected $($Positive.Expected), got $($RunPredicateV2.Output)" }
    if ($RunPredicateV2.Output.Contains('H0701') -or $RunPredicateV2.Output.Contains('H0704')) { throw "Session AF positive $($Positive.Entry) must be executable without H0701/H0704" }
  }

  foreach ($Wrong in @(
    @{ File = 'fixtures/run/session_af_predicate_v2_wrong_count_fail.hum'; Entry = 'wrong_count' },
    @{ File = 'fixtures/run/session_af_predicate_v2_wrong_content_fail.hum'; Entry = 'wrong_content' },
    @{ File = 'fixtures/run/session_af_predicate_v2_wrong_text_fail.hum'; Entry = 'wrong_text' },
    @{ File = 'fixtures/run/session_af_predicate_v2_valid_neighbor_fail.hum'; Entry = 'valid_neighbor' }
  )) {
    $RunPredicateV2Wrong = Read-NativeOutputWithExit "run Session AF wrong implementation $($Wrong.Entry)" $Hum @('run', $Wrong.File, '--entry', $Wrong.Entry)
    if ($RunPredicateV2Wrong.ExitCode -ne 1 -or -not $RunPredicateV2Wrong.Output.Contains('H0703') -or $RunPredicateV2Wrong.Output.Contains('H0704')) { throw "Session AF wrong implementation must be exactly H0703: $($RunPredicateV2Wrong.Output)" }
  }

  $PredicateV2Human = Read-NativeOutputWithExit 'full type Session AF typed rejection' $Hum @('full-type-check', 'fixtures/full_type_check/session_af_predicate_v2_text_uint_fail.hum')
  $PredicateV2Json = Read-NativeOutputWithExit 'full type JSON Session AF typed rejection' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_af_predicate_v2_text_uint_fail.hum')
  foreach ($Evidence in @($PredicateV2Human.Output, $PredicateV2Json.Output)) {
    foreach ($Expected in @('H0704', 'rejected_executable_predicate_semantics_v2', 'cross_type_comparison_v2', 'Text', 'integer literal', 'intent_span', 'offending_span')) {
      if (-not $Evidence.Contains($Expected)) { throw "Session AF H0704 evidence missing ${Expected}: $Evidence" }
    }
  }
  $PredicateV2Run = Read-NativeOutputWithExit 'run Session AF typed rejection preflight' $Hum @('run', 'fixtures/full_type_check/session_af_predicate_v2_text_uint_fail.hum', '--entry', 'wrong_type', '--args', 'text')
  if ($PredicateV2Run.ExitCode -ne 2 -or -not $PredicateV2Run.Output.Contains('H0704') -or $PredicateV2Run.Output.Contains('H0701') -or $PredicateV2Run.Output.Contains('runtime trap')) { throw "Session AF typed rejection must be H0704 preflight without prose/trap: $($PredicateV2Run.Output)" }

  foreach ($Prose in @('fixtures/run/session_af_predicate_v2_prose_warning.hum','fixtures/run/session_af_predicate_v2_quoted_prose_warning.hum')) {
    $PredicateV2Prose = Read-NativeOutputWithExit 'run Session AF honest prose' $Hum @('run', $Prose)
    if ($PredicateV2Prose.ExitCode -ne 0 -or -not $PredicateV2Prose.Output.Contains('H0701') -or $PredicateV2Prose.Output.Contains('H0704')) { throw "Session AF prose boundary must remain H0701: $($PredicateV2Prose.Output)" }
  }

  foreach ($Malformed in @(
    @{ File = 'fixtures/run/session_af_predicate_v2_malformed_neighbor_fail.hum'; Entry = 'malformed_neighbor' },
    @{ File = 'fixtures/run/session_af_predicate_v2_lone_bang_fail.hum'; Entry = 'lone_bang' }
  )) {
    $PredicateV2Malformed = Read-NativeOutputWithExit 'run Session AF malformed boundary' $Hum @('run', $Malformed.File, '--entry', $Malformed.Entry)
    if ($PredicateV2Malformed.ExitCode -ne 2 -or -not $PredicateV2Malformed.Output.Contains('H0704') -or $PredicateV2Malformed.Output.Contains('H0701') -or $PredicateV2Malformed.Output.Contains('runtime trap')) { throw "Session AF malformed boundary must be exact H0704: $($PredicateV2Malformed.Output)" }
  }

  foreach ($PositiveInequality in @(
    @{ Entry = 'text_inequality'; Expected = 'parse' },
    @{ Entry = 'list_inequality'; Expected = '[parse, check]' }
  )) {
    $Run = Read-NativeOutputWithExit "run Session AF positive inequality $($PositiveInequality.Entry)" $Hum @('run', 'fixtures/run/session_af_predicate_v2_inequality_pass.hum', '--entry', $PositiveInequality.Entry)
    if ($Run.ExitCode -ne 0 -or $Run.Output.Trim() -ne $PositiveInequality.Expected) { throw "Session AF positive inequality failed: $($Run.Output)" }
  }
  foreach ($FalseInequality in @('false_text_inequality', 'false_list_inequality')) {
    $Run = Read-NativeOutputWithExit "run Session AF false inequality $FalseInequality" $Hum @('run', 'fixtures/run/session_af_predicate_v2_inequality_fail.hum', '--entry', $FalseInequality)
    if ($Run.ExitCode -ne 1 -or [regex]::Matches($Run.Output, 'error\[H0703\]').Count -ne 1 -or $Run.Output.Contains('H0704')) { throw "Session AF false inequality must be exactly H0703: $($Run.Output)" }
  }

  $PredicateV2Places = 'fixtures/full_type_check/session_af_predicate_v2_places_fail.hum'
  foreach ($SurfaceName in @('resolve', 'type-env', 'type-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutputWithExit "Session AF structured place facts $SurfaceName" $Hum @($SurfaceName, '--format', 'json', $PredicateV2Places)
    if ($Surface.ExitCode -ne 0) { throw "Session AF place fixture unexpectedly blocked $SurfaceName" }
    Assert-Json "Session AF structured place facts $SurfaceName" $Surface.Output
    foreach ($Expected in @('predicate_place_facts', 'scope_id', 'root_definition_id', 'definition_id', 'resolution', 'eligibility', 'missing_value', 'helper_task', 'text_value', 'result')) {
      if (-not $Surface.Output.Contains($Expected)) { throw "Session AF $SurfaceName place facts missing $Expected" }
    }
  }
  $PlaceFullType = Read-NativeOutputWithExit 'Session AF structured place facts full type' $Hum @('full-type-check', '--format', 'json', $PredicateV2Places)
  if ($PlaceFullType.ExitCode -ne 1) { throw 'Session AF place fixture must block full type' }
  Assert-Json 'Session AF structured place facts full type' $PlaceFullType.Output
  foreach ($Expected in @('"places"', 'scope_id', 'root_definition_id', 'definition_id', 'resolution', 'eligibility', 'predicate_place_unresolved_v2', 'predicate_place_ineligible_v2', 'cross_type_comparison_v2')) {
    if (-not $PlaceFullType.Output.Contains($Expected)) { throw "Session AF full type place facts missing $Expected" }
  }
  $CrossType = @((($PlaceFullType.Output | ConvertFrom-Json).predicate_facts) | Where-Object { $_.reason -eq 'cross_type_comparison_v2' })
  if ($CrossType.Count -ne 1 -or $CrossType[0].offending_span.column -le $CrossType[0].places[0].span.column) { throw 'Session AF cross-type H0704 must blame the mismatched right operand, not its left place or operator' }
  $PlaceGraph = Read-NativeOutputWithExit 'Session AF structured place facts graph' $Hum @('graph', $PredicateV2Places)
  if ($PlaceGraph.ExitCode -ne 0) { throw 'Session AF place fixture graph must remain available' }
  Assert-Json 'Session AF structured place facts graph' $PlaceGraph.Output
  if (-not $PlaceGraph.Output.Contains('predicate_place_facts') -or -not $PlaceGraph.Output.Contains('missing_value')) { throw 'Session AF graph must expose structured predicate place facts' }
  foreach ($MalformedPlace in @('item..done == true', '.item == true', 'item. == true')) {
    if (-not $PlaceFullType.Output.Contains($MalformedPlace)) { throw "Session AF malformed place matrix missing $MalformedPlace" }
  }

  $Aggregate = Read-NativeOutputWithExit 'run Session AF aggregate malformed candidates' $Hum @('run', 'fixtures/full_type_check/session_af_predicate_v2_boundary_fail.hum', '--entry', 'malformed_boundaries')
  if ($Aggregate.ExitCode -ne 2 -or [regex]::Matches($Aggregate.Output, 'error\[H0704\]').Count -ne 19 -or $Aggregate.Output.Contains('runtime trap')) { throw "Session AF runtime must aggregate all 19 independent H0704 rows exactly once: $($Aggregate.Output)" }

  $Reachable = Read-NativeChannelsWithExit 'run Session AF reachable malformed callee before output' $Hum @('run', 'fixtures/run/session_af_predicate_v2_reachable_callee_fail.hum', '--allow', 'stdout.write')
  if ($Reachable.ExitCode -ne 2 -or $Reachable.Stdout -ne '' -or [regex]::Matches($Reachable.Stderr, 'error\[H0704\]').Count -ne 1 -or $Reachable.Stderr.Contains('runtime trap')) { throw 'Session AF reachable malformed callee must reject with exactly one H0704 and exit 2 before output or any generic trap' }
  $MixedPredicateTypeJson = Read-NativeOutputWithExit 'full type Session AF mixed predicate and body type failure' $Hum @('full-type-check', '--format', 'json', 'fixtures/run/session_af_predicate_v2_mixed_full_type_fail.hum')
  if ($MixedPredicateTypeJson.ExitCode -ne 1) { throw 'Session AF mixed fixture must block full type' }
  Assert-Json 'full type Session AF mixed predicate and body type failure' $MixedPredicateTypeJson.Output
  $MixedPredicateTypeParsed = $MixedPredicateTypeJson.Output | ConvertFrom-Json
  $MixedH0704 = @($MixedPredicateTypeParsed.predicate_facts | Where-Object { $_.diagnostic_code -eq 'H0704' })
  if ($MixedPredicateTypeParsed.summary.rejected_statements -ne 1 -or $MixedPredicateTypeParsed.summary.unchecked_statements -ne 0 -or $MixedPredicateTypeParsed.summary.unsupported_statements -ne 0 -or $MixedH0704.Count -ne 1 -or $MixedPredicateTypeParsed.summary.blocking_issues -ne 2) { throw 'Session AF mixed fixture must contain exactly one rejected body statement and one H0704 with no third blocker' }
  $MixedPredicateType = Read-NativeChannelsWithExit 'run Session AF mixed predicate and full-type failure' $Hum @('run', 'fixtures/run/session_af_predicate_v2_mixed_full_type_fail.hum')
  if ($MixedPredicateType.ExitCode -ne 1 -or $MixedPredicateType.Stdout -ne '' -or -not $MixedPredicateType.Stderr.Contains('rejected_statement_type_mismatch_v0') -or -not $MixedPredicateType.Stderr.Contains('H0704') -or $MixedPredicateType.Stderr.Contains('runtime trap')) { throw 'Session AF mixed full-type failures must remain exit 1 and preserve exactly the predicate and independent type evidence without a trap' }

  $CallablePositive = 'examples/probes/passed_pure_callable.hum'
  $CallablePositiveSource = Get-Content -Raw -LiteralPath $CallablePositive
  if ($CallablePositiveSource.Contains('allocates:')) { throw 'Session AL pinned positive source must not add an allocation declaration' }
  $CallableFirst = Read-NativeChannelsWithExit 'run Session AL passed pure callable first fresh run' $Hum @('run', $CallablePositive, '--entry', 'run_passed_callable')
  $CallableSecond = Read-NativeChannelsWithExit 'run Session AL passed pure callable second fresh run' $Hum @('run', $CallablePositive, '--entry', 'run_passed_callable')
  foreach ($Run in @($CallableFirst, $CallableSecond)) {
    if ($Run.ExitCode -ne 0 -or $Run.Stdout -ne "42`n" -or $Run.Stderr -ne '') { throw "Session AL positive must produce exact bytes 34 32 0A with empty stderr: stdout=$($Run.Stdout) stderr=$($Run.Stderr)" }
  }
  if ($CallableFirst.Stdout -ne $CallableSecond.Stdout -or $CallableFirst.Stderr -ne $CallableSecond.Stderr -or $CallableFirst.ExitCode -ne $CallableSecond.ExitCode) { throw 'Session AL fresh runs must be byte-identical' }

  foreach ($Surface in @('resolve','type-env','type-check','full-type-check','effect-check','ownership-check','resource-check','core-preview','core-lower','core-verify')) {
    $CallableSurface = Read-NativeOutputWithExit "Session AL positive $Surface JSON" $Hum @($Surface, '--format', 'json', $CallablePositive)
    if ($CallableSurface.ExitCode -ne 0) { throw "Session AL positive blocked $Surface" }
    Assert-Json "Session AL positive $Surface JSON" $CallableSurface.Output
    foreach ($Expected in @('canonical_callable_semantic_spine_am_v0','accepted_al_v0','closed_empty_v0','failure_root','none','application_facts')) {
      if (-not $CallableSurface.Output.Contains($Expected)) { throw "Session AL $Surface missing $Expected" }
    }
    if ($CallableSurface.Output.Contains('H1401') -or $CallableSurface.Output.Contains('H1402')) { throw "Session AL positive unexpectedly contains callable diagnostics in $Surface" }
    if ($Surface.StartsWith('core-') -and -not $CallableSurface.Output.Contains('core_nodes')) { throw "Session AL $Surface must expose callable Core nodes" }
  }
  $CallableGraph = Read-NativeOutputWithExit 'Session AL positive graph' $Hum @('graph', $CallablePositive)
  if ($CallableGraph.ExitCode -ne 0) { throw 'Session AL graph positive blocked' }
  Assert-Json 'Session AL positive graph' $CallableGraph.Output
  foreach ($Expected in @('definition','value_use','passed_as_argument','parameter_bind','application')) {
    if (-not $CallableGraph.Output.Contains($Expected)) { throw "Session AL graph missing $Expected edge" }
  }

  $CallableLexical = Read-NativeOutputWithExit 'run Session AL lexical identity fixture' $Hum @('run', 'fixtures/callable/session_al_lexical_identity_pass.hum', '--entry', 'run_tool')
  if ($CallableLexical.ExitCode -ne 0 -or $CallableLexical.Output.Trim() -ne '42') { throw "Session AL lexical identity must select the nested resolved task: $($CallableLexical.Output)" }
  $CallableShadowedInvalid = Read-NativeChannelsWithExit 'run Session AL shadowed invalid receiver identity' $Hum @('run', 'fixtures/callable/session_al_shadowed_invalid_receiver_pass.hum', '--entry', 'run_tool')
  if ($CallableShadowedInvalid.ExitCode -ne 0 -or $CallableShadowedInvalid.Stdout.Trim() -ne '42' -or $CallableShadowedInvalid.Stderr.Contains('H0605') -or $CallableShadowedInvalid.Stderr.Contains('H1401') -or $CallableShadowedInvalid.Stderr.Contains('H1402') -or $CallableShadowedInvalid.Stderr.Contains('runtime trap')) { throw 'Session AL runtime reachability must follow the resolver-owned app-local receiver identity' }
  $CallableSelectedInvalidReceiver = Read-NativeChannelsWithExit 'run Session AL selected invalid receiver H0605' $Hum @('run', 'fixtures/callable/session_al_selected_invalid_receiver_fail.hum', '--entry', 'run_tool', '--args', '1')
  if ($CallableSelectedInvalidReceiver.ExitCode -ne 2 -or $CallableSelectedInvalidReceiver.Stdout -ne '' -or [regex]::Matches($CallableSelectedInvalidReceiver.Stderr, 'error\[H0605\]').Count -ne 1 -or $CallableSelectedInvalidReceiver.Stderr.Contains('H1401') -or $CallableSelectedInvalidReceiver.Stderr.Contains('H1402') -or $CallableSelectedInvalidReceiver.Stderr.Contains('runtime trap')) { throw 'Session AL genuinely selected invalid receiver must retain H0605 ownership' }

  foreach ($Misuse in @(
    @{ File = 'fixtures/callable/session_al_wrong_input_fail.hum'; Entry = 'run'; Code = 'H1402' },
    @{ File = 'fixtures/callable/session_al_wrong_result_fail.hum'; Entry = 'run'; Code = 'H1402' },
    @{ File = 'fixtures/callable/session_al_fallible_task_fail.hum'; Entry = 'run'; Code = 'H1402' },
    @{ File = 'fixtures/callable/session_al_unproven_row_fail.hum'; Entry = 'run'; Code = 'H1402' },
    @{ File = 'fixtures/callable/session_al_unresolved_value_fail.hum'; Entry = 'run'; Code = 'H0601' },
    @{ File = 'fixtures/callable/session_al_non_task_value_fail.hum'; Entry = 'run'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_stored_callable_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_set_transport_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_compound_transport_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_save_transport_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_zero_application_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_multiple_application_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_permission_type_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_permission_argument_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_parameter_hws_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_argument_hws_fail.hum'; Entry = 'run'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_missing_indirect_close_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_mismatched_delimiter_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_zero_indirect_arguments_fail.hum'; Entry = 'apply_once'; Code = 'H1402' },
    @{ File = 'fixtures/callable/session_al_two_indirect_arguments_fail.hum'; Entry = 'apply_once'; Code = 'H1402' },
    @{ File = 'fixtures/callable/session_al_extra_indirect_close_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_trailing_indirect_prose_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_chained_application_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_nested_application_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_returned_callable_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_anonymous_callable_fail.hum'; Entry = 'run'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_nested_callable_escape_fail.hum'; Entry = 'apply_once'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_recursive_relationship_fail.hum'; Entry = 'run'; Code = 'H1401' },
    @{ File = 'fixtures/callable/session_al_zero_target_params_fail.hum'; Entry = 'run'; Code = 'H1402' },
    @{ File = 'fixtures/callable/session_al_multiple_target_params_fail.hum'; Entry = 'run'; Code = 'H1402' }
  )) {
    $CallableStages = if ($Misuse.Code -eq 'H1402') {
      foreach ($EarlySurface in @('resolve','type-env')) {
        $CallableEarlySurface = Read-NativeOutputWithExit "Session AL H1402 deferred through $EarlySurface $($Misuse.File)" $Hum @($EarlySurface, '--format', 'json', $Misuse.File)
        if ($CallableEarlySurface.ExitCode -ne 0 -or $CallableEarlySurface.Output.Contains('H1402')) { throw "Session AL H1402 must wait for ordinary typing at ${EarlySurface}: $($Misuse.File)" }
      }
      @('type-check','full-type-check','effect-check','ownership-check','resource-check','core-preview','core-lower','core-verify')
    } else {
      @('resolve','type-env','type-check','full-type-check','effect-check','ownership-check','resource-check','core-preview','core-lower','core-verify')
    }
    foreach ($Surface in $CallableStages) {
      foreach ($Format in @('human','json')) {
        $CallableMisuseSurface = Read-NativeOutputWithExit "Session AL misuse $Surface $Format $($Misuse.File)" $Hum @($Surface, '--format', $Format, $Misuse.File)
        if ($CallableMisuseSurface.ExitCode -ne 1) { throw "Session AL misuse must block $Surface ${Format}: $($Misuse.File)" }
        if ($Format -eq 'json') { Assert-Json "Session AL misuse $Surface JSON $($Misuse.File)" $CallableMisuseSurface.Output }
        $CallableExpectedEvidence = if ($Misuse.Code.StartsWith('H140')) { @($Misuse.Code, 'detail_reason', 'primary_span', 'help', 'related') } else { @($Misuse.Code) }
        foreach ($Expected in $CallableExpectedEvidence) {
          if (-not $CallableMisuseSurface.Output.Contains($Expected)) { throw "Session AL misuse $Surface $Format lacks ${Expected}: $($Misuse.File)" }
        }
      }
    }
    $CallableMisuseRun = Read-NativeChannelsWithExit "Session AL misuse runtime $($Misuse.File)" $Hum @('run', $Misuse.File, '--entry', $Misuse.Entry)
    if ($CallableMisuseRun.ExitCode -ne 2 -or $CallableMisuseRun.Stdout -ne '' -or -not $CallableMisuseRun.Stderr.Contains($Misuse.Code) -or $CallableMisuseRun.Stderr.Contains('runtime trap')) { throw "Session AL misuse must preflight before output without a trap: $($Misuse.File)" }
  }

  foreach ($Surface in @('resolve','type-env','type-check','full-type-check','effect-check','ownership-check','resource-check','core-preview','core-lower','core-verify')) {
    foreach ($Format in @('human','json')) {
      $CallableCrossFile = Read-NativeOutputWithExit "Session AL cross-file H1401 $Surface $Format" $Hum @($Surface, '--format', $Format, 'fixtures/callable/session_al_cross_file_fail')
      if ($CallableCrossFile.ExitCode -ne 1 -or -not $CallableCrossFile.Output.Contains('H1401') -or -not $CallableCrossFile.Output.Contains('cross_file_callable_value_unsupported_v0') -or -not $CallableCrossFile.Output.Contains('related')) { throw "Session AL cross-file callable boundary must be owned by H1401 with relationship spans in $Surface $Format" }
      if ($Format -eq 'json') { Assert-Json "Session AL cross-file H1401 $Surface JSON" $CallableCrossFile.Output }
    }
  }
  $CallableUnknownResolve = Read-NativeOutputWithExit 'Session AL unknown ordinary type resolver precedence' $Hum @('resolve', '--format', 'json', 'fixtures/callable/session_al_unknown_ordinary_type_fail.hum')
  if ($CallableUnknownResolve.ExitCode -ne 0 -or $CallableUnknownResolve.Output.Contains('H1402')) { throw 'Session AL unknown ordinary type must not be claimed by H1402 during resolution' }
  $CallableUnknownType = Read-NativeOutputWithExit 'Session AL unknown ordinary type H0605 precedence' $Hum @('type-check', '--format', 'json', 'fixtures/callable/session_al_unknown_ordinary_type_fail.hum')
  if ($CallableUnknownType.ExitCode -ne 1 -or -not $CallableUnknownType.Output.Contains('H0605') -or $CallableUnknownType.Output.Contains('H1402')) { throw 'Session AL unknown ordinary type must be owned only by H0605' }
  foreach ($Surface in @('full-type-check','effect-check','ownership-check','resource-check','core-preview','core-lower','core-verify')) {
    foreach ($Format in @('human','json')) {
      $CallableUnknownLater = Read-NativeOutputWithExit "Session AL unknown ordinary type $Surface $Format" $Hum @($Surface, '--format', $Format, 'fixtures/callable/session_al_unknown_ordinary_type_fail.hum')
      if ($CallableUnknownLater.ExitCode -ne 1 -or [regex]::Matches($CallableUnknownLater.Output, 'H0605').Count -lt 1 -or $CallableUnknownLater.Output.Contains('H1402') -or $CallableUnknownLater.Output.Contains('runtime trap')) { throw "Session AL unknown ordinary type must preserve H0605 through $Surface $Format" }
      if ($Format -eq 'json') { Assert-Json "Session AL unknown ordinary type $Surface JSON" $CallableUnknownLater.Output }
    }
  }
  $CallableUnknownRun = Read-NativeChannelsWithExit 'Session AL unknown ordinary type runtime preflight' $Hum @('run', 'fixtures/callable/session_al_unknown_ordinary_type_fail.hum', '--entry', 'unknown', '--args', '1')
  if ($CallableUnknownRun.ExitCode -ne 2 -or $CallableUnknownRun.Stdout -ne '' -or [regex]::Matches($CallableUnknownRun.Stderr, 'error\[H0605\]').Count -ne 1 -or $CallableUnknownRun.Stderr.Contains('H1402') -or $CallableUnknownRun.Stderr.Contains('runtime trap')) { throw 'Session AL unknown ordinary type must preflight as exactly H0605 before runtime' }
  $CallableUnknownRoute = Read-NativeChannelsWithExit 'Session AL reachable callable target H0605 preflight' $Hum @('run', 'fixtures/callable/session_al_unknown_ordinary_type_fail.hum', '--entry', 'run', '--args', '1')
  if ($CallableUnknownRoute.ExitCode -ne 2 -or $CallableUnknownRoute.Stdout -ne '' -or [regex]::Matches($CallableUnknownRoute.Stderr, 'error\[H0605\]').Count -ne 1 -or $CallableUnknownRoute.Stderr.Contains('H1402') -or $CallableUnknownRoute.Stderr.Contains('runtime trap')) { throw 'Session AL H0605 must follow the selected callable route before runtime' }
  $CallableHealthyUnrelated = Read-NativeChannelsWithExit 'Session AL unrelated unknown type does not block healthy entry' $Hum @('run', 'fixtures/callable/session_al_unrelated_unknown_type_pass.hum', '--entry', 'healthy')
  if ($CallableHealthyUnrelated.ExitCode -ne 0 -or $CallableHealthyUnrelated.Stdout.Trim() -ne '7' -or $CallableHealthyUnrelated.Stderr.Contains('H0605') -or $CallableHealthyUnrelated.Stderr.Contains('runtime trap')) { throw 'Session AL H0605 runtime preflight must be scoped to the selected reachable route' }
  $CallableResourceNonparticipant = Read-NativeOutputWithExit 'Session AL unrelated resource task remains checked' $Hum @('resource-check', '--format', 'json', 'fixtures/callable/session_al_resource_nonparticipant_fail.hum')
  if ($CallableResourceNonparticipant.ExitCode -ne 1 -or -not $CallableResourceNonparticipant.Output.Contains('hum_resource_item_unrelated') -or -not $CallableResourceNonparticipant.Output.Contains('rejected_missing_allocation_declaration_v0')) { throw 'Session AL resource exemption must apply only to exact callable participants' }
  $CallableOuterClose = Read-NativeOutputWithExit 'Session AL malformed outer task header' $Hum @('check', 'fixtures/callable/session_al_missing_outer_close_fail.hum')
  if ($CallableOuterClose.ExitCode -ne 1 -or -not $CallableOuterClose.Output.Contains('H0007') -or $CallableOuterClose.Output.Contains('H1401')) { throw 'Session AL malformed outer task header must remain owned by H0007' }
  $CallableMixed = Read-NativeOutputWithExit 'Session AL mixed callable and body type errors' $Hum @('full-type-check', '--format', 'json', 'fixtures/callable/session_al_mixed_body_type_fail.hum')
  if ($CallableMixed.ExitCode -ne 1) { throw 'Session AL mixed fixture must block full type' }
  Assert-Json 'Session AL mixed callable and body type errors' $CallableMixed.Output
  $CallableMixedJson = $CallableMixed.Output | ConvertFrom-Json
  if ($CallableMixedJson.summary.rejected_statements -ne 1 -or $CallableMixedJson.summary.accepted_statements -ne 1 -or $CallableMixedJson.summary.unchecked_statements -ne 1 -or $CallableMixedJson.callable_facts.diagnostics -ne 1 -or -not $CallableMixed.Output.Contains('rejected_statement_type_mismatch_v0') -or -not $CallableMixed.Output.Contains('H1401')) { throw 'Session AL mixed fixture must preserve one independent body mismatch, one accepted bare return, and only the malformed callable statement as unchecked beside one callable blocker' }

  foreach ($Code in @('H1401','H1402')) {
    $CallableExplain = Read-NativeOutputWithExit "Session AL explain $Code" $Hum @('explain', $Code)
    if ($CallableExplain.ExitCode -ne 0 -or -not $CallableExplain.Output.Contains($Code)) { throw "Session AL diagnostic catalog missing $Code" }
  }

  $CallableRowPositive = 'examples/probes/passed_callable_row.hum'
  $CallableRowFirst = Read-NativeChannelsWithExit 'run Session AM callable row first fresh run' $Hum @('run', $CallableRowPositive, '--entry', 'run_passed_callable_row')
  $CallableRowSecond = Read-NativeChannelsWithExit 'run Session AM callable row second fresh run' $Hum @('run', $CallableRowPositive, '--entry', 'run_passed_callable_row')
  foreach ($Run in @($CallableRowFirst, $CallableRowSecond)) {
    if ($Run.ExitCode -ne 0 -or $Run.Stdout -ne "42`n" -or $Run.Stderr -ne '') { throw 'Session AM nonempty-row positive must produce exact 42 newline with empty stderr' }
  }
  if ($CallableRowFirst.Stdout -ne $CallableRowSecond.Stdout -or $CallableRowFirst.Stderr -ne $CallableRowSecond.Stderr -or $CallableRowFirst.ExitCode -ne $CallableRowSecond.ExitCode) { throw 'Session AM fresh runtime runs must be byte-identical' }
  foreach ($Surface in @('resolve','type-env','type-check','full-type-check','effect-check','ownership-check','resource-check','core-preview','core-lower','core-verify')) {
    foreach ($Format in @('human','json')) {
      $CallableRowArgs = if ($Format -eq 'json') { @($Surface, '--format', 'json', $CallableRowPositive) } else { @($Surface, $CallableRowPositive) }
      $CallableRowSurface = Read-NativeOutputWithExit "Session AM nonempty row $Surface $Format" $Hum $CallableRowArgs
      if ($CallableRowSurface.ExitCode -ne 0) { throw "Session AM nonempty-row positive blocked $Surface $Format" }
      if ($Format -eq 'json') { Assert-Json "Session AM nonempty row $Surface JSON" $CallableRowSurface.Output }
      foreach ($Expected in @('canonical_callable_semantic_spine_am_v0','accepted_am_v0','open_single_tail_v0','complete_latent_row_propagated_v0','normalized_labels','normalized_tail')) {
        if (-not $CallableRowSurface.Output.Contains($Expected)) { throw "Session AM $Surface $Format missing $Expected" }
      }
      if ($Format -eq 'json') {
        foreach ($Expected in @('effect_label_occurrence_facts','row_substitution_facts','resolver_reference_id','target_definition_id')) {
          if (-not $CallableRowSurface.Output.Contains($Expected)) { throw "Session AM $Surface JSON missing $Expected" }
        }
      }
    }
  }

  function Assert-SessionAmGraphFacts($Label, $GraphReport, $ExpectRowRelationships) {
    $Edges = @($GraphReport.callable_facts.graph_edges)
    if ($Edges.Count -eq 0) { throw "$Label has no callable graph edges" }
    foreach ($Edge in $Edges) {
      if ([string]::IsNullOrWhiteSpace($Edge.id) -or [string]::IsNullOrWhiteSpace($Edge.kind) -or [string]::IsNullOrWhiteSpace($Edge.from) -or [string]::IsNullOrWhiteSpace($Edge.to) -or [string]::IsNullOrWhiteSpace($Edge.owner_definition_id) -or [string]::IsNullOrWhiteSpace($Edge.application_id) -or [string]::IsNullOrWhiteSpace($Edge.span)) { throw "$Label has an incomplete callable graph edge" }
    }
    $Applications = @($GraphReport.callable_facts.application_facts)
    if ($Applications.Count -ne 1) { throw "$Label must expose exactly one accepted application" }
    $Application = $Applications[0]
    foreach ($Edge in $Edges) {
      if ($Edge.application_id -ne $Application.id -or $Edge.owner_definition_id -ne $Application.receiver_definition_id) { throw "$Label graph edge is not joined to the exact receiver and application" }
    }
    $RowKinds = @('effect_label_occurrence','effect_alias','row_variable','row_substitution','row_argument','row_propagation')
    if ($ExpectRowRelationships) {
      foreach ($Kind in $RowKinds) {
        if (@($Edges | Where-Object kind -eq $Kind).Count -lt 1) { throw "$Label missing exact $Kind graph relationship" }
      }
      $Substitutions = @($GraphReport.callable_facts.row_substitution_facts)
      if ($Substitutions.Count -ne 1) { throw "$Label must expose exactly one row substitution" }
      $Substitution = $Substitutions[0]
      $RowVariable = @($Edges | Where-Object kind -eq 'row_variable')
      if ($RowVariable.Count -ne 1 -or $RowVariable[0].from -ne $Substitution.input_row_id -or $RowVariable[0].to -ne $Substitution.tail_id -or $RowVariable[0].span -ne $Application.direct_call_span) { throw "$Label row variable must retain exact row, tail, and application span" }
      $Occurrences = @($GraphReport.callable_facts.effect_label_occurrence_facts)
      foreach ($Occurrence in $Occurrences) {
        $OccurrenceEdge = @($Edges | Where-Object { $_.kind -eq 'effect_label_occurrence' -and $_.from -eq $Occurrence.owner_definition_id -and $_.to -eq $Occurrence.id -and $_.span -eq $Occurrence.source_span })
        $AliasEdge = @($Edges | Where-Object { $_.kind -eq 'effect_alias' -and $_.from -eq $Occurrence.alias_id -and $_.to -eq $Occurrence.id -and $_.span -eq $Occurrence.source_span })
        if ($OccurrenceEdge.Count -ne 1 -or $AliasEdge.Count -ne 1) { throw "$Label occurrence and alias edges must preserve exact identities and spans" }
      }
    } elseif (@($Edges | Where-Object { $RowKinds -contains $_.kind }).Count -ne 0) {
      throw "$Label must not expose row relationships for the retained closed-empty application"
    }
  }

  $CallableRowGraph = Read-NativeOutputWithExit 'Session AM nonempty row graph' $Hum @('graph', $CallableRowPositive)
  if ($CallableRowGraph.ExitCode -ne 0) { throw 'Session AM graph positive blocked' }
  Assert-Json 'Session AM nonempty row graph' $CallableRowGraph.Output
  $CallableRowGraphReport = $CallableRowGraph.Output | ConvertFrom-Json
  Assert-SessionAmGraphFacts 'Session AM positive' $CallableRowGraphReport $true

  $CallableAmBoundaries = @(
    @{ File = 'fixtures/callable/session_am_multiple_direct_applications_fail.hum'; Entry = 'run_second'; Mixed = $false },
    @{ File = 'fixtures/callable/session_am_mixed_pure_effectful_applications_fail.hum'; Entry = 'run_effectful'; Mixed = $true }
  )
  foreach ($Boundary in $CallableAmBoundaries) {
    foreach ($Surface in @('resolve','type-env','type-check','full-type-check','effect-check','ownership-check','resource-check','core-preview','core-lower','core-verify')) {
      foreach ($Format in @('human','json')) {
        $BoundaryArgs = if ($Format -eq 'json') { @($Surface, '--format', 'json', $Boundary.File) } else { @($Surface, $Boundary.File) }
        $BoundarySurface = Read-NativeOutputWithExit "Session AM single-relationship boundary $Surface $Format $($Boundary.File)" $Hum $BoundaryArgs
        if ($BoundarySurface.ExitCode -ne 1) { throw "Session AM single-relationship boundary must block $Surface $Format for $($Boundary.File)" }
        if ($Format -eq 'json') {
          Assert-Json "Session AM single-relationship boundary $Surface JSON $($Boundary.File)" $BoundarySurface.Output
          $BoundaryReport = $BoundarySurface.Output | ConvertFrom-Json
          if ($BoundaryReport.callable_facts.diagnostics -ne 1) { throw "Session AM boundary must own one JSON diagnostic in $Surface for $($Boundary.File)" }
        } elseif ([regex]::Matches($BoundarySurface.Output, 'code=H1401').Count -ne 1) {
          throw "Session AM boundary must own one human H1401 in $Surface for $($Boundary.File)"
        }
        if (-not $BoundarySurface.Output.Contains('H1401') -or -not $BoundarySurface.Output.Contains('multiple_direct_callable_applications_unsupported_v0') -or $BoundarySurface.Output.Contains('H1402') -or $BoundarySurface.Output.Contains('runtime trap')) { throw "Session AM single-relationship boundary ownership disagrees in $Surface $Format for $($Boundary.File)" }
      }
    }
    $BoundaryGraph = Read-NativeOutputWithExit "Session AM single-relationship graph $($Boundary.File)" $Hum @('graph', $Boundary.File)
    if ($BoundaryGraph.ExitCode -ne 1) { throw "Session AM graph boundary must block for $($Boundary.File)" }
    Assert-Json "Session AM single-relationship graph $($Boundary.File)" $BoundaryGraph.Output
    $BoundaryGraphReport = $BoundaryGraph.Output | ConvertFrom-Json
    if ($BoundaryGraphReport.callable_facts.diagnostics -ne 1) { throw "Session AM graph boundary must retain exactly one callable diagnostic for $($Boundary.File)" }
    Assert-SessionAmGraphFacts "Session AM boundary $($Boundary.File)" $BoundaryGraphReport (-not $Boundary.Mixed)
    $BoundaryRun = Read-NativeChannelsWithExit "Session AM single-relationship runtime $($Boundary.File)" $Hum @('run', $Boundary.File, '--entry', $Boundary.Entry)
    if ($BoundaryRun.ExitCode -ne 2 -or $BoundaryRun.Stdout -ne '' -or [regex]::Matches($BoundaryRun.Stderr, 'error\[H1401\]').Count -ne 1 -or $BoundaryRun.Stderr.Contains('H1402') -or $BoundaryRun.Stderr.Contains('runtime trap')) { throw "Session AM runtime boundary must reject once before execution for $($Boundary.File)" }
    if ($Boundary.Mixed) {
      $MixedJson = Read-NativeOutputWithExit 'Session AM mixed pure/effectful retained row' $Hum @('full-type-check', '--format', 'json', $Boundary.File)
      $MixedReport = $MixedJson.Output | ConvertFrom-Json
      if ($MixedReport.callable_facts.applications -ne 1 -or $MixedReport.callable_facts.substitutions -ne 0 -or -not $MixedJson.Output.Contains('closed_empty_v0') -or $MixedJson.Output.Contains('open_single_tail_v0')) { throw 'Session AM rejected second effectful application must not rewrite the accepted pure receiver row' }
    }
  }

  $CallableAmOutsideRow = Read-NativeOutputWithExit 'Session AM bounded-row H1402 reason' $Hum @('full-type-check', '--format', 'json', 'fixtures/callable/session_al_unproven_row_fail.hum')
  if ($CallableAmOutsideRow.ExitCode -ne 1 -or -not $CallableAmOutsideRow.Output.Contains('callable_latent_row_outside_bounded_am_slice_v0') -or $CallableAmOutsideRow.Output.Contains('callable_latent_row_not_closed_empty_v0')) { throw 'Session AM H1402 must describe the bounded empty/change row slice' }

  $RunSessionVWriteThrough = Read-NativeOutput 'run Session V writable alias write-through fixture' $Hum @('run', 'examples/probes/writable_field_aliases.hum', '--entry', 'write_x_through_alias', '--args', '{x:1,y:2}')
  if ($RunSessionVWriteThrough.Trim() -ne '{x: 9, y: 2}') { throw "Session V write-through expected {x: 9, y: 2}, got $RunSessionVWriteThrough" }

  $RunSessionVSwap = Read-NativeOutput 'run Session V alias swap fixture' $Hum @('run', 'examples/probes/writable_field_aliases.hum', '--entry', 'swap_xy_with_aliases', '--args', '{x:1,y:2}')
  if ($RunSessionVSwap.Trim() -ne '{x: 2, y: 1}') { throw "Session V alias swap expected {x: 2, y: 1}, got $RunSessionVSwap" }

  $RunSessionVDistinct = Read-NativeOutput 'run Session V distinct-field fixture' $Hum @('run', 'examples/probes/writable_field_aliases.hum', '--entry', 'distinct_field_access', '--args', '{x:1,y:2}')
  if ($RunSessionVDistinct.Trim() -ne '{x: 2, y: 7}') { throw "Session V distinct-field access expected {x: 2, y: 7}, got $RunSessionVDistinct" }

  $RunSessionVSequential = Read-NativeOutput 'run Session V sequential alias fixture' $Hum @('run', 'examples/probes/writable_field_aliases.hum', '--entry', 'sequential_x_aliases', '--args', '{x:1,y:2}')
  if ($RunSessionVSequential.Trim() -ne '{x: 7, y: 2}') { throw "Session V sequential aliases expected {x: 7, y: 2}, got $RunSessionVSequential" }

  $RunSessionVWriteOverlap = Read-NativeOutputWithExit 'run Session V pinned overlapping-write misuse' $Hum @('run', 'fixtures/ownership_check/session_v_program8_overlap_write_fail.hum', '--entry', 'overlapping_write', '--args', '{x:1,y:2}')
  if ($RunSessionVWriteOverlap.ExitCode -ne 2) { throw "Session V overlapping write expected exit 2, got $($RunSessionVWriteOverlap.ExitCode)" }
  foreach ($Expected in @('H0808', 'alias_to_x', 'point.x', ':13:5', ':14:5', ':15:5', 'not known independent', 'definitely distinct direct field', 'last use')) {
    if (-not $RunSessionVWriteOverlap.Output.Contains($Expected)) { throw "Session V overlapping write missing $Expected, got $($RunSessionVWriteOverlap.Output)" }
  }

  $RunSessionVReadOverlap = Read-NativeOutputWithExit 'run Session V overlapping-read misuse' $Hum @('run', 'fixtures/ownership_check/session_v_overlap_read_fail.hum', '--entry', 'overlapping_read', '--args', '{x:1,y:2}')
  if ($RunSessionVReadOverlap.ExitCode -ne 2) { throw "Session V overlapping read expected exit 2, got $($RunSessionVReadOverlap.ExitCode)" }
  if (-not $RunSessionVReadOverlap.Output.Contains('H0808')) { throw "Session V overlapping read expected H0808, got $($RunSessionVReadOverlap.Output)" }

  $RunSessionVSecondAlias = Read-NativeOutputWithExit 'run Session V second-alias misuse' $Hum @('run', 'fixtures/ownership_check/session_v_second_alias_fail.hum', '--entry', 'second_overlapping_alias', '--args', '{x:1,y:2}')
  if ($RunSessionVSecondAlias.ExitCode -ne 2) { throw "Session V second alias expected exit 2, got $($RunSessionVSecondAlias.ExitCode)" }
  if (-not $RunSessionVSecondAlias.Output.Contains('H0808')) { throw "Session V second alias expected H0808, got $($RunSessionVSecondAlias.Output)" }

  $RunSessionVEscape = Read-NativeOutputWithExit 'run Session V writable-alias escape misuse' $Hum @('run', 'fixtures/ownership_check/session_v_alias_escape_fail.hum', '--entry', 'escaped_alias', '--args', '{x:1,y:2}')
  if ($RunSessionVEscape.ExitCode -ne 2) { throw "Session V alias escape expected exit 2, got $($RunSessionVEscape.ExitCode)" }
  if (-not $RunSessionVEscape.Output.Contains('H0809')) { throw "Session V alias escape expected H0809, got $($RunSessionVEscape.Output)" }
  if (-not $RunSessionVEscape.Output.Contains('non-escaping slice')) { throw "Session V alias escape expected scope help, got $($RunSessionVEscape.Output)" }

  $RunSessionVAliasToAlias = Read-NativeOutputWithExit 'run Session V alias-to-alias misuse' $Hum @('run', 'fixtures/ownership_check/session_v_alias_to_alias_fail.hum', '--entry', 'alias_to_alias', '--args', '{x:1,y:2}')
  if ($RunSessionVAliasToAlias.ExitCode -ne 2) { throw "Session V alias-to-alias expected exit 2, got $($RunSessionVAliasToAlias.ExitCode)" }
  foreach ($Expected in @('H0809', 'writable_alias_to_alias_binding_v0', 'writable alias `first`')) {
    if (-not $RunSessionVAliasToAlias.Output.Contains($Expected)) { throw "Session V alias-to-alias runtime is missing $Expected, got $($RunSessionVAliasToAlias.Output)" }
  }

  $RunSessionVNestedAliasPlace = Read-NativeOutputWithExit 'run Session V nested alias-place misuse' $Hum @('run', 'fixtures/ownership_check/session_v_nested_alias_place_fail.hum', '--entry', 'nested_alias_place', '--args', '{x:1,y:2}')
  if ($RunSessionVNestedAliasPlace.ExitCode -ne 2) { throw "Session V nested alias place expected exit 2, got $($RunSessionVNestedAliasPlace.ExitCode)" }
  foreach ($Expected in @('H0809', 'writable_alias_shape_outside_direct_field_slice_v0', 'point.x.deep')) {
    if (-not $RunSessionVNestedAliasPlace.Output.Contains($Expected)) { throw "Session V nested alias-place runtime is missing $Expected, got $($RunSessionVNestedAliasPlace.Output)" }
  }

  $RunSessionVControl = Read-NativeOutputWithExit 'run Session V control-flow misuse' $Hum @('run', 'fixtures/ownership_check/session_v_alias_control_flow_fail.hum', '--entry', 'alias_across_branch', '--args', '{x:1,y:2}', 'true')
  if ($RunSessionVControl.ExitCode -ne 2) { throw "Session V alias control-flow misuse expected exit 2, got $($RunSessionVControl.ExitCode)" }
  if (-not $RunSessionVControl.Output.Contains('H0809')) { throw "Session V alias control-flow misuse expected H0809, got $($RunSessionVControl.Output)" }

  $RunSessionVBorrow = Read-NativeOutputWithExit 'run Session V borrowed-owner alias misuse' $Hum @('run', 'fixtures/ownership_check/session_v_borrowed_owner_alias_fail.hum', '--entry', 'borrowed_owner_alias', '--args', '{x:1,y:2}')
  if ($RunSessionVBorrow.ExitCode -ne 2) { throw "Session V borrowed owner alias expected exit 2, got $($RunSessionVBorrow.ExitCode)" }
  if (-not $RunSessionVBorrow.Output.Contains('H0802')) { throw "Session V borrowed owner alias expected H0802, got $($RunSessionVBorrow.Output)" }

  $RunSessionVPermissionWrapper = Read-NativeOutputWithExit 'run Session V alias permission-wrapper misuse' $Hum @('run', 'fixtures/ownership_check/session_v_alias_permission_wrapper_fail.hum', '--entry', 'permission_wrapped_alias', '--args', '{x:1,y:2}')
  if ($RunSessionVPermissionWrapper.ExitCode -ne 2) { throw "Session V alias permission wrapper expected exit 2, got $($RunSessionVPermissionWrapper.ExitCode)" }
  if (-not $RunSessionVPermissionWrapper.Output.Contains('H0809')) { throw "Session V alias permission wrapper expected H0809, got $($RunSessionVPermissionWrapper.Output)" }
  if (-not $RunSessionVPermissionWrapper.Output.Contains('writable_alias_permission_wrapper_v0')) { throw "Session V alias permission wrapper expected its stable reason, got $($RunSessionVPermissionWrapper.Output)" }

  $RunSessionVRebindOwner = Read-NativeOutputWithExit 'run Session V alias-owner rebinding misuse' $Hum @('run', 'fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum', '--entry', 'alias_rebinds_owner', '--args', '{x:1,y:2}')
  if ($RunSessionVRebindOwner.ExitCode -ne 2) { throw "Session V alias-owner rebinding expected exit 2, got $($RunSessionVRebindOwner.ExitCode)" }
  if (-not $RunSessionVRebindOwner.Output.Contains('H0809')) { throw "Session V alias-owner rebinding expected H0809, got $($RunSessionVRebindOwner.Output)" }
  if (-not $RunSessionVRebindOwner.Output.Contains('writable_alias_rebinds_its_owner_v0')) { throw "Session V alias-owner rebinding expected its stable reason, got $($RunSessionVRebindOwner.Output)" }

  $RunSessionVNameCollision = Read-NativeOutputWithExit 'run Session V alias-name collision misuse' $Hum @('run', 'fixtures/ownership_check/session_v_alias_name_collision_fail.hum', '--entry', 'alias_name_collision', '--args', '{x:1,y:2}', '7')
  if ($RunSessionVNameCollision.ExitCode -ne 2) { throw "Session V alias-name collision expected exit 2, got $($RunSessionVNameCollision.ExitCode)" }
  if (-not $RunSessionVNameCollision.Output.Contains('H0809')) { throw "Session V alias-name collision expected H0809, got $($RunSessionVNameCollision.Output)" }
  if (-not $RunSessionVNameCollision.Output.Contains('writable_alias_binding_rebinding_v0')) { throw "Session V alias-name collision expected its stable reason, got $($RunSessionVNameCollision.Output)" }

  $RunSessionVDeclaredNameCollision = Read-NativeOutputWithExit 'run Session V alias-declared-name collision misuse' $Hum @('run', 'fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum', '--entry', 'alias_declared_name_collision', '--args', '{x:1,y:2}')
  if ($RunSessionVDeclaredNameCollision.ExitCode -ne 2) { throw "Session V alias-declared-name collision expected exit 2, got $($RunSessionVDeclaredNameCollision.ExitCode)" }
  if (-not $RunSessionVDeclaredNameCollision.Output.Contains('H0809')) { throw "Session V alias-declared-name collision expected H0809, got $($RunSessionVDeclaredNameCollision.Output)" }
  if (-not $RunSessionVDeclaredNameCollision.Output.Contains('writable_alias_binding_rebinding_v0')) { throw "Session V alias-declared-name collision expected its stable reason, got $($RunSessionVDeclaredNameCollision.Output)" }

  $RunSessionVBorrowOverlap = Read-NativeOutputWithExit 'run Session V borrowed-owner overlap precedence' $Hum @('run', 'fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum', '--entry', 'borrowed_owner_overlap', '--args', '{x:1,y:2}')
  if ($RunSessionVBorrowOverlap.ExitCode -ne 2) { throw "Session V borrowed-owner overlap expected exit 2, got $($RunSessionVBorrowOverlap.ExitCode)" }
  if (-not $RunSessionVBorrowOverlap.Output.Contains('H0802')) { throw "Session V borrowed-owner overlap expected H0802, got $($RunSessionVBorrowOverlap.Output)" }
  if ($RunSessionVBorrowOverlap.Output.Contains('H0808')) { throw "Session V borrowed-owner overlap must keep authority precedence, got $($RunSessionVBorrowOverlap.Output)" }

  $RunSessionWSameRoot = Read-NativeOutputWithExit 'run Session W same-root success' $Hum @('run', 'examples/probes/causal_failures.hum', '--entry', 'same_root', '--args', 'false')
  if ($RunSessionWSameRoot.ExitCode -ne 0 -or $RunSessionWSameRoot.Output.Trim() -ne '7') { throw "Session W same-root success expected 7, got $($RunSessionWSameRoot.Output)" }

  $RunSessionWWrapSuccess = Read-NativeOutputWithExit 'run Session W wrapping success' $Hum @('run', 'examples/probes/causal_failures.hum', '--entry', 'outer_value', '--args', 'false')
  if ($RunSessionWWrapSuccess.ExitCode -ne 0 -or $RunSessionWWrapSuccess.Output.Trim() -ne '7') { throw "Session W wrapping success expected 7, got $($RunSessionWWrapSuccess.Output)" }

  $RunSessionWCause = Read-NativeOutputWithExit 'run Session W causal chain' $Hum @('run', 'examples/probes/causal_failures.hum', '--entry', 'outer_value', '--args', 'true')
  if ($RunSessionWCause.ExitCode -ne 1) { throw "Session W causal chain expected exit 1, got $($RunSessionWCause.ExitCode)" }
  foreach ($Expected in @('failure: OuterError.context', 'caused by: MiddleError.context', 'caused by: RootError.origin', ':74:5', ':59:5', ':27:7')) {
    if (-not $RunSessionWCause.Output.Contains($Expected)) { throw "Session W causal chain is missing $Expected" }
  }
  if ($RunSessionWCause.Output.Contains('runtime trap')) { throw 'Session W typed failure must not be labeled a runtime trap' }

  $RunSessionWRootCause = Read-NativeOutputWithExit 'run Session W direct root cause' $Hum @('run', 'examples/probes/causal_failures.hum', '--entry', 'root_value', '--args', 'true')
  if ($RunSessionWRootCause.ExitCode -ne 1) { throw "Session W direct root cause expected exit 1, got $($RunSessionWRootCause.ExitCode)" }
  foreach ($Expected in @('failure: RootError.origin', 'originated at examples/probes/causal_failures.hum:27:7')) {
    if (-not $RunSessionWRootCause.Output.Contains($Expected)) { throw "Session W direct root cause is missing $Expected" }
  }

  $RunSessionWSameRootCause = Read-NativeOutputWithExit 'run Session W same-root cause' $Hum @('run', 'examples/probes/causal_failures.hum', '--entry', 'same_root', '--args', 'true')
  if ($RunSessionWSameRootCause.ExitCode -ne 1) { throw "Session W same-root cause expected exit 1, got $($RunSessionWSameRootCause.ExitCode)" }
  foreach ($Expected in @('failure: RootError.origin', 'propagated at examples/probes/causal_failures.hum:44:5', 'originated at examples/probes/causal_failures.hum:27:7')) {
    if (-not $RunSessionWSameRootCause.Output.Contains($Expected)) { throw "Session W same-root cause is missing $Expected" }
  }

  $SessionWMisuses = @(
    @{ Path = 'fixtures/full_type_check/session_w_implicit_fallible_call_fail.hum'; Code = 'H0901' },
    @{ Path = 'fixtures/full_type_check/session_w_incompatible_unwrapped_fail.hum'; Code = 'H0902' },
    @{ Path = 'fixtures/full_type_check/session_w_wrong_wrapper_root_fail.hum'; Code = 'H0903' },
    @{ Path = 'fixtures/full_type_check/session_w_try_infallible_fail.hum'; Code = 'H0904' },
    @{ Path = 'fixtures/full_type_check/session_w_direct_wrong_root_fail.hum'; Code = 'H0905' },
    @{ Path = 'fixtures/full_type_check/session_w_unsupported_try_shape_fail.hum'; Code = 'H0906' }
  )
  foreach ($Misuse in $SessionWMisuses) {
    $Human = Read-NativeOutputWithExit "full type Session W $($Misuse.Code) human" $Hum @('full-type-check', $Misuse.Path)
    if ($Human.ExitCode -ne 1 -or -not $Human.Output.Contains($Misuse.Code) -or -not $Human.Output.Contains('help=')) { throw "Session W human evidence is incomplete for $($Misuse.Code)" }
    $Json = Read-NativeOutputWithExit "full type Session W $($Misuse.Code) JSON" $Hum @('full-type-check', '--format', 'json', $Misuse.Path)
    if ($Json.ExitCode -ne 1) { throw "Session W JSON expected exit 1 for $($Misuse.Code)" }
    Assert-Json "full type Session W $($Misuse.Code) JSON" $Json.Output
    foreach ($Expected in @("`"diagnostic_code`": `"$($Misuse.Code)`"", '"call_span":', '"caller_span":', '"help":')) {
      if (-not $Json.Output.Contains($Expected)) { throw "Session W JSON is missing $Expected for $($Misuse.Code)" }
    }
    $Runtime = Read-NativeOutputWithExit "run Session W $($Misuse.Code) misuse" $Hum @('run', $Misuse.Path, '--entry', 'caller')
    if ($Runtime.ExitCode -ne 2) { throw "Session W runtime expected rejection exit 2 for $($Misuse.Code), got $($Runtime.ExitCode)" }
    foreach ($Expected in @($Misuse.Code, 'call site', 'caller', 'help:')) {
      if (-not $Runtime.Output.Contains($Expected)) { throw "Session W runtime is missing $Expected for $($Misuse.Code)" }
    }
  }

  $NestedImplicitHuman = Read-NativeOutputWithExit 'full type Session W nested implicit calls human' $Hum @('full-type-check', 'fixtures/full_type_check/session_w_nested_implicit_calls_fail.hum')
  if ($NestedImplicitHuman.ExitCode -ne 1 -or [regex]::Matches($NestedImplicitHuman.Output, 'diagnostic=H0901').Count -ne 3) { throw 'Session W nested implicit calls must produce exactly three human H0901 rows' }
  $NestedImplicit = Read-NativeOutputWithExit 'full type Session W nested implicit calls JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_w_nested_implicit_calls_fail.hum')
  if ($NestedImplicit.ExitCode -ne 1) { throw 'Session W nested implicit calls expected full-type exit 1' }
  Assert-Json 'full type Session W nested implicit calls JSON' $NestedImplicit.Output
  if ([regex]::Matches($NestedImplicit.Output, '"diagnostic_code": "H0901"').Count -ne 3) { throw 'Session W nested implicit calls must produce exactly three H0901 rows' }
  foreach ($Expected in @('"callee": "source"', '"callee": "source_list"', '"statement_kind": "for_each_header"')) {
    if (-not $NestedImplicit.Output.Contains($Expected)) { throw "Session W nested implicit call evidence is missing $Expected" }
  }
  if ($NestedImplicit.Output.Contains('H0906')) { throw 'Session W nested implicit calls must not be misclassified as unsupported try forms' }
  foreach ($Entry in @('operator_caller', 'argument_caller', 'loop_caller')) {
    $Runtime = Read-NativeOutputWithExit "run Session W nested implicit $Entry" $Hum @('run', 'fixtures/full_type_check/session_w_nested_implicit_calls_fail.hum', '--entry', $Entry)
    if ($Runtime.ExitCode -ne 2 -or -not $Runtime.Output.Contains('H0901')) { throw "Session W nested implicit $Entry expected runtime H0901 exit 2" }
  }

  $TryingFullType = Read-NativeOutput 'full type Session W trying-prefix pass JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_w_trying_infallible_pass.hum')
  Assert-Json 'full type Session W trying-prefix pass JSON' $TryingFullType
  if (-not $TryingFullType.Contains('"blocking_issues": 0') -or $TryingFullType.Contains('H0906')) { throw 'Session W ordinary trying() call must pass without H0906' }
  $TryingRuntime = Read-NativeOutputWithExit 'run Session W trying-prefix pass' $Hum @('run', 'fixtures/full_type_check/session_w_trying_infallible_pass.hum', '--entry', 'caller')
  if ($TryingRuntime.ExitCode -ne 0 -or $TryingRuntime.Output.Trim() -ne '7') { throw "Session W ordinary trying() call expected 7, got $($TryingRuntime.Output)" }

  $TryPrefix = Read-NativeOutputWithExit 'full type Session W try-prefix fallible call JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_w_try_prefix_fallible_fail.hum')
  if ($TryPrefix.ExitCode -ne 1) { throw 'Session W try_value() implicit fallible call expected full-type exit 1' }
  Assert-Json 'full type Session W try-prefix fallible call JSON' $TryPrefix.Output
  if (-not $TryPrefix.Output.Contains('"diagnostic_code": "H0901"') -or $TryPrefix.Output.Contains('H0906')) { throw 'Session W try_value() must produce H0901 rather than H0906' }
  $TryPrefixRuntime = Read-NativeOutputWithExit 'run Session W try-prefix fallible call' $Hum @('run', 'fixtures/full_type_check/session_w_try_prefix_fallible_fail.hum', '--entry', 'caller')
  if ($TryPrefixRuntime.ExitCode -ne 2 -or -not $TryPrefixRuntime.Output.Contains('H0901') -or $TryPrefixRuntime.Output.Contains('H0906')) { throw 'Session W try_value() runtime must produce H0901 rather than H0906' }

  $UnsupportedTry = Read-NativeOutputWithExit 'full type Session W unsupported try Core corpus JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_w_unsupported_try_core_fail.hum')
  if ($UnsupportedTry.ExitCode -ne 1) { throw 'Session W unsupported try Core corpus expected full-type exit 1' }
  Assert-Json 'full type Session W unsupported try Core corpus JSON' $UnsupportedTry.Output
  if ([regex]::Matches($UnsupportedTry.Output, '"diagnostic_code": "H0906"').Count -ne 9) { throw 'Session W unsupported try Core corpus must produce exactly nine H0906 rows' }

  $UnsupportedPreview = Read-NativeOutput 'core preview Session W unsupported try JSON' $Hum @('core-preview', '--format', 'json', 'fixtures/full_type_check/session_w_unsupported_try_core_fail.hum')
  Assert-Json 'core preview Session W unsupported try JSON' $UnsupportedPreview
  if (-not $UnsupportedPreview.Contains('"blocked_statements": 9') -or [regex]::Matches($UnsupportedPreview, '"core_operation": "unsupported_try_expression"').Count -lt 9) { throw 'Session W Core preview must block all nine unsupported try forms' }

  $UnsupportedLower = Read-NativeOutput 'core lower Session W unsupported try JSON' $Hum @('core-lower', '--format', 'json', 'fixtures/full_type_check/session_w_unsupported_try_core_fail.hum')
  Assert-Json 'core lower Session W unsupported try JSON' $UnsupportedLower
  if (-not $UnsupportedLower.Contains('"blocked_operations": 9') -or [regex]::Matches($UnsupportedLower, '"core_operation": "blocked_unsupported_try_expression"').Count -ne 9) { throw 'Session W Core lower must preserve all nine unsupported try blockers' }

  $UnsupportedVerify = Read-NativeOutput 'core verify Session W unsupported try JSON' $Hum @('core-verify', '--format', 'json', 'fixtures/full_type_check/session_w_unsupported_try_core_fail.hum')
  Assert-Json 'core verify Session W unsupported try JSON' $UnsupportedVerify
  foreach ($Expected in @('"lower_blocked_operations": 9', '"failed_checks": 0')) {
    if (-not $UnsupportedVerify.Contains($Expected)) { throw "Session W Core verify unsupported try evidence is missing $Expected" }
  }
  $UnsupportedRuntime = Read-NativeOutputWithExit 'run Session W unsupported nested try' $Hum @('run', 'fixtures/full_type_check/session_w_unsupported_try_core_fail.hum', '--entry', 'nested_try', '--args', '1')
  if ($UnsupportedRuntime.ExitCode -ne 2 -or -not $UnsupportedRuntime.Output.Contains('H0906')) { throw 'Session W unsupported nested try expected runtime H0906 exit 2' }

  $SessionWPrecedence = Read-NativeOutputWithExit 'full type Session W precedence JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_w_precedence_fail.hum')
  if ($SessionWPrecedence.ExitCode -ne 1) { throw 'Session W precedence fixture expected full-type exit 1' }
  Assert-Json 'full type Session W precedence JSON' $SessionWPrecedence.Output
  foreach ($Code in @('H0901', 'H0902', 'H0906')) {
    if ([regex]::Matches($SessionWPrecedence.Output, "`"diagnostic_code`": `"$Code`"").Count -ne 1) { throw "Session W precedence fixture expected exactly one $Code" }
  }
  if ($SessionWPrecedence.Output.Contains('H0907')) { throw 'Session W relationship diagnostics must take precedence over H0907' }

  $AvoidsFailure = Read-NativeOutputWithExit 'effect Session W avoids failure JSON' $Hum @('effect-check', '--format', 'json', 'fixtures/effect_check/session_w_avoids_failure_fail.hum')
  if ($AvoidsFailure.ExitCode -ne 1) { throw 'Session W avoids failure fixture expected effect exit 1' }
  Assert-Json 'effect Session W avoids failure JSON' $AvoidsFailure.Output
  foreach ($Expected in @('"effect_kind": "typed_failure_propagation"', '"effect_kind": "typed_failure_wrap"', '"rejected_boundary_checks": 2')) {
    if (-not $AvoidsFailure.Output.Contains($Expected)) { throw "Session W avoids failure evidence is missing $Expected" }
  }

  $FullTypeWJson = Read-NativeOutput 'full type Session W positive JSON' $Hum @('full-type-check', '--format', 'json', 'examples/probes/causal_failures.hum')
  Assert-Json 'full type Session W positive JSON' $FullTypeWJson
  foreach ($Expected in @('accepted_same_root_failure_propagation_v0', 'accepted_causal_failure_wrap_v0', '"blocking_issues": 0', '"callee_result_root": "RootError"', '"caller_result_root": "OuterError"')) {
    if (-not $FullTypeWJson.Contains($Expected)) { throw "Session W full type positive JSON is missing $Expected" }
  }

  $EffectWJson = Read-NativeOutput 'effect Session W positive JSON' $Hum @('effect-check', '--format', 'json', 'examples/probes/causal_failures.hum')
  Assert-Json 'effect Session W positive JSON' $EffectWJson
  foreach ($Expected in @('accepted_declared_failure_propagation_v0', 'accepted_declared_failure_wrap_v0', '"blocking_issues": 0')) {
    if (-not $EffectWJson.Contains($Expected)) { throw "Session W effect positive JSON is missing $Expected" }
  }

  $EffectWMissing = Read-NativeOutputWithExit 'effect Session W missing fails when JSON' $Hum @('effect-check', '--format', 'json', 'fixtures/effect_check/session_w_missing_fails_when_fail.hum')
  if ($EffectWMissing.ExitCode -ne 1) { throw 'Session W missing fails-when effect fixture expected exit 1' }
  Assert-Json 'effect Session W missing fails when JSON' $EffectWMissing.Output
  foreach ($Expected in @('"diagnostic_code": "H0907"', 'rejected_missing_fails_when_declaration_v0', '"caller_span":', '"help":')) {
    if (-not $EffectWMissing.Output.Contains($Expected)) { throw "Session W missing fails-when JSON is missing $Expected" }
  }

  $SessionWPlaceholderFailures = @(
    @{ Name = 'propagation'; Path = 'fixtures/effect_check/session_w_placeholder_fails_when_propagation_fail.hum'; Form = 'failure_propagation'; Args = @('true') },
    @{ Name = 'direct failure'; Path = 'fixtures/effect_check/session_w_placeholder_fails_when_direct_fail.hum'; Form = 'direct_failure'; Args = @() },
    @{ Name = 'wrapping'; Path = 'fixtures/effect_check/session_w_placeholder_fails_when_wrap_fail.hum'; Form = 'failure_wrap'; Args = @('true') }
  )
  foreach ($Fixture in $SessionWPlaceholderFailures) {
    $Human = Read-NativeOutputWithExit "effect Session W placeholder $($Fixture.Name) human" $Hum @('effect-check', $Fixture.Path)
    if ($Human.ExitCode -ne 1 -or [regex]::Matches($Human.Output, 'code=H0907').Count -ne 1) { throw "Session W placeholder $($Fixture.Name) must produce exactly one human H0907 row" }
    foreach ($Expected in @('rejected_missing_fails_when_declaration_v0', "failure_form=$($Fixture.Form)")) {
      if (-not $Human.Output.Contains($Expected)) { throw "Session W placeholder $($Fixture.Name) human evidence is missing $Expected" }
    }

    $Json = Read-NativeOutputWithExit "effect Session W placeholder $($Fixture.Name) JSON" $Hum @('effect-check', '--format', 'json', $Fixture.Path)
    if ($Json.ExitCode -ne 1) { throw "Session W placeholder $($Fixture.Name) expected effect JSON exit 1" }
    Assert-Json "effect Session W placeholder $($Fixture.Name) JSON" $Json.Output
    if ([regex]::Matches($Json.Output, '"diagnostic_code": "H0907"').Count -ne 1) { throw "Session W placeholder $($Fixture.Name) must produce exactly one JSON H0907 row" }
    foreach ($Expected in @('rejected_missing_fails_when_declaration_v0', "`"failure_form`": `"$($Fixture.Form)`"", 'typed_failure_requires_fails_when_v0')) {
      if (-not $Json.Output.Contains($Expected)) { throw "Session W placeholder $($Fixture.Name) JSON evidence is missing $Expected" }
    }

    $RunArgs = @('run', $Fixture.Path, '--entry', 'caller')
    if ($Fixture.Args.Count -gt 0) { $RunArgs += @('--args') + $Fixture.Args }
    $Runtime = Read-NativeOutputWithExit "run Session W placeholder $($Fixture.Name)" $Hum $RunArgs
    if ($Runtime.ExitCode -ne 2 -or -not $Runtime.Output.Contains('H0907')) { throw "Session W placeholder $($Fixture.Name) must reject before execution with runtime H0907 exit 2" }
    if ($Runtime.Output.Contains('failure: RootError.origin')) { throw "Session W placeholder $($Fixture.Name) executed instead of failing preflight" }
  }

  foreach ($Command in @('resolve', 'core-preview', 'core-lower', 'core-verify', 'ownership-check', 'resource-check')) {
    $Surface = Read-NativeOutput "Session W $Command positive" $Hum @($Command, '--format', 'json', 'examples/probes/causal_failures.hum')
    Assert-Json "Session W $Command positive" $Surface
  }

  $RunSessionXPure = Read-NativeOutputWithExit 'run Session X pure app' $Hum @('run', 'examples/probes/pure_app_entry.hum', '--args', 'hello')
  if ($RunSessionXPure.ExitCode -ne 0 -or $RunSessionXPure.Output -ne '') { throw "Session X pure app must exit 0 with no automatic Unit output, got '$($RunSessionXPure.Output)'" }

  $RunSessionXFallibleSuccess = Read-NativeOutputWithExit 'run Session X fallible app success' $Hum @('run', 'examples/probes/fallible_app_entry.hum', '--args', 'false')
  if ($RunSessionXFallibleSuccess.ExitCode -ne 0 -or $RunSessionXFallibleSuccess.Output -ne '') { throw "Session X fallible app success must exit 0 with no automatic Unit output, got '$($RunSessionXFallibleSuccess.Output)'" }

  $RunSessionXShadowing = Read-NativeOutputWithExit 'run Session X direct-child shadowing' $Hum @('run', 'fixtures/app_entry/session_x_direct_child_shadows_external_pass.hum')
  if ($RunSessionXShadowing.ExitCode -ne 0 -or $RunSessionXShadowing.Output -ne '') { throw 'Session X app mode must select the direct child instead of the same-named failing external task' }

  $RunSessionXScopedCall = Read-NativeOutputWithExit 'run Session X app-local helper lookup' $Hum @('run', 'fixtures/app_entry/session_x_nested_call_shadows_external_pass.hum')
  if ($RunSessionXScopedCall.ExitCode -ne 0 -or $RunSessionXScopedCall.Output -ne '') { throw 'Session X app-mode helper calls must remain inside the selected app subtree' }

  $RunSessionXExternalSentinel = Read-NativeOutputWithExit 'run Session X external helper sentinel' $Hum @('run', 'fixtures/app_entry/session_x_nested_call_shadows_external_pass.hum', '--entry', 'choose')
  if ($RunSessionXExternalSentinel.ExitCode -ne 1 -or -not $RunSessionXExternalSentinel.Output.Contains('failure: OutsideError.wrong_scope')) { throw 'Session X direct --entry choose must reach the failing external sentinel' }
  if ($RunSessionXExternalSentinel.Output.Contains('runtime trap')) { throw 'Session X external helper sentinel must remain a typed failure' }

  $RunSessionXFailure = Read-NativeOutputWithExit 'run Session X typed app failure' $Hum @('run', 'examples/probes/fallible_app_entry.hum', '--args', 'true')
  if ($RunSessionXFailure.ExitCode -ne 1) { throw "Session X typed app failure expected exit 1, got $($RunSessionXFailure.ExitCode)" }
  foreach ($Expected in @('failure: LaunchError.requested', 'originated at examples/probes/fallible_app_entry.hum:23:9')) {
    if (-not $RunSessionXFailure.Output.Contains($Expected)) { throw "Session X typed app failure is missing $Expected" }
  }
  if ($RunSessionXFailure.Output.Contains('runtime trap')) { throw 'Session X typed app failure must not be labeled a runtime trap' }

  $SessionXPureChannels = Read-NativeChannelsWithExit 'run Session X pure app channel agreement' $Hum @('run', 'examples/probes/pure_app_entry.hum', '--args', 'hello')
  if ($SessionXPureChannels.ExitCode -ne 0 -or $SessionXPureChannels.Stdout -ne '' -or $SessionXPureChannels.Stderr -ne '') { throw 'Session X pure app must leave stdout and stderr empty' }
  $SessionXFailureChannels = Read-NativeChannelsWithExit 'run Session X typed app failure channel agreement' $Hum @('run', 'examples/probes/fallible_app_entry.hum', '--args', 'true')
  if ($SessionXFailureChannels.ExitCode -ne 1 -or $SessionXFailureChannels.Stdout -ne '' -or -not $SessionXFailureChannels.Stderr.Contains('failure: LaunchError.requested')) { throw 'Session X typed app failure must render only on stderr with exit 1' }

  $SessionXMisuses = @(
    @{ Path = 'fixtures/app_entry/session_x_missing_start_fail.hum'; Code = 'H0610' },
    @{ Path = 'fixtures/app_entry/session_x_empty_start_fail.hum'; Code = 'H0611' },
    @{ Path = 'fixtures/app_entry/session_x_duplicate_start_fail.hum'; Code = 'H0612' },
    @{ Path = 'fixtures/app_entry/session_x_multiple_start_lines_fail.hum'; Code = 'H0612' },
    @{ Path = 'fixtures/app_entry/session_x_invalid_start_name_fail.hum'; Code = 'H0613' },
    @{ Path = 'fixtures/app_entry/session_x_unknown_start_fail.hum'; Code = 'H0614' },
    @{ Path = 'fixtures/app_entry/session_x_external_same_name_fail.hum'; Code = 'H0614' },
    @{ Path = 'fixtures/app_entry/session_x_nested_non_child_fail.hum'; Code = 'H0614' },
    @{ Path = 'fixtures/app_entry/session_x_multiple_apps_fail.hum'; Code = 'H0615' },
    @{ Path = 'fixtures/app_entry/session_x_invalid_result_fail.hum'; Code = 'H0616' }
  )
  foreach ($Misuse in $SessionXMisuses) {
    $Human = Read-NativeOutputWithExit "check Session X $($Misuse.Code) human" $Hum @('check', $Misuse.Path)
    if ($Human.ExitCode -ne 1 -or [regex]::Matches($Human.Output, "error\[$($Misuse.Code)\]").Count -ne 1) { throw "Session X human evidence must contain exactly one $($Misuse.Code) for $($Misuse.Path)" }
    if (-not $Human.Output.Contains('help:')) { throw "Session X human evidence is missing repair help for $($Misuse.Path)" }

    $Json = Read-NativeOutputWithExit "check Session X $($Misuse.Code) JSON" $Hum @('check', '--format', 'json', $Misuse.Path)
    if ($Json.ExitCode -ne 1) { throw "Session X JSON expected exit 1 for $($Misuse.Path)" }
    Assert-Json "check Session X $($Misuse.Code) JSON" $Json.Output
    $Parsed = $Json.Output | ConvertFrom-Json
    if (@($Parsed.diagnostics).Count -ne 1 -or $Parsed.diagnostics[0].code -ne $Misuse.Code) { throw "Session X JSON must contain exactly one $($Misuse.Code) for $($Misuse.Path)" }
    foreach ($Expected in @('span', 'help')) {
      if ($null -eq $Parsed.diagnostics[0].$Expected) { throw "Session X JSON is missing $Expected for $($Misuse.Path)" }
    }
    if (-not $Human.Output.Contains($Parsed.diagnostics[0].message) -or -not $Human.Output.Contains($Parsed.diagnostics[0].help)) { throw "Session X human/JSON diagnostic text disagrees for $($Misuse.Path)" }

    $Runtime = Read-NativeOutputWithExit "run Session X $($Misuse.Code) misuse" $Hum @('run', $Misuse.Path)
    if ($Runtime.ExitCode -ne 1 -or [regex]::Matches($Runtime.Output, "error\[$($Misuse.Code)\]").Count -ne 1) { throw "Session X run preflight must contain exactly one $($Misuse.Code) and exit 1 for $($Misuse.Path)" }
    if ($Runtime.Output.Contains('runtime trap')) { throw "Session X source misuse must reject before runtime for $($Misuse.Path)" }
  }

  $SessionXExternalJson = Read-NativeOutputWithExit 'check Session X external same-name relationship JSON' $Hum @('check', '--format', 'json', 'fixtures/app_entry/session_x_external_same_name_fail.hum')
  $SessionXExternalParsed = $SessionXExternalJson.Output | ConvertFrom-Json
  $RelatedLabels = @($SessionXExternalParsed.diagnostics[0].related_spans | ForEach-Object { $_.label }) -join "`n"
  foreach ($Expected in @('app `lexical_tool`', 'non-child task `run_tool` is not selectable')) {
    if (-not $RelatedLabels.Contains($Expected)) { throw "Session X external same-name relationship is missing $Expected" }
  }

  $SessionXNestedJson = Read-NativeOutputWithExit 'check Session X nested non-child relationship JSON' $Hum @('check', '--format', 'json', 'fixtures/app_entry/session_x_nested_non_child_fail.hum')
  if ($SessionXNestedJson.ExitCode -ne 1) { throw 'Session X nested non-child fixture expected check exit 1' }
  $SessionXNestedParsed = $SessionXNestedJson.Output | ConvertFrom-Json
  if ($SessionXNestedParsed.diagnostics[0].code -ne 'H0614' -or @($SessionXNestedParsed.diagnostics[0].related_spans).Count -ne 2) { throw 'Session X nested non-child fixture must expose app and candidate-task relationships under H0614' }
  $SessionXNestedLabels = @($SessionXNestedParsed.diagnostics[0].related_spans | ForEach-Object { $_.label }) -join "`n"
  foreach ($Expected in @('app `outer_tool`', 'non-child task `run_tool` is not selectable')) {
    if (-not $SessionXNestedLabels.Contains($Expected)) { throw "Session X nested non-child relationship is missing $Expected" }
  }

  $SessionXScopedResolve = Read-NativeOutput 'resolve Session X app-local helper JSON' $Hum @('resolve', '--format', 'json', 'fixtures/app_entry/session_x_nested_call_shadows_external_pass.hum')
  Assert-Json 'resolve Session X app-local helper JSON' $SessionXScopedResolve
  $SessionXScopedResolveParsed = $SessionXScopedResolve | ConvertFrom-Json
  $SessionXScopeIds = @($SessionXScopedResolveParsed.scopes | ForEach-Object { $_.id })
  if (@($SessionXScopeIds | Sort-Object -Unique).Count -ne $SessionXScopeIds.Count) { throw 'Session X resolver scope IDs must be unique by lexical/source identity' }
  if ($SessionXScopedResolveParsed.summary.resolver_errors -ne 0 -or $SessionXScopedResolve.Contains('H0602')) { throw 'Session X same-named external/app-local helpers and locals must not produce resolver errors or false H0602' }
  $SessionXSharedDefinitions = @($SessionXScopedResolveParsed.definitions | Where-Object { $_.name -eq 'shared' })
  $SessionXSharedReferences = @($SessionXScopedResolveParsed.references | Where-Object { $_.reference_kind -eq 'name_ref' -and $_.name -eq 'shared' })
  if ($SessionXSharedDefinitions.Count -ne 2 -or $SessionXSharedReferences.Count -ne 2) { throw 'Session X lexical identity fixture must expose both same-named local definitions and reads' }
  foreach ($Reference in $SessionXSharedReferences) {
    $Definition = @($SessionXSharedDefinitions | Where-Object { $_.id -eq $Reference.resolved_definition_id })[0]
    if ($null -eq $Definition -or $Reference.resolution_status -ne 'resolved_v0' -or $Definition.scope_id -ne $Reference.scope_id) { throw 'Session X each same-named local read must resolve within its own callable scope' }
  }
  $SessionXAppScope = @($SessionXScopedResolveParsed.scopes | Where-Object { $_.scope_kind -eq 'app' -and $_.owner_name -eq 'scoped_calls' })[0]
  $SessionXChooseReference = @($SessionXScopedResolveParsed.references | Where-Object { $_.reference_kind -eq 'callee_ref' -and $_.name -eq 'choose' })[0]
  $SessionXChooseDefinition = @($SessionXScopedResolveParsed.definitions | Where-Object { $_.id -eq $SessionXChooseReference.resolved_definition_id -and $_.scope_id -eq $SessionXAppScope.id })[0]
  if ($SessionXChooseReference.resolution_status -ne 'resolved_v0' -or $null -eq $SessionXChooseDefinition) { throw 'Session X resolver must bind choose() to the helper defined directly inside the app scope' }
  foreach ($Command in @('full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutput "Session X $Command app-local helper" $Hum @($Command, '--format', 'json', 'fixtures/app_entry/session_x_nested_call_shadows_external_pass.hum')
    Assert-Json "Session X $Command app-local helper" $Surface
    if ($Surface.Contains('H0901') -or $Surface.Contains('blocked_by_full_type_check_errors')) { throw "Session X $Command disagrees with app-local helper identity" }
  }
  $SessionXScopedGraph = Read-NativeOutput 'graph Session X app-local helper' $Hum @('graph', 'fixtures/app_entry/session_x_nested_call_shadows_external_pass.hum')
  Assert-Json 'graph Session X app-local helper' $SessionXScopedGraph
  if ($SessionXScopedGraph.Contains('H0901')) { throw 'Session X graph must not contain H0901 for the app-local helper' }

  $SessionXStaticBlockers = @(
    @{ Name = 'external-only helper'; Path = 'fixtures/app_entry/session_x_external_helper_fail.hum'; Command = 'resolve'; Code = 'H0601' },
    @{ Name = 'duplicate direct child'; Path = 'fixtures/app_entry/session_x_duplicate_direct_child_fail.hum'; Command = 'resolve'; Code = 'H0602' },
    @{ Name = 'undeclared start result root'; Path = 'fixtures/app_entry/session_x_undeclared_result_root_fail.hum'; Command = 'type-check'; Code = 'H0605' },
    @{ Name = 'Unit return mismatch'; Path = 'fixtures/app_entry/session_x_unit_return_mismatch_fail.hum'; Command = 'type-check'; Code = 'H0606' }
  )
  foreach ($Blocker in $SessionXStaticBlockers) {
    $Static = Read-NativeOutputWithExit "Session X $($Blocker.Name) static gate" $Hum @($Blocker.Command, '--format', 'json', $Blocker.Path)
    if ($Static.ExitCode -ne 1 -or [regex]::Matches($Static.Output, "`"code`": `"$($Blocker.Code)`"").Count -ne 1) { throw "Session X $($Blocker.Name) must produce exactly one $($Blocker.Code) static diagnostic" }
    Assert-Json "Session X $($Blocker.Name) static gate" $Static.Output
    $Run = Read-NativeOutputWithExit "run Session X $($Blocker.Name) blocker" $Hum @('run', $Blocker.Path)
    if ($Run.ExitCode -ne 1 -or -not $Run.Output.Contains($Blocker.Code)) { throw "Session X app execution must stop on $($Blocker.Code) for $($Blocker.Name)" }
    if ($Blocker.Name -eq 'external-only helper') {
      $StaticParsed = $Static.Output | ConvertFrom-Json
      $BoundaryHelp = $StaticParsed.diagnostics[0].help
      if (-not $BoundaryHelp.Contains('inside the current app') -or $BoundaryHelp.Contains('uses:')) { throw 'Session X app-boundary H0601 must recommend an app-local helper and must not recommend uses:' }
      if (-not $Run.Output.Contains($BoundaryHelp)) { throw 'Session X app-boundary H0601 human and JSON help must agree' }
    }
  }

  $SessionXFullTypeBlocker = Read-NativeOutputWithExit 'full type Session X binding mismatch' $Hum @('full-type-check', '--format', 'json', 'fixtures/app_entry/session_x_full_type_binding_mismatch_fail.hum')
  if ($SessionXFullTypeBlocker.ExitCode -ne 1) { throw 'Session X binding mismatch expected full-type exit 1' }
  Assert-Json 'full type Session X binding mismatch' $SessionXFullTypeBlocker.Output
  foreach ($Expected in @('rejected_statement_type_mismatch_v0', 'statement_expression_type_mismatch')) {
    if (-not $SessionXFullTypeBlocker.Output.Contains($Expected)) { throw "Session X full-type blocker is missing $Expected" }
  }
  $SessionXFullTypeRun = Read-NativeOutputWithExit 'run Session X binding mismatch blocker' $Hum @('run', 'fixtures/app_entry/session_x_full_type_binding_mismatch_fail.hum')
  if ($SessionXFullTypeRun.ExitCode -ne 1 -or -not $SessionXFullTypeRun.Output.Contains('rejected_statement_type_mismatch_v0')) { throw 'Session X app execution must stop on the full-type binding mismatch' }

  $SessionXSourceChannels = Read-NativeChannelsWithExit 'run Session X source diagnostic channel agreement' $Hum @('run', 'fixtures/app_entry/session_x_external_same_name_fail.hum')
  if ($SessionXSourceChannels.ExitCode -ne 1 -or $SessionXSourceChannels.Stdout -ne '' -or -not $SessionXSourceChannels.Stderr.Contains('error[H0614]')) { throw 'Session X source diagnostics must render only on stderr before execution' }

  $RunSessionXDirectProbe = Read-NativeOutputWithExit 'run Session X direct --entry compatibility' $Hum @('run', 'fixtures/app_entry/session_x_external_same_name_fail.hum', '--entry', 'run_tool')
  if ($RunSessionXDirectProbe.ExitCode -ne 0 -or $RunSessionXDirectProbe.Output.Trim() -ne '()') { throw "Session X direct --entry probe expected legacy (), got $($RunSessionXDirectProbe.Output)" }

  $SessionYPositive = 'examples/probes/capability_root.hum'
  $RunSessionYPositive = Read-NativeOutputWithExit 'run Session Y capability root' $Hum @('run', $SessionYPositive)
  if ($RunSessionYPositive.ExitCode -ne 0 -or $RunSessionYPositive.Output -ne '') { throw 'Session Y capability-root app must run without output or host effects' }
  foreach ($Command in @('resolve', 'full-type-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutput "Session Y $Command positive" $Hum @($Command, '--format', 'json', $SessionYPositive)
    Assert-Json "Session Y $Command positive" $Surface
  }
  $SessionYGraph = Read-NativeOutput 'graph Session Y capability root' $Hum @('graph', $SessionYPositive)
  Assert-Json 'graph Session Y capability root' $SessionYGraph
  foreach ($Capability in @('stdout.write', 'clock.replay', 'files.read')) {
    if (-not $SessionYGraph.Contains($Capability)) { throw "Session Y graph is missing exact source capability $Capability" }
  }

  $SessionYEffect = Read-NativeOutput 'effect check Session Y capability root' $Hum @('effect-check', '--format', 'json', $SessionYPositive)
  Assert-Json 'effect check Session Y capability root' $SessionYEffect
  $SessionYEffectParsed = $SessionYEffect | ConvertFrom-Json
  $SessionYRoutes = @($SessionYEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $null -ne $_.capability_id })
  if ($SessionYRoutes.Count -ne 15) { throw "Session Y positive expected 15 source-policy rows, got $($SessionYRoutes.Count)" }
  $SessionYPolicy = @{
    'stdout.write' = @{ Core = 'output'; Target = 'bounded_bootstrap_stdout_adapter_reserved_os.stdio'; Kind = 'output_stream'; Scope = 'app_stdout'; Strength = 'write'; Mapping = 'implemented_bounded_output_v0_reserved_os.stdio_mapping' }
    'clock.replay' = @{ Core = 'time'; Target = 'ordered_runner_replay_input_no_host_clock'; Kind = 'replay_input'; Scope = 'runner_tick_sequence'; Strength = 'read_ordered'; Mapping = 'implemented_runner_replay_input_v0_no_os.clock' }
    'files.read' = @{ Core = 'file'; Target = 'one_exact_native_path_via_os.filesystem_adapter'; Kind = 'file'; Scope = 'exact_native_path'; Strength = 'read'; Mapping = 'implemented_hardened_exact_file_read_v0_reserved_os.filesystem' }
  }
  foreach ($Capability in @('stdout.write', 'clock.replay', 'files.read')) {
    $CapabilityRows = @($SessionYRoutes | Where-Object { $_.capability_id -eq $Capability })
    foreach ($Status in @('accepted_source_capability_budget_v0', 'accepted_app_capability_maximum_v0', 'accepted_caller_capability_closure_v0', 'accepted_app_capability_closure_v0')) {
      if (@($CapabilityRows | Where-Object { $_.status -eq $Status }).Count -eq 0) { throw "Session Y $Capability is missing $Status" }
    }
    foreach ($Row in $CapabilityRows) {
      foreach ($Field in @('id', 'core_effect', 'runtime_target_meaning', 'grant_kind', 'grant_scope', 'grant_strength', 'grant_lifetime', 'severity_tier', 'mapping_status', 'app_span', 'caller_span', 'declaration_span')) {
        if ($null -eq $Row.$Field) { throw "Session Y $Capability policy row is missing $Field" }
      }
      if ($Row.grant_lifetime -ne 'one_run' -or $Row.severity_tier -ne 'ordinary_external_authority') { throw "Session Y $Capability must remain an exact ordinary one-run source budget" }
      $Policy = $SessionYPolicy[$Capability]
      if ($Row.core_effect -ne $Policy.Core -or $Row.runtime_target_meaning -ne $Policy.Target -or $Row.grant_kind -ne $Policy.Kind -or $Row.grant_scope -ne $Policy.Scope -or $Row.grant_strength -ne $Policy.Strength -or $Row.mapping_status -ne $Policy.Mapping) { throw "Session Y $Capability typed policy dimensions drifted" }
    }
    $CallerRoute = @($CapabilityRows | Where-Object { $_.status -eq 'accepted_caller_capability_closure_v0' })[0]
    if ((@($CallerRoute.route_tasks) -join '->') -ne 'run_tool->authority_helper' -or @($CallerRoute.route_spans).Count -ne 1 -or $null -eq $CallerRoute.callee_span) { throw "Session Y $Capability caller route lacks forensic call/callee facts" }
  }
  $SessionYEffectRepeat = Read-NativeOutput 'effect check Session Y stable policy identifiers' $Hum @('effect-check', '--format', 'json', $SessionYPositive)
  Assert-Json 'effect check Session Y stable policy identifiers' $SessionYEffectRepeat
  $SessionYRepeatParsed = $SessionYEffectRepeat | ConvertFrom-Json
  $SessionYRepeatIds = @($SessionYRepeatParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $null -ne $_.capability_id } | ForEach-Object { $_.id } | Sort-Object)
  $SessionYRouteIds = @($SessionYRoutes | ForEach-Object { $_.id } | Sort-Object)
  if ((ConvertTo-Json -Compress $SessionYRouteIds) -ne (ConvertTo-Json -Compress $SessionYRepeatIds)) { throw 'Session Y source-policy identifiers must be stable for forensic joins' }

  $SessionYOccurrencePath = 'fixtures/app_entry/session_y_policy_id_occurrences_pass.hum'
  $RunSessionYOccurrences = Read-NativeOutputWithExit 'run Session Y policy-ID occurrences' $Hum @('run', $SessionYOccurrencePath)
  if ($RunSessionYOccurrences.ExitCode -ne 0 -or $RunSessionYOccurrences.Output -ne '') { throw 'Session Y policy-ID occurrence fixture must execute without output' }
  $SessionYOccurrences = Read-NativeOutput 'effect check Session Y policy-ID occurrences' $Hum @('effect-check', '--format', 'json', $SessionYOccurrencePath)
  Assert-Json 'effect check Session Y policy-ID occurrences' $SessionYOccurrences
  $SessionYOccurrencesParsed = $SessionYOccurrences | ConvertFrom-Json
  $SessionYOccurrenceRows = @($SessionYOccurrencesParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq 'accepted_caller_capability_closure_v0' -and $_.caller -eq 'run_tool' -and $_.capability_id -eq 'stdout.write' })
  if ($SessionYOccurrenceRows.Count -ne 3 -or @($SessionYOccurrenceRows.id | Sort-Object -Unique).Count -ne 3 -or @($SessionYOccurrenceRows.source_span.column | Sort-Object -Unique).Count -ne 3) { throw 'Session Y must give every same-statement call occurrence a unique lexical policy identity' }
  if ((@($SessionYOccurrenceRows.callee) -join ',') -ne 'left_helper,right_helper,left_helper') { throw 'Session Y policy-ID fixture must preserve different and repeated same-callee occurrence order' }
  $SessionYOccurrencesRepeat = Read-NativeOutput 'effect check Session Y policy-ID repeat stability' $Hum @('effect-check', '--format', 'json', $SessionYOccurrencePath)
  Assert-Json 'effect check Session Y policy-ID repeat stability' $SessionYOccurrencesRepeat
  $SessionYOccurrencesRepeatParsed = $SessionYOccurrencesRepeat | ConvertFrom-Json
  $SessionYOccurrenceRepeatIds = @($SessionYOccurrencesRepeatParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq 'accepted_caller_capability_closure_v0' -and $_.caller -eq 'run_tool' -and $_.capability_id -eq 'stdout.write' } | ForEach-Object { $_.id })
  if ((@($SessionYOccurrenceRows.id) -join ',') -ne ($SessionYOccurrenceRepeatIds -join ',')) { throw 'Session Y lexical policy identities must repeat deterministically' }
  foreach ($Expected in @('effect report does not prove operator grants, consent prompts, persistence, or wildcard authority', 'effect report performs no host capability operation or runtime audit exercise')) {
    if (-not $SessionYEffect.Contains($Expected)) { throw "Session Y effect honesty lock is missing $Expected" }
  }

  $SessionYMisuses = @(
    @{ Name = 'unknown capability'; Path = 'fixtures/app_entry/session_y_unknown_capability_fail.hum'; Code = 'H0617'; Status = 'rejected_unknown_source_capability_v0'; Related = 1 },
    @{ Name = 'missing caller closure'; Path = 'fixtures/app_entry/session_y_missing_caller_capability_fail.hum'; Code = 'H0618'; Status = 'rejected_missing_caller_capability_v0'; Related = 3 },
    @{ Name = 'app maximum mismatch'; Path = 'fixtures/app_entry/session_y_app_capability_mismatch_fail.hum'; Code = 'H0619'; Status = 'rejected_app_capability_mismatch_v0'; Related = 3 }
  )
  foreach ($Misuse in $SessionYMisuses) {
    $Human = Read-NativeOutputWithExit "check Session Y $($Misuse.Name) human" $Hum @('check', $Misuse.Path)
    if ($Human.ExitCode -ne 1 -or [regex]::Matches($Human.Output, "error\[$($Misuse.Code)\]").Count -ne 1) { throw "Session Y $($Misuse.Name) must emit exactly one error[$($Misuse.Code)]" }
    $Json = Read-NativeOutputWithExit "check Session Y $($Misuse.Name) JSON" $Hum @('check', '--format', 'json', $Misuse.Path)
    if ($Json.ExitCode -ne 1) { throw "Session Y $($Misuse.Name) JSON expected exit 1" }
    Assert-Json "check Session Y $($Misuse.Name) JSON" $Json.Output
    $Parsed = $Json.Output | ConvertFrom-Json
    $Diagnostic = @($Parsed.diagnostics | Where-Object { $_.code -eq $Misuse.Code })[0]
    if ($null -eq $Diagnostic -or @($Diagnostic.related_spans).Count -ne $Misuse.Related -or -not $Human.Output.Contains($Diagnostic.message) -or -not $Human.Output.Contains($Diagnostic.help)) { throw "Session Y $($Misuse.Name) human/JSON blame disagrees" }
    $Effect = Read-NativeOutputWithExit "effect check Session Y $($Misuse.Name)" $Hum @('effect-check', '--format', 'json', $Misuse.Path)
    if ($Effect.ExitCode -ne 1) { throw "Session Y $($Misuse.Name) effect check expected exit 1" }
    Assert-Json "effect check Session Y $($Misuse.Name)" $Effect.Output
    $EffectParsed = $Effect.Output | ConvertFrom-Json
    $EffectRow = @($EffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq $Misuse.Status })[0]
    if ($null -eq $EffectRow -or $EffectRow.diagnostic_code -ne $Misuse.Code -or $null -eq $EffectRow.declaration_span) { throw "Session Y $($Misuse.Name) effect policy row disagrees" }
    $Run = Read-NativeOutputWithExit "run Session Y $($Misuse.Name)" $Hum @('run', $Misuse.Path)
    if ($Run.ExitCode -ne 1 -or [regex]::Matches($Run.Output, "error\[$($Misuse.Code)\]").Count -ne 1 -or $Run.Output.Contains('runtime trap')) { throw "Session Y $($Misuse.Name) must block before runtime" }
  }
  $SessionYUnknownEffect = Read-NativeOutputWithExit 'effect check Session Y sandbox bypass tier' $Hum @('effect-check', '--format', 'json', 'fixtures/app_entry/session_y_unknown_capability_fail.hum')
  $SessionYUnknownParsed = $SessionYUnknownEffect.Output | ConvertFrom-Json
  $SessionYUnknownRow = @($SessionYUnknownParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq 'rejected_unknown_source_capability_v0' })[0]
  if ($SessionYUnknownRow.capability_id -ne 'process.run' -or $SessionYUnknownRow.severity_tier -ne 'sandbox_bypass_authority' -or $SessionYUnknownRow.mapping_status -ne 'forbidden_in_work_order_6_v0') { throw 'Session Y sandbox-bypass authority must remain a separate forbidden severity tier' }

  $SessionYEntryPath = 'fixtures/app_entry/session_y_entry_capability_bypass_fail.hum'
  $RunSessionYEntry = Read-NativeOutputWithExit 'run Session Y authority-bearing direct entry' $Hum @('run', $SessionYEntryPath, '--entry', 'run_tool')
  if ($RunSessionYEntry.ExitCode -ne 1 -or [regex]::Matches($RunSessionYEntry.Output, 'error\[H0620\]').Count -ne 1 -or -not $RunSessionYEntry.Output.Contains('clock.replay')) { throw 'Session Y authority-bearing --entry must emit exactly one H0620' }
  $RunSessionYEntryApp = Read-NativeOutputWithExit 'run Session Y entry fixture in app mode' $Hum @('run', $SessionYEntryPath)
  if ($RunSessionYEntryApp.ExitCode -ne 0 -or $RunSessionYEntryApp.Output -ne '') { throw 'Session Y entry-bypass fixture must remain valid through structural app mode' }
  foreach ($EntryUnknown in @(
    @{ Name = 'transitive sandbox bypass'; Path = 'fixtures/app_entry/session_y_entry_transitive_process_fail.hum'; Capability = 'process.run' },
    @{ Name = 'transitive wildcard'; Path = 'fixtures/app_entry/session_y_entry_transitive_wildcard_fail.hum'; Capability = 'stdout.*' }
  )) {
    $RunEntryUnknown = Read-NativeOutputWithExit "run Session Y $($EntryUnknown.Name) direct entry" $Hum @('run', $EntryUnknown.Path, '--entry', 'run_tool')
    if ($RunEntryUnknown.ExitCode -ne 1 -or [regex]::Matches($RunEntryUnknown.Output, 'error\[H0617\]').Count -ne 1 -or -not $RunEntryUnknown.Output.Contains($EntryUnknown.Capability) -or -not $RunEntryUnknown.Output.Contains('direct-entry authority route call 1') -or $RunEntryUnknown.Output -match '(?m)^\(\)$' -or $RunEntryUnknown.Output.Contains('runtime trap')) { throw "Session Y $($EntryUnknown.Name) must reject transitively with routed H0617 before runtime" }
  }
  $RunSessionYPureEntry = Read-NativeOutputWithExit 'run Session Y pure direct entry compatibility' $Hum @('run', 'examples/probes/pure_app_entry.hum', '--entry', 'run_tool', '--args', 'hello')
  if ($RunSessionYPureEntry.ExitCode -ne 0 -or $RunSessionYPureEntry.Output.Trim() -ne '()') { throw 'Session Y must preserve pure direct --entry behavior' }

  $SessionZPositive = 'examples/probes/bounded_stdout.hum'
  foreach ($Command in @('resolve', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutput "Session Z $Command positive" $Hum @($Command, '--format', 'json', $SessionZPositive)
    Assert-Json "Session Z $Command positive" $Surface
  }
  $SessionZGraph = Read-NativeOutput 'Session Z graph positive' $Hum @('graph', $SessionZPositive)
  Assert-Json 'Session Z graph positive' $SessionZGraph
  $SessionZAllow = Read-NativeChannelsWithExit 'run Session Z exact allow' $Hum @('run', $SessionZPositive, '--allow', 'stdout.write', '--args', 'hello')
  if ($SessionZAllow.ExitCode -ne 0 -or $SessionZAllow.Stdout -ne 'hello' -or $SessionZAllow.Stderr -ne '') { throw 'Session Z exact allow must write only exact bytes with no newline or diagnostic channel output' }
  if ($IsWindows) {
    $SessionZWindowsPath = 'examples\probes\bounded_stdout.hum'
    $SessionZWindowsAllow = Read-NativeChannelsWithExit 'run Session Z Windows separator identity' $Hum @('run', $SessionZWindowsPath, '--allow', 'stdout.write', '--args', 'hello')
    if ($SessionZWindowsAllow.ExitCode -ne $SessionZAllow.ExitCode -or $SessionZWindowsAllow.Stdout -ne $SessionZAllow.Stdout -or $SessionZWindowsAllow.Stderr -ne $SessionZAllow.Stderr) { throw 'Session Z forward-slash and backslash source paths must select the same output policy and channels' }
  }
  $SessionZDuplicateAllow = Read-NativeChannelsWithExit 'run Session Z duplicate allow' $Hum @('run', $SessionZPositive, '--allow', 'stdout.write', '--allow=stdout.write', '--args', 'hello')
  if ($SessionZDuplicateAllow.ExitCode -ne 0 -or $SessionZDuplicateAllow.Stdout -ne 'hello' -or $SessionZDuplicateAllow.Stderr -ne '') { throw 'Session Z duplicate exact allows must be idempotent' }
  foreach ($Denied in @(
    @{ Name = 'default deny'; Args = @('run', $SessionZPositive, '--args', 'blocked') },
    @{ Name = 'explicit deny'; Args = @('run', $SessionZPositive, '--deny', 'stdout.write', '--args', 'blocked') },
    @{ Name = 'deny overrides allow'; Args = @('run', $SessionZPositive, '--allow', 'stdout.write', '--deny', 'stdout.write', '--args', 'blocked') }
  )) {
    $DeniedRun = Read-NativeChannelsWithExit "run Session Z $($Denied.Name)" $Hum $Denied.Args
    if ($DeniedRun.ExitCode -ne 1 -or $DeniedRun.Stdout -ne '' -or -not $DeniedRun.Stderr.Contains('failure: AppError.output') -or -not $DeniedRun.Stderr.Contains('caused by: OutputError.denied') -or -not $DeniedRun.Stderr.Contains('while calling `stdout_write`') -or $DeniedRun.Stderr.Contains('runtime trap')) { throw "Session Z $($Denied.Name) must be a typed causal denial with zero program output" }
  }
  $SessionZEntry = Read-NativeChannelsWithExit 'run Session Z allowed direct entry rejection' $Hum @('run', $SessionZPositive, '--allow', 'stdout.write', '--entry', 'run_tool', '--args', 'blocked')
  if ($SessionZEntry.ExitCode -ne 1 -or $SessionZEntry.Stdout -ne '' -or [regex]::Matches($SessionZEntry.Stderr, 'error\[H0620\]').Count -ne 1) { throw 'Session Z --entry must not bypass the app even with an allow' }

  foreach ($Usage in @(
    @{ Name = 'incomplete file allow'; Args = @('run', $SessionZPositive, '--allow', 'files.read'); Text = 'incomplete grant' },
    @{ Name = 'payload allow'; Args = @('run', $SessionZPositive, '--allow=stdout.write:console'); Text = 'forbidden payload' },
    @{ Name = 'missing allow value'; Args = @('run', $SessionZPositive, '--allow'); Text = 'requires an exact grant' }
  )) {
    $UsageRun = Read-NativeChannelsWithExit "run Session Z $($Usage.Name)" $Hum $Usage.Args
    if ($UsageRun.ExitCode -ne 2 -or $UsageRun.Stdout -ne '' -or -not $UsageRun.Stderr.Contains($Usage.Text)) { throw "Session Z $($Usage.Name) must be a stable CLI usage error" }
  }

  $SessionZMissing = 'fixtures/app_entry/session_z_missing_stdout_source_fail.hum'
  $SessionZMissingHuman = Read-NativeOutputWithExit 'check Session Z missing source human' $Hum @('check', $SessionZMissing)
  $SessionZMissingJson = Read-NativeOutputWithExit 'check Session Z missing source JSON' $Hum @('check', '--format', 'json', $SessionZMissing)
  if ($SessionZMissingHuman.ExitCode -ne 1 -or [regex]::Matches($SessionZMissingHuman.Output, 'error\[H0621\]').Count -ne 1 -or $SessionZMissingJson.ExitCode -ne 1) { throw 'Session Z missing source authority must emit exactly one H0621' }
  Assert-Json 'check Session Z missing source JSON' $SessionZMissingJson.Output
  $SessionZMissingParsed = $SessionZMissingJson.Output | ConvertFrom-Json
  $SessionZMissingDiagnostic = @($SessionZMissingParsed.diagnostics | Where-Object { $_.code -eq 'H0621' })[0]
  if ($null -eq $SessionZMissingDiagnostic -or @($SessionZMissingDiagnostic.related_spans).Count -ne 2 -or -not $SessionZMissingHuman.Output.Contains($SessionZMissingDiagnostic.message) -or -not $SessionZMissingHuman.Output.Contains($SessionZMissingDiagnostic.help)) { throw 'Session Z H0621 human/JSON call-task-app blame disagrees' }
  $SessionZMissingEffect = Read-NativeOutputWithExit 'effect check Session Z missing source' $Hum @('effect-check', '--format', 'json', $SessionZMissing)
  if ($SessionZMissingEffect.ExitCode -ne 1) { throw 'Session Z missing source effect check must block' }
  Assert-Json 'effect check Session Z missing source' $SessionZMissingEffect.Output
  $SessionZMissingEffectParsed = $SessionZMissingEffect.Output | ConvertFrom-Json
  $SessionZMissingRow = @($SessionZMissingEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq 'rejected_missing_output_source_authority_v0' })[0]
  if ($null -eq $SessionZMissingRow -or $SessionZMissingRow.diagnostic_code -ne 'H0621' -or $null -eq $SessionZMissingRow.app_span -or $null -eq $SessionZMissingRow.caller_span -or @($SessionZMissingRow.route_spans).Count -ne 1) { throw 'Session Z effect row must preserve H0621 call-task-app policy facts' }
  $SessionZMissingRun = Read-NativeChannelsWithExit 'run Session Z missing source' $Hum @('run', $SessionZMissing, '--allow', 'stdout.write')
  if ($SessionZMissingRun.ExitCode -ne 1 -or $SessionZMissingRun.Stdout -ne '' -or [regex]::Matches($SessionZMissingRun.Stderr, 'error\[H0621\]').Count -ne 1 -or $SessionZMissingRun.Stderr.Contains('runtime trap')) { throw 'Session Z H0621 must block before adapter execution' }
  $SessionZLegacyOutput = Read-NativeChannelsWithExit 'run Session Z legacy output without app' $Hum @('run', 'fixtures/app_entry/session_z_legacy_output_without_app_fail.hum', '--allow', 'stdout.write')
  if ($SessionZLegacyOutput.ExitCode -ne 1 -or $SessionZLegacyOutput.Stdout -ne '' -or [regex]::Matches($SessionZLegacyOutput.Stderr, 'error\[H0621\]').Count -ne 1 -or -not $SessionZLegacyOutput.Stderr.Contains('app declaration') -or $SessionZLegacyOutput.Stderr.Contains('runtime trap')) { throw 'Session Z output without a structural app must fail statically under H0621' }
  $SessionZMissingCaller = Read-NativeChannelsWithExit 'run Session Z output authority laundering' $Hum @('run', 'fixtures/app_entry/session_z_missing_output_caller_fail.hum', '--allow', 'stdout.write')
  if ($SessionZMissingCaller.ExitCode -ne 1 -or $SessionZMissingCaller.Stdout -ne '' -or [regex]::Matches($SessionZMissingCaller.Stderr, 'error\[H0618\]').Count -ne 1 -or -not $SessionZMissingCaller.Stderr.Contains('run_tool') -or -not $SessionZMissingCaller.Stderr.Contains('emit') -or $SessionZMissingCaller.Stderr.Contains('error[H0621]') -or $SessionZMissingCaller.Stderr.Contains('runtime trap')) { throw 'Session Z helper routing must reject authority laundering under H0618 before runtime' }

  $SessionZReservedName = 'fixtures/app_entry/session_z_reserved_stdout_name_fail.hum'
  $SessionZReservedNameHuman = Read-NativeOutputWithExit 'check Session Z reserved stdout name human' $Hum @('check', $SessionZReservedName)
  $SessionZReservedNameJson = Read-NativeOutputWithExit 'check Session Z reserved stdout name JSON' $Hum @('check', '--format', 'json', $SessionZReservedName)
  if ($SessionZReservedNameHuman.ExitCode -ne 1 -or [regex]::Matches($SessionZReservedNameHuman.Output, 'error\[H0623\]').Count -ne 1 -or $SessionZReservedNameJson.ExitCode -ne 1) { throw 'Session Z reserved stdout name must produce exactly one H0623' }
  Assert-Json 'check Session Z reserved stdout name JSON' $SessionZReservedNameJson.Output
  $SessionZReservedNameParsed = $SessionZReservedNameJson.Output | ConvertFrom-Json
  if (@($SessionZReservedNameParsed.diagnostics | Where-Object { $_.code -eq 'H0623' }).Count -ne 1) { throw 'Session Z H0623 human/JSON identity disagrees' }
  foreach ($Command in @('resolve', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutputWithExit "Session Z reserved stdout name $Command" $Hum @($Command, '--format', 'json', $SessionZReservedName)
    if ($Surface.ExitCode -ne 1) { throw "Session Z reserved stdout name must block $Command" }
    Assert-Json "Session Z reserved stdout name $Command" $Surface.Output
  }
  $SessionZReservedNameRun = Read-NativeChannelsWithExit 'run Session Z reserved stdout name' $Hum @('run', $SessionZReservedName, '--allow', 'stdout.write')
  if ($SessionZReservedNameRun.ExitCode -ne 1 -or $SessionZReservedNameRun.Stdout -ne '' -or [regex]::Matches($SessionZReservedNameRun.Stderr, 'error\[H0623\]').Count -ne 1 -or $SessionZReservedNameRun.Stderr.Contains('ShadowError.sentinel')) { throw 'Session Z H0623 must prevent runtime from substituting either callable body' }

  $SessionZOutputRecursion = 'fixtures/app_entry/session_z_output_recursion_fail.hum'
  $SessionZOutputRecursionHuman = Read-NativeOutputWithExit 'check Session Z output recursion human' $Hum @('check', $SessionZOutputRecursion)
  $SessionZOutputRecursionJson = Read-NativeOutputWithExit 'check Session Z output recursion JSON' $Hum @('check', '--format', 'json', $SessionZOutputRecursion)
  if ($SessionZOutputRecursionHuman.ExitCode -ne 1 -or [regex]::Matches($SessionZOutputRecursionHuman.Output, 'error\[H0624\]').Count -ne 1 -or $SessionZOutputRecursionJson.ExitCode -ne 1) { throw 'Session Z output-reachable recursion must produce exactly one H0624' }
  Assert-Json 'check Session Z output recursion JSON' $SessionZOutputRecursionJson.Output
  $SessionZOutputRecursionParsed = $SessionZOutputRecursionJson.Output | ConvertFrom-Json
  $SessionZOutputRecursionDiagnostic = @($SessionZOutputRecursionParsed.diagnostics | Where-Object { $_.code -eq 'H0624' })[0]
  if ($null -eq $SessionZOutputRecursionDiagnostic -or @($SessionZOutputRecursionDiagnostic.related_spans).Count -lt 5 -or -not $SessionZOutputRecursionDiagnostic.message.Contains('emit_again')) { throw 'Session Z H0624 must preserve the recursive task and complete finite route-to-cycle blame' }
  foreach ($Command in @('resolve', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutputWithExit "Session Z output recursion $Command" $Hum @($Command, '--format', 'json', $SessionZOutputRecursion)
    if ($Surface.ExitCode -ne 1) { throw "Session Z output recursion must block $Command" }
    Assert-Json "Session Z output recursion $Command" $Surface.Output
  }
  $SessionZOutputRecursionGraph = Read-NativeOutputWithExit 'graph Session Z output recursion' $Hum @('graph', $SessionZOutputRecursion)
  if ($SessionZOutputRecursionGraph.ExitCode -ne 1) { throw 'Session Z output recursion must block graph' }
  Assert-Json 'graph Session Z output recursion' $SessionZOutputRecursionGraph.Output
  $SessionZOutputRecursionRun = Read-NativeChannelsWithExit 'run Session Z output recursion' $Hum @('run', $SessionZOutputRecursion, '--allow', 'stdout.write')
  if ($SessionZOutputRecursionRun.ExitCode -ne 1 -or $SessionZOutputRecursionRun.Stdout -ne '' -or [regex]::Matches($SessionZOutputRecursionRun.Stderr, 'error\[H0624\]').Count -ne 1 -or $SessionZOutputRecursionRun.Stderr.Contains('runtime trap')) { throw 'Session Z H0624 must reject before output or a generic runtime trap' }

  foreach ($Precedence in @(
    @{ Name = 'recursion plus missing source authority'; Path = 'fixtures/app_entry/session_z_recursion_missing_source_precedence_fail.hum'; Code = 'H0621'; Status = 'rejected_missing_output_source_authority_v0' },
    @{ Name = 'recursion plus missing caller authority'; Path = 'fixtures/app_entry/session_z_recursion_missing_caller_precedence_fail.hum'; Code = 'H0618'; Status = 'rejected_missing_caller_capability_v0' }
  )) {
    $PrecedenceHuman = Read-NativeOutputWithExit "check Session Z $($Precedence.Name) human" $Hum @('check', $Precedence.Path)
    $PrecedenceJson = Read-NativeOutputWithExit "check Session Z $($Precedence.Name) JSON" $Hum @('check', '--format', 'json', $Precedence.Path)
    if ($PrecedenceHuman.ExitCode -ne 1 -or [regex]::Matches($PrecedenceHuman.Output, "error\[$($Precedence.Code)\]").Count -ne 1 -or $PrecedenceHuman.Output.Contains('H0624') -or $PrecedenceJson.ExitCode -ne 1) { throw "Session Z $($Precedence.Name) must have exactly one fundamental $($Precedence.Code) diagnostic" }
    Assert-Json "check Session Z $($Precedence.Name) JSON" $PrecedenceJson.Output
    $PrecedenceJsonParsed = $PrecedenceJson.Output | ConvertFrom-Json
    $PrecedenceErrors = @($PrecedenceJsonParsed.diagnostics | Where-Object { $_.severity -eq 'error' })
    if ($PrecedenceErrors.Count -ne 1 -or $PrecedenceErrors[0].code -ne $Precedence.Code) { throw "Session Z $($Precedence.Name) JSON ownership disagrees" }

    $PrecedenceEffect = Read-NativeOutputWithExit "effect Session Z $($Precedence.Name)" $Hum @('effect-check', '--format', 'json', $Precedence.Path)
    if ($PrecedenceEffect.ExitCode -ne 1) { throw "Session Z $($Precedence.Name) must block effect checking" }
    Assert-Json "effect Session Z $($Precedence.Name)" $PrecedenceEffect.Output
    $PrecedenceEffectParsed = $PrecedenceEffect.Output | ConvertFrom-Json
    $PrecedenceRejectedRows = @($PrecedenceEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $null -ne $_.diagnostic_code })
    if ($PrecedenceRejectedRows.Count -ne 1 -or $PrecedenceRejectedRows[0].diagnostic_code -ne $Precedence.Code -or $PrecedenceRejectedRows[0].status -ne $Precedence.Status) { throw "Session Z $($Precedence.Name) effect ownership disagrees" }

    $PrecedenceGraph = Read-NativeOutputWithExit "graph Session Z $($Precedence.Name)" $Hum @('graph', $Precedence.Path)
    if ($PrecedenceGraph.ExitCode -ne 1) { throw "Session Z $($Precedence.Name) must block graph" }
    Assert-Json "graph Session Z $($Precedence.Name)" $PrecedenceGraph.Output
    $PrecedenceGraphParsed = $PrecedenceGraph.Output | ConvertFrom-Json
    $PrecedenceGraphErrors = @($PrecedenceGraphParsed.diagnostics | Where-Object { $_.severity -eq 'error' })
    if ($PrecedenceGraphErrors.Count -ne 1 -or $PrecedenceGraphErrors[0].code -ne $Precedence.Code) { throw "Session Z $($Precedence.Name) graph ownership disagrees" }

    $PrecedenceRun = Read-NativeChannelsWithExit "run Session Z $($Precedence.Name)" $Hum @('run', $Precedence.Path, '--allow', 'stdout.write')
    if ($PrecedenceRun.ExitCode -ne 1 -or $PrecedenceRun.Stdout -ne '' -or [regex]::Matches($PrecedenceRun.Stderr, "error\[$($Precedence.Code)\]").Count -ne 1 -or $PrecedenceRun.Stderr.Contains('H0624') -or $PrecedenceRun.Stderr.Contains('runtime trap')) { throw "Session Z $($Precedence.Name) runtime-preflight ownership disagrees" }
  }

  $SessionZReservedTarget = 'fixtures/target_facts/session_z_reserved_stdio_requirement_fail.hum'
  $SessionZReservedTargetHuman = Read-NativeOutputWithExit 'check Session Z reserved stdio target human' $Hum @('check', $SessionZReservedTarget)
  $SessionZReservedTargetJson = Read-NativeOutputWithExit 'check Session Z reserved stdio target JSON' $Hum @('check', '--format', 'json', $SessionZReservedTarget)
  if ($SessionZReservedTargetHuman.ExitCode -ne 1 -or [regex]::Matches($SessionZReservedTargetHuman.Output, 'error\[H1204\]').Count -ne 1 -or -not $SessionZReservedTargetHuman.Output.Contains('reserved_mapping_only') -or $SessionZReservedTargetJson.ExitCode -ne 1) { throw 'Session Z reserved os.stdio mapping must fail closed under H1204' }
  Assert-Json 'check Session Z reserved stdio target JSON' $SessionZReservedTargetJson.Output
  $SessionZReservedTargetParsed = $SessionZReservedTargetJson.Output | ConvertFrom-Json
  $SessionZReservedTargetDiagnostic = @($SessionZReservedTargetParsed.diagnostics | Where-Object { $_.code -eq 'H1204' })[0]
  if ($null -eq $SessionZReservedTargetDiagnostic -or -not $SessionZReservedTargetDiagnostic.message.Contains('os.stdio') -or -not $SessionZReservedTargetDiagnostic.message.Contains('reserved_mapping_only')) { throw 'Session Z reserved target H1204 JSON lacks exact non-availability facts' }
  $SessionZReservedTargetGraph = Read-NativeOutputWithExit 'graph Session Z reserved stdio target' $Hum @('graph', $SessionZReservedTarget)
  if ($SessionZReservedTargetGraph.ExitCode -ne 1) { throw 'Session Z reserved stdio target graph must retain the blocker' }
  Assert-Json 'graph Session Z reserved stdio target' $SessionZReservedTargetGraph.Output
  $SessionZReservedTargetGraphParsed = $SessionZReservedTargetGraph.Output | ConvertFrom-Json
  if (@($SessionZReservedTargetGraphParsed.portability.unavailable_capability_families) -notcontains 'os.stdio') { throw 'Session Z graph must classify reserved os.stdio as non-satisfying' }

  $SessionZWrongType = Read-NativeOutputWithExit 'full type Session Z wrong output argument' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_z_stdout_wrong_type_fail.hum')
  if ($SessionZWrongType.ExitCode -ne 1) { throw 'Session Z wrong output argument must block full type' }
  Assert-Json 'full type Session Z wrong output argument' $SessionZWrongType.Output
  $SessionZWrongTypeParsed = $SessionZWrongType.Output | ConvertFrom-Json
  $SessionZWrongTypeRow = @($SessionZWrongTypeParsed.typed_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0622' })[0]
  if ($null -eq $SessionZWrongTypeRow -or $SessionZWrongTypeRow.expected_type -ne 'Text' -or $SessionZWrongTypeRow.actual_type -ne 'integer_literal') { throw 'Session Z H0622 must pin the exact Text argument signature' }

  $SessionZImplicit = 'fixtures/full_type_check/session_z_implicit_stdout_fail.hum'
  $SessionZImplicitType = Read-NativeOutputWithExit 'full type Session Z implicit output failure' $Hum @('full-type-check', '--format', 'json', $SessionZImplicit)
  if ($SessionZImplicitType.ExitCode -ne 1) { throw 'Session Z implicit output call must block full type' }
  Assert-Json 'full type Session Z implicit output failure' $SessionZImplicitType.Output
  $SessionZImplicitTypeParsed = $SessionZImplicitType.Output | ConvertFrom-Json
  $SessionZImplicitRows = @($SessionZImplicitTypeParsed.typed_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0901' })
  if ($SessionZImplicitRows.Count -ne 1 -or $SessionZImplicitRows[0].callee -ne 'stdout_write' -or $SessionZImplicitRows[0].callee_result_root -ne 'OutputError') { throw 'Session Z implicit output call must preserve the Session W H0901 doctrine' }
  $SessionZImplicitRun = Read-NativeChannelsWithExit 'run Session Z implicit output failure' $Hum @('run', $SessionZImplicit, '--allow', 'stdout.write')
  if ($SessionZImplicitRun.ExitCode -ne 1 -or $SessionZImplicitRun.Stdout -ne '' -or [regex]::Matches($SessionZImplicitRun.Stderr, 'diagnostic=H0901').Count -ne 1 -or $SessionZImplicitRun.Stderr.Contains('runtime trap')) { throw 'Session Z implicit output call must fail before the output adapter' }

  $SessionZEffect = Read-NativeOutput 'effect check Session Z output policy' $Hum @('effect-check', '--format', 'json', $SessionZPositive)
  Assert-Json 'effect check Session Z output policy' $SessionZEffect
  $SessionZEffectParsed = $SessionZEffect | ConvertFrom-Json
  $SessionZOutputRow = @($SessionZEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq 'accepted_declared_output_operation_v0' })[0]
  if ($null -eq $SessionZOutputRow -or $SessionZOutputRow.capability_id -ne 'stdout.write' -or $SessionZOutputRow.core_effect -ne 'output' -or $SessionZOutputRow.grant_lifetime -ne 'one_run' -or $SessionZOutputRow.mapping_status -ne 'implemented_bounded_output_v0_reserved_os.stdio_mapping' -or @($SessionZOutputRow.route_tasks).Count -ne 3 -or $SessionZOutputRow.route_tasks[0] -ne 'bounded_stdout_probe' -or $SessionZOutputRow.route_tasks[1] -ne 'run_tool' -or $SessionZOutputRow.route_tasks[2] -ne 'emit' -or @($SessionZOutputRow.route_spans).Count -ne 2) { throw 'Session Z output policy row is missing the complete source and forensic route' }

  $SessionZHelp = Read-NativeOutput 'Session Z help text' $Hum @('--help')
  if (-not $SessionZHelp.Contains('--allow stdout.write') -or -not $SessionZHelp.Contains('--deny stdout.write')) { throw 'Session Z help must document exact output grant and deny flags' }

  $SessionAAPositive = 'examples/probes/runner_replay_clock.hum'
  foreach ($Command in @('resolve', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutput "Session AA $Command positive" $Hum @($Command, '--format', 'json', $SessionAAPositive)
    Assert-Json "Session AA $Command positive" $Surface
  }
  $SessionAAGraph = Read-NativeOutput 'Session AA graph positive' $Hum @('graph', $SessionAAPositive)
  Assert-Json 'Session AA graph positive' $SessionAAGraph

  $SessionAAFirst = Read-NativeChannelsWithExit 'run Session AA ordered replay first' $Hum @('run', $SessionAAPositive, '--allow', 'clock.replay', '--allow', 'stdout.write', '--replay-tick', '1', '--replay-tick', '7')
  $SessionAARepeat = Read-NativeChannelsWithExit 'run Session AA ordered replay repeat' $Hum @('run', $SessionAAPositive, '--allow', 'clock.replay', '--allow', 'stdout.write', '--replay-tick=1', '--replay-tick=7')
  if ($SessionAAFirst.ExitCode -ne 0 -or $SessionAAFirst.Stdout -ne 'seven' -or $SessionAAFirst.Stderr -ne '' -or $SessionAARepeat.ExitCode -ne $SessionAAFirst.ExitCode -or $SessionAARepeat.Stdout -ne $SessionAAFirst.Stdout -or $SessionAARepeat.Stderr -ne $SessionAAFirst.Stderr) { throw 'Session AA identical complete replay inputs must reproduce exact process channels' }
  $SessionAAChanged = Read-NativeChannelsWithExit 'run Session AA changed replay tick' $Hum @('run', $SessionAAPositive, '--allow', 'clock.replay', '--allow', 'stdout.write', '--replay-tick', '1', '--replay-tick', '8')
  if ($SessionAAChanged.ExitCode -ne 0 -or $SessionAAChanged.Stdout -ne 'other' -or $SessionAAChanged.Stderr -ne '') { throw 'Session AA changed second tick must select the other fixed literal' }

  foreach ($Denied in @(
    @{ Name = 'default deny despite supplied values'; Args = @('run', $SessionAAPositive, '--allow', 'stdout.write', '--replay-tick', '1', '--replay-tick', '7') },
    @{ Name = 'explicit deny overrides allow'; Args = @('run', $SessionAAPositive, '--allow', 'clock.replay', '--deny', 'clock.replay', '--allow', 'stdout.write', '--replay-tick', '1', '--replay-tick', '7') }
  )) {
    $DeniedRun = Read-NativeChannelsWithExit "run Session AA $($Denied.Name)" $Hum $Denied.Args
    if ($DeniedRun.ExitCode -ne 1 -or $DeniedRun.Stdout -ne '' -or -not $DeniedRun.Stderr.Contains('failure: ReplayAppError.replay') -or -not $DeniedRun.Stderr.Contains('caused by: ReplayClockError.denied') -or $DeniedRun.Stderr.Contains('ReplayClockError.exhausted') -or $DeniedRun.Stderr.Contains('runtime trap')) { throw "Session AA $($Denied.Name) must be typed denial before replay or output adapter access" }
  }

  $SessionAAExhausted = Read-NativeChannelsWithExit 'run Session AA exhausted replay sequence' $Hum @('run', $SessionAAPositive, '--allow', 'clock.replay', '--allow', 'stdout.write', '--replay-tick', '1')
  if ($SessionAAExhausted.ExitCode -ne 1 -or $SessionAAExhausted.Stdout -ne '' -or -not $SessionAAExhausted.Stderr.Contains('failure: ReplayAppError.replay') -or -not $SessionAAExhausted.Stderr.Contains('caused by: ReplayClockError.exhausted') -or -not $SessionAAExhausted.Stderr.Contains('while calling `read_tick`') -or -not $SessionAAExhausted.Stderr.Contains('originated at examples/probes/runner_replay_clock.hum:30:22') -or $SessionAAExhausted.Stderr.Contains('runtime trap')) { throw 'Session AA exhaustion must preserve the W-style outer-to-root causal chain' }

  $SessionAAMissing = 'fixtures/app_entry/session_aa_missing_replay_source_fail.hum'
  $SessionAAMissingHuman = Read-NativeOutputWithExit 'check Session AA missing replay source human' $Hum @('check', $SessionAAMissing)
  $SessionAAMissingJson = Read-NativeOutputWithExit 'check Session AA missing replay source JSON' $Hum @('check', '--format', 'json', $SessionAAMissing)
  if ($SessionAAMissingHuman.ExitCode -ne 1 -or [regex]::Matches($SessionAAMissingHuman.Output, 'error\[H0625\]').Count -ne 1 -or $SessionAAMissingJson.ExitCode -ne 1) { throw 'Session AA missing source authority must emit exactly one H0625' }
  Assert-Json 'check Session AA missing replay source JSON' $SessionAAMissingJson.Output
  $SessionAAMissingParsed = $SessionAAMissingJson.Output | ConvertFrom-Json
  $SessionAAMissingDiagnostic = @($SessionAAMissingParsed.diagnostics | Where-Object { $_.code -eq 'H0625' })[0]
  if ($null -eq $SessionAAMissingDiagnostic -or @($SessionAAMissingDiagnostic.related_spans).Count -ne 2 -or -not $SessionAAMissingHuman.Output.Contains($SessionAAMissingDiagnostic.message) -or -not $SessionAAMissingHuman.Output.Contains($SessionAAMissingDiagnostic.help)) { throw 'Session AA H0625 human/JSON call-task-app blame disagrees' }
  $SessionAAMissingEffect = Read-NativeOutputWithExit 'effect Session AA missing replay source' $Hum @('effect-check', '--format', 'json', $SessionAAMissing)
  Assert-Json 'effect Session AA missing replay source' $SessionAAMissingEffect.Output
  $SessionAAMissingEffectParsed = $SessionAAMissingEffect.Output | ConvertFrom-Json
  $SessionAAMissingRow = @($SessionAAMissingEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.diagnostic_code -eq 'H0625' })[0]
  if ($null -eq $SessionAAMissingRow -or $SessionAAMissingRow.status -ne 'rejected_missing_replay_source_authority_v0' -or $SessionAAMissingRow.core_effect -ne 'time' -or $null -eq $SessionAAMissingRow.app_span -or $null -eq $SessionAAMissingRow.caller_span -or @($SessionAAMissingRow.route_spans).Count -ne 1) { throw 'Session AA H0625 effect row must preserve the exact replay route and Core time fact' }
  $SessionAAMissingRun = Read-NativeChannelsWithExit 'run Session AA missing replay source' $Hum @('run', $SessionAAMissing, '--allow', 'clock.replay', '--replay-tick', '1')
  if ($SessionAAMissingRun.ExitCode -ne 1 -or $SessionAAMissingRun.Stdout -ne '' -or [regex]::Matches($SessionAAMissingRun.Stderr, 'error\[H0625\]').Count -ne 1 -or $SessionAAMissingRun.Stderr.Contains('runtime trap')) { throw 'Session AA H0625 must block before replay adapter execution' }

  $SessionAAMissingCaller = Read-NativeChannelsWithExit 'run Session AA replay authority laundering' $Hum @('run', 'fixtures/app_entry/session_aa_missing_replay_caller_fail.hum', '--allow', 'clock.replay', '--replay-tick', '1')
  if ($SessionAAMissingCaller.ExitCode -ne 1 -or $SessionAAMissingCaller.Stdout -ne '' -or [regex]::Matches($SessionAAMissingCaller.Stderr, 'error\[H0618\]').Count -ne 1 -or $SessionAAMissingCaller.Stderr.Contains('H0625') -or $SessionAAMissingCaller.Stderr.Contains('runtime trap')) { throw 'Session AA caller closure must prevent replay authority laundering under H0618' }

  $SessionAAInvalid = 'fixtures/full_type_check/session_aa_invalid_replay_call_fail.hum'
  $SessionAAInvalidHuman = Read-NativeOutputWithExit 'full type Session AA invalid replay call human' $Hum @('full-type-check', $SessionAAInvalid)
  $SessionAAInvalidType = Read-NativeOutputWithExit 'full type Session AA invalid replay call' $Hum @('full-type-check', '--format', 'json', $SessionAAInvalid)
  if ($SessionAAInvalidHuman.ExitCode -ne 1 -or $SessionAAInvalidType.ExitCode -ne 1) { throw 'Session AA invalid replay call must block full type human and JSON' }
  Assert-Json 'full type Session AA invalid replay call' $SessionAAInvalidType.Output
  $SessionAAInvalidParsed = $SessionAAInvalidType.Output | ConvertFrom-Json
  $SessionAAInvalidRow = @($SessionAAInvalidParsed.typed_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0626' })[0]
  if ($null -eq $SessionAAInvalidRow -or $SessionAAInvalidRow.expected_type -ne 'no arguments' -or $SessionAAInvalidRow.reason -ne 'clock_replay_tick_requires_zero_arguments_v0' -or $SessionAAInvalidType.Output.Contains('H0907') -or -not $SessionAAInvalidHuman.Output.Contains('diagnostic=H0626') -or -not $SessionAAInvalidHuman.Output.Contains($SessionAAInvalidRow.help)) { throw 'Session AA H0626 must agree in human/JSON, pin the exact zero-argument signature, and precede missing fails-when blame' }
  $SessionAAInvalidRun = Read-NativeChannelsWithExit 'run Session AA invalid replay call' $Hum @('run', $SessionAAInvalid, '--allow', 'clock.replay', '--replay-tick', '1')
  if ($SessionAAInvalidRun.ExitCode -ne 1 -or $SessionAAInvalidRun.Stdout -ne '' -or -not $SessionAAInvalidRun.Stderr.Contains('diagnostic=H0626') -or $SessionAAInvalidRun.Stderr.Contains('H0907') -or $SessionAAInvalidRun.Stderr.Contains('runtime trap')) { throw 'Session AA H0626 must block before replay adapter execution and missing fails-when blame' }

  $SessionAAImplicit = 'fixtures/full_type_check/session_aa_implicit_replay_fail.hum'
  $SessionAAImplicitType = Read-NativeOutputWithExit 'full type Session AA implicit replay failure' $Hum @('full-type-check', '--format', 'json', $SessionAAImplicit)
  if ($SessionAAImplicitType.ExitCode -ne 1) { throw 'Session AA implicit replay call must block full type' }
  Assert-Json 'full type Session AA implicit replay failure' $SessionAAImplicitType.Output
  $SessionAAImplicitParsed = $SessionAAImplicitType.Output | ConvertFrom-Json
  $SessionAAImplicitRows = @($SessionAAImplicitParsed.typed_items | ForEach-Object { $_.statements } | Where-Object { $_.diagnostic_code -eq 'H0901' })
  if ($SessionAAImplicitRows.Count -ne 1 -or $SessionAAImplicitRows[0].callee -ne 'clock_replay_tick' -or $SessionAAImplicitRows[0].callee_result_root -ne 'ReplayClockError') { throw 'Session AA implicit replay call must preserve the Session W H0901 doctrine' }
  $SessionAAImplicitRun = Read-NativeChannelsWithExit 'run Session AA implicit replay failure' $Hum @('run', $SessionAAImplicit, '--allow', 'clock.replay', '--replay-tick', '1')
  if ($SessionAAImplicitRun.ExitCode -ne 1 -or $SessionAAImplicitRun.Stdout -ne '' -or [regex]::Matches($SessionAAImplicitRun.Stderr, 'diagnostic=H0901').Count -ne 1 -or $SessionAAImplicitRun.Stderr.Contains('runtime trap')) { throw 'Session AA implicit replay call must fail before the replay adapter' }

  $SessionAAReserved = 'fixtures/app_entry/session_aa_reserved_replay_name_fail.hum'
  $SessionAAReservedHuman = Read-NativeOutputWithExit 'check Session AA reserved replay name' $Hum @('check', $SessionAAReserved)
  $SessionAAReservedJson = Read-NativeOutputWithExit 'check Session AA reserved replay name JSON' $Hum @('check', '--format', 'json', $SessionAAReserved)
  if ($SessionAAReservedHuman.ExitCode -ne 1 -or [regex]::Matches($SessionAAReservedHuman.Output, 'error\[H0627\]').Count -ne 1 -or $SessionAAReservedJson.ExitCode -ne 1) { throw 'Session AA reserved replay name must emit exactly one H0627' }
  Assert-Json 'check Session AA reserved replay name JSON' $SessionAAReservedJson.Output
  $SessionAAReservedParsed = $SessionAAReservedJson.Output | ConvertFrom-Json
  $SessionAAReservedDiagnostic = @($SessionAAReservedParsed.diagnostics | Where-Object { $_.code -eq 'H0627' })[0]
  if ($null -eq $SessionAAReservedDiagnostic -or -not $SessionAAReservedHuman.Output.Contains($SessionAAReservedDiagnostic.message) -or -not $SessionAAReservedHuman.Output.Contains($SessionAAReservedDiagnostic.help)) { throw 'Session AA H0627 human/JSON identity disagrees' }
  foreach ($Command in @('resolve', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutputWithExit "Session AA reserved replay name $Command" $Hum @($Command, '--format', 'json', $SessionAAReserved)
    if ($Surface.ExitCode -ne 1) { throw "Session AA reserved replay name must block $Command" }
    Assert-Json "Session AA reserved replay name $Command" $Surface.Output
  }
  $SessionAAReservedRun = Read-NativeChannelsWithExit 'run Session AA reserved replay name' $Hum @('run', $SessionAAReserved, '--allow', 'clock.replay', '--replay-tick', '1')
  if ($SessionAAReservedRun.ExitCode -ne 1 -or $SessionAAReservedRun.Stdout -ne '' -or [regex]::Matches($SessionAAReservedRun.Stderr, 'error\[H0627\]').Count -ne 1) { throw 'Session AA H0627 must prevent runtime callable substitution' }

  $SessionAARecursion = 'fixtures/app_entry/session_aa_replay_recursion_fail.hum'
  $SessionAARecursionHuman = Read-NativeOutputWithExit 'check Session AA replay recursion human' $Hum @('check', $SessionAARecursion)
  $SessionAARecursionJson = Read-NativeOutputWithExit 'check Session AA replay recursion JSON' $Hum @('check', '--format', 'json', $SessionAARecursion)
  if ($SessionAARecursionHuman.ExitCode -ne 1 -or [regex]::Matches($SessionAARecursionHuman.Output, 'error\[H0628\]').Count -ne 1 -or $SessionAARecursionJson.ExitCode -ne 1) { throw 'Session AA replay-reachable recursion must produce exactly one H0628' }
  Assert-Json 'check Session AA replay recursion JSON' $SessionAARecursionJson.Output
  $SessionAARecursionParsed = $SessionAARecursionJson.Output | ConvertFrom-Json
  $SessionAARecursionDiagnostic = @($SessionAARecursionParsed.diagnostics | Where-Object { $_.code -eq 'H0628' })[0]
  if ($null -eq $SessionAARecursionDiagnostic -or @($SessionAARecursionDiagnostic.related_spans).Count -lt 5) { throw 'Session AA H0628 must preserve the complete finite route-to-cycle blame' }
  $SessionAARecursionRun = Read-NativeChannelsWithExit 'run Session AA replay recursion' $Hum @('run', $SessionAARecursion, '--allow', 'clock.replay', '--replay-tick', '1')
  if ($SessionAARecursionRun.ExitCode -ne 1 -or $SessionAARecursionRun.Stdout -ne '' -or [regex]::Matches($SessionAARecursionRun.Stderr, 'error\[H0628\]').Count -ne 1 -or $SessionAARecursionRun.Stderr.Contains('runtime trap')) { throw 'Session AA H0628 must reject before replay or a generic runtime trap' }

  foreach ($Precedence in @(
    @{ Name = 'recursion plus missing source authority'; Path = 'fixtures/app_entry/session_aa_recursion_missing_source_precedence_fail.hum'; Code = 'H0625'; Status = 'rejected_missing_replay_source_authority_v0' },
    @{ Name = 'recursion plus missing caller authority'; Path = 'fixtures/app_entry/session_aa_recursion_missing_caller_precedence_fail.hum'; Code = 'H0618'; Status = 'rejected_missing_caller_capability_v0' }
  )) {
    $PrecedenceHuman = Read-NativeOutputWithExit "check Session AA $($Precedence.Name) human" $Hum @('check', $Precedence.Path)
    $PrecedenceJson = Read-NativeOutputWithExit "check Session AA $($Precedence.Name) JSON" $Hum @('check', '--format', 'json', $Precedence.Path)
    if ($PrecedenceHuman.ExitCode -ne 1 -or [regex]::Matches($PrecedenceHuman.Output, "error\[$($Precedence.Code)\]").Count -ne 1 -or $PrecedenceHuman.Output.Contains('H0628') -or $PrecedenceJson.ExitCode -ne 1) { throw "Session AA $($Precedence.Name) must have exactly one fundamental $($Precedence.Code) diagnostic" }
    Assert-Json "check Session AA $($Precedence.Name) JSON" $PrecedenceJson.Output
    $PrecedenceParsed = $PrecedenceJson.Output | ConvertFrom-Json
    $PrecedenceErrors = @($PrecedenceParsed.diagnostics | Where-Object { $_.severity -eq 'error' })
    if ($PrecedenceErrors.Count -ne 1 -or $PrecedenceErrors[0].code -ne $Precedence.Code) { throw "Session AA $($Precedence.Name) JSON ownership disagrees" }
    $PrecedenceEffect = Read-NativeOutputWithExit "effect Session AA $($Precedence.Name)" $Hum @('effect-check', '--format', 'json', $Precedence.Path)
    Assert-Json "effect Session AA $($Precedence.Name)" $PrecedenceEffect.Output
    $PrecedenceEffectParsed = $PrecedenceEffect.Output | ConvertFrom-Json
    $PrecedenceRejectedRows = @($PrecedenceEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $null -ne $_.diagnostic_code })
    if ($PrecedenceRejectedRows.Count -ne 1 -or $PrecedenceRejectedRows[0].diagnostic_code -ne $Precedence.Code -or $PrecedenceRejectedRows[0].status -ne $Precedence.Status) { throw "Session AA $($Precedence.Name) effect ownership disagrees" }
    $PrecedenceRun = Read-NativeChannelsWithExit "run Session AA $($Precedence.Name)" $Hum @('run', $Precedence.Path, '--allow', 'clock.replay', '--replay-tick', '1')
    if ($PrecedenceRun.ExitCode -ne 1 -or $PrecedenceRun.Stdout -ne '' -or [regex]::Matches($PrecedenceRun.Stderr, "error\[$($Precedence.Code)\]").Count -ne 1 -or $PrecedenceRun.Stderr.Contains('H0628') -or $PrecedenceRun.Stderr.Contains('runtime trap')) { throw "Session AA $($Precedence.Name) runtime-preflight ownership disagrees" }
  }

  $SessionAAEffect = Read-NativeOutput 'effect Session AA replay policy' $Hum @('effect-check', '--format', 'json', $SessionAAPositive)
  Assert-Json 'effect Session AA replay policy' $SessionAAEffect
  $SessionAAEffectParsed = $SessionAAEffect | ConvertFrom-Json
  $SessionAARows = @($SessionAAEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq 'accepted_declared_runner_replay_operation_v0' })
  if ($SessionAARows.Count -ne 2) { throw "Session AA expected two route-specific replay operation rows, got $($SessionAARows.Count)" }
  if (@($SessionAARows | Where-Object { $_.capability_id -ne 'clock.replay' -or $_.core_effect -ne 'time' -or $_.mapping_status -ne 'implemented_runner_replay_input_v0_no_os.clock' -or @($_.route_tasks).Count -ne 3 -or @($_.route_spans).Count -ne 2 }).Count -ne 0) { throw 'Session AA replay policy rows must preserve Core time, no-host-clock mapping, and complete routes' }
  if ($SessionAARows[0].id -eq $SessionAARows[1].id) { throw 'Session AA repeated replay calls must retain distinct stable route policy IDs' }

  $SessionAAEntry = Read-NativeChannelsWithExit 'run Session AA authority-bearing direct entry' $Hum @('run', $SessionAAPositive, '--entry', 'run_tool', '--allow', 'clock.replay', '--allow', 'stdout.write', '--replay-tick', '1', '--replay-tick', '7')
  if ($SessionAAEntry.ExitCode -ne 1 -or $SessionAAEntry.Stdout -ne '' -or [regex]::Matches($SessionAAEntry.Stderr, 'error\[H0620\]').Count -ne 1) { throw 'Session AA --entry must not bypass the structural app authority root' }

  $SessionAAInvalidTick = Read-NativeChannelsWithExit 'run Session AA invalid negative tick' $Hum @('run', $SessionAAPositive, '--replay-tick=-1')
  if ($SessionAAInvalidTick.ExitCode -ne 2 -or $SessionAAInvalidTick.Stdout -ne '' -or -not $SessionAAInvalidTick.Stderr.Contains('must not be negative')) { throw 'Session AA malformed replay input must remain a CLI usage error' }
  $SessionAATooManyArgs = @('run', $SessionAAPositive)
  foreach ($Index in 1..1025) { $SessionAATooManyArgs += '--replay-tick=1' }
  $SessionAATooMany = Read-NativeChannelsWithExit 'run Session AA replay input limit' $Hum $SessionAATooManyArgs
  if ($SessionAATooMany.ExitCode -ne 2 -or $SessionAATooMany.Stdout -ne '' -or -not $SessionAATooMany.Stderr.Contains('at most 1024')) { throw 'Session AA must reject more than 1024 replay ticks as CLI usage' }

  $SessionAAHelp = Read-NativeOutput 'Session AA help text' $Hum @('--help')
  $SessionAAUsage = '  hum run [--timings] [--allow stdout.write] [--deny stdout.write] [--allow clock.replay] [--deny clock.replay] [--allow files.read=<path>] [--deny files.read] [--replay-tick <UInt>]... <file> [--entry <task>] [--args ...]'
  if (-not $SessionAAHelp.Contains($SessionAAUsage)) { throw 'Session AA canonical Usage line must document exact output/replay consent and repeatable bounded runner input' }

  $SessionABPositive = 'examples/probes/opaque_native_path.hum'
  foreach ($Command in @('resolve', 'type-env', 'type-check', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutput "Session AB $Command positive" $Hum @($Command, '--format', 'json', $SessionABPositive)
    Assert-Json "Session AB $Command positive" $Surface
  }
  $SessionABGraph = Read-NativeOutput 'Session AB graph positive' $Hum @('graph', $SessionABPositive)
  Assert-Json 'Session AB graph positive' $SessionABGraph
  $SessionABTypeEnv = Read-NativeOutput 'Session AB reserved Path type' $Hum @('type-env', '--format', 'json', $SessionABPositive)
  if (-not $SessionABTypeEnv.Contains('"text": "Path"') -or -not $SessionABTypeEnv.Contains('"status": "reserved_type_v0"')) { throw 'Session AB Path must be a reserved opaque type root' }
  $SessionABProse = 'fixtures/full_type_check/session_af_path_trailing_prose_fail.hum'
  $SessionABProseHuman = Read-NativeOutputWithExit 'check Session AB comparison-shaped Path prose human' $Hum @('check', $SessionABProse)
  $SessionABProseJson = Read-NativeOutputWithExit 'check Session AB comparison-shaped Path prose JSON' $Hum @('check', '--format', 'json', $SessionABProse)
  if ($SessionABProseHuman.ExitCode -ne 0 -or $SessionABProseHuman.Output.Contains('H0630') -or $SessionABProseJson.ExitCode -ne 0) { throw 'Session AB comparison-shaped unchecked prose must remain statically unblocked without H0630' }
  Assert-Json 'check Session AB comparison-shaped Path prose JSON' $SessionABProseJson.Output
  $SessionABProseParsed = $SessionABProseJson.Output | ConvertFrom-Json
  $SessionABProseDiagnostics = @($SessionABProseParsed.diagnostics)
  if ($SessionABProseDiagnostics.Count -ne 0) { throw 'Session AB comparison-shaped prose must have no static source diagnostic' }
  foreach ($Command in @('resolve', 'type-env', 'type-check')) {
    $Surface = Read-NativeOutputWithExit "Session AB comparison-shaped prose $Command" $Hum @($Command, '--format', 'json', $SessionABProse)
    if ($Surface.ExitCode -ne 0 -or $Surface.Output.Contains('H0630')) { throw "Session AB comparison-shaped prose must remain unblocked in $Command" }
    Assert-Json "Session AB comparison-shaped prose $Command" $Surface.Output
  }
  foreach ($Command in @('core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutputWithExit "Session AF comparison-shaped Path malformed Core $Command" $Hum @($Command, '--format', 'json', $SessionABProse)
    if ($Surface.ExitCode -ne 0 -or -not $Surface.Output.Contains('blocked_contract_predicate_v2') -or $Surface.Output.Contains('H0630')) { throw "Session AF comparison-shaped Path malformed contract must preserve a Core blocker in $Command" }
    Assert-Json "Session AF comparison-shaped Path malformed Core $Command" $Surface.Output
  }
  foreach ($Command in @('full-type-check', 'effect-check', 'ownership-check', 'resource-check')) {
    $Surface = Read-NativeOutputWithExit "Session AF comparison-shaped Path malformed $Command" $Hum @($Command, '--format', 'json', $SessionABProse)
    if ($Surface.ExitCode -ne 1 -or $Surface.Output.Contains('H0630')) { throw "Session AF comparison-shaped Path malformed contract must block $Command without H0630" }
    Assert-Json "Session AF comparison-shaped Path malformed $Command" $Surface.Output
  }
  $SessionABSeparator = [string][char]92
  $SessionABMissingPath = 'C' + ':' + $SessionABSeparator + 'hum-session-ab' + $SessionABSeparator + 'missing.bin'

  foreach ($Misuse in @(
    @{ Name = 'multiple Path parameters'; Path = 'fixtures/app_entry/session_ab_multiple_path_parameters_fail.hum'; Code = 'H0629' },
    @{ Name = 'Text literal Path construction'; Path = 'fixtures/full_type_check/session_ab_text_literal_path_fail.hum'; Code = 'H0630' },
    @{ Name = 'Path source storage'; Path = 'fixtures/full_type_check/session_ab_path_storage_fail.hum'; Code = 'H0630' },
    @{ Name = 'Path contract comparison'; Path = 'fixtures/full_type_check/session_ab_path_contract_comparison_fail.hum'; Code = 'H0630' },
    @{ Name = 'test Path construction'; Path = 'fixtures/full_type_check/session_ab_test_path_construction_fail.hum'; Code = 'H0630' }
  )) {
    $Human = Read-NativeOutputWithExit "check Session AB $($Misuse.Name) human" $Hum @('check', $Misuse.Path)
    $Json = Read-NativeOutputWithExit "check Session AB $($Misuse.Name) JSON" $Hum @('check', '--format', 'json', $Misuse.Path)
    if ($Human.ExitCode -ne 1 -or [regex]::Matches($Human.Output, "error\[$($Misuse.Code)\]").Count -ne 1 -or $Json.ExitCode -ne 1) { throw "Session AB $($Misuse.Name) must produce exactly one $($Misuse.Code)" }
    Assert-Json "check Session AB $($Misuse.Name) JSON" $Json.Output
    $Parsed = $Json.Output | ConvertFrom-Json
    $Errors = @($Parsed.diagnostics | Where-Object { $_.severity -eq 'error' })
    if ($Errors.Count -ne 1 -or $Errors[0].code -ne $Misuse.Code -or -not $Human.Output.Contains($Errors[0].message) -or -not $Human.Output.Contains($Errors[0].help)) { throw "Session AB $($Misuse.Name) human/JSON ownership disagrees" }
    if ($Misuse.Code -eq 'H0629') {
      $FirstPath = @($Errors[0].related_spans | Where-Object { $_.label -eq 'first Path parameter' })
      if ($FirstPath.Count -ne 1 -or $FirstPath[0].span.line -ne $Errors[0].span.line -or $FirstPath[0].span.column -eq $Errors[0].span.column) { throw 'Session AB H0629 must blame distinct first and conflicting Path parameter columns' }
    }
    foreach ($Command in @('resolve', 'type-env', 'type-check', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
      $Blocked = Read-NativeOutputWithExit "Session AB $($Misuse.Name) $Command" $Hum @($Command, '--format', 'json', $Misuse.Path)
      if ($Blocked.ExitCode -ne 1) { throw "Session AB $($Misuse.Name) must block $Command" }
      Assert-Json "Session AB $($Misuse.Name) $Command" $Blocked.Output
    }
    $BlockedRun = Read-NativeChannelsWithExit "run Session AB $($Misuse.Name)" $Hum @('run', $Misuse.Path, '--args', $SessionABMissingPath)
    if ($BlockedRun.ExitCode -ne 1 -or $BlockedRun.Stdout -ne '' -or [regex]::Matches($BlockedRun.Stderr, "error\[$($Misuse.Code)\]").Count -ne 1 -or $BlockedRun.Stderr.Contains('runtime trap')) { throw "Session AB $($Misuse.Name) runtime preflight ownership disagrees" }
  }

  if ($env:OS -eq 'Windows_NT') {
    $SessionABArgument = 'C' + ':' + $SessionABSeparator + 'hum-session-ab' + $SessionABSeparator + 'definitely-missing.bin'
    $SessionABGrantPath = 'C' + ':' + $SessionABSeparator + 'hum-session-ab' + $SessionABSeparator + 'grant-missing.bin'
    $SessionABGrantFlag = '--allow=files.read=' + $SessionABGrantPath
    $SessionABRun = Read-NativeChannelsWithExit 'run Session AB opaque Path positive' $Hum @('run', $SessionABPositive, '--allow', 'stdout.write', '--args', $SessionABArgument)
    if ($SessionABRun.ExitCode -ne 0 -or $SessionABRun.Stdout -ne 'path accepted' -or $SessionABRun.Stderr -ne '') { throw 'Session AB positive must emit only the fixed literal without probing or displaying Path' }

    $SessionABProseRun = Read-NativeChannelsWithExit 'run Session AB comparison-shaped Path prose' $Hum @('run', $SessionABProse, '--args', $SessionABArgument)
    if ($SessionABProseRun.ExitCode -ne 2 -or $SessionABProseRun.Stdout -ne '' -or -not $SessionABProseRun.Stderr.Contains('H0704') -or $SessionABProseRun.Stderr.Contains('H0630') -or $SessionABProseRun.Stderr.Contains('runtime trap')) { throw 'Session AF comparison-shaped Path malformed contract must block as H0704 with exit 2 without inspecting Path' }

    $SessionABGrant = Read-NativeChannelsWithExit 'run Session AB nonexistent exact grant and deny' $Hum @('run', $SessionABPositive, '--allow', 'stdout.write', $SessionABGrantFlag, $SessionABGrantFlag, '--deny', 'files.read', '--args', $SessionABArgument)
    if ($SessionABGrant.ExitCode -ne 0 -or $SessionABGrant.Stdout -ne 'path accepted' -or $SessionABGrant.Stderr -ne '') { throw 'Session AB nonexistent exact grant must parse idempotently and deny without authorizing a host operation' }

    $SessionABFirstPath = 'C' + ':' + $SessionABSeparator + 'one' + $SessionABSeparator + 'missing.bin'
    $SessionABSecondPath = 'D' + ':' + $SessionABSeparator + 'two' + $SessionABSeparator + 'missing.bin'
    $SessionABDistinctGrant = Read-NativeChannelsWithExit 'run Session AB distinct file grants' $Hum @('run', $SessionABPositive, ('--allow=files.read=' + $SessionABFirstPath), ('--allow=files.read=' + $SessionABSecondPath), '--args', $SessionABFirstPath)
    if ($SessionABDistinctGrant.ExitCode -ne 2 -or $SessionABDistinctGrant.Stdout -ne '' -or -not $SessionABDistinctGrant.Stderr.Contains('at most one distinct')) { throw 'Session AB must reject two distinct files.read payloads as CLI usage' }

    foreach ($Rejected in @(
      @{ Name = 'relative'; Value = ('relative' + $SessionABSeparator + 'file'); Reason = 'not_ordinary_drive_letter_rooted_v0' },
      @{ Name = 'drive relative'; Value = 'C:file'; Reason = 'not_ordinary_drive_letter_rooted_v0' },
      @{ Name = 'traversal'; Value = ('C' + ':' + $SessionABSeparator + 'one' + $SessionABSeparator + '..' + $SessionABSeparator + 'file'); Reason = 'dot_or_dot_dot_component_forbidden_v0' },
      @{ Name = 'empty component'; Value = ('C' + ':' + $SessionABSeparator + 'one' + $SessionABSeparator + $SessionABSeparator + 'file'); Reason = 'empty_path_component_forbidden_v0' },
      @{ Name = 'UNC'; Value = ($SessionABSeparator + $SessionABSeparator + 'server' + $SessionABSeparator + 'share' + $SessionABSeparator + 'file'); Reason = 'windows_namespace_prefix_forbidden_v0' },
      @{ Name = 'verbatim'; Value = ($SessionABSeparator + $SessionABSeparator + '?' + $SessionABSeparator + 'C' + ':' + $SessionABSeparator + 'file'); Reason = 'windows_namespace_prefix_forbidden_v0' },
      @{ Name = 'device'; Value = ($SessionABSeparator + $SessionABSeparator + '.' + $SessionABSeparator + 'C' + ':' + $SessionABSeparator + 'file'); Reason = 'windows_namespace_prefix_forbidden_v0' },
      @{ Name = 'ADS'; Value = ('C' + ':' + $SessionABSeparator + 'one' + $SessionABSeparator + 'file:stream'); Reason = 'alternate_data_stream_or_extra_colon_forbidden_v0' },
      @{ Name = 'trailing dot'; Value = ('C' + ':' + $SessionABSeparator + 'one' + $SessionABSeparator + 'name.'); Reason = 'component_trailing_dot_or_space_forbidden_v0' },
      @{ Name = 'DOS device'; Value = ('C' + ':' + $SessionABSeparator + 'one' + $SessionABSeparator + 'CON.txt'); Reason = 'win32_dos_device_alias_forbidden_v0' }
    )) {
      $RejectedRun = Read-NativeChannelsWithExit "run Session AB rejected $($Rejected.Name) Path" $Hum @('run', $SessionABPositive, '--args', $Rejected.Value)
      if ($RejectedRun.ExitCode -ne 2 -or $RejectedRun.Stdout -ne '' -or -not $RejectedRun.Stderr.Contains($Rejected.Reason) -or -not $RejectedRun.Stderr.Contains('no host access was attempted')) { throw "Session AB rejected $($Rejected.Name) Path must fail before host access" }
    }

    $SessionABMultipleArgs = Read-NativeChannelsWithExit 'run Session AB multiple Path arguments' $Hum @('run', $SessionABPositive, '--args', $SessionABFirstPath, $SessionABSecondPath)
    if ($SessionABMultipleArgs.ExitCode -ne 2 -or $SessionABMultipleArgs.Stdout -ne '' -or -not $SessionABMultipleArgs.Stderr.Contains('expects 1 argument(s), got 2')) { throw 'Session AB must reject more than one runtime Path argument' }

    $SessionABDirect = Read-NativeChannelsWithExit 'run Session AB direct entry Path' $Hum @('run', 'fixtures/app_entry/session_ab_direct_entry_path_fail.hum', '--entry', 'run_tool', '--args', $SessionABMissingPath)
    if ($SessionABDirect.ExitCode -ne 2 -or $SessionABDirect.Stdout -ne '' -or -not $SessionABDirect.Stderr.Contains('constructed only by structural app entry')) { throw 'Session AB direct entry must not construct opaque Path' }
  }

  if (-not $SessionAAHelp.Contains('--allow files.read=<path>') -or -not $SessionAAHelp.Contains('--deny files.read')) { throw 'Session AB help must document the one exact native file grant and deny' }

  $SessionADPositive = 'examples/probes/exact_file_read.hum'
  foreach ($Command in @('resolve', 'type-env', 'type-check', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutput "Session AD $Command positive" $Hum @($Command, '--format', 'json', $SessionADPositive)
    Assert-Json "Session AD $Command positive" $Surface
  }
  $SessionADGraph = Read-NativeOutput 'Session AD graph positive' $Hum @('graph', $SessionADPositive)
  Assert-Json 'Session AD graph positive' $SessionADGraph

  $SessionADEffect = Read-NativeOutput 'Session AD file policy effect row' $Hum @('effect-check', '--format', 'json', $SessionADPositive)
  Assert-Json 'Session AD file policy effect row' $SessionADEffect
  $SessionADEffectParsed = $SessionADEffect | ConvertFrom-Json
  $SessionADFileRows = @($SessionADEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.status -eq 'accepted_declared_exact_file_read_operation_v0' })
  if ($SessionADFileRows.Count -ne 1 -or $SessionADFileRows[0].capability_id -ne 'files.read' -or $SessionADFileRows[0].core_effect -ne 'file' -or $SessionADFileRows[0].mapping_status -ne 'implemented_hardened_exact_file_read_v0_reserved_os.filesystem' -or @($SessionADFileRows[0].route_tasks).Count -ne 2 -or @($SessionADFileRows[0].route_spans).Count -ne 1) { throw 'Session AD file row must preserve Core file, exact mapping status, and complete route identity' }

  foreach ($Misuse in @(
    @{ Name = 'missing file source'; Path = 'fixtures/app_entry/session_ad_missing_file_source_fail.hum'; Surface = 'check'; Code = 'H0631' },
    @{ Name = 'wrong file argument type'; Path = 'fixtures/full_type_check/session_ad_file_read_wrong_type_fail.hum'; Surface = 'full-type-check'; Code = 'H0632' },
    @{ Name = 'reserved file builtin'; Path = 'fixtures/app_entry/session_ad_reserved_file_read_name_fail.hum'; Surface = 'check'; Code = 'H0633' }
  )) {
    $Human = Read-NativeOutputWithExit "Session AD $($Misuse.Name) human" $Hum @($Misuse.Surface, $Misuse.Path)
    $Json = Read-NativeOutputWithExit "Session AD $($Misuse.Name) JSON" $Hum @($Misuse.Surface, '--format', 'json', $Misuse.Path)
    if ($Human.ExitCode -ne 1 -or -not $Human.Output.Contains($Misuse.Code) -or $Json.ExitCode -ne 1 -or -not $Json.Output.Contains(('"diagnostic_code": "' + $Misuse.Code + '"')) -and -not $Json.Output.Contains(('"code": "' + $Misuse.Code + '"'))) { throw "Session AD $($Misuse.Name) must agree on $($Misuse.Code) in human and JSON" }
    Assert-Json "Session AD $($Misuse.Name) JSON" $Json.Output
  }

  $SessionADImplicit = Read-NativeOutputWithExit 'Session AD implicit file failure JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/session_ad_implicit_file_read_fail.hum')
  if ($SessionADImplicit.ExitCode -ne 1 -or -not $SessionADImplicit.Output.Contains('"diagnostic_code": "H0901"') -or -not $SessionADImplicit.Output.Contains('"callee": "files_read_text"') -or $SessionADImplicit.Output.Contains('H0630')) { throw 'Session AD implicit file call must preserve H0901 without Path-boundary masking' }
  Assert-Json 'Session AD implicit file failure JSON' $SessionADImplicit.Output

  if ($env:OS -eq 'Windows_NT') {
    $SessionADPath = (Resolve-Path 'fixtures/file_read/session_ad_utf8.txt').Path
    $SessionADGrant = '--allow=files.read=' + $SessionADPath
    foreach ($Denied in @(
      @{ Name = 'default deny'; Args = @('run', $SessionADPositive, '--allow', 'stdout.write', '--args', $SessionADPath) },
      @{ Name = 'exact deny'; Args = @('run', $SessionADPositive, '--allow', 'stdout.write', $SessionADGrant, '--deny', 'files.read', '--args', $SessionADPath) }
    )) {
      $Run = Read-NativeChannelsWithExit "Session AD $($Denied.Name)" $Hum $Denied.Args
      if ($Run.ExitCode -ne 1 -or $Run.Stdout -ne '' -or -not $Run.Stderr.Contains('failure: FileAppError.file') -or -not $Run.Stderr.Contains('caused by: FileReadError.denied') -or -not $Run.Stderr.Contains('while calling `files_read_text`') -or $Run.Stderr.Contains('runtime trap')) { throw "Session AD $($Denied.Name) must be causal typed denial before candidate access" }
    }

    $SessionADOtherPath = [System.IO.Path]::Combine([System.IO.Path]::GetDirectoryName($SessionADPath), 'different.txt')
    $SessionADOutside = Read-NativeChannelsWithExit 'Session AD outside exact grant' $Hum @('run', $SessionADPositive, '--allow', 'stdout.write', ('--allow=files.read=' + $SessionADOtherPath), '--args', $SessionADPath)
    if ($SessionADOutside.ExitCode -ne 1 -or $SessionADOutside.Stdout -ne '' -or -not $SessionADOutside.Stderr.Contains('caused by: FileReadError.outside_grant') -or $SessionADOutside.Stderr.Contains('runtime trap')) { throw 'Session AD a different exact native grant must fail outside authority before candidate access' }

    $SessionADHosted = Read-NativeChannelsWithExit 'Session AD hosted exact file' $Hum @('run', $SessionADPositive, '--allow', 'stdout.write', $SessionADGrant, '--args', $SessionADPath)
    if ($SessionADHosted.ExitCode -eq 0) {
      if (-not $SessionADHosted.Stdout.Contains('Hum reads exact UTF-8: lambda=') -or $SessionADHosted.Stderr -ne '') { throw 'Session AD fixed-local hosted success must emit only the checked fixture bytes' }
    } elseif ($SessionADHosted.ExitCode -ne 1 -or $SessionADHosted.Stdout -ne '' -or -not $SessionADHosted.Stderr.Contains('caused by: FileReadError.unavailable') -or $SessionADHosted.Stderr.Contains('runtime trap')) {
      throw 'Session AD hosted drive may only succeed under fixed_local_v0 or fail closed as typed unavailable'
    }
  }

  $SessionAEPositive = 'examples/probes/integrated_local_app.hum'
  foreach ($Command in @('resolve', 'type-env', 'type-check', 'full-type-check', 'effect-check', 'ownership-check', 'resource-check', 'core-preview', 'core-lower', 'core-verify')) {
    $Surface = Read-NativeOutput "Session AE $Command integrated app" $Hum @($Command, '--format', 'json', $SessionAEPositive)
    Assert-Json "Session AE $Command integrated app" $Surface
  }
  $SessionAEGraph = Read-NativeOutput 'Session AE graph integrated app' $Hum @('graph', $SessionAEPositive)
  Assert-Json 'Session AE graph integrated app' $SessionAEGraph

  $SessionAEEffect = Read-NativeOutput 'Session AE composed capability rows' $Hum @('effect-check', '--format', 'json', $SessionAEPositive)
  Assert-Json 'Session AE composed capability rows' $SessionAEEffect
  $SessionAEEffectParsed = $SessionAEEffect | ConvertFrom-Json
  $SessionAEOperations = @($SessionAEEffectParsed.effect_items | ForEach-Object { $_.boundary_checks } | Where-Object { $_.check -in @('source_capability_output_operation', 'source_capability_replay_operation', 'source_capability_file_operation') })
  foreach ($Capability in @('files.read', 'clock.replay')) {
    $Rows = @($SessionAEOperations | Where-Object { $_.capability_id -eq $Capability })
    if ($Rows.Count -ne 1 -or @($Rows[0].route_tasks).Count -ne 2 -or $Rows[0].route_tasks[0] -ne 'integrated_local_app' -or $Rows[0].route_tasks[1] -ne 'run_tool') { throw "Session AE $Capability must preserve one complete integrated source-policy route" }
  }
  $SessionAEOutputRows = @($SessionAEOperations | Where-Object { $_.capability_id -eq 'stdout.write' })
  if ($SessionAEOutputRows.Count -ne 3 -or @($SessionAEOutputRows | Select-Object -ExpandProperty id -Unique).Count -ne 3) { throw 'Session AE must preserve three distinct bounded-output call occurrences' }

  $SessionVProgram8 = Read-NativeOutputWithExit 'Session AE Program 8 pinned overlap honesty lock' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_program8_overlap_write_fail.hum')
  if ($SessionVProgram8.ExitCode -ne 1 -or -not $SessionVProgram8.Output.Contains('"diagnostic_code": "H0808"')) { throw 'Session AE may count Program 8 only while the exact Session V H0808 misuse remains green' }
  Assert-Json 'Session AE Program 8 pinned overlap honesty lock' $SessionVProgram8.Output

  $CheckJson = Read-NativeOutput 'check JSON' $Hum @('check', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'check JSON' $CheckJson
  if (-not $CheckJson.Contains('"schema": "hum.check.v0"')) { throw 'check JSON is missing hum.check.v0 schema' }

  $EvidenceJson = Read-NativeOutput 'evidence report JSON' $Hum @('evidence', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'evidence report JSON' $EvidenceJson
  if (-not $EvidenceJson.Contains('"schema": "hum.evidence.v0"')) { throw 'evidence report JSON is missing hum.evidence.v0 schema' }
  if (-not $EvidenceJson.Contains('"linked_evidence"')) { throw 'evidence report JSON is missing linked_evidence' }
  if (-not $EvidenceJson.Contains('"verification_status": "linked"')) { throw 'evidence report JSON is missing linked evidence status' }

  $MathObligationsJson = Read-NativeOutput 'math obligations JSON' $Hum @('math-obligations', '--format', 'json', 'examples/control_flow.hum')
  Assert-Json 'math obligations JSON' $MathObligationsJson
  if (-not $MathObligationsJson.Contains('"schema": "hum.math_obligations.v0"')) { throw 'math obligations JSON is missing hum.math_obligations.v0 schema' }
  if (-not $MathObligationsJson.Contains('"schema_version": "hum.math_obligation.v0"')) { throw 'math obligations JSON is missing hum.math_obligation.v0 entries' }
  if (-not $MathObligationsJson.Contains('"obligation_kind": "allocation_freedom"')) { throw 'math obligations JSON is missing allocation_freedom obligation' }
  if (-not $MathObligationsJson.Contains('"confidence_requested": "evidence_only"')) { throw 'math obligations JSON must keep V0 exports evidence_only' }

  $ResourceReportJson = Read-NativeOutput 'resource report JSON' $Hum @('resource-report', '--format', 'json', 'examples/control_flow.hum')
  Assert-Json 'resource report JSON' $ResourceReportJson
  if (-not $ResourceReportJson.Contains('"schema": "hum.resource_report.v0"')) { throw 'resource report JSON is missing hum.resource_report.v0 schema' }
  if (-not $ResourceReportJson.Contains('"claim_kind": "time_complexity"')) { throw 'resource report JSON is missing time_complexity claims' }
  if (-not $ResourceReportJson.Contains('"verification_status": "declared"')) { throw 'resource report JSON must keep V0 claims declared' }
  if (-not $ResourceReportJson.Contains('"proof_status": "not_proven"')) { throw 'resource report JSON must not pretend proofs exist' }
  if (-not $ResourceReportJson.Contains('"benchmark_status": "not_measured"')) { throw 'resource report JSON must not pretend benchmarks exist' }

  $CorePreviewJson = Read-NativeOutput 'Core preview JSON' $Hum @('core-preview', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'Core preview JSON' $CorePreviewJson
  if (-not $CorePreviewJson.Contains('"schema": "hum.core_preview.v0"')) { throw 'Core preview JSON is missing hum.core_preview.v0 schema' }
  if (-not $CorePreviewJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'Core preview JSON is missing Core Hum contract schema' }
  if (-not $CorePreviewJson.Contains('"type_check_schema": "hum.type_check.v0"')) { throw 'Core preview JSON is missing type check schema provenance' }
  if (-not $CorePreviewJson.Contains('"execution_ready": 0')) { throw 'Core preview JSON must not claim execution readiness' }
  if (-not $CorePreviewJson.Contains('"core_operation": "return"')) { throw 'Core preview JSON is missing return operation preview' }
  if (-not $CorePreviewJson.Contains('"core_operation": "store_write_deferred"')) { throw 'Core preview JSON is missing deferred store write blocker' }
  if (-not $CorePreviewJson.Contains('"expression_previews"')) { throw 'Core preview JSON is missing expression preview counts' }
  if (-not $CorePreviewJson.Contains('"name_status": "name_preview_v0"')) { throw 'Core preview JSON is missing name preview status' }
  if (-not $CorePreviewJson.Contains('"name_preview"')) { throw 'Core preview JSON is missing name_preview facts' }
  if (-not $CorePreviewJson.Contains('"scope_model": "lexical_block_scope_preview_v0"')) { throw 'Core preview JSON is missing lexical block scope model' }
  if (-not $CorePreviewJson.Contains('"checked_resolver_status": "not_run_v0"')) { throw 'Core preview JSON must keep checked resolver status separate from preview facts' }
  if (-not $CorePreviewJson.Contains('"resolver_diagnostic_status": "preview_facts_only_v0"')) { throw 'Core preview JSON must mark name facts as non-diagnostic preview facts' }
  if (-not $CorePreviewJson.Contains('"resolver_diagnostic_count": 0')) { throw 'Core preview JSON must not emit checked resolver diagnostics yet' }
  if (-not $CorePreviewJson.Contains('"scope_count"')) { throw 'Core preview JSON is missing name scope counts' }
  if (-not $CorePreviewJson.Contains('"scopes"')) { throw 'Core preview JSON is missing name scope list' }
  if (-not $CorePreviewJson.Contains('"scope_kind": "root"')) { throw 'Core preview JSON is missing root name scope' }
  if (-not $CorePreviewJson.Contains('"scope_kind": "if_statement"')) { throw 'Core preview JSON is missing if statement name scope' }
  if (-not $CorePreviewJson.Contains('"parent_scope_id"')) { throw 'Core preview JSON is missing parent scope links' }
  if (-not $CorePreviewJson.Contains('"definition_kind": "parameter"')) { throw 'Core preview JSON is missing parameter definition preview' }
  if (-not $CorePreviewJson.Contains('"definition_kind": "let_binding"')) { throw 'Core preview JSON is missing let binding definition preview' }
  if (-not $CorePreviewJson.Contains('"reference_kind": "name_ref"')) { throw 'Core preview JSON is missing name reference preview' }
  if (-not $CorePreviewJson.Contains('"resolution_status": "resolved_preview_v0"')) { throw 'Core preview JSON is missing resolved name reference preview' }
  if (-not $CorePreviewJson.Contains('"resolution_status": "external_reference_preview_v0"')) { throw 'Core preview JSON is missing external name reference preview' }
  if (-not $CorePreviewJson.Contains('"unresolved_name_references": 0')) { throw 'Core preview JSON should report zero unresolved name references for reference fixture' }
  if (-not $CorePreviewJson.Contains('global_or_type_name_resolution_not_implemented')) { throw 'Core preview JSON is missing external name resolution honesty reason' }
  if (-not $CorePreviewJson.Contains('"block_status": "block_preview_v0"')) { throw 'Core preview JSON is missing block preview status' }
  if (-not $CorePreviewJson.Contains('"block_preview"')) { throw 'Core preview JSON is missing block_preview tree' }
  if (-not $CorePreviewJson.Contains('"block_count"')) { throw 'Core preview JSON is missing block_count' }
  if (-not $CorePreviewJson.Contains('"max_block_depth"')) { throw 'Core preview JSON is missing max_block_depth summary' }
  if (-not $CorePreviewJson.Contains('"unmatched_block_closes": 0')) { throw 'Core preview JSON should report zero unmatched block closes for reference fixture' }
  if (-not $CorePreviewJson.Contains('"unclosed_blocks": 0')) { throw 'Core preview JSON should report zero unclosed blocks for reference fixture' }
  if (-not $CorePreviewJson.Contains('"node_kind": "block"')) { throw 'Core preview JSON is missing block nodes' }
  if (-not $CorePreviewJson.Contains('"node_kind": "statement_ref"')) { throw 'Core preview JSON is missing statement refs in block tree' }
  if (-not $CorePreviewJson.Contains('"block_kind": "record_construction"')) { throw 'Core preview JSON is missing record construction block preview' }
  if (-not $CorePreviewJson.Contains('"header_statement_index"')) { throw 'Core preview JSON is missing block header statement index' }
  if (-not $CorePreviewJson.Contains('"closing_statement_index"')) { throw 'Core preview JSON is missing block closing statement index' }
  if (-not $CorePreviewJson.Contains('"expression_preview"')) { throw 'Core preview JSON is missing statement expression previews' }
  if (-not $CorePreviewJson.Contains('"expression_ast_nodes"')) { throw 'Core preview JSON is missing expression AST node counts' }
  if (-not $CorePreviewJson.Contains('"atoms"')) { throw 'Core preview JSON is missing expression atoms' }
  if (-not $CorePreviewJson.Contains('"operators"')) { throw 'Core preview JSON is missing expression operators' }
  if (-not $CorePreviewJson.Contains('"ast"')) { throw 'Core preview JSON is missing expression AST previews' }
  if (-not $CorePreviewJson.Contains('"node_count"')) { throw 'Core preview JSON is missing expression AST node_count' }
  if (-not $CorePreviewJson.Contains('"form": "binary_operation_candidate"')) { throw 'Core preview JSON is missing binary operation AST form' }
  if (-not $CorePreviewJson.Contains('"typed_expression_previews"')) { throw 'Core preview JSON is missing typed expression preview counts' }
  if (-not $CorePreviewJson.Contains('"type_status": "not_type_checked_v0"')) { throw 'Core preview JSON must keep unchecked expression AST type slots where no checked return fact exists' }
  if (-not $CorePreviewJson.Contains('"type_status": "checked_trivial_return_type_v0"')) { throw 'Core preview JSON is missing checked return expression type status' }
  if (-not $CorePreviewJson.Contains('"type_text": "WorkItem"')) { throw 'Core preview JSON is missing checked return expression type text' }
  if (-not $CorePreviewJson.Contains('"type_source": "record_literal_constructor_v0"')) { throw 'Core preview JSON is missing checked return expression type source' }
  if (-not $CorePreviewJson.Contains('"effect_status": "not_effect_checked_v0"')) { throw 'Core preview JSON must keep expression AST effect status unchecked' }
  if (-not $CorePreviewJson.Contains('"status": "compound_preview_v0"')) { throw 'Core preview JSON is missing compound expression preview status' }
  if (-not $CorePreviewJson.Contains('"kind": "path_or_field_read"')) { throw 'Core preview JSON is missing path or field read expression kind' }
  if (-not $CorePreviewJson.Contains('surface_save_requires_store_lowering')) { throw 'Core preview JSON is missing store save lowering blocker' }
  if (-not $CorePreviewJson.Contains('no executable semantics')) { throw 'Core preview JSON must keep V0 non-execution claim' }
  if (-not $CorePreviewJson.Contains('no independent type checking')) { throw 'Core preview JSON must keep type-check provenance honesty claim' }
  if (-not $CorePreviewJson.Contains('no broad expression type checking')) { throw 'Core preview JSON must keep broad expression type-checking non-goal' }
  if (-not $CorePreviewJson.Contains('no module or global name resolution')) { throw 'Core preview JSON must keep V0 name-resolution non-goal' }
  if (-not $CorePreviewJson.Contains('no checked name resolution')) { throw 'Core preview JSON must keep V0 checked name-resolution non-goal' }

  $CoreLowerJson = Read-NativeOutput 'Core lower JSON' $Hum @('core-lower', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'Core lower JSON' $CoreLowerJson
  if (-not $CoreLowerJson.Contains('"schema": "hum.core_lower.v0"')) { throw 'Core lower JSON is missing hum.core_lower.v0 schema' }
  if (-not $CoreLowerJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'Core lower JSON is missing Core Hum contract schema' }
  if (-not $CoreLowerJson.Contains('"core_preview_schema": "hum.core_preview.v0"')) { throw 'Core lower JSON is missing Core preview schema provenance' }
  if (-not $CoreLowerJson.Contains('"resolve_schema": "hum.resolve.v0"')) { throw 'Core lower JSON is missing resolver schema provenance' }
  if (-not $CoreLowerJson.Contains('"type_check_schema": "hum.type_check.v0"')) { throw 'Core lower JSON is missing type check schema provenance' }
  if (-not $CoreLowerJson.Contains('"lowering_status": "unverified_core_artifact_v0"')) { throw 'Core lower JSON is missing unverified artifact status' }
  if (-not $CoreLowerJson.Contains('"verification_status": "unverified_v0"')) { throw 'Core lower JSON must mark artifacts unverified' }
  if (-not $CoreLowerJson.Contains('"execution_ready": 0')) { throw 'Core lower JSON must not claim execution readiness' }
  if (-not $CoreLowerJson.Contains('"ir_ready": 0')) { throw 'Core lower JSON must not claim IR readiness' }
  if (-not $CoreLowerJson.Contains('"core_operation": "return"')) { throw 'Core lower JSON is missing return operation' }
  if (-not $CoreLowerJson.Contains('surface_save_requires_store_lowering')) { throw 'Core lower JSON is missing store save lowering blocker' }
  if (-not $CoreLowerJson.Contains('no executable semantics')) { throw 'Core lower JSON must keep V0 non-execution claim' }
  if (-not $CoreLowerJson.Contains('no Hum IR emission')) { throw 'Core lower JSON must keep V0 non-IR-emission claim' }

  $CoreVerifyJson = Read-NativeOutput 'Core verify JSON' $Hum @('core-verify', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'Core verify JSON' $CoreVerifyJson
  if (-not $CoreVerifyJson.Contains('"schema": "hum.core_verify.v0"')) { throw 'Core verify JSON is missing hum.core_verify.v0 schema' }
  if (-not $CoreVerifyJson.Contains('"core_lower_schema": "hum.core_lower.v0"')) { throw 'Core verify JSON is missing core lower schema link' }
  if (-not $CoreVerifyJson.Contains('"verification_status": "verified_non_executing_core_artifact_v0"')) { throw 'Core verify JSON is missing verified non-executing status' }
  if (-not $CoreVerifyJson.Contains('"mode": "non_executing_artifact_invariant_check_v0"')) { throw 'Core verify JSON is missing invariant-check mode' }
  if (-not $CoreVerifyJson.Contains('"rule": "source_span_sane"')) { throw 'Core verify JSON is missing source span checks' }
  if (-not $CoreVerifyJson.Contains('"rule": "operation_family_status_consistent"')) { throw 'Core verify JSON is missing operation consistency checks' }
  if (-not $CoreVerifyJson.Contains('"failed_checks": 0')) { throw 'Core verify JSON should pass for reference fixture' }
  if (-not $CoreVerifyJson.Contains('"execution_ready": 0')) { throw 'Core verify JSON must not claim execution readiness' }
  if (-not $CoreVerifyJson.Contains('"ir_ready": 0')) { throw 'Core verify JSON must not claim IR readiness' }
  if (-not $CoreVerifyJson.Contains('no memory-safety proof')) { throw 'Core verify JSON must keep memory-safety non-claim' }

  $ResolveJson = Read-NativeOutput 'resolve JSON' $Hum @('resolve', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'resolve JSON' $ResolveJson
  if (-not $ResolveJson.Contains('"schema": "hum.resolve.v0"')) { throw 'resolve JSON is missing hum.resolve.v0 schema' }
  if (-not $ResolveJson.Contains('"mode": "source_analysis_only_no_type_or_borrow_check"')) { throw 'resolve JSON is missing source-analysis mode' }
  if (-not $ResolveJson.Contains('"status": "checked_resolver_v0"')) { throw 'resolve JSON should pass for reference fixture' }
  if (-not $ResolveJson.Contains('"resolver_errors": 0')) { throw 'resolve JSON should have zero resolver errors for reference fixture' }
  if (-not $ResolveJson.Contains('"scopes"')) { throw 'resolve JSON is missing scopes' }
  if (-not $ResolveJson.Contains('"definitions"')) { throw 'resolve JSON is missing definitions' }
  if (-not $ResolveJson.Contains('"references"')) { throw 'resolve JSON is missing references' }
  if (-not $ResolveJson.Contains('"definition_kind": "store"')) { throw 'resolve JSON is missing store definition' }
  if (-not $ResolveJson.Contains('"definition_kind": "declared_change_permission"')) { throw 'resolve JSON is missing declared change permission' }
  if (-not $ResolveJson.Contains('"reference_kind": "declared_change"')) { throw 'resolve JSON is missing declared change reference' }
  if (-not $ResolveJson.Contains('"reference_kind": "store_write_target"')) { throw 'resolve JSON is missing store write target reference' }
  if (-not $ResolveJson.Contains('"resolution_status": "resolved_v0"')) { throw 'resolve JSON is missing resolved references' }
  if (-not $ResolveJson.Contains('"duplicate_definitions": 0')) { throw 'resolve JSON should have zero duplicate definitions for reference fixture' }
  if (-not $ResolveJson.Contains('"no type checking"')) { throw 'resolve JSON must not claim type checking' }
  if (-not $ResolveJson.Contains('"no borrow checking"')) { throw 'resolve JSON must not claim borrow checking' }
  if (-not $ResolveJson.Contains('"no executable semantics"')) { throw 'resolve JSON must not claim execution' }

  $TypeEnvJson = Read-NativeOutput 'type environment JSON' $Hum @('type-env', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'type environment JSON' $TypeEnvJson
  if (-not $TypeEnvJson.Contains('"schema": "hum.type_env.v0"')) { throw 'type environment JSON is missing hum.type_env.v0 schema' }
  if (-not $TypeEnvJson.Contains('"mode": "declaration_inventory_no_type_check"')) { throw 'type environment JSON is missing declaration-inventory mode' }
  if (-not $TypeEnvJson.Contains('"resolver"')) { throw 'type environment JSON is missing resolver summary' }
  if (-not $TypeEnvJson.Contains('"schema": "hum.resolve.v0"')) { throw 'type environment JSON is missing resolver schema link' }
  if (-not $TypeEnvJson.Contains('"type_names"')) { throw 'type environment JSON is missing type_names' }
  if (-not $TypeEnvJson.Contains('"declarations"')) { throw 'type environment JSON is missing declarations' }
  if (-not $TypeEnvJson.Contains('"resolver_definition_id"')) { throw 'type environment JSON is missing resolver definition links' }
  if (-not $TypeEnvJson.Contains('"status": "type_environment_v0"')) { throw 'type environment JSON should pass for reference fixture' }
  if (-not $TypeEnvJson.Contains('"unknown_type_references": 0')) { throw 'type environment JSON should have zero unknown type references for reference fixture' }
  if (-not $TypeEnvJson.Contains('"no full type checking"')) { throw 'type environment JSON must not claim full type checking' }
  if (-not $TypeEnvJson.Contains('"no executable semantics"')) { throw 'type environment JSON must not claim execution' }

  $TypeCheckJson = Read-NativeOutput 'type check JSON' $Hum @('type-check', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'type check JSON' $TypeCheckJson
  if (-not $TypeCheckJson.Contains('"schema": "hum.type_check.v0"')) { throw 'type check JSON is missing hum.type_check.v0 schema' }
  if (-not $TypeCheckJson.Contains('"mode": "declaration_annotation_and_trivial_return_check_v0"')) { throw 'type check JSON is missing declaration-and-return mode' }
  if (-not $TypeCheckJson.Contains('"schema": "hum.type_env.v0"')) { throw 'type check JSON is missing type-env schema link' }
  if (-not $TypeCheckJson.Contains('"status": "declaration_annotations_and_trivial_returns_checked_v0"')) { throw 'type check JSON should pass for reference fixture' }
  if (-not $TypeCheckJson.Contains('"type_errors": 0')) { throw 'type check JSON should have zero type errors for reference fixture' }
  if (-not $TypeCheckJson.Contains('"checked_returns"')) { throw 'type check JSON is missing checked return rows' }
  if (-not $TypeCheckJson.Contains('"accepted_return_expression_v0"')) { throw 'type check JSON is missing accepted return check status' }
  if (-not $TypeCheckJson.Contains('"expected_value_type": "WorkItem"')) { throw 'type check JSON is missing Result success value type' }
  if (-not $TypeCheckJson.Contains('"accepted_type_reference_v0"')) { throw 'type check JSON is missing accepted type references' }
  if (-not $TypeCheckJson.Contains('"no full expression type inference"')) { throw 'type check JSON must not claim full expression inference' }
  if (-not $TypeCheckJson.Contains('"no executable semantics"')) { throw 'type check JSON must not claim execution' }

  $FullTypeCheckJson = Read-NativeOutput 'full type check JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/simple_pass.hum')
  Assert-Json 'full type check JSON' $FullTypeCheckJson
  if (-not $FullTypeCheckJson.Contains('"schema": "hum.full_type_check.v0"')) { throw 'full type check JSON is missing hum.full_type_check.v0 schema' }
  if (-not $FullTypeCheckJson.Contains('"status": "recognized_core_body_types_checked_v0"')) { throw 'full type check JSON should pass for simple fixture' }
  if (-not $FullTypeCheckJson.Contains('"mode": "recognized_core_body_type_gate_v0"')) { throw 'full type check JSON is missing recognized body type gate mode' }
  if (-not $FullTypeCheckJson.Contains('"blocking_issues": 0')) { throw 'full type check JSON should have zero blocking issues for simple fixture' }
  if (-not $FullTypeCheckJson.Contains('"accepted_statement_type_v0"')) { throw 'full type check JSON is missing accepted statement type facts' }
  if (-not $FullTypeCheckJson.Contains('"accepted_inferred_binding_type_v0"')) { throw 'full type check JSON is missing inferred binding type facts' }
  if (-not $FullTypeCheckJson.Contains('"execution_ready": 0')) { throw 'full type check JSON must not claim execution readiness' }
  if (-not $FullTypeCheckJson.Contains('"ir_ready": 0')) { throw 'full type check JSON must not claim IR readiness' }
  if (-not $FullTypeCheckJson.Contains('no memory-safety proof')) { throw 'full type check JSON must keep memory-safety non-claim' }
  $EffectCheckJson = Read-NativeOutput 'effect check JSON' $Hum @('effect-check', '--format', 'json', 'fixtures/effect_check/simple_pass.hum')
  Assert-Json 'effect check JSON' $EffectCheckJson
  if (-not $EffectCheckJson.Contains('"schema": "hum.effect_check.v0"')) { throw 'effect check JSON is missing hum.effect_check.v0 schema' }
  if (-not $EffectCheckJson.Contains('"status": "recognized_core_effects_checked_v0"')) { throw 'effect check JSON should pass for simple fixture' }
  if (-not $EffectCheckJson.Contains('"mode": "recognized_core_effect_gate_v0"')) { throw 'effect check JSON is missing recognized effect gate mode' }
  if (-not $EffectCheckJson.Contains('"blocking_issues": 0')) { throw 'effect check JSON should have zero blocking issues for simple fixture' }
  if (-not $EffectCheckJson.Contains('"accepted_declared_failure_v0"')) { throw 'effect check JSON is missing declared failure facts' }
  if (-not $EffectCheckJson.Contains('"accepted_local_mutation_v0"')) { throw 'effect check JSON is missing local mutation facts' }
  if (-not $EffectCheckJson.Contains('"execution_ready": 0')) { throw 'effect check JSON must not claim execution readiness' }
  if (-not $EffectCheckJson.Contains('"ir_ready": 0')) { throw 'effect check JSON must not claim IR readiness' }
  if (-not $EffectCheckJson.Contains('no memory-safety proof')) { throw 'effect check JSON must keep memory-safety non-claim' }

  $OwnershipCheckJson = Read-NativeOutput 'ownership check JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_consume_pass.hum')
  Assert-Json 'ownership check JSON' $OwnershipCheckJson
  if (-not $OwnershipCheckJson.Contains('"schema": "hum.ownership_check.v0"')) { throw 'ownership check JSON is missing hum.ownership_check.v0 schema' }
  if (-not $OwnershipCheckJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw 'ownership check JSON should pass for Session J consume fixture' }
  if (-not $OwnershipCheckJson.Contains('"accepted_consume_argument_move_v0"')) { throw 'ownership check JSON is missing accepted consume move fact' }
  if (-not $OwnershipCheckJson.Contains('"accepted_return_move_v0"')) { throw 'ownership check JSON is missing accepted return move fact' }
  if (-not $OwnershipCheckJson.Contains('"execution_ready": 0')) { throw 'ownership check JSON must not claim execution readiness' }
  if (-not $OwnershipCheckJson.Contains('"ir_ready": 0')) { throw 'ownership check JSON must not claim IR readiness' }
  if (-not $OwnershipCheckJson.Contains('no memory-safety proof')) { throw 'ownership check JSON must keep memory-safety non-claim' }

  $OwnershipCheckChangeJson = Read-NativeOutput 'ownership check Session J change JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_change_pass.hum')
  Assert-Json 'ownership check Session J change JSON' $OwnershipCheckChangeJson
  if (-not $OwnershipCheckChangeJson.Contains('"accepted_parameter_mutation_v0"')) { throw 'ownership check JSON is missing accepted parameter mutation fact' }

  $OwnershipUseAfterMoveJson = Read-NativeOutputWithExit 'ownership check Session J use-after-move JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_use_after_move_fail.hum')
  if ($OwnershipUseAfterMoveJson.ExitCode -ne 1) { throw "ownership check use-after-move expected exit 1, got $($OwnershipUseAfterMoveJson.ExitCode)" }
  Assert-Json 'ownership check Session J use-after-move JSON' $OwnershipUseAfterMoveJson.Output
  if (-not $OwnershipUseAfterMoveJson.Output.Contains('"status": "ownership_errors_v0"')) { throw "ownership check use-after-move expected ownership_errors_v0, got $($OwnershipUseAfterMoveJson.Output)" }
  if (-not $OwnershipUseAfterMoveJson.Output.Contains('"diagnostic_code": "H0801"')) { throw "ownership check use-after-move expected H0801, got $($OwnershipUseAfterMoveJson.Output)" }
  if (-not $OwnershipUseAfterMoveJson.Output.Contains('"help"')) { throw "ownership check use-after-move expected blame help, got $($OwnershipUseAfterMoveJson.Output)" }

  $OwnershipBorrowWriteJson = Read-NativeOutputWithExit 'ownership check Session J borrowed-write JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_borrow_write_fail.hum')
  if ($OwnershipBorrowWriteJson.ExitCode -ne 1) { throw "ownership check borrowed-write expected exit 1, got $($OwnershipBorrowWriteJson.ExitCode)" }
  Assert-Json 'ownership check Session J borrowed-write JSON' $OwnershipBorrowWriteJson.Output
  if (-not $OwnershipBorrowWriteJson.Output.Contains('"diagnostic_code": "H0802"')) { throw "ownership check borrowed-write expected H0802, got $($OwnershipBorrowWriteJson.Output)" }
  if (-not $OwnershipBorrowWriteJson.Output.Contains('"help"')) { throw "ownership check borrowed-write expected blame help, got $($OwnershipBorrowWriteJson.Output)" }

  $OwnershipDoubleConsumeJson = Read-NativeOutputWithExit 'ownership check Session J double-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_double_consume_fail.hum')
  if ($OwnershipDoubleConsumeJson.ExitCode -ne 1) { throw "ownership check double-consume expected exit 1, got $($OwnershipDoubleConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session J double-consume JSON' $OwnershipDoubleConsumeJson.Output
  if (-not $OwnershipDoubleConsumeJson.Output.Contains('"diagnostic_code": "H0801"')) { throw "ownership check double-consume expected H0801, got $($OwnershipDoubleConsumeJson.Output)" }

  $OwnershipTransactionOnceJson = Read-NativeOutput 'ownership check Session K transaction JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/transaction_once.hum')
  Assert-Json 'ownership check Session K transaction JSON' $OwnershipTransactionOnceJson
  if (-not $OwnershipTransactionOnceJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check transaction expected pass, got $OwnershipTransactionOnceJson" }
  if (-not $OwnershipTransactionOnceJson.Contains('"blocking_issues": 0')) { throw "ownership check transaction expected zero blockers, got $OwnershipTransactionOnceJson" }

  $OwnershipKMissingConsumeJson = Read-NativeOutputWithExit 'ownership check Session K missing-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_k_missing_consume_fail.hum')
  if ($OwnershipKMissingConsumeJson.ExitCode -ne 1) { throw "ownership check missing-consume expected exit 1, got $($OwnershipKMissingConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session K missing-consume JSON' $OwnershipKMissingConsumeJson.Output
  if (-not $OwnershipKMissingConsumeJson.Output.Contains('"diagnostic_code": "H0803"')) { throw "ownership check missing-consume expected H0803, got $($OwnershipKMissingConsumeJson.Output)" }
  if (-not $OwnershipKMissingConsumeJson.Output.Contains('if line 31 true')) { throw "ownership check missing-consume expected true path name, got $($OwnershipKMissingConsumeJson.Output)" }

  $OwnershipKDoubleConsumeJson = Read-NativeOutputWithExit 'ownership check Session K double-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_k_double_consume_fail.hum')
  if ($OwnershipKDoubleConsumeJson.ExitCode -ne 1) { throw "ownership check double-consume expected exit 1, got $($OwnershipKDoubleConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session K double-consume JSON' $OwnershipKDoubleConsumeJson.Output
  if (-not $OwnershipKDoubleConsumeJson.Output.Contains('"diagnostic_code": "H0804"')) { throw "ownership check double-consume expected H0804, got $($OwnershipKDoubleConsumeJson.Output)" }
  if (-not $OwnershipKDoubleConsumeJson.Output.Contains('already consumed by commit')) { throw "ownership check double-consume expected previous commit in help, got $($OwnershipKDoubleConsumeJson.Output)" }

  $OwnershipKBranchConsumeJson = Read-NativeOutputWithExit 'ownership check Session K branch-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_k_branch_consume_fail.hum')
  if ($OwnershipKBranchConsumeJson.ExitCode -ne 1) { throw "ownership check branch-consume expected exit 1, got $($OwnershipKBranchConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session K branch-consume JSON' $OwnershipKBranchConsumeJson.Output
  if (-not $OwnershipKBranchConsumeJson.Output.Contains('"diagnostic_code": "H0803"')) { throw "ownership check branch-consume expected H0803, got $($OwnershipKBranchConsumeJson.Output)" }
  if (-not $OwnershipKBranchConsumeJson.Output.Contains('if line 24 false')) { throw "ownership check branch-consume expected false path name, got $($OwnershipKBranchConsumeJson.Output)" }

  $OwnershipLParameterViewJson = Read-NativeOutput 'ownership check Session L parameter returned-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_l_return_parameter_view_pass.hum')
  Assert-Json 'ownership check Session L parameter returned-view JSON' $OwnershipLParameterViewJson
  if (-not $OwnershipLParameterViewJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check returned-view expected pass, got $OwnershipLParameterViewJson" }
  if (-not $OwnershipLParameterViewJson.Contains('"return_dependencies"')) { throw "ownership check returned-view is missing return_dependencies, got $OwnershipLParameterViewJson" }
  if (-not $OwnershipLParameterViewJson.Contains('"source": "text"')) { throw "ownership check returned-view expected source text, got $OwnershipLParameterViewJson" }
  if (-not $OwnershipLParameterViewJson.Contains('"status": "accepted_return_dependency_parameter_v0"')) { throw "ownership check returned-view expected accepted dependency, got $OwnershipLParameterViewJson" }

  $OwnershipLLocalViewJson = Read-NativeOutputWithExit 'ownership check Session L local returned-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_l_return_view_local_fail.hum')
  if ($OwnershipLLocalViewJson.ExitCode -ne 1) { throw "ownership check local returned-view expected exit 1, got $($OwnershipLLocalViewJson.ExitCode)" }
  Assert-Json 'ownership check Session L local returned-view JSON' $OwnershipLLocalViewJson.Output
  if (-not $OwnershipLLocalViewJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check local returned-view expected H0805, got $($OwnershipLLocalViewJson.Output)" }
  if (-not $OwnershipLLocalViewJson.Output.Contains('"status": "rejected_return_dependency_local_v0"')) { throw "ownership check local returned-view expected local rejection, got $($OwnershipLLocalViewJson.Output)" }

  $OwnershipLInternalViewJson = Read-NativeOutputWithExit 'ownership check Session L internal-reference returned-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_l_return_view_internal_fail.hum')
  if ($OwnershipLInternalViewJson.ExitCode -ne 1) { throw "ownership check internal-reference returned-view expected exit 1, got $($OwnershipLInternalViewJson.ExitCode)" }
  Assert-Json 'ownership check Session L internal-reference returned-view JSON' $OwnershipLInternalViewJson.Output
  if (-not $OwnershipLInternalViewJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check internal-reference returned-view expected H0805, got $($OwnershipLInternalViewJson.Output)" }
  if (-not $OwnershipLInternalViewJson.Output.Contains('"status": "rejected_return_dependency_internal_reference_v0"')) { throw "ownership check internal-reference returned-view expected internal-reference rejection, got $($OwnershipLInternalViewJson.Output)" }

  $OwnershipNFirstWordJson = Read-NativeOutput 'ownership check Session N first_word JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/first_word.hum')
  Assert-Json 'ownership check Session N first_word JSON' $OwnershipNFirstWordJson
  if (-not $OwnershipNFirstWordJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check first_word expected pass, got $OwnershipNFirstWordJson" }
  if (-not $OwnershipNFirstWordJson.Contains('"source": "text"')) { throw "ownership check first_word expected source text, got $OwnershipNFirstWordJson" }
  if (-not $OwnershipNFirstWordJson.Contains('"status": "accepted_return_dependency_closed_view_derivation_v0"')) { throw "ownership check first_word expected closed derivation acceptance, got $OwnershipNFirstWordJson" }

  $OwnershipNLocalSliceJson = Read-NativeOutputWithExit 'ownership check Session N local sub-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_n_return_view_local_slice_fail.hum')
  if ($OwnershipNLocalSliceJson.ExitCode -ne 1) { throw "ownership check local sub-view expected exit 1, got $($OwnershipNLocalSliceJson.ExitCode)" }
  Assert-Json 'ownership check Session N local sub-view JSON' $OwnershipNLocalSliceJson.Output
  if (-not $OwnershipNLocalSliceJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check local sub-view expected H0805, got $($OwnershipNLocalSliceJson.Output)" }
  if (-not $OwnershipNLocalSliceJson.Output.Contains('"status": "rejected_return_dependency_local_v0"')) { throw "ownership check local sub-view expected local rejection, got $($OwnershipNLocalSliceJson.Output)" }

  $OwnershipNLostProvenanceJson = Read-NativeOutputWithExit 'ownership check Session N lost-provenance JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_n_return_view_lost_provenance_fail.hum')
  if ($OwnershipNLostProvenanceJson.ExitCode -ne 1) { throw "ownership check lost-provenance expected exit 1, got $($OwnershipNLostProvenanceJson.ExitCode)" }
  Assert-Json 'ownership check Session N lost-provenance JSON' $OwnershipNLostProvenanceJson.Output
  if (-not $OwnershipNLostProvenanceJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check lost-provenance expected H0805, got $($OwnershipNLostProvenanceJson.Output)" }
  if (-not $OwnershipNLostProvenanceJson.Output.Contains('"reason": "returned_view_expression_not_closed_view_derivation_v0"')) { throw "ownership check lost-provenance expected non-closed reason, got $($OwnershipNLostProvenanceJson.Output)" }
  $OwnershipOFieldPlacesJson = Read-NativeOutput 'ownership check Session O field places JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/field_places.hum')
  Assert-Json 'ownership check Session O field places JSON' $OwnershipOFieldPlacesJson
  if (-not $OwnershipOFieldPlacesJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check field places expected pass, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"status": "accepted_disjoint_field_mutation_v0"')) { throw "ownership check field places expected disjoint field mutation acceptance, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"target": "point.x"')) { throw "ownership check field places expected point.x target, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"target": "point.y"')) { throw "ownership check field places expected point.y target, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"target": "item.done"')) { throw "ownership check field places expected item.done target, got $OwnershipOFieldPlacesJson" }

  $OwnershipOBorrowFieldJson = Read-NativeOutputWithExit 'ownership check Session O borrowed field-write JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_o_field_write_borrow_fail.hum')
  if ($OwnershipOBorrowFieldJson.ExitCode -ne 1) { throw "ownership check borrowed field-write expected exit 1, got $($OwnershipOBorrowFieldJson.ExitCode)" }
  Assert-Json 'ownership check Session O borrowed field-write JSON' $OwnershipOBorrowFieldJson.Output
  if (-not $OwnershipOBorrowFieldJson.Output.Contains('"diagnostic_code": "H0802"')) { throw "ownership check borrowed field-write expected H0802, got $($OwnershipOBorrowFieldJson.Output)" }
  if (-not $OwnershipOBorrowFieldJson.Output.Contains('"target": "point.x"')) { throw "ownership check borrowed field-write expected point.x target, got $($OwnershipOBorrowFieldJson.Output)" }
  if (-not $OwnershipOBorrowFieldJson.Output.Contains('writes through borrowed parameter')) { throw "ownership check borrowed field-write expected field blame help, got $($OwnershipOBorrowFieldJson.Output)" }

  $OwnershipRFieldViewsJson = Read-NativeOutput 'ownership check Session R field views JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/field_views.hum')
  Assert-Json 'ownership check Session R field views JSON' $OwnershipRFieldViewsJson
  if (-not $OwnershipRFieldViewsJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check Session R field views expected pass, got $OwnershipRFieldViewsJson" }
  if (-not $OwnershipRFieldViewsJson.Contains('"status": "accepted_field_view_borrow_v0"')) { throw "ownership check Session R field views expected field-view borrow acceptance, got $OwnershipRFieldViewsJson" }
  if (-not $OwnershipRFieldViewsJson.Contains('"declaration": "borrow flags.right"')) { throw "ownership check Session R field views expected flags.right declaration, got $OwnershipRFieldViewsJson" }
  if (-not $OwnershipRFieldViewsJson.Contains('"status": "accepted_disjoint_field_mutation_v0"')) { throw "ownership check Session R field views expected disjoint field mutation, got $OwnershipRFieldViewsJson" }

  $OwnershipRStalePointJson = Read-NativeOutputWithExit 'ownership check Session R stale point field-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_r_stale_point_field_view_fail.hum')
  if ($OwnershipRStalePointJson.ExitCode -ne 1) { throw "ownership check Session R stale point field-view expected exit 1, got $($OwnershipRStalePointJson.ExitCode)" }
  Assert-Json 'ownership check Session R stale point field-view JSON' $OwnershipRStalePointJson.Output
  if (-not $OwnershipRStalePointJson.Output.Contains('"diagnostic_code": "H0807"')) { throw "ownership check Session R stale point expected H0807, got $($OwnershipRStalePointJson.Output)" }
  if (-not $OwnershipRStalePointJson.Output.Contains('"status": "rejected_stale_field_view_use_v0"')) { throw "ownership check Session R stale point expected stale view rejection, got $($OwnershipRStalePointJson.Output)" }
  if (-not $OwnershipRStalePointJson.Output.Contains('x_view borrowed point.x')) { throw "ownership check Session R stale point expected binding and source in help, got $($OwnershipRStalePointJson.Output)" }
  if (-not $OwnershipRStalePointJson.Output.Contains('point.x was written')) { throw "ownership check Session R stale point expected invalidating write in help, got $($OwnershipRStalePointJson.Output)" }

  $OwnershipRStaleItemJson = Read-NativeOutputWithExit 'ownership check Session R stale item field-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_r_stale_item_field_view_fail.hum')
  if ($OwnershipRStaleItemJson.ExitCode -ne 1) { throw "ownership check Session R stale item field-view expected exit 1, got $($OwnershipRStaleItemJson.ExitCode)" }
  Assert-Json 'ownership check Session R stale item field-view JSON' $OwnershipRStaleItemJson.Output
  if (-not $OwnershipRStaleItemJson.Output.Contains('"diagnostic_code": "H0807"')) { throw "ownership check Session R stale item expected H0807, got $($OwnershipRStaleItemJson.Output)" }
  if (-not $OwnershipRStaleItemJson.Output.Contains('done_view borrowed item.done')) { throw "ownership check Session R stale item expected binding and source in help, got $($OwnershipRStaleItemJson.Output)" }
  if (-not $OwnershipRStaleItemJson.Output.Contains('item.done was written')) { throw "ownership check Session R stale item expected invalidating write in help, got $($OwnershipRStaleItemJson.Output)" }

  $OwnershipSElementViewsJson = Read-NativeOutput 'ownership check Session S element views JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/element_views.hum')
  Assert-Json 'ownership check Session S element views JSON' $OwnershipSElementViewsJson
  if (-not $OwnershipSElementViewsJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check Session S element views expected pass, got $OwnershipSElementViewsJson" }
  if (-not $OwnershipSElementViewsJson.Contains('"status": "accepted_element_view_borrow_v0"')) { throw "ownership check Session S element views expected element-view borrow acceptance, got $OwnershipSElementViewsJson" }
  if (-not $OwnershipSElementViewsJson.Contains('"declaration": "borrow items[0]"')) { throw "ownership check Session S element views expected items[0] declaration, got $OwnershipSElementViewsJson" }

  $OwnershipSStaleElementJson = Read-NativeOutputWithExit 'ownership check Session S stale element-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_s_stale_element_view_fail.hum')
  if ($OwnershipSStaleElementJson.ExitCode -ne 1) { throw "ownership check Session S stale element-view expected exit 1, got $($OwnershipSStaleElementJson.ExitCode)" }
  Assert-Json 'ownership check Session S stale element-view JSON' $OwnershipSStaleElementJson.Output
  if (-not $OwnershipSStaleElementJson.Output.Contains('"diagnostic_code": "H0807"')) { throw "ownership check Session S stale element expected H0807, got $($OwnershipSStaleElementJson.Output)" }
  if (-not $OwnershipSStaleElementJson.Output.Contains('"status": "rejected_stale_element_view_use_v0"')) { throw "ownership check Session S stale element expected stale element rejection, got $($OwnershipSStaleElementJson.Output)" }
  if (-not $OwnershipSStaleElementJson.Output.Contains('first_view borrowed items[0]')) { throw "ownership check Session S stale element expected binding and source in help, got $($OwnershipSStaleElementJson.Output)" }
  if (-not $OwnershipSStaleElementJson.Output.Contains('list_append grew items')) { throw "ownership check Session S stale element expected append site in help, got $($OwnershipSStaleElementJson.Output)" }

  $ResolveVJson = Read-NativeOutput 'resolve Session V writable aliases JSON' $Hum @('resolve', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'resolve Session V writable aliases JSON' $ResolveVJson
  if (-not $ResolveVJson.Contains('"definition_kind": "writable_field_alias"')) { throw 'Session V resolver output is missing writable_field_alias definition kind' }
  if (-not $ResolveVJson.Contains('"state_kind": "writable_field_alias"')) { throw 'Session V resolver output is missing writable_field_alias state kind' }
  if (-not $ResolveVJson.Contains('"mutable_place_errors": 0')) { throw 'Session V resolver output must accept set through the alias' }

  $FullTypeVJson = Read-NativeOutput 'full type check Session V writable aliases JSON' $Hum @('full-type-check', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'full type check Session V writable aliases JSON' $FullTypeVJson
  if (-not $FullTypeVJson.Contains('"expression_text": "change point.x"')) { throw 'Session V full type output is missing change point.x' }
  if (-not $FullTypeVJson.Contains('"actual_type": "UInt"')) { throw 'Session V full type output is missing alias field type' }
  if (-not $FullTypeVJson.Contains('"blocking_issues": 0')) { throw 'Session V full type output must pass' }

  $EffectVJson = Read-NativeOutput 'effect check Session V writable aliases JSON' $Hum @('effect-check', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'effect check Session V writable aliases JSON' $EffectVJson
  if (-not $EffectVJson.Contains('"effect_kind": "writable_field_alias"')) { throw 'Session V effect output is missing writable alias binding' }
  if (-not $EffectVJson.Contains('"effect_kind": "writable_field_alias_write_through"')) { throw 'Session V effect output is missing write-through change' }
  if (-not $EffectVJson.Contains('"target": "point.x"')) { throw 'Session V effect output must resolve alias writes to point.x' }

  $OwnershipVJson = Read-NativeOutput 'ownership check Session V writable aliases JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'ownership check Session V writable aliases JSON' $OwnershipVJson
  foreach ($Expected in @('"accepted_writable_field_alias_v0"', '"accepted_writable_field_alias_write_through_v0"', '"accepted_disjoint_field_mutation_v0"', '"alias": "alias_to_x"', '"place": "point.x"', '"binding_span": {', '"last_use_span": {')) {
    if (-not $OwnershipVJson.Contains($Expected)) { throw "Session V ownership pass output is missing $Expected" }
  }

  $OwnershipVWriteText = Read-NativeOutputWithExit 'ownership check Session V pinned overlap human' $Hum @('ownership-check', 'fixtures/ownership_check/session_v_program8_overlap_write_fail.hum')
  if ($OwnershipVWriteText.ExitCode -ne 1) { throw "Session V pinned overlap ownership human expected exit 1, got $($OwnershipVWriteText.ExitCode)" }
  foreach ($Expected in @('H0808', 'alias_to_x', 'point.x', 'binding_span=', 'last_use_span=', 'conflict_span=', 'not known independent', 'definitely distinct direct field')) {
    if (-not $OwnershipVWriteText.Output.Contains($Expected)) { throw "Session V pinned overlap ownership human is missing $Expected" }
  }

  $OwnershipVWriteJson = Read-NativeOutputWithExit 'ownership check Session V pinned overlap JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_program8_overlap_write_fail.hum')
  if ($OwnershipVWriteJson.ExitCode -ne 1) { throw "Session V pinned overlap ownership JSON expected exit 1, got $($OwnershipVWriteJson.ExitCode)" }
  Assert-Json 'ownership check Session V pinned overlap JSON' $OwnershipVWriteJson.Output
  foreach ($Expected in @('"diagnostic_code": "H0808"', '"status": "rejected_writable_field_alias_overlap_v0"', '"conflict_place": "point.x"', '"conflict_span": {', '"line": 13', '"line": 14', '"line": 15')) {
    if (-not $OwnershipVWriteJson.Output.Contains($Expected)) { throw "Session V pinned overlap ownership JSON is missing $Expected" }
  }

  $OwnershipVReadJson = Read-NativeOutputWithExit 'ownership check Session V overlapping read JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_overlap_read_fail.hum')
  if ($OwnershipVReadJson.ExitCode -ne 1) { throw "Session V overlapping read ownership JSON expected exit 1, got $($OwnershipVReadJson.ExitCode)" }
  if (-not $OwnershipVReadJson.Output.Contains('"reason": "direct_read_overlaps_live_writable_alias_v0"')) { throw 'Session V overlapping read ownership JSON is missing direct-read reason' }

  $OwnershipVSecondJson = Read-NativeOutputWithExit 'ownership check Session V second alias JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_second_alias_fail.hum')
  if ($OwnershipVSecondJson.ExitCode -ne 1) { throw "Session V second alias ownership JSON expected exit 1, got $($OwnershipVSecondJson.ExitCode)" }
  if (-not $OwnershipVSecondJson.Output.Contains('"reason": "second_writable_alias_overlaps_live_alias_v0"')) { throw 'Session V second alias ownership JSON is missing overlap reason' }

  $OwnershipVEscapeJson = Read-NativeOutputWithExit 'ownership check Session V alias escape JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_escape_fail.hum')
  if ($OwnershipVEscapeJson.ExitCode -ne 1) { throw "Session V alias escape ownership JSON expected exit 1, got $($OwnershipVEscapeJson.ExitCode)" }
  if (-not $OwnershipVEscapeJson.Output.Contains('"diagnostic_code": "H0809"')) { throw 'Session V alias escape ownership JSON is missing H0809' }
  if (-not $OwnershipVEscapeJson.Output.Contains('"reason": "writable_alias_escape_v0"')) { throw 'Session V alias escape ownership JSON is missing escape reason' }

  $FullTypeVAliasToAliasJson = Read-NativeOutput 'full type check Session V alias-to-alias JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_to_alias_fail.hum')
  Assert-Json 'full type check Session V alias-to-alias JSON' $FullTypeVAliasToAliasJson
  foreach ($Expected in @('"status": "accepted_writable_field_alias_candidate_deferred_to_ownership_v0"', '"reason": "writable_field_alias_shape_deferred_to_ownership_v0"', '"unchecked_statements": 0', '"blocking_issues": 0')) {
    if (-not $FullTypeVAliasToAliasJson.Contains($Expected)) { throw "Session V alias-to-alias full type JSON is missing $Expected" }
  }
  if ($FullTypeVAliasToAliasJson.Contains('blocked_by_unchecked_body_types_v0')) { throw 'Session V alias-to-alias must not be masked by full type checking' }

  $EffectVAliasToAliasJson = Read-NativeOutput 'effect check Session V alias-to-alias JSON' $Hum @('effect-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_to_alias_fail.hum')
  Assert-Json 'effect check Session V alias-to-alias JSON' $EffectVAliasToAliasJson
  if (-not $EffectVAliasToAliasJson.Contains('"status": "recognized_core_effects_checked_v0"')) { throw 'Session V alias-to-alias must reach ownership through effect checking' }
  if ($EffectVAliasToAliasJson.Contains('blocked_by_full_type_check_errors')) { throw 'Session V alias-to-alias must not be masked by effect checking' }

  $OwnershipVAliasToAliasText = Read-NativeOutputWithExit 'ownership check Session V alias-to-alias human' $Hum @('ownership-check', 'fixtures/ownership_check/session_v_alias_to_alias_fail.hum')
  if ($OwnershipVAliasToAliasText.ExitCode -ne 1) { throw "Session V alias-to-alias ownership human expected exit 1, got $($OwnershipVAliasToAliasText.ExitCode)" }
  foreach ($Expected in @('H0809', 'writable_alias_to_alias_binding_v0', 'writable alias `first`')) {
    if (-not $OwnershipVAliasToAliasText.Output.Contains($Expected)) { throw "Session V alias-to-alias ownership human is missing $Expected" }
  }

  $OwnershipVAliasToAliasJson = Read-NativeOutputWithExit 'ownership check Session V alias-to-alias JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_to_alias_fail.hum')
  if ($OwnershipVAliasToAliasJson.ExitCode -ne 1) { throw "Session V alias-to-alias ownership JSON expected exit 1, got $($OwnershipVAliasToAliasJson.ExitCode)" }
  Assert-Json 'ownership check Session V alias-to-alias JSON' $OwnershipVAliasToAliasJson.Output
  foreach ($Expected in @('"diagnostic_code": "H0809"', '"reason": "writable_alias_to_alias_binding_v0"', '"conflict_place": "first"')) {
    if (-not $OwnershipVAliasToAliasJson.Output.Contains($Expected)) { throw "Session V alias-to-alias ownership JSON is missing $Expected" }
  }

  $FullTypeVNestedAliasJson = Read-NativeOutput 'full type check Session V nested alias-place JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/ownership_check/session_v_nested_alias_place_fail.hum')
  Assert-Json 'full type check Session V nested alias-place JSON' $FullTypeVNestedAliasJson
  foreach ($Expected in @('"status": "accepted_writable_field_alias_candidate_deferred_to_ownership_v0"', '"reason": "writable_field_alias_shape_deferred_to_ownership_v0"', '"unchecked_statements": 0', '"blocking_issues": 0')) {
    if (-not $FullTypeVNestedAliasJson.Contains($Expected)) { throw "Session V nested alias-place full type JSON is missing $Expected" }
  }
  if ($FullTypeVNestedAliasJson.Contains('blocked_by_unchecked_body_types_v0')) { throw 'Session V nested alias place must not be masked by full type checking' }

  $EffectVNestedAliasJson = Read-NativeOutput 'effect check Session V nested alias-place JSON' $Hum @('effect-check', '--format', 'json', 'fixtures/ownership_check/session_v_nested_alias_place_fail.hum')
  Assert-Json 'effect check Session V nested alias-place JSON' $EffectVNestedAliasJson
  if (-not $EffectVNestedAliasJson.Contains('"status": "recognized_core_effects_checked_v0"')) { throw 'Session V nested alias place must reach ownership through effect checking' }
  if ($EffectVNestedAliasJson.Contains('blocked_by_full_type_check_errors')) { throw 'Session V nested alias place must not be masked by effect checking' }

  $OwnershipVNestedAliasText = Read-NativeOutputWithExit 'ownership check Session V nested alias-place human' $Hum @('ownership-check', 'fixtures/ownership_check/session_v_nested_alias_place_fail.hum')
  if ($OwnershipVNestedAliasText.ExitCode -ne 1) { throw "Session V nested alias-place ownership human expected exit 1, got $($OwnershipVNestedAliasText.ExitCode)" }
  foreach ($Expected in @('H0809', 'writable_alias_shape_outside_direct_field_slice_v0', 'point.x.deep')) {
    if (-not $OwnershipVNestedAliasText.Output.Contains($Expected)) { throw "Session V nested alias-place ownership human is missing $Expected" }
  }

  $OwnershipVNestedAliasJson = Read-NativeOutputWithExit 'ownership check Session V nested alias-place JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_nested_alias_place_fail.hum')
  if ($OwnershipVNestedAliasJson.ExitCode -ne 1) { throw "Session V nested alias-place ownership JSON expected exit 1, got $($OwnershipVNestedAliasJson.ExitCode)" }
  Assert-Json 'ownership check Session V nested alias-place JSON' $OwnershipVNestedAliasJson.Output
  foreach ($Expected in @('"diagnostic_code": "H0809"', '"reason": "writable_alias_shape_outside_direct_field_slice_v0"', '"place": "point.x.deep"')) {
    if (-not $OwnershipVNestedAliasJson.Output.Contains($Expected)) { throw "Session V nested alias-place ownership JSON is missing $Expected" }
  }

  $OwnershipVBorrowJson = Read-NativeOutputWithExit 'ownership check Session V borrowed owner JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_borrowed_owner_alias_fail.hum')
  if ($OwnershipVBorrowJson.ExitCode -ne 1) { throw "Session V borrowed owner ownership JSON expected exit 1, got $($OwnershipVBorrowJson.ExitCode)" }
  if (-not $OwnershipVBorrowJson.Output.Contains('"diagnostic_code": "H0802"')) { throw 'Session V borrowed owner ownership JSON is missing H0802' }

  $ResolveVRebindJson = Read-NativeOutput 'resolve Session V alias-owner rebinding JSON' $Hum @('resolve', '--format', 'json', 'fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum')
  Assert-Json 'resolve Session V alias-owner rebinding JSON' $ResolveVRebindJson
  if (-not $ResolveVRebindJson.Contains('"status": "duplicate_definition_deferred_to_ownership_v0"')) { throw 'Session V resolver rebinding output must defer duplicate blame to ownership' }
  if (-not $ResolveVRebindJson.Contains('"resolver_errors": 0')) { throw 'Session V resolver rebinding output must not mask H0809' }

  $OwnershipVPermissionJson = Read-NativeOutputWithExit 'ownership check Session V permission-wrapper JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_permission_wrapper_fail.hum')
  if ($OwnershipVPermissionJson.ExitCode -ne 1) { throw "Session V permission-wrapper ownership JSON expected exit 1, got $($OwnershipVPermissionJson.ExitCode)" }
  if (-not $OwnershipVPermissionJson.Output.Contains('"diagnostic_code": "H0809"')) { throw 'Session V permission-wrapper ownership JSON is missing H0809' }
  if (-not $OwnershipVPermissionJson.Output.Contains('writable_alias_permission_wrapper_v0')) { throw 'Session V permission-wrapper ownership JSON is missing its stable reason' }

  $OwnershipVRebindJson = Read-NativeOutputWithExit 'ownership check Session V alias-owner rebinding JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_rebind_owner_fail.hum')
  if ($OwnershipVRebindJson.ExitCode -ne 1) { throw "Session V alias-owner rebinding ownership JSON expected exit 1, got $($OwnershipVRebindJson.ExitCode)" }
  if (-not $OwnershipVRebindJson.Output.Contains('"diagnostic_code": "H0809"')) { throw 'Session V alias-owner rebinding ownership JSON is missing H0809' }
  if (-not $OwnershipVRebindJson.Output.Contains('writable_alias_rebinds_its_owner_v0')) { throw 'Session V alias-owner rebinding ownership JSON is missing its stable reason' }

  $ResolveVNameCollisionJson = Read-NativeOutput 'resolve Session V alias-name collision JSON' $Hum @('resolve', '--format', 'json', 'fixtures/ownership_check/session_v_alias_name_collision_fail.hum')
  Assert-Json 'resolve Session V alias-name collision JSON' $ResolveVNameCollisionJson
  if (-not $ResolveVNameCollisionJson.Contains('"status": "duplicate_definition_deferred_to_ownership_v0"')) { throw 'Session V resolver name-collision output must defer duplicate blame to ownership' }
  if (-not $ResolveVNameCollisionJson.Contains('"resolver_errors": 0')) { throw 'Session V resolver name-collision output must not mask H0809' }

  $OwnershipVNameCollisionJson = Read-NativeOutputWithExit 'ownership check Session V alias-name collision JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_name_collision_fail.hum')
  if ($OwnershipVNameCollisionJson.ExitCode -ne 1) { throw "Session V alias-name collision ownership JSON expected exit 1, got $($OwnershipVNameCollisionJson.ExitCode)" }
  if (-not $OwnershipVNameCollisionJson.Output.Contains('"diagnostic_code": "H0809"')) { throw 'Session V alias-name collision ownership JSON is missing H0809' }
  if (-not $OwnershipVNameCollisionJson.Output.Contains('writable_alias_binding_rebinding_v0')) { throw 'Session V alias-name collision ownership JSON is missing its stable reason' }

  $ResolveVDeclaredNameCollisionJson = Read-NativeOutput 'resolve Session V alias-declared-name collision JSON' $Hum @('resolve', '--format', 'json', 'fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum')
  Assert-Json 'resolve Session V alias-declared-name collision JSON' $ResolveVDeclaredNameCollisionJson
  if (-not $ResolveVDeclaredNameCollisionJson.Contains('"status": "duplicate_definition_deferred_to_ownership_v0"')) { throw 'Session V resolver declared-name collision output must defer duplicate blame to ownership' }
  if (-not $ResolveVDeclaredNameCollisionJson.Contains('"resolver_errors": 0')) { throw 'Session V resolver declared-name collision output must not mask H0809' }

  $OwnershipVDeclaredNameCollisionJson = Read-NativeOutputWithExit 'ownership check Session V alias-declared-name collision JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_alias_declared_name_collision_fail.hum')
  if ($OwnershipVDeclaredNameCollisionJson.ExitCode -ne 1) { throw "Session V alias-declared-name collision ownership JSON expected exit 1, got $($OwnershipVDeclaredNameCollisionJson.ExitCode)" }
  if (-not $OwnershipVDeclaredNameCollisionJson.Output.Contains('"diagnostic_code": "H0809"')) { throw 'Session V alias-declared-name collision ownership JSON is missing H0809' }
  if (-not $OwnershipVDeclaredNameCollisionJson.Output.Contains('writable_alias_binding_rebinding_v0')) { throw 'Session V alias-declared-name collision ownership JSON is missing its stable reason' }

  $OwnershipVBorrowOverlapJson = Read-NativeOutputWithExit 'ownership check Session V borrowed-owner overlap precedence JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_v_borrowed_owner_overlap_fail.hum')
  if ($OwnershipVBorrowOverlapJson.ExitCode -ne 1) { throw "Session V borrowed-owner overlap ownership JSON expected exit 1, got $($OwnershipVBorrowOverlapJson.ExitCode)" }
  if (-not $OwnershipVBorrowOverlapJson.Output.Contains('"diagnostic_code": "H0802"')) { throw 'Session V borrowed-owner overlap ownership JSON is missing H0802' }
  if ($OwnershipVBorrowOverlapJson.Output.Contains('"diagnostic_code": "H0808"')) { throw 'Session V borrowed-owner overlap ownership JSON must not report H0808' }

  $ResourceVJson = Read-NativeOutput 'resource check Session V writable aliases JSON' $Hum @('resource-check', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'resource check Session V writable aliases JSON' $ResourceVJson
  if (-not $ResourceVJson.Contains('"status": "recognized_core_resources_checked_v0"')) { throw 'Session V resource output must pass' }
  if (-not $ResourceVJson.Contains('"blocking_issues": 0')) { throw 'Session V resource output must have zero blockers' }

  $CorePreviewVJson = Read-NativeOutput 'core preview Session V writable aliases JSON' $Hum @('core-preview', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'core preview Session V writable aliases JSON' $CorePreviewVJson
  if (-not $CorePreviewVJson.Contains('let alias_to_x = change point.x')) { throw 'Session V Core preview must preserve the alias surface' }
  if (-not $CorePreviewVJson.Contains('"blocked_statements": 0')) { throw 'Session V Core preview must have zero blocked statements' }

  $CoreLowerVJson = Read-NativeOutput 'core lower Session V writable aliases JSON' $Hum @('core-lower', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'core lower Session V writable aliases JSON' $CoreLowerVJson
  if (-not $CoreLowerVJson.Contains('"blocked_operations": 0')) { throw 'Session V Core lower must have zero blocked operations' }
  if (-not $CoreLowerVJson.Contains('"surface_text": "let alias_to_x = change point.x"')) { throw 'Session V Core lower must preserve the alias surface text' }

  $CoreVerifyVJson = Read-NativeOutput 'core verify Session V writable aliases JSON' $Hum @('core-verify', '--format', 'json', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'core verify Session V writable aliases JSON' $CoreVerifyVJson
  if (-not $CoreVerifyVJson.Contains('"failed_checks": 0')) { throw 'Session V Core verify must pass every check' }

  $GraphVJson = Read-NativeOutput 'graph Session V writable aliases JSON' $Hum @('graph', 'examples/probes/writable_field_aliases.hum')
  Assert-Json 'graph Session V writable aliases JSON' $GraphVJson
  if (-not $GraphVJson.Contains('"errors": 0')) { throw 'Session V graph must have zero errors' }
  if (-not $GraphVJson.Contains('"warnings": 0')) { throw 'Session V graph must have zero warnings' }

  $ResourceCheckJson = Read-NativeOutput 'resource check JSON' $Hum @('resource-check', '--format', 'json', 'fixtures/resource_check/simple_pass.hum')
  Assert-Json 'resource check JSON' $ResourceCheckJson
  if (-not $ResourceCheckJson.Contains('"schema": "hum.resource_check.v0"')) { throw 'resource check JSON is missing hum.resource_check.v0 schema' }
  if (-not $ResourceCheckJson.Contains('"status": "recognized_core_resources_checked_v0"')) { throw 'resource check JSON should pass for simple fixture' }
  if (-not $ResourceCheckJson.Contains('"mode": "recognized_core_resource_gate_v0"')) { throw 'resource check JSON is missing resource gate mode' }
  if (-not $ResourceCheckJson.Contains('"ownership_check_schema": "hum.ownership_check.v0"')) { throw 'resource check JSON is missing ownership-check dependency schema' }
  if (-not $ResourceCheckJson.Contains('"resource_report_schema": "hum.resource_report.v0"')) { throw 'resource check JSON is missing resource-report dependency schema' }
  if (-not $ResourceCheckJson.Contains('"blocking_issues": 0')) { throw 'resource check JSON should have zero blocking issues for simple fixture' }
  if (-not $ResourceCheckJson.Contains('"accepted_conservative_allocation_free_claim_v0"')) { throw 'resource check JSON is missing accepted allocation-free fact' }
  if (-not $ResourceCheckJson.Contains('"proof_ready": 0')) { throw 'resource check JSON must not claim proof readiness' }
  if (-not $ResourceCheckJson.Contains('"execution_ready": 0')) { throw 'resource check JSON must not claim execution readiness' }
  if (-not $ResourceCheckJson.Contains('"ir_ready": 0')) { throw 'resource check JSON must not claim IR readiness' }
  if (-not $ResourceCheckJson.Contains('no allocation-freedom proof')) { throw 'resource check JSON must keep allocation-proof non-claim' }
  if (-not $ResourceCheckJson.Contains('no memory-safety proof')) { throw 'resource check JSON must keep memory-safety non-claim' }

  $ProfileCheckJson = Read-NativeOutput 'profile check JSON' $Hum @('profile-check', '--format', 'json', 'fixtures/profile_check/simple_pass.hum')
  Assert-Json 'profile check JSON' $ProfileCheckJson
  if (-not $ProfileCheckJson.Contains('"schema": "hum.profile_check.v0"')) { throw 'profile check JSON is missing hum.profile_check.v0 schema' }
  if (-not $ProfileCheckJson.Contains('"status": "recognized_profile_policy_checked_v0"')) { throw 'profile check JSON should pass for simple fixture' }
  if (-not $ProfileCheckJson.Contains('"mode": "recognized_profile_policy_gate_v0"')) { throw 'profile check JSON is missing profile policy gate mode' }
  if (-not $ProfileCheckJson.Contains('"resource_check_schema": "hum.resource_check.v0"')) { throw 'profile check JSON is missing resource-check dependency schema' }
  if (-not $ProfileCheckJson.Contains('"runtime_profiles_schema": "hum.runtime_profiles.v0"')) { throw 'profile check JSON is missing runtime profiles dependency schema' }
  if (-not $ProfileCheckJson.Contains('"runtime_profile_mode": "contract_only_no_profile_enforcement"')) { throw 'profile check JSON must keep runtime profile catalog mode' }
  if (-not $ProfileCheckJson.Contains('"profile_id": "normal"')) { throw 'profile check JSON is missing accepted normal profile id' }
  if (-not $ProfileCheckJson.Contains('"accepted_normal_profile_policy_v0"')) { throw 'profile check JSON is missing accepted normal profile check' }
  if (-not $ProfileCheckJson.Contains('"blocking_issues": 0')) { throw 'profile check JSON should have zero blocking issues for simple fixture' }
  if (-not $ProfileCheckJson.Contains('"proof_ready": 0')) { throw 'profile check JSON must not claim proof readiness' }
  if (-not $ProfileCheckJson.Contains('"execution_ready": 0')) { throw 'profile check JSON must not claim execution readiness' }
  if (-not $ProfileCheckJson.Contains('"ir_ready": 0')) { throw 'profile check JSON must not claim IR readiness' }
  if (-not $ProfileCheckJson.Contains('no profile enforcement')) { throw 'profile check JSON must keep profile-enforcement non-claim' }
  if (-not $ProfileCheckJson.Contains('no certification claim')) { throw 'profile check JSON must keep certification non-claim' }

  $IrReadinessJson = Read-NativeOutput 'IR readiness JSON' $Hum @('ir-readiness', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'IR readiness JSON' $IrReadinessJson
  if (-not $IrReadinessJson.Contains('"schema": "hum.ir_readiness.v0"')) { throw 'IR readiness JSON is missing hum.ir_readiness.v0 schema' }
  if (-not $IrReadinessJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'IR readiness JSON is missing Core Hum contract schema' }
  if (-not $IrReadinessJson.Contains('"ir_contract_schema": "hum.ir_contract.v0"')) { throw 'IR readiness JSON is missing Hum IR contract schema' }
  if (-not $IrReadinessJson.Contains('"resolver"')) { throw 'IR readiness JSON is missing resolver summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.resolve.v0"')) { throw 'IR readiness JSON is missing hum.resolve.v0 resolver schema' }
  if (-not $IrReadinessJson.Contains('"status": "checked_resolver_v0"')) { throw 'IR readiness JSON should include checked resolver status for reference fixture' }
  if (-not $IrReadinessJson.Contains('"mode": "source_analysis_only_no_type_or_borrow_check"')) { throw 'IR readiness JSON is missing resolver source-analysis mode' }
  if (-not $IrReadinessJson.Contains('"resolver_errors": 0')) { throw 'IR readiness JSON should have zero resolver errors for reference fixture' }
  if (-not $IrReadinessJson.Contains('"type_check"')) { throw 'IR readiness JSON is missing type_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.type_check.v0"')) { throw 'IR readiness JSON is missing hum.type_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"core_preview"')) { throw 'IR readiness JSON is missing core_preview summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.core_preview.v0"')) { throw 'IR readiness JSON is missing hum.core_preview.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"core_lower"')) { throw 'IR readiness JSON is missing core_lower summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.core_lower.v0"')) { throw 'IR readiness JSON is missing hum.core_lower.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"core_verify"')) { throw 'IR readiness JSON is missing core_verify summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.core_verify.v0"')) { throw 'IR readiness JSON is missing hum.core_verify.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"full_type_check"')) { throw 'IR readiness JSON is missing full_type_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.full_type_check.v0"')) { throw 'IR readiness JSON is missing hum.full_type_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"effect_check"')) { throw 'IR readiness JSON is missing effect_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.effect_check.v0"')) { throw 'IR readiness JSON is missing hum.effect_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"ownership_check"')) { throw 'IR readiness JSON is missing ownership_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.ownership_check.v0"')) { throw 'IR readiness JSON is missing hum.ownership_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"resource_check"')) { throw 'IR readiness JSON is missing resource_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.resource_check.v0"')) { throw 'IR readiness JSON is missing hum.resource_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"mode": "recognized_core_resource_gate_v0"')) { throw 'IR readiness JSON is missing resource-check gate mode' }
  if (-not $IrReadinessJson.Contains('"profile_check"')) { throw 'IR readiness JSON is missing profile_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.profile_check.v0"')) { throw 'IR readiness JSON is missing hum.profile_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"mode": "recognized_profile_policy_gate_v0"')) { throw 'IR readiness JSON is missing profile-check gate mode' }
  if (-not $IrReadinessJson.Contains('"mode": "non_executing_artifact_invariant_check_v0"')) { throw 'IR readiness JSON is missing core verify mode' }
  if (-not $IrReadinessJson.Contains('"status": "unverified_core_artifact_v0"')) { throw 'IR readiness JSON is missing unverified core lower status' }
  if (-not $IrReadinessJson.Contains('"typed_expression_previews": 2')) { throw 'IR readiness JSON is missing core preview typed expression count' }
  if (-not $IrReadinessJson.Contains('"status": "declaration_annotations_and_trivial_returns_checked_v0"')) { throw 'IR readiness JSON should include clean type-check status for reference fixture' }
  if (-not $IrReadinessJson.Contains('"type_errors": 0')) { throw 'IR readiness JSON should have zero type errors for reference fixture' }
  if (-not $IrReadinessJson.Contains('"unknown_type_references": 0')) { throw 'IR readiness JSON should have zero unknown type refs for reference fixture' }
  if (-not $IrReadinessJson.Contains('"checked_returns"')) { throw 'IR readiness JSON is missing type-check return counters' }
  if (-not $IrReadinessJson.Contains('"trivial_return_checks_v0"')) { throw 'IR readiness JSON is missing trivial return fact' }
  if (-not $IrReadinessJson.Contains('"type_check_summary_v0"')) { throw 'IR readiness JSON is missing type check summary fact' }
  if (-not $IrReadinessJson.Contains('"core_preview_summary_v0"')) { throw 'IR readiness JSON is missing core preview summary fact' }
  if (-not $IrReadinessJson.Contains('"core_lower_summary_v0"')) { throw 'IR readiness JSON is missing core lower summary fact' }
  if (-not $IrReadinessJson.Contains('"core_verify_summary_v0"')) { throw 'IR readiness JSON is missing core verify summary fact' }
  if (-not $IrReadinessJson.Contains('"full_type_check_summary_v0"')) { throw 'IR readiness JSON is missing full type check summary fact' }
  if (-not $IrReadinessJson.Contains('"effect_check_summary_v0"')) { throw 'IR readiness JSON is missing effect check summary fact' }
  if (-not $IrReadinessJson.Contains('"ownership_check_summary_v0"')) { throw 'IR readiness JSON is missing ownership check summary fact' }
  if (-not $IrReadinessJson.Contains('"resource_check_summary_v0"')) { throw 'IR readiness JSON is missing resource check summary fact' }
  if (-not $IrReadinessJson.Contains('"profile_check_summary_v0"')) { throw 'IR readiness JSON is missing profile check summary fact' }
  if (-not $IrReadinessJson.Contains('"unverified_core_artifact_rows_v0"')) { throw 'IR readiness JSON is missing unverified core artifact row fact' }
  if (-not $IrReadinessJson.Contains('"verified_core_artifact_rows_v0"')) { throw 'IR readiness JSON is missing verified core artifact row fact' }
  if (-not $IrReadinessJson.Contains('"checked_return_expression_type_slots_v0"')) { throw 'IR readiness JSON is missing checked return expression slot fact' }
  if (-not $IrReadinessJson.Contains('"name": "resolve"')) { throw 'IR readiness JSON is missing resolve pass status' }
  if (-not $IrReadinessJson.Contains('"checked_report_available"')) { throw 'IR readiness JSON is missing checked resolver pass availability' }
  if (-not $IrReadinessJson.Contains('"resolver_summary_v0"')) { throw 'IR readiness JSON is missing resolver summary fact' }
  if (-not $IrReadinessJson.Contains('"ready_for_ir": 0')) { throw 'IR readiness JSON must not claim IR readiness yet' }
  if (-not $IrReadinessJson.Contains('"body_grammar"')) { throw 'IR readiness JSON is missing body_grammar facts' }
  if (-not $IrReadinessJson.Contains('"name": "core_preview"')) { throw 'IR readiness JSON is missing core_preview pass status' }
  if (-not $IrReadinessJson.Contains('"body_grammar_partial_v0"')) { throw 'IR readiness JSON is missing partial body grammar fact' }
  if (-not $IrReadinessJson.Contains('"body_grammar_unsupported_lines"')) { throw 'IR readiness JSON is missing body grammar unsupported count' }
  if (-not $IrReadinessJson.Contains('"surface_save_requires_store_lowering"')) { throw 'IR readiness JSON is missing store save lowering blocker' }
  if (-not $IrReadinessJson.Contains('"core_lowering"')) { throw 'IR readiness JSON is missing core_lowering pass status' }
  if (-not $IrReadinessJson.Contains('"core_verify"')) { throw 'IR readiness JSON is missing core_verify pass status' }
  if (-not $IrReadinessJson.Contains('"blocked_by_full_type_check_errors"')) { throw 'IR readiness JSON is missing full type check blocker status' }
  if (-not $IrReadinessJson.Contains('"declaration_and_trivial_return_check_available"')) { throw 'IR readiness JSON is missing type-check pass availability' }
  if (-not $IrReadinessJson.Contains('"full_type_check"')) { throw 'IR readiness JSON is missing full_type_check blocker' }
  if (-not $IrReadinessJson.Contains('"full_type_check_errors"')) { throw 'IR readiness JSON is missing full type-check blocker reason' }
  if (-not $IrReadinessJson.Contains('"recognized_core_effect_gate_available_v0"')) { throw 'IR readiness JSON is missing effect-check pass availability' }
  if (-not $IrReadinessJson.Contains('"recognized_core_ownership_gate_available_v0"')) { throw 'IR readiness JSON is missing ownership-check pass availability' }
  if (-not $IrReadinessJson.Contains('"recognized_core_resource_gate_available_v0"')) { throw 'IR readiness JSON is missing resource-check pass availability' }
  if (-not $IrReadinessJson.Contains('"recognized_core_profile_gate_available_v0"')) { throw 'IR readiness JSON is missing profile-check pass availability' }
  if ($IrReadinessJson.Contains('"allocation_resource_check_not_implemented"')) { throw 'IR readiness JSON should not report resource check as not implemented' }
  if ($IrReadinessJson.Contains('"profile_check_not_implemented"')) { throw 'IR readiness JSON should not report profile check as not implemented' }
  if (-not $IrReadinessJson.Contains('"ir_verify_not_implemented"')) { throw 'IR readiness JSON is missing IR verifier blocker' }
  if (-not $IrReadinessJson.Contains('"not_implemented"')) { throw 'IR readiness JSON is missing not_implemented blockers' }
  if (-not $IrReadinessJson.Contains('"no IR emission"')) { throw 'IR readiness JSON must keep V0 non-emission claim' }

  $MathOutDir = Join-Path (Join-Path $RepoRoot 'target') ('hum-math-obligations-smoke-' + [System.Guid]::NewGuid().ToString('N'))
  Invoke-Native 'math obligations out-dir' $Hum @('math-obligations', '--out-dir', $MathOutDir, 'examples/control_flow.hum')
  $MathFiles = @(Get-ChildItem -LiteralPath $MathOutDir -Filter '*.json' -File)
  if ($MathFiles.Count -lt 1) { throw 'math obligations out-dir wrote no JSON files' }

  Invoke-Native 'reference fixture coverage smoke' $Hum @('test-skeletons', 'examples/reference_surface.hum')

  $GraphJson = Read-NativeOutput 'reference fixture graph JSON' $Hum @('graph', 'examples/reference_surface.hum')
  Assert-Json 'reference fixture graph JSON' $GraphJson
  $Graph = $GraphJson | ConvertFrom-Json
  if (-not $GraphJson.Contains('"folding_ranges"')) { throw 'reference fixture graph JSON is missing folding_ranges' }
  if (-not $GraphJson.Contains('"symbols"')) { throw 'reference fixture graph JSON is missing symbols' }
  if (-not $GraphJson.Contains('"portability"')) { throw 'reference fixture graph JSON is missing portability reservation' }
  if (-not $GraphJson.Contains('"status": "reserved_v0"')) { throw 'reference fixture graph JSON is missing reserved portability status' }
  if (-not $GraphJson.Contains('"mode": "source_analysis_only_no_target_selection"')) { throw 'reference fixture graph JSON must not select a target in V0' }
  if (-not $GraphJson.Contains('"target_facts_schema": "hum.target_facts.v0"')) { throw 'reference fixture graph JSON is missing target facts schema link' }
  if (-not $GraphJson.Contains('"target_fact_record_schema": "hum.target_fact_record.v0"')) { throw 'reference fixture graph JSON is missing target fact record schema link' }
  if (-not $GraphJson.Contains('"target_fact_records": ["wasm32-wasi-preview1"]')) { throw 'reference fixture graph JSON is missing source-declared target fact record' }
  if (-not $GraphJson.Contains('"required_capability_families": ["os.clock", "os.filesystem"]')) { throw 'reference fixture graph JSON is missing source-declared required capability families' }
  if (-not $GraphJson.Contains('"denied_capability_families": ["os.network"]')) { throw 'reference fixture graph JSON is missing source-declared denied capability family' }
  if (-not $GraphJson.Contains('"unavailable_capability_families": []')) { throw 'reference fixture graph JSON should have no unavailable capability families' }
  if (-not $GraphJson.Contains('"source_target_declarations"')) { throw 'reference fixture graph JSON is missing source target declarations' }
  if (-not $GraphJson.Contains('"status": "declared_not_enforced_v0"')) { throw 'reference fixture graph JSON must mark source target declarations unenforced' }
  if (-not $GraphJson.Contains('"source_section": "targets"')) { throw 'reference fixture graph JSON is missing targets source section links' }
  if (-not $GraphJson.Contains('"no target selected"')) { throw 'reference fixture graph JSON must keep portability non-claim' }
  Assert-ReferenceEvidenceCoverage $Graph

  $GraphParameterViewJson = Read-NativeOutput 'Session L parameter returned-view graph JSON' $Hum @('graph', 'fixtures/ownership_check/session_l_return_parameter_view_pass.hum')
  Assert-Json 'Session L parameter returned-view graph JSON' $GraphParameterViewJson
  if (-not $GraphParameterViewJson.Contains('"return_dependencies"')) { throw 'Session L graph JSON is missing return_dependencies' }
  if (-not $GraphParameterViewJson.Contains('"source": "text"')) { throw 'Session L graph JSON is missing returned-view source text' }
  if (-not $GraphParameterViewJson.Contains('"status": "declared_return_dependency_parameter_v0"')) { throw 'Session L graph JSON is missing returned-view parameter status' }

  $GraphFirstWordJson = Read-NativeOutput 'Session N first_word graph JSON' $Hum @('graph', 'examples/probes/first_word.hum')
  Assert-Json 'Session N first_word graph JSON' $GraphFirstWordJson
  if (-not $GraphFirstWordJson.Contains('"return_dependencies"')) { throw 'Session N graph JSON is missing return_dependencies' }
  if (-not $GraphFirstWordJson.Contains('"source": "text"')) { throw 'Session N graph JSON is missing returned-view source text' }
  if (-not $GraphFirstWordJson.Contains('"status": "declared_return_dependency_parameter_v0"')) { throw 'Session N graph JSON is missing returned-view parameter status' }
  if (-not $GraphFirstWordJson.Contains('slice_until(text, \" \"')) { throw 'Session N graph JSON is missing slice_until body evidence' }
  Invoke-RepoScript 'editor fixture recovery' 'check_editor_fixtures.ps1'

  $SyntaxJson = Read-NativeOutput 'syntax surface JSON' $Hum @('syntax')
  Assert-Json 'syntax surface JSON' $SyntaxJson
  if (-not $SyntaxJson.Contains('"section_catalog"')) { throw 'syntax surface JSON is missing section_catalog' }
  if (-not $SyntaxJson.Contains('"targets"')) { throw 'syntax surface JSON is missing targets section' }
  if (-not $SyntaxJson.Contains('"hover"')) { throw 'syntax surface JSON is missing hover metadata' }
  if (-not $SyntaxJson.Contains('"semantic_tokens"')) { throw 'syntax surface JSON is missing semantic_tokens' }
  if (-not $SyntaxJson.Contains('"token_types"')) { throw 'syntax surface JSON is missing semantic token types' }
  if (-not $SyntaxJson.Contains('"parameter_permission_modes": ["borrow", "change", "consume"]')) { throw 'syntax surface JSON is missing parameter permission modes' }
  if (-not $SyntaxJson.Contains('"writable_field_alias_form": "let <alias> = change <owner>.<field>"')) { throw 'syntax surface JSON is missing writable field alias form' }
  if (-not $SyntaxJson.Contains('"source": "writable_field_alias_permission"')) { throw 'syntax surface JSON is missing writable field alias semantic token role' }

  $TextMateJson = Read-NativeOutput 'TextMate grammar JSON' $Hum @('syntax', '--format', 'textmate')
  Assert-Json 'TextMate grammar JSON' $TextMateJson
  if (-not $TextMateJson.Contains('storage.modifier.writable-field-alias.hum')) { throw 'TextMate grammar JSON is missing writable field alias rule' }
  Assert-TextMateSnapshot $TextMateJson
  Assert-ReadmeHumExamplesMatch
  Assert-SessionASurfaceRules

  Invoke-Native 'git diff --check' $Git @('-c', "safe.directory=$GitRepoRoot", '-C', $GitRepoRoot, 'diff', '--check')
  Invoke-Native 'git diff --cached --check' $Git @('-c', "safe.directory=$GitRepoRoot", '-C', $GitRepoRoot, 'diff', '--cached', '--check')

  $DecisionIndex = [System.IO.File]::ReadAllText((Join-Path (Join-Path $RepoRoot 'docs') 'decisions\README.md'))
  if (-not $DecisionIndex.Contains('0009-adopt-formal-readability-not-english-mimicry.md')) { throw 'decision index is missing formal readability ADR' }
  if (-not $DecisionIndex.Contains('0010-adopt-explicit-state-model.md')) { throw 'decision index is missing explicit state model ADR' }
  if (-not $DecisionIndex.Contains('0011-add-checked-resolver-before-execution.md')) { throw 'decision index is missing checked resolver ADR' }
  $ArchitectureText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\ARCHITECTURE.md'))
  if (-not $ArchitectureText.Contains('Formal-readability doctrine')) { throw 'architecture is missing formal-readability doctrine' }
  if (-not $ArchitectureText.Contains('Debuggability doctrine')) { throw 'architecture is missing debuggability doctrine' }
  if (-not $ArchitectureText.Contains('State-management doctrine')) { throw 'architecture is missing state-management doctrine' }
  if (-not $ArchitectureText.Contains('Resolution doctrine')) { throw 'architecture is missing resolution doctrine' }
  if (-not $ArchitectureText.Contains('STATE_MODEL.md')) { throw 'architecture is missing state model link' }
  if (-not $ArchitectureText.Contains('HUM_RESOLVE_SCHEMA.md')) { throw 'architecture is missing resolve schema link' }
  if (-not $ArchitectureText.Contains('HUM_CORE_LOWER_SCHEMA.md')) { throw 'architecture is missing core lower schema link' }
  if (-not $ArchitectureText.Contains('HUM_CORE_VERIFY_SCHEMA.md')) { throw 'architecture is missing core verify schema link' }
  if (-not $ArchitectureText.Contains('HUM_RESOURCE_CHECK_SCHEMA.md')) { throw 'architecture is missing resource check schema link' }
  if (-not $ArchitectureText.Contains('HUM_PROFILE_CHECK_SCHEMA.md')) { throw 'architecture is missing profile check schema link' }
  if (-not $ArchitectureText.Contains('recognized_core_resource_gate_available_v0')) { throw 'architecture is missing current resource-check gate' }
  if (-not $ArchitectureText.Contains('recognized_core_profile_gate_available_v0')) { throw 'architecture is missing current profile-check gate' }
  if (-not $ArchitectureText.Contains('PORTABILITY_BOUNDARY_MODEL.md')) { throw 'architecture is missing portability boundary model link' }
  $LanguageReferenceText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\LANGUAGE_REFERENCE.md'))
  if (-not $LanguageReferenceText.Contains('traditional language reference spine')) { throw 'language reference is missing reference spine marker' }
  if (-not $LanguageReferenceText.Contains('PORTABILITY_BOUNDARY_MODEL.md')) { throw 'language reference is missing portability boundary link' }
  if (-not $LanguageReferenceText.Contains('STATE_MODEL.md')) { throw 'language reference is missing state model link' }
  if (-not $LanguageReferenceText.Contains('HUM_RESOLVE_SCHEMA.md')) { throw 'language reference is missing resolve schema link' }
  if (-not $LanguageReferenceText.Contains('HUM_RESOURCE_CHECK_SCHEMA.md')) { throw 'language reference is missing resource check schema link' }
  if (-not $LanguageReferenceText.Contains('HUM_PROFILE_CHECK_SCHEMA.md')) { throw 'language reference is missing profile check schema link' }
  if (-not $LanguageReferenceText.Contains('hum resource-check --format json')) { throw 'language reference is missing resource-check command' }
  if (-not $LanguageReferenceText.Contains('hum profile-check --format json')) { throw 'language reference is missing profile-check command' }
  if (-not $LanguageReferenceText.Contains('hum state-model --format json')) { throw 'language reference is missing state-model command' }
  if (-not $LanguageReferenceText.Contains('hum resolve --format json')) { throw 'language reference is missing resolve command' }
  if (-not $LanguageReferenceText.Contains('H1205')) { throw 'language reference is missing target declaration diagnostics' }
  $PortabilityBoundaryText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\PORTABILITY_BOUNDARY_MODEL.md'))
  if (-not $PortabilityBoundaryText.Contains('Hum Portability Boundary Model')) { throw 'portability boundary model is missing title' }
  if (-not $PortabilityBoundaryText.Contains('Absence Is A First-Class Case')) { throw 'portability boundary model is missing absence rule' }
  if (-not $PortabilityBoundaryText.Contains('Artifact Evidence')) { throw 'portability boundary model is missing artifact evidence rule' }
  if (-not $PortabilityBoundaryText.Contains('TARGET_FACTS_SCHEMA.md')) { throw 'portability boundary model is missing target facts schema link' }
  $SemanticGraphSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\SEMANTIC_GRAPH_SCHEMA.md'))
  if (-not $SemanticGraphSchemaText.Contains('"portability": {}')) { throw 'semantic graph schema doc is missing portability top-level shape' }
  if (-not $SemanticGraphSchemaText.Contains('source_analysis_only_no_target_selection')) { throw 'semantic graph schema doc is missing source-analysis portability mode' }
  if (-not $SemanticGraphSchemaText.Contains('target_facts_schema')) { throw 'semantic graph schema doc is missing target_facts_schema field' }
  if (-not $SemanticGraphSchemaText.Contains('target_fact_record_schema')) { throw 'semantic graph schema doc is missing target_fact_record_schema field' }
  if (-not $SemanticGraphSchemaText.Contains('reserved_v0')) { throw 'semantic graph schema doc is missing reserved portability status' }
  if (-not $SemanticGraphSchemaText.Contains('declared_not_enforced_v0')) { throw 'semantic graph schema doc is missing source target declaration status' }
  if (-not $SemanticGraphSchemaText.Contains('unavailable_capability_families')) { throw 'semantic graph schema doc is missing unavailable capability families' }
  $TargetFactsSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\TARGET_FACTS_SCHEMA.md'))
  if (-not $TargetFactsSchemaText.Contains('hum.target_facts.v0')) { throw 'target facts schema doc is missing hum.target_facts.v0' }
  if (-not $TargetFactsSchemaText.Contains('hum.target_fact_record.v0')) { throw 'target facts schema doc is missing hum.target_fact_record.v0' }
  if (-not $TargetFactsSchemaText.Contains('contract_only_no_host_probe')) { throw 'target facts schema doc is missing no-probe mode' }
  if (-not $TargetFactsSchemaText.Contains('unknown_fails_closed')) { throw 'target facts schema doc is missing fail-closed policy' }
  if (-not $TargetFactsSchemaText.Contains('../fixtures/target_facts')) { throw 'target facts schema doc is missing fixture link' }
  if (-not $TargetFactsSchemaText.Contains('Semantic Graph Link')) { throw 'target facts schema doc is missing semantic graph link section' }
  if (-not $TargetFactsSchemaText.Contains('targets:')) { throw 'target facts schema doc is missing targets section link' }
  if (-not $TargetFactsSchemaText.Contains('H1205')) { throw 'target facts schema doc is missing target declaration diagnostics' }
  $DebugDoctrineText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\DEBUGGABILITY_DOCTRINE.md'))
  if (-not $DebugDoctrineText.Contains('hum.debug_info.v0')) { throw 'debuggability doctrine is missing debug info schema direction' }
  if (-not $DebugDoctrineText.Contains('faster and clearer than adding `printf`')) { throw 'debuggability doctrine is missing debugger speed rule' }
  if (-not $DebugDoctrineText.Contains('type-attached visualizers')) { throw 'debuggability doctrine is missing type-attached visualizer rule' }
  if (-not $DebugDoctrineText.Contains('debug probe sites')) { throw 'debuggability doctrine is missing debug probe site rule' }
  if (-not $DebugDoctrineText.Contains('DEBUG_INFO_AND_VISUALIZER_MODEL.md')) { throw 'debuggability doctrine is missing debug info model link' }
  $DebugInfoModelText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\DEBUG_INFO_AND_VISUALIZER_MODEL.md'))
  if (-not $DebugInfoModelText.Contains('Target schema: `hum.debug_info.v0`')) { throw 'debug info model is missing target schema' }
  if (-not $DebugInfoModelText.Contains('many-to-many provenance')) { throw 'debug info model is missing source-map provenance rule' }
  if (-not $DebugInfoModelText.Contains('Probe sites unify')) { throw 'debug info model is missing probe-site model' }
  if (-not $DebugInfoModelText.Contains('Visualizers must be reversible')) { throw 'debug info model is missing reversible visualizer rule' }
  if (-not $DebugInfoModelText.Contains('Native DWARF and PDB are compatibility targets')) { throw 'debug info model is missing native debug bridge rule' }
  $ResearchMapText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\RESEARCH_MAP_2026.md'))
  if (-not $ResearchMapText.Contains('2026-07-07-rad-debugger-lessons.md')) { throw 'research map is missing RAD Debugger lessons' }
  if (-not $ResearchMapText.Contains('DEBUG_INFO_AND_VISUALIZER_MODEL.md')) { throw 'research map is missing debug info model gate' }
  if (-not $ResearchMapText.Contains('2026-07-07-bellard-systems-lessons.md')) { throw 'research map is missing Bellard systems lessons' }
  $BellardResearchText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\research\2026-07-07-bellard-systems-lessons.md'))
  if (-not $BellardResearchText.Contains('Bellard Test For Hum')) { throw 'Bellard research note is missing Bellard Test' }
  if (-not $BellardResearchText.Contains('footprint')) { throw 'Bellard research note is missing footprint pressure' }
  if (-not $BellardResearchText.Contains('deterministic artifacts')) { throw 'Bellard research note is missing deterministic artifact rule' }
  $RuntimeProfilesText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\RUNTIME_PROFILES.md'))
  if (-not $RuntimeProfilesText.Contains('footprint constrained')) { throw 'runtime profiles are missing footprint constrained profile' }
  if (-not $RuntimeProfilesText.Contains('hum.runtime_profiles.v0')) { throw 'runtime profiles doc is missing runtime profiles schema' }
  if (-not $RuntimeProfilesText.Contains('contract_only_no_profile_enforcement')) { throw 'runtime profiles doc is missing contract-only mode' }
  if (-not $RuntimeProfilesText.Contains('HUM_RUNTIME_PROFILES_SCHEMA.md')) { throw 'runtime profiles doc is missing schema link' }
  $RuntimeProfilesSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_RUNTIME_PROFILES_SCHEMA.md'))
  if (-not $RuntimeProfilesSchemaText.Contains('hum.runtime_profiles.v0')) { throw 'runtime profile schema doc is missing catalog schema' }
  if (-not $RuntimeProfilesSchemaText.Contains('hum.runtime_profile.v0')) { throw 'runtime profile schema doc is missing entry schema' }
  if (-not $RuntimeProfilesSchemaText.Contains('contract_only_no_profile_enforcement')) { throw 'runtime profile schema doc is missing contract-only mode' }
  if (-not $RuntimeProfilesSchemaText.Contains('HUM_PROFILE_CHECK_SCHEMA.md')) { throw 'runtime profile schema doc is missing profile check schema link' }
  $StateModelText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\STATE_MODEL.md'))
  if (-not $StateModelText.Contains('Hum State Model')) { throw 'state model doc is missing title' }
  if (-not $StateModelText.Contains('hum.state_model.v0')) { throw 'state model doc is missing state model schema' }
  if (-not $StateModelText.Contains('contract_only_partial_declared_mutation_check')) { throw 'state model doc is missing contract-only mode' }
  if (-not $StateModelText.Contains('No new Hum feature is allowed to hide state')) { throw 'state model doc is missing brutal state rule' }
  if (-not $StateModelText.Contains('hum resolve --format json')) { throw 'state model doc is missing resolver link' }
  $ResolveSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_RESOLVE_SCHEMA.md'))
  if (-not $ResolveSchemaText.Contains('hum.resolve.v0')) { throw 'resolve schema doc is missing hum.resolve.v0' }
  if (-not $ResolveSchemaText.Contains('source_analysis_only_no_type_or_borrow_check')) { throw 'resolve schema doc is missing source-analysis mode' }
  if (-not $ResolveSchemaText.Contains('H0604')) { throw 'resolve schema doc is missing resolver diagnostics' }
  if (-not $ResolveSchemaText.Contains('blocked_by_resolver_errors')) { throw 'resolve schema doc is missing IR readiness link' }
  $TypeEnvSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_TYPE_ENV_SCHEMA.md'))
  if (-not $TypeEnvSchemaText.Contains('hum.type_env.v0')) { throw 'type environment schema doc is missing hum.type_env.v0' }
  if (-not $TypeEnvSchemaText.Contains('declaration_inventory_no_type_check')) { throw 'type environment schema doc is missing no-type-check mode' }
  if (-not $TypeEnvSchemaText.Contains('unknown_type_name_v0')) { throw 'type environment schema doc is missing unknown type status' }
  $TypeCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_TYPE_CHECK_SCHEMA.md'))
  if (-not $TypeCheckSchemaText.Contains('hum.type_check.v0')) { throw 'type check schema doc is missing hum.type_check.v0' }
  if (-not $TypeCheckSchemaText.Contains('declaration_annotation_and_trivial_return_check_v0')) { throw 'type check schema doc is missing declaration-and-return mode' }
  if (-not $TypeCheckSchemaText.Contains('H0605')) { throw 'type check schema doc is missing H0605' }
  if (-not $TypeCheckSchemaText.Contains('H0606')) { throw 'type check schema doc is missing H0606' }
  if (-not $TypeCheckSchemaText.Contains('checked_returns')) { throw 'type check schema doc is missing checked_returns' }
  $IrReadinessSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_IR_READINESS_SCHEMA.md'))
  if (-not $IrReadinessSchemaText.Contains('hum.resolve.v0')) { throw 'IR readiness schema doc is missing resolver schema link' }
  if (-not $IrReadinessSchemaText.Contains('checked_resolver_errors')) { throw 'IR readiness schema doc is missing resolver blocker' }
  if (-not $IrReadinessSchemaText.Contains('hum.type_check.v0')) { throw 'IR readiness schema doc is missing type-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.core_preview.v0')) { throw 'IR readiness schema doc is missing core-preview schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.core_lower.v0')) { throw 'IR readiness schema doc is missing core-lower schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.core_verify.v0')) { throw 'IR readiness schema doc is missing core-verify schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.full_type_check.v0')) { throw 'IR readiness schema doc is missing full type-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.effect_check.v0')) { throw 'IR readiness schema doc is missing effect-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.ownership_check.v0')) { throw 'IR readiness schema doc is missing ownership-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.resource_check.v0')) { throw 'IR readiness schema doc is missing resource-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.profile_check.v0')) { throw 'IR readiness schema doc is missing profile-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_full_type_check_errors')) { throw 'IR readiness schema doc is missing full type-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_effect_check_errors')) { throw 'IR readiness schema doc is missing effect-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_ownership_check_errors')) { throw 'IR readiness schema doc is missing ownership-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_resource_check_errors')) { throw 'IR readiness schema doc is missing resource-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_profile_check_errors')) { throw 'IR readiness schema doc is missing profile-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_before_ir_verify')) { throw 'IR readiness schema doc is missing before-IR-verifier blocker' }
  if (-not $IrReadinessSchemaText.Contains('resource_check_summary_v0')) { throw 'IR readiness schema doc is missing resource check summary fact' }
  if (-not $IrReadinessSchemaText.Contains('profile_check_summary_v0')) { throw 'IR readiness schema doc is missing profile check summary fact' }
  if (-not $IrReadinessSchemaText.Contains('recognized_core_resource_gate_available_v0')) { throw 'IR readiness schema doc is missing resource-check pass status' }
  if (-not $IrReadinessSchemaText.Contains('recognized_core_profile_gate_available_v0')) { throw 'IR readiness schema doc is missing profile-check pass status' }
  if ($IrReadinessSchemaText.Contains('allocation_resource_check_not_implemented')) { throw 'IR readiness schema doc should not call resource check not implemented' }
  if ($IrReadinessSchemaText.Contains('profile_check_not_implemented')) { throw 'IR readiness schema doc should not call profile check not implemented' }
  if (-not $IrReadinessSchemaText.Contains('unverified_core_artifact_rows_v0')) { throw 'IR readiness schema doc is missing unverified core artifact row fact' }
  if (-not $IrReadinessSchemaText.Contains('core_verify')) { throw 'IR readiness schema doc is missing core_verify pass' }
  if (-not $IrReadinessSchemaText.Contains('verified_core_artifact_rows_v0')) { throw 'IR readiness schema doc is missing verified core artifact row fact' }
  if (-not $IrReadinessSchemaText.Contains('checked_return_expression_type_slots_v0')) { throw 'IR readiness schema doc is missing checked return expression slot fact' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_type_errors')) { throw 'IR readiness schema doc is missing type-error blocker' }
  if (-not $IrReadinessSchemaText.Contains('declaration_and_trivial_return_check_available')) { throw 'IR readiness schema doc is missing type-check pass status' }
  $ArchitectureText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\ARCHITECTURE.md'))
  if (-not $ArchitectureText.Contains('Current Compiler Spine')) { throw 'architecture doc is missing current compiler spine' }
  if (-not $ArchitectureText.Contains('recognized_core_body_type_gate_available_v0')) { throw 'architecture doc is missing current full type-check gate' }
  $CoreContractSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_CORE_CONTRACT_SCHEMA.md'))
  if (-not $CoreContractSchemaText.Contains('hum core-lower')) { throw 'Core contract schema doc is missing core-lower command link' }
  if (-not $CoreContractSchemaText.Contains('unverified_core_artifact_v0')) { throw 'Core contract schema doc is missing unverified core artifact gate' }
  if (-not $CoreContractSchemaText.Contains('declaration_and_trivial_return_check_available')) { throw 'Core contract schema doc is missing narrow type-check gate' }
  if (-not $CoreContractSchemaText.Contains('full_type_check')) { throw 'Core contract schema doc is missing full type-check gate' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_body_type_gate_available_v0')) { throw 'Core contract schema doc is missing full type-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_effect_gate_available_v0')) { throw 'Core contract schema doc is missing effect-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_ownership_gate_available_v0')) { throw 'Core contract schema doc is missing ownership-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_resource_gate_available_v0')) { throw 'Core contract schema doc is missing resource-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_profile_gate_available_v0')) { throw 'Core contract schema doc is missing profile-check gate status' }
  if (-not $CoreContractSchemaText.Contains('hum resource-check')) { throw 'Core contract schema doc is missing resource-check command link' }
  if (-not $CoreContractSchemaText.Contains('hum profile-check')) { throw 'Core contract schema doc is missing profile-check command link' }
  $FullTypeCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_FULL_TYPE_CHECK_SCHEMA.md'))
  if (-not $FullTypeCheckSchemaText.Contains('hum.full_type_check.v0')) { throw 'full type check schema doc is missing hum.full_type_check.v0' }
  if (-not $FullTypeCheckSchemaText.Contains('recognized_core_body_type_gate_v0')) { throw 'full type check schema doc is missing gate mode' }
  if (-not $FullTypeCheckSchemaText.Contains('hum full-type-check')) { throw 'full type check schema doc is missing command' }
  if (-not $FullTypeCheckSchemaText.Contains('no executable semantics')) { throw 'full type check schema doc must keep non-execution claim' }
  $EffectCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_EFFECT_CHECK_SCHEMA.md'))
  if (-not $EffectCheckSchemaText.Contains('hum.effect_check.v0')) { throw 'effect check schema doc is missing hum.effect_check.v0' }
  if (-not $EffectCheckSchemaText.Contains('recognized_core_effect_gate_v0')) { throw 'effect check schema doc is missing gate mode' }
  if (-not $EffectCheckSchemaText.Contains('hum effect-check')) { throw 'effect check schema doc is missing command' }
  if (-not $EffectCheckSchemaText.Contains('no executable semantics')) { throw 'effect check schema doc must keep non-execution claim' }
  $OwnershipCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_OWNERSHIP_CHECK_SCHEMA.md'))
  if (-not $OwnershipCheckSchemaText.Contains('hum.ownership_check.v0')) { throw 'ownership check schema doc is missing hum.ownership_check.v0' }
  if (-not $OwnershipCheckSchemaText.Contains('recognized_core_ownership_gate_v0')) { throw 'ownership check schema doc is missing gate mode' }
  if (-not $OwnershipCheckSchemaText.Contains('hum ownership-check')) { throw 'ownership check schema doc is missing command' }
  if (-not $OwnershipCheckSchemaText.Contains('no executable semantics')) { throw 'ownership check schema doc must keep non-execution claim' }
  $ResourceCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_RESOURCE_CHECK_SCHEMA.md'))
  if (-not $ResourceCheckSchemaText.Contains('hum.resource_check.v0')) { throw 'resource check schema doc is missing hum.resource_check.v0' }
  if (-not $ResourceCheckSchemaText.Contains('recognized_core_resource_gate_v0')) { throw 'resource check schema doc is missing gate mode' }
  if (-not $ResourceCheckSchemaText.Contains('hum resource-check')) { throw 'resource check schema doc is missing command' }
  if (-not $ResourceCheckSchemaText.Contains('no executable semantics')) { throw 'resource check schema doc must keep non-execution claim' }
  if (-not $ResourceCheckSchemaText.Contains('no allocation-freedom proof')) { throw 'resource check schema doc must keep allocation-proof non-claim' }
  $ProfileCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_PROFILE_CHECK_SCHEMA.md'))
  if (-not $ProfileCheckSchemaText.Contains('hum.profile_check.v0')) { throw 'profile check schema doc is missing hum.profile_check.v0' }
  if (-not $ProfileCheckSchemaText.Contains('recognized_profile_policy_gate_v0')) { throw 'profile check schema doc is missing gate mode' }
  if (-not $ProfileCheckSchemaText.Contains('hum profile-check')) { throw 'profile check schema doc is missing command' }
  if (-not $ProfileCheckSchemaText.Contains('blocked_by_unchecked_profile_policy_v0')) { throw 'profile check schema doc is missing strict profile blocker' }
  if (-not $ProfileCheckSchemaText.Contains('no profile enforcement')) { throw 'profile check schema doc must keep profile-enforcement non-claim' }
  if (-not $ProfileCheckSchemaText.Contains('no certification claim')) { throw 'profile check schema doc must keep certification non-claim' }
  $CoreLowerSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_CORE_LOWER_SCHEMA.md'))
  if (-not $CoreLowerSchemaText.Contains('hum.core_lower.v0')) { throw 'Core lower schema doc is missing hum.core_lower.v0' }
  if (-not $CoreLowerSchemaText.Contains('unverified_core_artifact_v0')) { throw 'Core lower schema doc is missing unverified artifact status' }
  if (-not $CoreLowerSchemaText.Contains('hum.core_verify.v0')) { throw 'Core lower schema doc is missing core verify sync link' }
  if (-not $CoreLowerSchemaText.Contains('no Hum IR emission')) { throw 'Core lower schema doc must keep non-IR-emission claim' }
  $CoreVerifySchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_CORE_VERIFY_SCHEMA.md'))
  if (-not $CoreVerifySchemaText.Contains('hum.core_verify.v0')) { throw 'Core verify schema doc is missing hum.core_verify.v0' }
  if (-not $CoreVerifySchemaText.Contains('non_executing_artifact_invariant_check_v0')) { throw 'Core verify schema doc is missing invariant-check mode' }
  if (-not $CoreVerifySchemaText.Contains('verified_non_executing_core_artifact_v0')) { throw 'Core verify schema doc is missing verification status' }
  if (-not $CoreVerifySchemaText.Contains('source_span_sane')) { throw 'Core verify schema doc is missing source span rule' }
  if (-not $CoreVerifySchemaText.Contains('no memory-safety proof')) { throw 'Core verify schema doc must keep memory-safety non-claim' }
  $ResolveDecisionText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\decisions\0011-add-checked-resolver-before-execution.md'))
  if (-not $ResolveDecisionText.Contains('checked resolver')) { throw 'checked resolver ADR is missing decision language' }
  $StateModelSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_STATE_MODEL_SCHEMA.md'))
  if (-not $StateModelSchemaText.Contains('hum.state_model.v0')) { throw 'state model schema doc is missing catalog schema' }
  if (-not $StateModelSchemaText.Contains('hum.state_permission.v0')) { throw 'state model schema doc is missing permission schema' }
  if (-not $StateModelSchemaText.Contains('contract_only_partial_declared_mutation_check')) { throw 'state model schema doc is missing contract-only mode' }
  $StateModelDecisionText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\decisions\0010-adopt-explicit-state-model.md'))
  if (-not $StateModelDecisionText.Contains('Hum adopts a source-visible state model')) { throw 'state model ADR is missing decision statement' }
  $OptimizationText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\OPTIMIZATION_AND_DSA_STRATEGY.md'))
  if (-not $OptimizationText.Contains('Bellard Constraint Rule')) { throw 'optimization strategy is missing Bellard constraint rule' }
  if (-not $ResearchMapText.Contains('2026-07-07-systems-legends-lessons.md')) { throw 'research map is missing systems legends lessons' }
  $LegendsResearchText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\research\2026-07-07-systems-legends-lessons.md'))
  if (-not $LegendsResearchText.Contains('Combined Legends Test For Hum')) { throw 'Systems legends research note is missing combined test' }
  if (-not $LegendsResearchText.Contains('traditional language reference')) { throw 'Systems legends research note is missing language reference consequence' }
  if (-not $LegendsResearchText.Contains('portability-boundary')) { throw 'Systems legends research note is missing portability boundary consequence' }
  if (-not $ResearchMapText.Contains('Systems Legends And Durable Taste')) { throw 'research map is missing systems legends cluster' }

  Invoke-RepoScript 'text hygiene' 'check_text_hygiene.ps1'
  Invoke-RepoScript 'public readiness' 'check_public_readiness.ps1'
  Invoke-RepoScript 'release readiness' 'check_release_readiness.ps1'

  Write-Host 'All Hum preflight checks passed.'
} finally {
  Pop-Location
}
