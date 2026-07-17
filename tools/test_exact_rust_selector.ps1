$ErrorActionPreference = 'Stop'
$script:ExactRustSelectorCredits = New-Object 'System.Collections.Generic.List[string]'

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

function Invoke-ExactRustTest {
  param(
    [string] $Label,
    [string] $Cargo,
    [string] $Selector
  )

  Write-Host "==> $Label"
  Assert-ExactRustSelectorSyntax $Selector

  $ListResult = Invoke-ExactRustNativeCapture $Cargo @('test', $Selector, '--', '--exact', '--list')
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

  $RunResult = Invoke-ExactRustNativeCapture $Cargo @('test', $Selector, '--', '--exact')
  $RunResult.Output | ForEach-Object { Write-Host $_ }
  if ($RunResult.ExitCode -ne 0) {
    throw "$Label failed with exit code $($RunResult.ExitCode)"
  }

  Assert-ExactRustSelectorEvidence $Selector $ListResult.Output $RunResult.Output
  $script:ExactRustSelectorCredits.Add($Selector)
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
