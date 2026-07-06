$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ExcludedDirectories = @('.git', 'target')
$TextExtensions = @(
  '.hum',
  '.md',
  '.ps1',
  '.rs',
  '.toml',
  '.txt'
)
$TextFileNames = @(
  '.gitattributes',
  '.gitignore',
  'Cargo.lock',
  'LICENSE',
  'NOTICE.md'
)

$Utf8Strict = New-Object System.Text.UTF8Encoding($false, $true)
$Failures = New-Object System.Collections.Generic.List[string]

function Get-RepoRelativePath {
  param([string] $Path)

  $fullPath = [System.IO.Path]::GetFullPath($Path)
  if ($fullPath.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    return $fullPath.Substring($RepoRoot.Length).TrimStart('\', '/')
  }

  return $fullPath
}

function Add-Failure {
  param(
    [string] $Path,
    [string] $Message
  )

  $Failures.Add(("{0}: {1}" -f (Get-RepoRelativePath $Path), $Message))
}

function Test-ExcludedPath {
  param([System.IO.FileSystemInfo] $Item)

  $relative = Get-RepoRelativePath $Item.FullName
  foreach ($directory in $ExcludedDirectories) {
    if ($relative -eq $directory -or $relative.StartsWith("$directory\", [System.StringComparison]::OrdinalIgnoreCase)) {
      return $true
    }
  }

  return $false
}

function Test-TextFile {
  param([System.IO.FileInfo] $File)

  if (Test-ExcludedPath $File) {
    return $false
  }

  if ($TextFileNames -contains $File.Name) {
    return $true
  }

  return $TextExtensions -contains $File.Extension
}

function Get-LineNumber {
  param(
    [string] $Text,
    [int] $Index
  )

  if ($Index -lt 0) {
    return 1
  }

  $line = 1
  for ($i = 0; $i -lt $Index; $i++) {
    if ($Text[$i] -eq "`n") {
      $line++
    }
  }

  return $line
}

$UserProfileEnvName = '$env:USER' + 'PROFILE'
$PrivateProjectRoot = 'Anti' + 'gravity' + 'Projects'
$PendingRepoPlaceholder = 'repository URL ' + 'pending'
$LocalMachinePhrase = 'On this ' + 'machine'
$LocalMachinePhraseLower = 'on this ' + 'machine'
$NamedMachinePhrase = 'Ocean' + [char]39 + 's ' + 'machine'
$NamedMainMachinePhrase = 'Ocean' + [char]39 + 's main ' + 'machine'
$NoReplyNumericId = '204' + '957' + '658'
$NoReplyHandle = 'under' + 'ground' + 'rap'
$NoReplyDomain = 'users' + [char]46 + 'noreply' + [char]46 + 'github' + [char]46 + 'com'

$PublicBlockers = @(
  @{ Name = 'Windows user home path'; Pattern = '[A-Za-z]:[\\/]Users[\\/][^\\/\s)<>]+' },
  @{ Name = 'private project root path'; Pattern = $PrivateProjectRoot },
  @{ Name = 'hardcoded user-profile cargo path'; Pattern = [regex]::Escape($UserProfileEnvName) + '\\\.cargo' },
  @{ Name = 'pending repository placeholder'; Pattern = $PendingRepoPlaceholder },
  @{ Name = 'local machine phrasing'; Pattern = (($LocalMachinePhrase, $LocalMachinePhraseLower, $NamedMachinePhrase, $NamedMainMachinePhrase | ForEach-Object { [regex]::Escape($_) }) -join '|') },
  @{ Name = 'GitHub account identity artifact'; Pattern = (($NoReplyNumericId, $NoReplyHandle, $NoReplyDomain | ForEach-Object { [regex]::Escape($_) }) -join '|') }
)
$files = Get-ChildItem -LiteralPath $RepoRoot -Recurse -File -Force | Where-Object { Test-TextFile $_ }

foreach ($file in $files) {
  try {
    $text = $Utf8Strict.GetString([System.IO.File]::ReadAllBytes($file.FullName))
  } catch {
    Add-Failure $file.FullName ("cannot scan invalid UTF-8 file: {0}" -f $_.Exception.Message)
    continue
  }

  foreach ($blocker in $PublicBlockers) {
    $match = [regex]::Match($text, $blocker.Pattern)
    if ($match.Success) {
      $line = Get-LineNumber $text $match.Index
      Add-Failure $file.FullName ("public-readiness blocker on line {0}: {1}" -f $line, $blocker.Name)
    }
  }
}

if ($Failures.Count -gt 0) {
  Write-Host 'Public readiness check failed:'
  foreach ($failure in $Failures) {
    Write-Host ("- {0}" -f $failure)
  }
  exit 1
}

Write-Host ("Public readiness check passed for {0} files." -f $files.Count)