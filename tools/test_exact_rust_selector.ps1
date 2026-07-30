$ErrorActionPreference = 'Stop'
$script:ExactRustSelectorCredits = New-Object 'System.Collections.Generic.List[string]'
$script:RootRustTestCreditEnabled = $false
$script:RootRustDeclaredSelectors = @()
$script:RootRustListOutput = @()
$script:RootRustRunOutput = @()
$script:RootRustEvidenceReady = $false
$script:ExactRustEvidenceConfiguration = $null
$script:ExactRustSelectorConclusions = New-Object 'System.Collections.Generic.List[object]'

function Reset-ExactRustSelectorCredits {
  $script:ExactRustSelectorCredits.Clear()
  $script:ExactRustSelectorConclusions.Clear()
}

function Get-ExactRustSelectorCredits {
  return $script:ExactRustSelectorCredits.ToArray()
}

function Get-ExactRustSelectorConclusions {
  return $script:ExactRustSelectorConclusions.ToArray()
}

function Get-ExactRustEvidenceConfiguration {
  return $script:ExactRustEvidenceConfiguration
}

function Clear-ExactRustEvidenceConfiguration {
  $script:ExactRustEvidenceConfiguration = $null
  $script:ExactRustSelectorConclusions.Clear()
}

function Get-ExactRustConfigurationSha256 {
  param([string] $Text)

  $Bytes = (New-Object System.Text.UTF8Encoding($false)).GetBytes($Text)
  $Hasher = [System.Security.Cryptography.SHA256]::Create()
  try {
    return ([System.BitConverter]::ToString($Hasher.ComputeHash($Bytes))).Replace('-', '').ToLowerInvariant()
  } finally {
    $Hasher.Dispose()
  }
}

function Set-ExactRustEvidenceConfiguration {
  param([System.Collections.IDictionary] $Fields)

  if ($null -ne $script:ExactRustEvidenceConfiguration) {
    throw 'exact Rust evidence configuration is already bound'
  }
  $Required = @(
    'Executable',
    'Toolchain',
    'RepositoryCommit',
    'DirtyManifestSha256',
    'WorkingDirectory',
    'Package',
    'Manifest',
    'Target',
    'TargetDirectory',
    'Features',
    'DefaultFeatures',
    'Profile',
    'Environment',
    'EvidenceTier',
    'TestFilter',
    'IgnoredState',
    'Harness',
    'SourcesAndOrder',
    'Platform',
    'AdaptersAndAuthority'
  )
  if ($Fields.Count -ne $Required.Count) {
    throw "exact Rust equivalence tuple must contain exactly $($Required.Count) fields; found $($Fields.Count)"
  }
  $Canonical = New-Object 'System.Collections.Generic.List[string]'
  foreach ($Name in $Required) {
    if (-not $Fields.Contains($Name)) {
      throw "exact Rust equivalence tuple lost $Name"
    }
    $Value = [string] $Fields[$Name]
    if ([string]::IsNullOrWhiteSpace($Value) -or $Value -match "[`t`r`n]") {
      throw "exact Rust equivalence tuple has an invalid $Name"
    }
    $Canonical.Add("$Name`t$Value")
  }
  $Unexpected = @($Fields.Keys | Where-Object { $Required -cnotcontains [string] $_ })
  if ($Unexpected.Count -ne 0) {
    throw "exact Rust equivalence tuple has unexpected fields: $($Unexpected -join ',')"
  }
  $CanonicalText = ($Canonical -join "`n") + "`n"
  $script:ExactRustEvidenceConfiguration = [pscustomobject] @{
    Identity = Get-ExactRustConfigurationSha256 $CanonicalText
    Fields = [ordered] @{}
    CanonicalText = $CanonicalText
  }
  foreach ($Name in $Required) {
    $script:ExactRustEvidenceConfiguration.Fields[$Name] = [string] $Fields[$Name]
  }
}

function Assert-ExactRustEvidenceRuntimeBinding {
  param(
    [string] $Cargo,
    [string] $EvidenceTier
  )

  $Configuration = $script:ExactRustEvidenceConfiguration
  if ($null -eq $Configuration) {
    throw 'exact Rust selector evidence requires a complete equivalence tuple'
  }
  $ResolvedCargo = (Resolve-Path -LiteralPath $Cargo).Path
  $ResolvedWorkingDirectory = (Resolve-Path -LiteralPath (Get-Location).Path).Path
  if ($Configuration.Fields.Executable -cne $ResolvedCargo -or
      $Configuration.Fields.WorkingDirectory -cne $ResolvedWorkingDirectory -or
      $Configuration.Fields.EvidenceTier -cne $EvidenceTier -or
      $env:HUM_CANONICAL_SEAL_EVIDENCE_TIER -cne $EvidenceTier) {
    throw 'exact Rust selector runtime differs from its recorded executable, cwd, or evidence tier'
  }
}

function Add-ExactRustSelectorConclusion {
  param(
    [string] $Selector,
    [string] $ExecutionMode,
    [string] $RetainedProducer,
    [string] $RetainedTranscript
  )

  if ($null -eq $script:ExactRustEvidenceConfiguration) {
    throw "selector '$Selector' cannot record a conclusion without an equivalence tuple"
  }
  if (@($script:ExactRustSelectorConclusions | Where-Object { $_.Selector -ceq $Selector }).Count -ne 0) {
    throw "selector '$Selector' already has a conclusion record"
  }
  $script:ExactRustSelectorConclusions.Add([pscustomobject] @{
    ConclusionId = "rust-selector::$Selector"
    Selector = $Selector
    ConfigurationId = $script:ExactRustEvidenceConfiguration.Identity
    Assertions = 'listed exactly once; executed successfully exactly once; aggregate passed=1 for exact execution or matching root result=ok'
    RequiredAbsences = 'no missing, renamed, duplicate, ignored, failed, measured, zero-test, conflicting, or configuration-drifted result'
    OutputChannelExitRelationship = 'captured combined native stdout/stderr contains the harness evidence and cargo exits 0'
    RetainedProducer = $RetainedProducer
    RetainedTranscript = $RetainedTranscript
    ExecutionMode = $ExecutionMode
  })
}

function Get-GuardedFastSelectorInventory {
  param(
    [string] $ScriptPath,
    [string] $ExcludedSelector
  )

  if (-not (Test-Path -LiteralPath $ScriptPath -PathType Leaf)) {
    throw "guarded Fast selector inventory source is unavailable: $ScriptPath"
  }

  $Tokens = $null
  $ParseErrors = $null
  $Ast = [System.Management.Automation.Language.Parser]::ParseFile(
    (Resolve-Path -LiteralPath $ScriptPath).Path,
    [ref] $Tokens,
    [ref] $ParseErrors
  )
  if (@($ParseErrors).Count -ne 0) {
    throw "guarded Fast selector inventory source has $(@($ParseErrors).Count) parse error(s)"
  }

  $Selectors = New-Object 'System.Collections.Generic.List[string]'
  $Commands = @($Ast.FindAll({
    param($Node)
    $Node -is [System.Management.Automation.Language.CommandAst] -and
      $Node.GetCommandName() -ceq 'Invoke-ExactRustTest'
  }, $true))
  foreach ($Command in $Commands) {
    $SelectorExpression = $Command.CommandElements[-1]
    if ($SelectorExpression -is [System.Management.Automation.Language.StringConstantExpressionAst]) {
      $Selectors.Add($SelectorExpression.Value)
      continue
    }

    if ($SelectorExpression -isnot [System.Management.Automation.Language.VariableExpressionAst]) {
      throw "guarded Fast selector inventory found a non-literal selector expression: $($SelectorExpression.Extent.Text)"
    }

    $ForEach = $Command.Parent
    while ($null -ne $ForEach -and
           $ForEach -isnot [System.Management.Automation.Language.ForEachStatementAst]) {
      $ForEach = $ForEach.Parent
    }
    if ($null -eq $ForEach -or
        $ForEach.Variable.VariablePath.UserPath -cne $SelectorExpression.VariablePath.UserPath) {
      throw "guarded Fast selector variable '$($SelectorExpression.Extent.Text)' is not owned by its nearest foreach"
    }

    $LoopSelectors = @($ForEach.Condition.FindAll({
      param($Node)
      $Node -is [System.Management.Automation.Language.StringConstantExpressionAst]
    }, $true))
    if ($LoopSelectors.Count -eq 0) {
      throw "guarded Fast selector foreach '$($SelectorExpression.Extent.Text)' has no literal inventory"
    }
    foreach ($LoopSelector in $LoopSelectors) {
      $Selectors.Add($LoopSelector.Value)
    }
  }

  $Inventory = @($Selectors | Where-Object { $_ -cne $ExcludedSelector })
  foreach ($Selector in $Inventory) {
    Assert-ExactRustSelectorSyntax $Selector
  }
  $Unique = @($Inventory | Sort-Object -Unique)
  if ($Inventory.Count -eq 0 -or $Inventory.Count -ne $Unique.Count) {
    throw "guarded Fast selector inventory must be nonempty and unique; declared $($Inventory.Count), unique $($Unique.Count)"
  }
  return $Inventory
}

function Enable-RootRustTestCredit {
  param([string[]] $Selectors)

  if ($script:RootRustTestCreditEnabled) {
    throw 'root Rust test credit is already enabled'
  }
  foreach ($Selector in $Selectors) {
    Assert-ExactRustSelectorSyntax $Selector
  }
  $Unique = @($Selectors | Sort-Object -Unique)
  if ($Selectors.Count -eq 0 -or $Selectors.Count -ne $Unique.Count) {
    throw "root Rust test credit requires a nonempty unique selector inventory; declared $($Selectors.Count), unique $($Unique.Count)"
  }

  $script:RootRustTestCreditEnabled = $true
  $script:RootRustDeclaredSelectors = @($Selectors)
  $script:RootRustListOutput = @()
  $script:RootRustRunOutput = @()
  $script:RootRustEvidenceReady = $false
}

function Disable-RootRustTestCredit {
  $script:RootRustTestCreditEnabled = $false
  $script:RootRustDeclaredSelectors = @()
  $script:RootRustListOutput = @()
  $script:RootRustRunOutput = @()
  $script:RootRustEvidenceReady = $false
}

function Assert-RootRustSelectorEvidence {
  param(
    [string] $Selector,
    [string[]] $ListOutput,
    [string[]] $RunOutput
  )

  Assert-ExactRustSelectorSyntax $Selector
  $EscapedSelector = [regex]::Escape($Selector)
  $ExactListings = @($ListOutput | Where-Object { $_ -match "^${EscapedSelector}: test$" })
  if ($ExactListings.Count -ne 1) {
    throw "root Rust listing must contain '$Selector' exactly once; found $($ExactListings.Count)"
  }

  $ExactSuccesses = @($RunOutput | Where-Object { $_ -match "^test ${EscapedSelector} \.\.\. ok$" })
  $ExactTerminalResults = @($RunOutput | Where-Object {
    $_ -match "^test ${EscapedSelector} \.\.\. (?:ok|FAILED|ignored)$"
  })
  if ($ExactSuccesses.Count -ne 1 -or $ExactTerminalResults.Count -ne 1) {
    throw "root Rust transcript must execute '$Selector' successfully exactly once; successful $($ExactSuccesses.Count), terminal $($ExactTerminalResults.Count)"
  }
}

function Invoke-RootRustTestProducer {
  param(
    [string] $Label,
    [string] $Cargo
  )

  if (-not $script:RootRustTestCreditEnabled) {
    throw "$Label requires root Rust test credit to be enabled first"
  }
  if ($script:RootRustEvidenceReady) {
    throw "$Label cannot run more than once"
  }
  Assert-ExactRustEvidenceRuntimeBinding $Cargo 'fast'
  if ($script:ExactRustEvidenceConfiguration.Fields.TestFilter -cne 'none (full default root suite)' -or
      $script:ExactRustEvidenceConfiguration.Fields.IgnoredState -cne 'default: ignored tests do not execute' -or
      $script:ExactRustEvidenceConfiguration.Fields.Harness -cne 'cargo test -- --list followed by cargo test') {
    throw "$Label equivalence tuple does not describe the root listing/execution producer"
  }

  Write-Host "==> $Label listing"
  $ListStartTimestamp = [System.Diagnostics.Stopwatch]::GetTimestamp()
  $ListStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
  $ListResult = Invoke-ExactRustNativeCapture $Cargo @('test', '--', '--list')
  $ListStopwatch.Stop()
  $ListEndTimestamp = [System.Diagnostics.Stopwatch]::GetTimestamp()
  $ListResult.Output | ForEach-Object { Write-Host $_ }
  if ($ListResult.ExitCode -ne 0) {
    throw "$Label listing failed with exit code $($ListResult.ExitCode)"
  }
  foreach ($Selector in $script:RootRustDeclaredSelectors) {
    $EscapedSelector = [regex]::Escape($Selector)
    $Matches = @($ListResult.Output | Where-Object { $_ -match "^${EscapedSelector}: test$" })
    if ($Matches.Count -ne 1) {
      throw "$Label listing must contain '$Selector' exactly once; found $($Matches.Count)"
    }
  }

  Write-Host "==> $Label execution"
  $RunStartTimestamp = [System.Diagnostics.Stopwatch]::GetTimestamp()
  $RunStartedUtc = [DateTimeOffset]::UtcNow
  $RunStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
  $RunResult = Invoke-ExactRustNativeCapture $Cargo @('test')
  $RunStopwatch.Stop()
  $RunCompletedUtc = [DateTimeOffset]::UtcNow
  $RunEndTimestamp = [System.Diagnostics.Stopwatch]::GetTimestamp()
  $RunResult.Output | ForEach-Object { Write-Host $_ }
  if ($RunResult.ExitCode -ne 0) {
    throw "$Label execution failed with exit code $($RunResult.ExitCode)"
  }
  $VerificationStartTimestamp = [System.Diagnostics.Stopwatch]::GetTimestamp()
  $VerificationStopwatch = [System.Diagnostics.Stopwatch]::StartNew()
  foreach ($Selector in $script:RootRustDeclaredSelectors) {
    Assert-RootRustSelectorEvidence $Selector $ListResult.Output $RunResult.Output
    Add-ExactRustSelectorConclusion `
      $Selector `
      'credited-from-root' `
      'one default root cargo test listing and execution' `
      'producer.stdout.raw'
    $script:ExactRustSelectorCredits.Add($Selector)
  }
  $VerificationStopwatch.Stop()
  $VerificationEndTimestamp = [System.Diagnostics.Stopwatch]::GetTimestamp()

  $script:RootRustListOutput = @($ListResult.Output)
  $script:RootRustRunOutput = @($RunResult.Output)
  $script:RootRustEvidenceReady = $true
  return [pscustomobject] @{
    DeclaredSelectors = $script:RootRustDeclaredSelectors.Count
    ListedSelectors = $script:RootRustDeclaredSelectors.Count
    ExecutedSelectors = $script:RootRustDeclaredSelectors.Count
    ListingMicroseconds = [int64] [math]::Round(
      $ListStopwatch.ElapsedTicks * 1000000.0 / [System.Diagnostics.Stopwatch]::Frequency
    )
    ExecutionMicroseconds = [int64] [math]::Round(
      $RunStopwatch.ElapsedTicks * 1000000.0 / [System.Diagnostics.Stopwatch]::Frequency
    )
    ListingStartTimestamp = [int64] $ListStartTimestamp
    ListingEndTimestamp = [int64] $ListEndTimestamp
    ExecutionStartTimestamp = [int64] $RunStartTimestamp
    ExecutionEndTimestamp = [int64] $RunEndTimestamp
    ExecutionStartedUtc = $RunStartedUtc
    ExecutionCompletedUtc = $RunCompletedUtc
    VerificationMicroseconds = [int64] [math]::Round(
      $VerificationStopwatch.ElapsedTicks * 1000000.0 / [System.Diagnostics.Stopwatch]::Frequency
    )
    VerificationStartTimestamp = [int64] $VerificationStartTimestamp
    VerificationEndTimestamp = [int64] $VerificationEndTimestamp
  }
}

function Assert-RootRustTestCreditsComplete {
  if (-not $script:RootRustTestCreditEnabled -or -not $script:RootRustEvidenceReady) {
    throw 'root Rust test credit cannot reconcile before its listing and execution producer'
  }
  $Credits = @(Get-ExactRustSelectorCredits)
  if ($Credits.Count -ne $script:RootRustDeclaredSelectors.Count) {
    throw "root Rust credit count drifted: declared $($script:RootRustDeclaredSelectors.Count), credited $($Credits.Count)"
  }
  for ($Index = 0; $Index -lt $Credits.Count; $Index += 1) {
    if ($Credits[$Index] -cne $script:RootRustDeclaredSelectors[$Index]) {
      throw "root Rust credit order drifted at $Index`: declared '$($script:RootRustDeclaredSelectors[$Index])', credited '$($Credits[$Index])'"
    }
  }
  Assert-ExactRustSelectorConclusionCoverage `
    $script:RootRustDeclaredSelectors `
    (Get-ExactRustSelectorConclusions) `
    $Credits `
    $script:ExactRustEvidenceConfiguration.Identity
}

function Assert-ExactRustSelectorConclusionCoverage {
  param(
    [string[]] $DeclaredSelectors,
    [object[]] $Records,
    [string[]] $Credits,
    [string] $ConfigurationId
  )

  if ([string]::IsNullOrWhiteSpace($ConfigurationId) -or $ConfigurationId -notmatch '^[0-9a-f]{64}$') {
    throw 'selector conclusion coverage requires one canonical configuration identity'
  }
  $UniqueDeclared = @($DeclaredSelectors | Sort-Object -Unique)
  $UniqueCredits = @($Credits | Sort-Object -Unique)
  if ($DeclaredSelectors.Count -eq 0 -or
      $UniqueDeclared.Count -ne $DeclaredSelectors.Count -or
      $Credits.Count -ne $DeclaredSelectors.Count -or
      $UniqueCredits.Count -ne $Credits.Count) {
    throw 'selector conclusion coverage requires equal nonempty unique declaration and credit inventories'
  }
  if ($Records.Count -ne $DeclaredSelectors.Count) {
    throw "selector conclusion coverage expected $($DeclaredSelectors.Count) records, found $($Records.Count)"
  }
  $SeenConclusions = @{}
  $SeenSelectors = @{}
  foreach ($Record in $Records) {
    foreach ($Property in @(
      'ConclusionId',
      'Selector',
      'ConfigurationId',
      'Assertions',
      'RequiredAbsences',
      'OutputChannelExitRelationship',
      'RetainedProducer',
      'RetainedTranscript',
      'ExecutionMode'
    )) {
      if ($null -eq $Record.PSObject.Properties[$Property] -or
          [string]::IsNullOrWhiteSpace([string] $Record.$Property)) {
        throw "selector conclusion record lost $Property"
      }
    }
    if ($Record.ConclusionId -cne "rust-selector::$($Record.Selector)" -or
        $Record.ConfigurationId -cne $ConfigurationId -or
        $DeclaredSelectors -cnotcontains $Record.Selector -or
        $Credits -cnotcontains $Record.Selector -or
        $Record.ExecutionMode -cnotin @('credited-from-root', 'executed-exactly-once') -or
        $Record.OutputChannelExitRelationship -notmatch 'exit.*0' -or
        $SeenConclusions.ContainsKey([string] $Record.ConclusionId) -or
        $SeenSelectors.ContainsKey([string] $Record.Selector)) {
      throw "selector conclusion record is conflicting, duplicated, or unmapped: $($Record.Selector)"
    }
    $SeenConclusions[[string] $Record.ConclusionId] = $true
    $SeenSelectors[[string] $Record.Selector] = $true
  }
  for ($Index = 0; $Index -lt $DeclaredSelectors.Count; $Index += 1) {
    if ($Credits[$Index] -cne $DeclaredSelectors[$Index] -or
        -not $SeenSelectors.ContainsKey($DeclaredSelectors[$Index])) {
      throw "selector conclusion/credit order or mapping drifted at index $Index"
    }
  }
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

function Invoke-ExactRustTest {
  param(
    [string] $Label,
    [string] $Cargo,
    [string] $Selector
  )

  Write-Host "==> $Label"
  Assert-ExactRustSelectorSyntax $Selector

  if ($script:RootRustTestCreditEnabled) {
    if (-not $script:RootRustEvidenceReady) {
      throw "$Label cannot credit '$Selector' before the root Rust producer"
    }
    $DeclaredMatches = @($script:RootRustDeclaredSelectors | Where-Object { $_ -ceq $Selector })
    if ($DeclaredMatches.Count -ne 1) {
      throw "$Label selector '$Selector' is not declared exactly once in the guarded Fast inventory"
    }
    if (@($script:ExactRustSelectorCredits | Where-Object { $_ -ceq $Selector }).Count -ne 1) {
      throw "$Label selector '$Selector' was not credited exactly once by the root producer"
    }
    Assert-RootRustSelectorEvidence $Selector $script:RootRustListOutput $script:RootRustRunOutput
    Write-Host "verified one retained root-suite credit: $Selector"
    return
  }

  $StandaloneEvidenceTier = [string] $script:ExactRustEvidenceConfiguration.Fields.EvidenceTier
  Assert-ExactRustEvidenceRuntimeBinding $Cargo $StandaloneEvidenceTier
  if ($script:ExactRustEvidenceConfiguration.Fields.TestFilter -cne "exact selector: $Selector" -or
      $script:ExactRustEvidenceConfiguration.Fields.IgnoredState -cne 'not ignored; --exact required' -or
      $script:ExactRustEvidenceConfiguration.Fields.Harness -cne 'cargo test <selector> -- --exact --list followed by --exact') {
    throw "$Label equivalence tuple does not describe the standalone exact producer"
  }
  $ListResult = Invoke-ExactRustNativeCapture $Cargo @('test', $Selector, '--', '--exact', '--list')
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

  $RunResult = Invoke-ExactRustNativeCapture $Cargo @('test', $Selector, '--', '--exact')
  $RunResult.Output | ForEach-Object { Write-Host $_ }
  if ($RunResult.ExitCode -ne 0) {
    throw "$Label failed with exit code $($RunResult.ExitCode)"
  }

  Assert-ExactRustSelectorEvidence $Selector $ListResult.Output $RunResult.Output
  $script:ExactRustSelectorCredits.Add($Selector)
  Add-ExactRustSelectorConclusion `
    $Selector `
    'executed-exactly-once' `
    'standalone exact cargo selector listing and execution' `
    'producer.stdout.raw'
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
  Clear-ExactRustEvidenceConfiguration
  Reset-ExactRustSelectorCredits
  $Selector = 'tests::selected'
  $ValidList = @('tests::selected: test', '1 test, 0 benchmarks')
  $ValidRun = @(
    'running 1 test',
    'test tests::selected ... ok',
    '',
    'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
  )
  Assert-ExactRustSelectorEvidence $Selector $ValidList $ValidRun
  $RootValidList = @('tests::other: test', 'tests::selected: test', '2 tests, 0 benchmarks')
  $RootValidRun = @(
    'running 2 tests',
    'test tests::other ... ok',
    'test tests::selected ... ok',
    '',
    'test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
  )
  Assert-RootRustSelectorEvidence $Selector $RootValidList $RootValidRun
  $SyntheticConfigurationId = 'a' * 64
  $SyntheticRecord = [pscustomobject] @{
    ConclusionId = "rust-selector::$Selector"
    Selector = $Selector
    ConfigurationId = $SyntheticConfigurationId
    Assertions = 'one exact success'
    RequiredAbsences = 'no ignored, failed, duplicate, or missing result'
    OutputChannelExitRelationship = 'combined harness output and exit=0'
    RetainedProducer = 'synthetic producer'
    RetainedTranscript = 'synthetic transcript'
    ExecutionMode = 'executed-exactly-once'
  }
  Assert-ExactRustSelectorConclusionCoverage `
    @($Selector) `
    @($SyntheticRecord) `
    @($Selector) `
    $SyntheticConfigurationId
  Assert-ExactRustSelectorThrows 'missing selector conclusion record' {
    Assert-ExactRustSelectorConclusionCoverage @($Selector) @() @($Selector) $SyntheticConfigurationId
  }
  Assert-ExactRustSelectorThrows 'duplicated selector conclusion record' {
    Assert-ExactRustSelectorConclusionCoverage @($Selector) @($SyntheticRecord, $SyntheticRecord) @($Selector) $SyntheticConfigurationId
  }
  $ConflictingRecord = $SyntheticRecord.PSObject.Copy()
  $ConflictingRecord.ConfigurationId = 'b' * 64
  Assert-ExactRustSelectorThrows 'conflicting selector equivalence tuple' {
    Assert-ExactRustSelectorConclusionCoverage @($Selector) @($ConflictingRecord) @($Selector) $SyntheticConfigurationId
  }
  $UnmappedRecord = $SyntheticRecord.PSObject.Copy()
  $UnmappedRecord.Selector = 'tests::unmapped'
  $UnmappedRecord.ConclusionId = 'rust-selector::tests::unmapped'
  Assert-ExactRustSelectorThrows 'unmapped selector conclusion record' {
    Assert-ExactRustSelectorConclusionCoverage @($Selector) @($UnmappedRecord) @($Selector) $SyntheticConfigurationId
  }

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
  Assert-ExactRustSelectorThrows 'root listing lost declared selector' {
    Assert-RootRustSelectorEvidence $Selector @('tests::other: test') $RootValidRun
  }
  Assert-ExactRustSelectorThrows 'root listing duplicated declared selector' {
    Assert-RootRustSelectorEvidence $Selector @(
      'tests::selected: test',
      'tests::selected: test'
    ) $RootValidRun
  }
  Assert-ExactRustSelectorThrows 'root execution ignored declared selector' {
    Assert-RootRustSelectorEvidence $Selector $RootValidList @(
      'running 2 tests',
      'test tests::other ... ok',
      'test tests::selected ... ignored',
      'test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s'
    )
  }
  Assert-ExactRustSelectorThrows 'unavailable cargo' {
    Invoke-ExactRustTest 'unavailable cargo sabotage' (Join-Path ([System.IO.Path]::GetTempPath()) "missing-cargo-$([guid]::NewGuid().ToString('N'))") $Selector
  }

  $TempRoot = Join-Path ([System.IO.Path]::GetTempPath()) "hum-exact-selector-$([guid]::NewGuid().ToString('N'))"
  $OriginalLocation = Get-Location
  $SelectorSelfTestPreviousTier = $env:HUM_CANONICAL_SEAL_EVIDENCE_TIER
  try {
    [System.IO.Directory]::CreateDirectory((Join-Path $TempRoot 'src/bin')) | Out-Null
    $InventoryProbe = Join-Path $TempRoot 'inventory.ps1'
    [System.IO.File]::WriteAllText(
      $InventoryProbe,
      (
        "Invoke-ExactRustTest 'first' `$Cargo 'tests::first'`n" +
        "foreach (`$EvidenceTest in @('tests::second', 'tests::third')) {`n" +
        "  Invoke-ExactRustTest `"loop `$EvidenceTest`" `$Cargo `$EvidenceTest`n" +
        "}`n" +
        "Invoke-ExactRustTest 'excluded' `$Cargo 'tests::excluded'`n"
      ),
      (New-Object System.Text.UTF8Encoding($false))
    )
    $Inventory = @(Get-GuardedFastSelectorInventory $InventoryProbe 'tests::excluded')
    if ($Inventory.Count -ne 3 -or
        $Inventory[0] -cne 'tests::first' -or
        $Inventory[1] -cne 'tests::second' -or
        $Inventory[2] -cne 'tests::third') {
      throw 'guarded Fast selector AST inventory lost literal or foreach order'
    }
    [System.IO.File]::WriteAllText(
      $InventoryProbe,
      "Invoke-ExactRustTest 'first' `$Cargo 'tests::first'`nInvoke-ExactRustTest 'duplicate' `$Cargo 'tests::first'`n",
      (New-Object System.Text.UTF8Encoding($false))
    )
    Assert-ExactRustSelectorThrows 'guarded Fast selector duplicate inventory' {
      Get-GuardedFastSelectorInventory $InventoryProbe '' | Out-Null
    }
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
    if ([string]::IsNullOrWhiteSpace($env:HUM_CANONICAL_SEAL_EVIDENCE_TIER)) {
      $env:HUM_CANONICAL_SEAL_EVIDENCE_TIER = 'selector-self-test'
    }
    Set-ExactRustEvidenceConfiguration ([ordered] @{
      Executable = (Resolve-Path -LiteralPath $Cargo).Path
      Toolchain = 'selector-self-test-toolchain'
      RepositoryCommit = '0000000000000000000000000000000000000000'
      DirtyManifestSha256 = ('0' * 64)
      WorkingDirectory = (Resolve-Path -LiteralPath $TempRoot).Path
      Package = 'exact-selector-probe'
      Manifest = (Join-Path $TempRoot 'Cargo.toml')
      Target = 'bin:first'
      TargetDirectory = (Join-Path $TempRoot 'target')
      Features = 'default'
      DefaultFeatures = 'enabled'
      Profile = 'test'
      Environment = 'selector-self-test'
      EvidenceTier = [string] $env:HUM_CANONICAL_SEAL_EVIDENCE_TIER
      TestFilter = "exact selector: $Selector"
      IgnoredState = 'not ignored; --exact required'
      Harness = 'cargo test <selector> -- --exact --list followed by --exact'
      SourcesAndOrder = 'src/bin/first.rs'
      Platform = "os=$([System.Environment]::OSVersion.Platform);process=$([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture)"
      AdaptersAndAuthority = 'none'
    })

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
    Clear-ExactRustEvidenceConfiguration
    Reset-ExactRustSelectorCredits
    if ($null -eq $SelectorSelfTestPreviousTier) {
      Remove-Item Env:HUM_CANONICAL_SEAL_EVIDENCE_TIER -ErrorAction SilentlyContinue
    } else {
      $env:HUM_CANONICAL_SEAL_EVIDENCE_TIER = $SelectorSelfTestPreviousTier
    }
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
    $CargoHome = if ([string]::IsNullOrWhiteSpace($env:USERPROFILE)) {
      [Environment]::GetFolderPath('UserProfile')
    } else {
      $env:USERPROFILE
    }
    $CargoPath = Join-Path $CargoHome '.cargo/bin/cargo.exe'
  }
  Invoke-ExactRustSelectorSelfTests $CargoPath
}
