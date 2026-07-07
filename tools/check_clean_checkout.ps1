param(
  [string] $WorkRoot = '',
  [switch] $KeepClone
)

$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if ($WorkRoot -eq '') {
  $WorkRoot = Join-Path (Join-Path $RepoRoot 'target') 'clean-checkout'
}

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

function Assert-CleanStatus {
  param(
    [string] $Label,
    [string] $Path
  )

  $Status = & $Git -C $Path status --short
  if ($LASTEXITCODE -ne 0) {
    throw "$Label git status failed with exit code $LASTEXITCODE"
  }
  if ($Status.Count -gt 0) {
    throw "$Label is not clean; this smoke tests committed HEAD only:`n$($Status -join "`n")"
  }
}

function Remove-CreatedClone {
  param(
    [string] $ClonePath,
    [string] $AllowedRoot
  )

  $ResolvedClone = [System.IO.Path]::GetFullPath($ClonePath)
  $ResolvedAllowedRoot = [System.IO.Path]::GetFullPath($AllowedRoot)
  $Boundary = $ResolvedAllowedRoot.TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar) + [System.IO.Path]::DirectorySeparatorChar

  if (-not $ResolvedClone.StartsWith($Boundary, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to remove clone outside work root: $ResolvedClone"
  }

  Remove-Item -LiteralPath $ResolvedClone -Recurse -Force
}

$Git = Resolve-Tool 'git' 'git was not found on PATH'

Assert-CleanStatus 'source checkout' $RepoRoot

$Head = Read-NativeOutput 'resolve source HEAD' $Git @('-C', $RepoRoot, 'rev-parse', '--verify', 'HEAD')
$ShortHead = Read-NativeOutput 'resolve short HEAD' $Git @('-C', $RepoRoot, 'rev-parse', '--short', 'HEAD')

$WorkRoot = [System.IO.Path]::GetFullPath($WorkRoot)
New-Item -ItemType Directory -Force -Path $WorkRoot | Out-Null

$Suffix = [System.Guid]::NewGuid().ToString('N').Substring(0, 8)
$CloneRoot = Join-Path $WorkRoot ("hum-clean-{0}-{1}" -f $ShortHead, $Suffix)

try {
  Invoke-Native 'clone committed repo without local hardlinks' $Git @('clone', '--no-local', '--quiet', $RepoRoot, $CloneRoot)
  Invoke-Native 'checkout exact source HEAD' $Git @('-C', $CloneRoot, 'checkout', '--quiet', $Head)
  Assert-CleanStatus 'fresh clone' $CloneRoot

  $CloneHead = Read-NativeOutput 'resolve clone HEAD' $Git @('-C', $CloneRoot, 'rev-parse', '--verify', 'HEAD')
  if ($CloneHead -ne $Head) {
    throw "fresh clone HEAD $CloneHead does not match source HEAD $Head"
  }

  Push-Location $CloneRoot
  try {
    Invoke-Native 'fresh clone preflight' (Join-Path $CloneRoot 'tools/check_all.ps1') @()
  } finally {
    Pop-Location
  }

  Write-Host "Clean checkout smoke passed for $ShortHead."
} finally {
  if ($KeepClone) {
    Write-Host "Kept clean checkout at $CloneRoot"
  } elseif (Test-Path -LiteralPath $CloneRoot) {
    Remove-CreatedClone $CloneRoot $WorkRoot
  }
}
