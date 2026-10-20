$ErrorActionPreference = 'Stop'
$script:Wo22RustTargetAdmissionOwner = $null
$script:ExactRustSelectorCredits = New-Object 'System.Collections.Generic.List[string]'
$script:ExactRustMutationGenerationOwner = $null

function Reset-ExactRustSelectorCredits {
  $script:ExactRustSelectorCredits.Clear()
}

function Get-ExactRustSelectorCredits {
  return @($script:ExactRustSelectorCredits)
}

function Assert-ExactRustSelectorSyntax {
  param([string] $Selector)

  if ([string]::IsNullOrWhiteSpace($Selector) -or
      -not [regex]::IsMatch($Selector, '^[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+$')) {
    throw "Rust test selector must be one fully qualified test name: '$Selector'"
  }
}

function Invoke-ExactRustNativeCapture {
  param(
    [string] $Cargo,
    [string[]] $Arguments
  )

  if (-not (Test-Path -LiteralPath $Cargo -PathType Leaf)) {
    throw "cargo executable is unavailable: $Cargo"
  }

  $PreviousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $Output = @(& $Cargo @Arguments 2>&1 | ForEach-Object { $_.ToString() })
    $ExitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $PreviousErrorActionPreference
  }

  return [pscustomobject] @{
    Output = $Output
    ExitCode = $ExitCode
  }
}

function Assert-ExactRustSelectorEvidence {
  param(
    [string] $Selector,
    [string[]] $ListOutput,
    [string[]] $RunOutput
  )

  Assert-ExactRustSelectorSyntax $Selector
  $EscapedSelector = [regex]::Escape($Selector)

  $ListedTests = @($ListOutput | Where-Object { $_ -match ': test$' })
  $ExactListings = @($ListedTests | Where-Object { $_ -match "^${EscapedSelector}: test$" })
  if ($ListedTests.Count -ne 1 -or $ExactListings.Count -ne 1) {
    throw "exact Rust selector '$Selector' must list exactly one test; listed $($ListedTests.Count) total and $($ExactListings.Count) exact"
  }

  $RunningCounts = @(
    $RunOutput |
      ForEach-Object {
        if ($_ -match '^running (?<count>[0-9]+) tests?$') {
          [int] $Matches['count']
        }
      }
  )
  if ($RunningCounts.Count -eq 0 -or ($RunningCounts | Measure-Object -Sum).Sum -ne 1) {
    $Total = if ($RunningCounts.Count -eq 0) { 0 } else { ($RunningCounts | Measure-Object -Sum).Sum }
    throw "exact Rust selector '$Selector' must run exactly one test; harness reported $Total"
  }

  $NamedResults = @($RunOutput | Where-Object { $_ -match '^test .+ \.\.\. (?:ok|FAILED|ignored)$' })
  $ExactResults = @($NamedResults | Where-Object { $_ -match "^test ${EscapedSelector} \.\.\. ok$" })
  if ($NamedResults.Count -ne 1 -or $ExactResults.Count -ne 1) {
    throw "exact Rust selector '$Selector' must produce one matching successful result; saw $($NamedResults.Count) named and $($ExactResults.Count) exact"
  }

  $SummaryCounts = @(
    $RunOutput |
      ForEach-Object {
        if ($_ -match '^test result: ok\. (?<passed>[0-9]+) passed; (?<failed>[0-9]+) failed; (?<ignored>[0-9]+) ignored; (?<measured>[0-9]+) measured; (?<filtered>[0-9]+) filtered out;') {
          [pscustomobject] @{
            Passed = [int] $Matches['passed']
            Failed = [int] $Matches['failed']
            Ignored = [int] $Matches['ignored']
            Measured = [int] $Matches['measured']
          }
        }
      }
  )
  $Passed = ($SummaryCounts | Measure-Object -Property Passed -Sum).Sum
  $Failed = ($SummaryCounts | Measure-Object -Property Failed -Sum).Sum
  $Ignored = ($SummaryCounts | Measure-Object -Property Ignored -Sum).Sum
  $Measured = ($SummaryCounts | Measure-Object -Property Measured -Sum).Sum
  if ($SummaryCounts.Count -eq 0 -or $Passed -ne 1 -or $Failed -ne 0 -or $Ignored -ne 0 -or $Measured -ne 0) {
    throw "exact Rust selector '$Selector' has invalid aggregate result: passed=$Passed failed=$Failed ignored=$Ignored measured=$Measured"
  }
}

# Cargo still owns every listing, run, target choice, freshness decision and exit.
# Only WO24 mutations use a private source/target generation; honest calls keep
# the original working directory and warm target. No executable/result is replayed.
function Get-ExactRustGenerationDigest {
  param([byte[]] $Bytes)
  $Hasher = [Security.Cryptography.SHA256]::Create()
  try { return [BitConverter]::ToString($Hasher.ComputeHash($Bytes)) } finally { $Hasher.Dispose() }
}

function New-ExactRustMutationGeneration {
  param([string] $RepoRoot, [string] $Cargo)

  if ($null -ne $script:ExactRustMutationGenerationOwner) { throw 'WO24 Cargo generation is already active' }
  $OwnedRoot = $null
  $OwnedCreated = $false
  try {
    $Root = [IO.Path]::GetFullPath($RepoRoot)
    $Comparison = if ([IO.Path]::DirectorySeparatorChar -eq '\') { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
    if (-not [string]::Equals($Root, [IO.Path]::GetFullPath((Get-Location).Path), $Comparison)) { throw 'working directory differs' }
    $RootInfo = [IO.DirectoryInfo]::new($Root)
    if (-not $RootInfo.Exists -or ($RootInfo.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'nonordinary source root' }
    $Git = @(Get-Command git -CommandType Application -ErrorAction Stop)[0].Source
    $Status = Invoke-ExactRustNativeCapture $Git @('-C', $Root, 'status', '--porcelain=v1', '--untracked-files=all')
    if ($Status.ExitCode -ne 0 -or @($Status.Output | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count -ne 0) { throw 'working tree is not clean' }
    $Tracked = Invoke-ExactRustNativeCapture $Git @('-C', $Root, '-c', 'core.quotePath=true', 'ls-files', '--stage')
    if ($Tracked.ExitCode -ne 0 -or $Tracked.Output.Count -eq 0) { throw 'tracked source inventory unavailable' }
    $Ignored = Invoke-ExactRustNativeCapture $Git @('-C', $Root, 'ls-files', '--others', '--ignored', '--exclude-standard', '--', '.', ':(exclude)target', ':(exclude).cargo-home')
    if ($Ignored.ExitCode -ne 0 -or @($Ignored.Output | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }).Count -ne 0) { throw 'unbound ignored source inputs' }

    $Files = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
    $SourceBytes = [Collections.Generic.Dictionary[string,byte[]]]::new([StringComparer]::Ordinal)
    foreach ($Line in $Tracked.Output) {
      if ($Line -cnotmatch '^100644 [0-9a-f]{40} 0\t(?<path>[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*)$') { throw 'unsupported source inventory shape' }
      $Relative = $Matches['path']
      if ($Relative -match '(^|/)\.\.?(/|$)|(^|/)(?:build\.rs|rust-toolchain(?:\.toml)?)$|^\.cargo/' -or $Relative -match '^(?:target|\.cargo-home|\.git)(?:/|$)' -or $Files.ContainsKey($Relative)) { throw 'unsupported source context' }
      $Path = Join-Path $Root $Relative
      $Info = [IO.FileInfo]::new($Path)
      if (-not $Info.Exists -or ($Info.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'nonordinary source input' }
      $Parent = $Info.Directory
      while (-not [string]::Equals($Parent.FullName, $Root, $Comparison)) {
        if (($Parent.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0 -or $null -eq $Parent.Parent) { throw 'nonordinary source parent' }
        $Parent = $Parent.Parent
      }
      $Bytes = [IO.File]::ReadAllBytes($Path)
      $Files.Add($Relative, (Get-ExactRustGenerationDigest $Bytes))
      $SourceBytes.Add($Relative, $Bytes)
    }
    if (-not $Files.ContainsKey('Cargo.toml') -or -not $Files.ContainsKey('Cargo.lock')) { throw 'root Cargo inputs unavailable' }
    $OwnedRoot = Join-Path ([IO.Path]::GetTempPath()) ('hum-wo24-cargo-' + [Guid]::NewGuid().ToString('N'))
    if ([IO.Directory]::Exists($OwnedRoot)) { throw 'generation directory already exists' }
    $null = [IO.Directory]::CreateDirectory($OwnedRoot)
    $OwnedCreated = $true
    $MutableRoot = Join-Path $OwnedRoot 'source'
    foreach ($Relative in $Files.Keys) {
      $Destination = Join-Path $MutableRoot $Relative
      $null = [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($Destination))
      [IO.File]::WriteAllBytes($Destination, $SourceBytes[$Relative])
    }
    $Generation = [pscustomobject] @{ Root = $Root; MutableRoot = $MutableRoot; OwnedRoot = $OwnedRoot; Target = (Join-Path $OwnedRoot 'target'); Cargo = $Cargo; Files = $Files; Active = $true }
    $BoundFiles = [Collections.Generic.Dictionary[string,string]]::new($Files, [StringComparer]::Ordinal)
    $script:ExactRustMutationGenerationOwner = [pscustomobject] @{ Instance = $Generation; Root = $Root; MutableRoot = $MutableRoot; OwnedRoot = $OwnedRoot; Target = $Generation.Target; Cargo = $Cargo; Files = $BoundFiles }
    Assert-ExactRustMutationGeneration $Generation $Cargo
    Write-Host '==> Work Order 24 uses an invocation-local mutable Cargo generation; honest source and target stay in place'
    return $Generation
  } catch {
    $PreparationFailure = $_
    $script:ExactRustMutationGenerationOwner = $null
    if ($OwnedCreated) {
      # A failed cleanup is not permission to hide residue and restart the suite.
      Remove-Item -LiteralPath $OwnedRoot -Recurse -Force -ErrorAction Stop
    }
    Write-Host ("==> Work Order 24 ordinary Cargo route: generation preparation unavailable ({0})" -f $PreparationFailure.Exception.Message)
    return $null
  }
}

function Assert-ExactRustMutationGeneration {
  param($Generation, [string] $Cargo, [string] $MutationPath = '', [byte[]] $MutationBytes = $null)

  $Owner = $script:ExactRustMutationGenerationOwner
  $WorkingRoot = [IO.Path]::GetFullPath((Get-Location).Path)
  $Comparison = if ([IO.Path]::DirectorySeparatorChar -eq '\') { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
  if ($null -eq $Owner -or -not [object]::ReferenceEquals($Generation, $Owner.Instance) -or -not $Generation.Active -or $Generation.Cargo -cne $Cargo -or
      -not [string]::Equals($WorkingRoot, $Owner.Root, $Comparison) -or $Generation.Root -cne $Owner.Root -or $Generation.MutableRoot -cne $Owner.MutableRoot -or $Generation.OwnedRoot -cne $Owner.OwnedRoot -or $Generation.Target -cne $Owner.Target -or $Cargo -cne $Owner.Cargo -or $Generation.Files.Count -ne $Owner.Files.Count) {
    throw 'WO24 Cargo generation lifetime or context changed'
  }
  if (-not [string]::IsNullOrEmpty($MutationPath) -and (-not $Generation.Files.ContainsKey($MutationPath) -or $null -eq $MutationBytes)) { throw 'WO24 Cargo generation mutation owner changed' }
  foreach ($Relative in $Owner.Files.Keys) {
    $Expected = $Owner.Files[$Relative]
    if (-not $Generation.Files.ContainsKey($Relative) -or $Generation.Files[$Relative] -cne $Expected) { throw 'WO24 Cargo generation input binding changed' }
    if ((Get-ExactRustGenerationDigest ([IO.File]::ReadAllBytes((Join-Path $Generation.Root $Relative)))) -cne $Expected) { throw 'WO24 honest Cargo source changed' }
    $MutableExpected = if ([string]::Equals($Relative, $MutationPath, [StringComparison]::Ordinal)) { Get-ExactRustGenerationDigest $MutationBytes } else { $Expected }
    if ((Get-ExactRustGenerationDigest ([IO.File]::ReadAllBytes((Join-Path $Generation.MutableRoot $Relative)))) -cne $MutableExpected) { throw 'WO24 mutable Cargo source changed' }
  }
}

function Remove-ExactRustMutationGeneration {
  param($Generation)
  if ($null -eq $Generation) { return }
  $Owner = $script:ExactRustMutationGenerationOwner
  if ($null -eq $Owner -or -not [object]::ReferenceEquals($Generation, $Owner.Instance)) { throw 'WO24 generation cleanup ownership changed' }
  $script:ExactRustMutationGenerationOwner = $null
  $Generation.Active = $false
  $Info = [IO.DirectoryInfo]::new($Owner.OwnedRoot)
  if (-not $Info.Exists -or ($Info.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'WO24 generation cleanup directory changed' }
  Remove-Item -LiteralPath $Owner.OwnedRoot -Recurse -Force -ErrorAction Stop
}

# WO22 alone may qualify a mutant after current broad honest admission. This
# invocation-local evidence is not a saved test result or a general target catalog.
function Read-Wo22RustTargetNamespace {
  param([string] $Root, [string] $Cargo)
  $Root = [IO.Path]::GetFullPath($Root)
  if ($Root -cne [IO.Path]::GetFullPath((Get-Location).Path)) { throw 'WO22 target namespace working directory changed' }
  $Files = [Collections.Generic.SortedDictionary[string,string]]::new([StringComparer]::Ordinal)
  $Pending = [Collections.Generic.Queue[IO.DirectoryInfo]]::new()
  $Pending.Enqueue([IO.DirectoryInfo]::new($Root))
  while ($Pending.Count -gt 0) {
    $Directory = $Pending.Dequeue()
    if (-not $Directory.Exists -or ($Directory.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'WO22 target namespace has nonordinary source directory' }
    foreach ($Entry in $Directory.EnumerateFileSystemInfos()) {
      $Relative = $Entry.FullName.Substring($Root.Length + 1).Replace('\','/')
      if ($Relative -cin @('.git','target','.cargo-home')) { continue }
      if (($Entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'WO22 target namespace has nonordinary source input' }
      if (($Entry.Attributes -band [IO.FileAttributes]::Directory) -ne 0) { $Pending.Enqueue([IO.DirectoryInfo] $Entry); continue }
      $Files.Add($Relative, (Get-ExactRustGenerationDigest ([IO.File]::ReadAllBytes($Entry.FullName))))
    }
  }
  foreach ($Required in @('Cargo.toml','Cargo.lock','src/main.rs','src/backend_cranelift.rs','tests/cli_decision0030.rs','tests/cli_wo29_trust_locality.rs')) {
    if (-not $Files.ContainsKey($Required)) { throw 'WO22 target namespace is incomplete' }
  }
  $Manifest = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes((Join-Path $Root 'Cargo.toml')))
  if ($Manifest -cnotmatch '(?m)^\[\[bin\]\]\nname = "hum"\npath = "src/main\.rs"\n' -or $Manifest -match '(?m)^\[lib\]|^\[\[test\]\]|^autotests\s*=|^default-members\s*=') { throw 'WO22 target ownership changed' }
  $Environment = [Collections.Generic.SortedDictionary[string,string]]::new([StringComparer]::Ordinal)
  foreach ($Pair in [Environment]::GetEnvironmentVariables().GetEnumerator()) { $Environment.Add([string] $Pair.Key, [string] $Pair.Value) }
  $Context = [Collections.Generic.List[string]]::new()
  foreach ($Pair in $Environment.GetEnumerator()) { $Context.Add(($Pair.Key.Length.ToString() + ':' + $Pair.Key + $Pair.Value.Length.ToString() + ':' + $Pair.Value)) }
  $Context.Add((Get-ExactRustGenerationDigest ([IO.File]::ReadAllBytes($Cargo))))
  $ConfigPaths = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
  $Parent = [IO.DirectoryInfo]::new($Root)
  while ($null -ne $Parent) {
    foreach ($Leaf in @('.cargo/config','.cargo/config.toml','rust-toolchain','rust-toolchain.toml')) { $null = $ConfigPaths.Add((Join-Path $Parent.FullName $Leaf)) }
    $Parent = $Parent.Parent
  }
  $CargoHome = if ($Environment.ContainsKey('CARGO_HOME')) { $Environment['CARGO_HOME'] } else { '' }
  if ([string]::IsNullOrEmpty($CargoHome)) { $CargoHome = Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cargo' }
  foreach ($Leaf in @('config','config.toml')) { $null = $ConfigPaths.Add((Join-Path $CargoHome $Leaf)) }
  foreach ($Path in @($ConfigPaths | Sort-Object -CaseSensitive)) {
    $Context.Add($Path)
    if ([IO.File]::Exists($Path)) {
      $Bytes = [IO.File]::ReadAllBytes($Path)
      if ([Text.Encoding]::UTF8.GetString($Bytes) -match '(?m)^\s*include\s*=') { throw 'WO22 target namespace cannot bind included Cargo configuration' }
      $Context.Add((Get-ExactRustGenerationDigest $Bytes))
    } else { $Context.Add('absent') }
  }
  return [pscustomobject] @{ Root = $Root; Files = $Files; Target = 'hum'; TargetSource = 'src/main.rs'; Context = (Get-ExactRustGenerationDigest ([Text.Encoding]::UTF8.GetBytes([string]::Join("`n", $Context)))) }
}

function Assert-Wo22RustNamespace {
  param($Expected, $Actual, [string] $BackendDigest)
  if ($Actual.Root -cne $Expected.Root -or $Actual.Context -cne $Expected.Context -or $Actual.Target -cne $Expected.Target -or $Actual.TargetSource -cne $Expected.TargetSource -or $Actual.Files.Count -ne $Expected.Files.Count) { throw 'WO22 target namespace or context changed' }
  foreach ($Pair in $Expected.Files.GetEnumerator()) {
    $Digest = if ($Pair.Key -ceq 'src/backend_cranelift.rs') { $BackendDigest } else { $Pair.Value }
    if (-not $Actual.Files.ContainsKey($Pair.Key) -or $Actual.Files[$Pair.Key] -cne $Digest) { throw 'WO22 target namespace source changed' }
  }
}

function New-Wo22RustTargetAdmission {
  param([string] $Cargo, [string] $BackendPath, [byte[]] $OriginalBytes, [object[]] $Mutations)
  if ($null -ne $script:Wo22RustTargetAdmissionOwner) { throw 'WO22 target admission is already active' }
  $Root = [IO.Path]::GetDirectoryName([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($BackendPath)))
  $Namespace = Read-Wo22RustTargetNamespace $Root $Cargo
  if ($Namespace.Root -cne $Root -or $Namespace.Target -cne 'hum' -or $Namespace.TargetSource -cne 'src/main.rs') { throw 'WO22 target admission ownership changed' }
  foreach ($Required in @('Cargo.toml','Cargo.lock','src/main.rs','src/backend_cranelift.rs','tests/cli_decision0030.rs','tests/cli_wo29_trust_locality.rs')) {
    if (-not $Namespace.Files.ContainsKey($Required)) { throw 'WO22 target admission namespace incomplete' }
  }
  $OriginalDigest = Get-ExactRustGenerationDigest $OriginalBytes
  if ($Namespace.Files['src/backend_cranelift.rs'] -cne $OriginalDigest -or $Mutations.Count -ne 16) { throw 'WO22 target admission source or row inventory changed' }
  $Utf8 = [Text.UTF8Encoding]::new($false, $true)
  $Original = $Utf8.GetString($OriginalBytes)
  $Start = $Original.IndexOf('fn probe_with_fault(', [StringComparison]::Ordinal)
  $End = $Original.IndexOf("`nfn backend_evidence_is_complete(", [StringComparison]::Ordinal)
  if ($Start -lt 0 -or $End -le $Start) { throw 'WO22 target admission production owner missing' }
  $Variants = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
  $Selectors = [Collections.Generic.SortedDictionary[string,string[]]]::new([StringComparer]::Ordinal)
  foreach ($Mutation in $Mutations) {
    if ($Mutation.Selector -cnotin @('backend_cranelift::tests::backend_go_no_go_rows_are_complete_and_load_bearing','backend_cranelift::tests::minimal_add_jit_probe_matrix_is_exact')) { throw 'WO22 target admission selector owner changed' }
    $Changes = @([pscustomobject] @{ Needle = $Mutation.Needle; Replacement = $Mutation.Replacement })
    if ($null -ne $Mutation.PSObject.Properties['AdditionalChanges']) { $Changes += @($Mutation.AdditionalChanges) }
    $Variant = $Original
    foreach ($Change in $Changes) {
      $At = $Original.IndexOf($Change.Needle, [StringComparison]::Ordinal)
      if ($At -lt $Start -or ($At + $Change.Needle.Length) -gt $End -or [regex]::Matches($Original,[regex]::Escape($Change.Needle)).Count -ne 1 -or
          $Change.Needle -cmatch '#|\b(?:fn|mod|extern|include|include_str|include_bytes|macro_rules|test|cfg)\b' -or $Change.Replacement -cmatch '#|\b(?:fn|mod|extern|include|include_str|include_bytes|macro_rules|test|cfg)\b') { throw 'WO22 mutation can change target or test namespace' }
      $Variant = $Variant.Replace($Change.Needle, $Change.Replacement)
    }
    if ($Variant -ceq $Original -or -not $Variant.StartsWith($Original.Substring(0,$Start),[StringComparison]::Ordinal) -or -not $Variant.EndsWith($Original.Substring($End),[StringComparison]::Ordinal)) { throw 'WO22 mutation namespace envelope changed' }
    $Key = 'Work Order 22 ' + $Mutation.Label + ' guarded production mutation|' + $Mutation.Selector
    $Variants.Add($Key, (Get-ExactRustGenerationDigest ($Utf8.GetBytes($Variant))))
    if (-not $Selectors.ContainsKey($Mutation.Selector)) { $Selectors.Add($Mutation.Selector, [string[]]@()) }
  }
  if ($Selectors.Count -ne 2) { throw 'WO22 target admission selector inventory changed' }
  foreach ($Selector in @($Selectors.Keys)) {
    $Result = Invoke-ExactRustNativeCapture $Cargo @('test', $Selector, '--', '--exact', '--list')
    $Listed = @($Result.Output | Where-Object { $_ -match ': test$' })
    if ($Result.ExitCode -ne 0 -or $Listed.Count -ne 1 -or $Listed[0] -cne ($Selector + ': test')) { throw 'WO22 broad honest singleton admission failed' }
    $Selectors[$Selector] = [string[]] $Result.Output.Clone()
    Assert-Wo22RustNamespace $Namespace (Read-Wo22RustTargetNamespace $Root $Cargo) $OriginalDigest
  }
  $Admission = [pscustomobject] @{ Active = $true; Root = $Root; Cargo = $Cargo; Context = $Namespace.Context; Target = $Namespace.Target; Selectors = $Selectors }
  $BoundSelectors = [Collections.Generic.SortedDictionary[string,string[]]]::new([StringComparer]::Ordinal)
  foreach ($Pair in $Selectors.GetEnumerator()) { $BoundSelectors.Add($Pair.Key,[string[]] $Pair.Value.Clone()) }
  $script:Wo22RustTargetAdmissionOwner = [pscustomobject] @{ Instance = $Admission; Namespace = $Namespace; Root = $Root; Cargo = $Cargo; Context = $Namespace.Context; Target = $Namespace.Target; Selectors = $BoundSelectors; Variants = $Variants }
  return $Admission
}

function Assert-Wo22RustTargetAdmission {
  param($Admission, [string] $Label, [string] $Cargo, [string] $Selector, [byte[]] $MutationBytes)
  $Owner = $script:Wo22RustTargetAdmissionOwner
  if ($null -eq $Owner -or -not [object]::ReferenceEquals($Admission,$Owner.Instance) -or -not $Admission.Active -or $Admission.Root -cne $Owner.Root -or $Admission.Cargo -cne $Cargo -or $Cargo -cne $Owner.Cargo -or $Admission.Context -cne $Owner.Context -or $Admission.Target -cne $Owner.Target -or $Admission.Selectors.Count -ne $Owner.Selectors.Count) { throw 'WO22 target admission lifetime or context changed' }
  foreach ($Pair in $Owner.Selectors.GetEnumerator()) {
    if (-not $Admission.Selectors.ContainsKey($Pair.Key) -or [string]::Join("`n",$Admission.Selectors[$Pair.Key]) -cne [string]::Join("`n",$Pair.Value)) { throw 'WO22 broad admission facts changed' }
  }
  $Key = $Label + '|' + $Selector
  if (-not $Owner.Selectors.ContainsKey($Selector) -or -not $Owner.Variants.ContainsKey($Key) -or $null -eq $MutationBytes) { throw 'WO22 target admission mutation owner changed' }
  $Digest = Get-ExactRustGenerationDigest $MutationBytes
  if ($Digest -cne $Owner.Variants[$Key]) { throw 'WO22 target admission variant changed' }
  Assert-Wo22RustNamespace $Owner.Namespace (Read-Wo22RustTargetNamespace $Owner.Root $Cargo) $Digest
}

function Close-Wo22RustTargetAdmission {
  param($Admission)
  if ($null -ne $Admission) { $Admission.Active = $false }
  $script:Wo22RustTargetAdmissionOwner = $null
}

function Invoke-ExactRustTest {
  param(
    [string] $Label,
    [string] $Cargo,
    [string] $Selector,
    $Generation = $null,
    [string] $MutationPath = '',
    [byte[]] $MutationBytes = $null,
    $Wo22Admission = $null,
    [byte[]] $Wo22MutationBytes = $null
  )

  Write-Host "==> $Label"
  Assert-ExactRustSelectorSyntax $Selector
  $UseWo22Target = $PSBoundParameters.ContainsKey('Wo22Admission')

  $SelectorPhasePrefix = ''
  try {
    # Only the current WO24 row/role owners receive nested diagnostics.
    $Rows = @(
      @('N01 program-neutral feature identity', 'n01', 'native_program::tests::native_feature_discrimination_is_typed_and_load_bearing'),
      @('N02 exact constant-Text body admission', 'n02', 'backend_input::tests::canonical_hello_world_backend_input_is_exact_and_nonforgeable'),
      @('N03 v2 live artifact equality', 'n03', 'ir_verify::tests::hello_world_artifact_rejection_matrix_is_complete'),
      @('N04 verified source literal getter', 'n04', 'backend_cranelift::tests::hello_world_lowering_is_source_driven_and_load_bearing'),
      @('N05 exact finalized invocation evidence', 'n05', 'backend_cranelift::tests::hello_world_lowering_is_source_driven_and_load_bearing'),
      @('N06 deny-first operator consent', 'n06', 'main::tests::native_hello_world_run_is_authority_bound_and_platform_exact'),
      @('N07 native rejection cannot fall back', 'n07', 'native_program::tests::native_feature_discrimination_is_typed_and_load_bearing'),
      @('N08 required target admission', 'n08', 'backend_cranelift::tests::hello_world_lowering_is_source_driven_and_load_bearing')
    )
    $Roles = @(
      @('honest selector before mutation', 'before'),
      @('initialized mutation', 'mutation'),
      @('honest selector after restoration', 'after')
    )
    foreach ($Row in $Rows) {
      foreach ($Role in $Roles) {
        if ([string]::Equals($Label, ('Work Order 24 ' + $Row[0] + ' ' + $Role[0]), [StringComparison]::Ordinal) -and
            [string]::Equals($Selector, $Row[2], [StringComparison]::Ordinal)) {
          $SelectorPhasePrefix = 'compiler-front-wo24-' + $Row[1] + '-' + $Role[1]
        }
      }
    }
  } catch { $SelectorPhasePrefix = '' }
  $WriteSelectorPhaseMarker = {
    param([string] $Part, [string] $Event, [string] $Outcome = '')
    if ([string]::IsNullOrEmpty($SelectorPhasePrefix)) { return }
    try { $null = Write-HumPhaseMarker ($SelectorPhasePrefix + '-' + $Part) $Event $Outcome } catch {
      # Diagnostics are optional, including when this helper is used standalone.
    }
  }

  $ListPhaseOutcome = 'failed'
  $null = & $WriteSelectorPhaseMarker 'list' 'start'
  try {
    $CargoArguments = @('test', $Selector)
    if ($UseWo22Target) {
      if ($null -ne $Generation) { throw 'WO22 target admission cannot mix with another generation' }
      Assert-Wo22RustTargetAdmission $Wo22Admission $Label $Cargo $Selector $Wo22MutationBytes
      $CargoArguments += @('--bin', $Wo22Admission.Target)
    }
    if ($null -ne $Generation) {
      Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes
      if (-not [string]::IsNullOrEmpty($MutationPath)) { $CargoArguments += @('--manifest-path', (Join-Path $Generation.MutableRoot 'Cargo.toml'), '--target-dir', $Generation.Target, '--config', 'build.incremental=true') }
    }
    $CargoArguments += @('--', '--exact', '--list')
    $ListResult = Invoke-ExactRustNativeCapture $Cargo $CargoArguments
    if ($UseWo22Target) { Assert-Wo22RustTargetAdmission $Wo22Admission $Label $Cargo $Selector $Wo22MutationBytes }
    if ($null -ne $Generation) { Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes }
    if ($ListResult.ExitCode -ne 0) {
      $ListResult.Output | ForEach-Object { Write-Host $_ }
      throw "$Label could not list '$Selector'; cargo exited $($ListResult.ExitCode)"
    }

    $EscapedSelector = [regex]::Escape($Selector)
    $ListedTests = @($ListResult.Output | Where-Object { $_ -match ': test$' })
    $ExactListings = @($ListedTests | Where-Object { $_ -match "^${EscapedSelector}: test$" })
    if ($ListedTests.Count -ne 1 -or $ExactListings.Count -ne 1) {
      $ListResult.Output | ForEach-Object { Write-Host $_ }
      throw "$Label must resolve '$Selector' to exactly one test before execution; listed $($ListedTests.Count) total and $($ExactListings.Count) exact"
    }
    $ListPhaseOutcome = 'completed'
  } finally { $null = & $WriteSelectorPhaseMarker 'list' 'end' $ListPhaseOutcome }

  $RunPhaseOutcome = 'failed'
  $null = & $WriteSelectorPhaseMarker 'run' 'start'
  try {
    $CargoArguments = @('test', $Selector)
    if ($UseWo22Target) {
      if ($null -ne $Generation) { throw 'WO22 target admission cannot mix with another generation' }
      Assert-Wo22RustTargetAdmission $Wo22Admission $Label $Cargo $Selector $Wo22MutationBytes
      $CargoArguments += @('--bin', $Wo22Admission.Target)
    }
    if ($null -ne $Generation) {
      Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes
      if (-not [string]::IsNullOrEmpty($MutationPath)) { $CargoArguments += @('--manifest-path', (Join-Path $Generation.MutableRoot 'Cargo.toml'), '--target-dir', $Generation.Target, '--config', 'build.incremental=true') }
    }
    $CargoArguments += @('--', '--exact')
    $RunResult = Invoke-ExactRustNativeCapture $Cargo $CargoArguments
    if ($UseWo22Target) { Assert-Wo22RustTargetAdmission $Wo22Admission $Label $Cargo $Selector $Wo22MutationBytes }
    if ($null -ne $Generation) { Assert-ExactRustMutationGeneration $Generation $Cargo $MutationPath $MutationBytes }
    $RunResult.Output | ForEach-Object { Write-Host $_ }
    if ($RunResult.ExitCode -ne 0) {
      throw "$Label failed with exit code $($RunResult.ExitCode)"
    }

    Assert-ExactRustSelectorEvidence $Selector $ListResult.Output $RunResult.Output
    $script:ExactRustSelectorCredits.Add($Selector)
    $RunPhaseOutcome = 'completed'
  } finally { $null = & $WriteSelectorPhaseMarker 'run' 'end' $RunPhaseOutcome }
}

function Assert-ExactRustSelectorThrows {
  param(
    [string] $Label,
    [scriptblock] $Action
  )

  $Threw = $false
  try {
    & $Action
  } catch {
    $Threw = $true
  }
  if (-not $Threw) {
    throw "exact Rust selector sabotage stayed green: $Label"
  }
}

function Invoke-ExactRustSelectorSelfTests {
  param([string] $Cargo)

  Write-Host '==> exact Rust selector guard tests'
  $Selector = 'tests::selected'
  $ValidList = @('tests::selected: test', '1 test, 0 benchmarks')
  $ValidRun = @(
    'running 1 test',
    'test tests::selected ... ok',
    '',
    'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
  )
  Assert-ExactRustSelectorEvidence $Selector $ValidList $ValidRun

  foreach ($Malformed in @('', 'tests', 'tests::*', 'tests::selected extra', '::tests::selected', 'tests::selected::')) {
    Assert-ExactRustSelectorThrows "malformed selector '$Malformed'" {
      Assert-ExactRustSelectorSyntax $Malformed
    }
  }
  Assert-ExactRustSelectorThrows 'selected test deleted or renamed' {
    Assert-ExactRustSelectorEvidence $Selector @('tests::selected_renamed: test') $ValidRun
  }
  Assert-ExactRustSelectorThrows 'nonexistent selector' {
    Assert-ExactRustSelectorEvidence 'tests::missing' @() @()
  }
  Assert-ExactRustSelectorThrows 'ambiguous duplicate exact selector' {
    Assert-ExactRustSelectorEvidence $Selector @('tests::selected: test', 'tests::selected: test') $ValidRun
  }
  Assert-ExactRustSelectorThrows 'filtered selector ran zero tests' {
    Assert-ExactRustSelectorEvidence $Selector $ValidList @(
      'running 0 tests',
      'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s'
    )
  }
  Assert-ExactRustSelectorThrows 'duplicate test execution' {
    Assert-ExactRustSelectorEvidence $Selector $ValidList @(
      'running 2 tests',
      'test tests::selected ... ok',
      'test tests::selected ... ok',
      'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    )
  }
  Assert-ExactRustSelectorThrows 'unavailable cargo' {
    Invoke-ExactRustTest 'unavailable cargo sabotage' (Join-Path ([System.IO.Path]::GetTempPath()) "missing-cargo-$([guid]::NewGuid().ToString('N'))") $Selector
  }

  $TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "hum-exact-selector-$([guid]::NewGuid().ToString('N'))"
  $OriginalLocation = Get-Location
  try {
    [System.IO.Directory]::CreateDirectory((Join-Path $TempRoot 'src/bin')) | Out-Null
    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'Cargo.toml'),
      "[package]`nname = `"exact-selector-probe`"`nversion = `"0.0.0`"`nedition = `"2021`"`n",
      (New-Object System.Text.UTF8Encoding($false))
    )
    $SelectedSource = "#[cfg(test)]`nmod tests {`n    #[test]`n    fn selected() {}`n}`n`nfn main() {}`n"
    $RenamedSource = $SelectedSource.Replace('fn selected()', 'fn selected_renamed()')
    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/first.rs'),
      $SelectedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    Push-Location $TempRoot

    Invoke-ExactRustTest 'exact selector positive control' $Cargo $Selector

    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/first.rs'),
      $RenamedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    Assert-ExactRustSelectorThrows 'real selected test rename' {
      Invoke-ExactRustTest 'real selected test rename sabotage' $Cargo $Selector
    }
    Assert-ExactRustSelectorThrows 'real nonexistent selection' {
      Invoke-ExactRustTest 'real nonexistent selector sabotage' $Cargo 'tests::missing'
    }

    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/first.rs'),
      $SelectedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    [System.IO.File]::WriteAllText(
      (Join-Path $TempRoot 'src/bin/second.rs'),
      $SelectedSource,
      (New-Object System.Text.UTF8Encoding($false))
    )
    Assert-ExactRustSelectorThrows 'real duplicate exact selection across test binaries' {
      Invoke-ExactRustTest 'real duplicate exact selector sabotage' $Cargo $Selector
    }
  } finally {
    while ((Get-Location).Path -ne $OriginalLocation.Path) {
      Pop-Location
    }
    if (Test-Path -LiteralPath $TempRoot) {
      Remove-Item -LiteralPath $TempRoot -Recurse -Force
    }
  }

  Write-Host 'exact Rust selector guard tests passed'
}

if ($MyInvocation.InvocationName -ne '.') {
  $CargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
  if ($null -ne $CargoCommand) {
    $CargoPath = $CargoCommand.Source
  } else {
    $CargoPath = Join-Path ([Environment]::GetFolderPath('UserProfile')) '.cargo/bin/cargo.exe'
  }
  Invoke-ExactRustSelectorSelfTests $CargoPath
}
