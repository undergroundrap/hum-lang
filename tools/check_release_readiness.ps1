$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
  param([string] $Message)
  $Failures.Add($Message)
}

function Read-RepoText {
  param([string] $RelativePath)

  $path = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $RelativePath))
  if (-not $path.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to read outside repo root: $path"
  }

  return [System.IO.File]::ReadAllText($path).Trim()
}

$version = Read-RepoText 'VERSION'
$semverPattern = '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'
if (-not [regex]::IsMatch($version, $semverPattern)) {
  Add-Failure "VERSION is not valid SemVer: $version"
}

$cargoToml = Read-RepoText 'Cargo.toml'
$cargoVersionMatch = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $cargoVersionMatch.Success) {
  Add-Failure 'Cargo.toml does not declare package version'
} elseif ($cargoVersionMatch.Groups[1].Value -ne $version) {
  Add-Failure ("Cargo.toml version {0} does not match VERSION {1}" -f $cargoVersionMatch.Groups[1].Value, $version)
}

$releaseDoc = Read-RepoText 'docs\RELEASE_AND_VERSIONING.md'
$markdownTick = [string] [char] 96
$expectedVersionText = 'Current repo version: ' + $markdownTick + $version + $markdownTick
if (-not $releaseDoc.Contains($expectedVersionText)) {
  Add-Failure "docs/RELEASE_AND_VERSIONING.md does not mention current version $version"
}

$readme = Read-RepoText 'README.md'
if (-not $readme.Contains('docs/RELEASE_AND_VERSIONING.md')) {
  Add-Failure 'README.md does not link docs/RELEASE_AND_VERSIONING.md'
}
if (-not $readme.Contains('SECURITY.md')) {
  Add-Failure 'README.md does not link SECURITY.md'
}

$securityPolicy = Read-RepoText 'SECURITY.md'
foreach ($required in @('Supported Versions', 'How To Report', 'Security Claims')) {
  if (-not $securityPolicy.Contains($required)) {
    Add-Failure "SECURITY.md does not mention $required"
  }
}


$setup = Read-RepoText 'docs\SETUP.md'
foreach ($required in @('Visual Studio', 'Eclipse', 'Jupyter', 'VS Code', 'Cursor', 'PyCharm')) {
  if (-not $setup.Contains($required)) {
    Add-Failure "docs/SETUP.md does not mention $required"
  }
}

& (Join-Path $PSScriptRoot 'check_alpha_claims.ps1')
if (-not $?) {
  Add-Failure 'alpha claims check failed'
}

if ($Failures.Count -gt 0) {
  Write-Host 'Release readiness check failed:'
  foreach ($failure in $Failures) {
    Write-Host ("- {0}" -f $failure)
  }
  exit 1
}

Write-Host "Release readiness check passed for version $version."
