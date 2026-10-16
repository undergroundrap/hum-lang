$ErrorActionPreference = 'Stop'
$script:ExactRustSelectorCredits = New-Object 'System.Collections.Generic.List[string]'
$script:ExactRustMutationGenerationOwner = $null

function Reset-ExactRustSelectorCredits {
  $script:ExactRustSelectorCredits.Clear()
}

function Get-ExactRustSelectorCredits {
  return @($script:ExactRustSelectorCredits)
}

function Assert-ExactRustSelectorSyntax {
  param([string] $Selector)

  if ([string]::IsNullOrWhiteSpace($Selector) -or
      -not [regex]::IsMatch($Selector, '^[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+$')) {
    throw "Rust test selector must be one fully qualified test name: '$Selector'"
  }
}

function Invoke-ExactRustNativeCapture {
  param(
    [string] $Cargo,
    [string[]] $Arguments
  )

  if (-not (Test-Path -LiteralPath $Cargo -PathType Leaf)) {
    throw "cargo executable is unavailable: $Cargo"
  }

  $PreviousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $Output = @(& $Cargo @Arguments 2>&1 | ForEach-Object { $_.ToString() })
    $ExitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $PreviousErrorActionPreference
  }

  return [pscustomobject] @{
    Output = $Output
    ExitCode = $ExitCode
  }
}

function Assert-ExactRustSelectorEvidence {
  param(
    [string] $Selector,
    [string[]] $ListOutput,
    [string[]] $RunOutput
  )

  Assert-ExactRustSelectorSyntax $Selector
  $EscapedSelector = [regex]::Escape($Selector)

  $ListedTests = @($ListOutput | Where-Object { $_ -match ': test$' })
  $ExactListings = @($ListedTests | Where-Object { $_ -match "^${EscapedSelector}: test$" })
  if ($ListedTests.Count -ne 1 -or $ExactListings.Count -ne 1) {
    throw "exact Rust selector '$Selector' must list exactly one test; listed $($ListedTests.Count) total and $($ExactListings.Count) exact"
  }

  $RunningCounts = @(
    $RunOutput |
      ForEach-Object {
        if ($_ -match '^running (?<count>[0-9]+) tests?$') {
          [int] $Matches['count']
        }
      }
  )
  if ($RunningCounts.Count -eq 0 -or ($RunningCounts | Measure-Object -Sum).Sum -ne 1) {
    $Total = if ($RunningCounts.Count -eq 0) { 0 } else { ($RunningCounts | Measure-Object -Sum).Sum }
    throw "exact Rust selector '$Selector' must run exactly one test; harness reported $Total"
  }

  $NamedResults = @($RunOutput | Where-Object { $_ -match '^test .+ \.\.\. (?:ok|FAILED|ignored)$' })
  $ExactResults = @($NamedResults | Where-Object { $_ -match "^test ${EscapedSelector} \.\.\. ok$" })
  if ($NamedResults.Count -ne 1 -or $ExactResults.Count -ne 1) {
    throw "exact Rust selector '$Selector' must produce one matching successful result; saw $($NamedResults.Count) named and $($ExactResults.Count) exact"
  }

  $SummaryCounts = @(
    $RunOutput |
      ForEach-Object {
        if ($_ -match '^test result: ok\. (?<passed>[0-9]+) passed; (?<failed>[0-9]+) failed; (?<ignored>[0-9]+) ignored; (?<measured>[0-9]+) measured; (?<filtered>[0-9]+) filtered out;') {
          [pscustomobject] @{
            Passed = [int] $Matches['passed']
            Failed = [int] $Matches['failed']
            Ignored = [int] $Matches['ignored']
            Measured = [int] $Matches['measured']
          }
        }
      }
  )
  $Passed = ($SummaryCounts | Measure-Object -Property Passed -Sum).Sum
  $Failed = ($SummaryCounts | Measure-Object -Property Failed -Sum).Sum
  $Ignored = ($SummaryCounts | Measure-Object -Property Ignored -Sum).Sum
  $Measured = ($SummaryCounts | Measure-Object -Property Measured -Sum).Sum
  if ($SummaryCounts.Count -eq 0 -or $Passed -ne 1 -or $Failed -ne 0 -or $Ignored -ne 0 -or $Measured -ne 0) {
    throw "exact Rust selector '$Selector' has invalid aggregate result: passed=$Passed failed=$Failed ignored=$Ignored measured=$Measured"
  }
}

# Cargo still owns every listing, run, target choice, freshness decision and exit.
# Only WO24 mutations use a private source/target generation; honest calls keep
# the original working directory and warm target. No executable/result is replayed.
function Get-ExactRustGenerationDigest {
  param([byte[]] $Bytes)
  $Hasher = [Security.Cryptography.SHA256]::Create()
  try { return [BitConverter]::ToString($Hasher.ComputeHash($Bytes)) } finally { $Hasher.Dispose() }
}

function New-ExactRustMutationGeneration {
  param([string] $RepoRoot, [string] $Cargo)

  if ($null -ne $script:ExactRustMutationGenerationOwner) { throw 'WO24 Cargo generation is already active' }
  $OwnedRoot = $null
  $OwnedCreated = $false
  try {
    $Root = [IO.Path]::GetFullPath($RepoRoot)
    $Comparison = if ([IO.Path]::DirectorySeparatorChar -eq '\') { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
    if (-not [string]::Equals($Root, [IO.Path]::GetFullPath((Get-Location).Path), $Comparison)) { throw 'working directory differs' }
    $RootInfo = [IO.DirectoryInfo]::new($Root)
    if (-not $RootInfo.Exists -or ($RootInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'nonordinary source root' }
    $Git = @(Get-Command git -CommandType Application -ErrorAction Stop)[0].Source
    $Status = Invoke-ExactRustNativeCapture $Git @('-C', $Root, 'status', '--porcelain=v1', '--untracked-files=all')
    if ($Status.ExitCode -ne 0 -or @($Status.Output | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count -ne 0) { throw 'working tree is not clean' }
    $Tracked = Invoke-ExactRustNativeCapture $Git @('-C', $Root, '-c', 'core.quotePath=true', 'ls-files', '--stage')
    if ($Tracked.ExitCode -ne 0 -or $Tracked.Output.Count -eq 0) { throw 'tracked source inventory unavailable' }
    $Ignored = Invoke-ExactRustNativeCapture $Git @('-C', $Root, 'ls-files', '--others', '--ignored', '--exclude-standard', '--', '.', ':(exclude)target', ':(exclude).cargo-home')
    if ($Ignored.ExitCode -ne 0 -or @($Ignored.Output | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count -ne 0) { throw 'unbound ignored source inputs' }

    $Files = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
    $SourceBytes = [Collections.Generic.Dictionary[string,byte[]]]::new([StringComparer]::Ordinal)
    foreach ($Line in $Tracked.Output) {
      if ($Line -cnotmatch '^100644 [0-9a-f]{40} 0\t(?<path>[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*)$') { throw 'unsupported source inventory shape' }
      $Relative = $Matches['path']
      if ($Relative -match '(^|/)\.\.?(/|$)|(^|/)(?:build\.rs|rust-toolchain(?:\.toml)?)$|^\.cargo/' -or $Relative -match '^(?:target|\.cargo-home|\.git)(?:/|$)' -or $Files.ContainsKey($Relative)) { throw 'unsupported source context' }
      $Path = Join-Path $Root $Relative
      $Info = [IO.FileInfo]::new($Path)
      if (-not $Info.Exists -or ($Info.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'nonordinary source input' }
      $Parent = $Info.Directory
      while (-not [string]::Equals($Parent.FullName, $Root, $Comparison)) {
        if (($Parent.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $null -eq $Parent.Parent) { throw 'nonordinary source parent' }
        $Parent = $Parent.Parent
      }
      $Bytes = [IO.File]::ReadAllBytes($Path)
      $Files.Add($Relative, (Get-ExactRustGenerationDigest $Bytes))
      $SourceBytes.Add($Relative, $Bytes)
    }
    if (-not $Files.ContainsKey('Cargo.toml') -or -not $Files.ContainsKey('Cargo.lock')) { throw 'root Cargo inputs unavailable' }
    $OwnedRoot = Join-Path ([IO.Path]::GetTempPath()) ('hum-wo24-cargo-' + [Guid]::NewGuid().ToString('N'))
    if ([IO.Directory]::Exists($OwnedRoot)) { throw 'generation directory already exists' }
    $null = [IO.Directory]::CreateDirectory($OwnedRoot)
    $OwnedCreated = $true
    $MutableRoot = Join-Path $OwnedRoot 'source'
    foreach ($Relative in $Files.Keys) {
      $Destination = Join-Path $MutableRoot $Relative
      $null = [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Destination))
      [IO.File]::WriteAllBytes($Destination, $SourceBytes[$Relative])
    }
    $Generation = [pscustomobject] @{ Root = $Root; MutableRoot = $MutableRoot; OwnedRoot = $OwnedRoot; Target = (Join-Path $OwnedRoot 'target'); Cargo = $Cargo; Files = $Files; Active = $true }
    $BoundFiles = [Collections.Generic.Dictionary[string,string]]::new($Files, [StringComparer]::Ordinal)
    $script:ExactRustMutationGenerationOwner = [pscustomobject] @{ Instance = $Generation; Root = $Root; MutableRoot = $MutableRoot; OwnedRoot = $OwnedRoot; Target = $Generation.Target; Cargo = $Cargo; Files = $BoundFiles }
    Assert-ExactRustMutationGeneration $Generation $Cargo
    Write-Host '==> Work Order 24 uses an invocation-local mutable Cargo generation; honest source and target stay in place'
    return $Generation
  } catch {
    $PreparationFailure = $_
    $script:ExactRustMutationGenerationOwner = $null
    if ($OwnedCreated) {
      # A failed cleanup is not permission to hide residue and restart the suite.
      Remove-Item -LiteralPath $OwnedRoot -Recurse -Force -ErrorAction Stop
    }
    Write-Host ("==> Work Order 24 ordinary Cargo route: generation preparation unavailable ({0})" -f $PreparationFailure.Exception.Message)
    return $null
  }
}

function Assert-ExactRustMutationGeneration {
  param($Generation, [string] $Cargo, [string] $MutationPath = '', [byte[]] $MutationBytes = $null)

  $Owner = $script:ExactRustMutationGenerationOwner
  $WorkingRoot = [IO.Path]::GetFullPath((Get-Location).Path)
  $Comparison = if ([IO.Path]::DirectorySeparatorChar -eq '\') { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
  if ($null -eq $Owner -or -not [object]::ReferenceEquals($Generation, $Owner.Instance) -or -not $Generation.Active -or $Generation.Cargo -cne $Cargo -or
      -not [string]::Equals($WorkingRoot, $Owner.Root, $Comparison) -or $Generation.Root -cne $Owner.Root -or $Generation.MutableRoot -cne $Owner.MutableRoot -or $Generation.OwnedRoot -cne $Owner.OwnedRoot -or $Generation.Target -cne $Owner.Target -or $Cargo -cne $Owner.Cargo -or $Generation.Files.Count -ne $Owner.Files.Count) {
    throw 'WO24 Cargo generation lifetime or context changed'
  }
  if (-not [string]::IsNullOrEmpty($MutationPath) -and (-not $Generation.Files.ContainsKey($MutationPath) -or $null -eq $MutationBytes)) { throw 'WO24 Cargo generation mutation owner changed' }
  foreach ($Relative in $Owner.Files.Keys) {
    $Expected = $Owner.Files[$Relative]
    if (-not $Generation.Files.ContainsKey($Relative) -or $Generation.Files[$Relative] -cne $Expected) { throw 'WO24 Cargo generation input binding changed' }
    if ((Get-ExactRustGenerationDigest ([IO.File]::ReadAllBytes((Join-Path $Generation.Root $Relative)))) -cne $Expected) { throw 'WO24 honest Cargo source changed' }
    $MutableExpected = if ([string]::Equals($Relative, $MutationPath, [StringComparison]::Ordinal)) { Get-ExactRustGenerationDigest $MutationBytes } else { $Expected }
    if ((Get-ExactRustGenerationDigest ([IO.File]::ReadAllBytes((Join-Path $Generation.MutableRoot $Relative)))) -cne $MutableExpected) { throw 'WO24 mutable Cargo source changed' }
  }
}

function Remove-ExactRustMutationGeneration {
  param($Generation)
  if ($null -eq $Generation) { return }
  $Owner = $script:ExactRustMutationGenerationOwner
  if ($null -eq $Owner -or -not [object]::ReferenceEquals($Generation, $Owner.Instance)) { throw 'WO24 generation cleanup ownership changed' }
  $script:ExactRustMutationGenerationOwner = $null
  $Generation.Active = $false
  $Info = [IO.DirectoryInfo]::new($Owner.OwnedRoot)
  if (-not $Info.Exists -or ($Info.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'WO24 generation cleanup directory changed' }
  Remove-Item -LiteralPath $Owner.OwnedRoot -Recurse -Force -ErrorAction Stop
}

function Invoke-ExactRustTest {
  param(
    [string] $Label,
    [string] $Cargo,
    [string] $Selector,
    $Generation = $null,
    [string] $MutationPath = '',
    [byte[]] $MutationBytes = $null
  )

  Write-Host "==> $Label"
  Assert-ExactRustSelectorSyntax $Selector

  $SelectorPhasePrefix = ''
  try {
    # Only the current WO24 row/role owners receive nested diagnostics.
    $Rows = @(
      @('N01 program-neutral feature identity', 'n01', 'native_program::tests::native_feature_discrimination_is_typed_and_load_bearing'),
      @('N02 exact constant-Text body admission', 'n02', 'backend_input::tests::canonical_hello_world_backend_input_is_exact_and_nonforgeable'),
      @('N03 v2 live artifact equality', 'n03', 'ir_verify::tests::hello_world_artifact_rejection_matrix_is_complete'),
      @('N04 verified source literal getter', 'n04', 'backend_cranelift::tests::hello_world_lowering_is_source_driven_and_load_bearing'),
      @('N05 exact finalized invocation evidence', 'n05', 'backend_cranelift::tests::hello_world_lowering_is_source_driven_and_load_bearing'),
      @('N06 deny-first operator consent', 'n06', 'main::tests::native_hello_world_run_is_authority_bound_and_platform_exact'),
      @('N07 native rejection cannot fall back', 'n07', 'native_program::tests::native_feature_discrimination_is_typed_and_load_bearing'),
      @('N08 required target admission', 'n08', 'backend_cranelift::tests::hello_world_lowering_is_source_driven_and_load_bearing')
    )
    $Roles = @(
      @('honest selector before mutation', 'before'),
      @('initialized mutation', 'mutation'),
      @('honest selector after restoration', 'after')
    )
    foreach ($Row in $Rows) {
      foreach ($Role in $Roles) {
        if ([string]::Equals($Label, ('Work Order 24 ' + $Row[0] + ' ' + $Role[0]), [StringComparison]::Ordinal) -and
            [string]::Equals($Selector, $Row[2], [StringComparison]::Ordinal)) {
          $SelectorPhasePrefix = 'compiler-front-wo24-' + $Row[1] + '-' + $Role[1]
        }
      }
    }
  } catch { $SelectorPhasePrefix = '' }
  $WriteSelectorPhaseMarker = {
    param([string] $Part, [string] $Event, [string] $Outcome = '')
    if ([string]::IsNullOrEmpty($SelectorPhasePrefix)) { return }
    try { $null = Write-HumPhaseMarker ($SelectorPhasePrefix + '-' + $Part) $Event $Outcome } catch {
      # Diagnostics are optional, including when this helper is used standalone.
    }
  }

  $ListPhaseOutcome = 'failed'
  $null = & $WriteSelectorPhaseMarker 'list' 'start'
  try {
    $CargoArguments = @('test', $Selector)
    if ($null -ne $Generation) {
      Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes
      if (-not [string]::IsNullOrEmpty($MutationPath)) { $CargoArguments += @('--manifest-path', (Join-Path $Generation.MutableRoot 'Cargo.toml'), '--target-dir', $Generation.Target) }
    }
    $CargoArguments += @('--', '--exact', '--list')
    $ListResult = Invoke-ExactRustNativeCapture $Cargo $CargoArguments
    if ($null -ne $Generation) { Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes }
    if ($ListResult.ExitCode -ne 0) {
      $ListResult.Output | ForEach-Object { Write-Host $_ }
      throw "$Label could not list '$Selector'; cargo exited $($ListResult.ExitCode)"
    }

    $EscapedSelector = [regex]::Escape($Selector)
    $ListedTests = @($ListResult.Output | Where-Object { $_ -match ': test$' })
    $ExactListings = @($ListedTests | Where-Object { $_ -match "^${EscapedSelector}: test$" })
    if ($ListedTests.Count -ne 1 -or $ExactListings.Count -ne 1) {
      $ListResult.Output | ForEach-Object { Write-Host $_ }
      throw "$Label must resolve '$Selector' to exactly one test before execution; listed $($ListedTests.Count) total and $($ExactListings.Count) exact"
    }
    $ListPhaseOutcome = 'completed'
  } finally { $null = & $WriteSelectorPhaseMarker 'list' 'end' $ListPhaseOutcome }

  $RunPhaseOutcome = 'failed'
  $null = & $WriteSelectorPhaseMarker 'run' 'start'
  try {
    $CargoArguments = @('test', $Selector)
    if ($null -ne $Generation) {
      Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes
      if (-not [string]::IsNullOrEmpty($MutationPath)) { $CargoArguments += @('--manifest-path', (Join-Path $Generation.MutableRoot 'Cargo.toml'), '--target-dir', $Generation.Target) }
    }
    $CargoArguments += @('--', '--exact')
    $RunResult = Invoke-ExactRustNativeCapture $Cargo $CargoArguments
    if ($null -ne $Generation) { Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes }
    $RunResult.Output | ForEach-Object { Write-Host $_ }
    if ($RunResult.ExitCode -ne 0) {
      throw "$Label failed with exit code $($RunResult.ExitCode)"
    }

    Assert-ExactRustSelectorEvidence $Selector $ListResult.Output $RunResult.Output
    $script:ExactRustSelectorCredits.Add($Selector)
    $RunPhaseOutcome = 'completed'
  } finally { $null = & $WriteSelectorPhaseMarker 'run' 'end' $RunPhaseOutcome }
}

function Assert-ExactRustSelectorThrows {
  param(
    [string] $Label,
    [scriptblock] $Action
  )

  $Threw = $false
  try {
    & $Action
  } catch {
    $Threw = $true
  }
  if (-not $Threw) {
    throw "exact Rust selector sabotage stayed green: $Label"
  }
}

function Invoke-ExactRustSelectorSelfTests {
  param([string] $Cargo)

  Write-Host '==> exact Rust selector guard tests'
  $Selector = 'tests::selected'
  $ValidList = @('tests::selected: test', '1 test, 0 benchmarks')
  $ValidRun = @(
    'running 1 test',
    'test tests::selected ... ok',
    '',
    'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
  )
  Assert-ExactRustSelectorEvidence $Selector $ValidList $ValidRun

  foreach ($Malformed in @('', 'tests', 'tests::*', 'tests::selected extra', '::tests::selected', 'tests::selected::')) {
    Assert-ExactRustSelectorThrows "malformed selector '$Malformed'" {
      Assert-ExactRustSelectorSyntax $Malformed
    }
  }
  Assert-ExactRustSelectorThrows 'selected test deleted or renamed' {
    Assert-ExactRustSelectorEvidence $Selector @('tests::selected_renamed: test') $ValidRun
  }
  Assert-ExactRustSelectorThrows 'nonexistent selector' {
    Assert-ExactRustSelectorEvidence 'tests::missing' @() @()
  }
  Assert-ExactRustSelectorThrows 'ambiguous duplicate exact selector' {
    Assert-ExactRustSelectorEvidence $Selector @('tests::selected: test', 'tests::selected: test') $ValidRun
  }
  Assert-ExactRustSelectorThrows 'filtered selector ran zero tests' {
    Assert-ExactRustSelectorEvidence $Selector $ValidList @(
      'running 0 tests',
      'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s'
    )
  }
  Assert-ExactRustSelectorThrows 'duplicate test execution' {
    Assert-ExactRustSelectorEvidence $Selector $ValidList @(
      'running 2 tests',
      'test tests::selected ... ok',
      'test tests::selected ... ok',
      'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    )
  }
  Assert-ExactRustSelectorThrows 'unavailable cargo' {
    Invoke-ExactRustTest 'unavailable cargo sabotage' (Join-Path ([System.IO.Path]::GetTempPath()) "missing-cargo-$([guid]::NewGuid().ToString('N'))") $Selector
  }

  $TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "hum-exact-selector-$([guid]::NewGuid().ToString('N'))"
  $OriginalLocation = Get-Location
  try {
    [System.IO.Directory]::CreateDirectory((Join-Path $TempRoot 'src/bin')) | Out-Null
    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'Cargo.toml'),
      "[package]`nname = `"exact-selector-probe`"`nversion = `"0.0.0`"`nedition = `"2021`"`n",
      (New-Object System.Text.UTF8Encoding($false))
    )
    $SelectedSource = "#[cfg(test)]`nmod tests {`n    #[test]`n    fn selected() {}`n}`n`nfn main() {}`n"
    $RenamedSource = $SelectedSource.Replace('fn selected()', 'fn selected_renamed()')
    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/first.rs'),
      $SelectedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    Push-Location $TempRoot

    Invoke-ExactRustTest 'exact selector positive control' $Cargo $Selector

    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/first.rs'),
      $RenamedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    Assert-ExactRustSelectorThrows 'real selected test rename' {
      Invoke-ExactRustTest 'real selected test rename sabotage' $Cargo $Selector
    }
    Assert-ExactRustSelectorThrows 'real nonexistent selection' {
      Invoke-ExactRustTest 'real nonexistent selector sabotage' $Cargo 'tests::missing'
    }

    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/first.rs'),
      $SelectedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/second.rs'),
      $SelectedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    Assert-ExactRustSelectorThrows 'real duplicate exact selection across test binaries' {
      Invoke-ExactRustTest 'real duplicate exact selector sabotage' $Cargo $Selector
    }
  } finally {
    while ((Get-Location).Path -ne $OriginalLocation.Path) {
      Pop-Location
    }
    if (Test-Path -LiteralPath $TempRoot) {
      Remove-Item -LiteralPath $TempRoot -Recurse -Force
    }
  }

  Write-Host 'exact Rust selector guard tests passed'
}

if ($MyInvocation.InvocationName -ne '.') {
  $CargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
  if ($null -ne $CargoCommand) {
    $CargoPath = $CargoCommand.Source
  } else {
    $CargoPath = Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cargo/bin/cargo.exe'
  }
  Invoke-ExactRustSelectorSelfTests $CargoPath
}
