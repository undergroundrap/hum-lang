$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

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

Push-Location $RepoRoot
try {
  Invoke-Native 'cargo fmt --check' $Cargo @('fmt', '--check')
  Invoke-Native 'cargo test' $Cargo @('test')
  Invoke-Native 'cargo clippy' $Cargo @('clippy', '--all-targets', '--', '-D', 'warnings')
  Invoke-Native 'cargo build' $Cargo @('build')

  $HumName = if ($env:OS -eq 'Windows_NT') { 'hum.exe' } else { 'hum' }
  $Hum = Join-Path (Join-Path (Join-Path $RepoRoot 'target') 'debug') $HumName

  $VersionJson = Read-NativeOutput 'version JSON' $Hum @('version', '--format', 'json')
  Assert-Json 'version JSON' $VersionJson

  $ExplainJson = Read-NativeOutput 'diagnostic explain JSON' $Hum @('explain', 'H0201', '--format', 'json')
  Assert-Json 'diagnostic explain JSON' $ExplainJson

  $DiagnosticsJson = Read-NativeOutput 'diagnostic catalog JSON' $Hum @('diagnostics', '--format', 'json')
  Assert-Json 'diagnostic catalog JSON' $DiagnosticsJson

  Invoke-Native 'hum check examples' $Hum @('check', 'examples')

  $CheckJson = Read-NativeOutput 'check JSON' $Hum @('check', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'check JSON' $CheckJson
  if (-not $CheckJson.Contains('"schema": "hum.check.v0"')) { throw 'check JSON is missing hum.check.v0 schema' }

  Invoke-Native 'reference fixture coverage smoke' $Hum @('test-skeletons', 'examples/reference_surface.hum')

  $GraphJson = Read-NativeOutput 'reference fixture graph JSON' $Hum @('graph', 'examples/reference_surface.hum')
  Assert-Json 'reference fixture graph JSON' $GraphJson
  if (-not $GraphJson.Contains('"folding_ranges"')) { throw 'reference fixture graph JSON is missing folding_ranges' }
  if (-not $GraphJson.Contains('"symbols"')) { throw 'reference fixture graph JSON is missing symbols' }

  Invoke-RepoScript 'editor fixture recovery' 'check_editor_fixtures.ps1'

  $SyntaxJson = Read-NativeOutput 'syntax surface JSON' $Hum @('syntax')
  Assert-Json 'syntax surface JSON' $SyntaxJson
  if (-not $SyntaxJson.Contains('"section_catalog"')) { throw 'syntax surface JSON is missing section_catalog' }
  if (-not $SyntaxJson.Contains('"hover"')) { throw 'syntax surface JSON is missing hover metadata' }
  if (-not $SyntaxJson.Contains('"semantic_tokens"')) { throw 'syntax surface JSON is missing semantic_tokens' }
  if (-not $SyntaxJson.Contains('"token_types"')) { throw 'syntax surface JSON is missing semantic token types' }

  $TextMateJson = Read-NativeOutput 'TextMate grammar JSON' $Hum @('syntax', '--format', 'textmate')
  Assert-Json 'TextMate grammar JSON' $TextMateJson
  Assert-TextMateSnapshot $TextMateJson

  Invoke-Native 'git diff --check' $Git @('diff', '--check')
  Invoke-Native 'git diff --cached --check' $Git @('diff', '--cached', '--check')

  Invoke-RepoScript 'text hygiene' 'check_text_hygiene.ps1'
  Invoke-RepoScript 'public readiness' 'check_public_readiness.ps1'
  Invoke-RepoScript 'release readiness' 'check_release_readiness.ps1'

  Write-Host 'All Hum preflight checks passed.'
} finally {
  Pop-Location
}
