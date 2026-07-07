$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ExcludedDirectories = @('.git', 'target')
$TextExtensions = @(
  '.code-workspace',
  '.classpath',
  '.csproj',
  '.factorypath',
  '.fsproj',
  '.hum',
  '.iml',
  '.ini',
  '.ipynb',
  '.json',
  '.jsonc',
  '.launch',
  '.md',
  '.project',
  '.props',
  '.ps1',
  '.rs',
  '.sln',
  '.targets',
  '.toml',
  '.txt',
  '.vbproj',
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
  'NOTICE.md',
  'VERSION'
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

function Test-ExcludedRelativePath {
  param([string] $RelativePath)

  foreach ($directory in $ExcludedDirectories) {
    if ($RelativePath -eq $directory) {
      return $true
    }

    foreach ($separator in @('\', '/')) {
      if ($RelativePath.StartsWith("$directory$separator", [System.StringComparison]::OrdinalIgnoreCase)) {
        return $true
      }
    }
  }

  return $false
}

function Test-ExcludedPath {
  param([System.IO.FileSystemInfo] $Item)

  return Test-ExcludedRelativePath (Get-RepoRelativePath $Item.FullName)
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

function Test-MarkdownLinks {
  param(
    [System.IO.FileInfo] $File,
    [string] $Text
  )

  $matches = [regex]::Matches($Text, '\[[^\]]+\]\(([^)]+)\)')
  foreach ($match in $matches) {
    $target = $match.Groups[1].Value.Trim()
    if ($target.Length -eq 0) {
      continue
    }

    if ($target.StartsWith('<') -and $target.EndsWith('>')) {
      $target = $target.Substring(1, $target.Length - 2)
    }

    if ($target -match '^[a-zA-Z][a-zA-Z0-9+.-]*:' -or $target.StartsWith('#')) {
      continue
    }

    $withoutAnchor = $target.Split('#')[0]
    if ($withoutAnchor.Length -eq 0) {
      continue
    }

    $withoutLine = $withoutAnchor -replace ':\d+$', ''
    $candidate = [System.IO.Path]::GetFullPath((Join-Path $File.DirectoryName $withoutLine))

    if (-not $candidate.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
      Add-Failure $File.FullName ("local Markdown link points outside repo: {0}" -f $target)
      continue
    }

    if (-not (Test-Path -LiteralPath $candidate)) {
      $line = Get-LineNumber $Text $match.Index
      Add-Failure $File.FullName ("broken local Markdown link on line {0}: {1}" -f $line, $target)
    }
  }
}

function New-StringFromCodePoints {
  param([int[]] $CodePoints)

  $builder = New-Object System.Text.StringBuilder
  foreach ($codePoint in $CodePoints) {
    [void] $builder.Append([char] $codePoint)
  }

  return $builder.ToString()
}

$MojibakePatterns = @(
  [string] [char] 0xFFFD,
  (New-StringFromCodePoints @(0x00E2)),
  (New-StringFromCodePoints @(0x00C3)),
  (New-StringFromCodePoints @(0x00C2)),
  (New-StringFromCodePoints @(0x00F0)),
  (New-StringFromCodePoints @(0x00EF, 0x00BB, 0x00BF))
)

foreach ($directory in $ExcludedDirectories) {
  foreach ($separator in @('\', '/')) {
    $samplePath = $directory + $separator + 'sample.txt'
    if (-not (Test-ExcludedRelativePath $samplePath)) {
      throw "Excluded directory check does not handle $samplePath"
    }
  }
}

$files = Get-ChildItem -LiteralPath $RepoRoot -Recurse -File -Force | Where-Object { Test-TextFile $_ }

foreach ($file in $files) {
  $bytes = [System.IO.File]::ReadAllBytes($file.FullName)
  if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
    Add-Failure $file.FullName 'UTF-8 BOM found'
  }

  try {
    $text = $Utf8Strict.GetString($bytes)
  } catch {
    Add-Failure $file.FullName ("invalid UTF-8: {0}" -f $_.Exception.Message)
    continue
  }

  $controlMatch = [regex]::Match($text, '[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]')
  if ($controlMatch.Success) {
    $line = Get-LineNumber $text $controlMatch.Index
    Add-Failure $file.FullName ("control character found on line {0}" -f $line)
  }

  $privateUseMatch = [regex]::Match($text, '[\uE000-\uF8FF]')
  if ($privateUseMatch.Success) {
    $line = Get-LineNumber $text $privateUseMatch.Index
    Add-Failure $file.FullName ("private-use Unicode character found on line {0}" -f $line)
  }

  foreach ($pattern in $MojibakePatterns) {
    $index = $text.IndexOf($pattern, [System.StringComparison]::Ordinal)
    if ($index -ge 0) {
      $line = Get-LineNumber $text $index
      Add-Failure $file.FullName ("suspicious mojibake marker on line {0}" -f $line)
    }
  }

  $escapedNewlinePattern = New-StringFromCodePoints @(0x0060, 0x0072, 0x0060, 0x006E)
  $escapedNewlineIndex = $text.IndexOf($escapedNewlinePattern, [System.StringComparison]::Ordinal)
  if ($escapedNewlineIndex -ge 0) {
    $line = Get-LineNumber $text $escapedNewlineIndex
    Add-Failure $file.FullName ("literal PowerShell newline escape scar on line {0}" -f $line)
  }

  if ($file.Extension -eq '.md') {
    Test-MarkdownLinks $file $text
  }
}

if ($Failures.Count -gt 0) {
  Write-Host 'Text hygiene check failed:'
  foreach ($failure in $Failures) {
    Write-Host ("- {0}" -f $failure)
  }
  exit 1
}

Write-Host ("Text hygiene check passed for {0} files." -f $files.Count)
