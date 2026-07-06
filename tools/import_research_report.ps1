param(
  [Parameter(Mandatory = $true)]
  [string] $SourcePath,

  [Parameter(Mandatory = $true)]
  [string] $Slug,

  [string] $Date = (Get-Date -Format 'yyyy-MM-dd')
)

$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ResearchDir = Join-Path $RepoRoot 'docs\research'
$Utf8Strict = New-Object System.Text.UTF8Encoding($false, $true)
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Get-LineNumber {
  param(
    [string] $Text,
    [int] $Index
  )

  $line = 1
  for ($i = 0; $i -lt $Index; $i++) {
    if ($Text[$i] -eq "`n") {
      $line++
    }
  }

  return $line
}

function Convert-ToRepoText {
  param([string] $Text)

  $citationStart = [regex]::Escape([string] [char] 0xE200)
  $citationEnd = [regex]::Escape([string] [char] 0xE201)
  $Text = [regex]::Replace($Text, ($citationStart + 'cite.*?' + $citationEnd), '')

  $apostrophe = [string] [char] 39
  $quote = [string] [char] 34
  $replacements = @(
    @{ Code = 0x2018; Value = $apostrophe },
    @{ Code = 0x2019; Value = $apostrophe },
    @{ Code = 0x201A; Value = $apostrophe },
    @{ Code = 0x201B; Value = $apostrophe },
    @{ Code = 0x201C; Value = $quote },
    @{ Code = 0x201D; Value = $quote },
    @{ Code = 0x201E; Value = $quote },
    @{ Code = 0x201F; Value = $quote },
    @{ Code = 0x2010; Value = '-' },
    @{ Code = 0x2011; Value = '-' },
    @{ Code = 0x2012; Value = '-' },
    @{ Code = 0x2013; Value = '-' },
    @{ Code = 0x2014; Value = '-' },
    @{ Code = 0x2015; Value = '-' },
    @{ Code = 0x2212; Value = '-' },
    @{ Code = 0x2026; Value = '...' },
    @{ Code = 0x2022; Value = '-' },
    @{ Code = 0x25CF; Value = '-' },
    @{ Code = 0x00A0; Value = ' ' },
    @{ Code = 0x2190; Value = '<-' },
    @{ Code = 0x2192; Value = '->' },
    @{ Code = 0x2264; Value = '<=' },
    @{ Code = 0x2265; Value = '>=' },
    @{ Code = 0x00D7; Value = 'x' },
    @{ Code = 0x200B; Value = '' },
    @{ Code = 0x200C; Value = '' },
    @{ Code = 0x200D; Value = '' },
    @{ Code = 0xFEFF; Value = '' }
  )

  foreach ($replacement in $replacements) {
    $Text = $Text.Replace(([string] [char] $replacement.Code), $replacement.Value)
  }

  $carriageReturn = [string] [char] 13
  $lineFeed = [string] [char] 10
  $Text = $Text.Replace(($carriageReturn + $lineFeed), $lineFeed)
  $Text = $Text.Replace($carriageReturn, $lineFeed)
  $Text = [regex]::Replace($Text, '[ \t]+(?=\n)', '')

  for ($i = 0; $i -lt $Text.Length; $i++) {
    $code = [int] [char] $Text[$i]
    $isAllowed = $code -eq 9 -or $code -eq 10 -or ($code -ge 32 -and $code -le 126)
    if (-not $isAllowed) {
      $line = Get-LineNumber $Text $i
      throw ("Unsupported non-ASCII character U+{0:X4} on line {1}" -f $code, $line)
    }
  }

  return ($Text.TrimEnd() + "`n")
}

if ($Date -notmatch '^\d{4}-\d{2}-\d{2}$') {
  throw 'Date must use yyyy-MM-dd format.'
}

$safeSlug = $Slug.ToLowerInvariant() -replace '[^a-z0-9]+', '-'
$safeSlug = $safeSlug.Trim('-')
if ([string]::IsNullOrWhiteSpace($safeSlug)) {
  throw 'Slug must contain at least one ASCII letter or digit.'
}

$resolvedSource = (Resolve-Path -LiteralPath $SourcePath).Path
$raw = $Utf8Strict.GetString([System.IO.File]::ReadAllBytes($resolvedSource))
$body = Convert-ToRepoText $raw

if (-not (Test-Path -LiteralPath $ResearchDir)) {
  [void] (New-Item -ItemType Directory -Path $ResearchDir)
}

$outputPath = Join-Path $ResearchDir ("{0}-{1}.md" -f $Date, $safeSlug)
$resolvedOutput = [System.IO.Path]::GetFullPath($outputPath)
if (-not $resolvedOutput.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
  throw "Refusing to write outside repo root: $resolvedOutput"
}

$note = @(
  '<!--',
  ("Research artifact imported on {0}." -f $Date),
  'Normalization: explicit UTF-8 decode, Deep Research UI citation markers stripped, typographic punctuation converted to ASCII, saved as UTF-8 without BOM.',
  'Source names are preserved, but citation-only evidence cells may be blank; future runs should request direct source URLs in the Markdown body.',
  '-->',
  ''
) -join "`n"

[System.IO.File]::WriteAllText($resolvedOutput, ($note + $body.TrimStart()), $Utf8NoBom)

$relative = $resolvedOutput.Substring($RepoRoot.Length).TrimStart('\', '/')
Write-Host ("Imported research report to {0}" -f $relative)
