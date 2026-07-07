$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

function Resolve-Tool {
  param(
    [string] $Name,
    [string] $MissingMessage
  )

  $Command = Get-Command $Name -ErrorAction SilentlyContinue
  if ($null -ne $Command) {
    return $Command.Source
  }

  throw $MissingMessage
}

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

  return ($Output -join "`n").Trim()
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

function Assert-CleanStatus {
  $Status = & $Git -C $RepoRoot status --short
  if ($LASTEXITCODE -ne 0) {
    throw "git status failed with exit code $LASTEXITCODE"
  }
  if ($Status.Count -gt 0) {
    throw "tag readiness requires a clean committed tree:`n$($Status -join "`n")"
  }
}

function Test-TagExists {
  param([string] $TagName)

  $null = & $Git -C $RepoRoot rev-parse --quiet --verify "refs/tags/$TagName"
  return $LASTEXITCODE -eq 0
}

function Assert-SemVer {
  param([string] $Version)

  $SemVerPattern = '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'
  if (-not [regex]::IsMatch($Version, $SemVerPattern)) {
    throw "VERSION is not valid SemVer: $Version"
  }
}

$Git = Resolve-Tool 'git' 'git was not found on PATH'

Push-Location $RepoRoot
try {
  Assert-CleanStatus

  $Version = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'VERSION')).Trim()
  Assert-SemVer $Version

  $TagName = "v$Version"
  if (Test-TagExists $TagName) {
    throw "tag $TagName already exists; do not reuse release tags"
  }

  $Head = Read-NativeOutput 'resolve HEAD' $Git @('-C', $RepoRoot, 'rev-parse', '--verify', 'HEAD')
  $ShortHead = Read-NativeOutput 'resolve short HEAD' $Git @('-C', $RepoRoot, 'rev-parse', '--short', 'HEAD')

  Invoke-RepoScript 'full preflight' 'check_all.ps1'
  Invoke-RepoScript 'clean checkout smoke' 'check_clean_checkout.ps1'

  Write-Host "Tag readiness passed for $TagName at $ShortHead."
  Write-Host 'No tag was created and no remote was touched.'
  Write-Host 'Human tag command:'
  Write-Host ("  git tag -a {0} -m `"Hum {1} pre-alpha`"" -f $TagName, $Version)
  Write-Host ("Verified commit: {0}" -f $Head)
} finally {
  Pop-Location
}
