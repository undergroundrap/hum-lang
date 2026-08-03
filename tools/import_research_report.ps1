param(
  [Parameter(Mandatory = $true)]
  [string] $SourcePath,

  [Parameter(Mandatory = $true)]
  [string] $Slug,

  [string] $Date = (Get-Date -Format 'yyyy-MM-dd')
)

$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ResearchDir = Join-Path (Join-Path $RepoRoot 'docs') 'research'
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

  # Zero-width and invisible formatting characters are deleted outright later,
  # so they must be removed before citation context is examined. Otherwise a
  # marker adjacent to one of them sees a non-word character, is removed
  # without a separator, and the invisible character then disappears too,
  # silently fusing the surrounding words. Deleting them first lets the
  # citation logic observe the real word boundaries.
  $invisible = @(0x200B, 0x200C, 0x200D, 0xFEFF)
  foreach ($code in $invisible) {
    $Text = $Text.Replace([string] [char] $code, '')
  }

  $citationStart = [regex]::Escape([string] [char] 0xE200)
  $citationEnd = [regex]::Escape([string] [char] 0xE201)
  # Deep Research citation markers are delimited by U+E200 and U+E201. The
  # inner text varies by export ('cite...', 'filecite...', and separators such
  # as U+E202), so strip the whole delimited span rather than one spelling.
  # The span must not cross a second opening marker: a malformed or
  # unterminated marker would otherwise swallow every character up to a later
  # terminator, deleting legitimate content. An unterminated marker is left in
  # place so the ASCII gate fails closed rather than silently losing text.
  # A citation marker is always inline. The body therefore excludes both a
  # second opening marker and any line break, so a malformed or unterminated
  # marker can never consume text across a line or paragraph boundary. Such
  # input keeps its markers and fails closed at the ASCII gate instead of
  # silently deleting the intervening content.
  $citationBody = '[^' + $citationStart + '\r\n]*?'
  $citationSpan = $citationStart + $citationBody + $citationEnd
  # Markers frequently appear in adjacent runs. The whole run is treated as one
  # unit, because testing spans individually would leave neither span of a pair
  # with word characters on both sides and would then fuse the surrounding
  # words. A run sitting between two word characters becomes a single space;
  # every other position - beside whitespace, punctuation, or a line boundary -
  # is removed outright so ordinary trailing citations leave no stray gap.
  $citationRun = '(?:' + $citationSpan + ')+'
  $Text = [regex]::Replace($Text, ('(?<=\w)' + $citationRun + '(?=\w)'), ' ')
  $Text = [regex]::Replace($Text, $citationRun, '')

  # Research reports cite mathematicians by name. Decompose accented Latin
  # letters and drop the combining marks so names such as Erdos, Poincare, and
  # Mobius survive as ASCII instead of failing the repository ASCII gate.
  $decomposed = $Text.Normalize([Text.NormalizationForm]::FormD)
  $builder = New-Object System.Text.StringBuilder
  foreach ($character in $decomposed.ToCharArray()) {
    $category = [System.Globalization.CharUnicodeInfo]::GetUnicodeCategory($character)
    if ($category -ne [System.Globalization.UnicodeCategory]::NonSpacingMark) {
      [void] $builder.Append($character)
    }
  }
  $Text = $builder.ToString()

  $apostrophe = [string] [char] 39
  $quote = [string] [char] 34
  $replacements = @(
    @{ Code = 0x00F8; Value = 'o' },
    @{ Code = 0x00D8; Value = 'O' },
    @{ Code = 0x00DF; Value = 'ss' },
    @{ Code = 0x00E6; Value = 'ae' },
    @{ Code = 0x00C6; Value = 'AE' },
    @{ Code = 0x0142; Value = 'l' },
    @{ Code = 0x0141; Value = 'L' },
    @{ Code = 0x0111; Value = 'd' },
    @{ Code = 0x0110; Value = 'D' },
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
