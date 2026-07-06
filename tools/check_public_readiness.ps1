$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ExcludedDirectories = @('.git', 'target')
$TextExtensions = @(
  '.code-workspace',
  '.hum',
  '.iml',
  '.ini',
  '.json',
  '.jsonc',
  '.md',
  '.ps1',
  '.rs',
  '.toml',
  '.txt',
  '.xml',
  '.yaml',
  '.yml'
)
$TextFileNames = @(
  '.editorconfig',
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

function Join-LiteralPattern {
  param([string[]] $Values)

  return (($Values | ForEach-Object { [regex]::Escape($_) }) -join '|')
}

$Slash = [string] [char] 47
$Backslash = [string] [char] 92
$Dot = [string] [char] 46
$SingleQuote = [string] [char] 39

$ShellWinCargoHome = '$env:USER' + 'PROFILE'
$WinProfileEnv = '%' + 'USER' + 'PROFILE' + '%'
$ShellHomeEnv = '$' + 'HOME'
$WinRoamEnv = '%' + 'APP' + 'DATA' + '%'
$WinLocalRoamEnv = '%' + 'LOCAL' + 'APP' + 'DATA' + '%'
$PrivateProjectRoot = 'Anti' + 'gravity' + 'Projects'
$PendingRepoPlaceholder = 'repository URL ' + 'pending'
$LocalMachinePhrase = 'On this ' + 'machine'
$LocalMachinePhraseLower = 'on this ' + 'machine'
$NamedMachinePhrase = 'Ocean' + $SingleQuote + 's ' + 'machine'
$NamedMainMachinePhrase = 'Ocean' + $SingleQuote + 's main ' + 'machine'
$NoReplyNumericId = '204' + '957' + '658'
$NoReplyHandle = 'under' + 'ground' + 'rap'
$NoReplyDomain = 'users' + $Dot + 'noreply' + $Dot + 'github' + $Dot + 'com'
$SyncMarkerA = 'One' + 'Drive'
$SyncMarkerB = 'Drop' + 'box'
$SyncMarkerC = 'iCloud' + 'Drive'
$WinInstallMarkerA = 'App' + 'Data'
$WinInstallMarkerB = 'Program ' + 'Files'

$VsCodeDir = $Dot + 'vscode'
$CursorDir = $Dot + 'cursor'
$JetBrainsDir = $Dot + 'idea'
$VisualStudioDir = $Dot + 'vs'
$FleetDir = $Dot + 'fleet'
$CodeWorkspaceExt = $Dot + 'code-workspace'
$JetBrainsModuleExt = $Dot + 'iml'
$PathSeparatorPattern = '[' + [regex]::Escape($Backslash) + [regex]::Escape($Slash) + ']'
$PathBoundary = '(^|' + $PathSeparatorPattern + ')'
$PathEnd = '(' + $PathSeparatorPattern + '|$)'

$PublicPathBlockers = @(
  @{ Name = 'VS Code or Cursor workspace directory'; Pattern = $PathBoundary + '(' + (Join-LiteralPattern @($VsCodeDir, $CursorDir)) + ')' + $PathEnd },
  @{ Name = 'JetBrains project directory'; Pattern = $PathBoundary + [regex]::Escape($JetBrainsDir) + $PathEnd },
  @{ Name = 'Visual Studio user-state directory'; Pattern = $PathBoundary + [regex]::Escape($VisualStudioDir) + $PathEnd },
  @{ Name = 'Fleet project directory'; Pattern = $PathBoundary + [regex]::Escape($FleetDir) + $PathEnd },
  @{ Name = 'editor workspace or module file'; Pattern = '(' + (Join-LiteralPattern @($CodeWorkspaceExt, $JetBrainsModuleExt)) + ')$' }
)

$PathPrefix = '(^|[\s"' + $SingleQuote + '(=])'
$NotPathSeparatorOrSpace = '[^' + [regex]::Escape($Backslash) + [regex]::Escape($Slash) + '\s]+'
$WindowsDrivePathPattern = $PathPrefix + '[A-Za-z]:' + $PathSeparatorPattern
$UncPathPattern = [regex]::Escape($Backslash + $Backslash) + $NotPathSeparatorOrSpace + $PathSeparatorPattern
$UnixHomePathPattern = $PathPrefix + [regex]::Escape($Slash) + '(Users|home)' + [regex]::Escape($Slash) + '[^' + [regex]::Escape($Slash) + '\s)<>]+'
$WslWindowsHomePattern = [regex]::Escape($Slash + 'mnt' + $Slash) + '[A-Za-z]' + [regex]::Escape($Slash + 'Users' + $Slash) + '[^' + [regex]::Escape($Slash) + '\s)<>]+'
$TildeHomePattern = $PathPrefix + '~' + $PathSeparatorPattern

$PublicContentBlockers = @(
  @{ Name = 'Windows drive-root absolute path'; Pattern = $WindowsDrivePathPattern },
  @{ Name = 'Windows network-share absolute path'; Pattern = $UncPathPattern },
  @{ Name = 'macOS or Linux home absolute path'; Pattern = $UnixHomePathPattern },
  @{ Name = 'WSL Windows home absolute path'; Pattern = $WslWindowsHomePattern },
  @{ Name = 'tilde home path'; Pattern = $TildeHomePattern },
  @{ Name = 'hardcoded user-profile Cargo path'; Pattern = [regex]::Escape($ShellWinCargoHome) + '\\\.cargo' },
  @{ Name = 'user-profile environment path'; Pattern = Join-LiteralPattern @($WinProfileEnv, $ShellHomeEnv) },
  @{ Name = 'application data environment path'; Pattern = Join-LiteralPattern @($WinRoamEnv, $WinLocalRoamEnv) },
  @{ Name = 'private project root path'; Pattern = [regex]::Escape($PrivateProjectRoot) },
  @{ Name = 'pending repository placeholder'; Pattern = [regex]::Escape($PendingRepoPlaceholder) },
  @{ Name = 'local machine phrasing'; Pattern = Join-LiteralPattern @($LocalMachinePhrase, $LocalMachinePhraseLower, $NamedMachinePhrase, $NamedMainMachinePhrase) },
  @{ Name = 'GitHub account identity artifact'; Pattern = Join-LiteralPattern @($NoReplyNumericId, $NoReplyHandle, $NoReplyDomain) },
  @{ Name = 'personal sync-folder path marker'; Pattern = Join-LiteralPattern @($SyncMarkerA, $SyncMarkerB, $SyncMarkerC) },
  @{ Name = 'Windows per-user install path marker'; Pattern = Join-LiteralPattern @($WinInstallMarkerA, $WinInstallMarkerB) }
)
$files = Get-ChildItem -LiteralPath $RepoRoot -Recurse -File -Force | Where-Object { Test-TextFile $_ }

foreach ($file in $files) {
  $relativePath = Get-RepoRelativePath $file.FullName
  foreach ($blocker in $PublicPathBlockers) {
    if ([regex]::IsMatch($relativePath, $blocker.Pattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)) {
      Add-Failure $file.FullName ("public-readiness path blocker: {0}" -f $blocker.Name)
    }
  }

  try {
    $text = $Utf8Strict.GetString([System.IO.File]::ReadAllBytes($file.FullName))
  } catch {
    Add-Failure $file.FullName ("cannot scan invalid UTF-8 file: {0}" -f $_.Exception.Message)
    continue
  }

  foreach ($blocker in $PublicContentBlockers) {
    $match = [regex]::Match($text, $blocker.Pattern, [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
    if ($match.Success) {
      $line = Get-LineNumber $text $match.Index
      Add-Failure $file.FullName ("public-readiness content blocker on line {0}: {1}" -f $line, $blocker.Name)
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