$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$HumName = if ($env:OS -eq 'Windows_NT') { 'hum.exe' } else { 'hum' }
$Hum = Join-Path (Join-Path (Join-Path $RepoRoot 'target') 'debug') $HumName

if ($args.Count -gt 0) {
  $Hum = [System.IO.Path]::GetFullPath($args[0])
}

if (-not (Test-Path -LiteralPath $Hum)) {
  throw "Hum binary was not found at $Hum; run cargo build first"
}

$Fixtures = @(
  @{ Path = 'fixtures\editor\mid_edit_missing_does.hum'; Codes = @('H0105'); ExitCode = 1 },
  @{ Path = 'fixtures\editor\incomplete_task_header.hum'; Codes = @('H0007'); ExitCode = 1 },
  @{ Path = 'fixtures\editor\missing_close_brace.hum'; Codes = @('H0004'); ExitCode = 1 },
  @{ Path = 'fixtures\editor\malformed_nested_item.hum'; Codes = @('H0003'); ExitCode = 1 },
  @{ Path = 'fixtures\editor\orphan_body_line.hum'; Codes = @('H0001'); ExitCode = 0 }
)

foreach ($fixture in $Fixtures) {
  $RelativePath = $fixture.Path
  $FixturePath = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $RelativePath))
  if (-not $FixturePath.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to read outside repo root: $FixturePath"
  }
  if (-not (Test-Path -LiteralPath $FixturePath)) {
    throw "Missing editor fixture: $RelativePath"
  }

  Write-Host "==> editor fixture graph $RelativePath"
  $global:LASTEXITCODE = 0
  $Output = & $Hum 'graph' $RelativePath
  $ExitCode = $LASTEXITCODE
  $Text = $Output -join "`n"

  try {
    $Graph = $Text | ConvertFrom-Json
  } catch {
    throw "editor fixture graph JSON did not parse for ${RelativePath}: $($_.Exception.Message)"
  }

  if ($Graph.schema -ne 'hum.semantic_graph.v0') {
    throw "editor fixture $RelativePath emitted unexpected schema $($Graph.schema)"
  }
  if ($ExitCode -ne $fixture.ExitCode) {
    throw "editor fixture $RelativePath exited $ExitCode; expected $($fixture.ExitCode)"
  }

  $ActualCodes = @($Graph.diagnostics | ForEach-Object { $_.code })
  if ($ActualCodes.Count -eq 0) {
    throw "editor fixture $RelativePath emitted no diagnostics"
  }

  foreach ($ExpectedCode in $fixture.Codes) {
    if ($ActualCodes -notcontains $ExpectedCode) {
      throw "editor fixture $RelativePath missing diagnostic $ExpectedCode; got $($ActualCodes -join ', ')"
    }
  }
}

Write-Host 'Editor fixture check passed.'