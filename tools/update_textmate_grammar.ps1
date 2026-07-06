$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$OutDir = Join-Path (Join-Path $RepoRoot 'editors') 'textmate'
$OutPath = Join-Path $OutDir 'hum.tmLanguage.json'

$CargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
if ($null -eq $CargoCommand) {
  $CargoHome = Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cargo'
  $CargoCandidate = Join-Path (Join-Path $CargoHome 'bin') 'cargo.exe'
  if (-not (Test-Path -LiteralPath $CargoCandidate)) {
    throw 'cargo was not found on PATH or in the standard user Cargo install directory'
  }
  $Cargo = $CargoCandidate
} else {
  $Cargo = $CargoCommand.Source
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Push-Location $RepoRoot
try {
  $Lines = & $Cargo run --quiet -- syntax --format textmate
  if ($LASTEXITCODE -ne 0) {
    throw "cargo run --quiet -- syntax --format textmate failed with exit code $LASTEXITCODE"
  }
} finally {
  Pop-Location
}

$Json = ($Lines -join "`n")
$Json | ConvertFrom-Json | Out-Null
if (-not $Json.EndsWith("`n")) {
  $Json += "`n"
}

$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($OutPath, $Json, $Utf8NoBom)
Write-Host "Updated editors/textmate/hum.tmLanguage.json"