$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path

function Resolve-Tool {
  param(
    [string] $Name,
    [string] $FallbackRelativeToHome,
    [string] $MissingMessage
  )

  $Command = Get-Command $Name -ErrorAction SilentlyContinue
  if ($null -ne $Command) {
    return $Command.Source
  }

  if ($FallbackRelativeToHome -ne '') {
    $ProfileRoot = [Environment]::GetFolderPath('UserProfile')
    $Candidate = Join-Path $ProfileRoot $FallbackRelativeToHome
    if (Test-Path -LiteralPath $Candidate) {
      return $Candidate
    }
  }

  throw $MissingMessage
}

$Cargo = Resolve-Tool 'cargo' '.cargo\bin\cargo.exe' 'cargo was not found on PATH or in the standard user Cargo install directory'
$Git = Resolve-Tool 'git' '' 'git was not found on PATH'

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

  return ($Output -join "`n")
}
function Read-NativeOutputWithExit {
  param(
    [string] $Label,
    [string] $FilePath,
    [string[]] $Arguments
  )

  Write-Host "==> $Label"
  $PreviousErrorActionPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $Output = & $FilePath @Arguments 2>&1
    $ExitCode = $LASTEXITCODE
  } finally {
    $ErrorActionPreference = $PreviousErrorActionPreference
  }
  return [pscustomobject] @{
    Output = ($Output -join "`n")
    ExitCode = $ExitCode
  }
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

function Assert-Json {
  param(
    [string] $Label,
    [string] $Text
  )

  Write-Host "==> parse $Label"
  $Text | ConvertFrom-Json | Out-Null
}

function Get-GraphItems {
  param([object[]] $Items)

  foreach ($Item in @($Items)) {
    if ($null -eq $Item) {
      continue
    }

    $Item
    if ($null -ne $Item.items) {
      Get-GraphItems @($Item.items)
    }
  }
}

function Assert-ReferenceEvidenceCoverage {
  param([object] $Graph)

  Write-Host '==> reference fixture evidence coverage smoke'
  $EvidenceObligations = @()
  foreach ($File in @($Graph.files)) {
    foreach ($Item in Get-GraphItems @($File.items)) {
      if ($Item.kind -eq 'task' -and $null -ne $Item.evidence_obligations) {
        $EvidenceObligations += @($Item.evidence_obligations)
      }
    }
  }

  if ($EvidenceObligations.Count -eq 0) {
    throw 'reference fixture graph JSON has no evidence obligations'
  }

  $Unlinked = @($EvidenceObligations | Where-Object {
    $_.verification_status -ne 'linked' -or $null -eq $_.linked_evidence -or @($_.linked_evidence).Count -eq 0
  })
  if ($Unlinked.Count -gt 0) {
    $Details = ($Unlinked | ForEach-Object {
      "  $($_.id) covers '$($_.covers)' with status '$($_.verification_status)'"
    }) -join "`n"
    throw "reference fixture has unlinked evidence obligations:`n$Details"
  }
}

function Assert-TextMateSnapshot {
  param([string] $Generated)

  Write-Host '==> generated TextMate grammar matches snapshot'
  if (-not $Generated.EndsWith("`n")) {
    $Generated += "`n"
  }

  $SnapshotPath = Join-Path (Join-Path (Join-Path $RepoRoot 'editors') 'textmate') 'hum.tmLanguage.json'
  $Snapshot = [System.IO.File]::ReadAllText($SnapshotPath)
  if ($Snapshot -ne $Generated) {
    throw 'editors/textmate/hum.tmLanguage.json is stale; run tools/update_textmate_grammar.ps1'
  }
}

function Assert-ReadmeHumExamplesMatch {
  Write-Host '==> README Hum examples match fixtures'
  $ReadmePath = Join-Path $RepoRoot 'README.md'
  $Readme = [System.IO.File]::ReadAllText($ReadmePath)
  $Pattern = '<!-- hum-example:start (?<path>[^\r\n]+) -->\s*```hum\r?\n(?<code>.*?)\r?\n```\s*<!-- hum-example:end -->'
  $Matches = [System.Text.RegularExpressions.Regex]::Matches($Readme, $Pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)
  if ($Matches.Count -lt 2) {
    throw 'README.md must contain at least two checked hum-example blocks'
  }

  foreach ($Match in $Matches) {
    $Relative = $Match.Groups['path'].Value.Trim()
    $FixturePath = Join-Path $RepoRoot ($Relative -replace '/', [System.IO.Path]::DirectorySeparatorChar)
    if (-not (Test-Path -LiteralPath $FixturePath)) {
      throw "README hum-example fixture is missing: $Relative"
    }
    $Fixture = [System.IO.File]::ReadAllText($FixturePath).Replace(([string][char]13 + [string][char]10), [string][char]10)
    $Code = $Match.Groups['code'].Value.Replace(([string][char]13 + [string][char]10), [string][char]10)
    if (-not $Fixture.Contains($Code)) {
      throw "README hum-example block does not match fixture text: $Relative"
    }
  }
}

function Assert-SessionASurfaceRules {
  Write-Host '==> Session A source-surface rules'
  $HumRoots = @((Join-Path $RepoRoot 'examples'), (Join-Path $RepoRoot 'fixtures'))
  foreach ($Root in $HumRoots) {
    foreach ($File in Get-ChildItem -LiteralPath $Root -Recurse -Filter '*.hum') {
      $Text = [System.IO.File]::ReadAllText($File.FullName)
      if ([regex]::IsMatch($Text, '(?m)^\s*task [a-z]+ [a-z]+')) {
        throw "spaced task name remains in $($File.FullName)"
      }
      if ([regex]::IsMatch($Text, '(?m)^\s*(store [a-z]+ [a-z]+|app [A-Z][A-Za-z]*|[^\r\n]*\((?!borrow |change |consume )[a-z]+ [a-z]+:)')) {
        throw "spaced or noncanonical declaration name remains in $($File.FullName)"
      }
      if ($Text.Contains('Number')) {
        throw "Number type usage remains in $($File.FullName)"
      }
    }
  }

  foreach ($Relative in @('README.md', 'SPEC.md')) {
    $Path = Join-Path $RepoRoot $Relative
    $Text = [System.IO.File]::ReadAllText($Path)
    if ($Text.Contains('Number')) {
      throw "Number type usage remains in $Relative"
    }
  }
}

Push-Location $RepoRoot
try {
  Invoke-Native 'cargo fmt --check' $Cargo @('fmt', '--check')
  Invoke-Native 'cargo test' $Cargo @('test')
  Invoke-Native 'cargo clippy' $Cargo @('clippy', '--all-targets', '--', '-D', 'warnings')
  Invoke-Native 'cargo build' $Cargo @('build')

  $HumName = if ($env:OS -eq 'Windows_NT') { 'hum.exe' } else { 'hum' }
  $Hum = Join-Path (Join-Path (Join-Path $RepoRoot 'target') 'debug') $HumName

  $VersionJson = Read-NativeOutput 'version JSON' $Hum @('version', '--format', 'json')
  Assert-Json 'version JSON' $VersionJson
  if (-not $VersionJson.Contains('"core_lower": "hum.core_lower.v0"')) { throw 'version JSON is missing hum.core_lower.v0 schema' }
  if (-not $VersionJson.Contains('"core_verify": "hum.core_verify.v0"')) { throw 'version JSON is missing hum.core_verify.v0 schema' }
  if (-not $VersionJson.Contains('"full_type_check": "hum.full_type_check.v0"')) { throw 'version JSON is missing hum.full_type_check.v0 schema' }
  if (-not $VersionJson.Contains('"effect_check": "hum.effect_check.v0"')) { throw 'version JSON is missing hum.effect_check.v0 schema' }
  if (-not $VersionJson.Contains('"ownership_check": "hum.ownership_check.v0"')) { throw 'version JSON is missing hum.ownership_check.v0 schema' }
  if (-not $VersionJson.Contains('"resource_check": "hum.resource_check.v0"')) { throw 'version JSON is missing hum.resource_check.v0 schema' }
  if (-not $VersionJson.Contains('"profile_check": "hum.profile_check.v0"')) { throw 'version JSON is missing hum.profile_check.v0 schema' }

  $ExplainJson = Read-NativeOutput 'diagnostic explain JSON' $Hum @('explain', 'H0201', '--format', 'json')
  Assert-Json 'diagnostic explain JSON' $ExplainJson

  $DiagnosticsJson = Read-NativeOutput 'diagnostic catalog JSON' $Hum @('diagnostics', '--format', 'json')
  Assert-Json 'diagnostic catalog JSON' $DiagnosticsJson
  if (-not $DiagnosticsJson.Contains('"code": "H0009"')) { throw 'diagnostic catalog JSON is missing H0009' }
  if (-not $DiagnosticsJson.Contains('"code": "H0601"')) { throw 'diagnostic catalog JSON is missing H0601' }
  if (-not $DiagnosticsJson.Contains('"code": "H0602"')) { throw 'diagnostic catalog JSON is missing H0602' }
  if (-not $DiagnosticsJson.Contains('"code": "H0603"')) { throw 'diagnostic catalog JSON is missing H0603' }
  if (-not $DiagnosticsJson.Contains('"code": "H0604"')) { throw 'diagnostic catalog JSON is missing H0604' }
  if (-not $DiagnosticsJson.Contains('"code": "H0605"')) { throw 'diagnostic catalog JSON is missing H0605' }
  if (-not $DiagnosticsJson.Contains('"code": "H0606"')) { throw 'diagnostic catalog JSON is missing H0606' }
  if (-not $DiagnosticsJson.Contains('"code": "H0701"')) { throw 'diagnostic catalog JSON is missing H0701' }
  if (-not $DiagnosticsJson.Contains('"code": "H0702"')) { throw 'diagnostic catalog JSON is missing H0702' }
  if (-not $DiagnosticsJson.Contains('"code": "H0703"')) { throw 'diagnostic catalog JSON is missing H0703' }
  if (-not $DiagnosticsJson.Contains('"code": "H0801"')) { throw 'diagnostic catalog JSON is missing H0801' }
  if (-not $DiagnosticsJson.Contains('"code": "H0802"')) { throw 'diagnostic catalog JSON is missing H0802' }
  if (-not $DiagnosticsJson.Contains('"code": "H0803"')) { throw 'diagnostic catalog JSON is missing H0803' }
  if (-not $DiagnosticsJson.Contains('"code": "H0804"')) { throw 'diagnostic catalog JSON is missing H0804' }
  if (-not $DiagnosticsJson.Contains('"code": "H0805"')) { throw 'diagnostic catalog JSON is missing H0805' }
  if (-not $DiagnosticsJson.Contains('"code": "H0806"')) { throw 'diagnostic catalog JSON is missing H0806' }
  if (-not $DiagnosticsJson.Contains('"code": "H0807"')) { throw 'diagnostic catalog JSON is missing H0807' }
  if (-not $DiagnosticsJson.Contains('"code": "H1201"')) { throw 'diagnostic catalog JSON is missing H1201' }
  if (-not $DiagnosticsJson.Contains('"code": "H1202"')) { throw 'diagnostic catalog JSON is missing H1202' }
  if (-not $DiagnosticsJson.Contains('"code": "H1203"')) { throw 'diagnostic catalog JSON is missing H1203' }
  if (-not $DiagnosticsJson.Contains('"code": "H1204"')) { throw 'diagnostic catalog JSON is missing H1204' }
  if (-not $DiagnosticsJson.Contains('"code": "H1205"')) { throw 'diagnostic catalog JSON is missing H1205' }

  $CapabilitiesJson = Read-NativeOutput 'capabilities JSON' $Hum @('capabilities', '--format', 'json')
  Assert-Json 'capabilities JSON' $CapabilitiesJson
  if (-not $CapabilitiesJson.Contains('"schema": "hum.capabilities.v0"')) { throw 'capabilities JSON is missing hum.capabilities.v0 schema' }
  if (-not $CapabilitiesJson.Contains('"editor_capabilities"')) { throw 'capabilities JSON is missing editor_capabilities' }
  if (-not $CapabilitiesJson.Contains('"document_symbols"')) { throw 'capabilities JSON is missing document_symbols capability' }
  if (-not $CapabilitiesJson.Contains('"semantic_token_legend"')) { throw 'capabilities JSON is missing semantic_token_legend capability' }
  if (-not $CapabilitiesJson.Contains('"lsp_capabilities"')) { throw 'capabilities JSON is missing lsp_capabilities schema' }
  if (-not $CapabilitiesJson.Contains('"evidence_report"')) { throw 'capabilities JSON is missing evidence_report schema' }
  if (-not $CapabilitiesJson.Contains('"math_obligations_report"')) { throw 'capabilities JSON is missing math_obligations_report schema' }
  if (-not $CapabilitiesJson.Contains('"math_obligation"')) { throw 'capabilities JSON is missing math_obligation schema' }
  if (-not $CapabilitiesJson.Contains('"resource_report"')) { throw 'capabilities JSON is missing resource_report schema' }
  if (-not $CapabilitiesJson.Contains('"core_preview"')) { throw 'capabilities JSON is missing core_preview schema' }
  if (-not $CapabilitiesJson.Contains('"core_lower"')) { throw 'capabilities JSON is missing core_lower schema' }
  if (-not $CapabilitiesJson.Contains('"core_lower_json"')) { throw 'capabilities JSON is missing core_lower_json command' }
  if (-not $CapabilitiesJson.Contains('"core_verify"')) { throw 'capabilities JSON is missing core_verify schema' }
  if (-not $CapabilitiesJson.Contains('"core_verify_json"')) { throw 'capabilities JSON is missing core_verify_json command' }
  if (-not $CapabilitiesJson.Contains('"resolve_report"')) { throw 'capabilities JSON is missing resolve_report schema' }
  if (-not $CapabilitiesJson.Contains('"resolve_json"')) { throw 'capabilities JSON is missing resolve_json command' }
  if (-not $CapabilitiesJson.Contains('"type_env"')) { throw 'capabilities JSON is missing type_env schema' }
  if (-not $CapabilitiesJson.Contains('"type_env_json"')) { throw 'capabilities JSON is missing type_env_json command' }
  if (-not $CapabilitiesJson.Contains('"type_check"')) { throw 'capabilities JSON is missing type_check schema' }
  if (-not $CapabilitiesJson.Contains('"type_check_json"')) { throw 'capabilities JSON is missing type_check_json command' }
  if (-not $CapabilitiesJson.Contains('"full_type_check"')) { throw 'capabilities JSON is missing full_type_check schema' }
  if (-not $CapabilitiesJson.Contains('"full_type_check_json"')) { throw 'capabilities JSON is missing full_type_check_json command' }
  if (-not $CapabilitiesJson.Contains('"effect_check"')) { throw 'capabilities JSON is missing effect_check schema' }
  if (-not $CapabilitiesJson.Contains('"effect_check_json"')) { throw 'capabilities JSON is missing effect_check_json command' }
  if (-not $CapabilitiesJson.Contains('"ownership_check"')) { throw 'capabilities JSON is missing ownership_check schema' }
  if (-not $CapabilitiesJson.Contains('"ownership_check_json"')) { throw 'capabilities JSON is missing ownership_check_json command' }
  if (-not $CapabilitiesJson.Contains('"resource_check"')) { throw 'capabilities JSON is missing resource_check schema' }
  if (-not $CapabilitiesJson.Contains('"resource_check_json"')) { throw 'capabilities JSON is missing resource_check_json command' }
  if (-not $CapabilitiesJson.Contains('"profile_check"')) { throw 'capabilities JSON is missing profile_check schema' }
  if (-not $CapabilitiesJson.Contains('"profile_check_json"')) { throw 'capabilities JSON is missing profile_check_json command' }
  if (-not $CapabilitiesJson.Contains('"ir_readiness"')) { throw 'capabilities JSON is missing ir_readiness schema' }
  if (-not $CapabilitiesJson.Contains('"core_contract"')) { throw 'capabilities JSON is missing core_contract schema' }
  if (-not $CapabilitiesJson.Contains('"ir_contract"')) { throw 'capabilities JSON is missing ir_contract schema' }
  if (-not $CapabilitiesJson.Contains('"backend_contract"')) { throw 'capabilities JSON is missing backend_contract schema' }
  if (-not $CapabilitiesJson.Contains('"runtime_profiles"')) { throw 'capabilities JSON is missing runtime_profiles schema' }
  if (-not $CapabilitiesJson.Contains('"runtime_profile"')) { throw 'capabilities JSON is missing runtime_profile schema' }
  if (-not $CapabilitiesJson.Contains('"runtime_profiles_json"')) { throw 'capabilities JSON is missing runtime_profiles_json command' }
  if (-not $CapabilitiesJson.Contains('"state_model"')) { throw 'capabilities JSON is missing state_model schema' }
  if (-not $CapabilitiesJson.Contains('"state_permission"')) { throw 'capabilities JSON is missing state_permission schema' }
  if (-not $CapabilitiesJson.Contains('"state_model_json"')) { throw 'capabilities JSON is missing state_model_json command' }
  if (-not $CapabilitiesJson.Contains('"doctor"')) { throw 'capabilities JSON is missing doctor schema' }
  if (-not $CapabilitiesJson.Contains('"target_facts"')) { throw 'capabilities JSON is missing target_facts schema' }
  if (-not $CapabilitiesJson.Contains('"target_fact_record"')) { throw 'capabilities JSON is missing target_fact_record schema' }

  $CoreContractJson = Read-NativeOutput 'Core contract JSON' $Hum @('core-contract', '--format', 'json')
  Assert-Json 'Core contract JSON' $CoreContractJson
  if (-not $CoreContractJson.Contains('"schema": "hum.core_contract.v0"')) { throw 'Core contract JSON is missing hum.core_contract.v0 schema' }
  if (-not $CoreContractJson.Contains('"lowers_from_schema": "hum.semantic_graph.v0"')) { throw 'Core contract JSON is missing semantic graph source schema' }
  if (-not $CoreContractJson.Contains('"lowers_to_schema": "hum.ir_contract.v0"')) { throw 'Core contract JSON is missing Hum IR target schema' }
  if (-not $CoreContractJson.Contains('"name": "statements"')) { throw 'Core contract JSON is missing statements catalog' }
  if (-not $CoreContractJson.Contains('"set_place"')) { throw 'Core contract JSON is missing set_place statement' }
  if (-not $CoreContractJson.Contains('"id": "body_grammar"')) { throw 'Core contract JSON is missing body_grammar gate' }
  if (-not $CoreContractJson.Contains('"status": "partial_v0"')) { throw 'Core contract JSON is missing partial_v0 body grammar status' }
  if (-not $CoreContractJson.Contains('"id": "core_preview"')) { throw 'Core contract JSON is missing core_preview gate' }
  if (-not $CoreContractJson.Contains('"status": "preview_v0"')) { throw 'Core contract JSON is missing preview_v0 status' }
  if (-not $CoreContractJson.Contains('"id": "core_lowering"')) { throw 'Core contract JSON is missing core_lowering gate' }
  if (-not $CoreContractJson.Contains('"status": "unverified_core_artifact_v0"')) { throw 'Core contract JSON is missing unverified core artifact gate status' }
  if (-not $CoreContractJson.Contains('"id": "type_check"')) { throw 'Core contract JSON is missing type_check gate' }
  if (-not $CoreContractJson.Contains('"status": "declaration_and_trivial_return_check_available"')) { throw 'Core contract JSON is missing narrow type-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "full_type_check"')) { throw 'Core contract JSON is missing full_type_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_body_type_gate_available_v0"')) { throw 'Core contract JSON is missing full type-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "effect_check"')) { throw 'Core contract JSON is missing effect_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_effect_gate_available_v0"')) { throw 'Core contract JSON is missing effect-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "ownership_check"')) { throw 'Core contract JSON is missing ownership_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_ownership_gate_available_v0"')) { throw 'Core contract JSON is missing ownership-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "allocation_resource_check"')) { throw 'Core contract JSON is missing allocation_resource_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_resource_gate_available_v0"')) { throw 'Core contract JSON is missing resource-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "profile_check"')) { throw 'Core contract JSON is missing profile_check gate' }
  if (-not $CoreContractJson.Contains('"status": "recognized_core_profile_gate_available_v0"')) { throw 'Core contract JSON is missing profile-check gate status' }
  if (-not $CoreContractJson.Contains('"id": "core_verify"')) { throw 'Core contract JSON is missing core_verify gate' }
  if (-not $CoreContractJson.Contains('"status": "verified_non_executing_core_artifact_v0"')) { throw 'Core contract JSON is missing core verify gate status' }
  if (-not $CoreContractJson.Contains('"no executable semantics"')) { throw 'Core contract JSON must keep V0 non-execution claim' }

  $IrContractJson = Read-NativeOutput 'IR contract JSON' $Hum @('ir-contract', '--format', 'json')
  Assert-Json 'IR contract JSON' $IrContractJson
  if (-not $IrContractJson.Contains('"schema": "hum.ir_contract.v0"')) { throw 'IR contract JSON is missing hum.ir_contract.v0 schema' }
  if (-not $IrContractJson.Contains('"semantic_owner": "hum_ir"')) { throw 'IR contract JSON is missing semantic owner' }
  if (-not $IrContractJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'IR contract JSON is missing Core Hum contract schema' }
  if (-not $IrContractJson.Contains('"id": "core_hum"')) { throw 'IR contract JSON is missing core_hum layer' }
  if (-not $IrContractJson.Contains('"id": "hum_ir"')) { throw 'IR contract JSON is missing hum_ir layer' }
  if (-not $IrContractJson.Contains('"typed_failure_edges"')) { throw 'IR contract JSON is missing typed failure facts' }
  if (-not $IrContractJson.Contains('"core_verify"')) { throw 'IR contract JSON is missing core_verify pass' }
  if (-not $IrContractJson.Contains('"full_type_check"')) { throw 'IR contract JSON is missing full_type_check pass' }
  if (-not $IrContractJson.Contains('"effect_check"')) { throw 'IR contract JSON is missing effect_check pass' }
  if (-not $IrContractJson.Contains('"ir_verify"')) { throw 'IR contract JSON is missing ir_verify pass' }
  if (-not $IrContractJson.Contains('"no IR emission for source files"')) { throw 'IR contract JSON must keep V0 non-emission claim' }

  $BackendContractJson = Read-NativeOutput 'backend contract JSON' $Hum @('backend-contract', '--format', 'json')
  Assert-Json 'backend contract JSON' $BackendContractJson
  if (-not $BackendContractJson.Contains('"schema": "hum.backend_contract.v0"')) { throw 'backend contract JSON is missing hum.backend_contract.v0 schema' }
  if (-not $BackendContractJson.Contains('"semantic_owner": "hum_ir"')) { throw 'backend contract JSON is missing semantic owner' }
  if (-not $BackendContractJson.Contains('"semantic_owner_schema": "hum.ir_contract.v0"')) { throw 'backend contract JSON is missing Hum IR owner schema' }
  if (-not $BackendContractJson.Contains('"id": "interpreter"')) { throw 'backend contract JSON is missing interpreter stage' }
  if (-not $BackendContractJson.Contains('"id": "cranelift"')) { throw 'backend contract JSON is missing cranelift stage' }
  if (-not $BackendContractJson.Contains('"id": "llvm"')) { throw 'backend contract JSON is missing llvm stage' }
  if (-not $BackendContractJson.Contains('"no code execution"')) { throw 'backend contract JSON must keep V0 non-execution claim' }

  $RuntimeProfilesJson = Read-NativeOutput 'runtime profiles JSON' $Hum @('profiles', '--format', 'json')
  Assert-Json 'runtime profiles JSON' $RuntimeProfilesJson
  if (-not $RuntimeProfilesJson.Contains('"schema": "hum.runtime_profiles.v0"')) { throw 'runtime profiles JSON is missing hum.runtime_profiles.v0 schema' }
  if (-not $RuntimeProfilesJson.Contains('"profile_schema": "hum.runtime_profile.v0"')) { throw 'runtime profiles JSON is missing hum.runtime_profile.v0 schema link' }
  if (-not $RuntimeProfilesJson.Contains('"mode": "contract_only_no_profile_enforcement"')) { throw 'runtime profiles JSON must stay contract-only in V0' }
  if (-not $RuntimeProfilesJson.Contains('"id": "agent_tool_sandbox"')) { throw 'runtime profiles JSON is missing agent_tool_sandbox profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "footprint_constrained"')) { throw 'runtime profiles JSON is missing footprint_constrained profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "hard_realtime"')) { throw 'runtime profiles JSON is missing hard_realtime profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "windows_service"')) { throw 'runtime profiles JSON is missing windows_service profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "driver_candidate"')) { throw 'runtime profiles JSON is missing driver_candidate profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "engine_hot_path"')) { throw 'runtime profiles JSON is missing engine_hot_path profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "medical_class_c"')) { throw 'runtime profiles JSON is missing medical_class_c profile' }
  if (-not $RuntimeProfilesJson.Contains('"id": "automotive_asil_d"')) { throw 'runtime profiles JSON is missing automotive_asil_d profile' }
  if (-not $RuntimeProfilesJson.Contains('"denied_capability_families"')) { throw 'runtime profiles JSON is missing denied capability policy' }
  if (-not $RuntimeProfilesJson.Contains('"os.network"')) { throw 'runtime profiles JSON is missing network capability policy' }
  if (-not $RuntimeProfilesJson.Contains('"no profile syntax enforcement"')) { throw 'runtime profiles JSON must keep V0 enforcement non-claim' }
  if (-not $RuntimeProfilesJson.Contains('"no certification claim"')) { throw 'runtime profiles JSON must not claim certification' }

  $StateModelJson = Read-NativeOutput 'state model JSON' $Hum @('state-model', '--format', 'json')
  Assert-Json 'state model JSON' $StateModelJson
  if (-not $StateModelJson.Contains('"schema": "hum.state_model.v0"')) { throw 'state model JSON is missing hum.state_model.v0 schema' }
  if (-not $StateModelJson.Contains('"permission_schema": "hum.state_permission.v0"')) { throw 'state model JSON is missing hum.state_permission.v0 schema link' }
  if (-not $StateModelJson.Contains('"mode": "contract_only_partial_declared_mutation_check"')) { throw 'state model JSON must keep V0 contract-only mode' }
  if (-not $StateModelJson.Contains('"id": "immutable_value"')) { throw 'state model JSON is missing immutable_value kind' }
  if (-not $StateModelJson.Contains('"id": "mutable_local"')) { throw 'state model JSON is missing mutable_local kind' }
  if (-not $StateModelJson.Contains('"id": "linear_resource"')) { throw 'state model JSON is missing linear_resource kind' }
  if (-not $StateModelJson.Contains('"id": "borrow_check"')) { throw 'state model JSON is missing borrow_check gate' }
  if (-not $StateModelJson.Contains('"no borrow checker implementation"')) { throw 'state model JSON must not claim borrow checking' }
  if (-not $StateModelJson.Contains('"no concurrency or memory-order model"')) { throw 'state model JSON must not claim concurrency checking' }

  $LspCapabilitiesJson = Read-NativeOutput 'LSP capabilities JSON' $Hum @('lsp', '--capabilities', '--format', 'json')
  Assert-Json 'LSP capabilities JSON' $LspCapabilitiesJson
  if (-not $LspCapabilitiesJson.Contains('"schema": "hum.lsp_capabilities.v0"')) { throw 'LSP capabilities JSON is missing hum.lsp_capabilities.v0 schema' }
  if (-not $LspCapabilitiesJson.Contains('"json_rpc_server": false')) { throw 'LSP capabilities JSON should say server mode is not implemented' }
  if (-not $LspCapabilitiesJson.Contains('"textDocument/publishDiagnostics"')) { throw 'LSP capabilities JSON is missing diagnostics method' }
  if (-not $LspCapabilitiesJson.Contains('"textDocument/documentSymbol"')) { throw 'LSP capabilities JSON is missing documentSymbol method' }

  $DoctorJson = Read-NativeOutput 'doctor JSON' $Hum @('doctor', '--format', 'json')
  Assert-Json 'doctor JSON' $DoctorJson
  if (-not $DoctorJson.Contains('"schema": "hum.doctor.v0"')) { throw 'doctor JSON is missing hum.doctor.v0 schema' }
  if (-not $DoctorJson.Contains('"summary"')) { throw 'doctor JSON is missing summary' }
  if (-not $DoctorJson.Contains('"status": "pass"')) { throw 'doctor JSON should pass from the repo root' }
  if (-not $DoctorJson.Contains('"text_hygiene_policy"')) { throw 'doctor JSON is missing text_hygiene_policy check' }
  if (-not $DoctorJson.Contains('"public_readiness_policy"')) { throw 'doctor JSON is missing public_readiness_policy check' }
  if (-not $DoctorJson.Contains('"clean_checkout_smoke"')) { throw 'doctor JSON is missing clean_checkout_smoke check' }
  if (-not $DoctorJson.Contains('"tag_readiness"')) { throw 'doctor JSON is missing tag_readiness check' }

  $TargetFactsJson = Read-NativeOutput 'target facts JSON' $Hum @('target-facts', '--format', 'json')
  Assert-Json 'target facts JSON' $TargetFactsJson
  if (-not $TargetFactsJson.Contains('"schema": "hum.target_facts.v0"')) { throw 'target facts JSON is missing hum.target_facts.v0 schema' }
  if (-not $TargetFactsJson.Contains('"record_schema": "hum.target_fact_record.v0"')) { throw 'target facts JSON is missing target fact record schema' }
  if (-not $TargetFactsJson.Contains('"mode": "contract_only_no_host_probe"')) { throw 'target facts JSON must stay non-probing in V0' }
  if (-not $TargetFactsJson.Contains('"default_policy": "unknown_fails_closed"')) { throw 'target facts JSON is missing fail-closed policy' }
  if (-not $TargetFactsJson.Contains('"field_catalog"')) { throw 'target facts JSON is missing field catalog' }
  if (-not $TargetFactsJson.Contains('"capability_families"')) { throw 'target facts JSON is missing capability families' }
  if (-not $TargetFactsJson.Contains('"family": "os.clock"')) { throw 'target facts JSON is missing clock capability family' }
  if (-not $TargetFactsJson.Contains('"availability": "host_import_profile_gated"')) { throw 'target facts JSON is missing WASI clock/random availability' }
  if (-not $TargetFactsJson.Contains('"id": "windows-x86_64-msvc"')) { throw 'target facts JSON is missing Windows fixture' }
  if (-not $TargetFactsJson.Contains('"id": "wasm32-wasi-preview1"')) { throw 'target facts JSON is missing WASI fixture' }
  if (-not $TargetFactsJson.Contains('"no host capability probing"')) { throw 'target facts JSON must reject host probing claims' }

  $TargetFactFixtureDir = Join-Path (Join-Path $RepoRoot 'fixtures') 'target_facts'
  $TargetFactFixtures = @(Get-ChildItem -LiteralPath $TargetFactFixtureDir -Filter '*.json' -File)
  if ($TargetFactFixtures.Count -lt 4) { throw 'target fact fixture directory must contain at least four JSON fixtures' }
  foreach ($Fixture in $TargetFactFixtures) {
    $FixtureText = [System.IO.File]::ReadAllText($Fixture.FullName)
    $FixtureJson = $FixtureText | ConvertFrom-Json
    if ($FixtureJson.schema -ne 'hum.target_fact_record.v0') { throw "target fact fixture $($Fixture.Name) has wrong schema" }
    if ($FixtureJson.status -ne 'fixture') { throw "target fact fixture $($Fixture.Name) must have fixture status" }
    if ($FixtureJson.absence_policy -ne 'unknown_or_absent_capabilities_fail_closed') { throw "target fact fixture $($Fixture.Name) must fail closed" }
    if ($null -eq $FixtureJson.facts.triple) { throw "target fact fixture $($Fixture.Name) is missing facts.triple" }
    if ($null -eq $FixtureJson.capabilities -or @($FixtureJson.capabilities).Count -eq 0) { throw "target fact fixture $($Fixture.Name) is missing capabilities" }
    if ($null -eq $FixtureJson.non_claims -or @($FixtureJson.non_claims).Count -eq 0) { throw "target fact fixture $($Fixture.Name) is missing non_claims" }
  }

  Invoke-Native 'hum check examples' $Hum @('check', 'examples')

  $RunAdd = Read-NativeOutput 'run core add' $Hum @('run', 'examples/core/add.hum', '--entry', 'add', '--args', '2', '3')
  if ($RunAdd.Trim() -ne '5') { throw "hum run add expected 5, got `$RunAdd" }

  $RunDivideZero = Read-NativeOutputWithExit 'run core divide needs violation' $Hum @('run', 'examples/core/divide.hum', '--entry', 'divide', '--args', '10', '0')
  if ($RunDivideZero.ExitCode -ne 1) { throw "hum run divide zero expected exit 1, got $($RunDivideZero.ExitCode)" }
  if (-not $RunDivideZero.Output.Contains('caller did not satisfy needs: b != 0')) { throw "hum run divide zero expected caller needs blame, got $($RunDivideZero.Output)" }
  if (-not $RunDivideZero.Output.Contains('examples/core/divide.hum:12:')) { throw "hum run divide zero expected source span for needs predicate, got $($RunDivideZero.Output)" }

  $RunCountCompleted = Read-NativeOutput 'run core count_completed' $Hum @('run', 'examples/core/count_completed.hum', '--entry', 'count_completed', '--args', '[{done:true},{done:false},{done:true}]')
  if ($RunCountCompleted.Trim() -ne '2') { throw "hum run count_completed expected 2, got `$RunCountCompleted" }

  $RunWrongAdd = Read-NativeOutputWithExit 'run wrong add contract violation' $Hum @('run', 'fixtures/run/wrong_add_contract.hum', '--entry', 'add', '--args', '2', '3')
  if ($RunWrongAdd.ExitCode -ne 1) { throw "hum run wrong add expected exit 1, got $($RunWrongAdd.ExitCode)" }
  if (-not $RunWrongAdd.Output.Contains('task `add` did not satisfy ensures: result == a + b')) { throw "hum run wrong add expected task ensures blame, got $($RunWrongAdd.Output)" }

  $RunWordCount = Read-NativeOutput 'run probe word_count' $Hum @('run', 'examples/probes/word_count.hum', '--entry', 'count_hum_literal')
  if ($RunWordCount.Trim() -ne '2') { throw "hum run word_count expected 2, got `$RunWordCount" }

  $RunTaskListFlow = Read-NativeOutput 'run probe task_list_flow' $Hum @('run', 'examples/probes/task_list_flow.hum', '--entry', 'task_list_demo')
  if ($RunTaskListFlow.Trim() -ne '1') { throw "hum run task_list_flow expected 1, got `$RunTaskListFlow" }

  $RunTransactionOnce = Read-NativeOutput 'run probe transaction_once' $Hum @('run', 'examples/probes/transaction_once.hum', '--entry', 'transfer', '--args', '10')
  if ($RunTransactionOnce.Trim() -ne 'ok') { throw "hum run transaction_once expected ok, got `$RunTransactionOnce" }

  $RunSessionJBorrow = Read-NativeOutput 'run Session J borrow fixture' $Hum @('run', 'fixtures/ownership_check/session_j_borrow_pass.hum', '--entry', 'echo', '--args', '7')
  if ($RunSessionJBorrow.Trim() -ne '7') { throw "Session J borrow run expected 7, got `$RunSessionJBorrow" }

  $RunSessionJChange = Read-NativeOutput 'run Session J change fixture' $Hum @('run', 'fixtures/ownership_check/session_j_change_pass.hum', '--entry', 'increment', '--args', '7')
  if ($RunSessionJChange.Trim() -ne '8') { throw "Session J change run expected 8, got `$RunSessionJChange" }

  $RunSessionJConsume = Read-NativeOutput 'run Session J consume fixture' $Hum @('run', 'fixtures/ownership_check/session_j_consume_pass.hum', '--entry', 'consume_demo')
  if ($RunSessionJConsume.Trim() -ne '7') { throw "Session J consume run expected 7, got `$RunSessionJConsume" }

  $RunSessionJUseAfterMove = Read-NativeOutputWithExit 'run Session J use-after-move misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_j_use_after_move_fail.hum', '--entry', 'use_after_move')
  if ($RunSessionJUseAfterMove.ExitCode -ne 2) { throw "Session J use-after-move run expected exit 2, got $($RunSessionJUseAfterMove.ExitCode)" }
  if (-not $RunSessionJUseAfterMove.Output.Contains('H0801')) { throw "Session J use-after-move run expected H0801, got $($RunSessionJUseAfterMove.Output)" }
  if (-not $RunSessionJUseAfterMove.Output.Contains('help:')) { throw "Session J use-after-move run expected blame help, got $($RunSessionJUseAfterMove.Output)" }

  $RunSessionJBorrowWrite = Read-NativeOutputWithExit 'run Session J borrowed-write misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_j_borrow_write_fail.hum', '--entry', 'write_borrow', '--args', '7')
  if ($RunSessionJBorrowWrite.ExitCode -ne 2) { throw "Session J borrowed-write run expected exit 2, got $($RunSessionJBorrowWrite.ExitCode)" }
  if (-not $RunSessionJBorrowWrite.Output.Contains('H0802')) { throw "Session J borrowed-write run expected H0802, got $($RunSessionJBorrowWrite.Output)" }
  if (-not $RunSessionJBorrowWrite.Output.Contains('help:')) { throw "Session J borrowed-write run expected blame help, got $($RunSessionJBorrowWrite.Output)" }

  $RunSessionJDoubleConsume = Read-NativeOutputWithExit 'run Session J double-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_j_double_consume_fail.hum', '--entry', 'double_consume')
  if ($RunSessionJDoubleConsume.ExitCode -ne 2) { throw "Session J double-consume run expected exit 2, got $($RunSessionJDoubleConsume.ExitCode)" }
  if (-not $RunSessionJDoubleConsume.Output.Contains('H0801')) { throw "Session J double-consume run expected H0801, got $($RunSessionJDoubleConsume.Output)" }
  if (-not $RunSessionJDoubleConsume.Output.Contains('help:')) { throw "Session J double-consume run expected blame help, got $($RunSessionJDoubleConsume.Output)" }

  $RunSessionKMissingConsume = Read-NativeOutputWithExit 'run Session K missing-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_k_missing_consume_fail.hum', '--entry', 'transfer_missing', '--args', '0')
  if ($RunSessionKMissingConsume.ExitCode -ne 2) { throw "Session K missing-consume run expected exit 2, got $($RunSessionKMissingConsume.ExitCode)" }
  if (-not $RunSessionKMissingConsume.Output.Contains('H0803')) { throw "Session K missing-consume run expected H0803, got $($RunSessionKMissingConsume.Output)" }
  if (-not $RunSessionKMissingConsume.Output.Contains('help:')) { throw "Session K missing-consume run expected blame help, got $($RunSessionKMissingConsume.Output)" }

  $RunSessionKDoubleConsume = Read-NativeOutputWithExit 'run Session K double-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_k_double_consume_fail.hum', '--entry', 'transfer_double_consume')
  if ($RunSessionKDoubleConsume.ExitCode -ne 2) { throw "Session K double-consume run expected exit 2, got $($RunSessionKDoubleConsume.ExitCode)" }
  if (-not $RunSessionKDoubleConsume.Output.Contains('H0804')) { throw "Session K double-consume run expected H0804, got $($RunSessionKDoubleConsume.Output)" }
  if (-not $RunSessionKDoubleConsume.Output.Contains('commit')) { throw "Session K double-consume run expected prior commit blame, got $($RunSessionKDoubleConsume.Output)" }

  $RunSessionKBranchConsume = Read-NativeOutputWithExit 'run Session K branch-consume misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_k_branch_consume_fail.hum', '--entry', 'branch_consume', '--args', '1')
  if ($RunSessionKBranchConsume.ExitCode -ne 2) { throw "Session K branch-consume run expected exit 2, got $($RunSessionKBranchConsume.ExitCode)" }
  if (-not $RunSessionKBranchConsume.Output.Contains('H0803')) { throw "Session K branch-consume run expected H0803, got $($RunSessionKBranchConsume.Output)" }
  if (-not $RunSessionKBranchConsume.Output.Contains('help:')) { throw "Session K branch-consume run expected blame help, got $($RunSessionKBranchConsume.Output)" }

  $RunSessionLParameterView = Read-NativeOutput 'run Session L parameter returned-view fixture' $Hum @('run', 'fixtures/ownership_check/session_l_return_parameter_view_pass.hum', '--entry', 'echo_view', '--args', 'hello hum')
  if ($RunSessionLParameterView.Trim() -ne 'hello hum') { throw "Session L parameter returned-view run expected hello hum, got `$RunSessionLParameterView" }

  $RunSessionLLocalView = Read-NativeOutputWithExit 'run Session L local returned-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_l_return_view_local_fail.hum', '--entry', 'local_view')
  if ($RunSessionLLocalView.ExitCode -ne 2) { throw "Session L local returned-view run expected exit 2, got $($RunSessionLLocalView.ExitCode)" }
  if (-not $RunSessionLLocalView.Output.Contains('H0805')) { throw "Session L local returned-view run expected H0805, got $($RunSessionLLocalView.Output)" }
  if (-not $RunSessionLLocalView.Output.Contains('help:')) { throw "Session L local returned-view run expected blame help, got $($RunSessionLLocalView.Output)" }

  $RunSessionNFirstWord = Read-NativeOutput 'run Session N first_word derived returned-view fixture' $Hum @('run', 'examples/probes/first_word.hum', '--entry', 'first_word', '--args', 'hum language')
  if ($RunSessionNFirstWord.Trim() -ne 'hum') { throw "Session N first_word run expected hum, got `$RunSessionNFirstWord" }

  $RunSessionNLocalSlice = Read-NativeOutputWithExit 'run Session N local sub-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_n_return_view_local_slice_fail.hum', '--entry', 'local_first_word')
  if ($RunSessionNLocalSlice.ExitCode -ne 2) { throw "Session N local sub-view run expected exit 2, got $($RunSessionNLocalSlice.ExitCode)" }
  if (-not $RunSessionNLocalSlice.Output.Contains('H0805')) { throw "Session N local sub-view run expected H0805, got $($RunSessionNLocalSlice.Output)" }
  if (-not $RunSessionNLocalSlice.Output.Contains('help:')) { throw "Session N local sub-view run expected blame help, got $($RunSessionNLocalSlice.Output)" }

  $RunSessionNLostProvenance = Read-NativeOutputWithExit 'run Session N lost-provenance misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_n_return_view_lost_provenance_fail.hum', '--entry', 'lost_first_word', '--args', 'hum language')
  if ($RunSessionNLostProvenance.ExitCode -ne 2) { throw "Session N lost-provenance run expected exit 2, got $($RunSessionNLostProvenance.ExitCode)" }
  if (-not $RunSessionNLostProvenance.Output.Contains('H0805')) { throw "Session N lost-provenance run expected H0805, got $($RunSessionNLostProvenance.Output)" }
  if (-not $RunSessionNLostProvenance.Output.Contains('non-closed derivation chains remain rejected')) { throw "Session N lost-provenance run expected non-closed help, got $($RunSessionNLostProvenance.Output)" }
  $RunSessionOSwap = Read-NativeOutput 'run Session O swap_xy field-place fixture' $Hum @('run', 'examples/probes/field_places.hum', '--entry', 'swap_xy', '--args', '{x:1,y:2}')
  if ($RunSessionOSwap.Trim() -ne '{x: 2, y: 1}') { throw "Session O swap_xy run expected {x: 2, y: 1}, got `$RunSessionOSwap" }

  $RunSessionOComplete = Read-NativeOutput 'run Session O complete_item field-place fixture' $Hum @('run', 'fixtures/run/session_o_complete_item_field_place.hum', '--entry', 'complete_item_demo')
  if ($RunSessionOComplete.Trim() -ne '{done: true, title: hum}') { throw "Session O complete_item run expected done true with preserved title, got $($RunSessionOComplete)" }

  $RunSessionOBorrowField = Read-NativeOutputWithExit 'run Session O borrowed field-write misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_o_field_write_borrow_fail.hum', '--entry', 'write_borrowed_field', '--args', '{x:1,y:2}')
  if ($RunSessionOBorrowField.ExitCode -ne 2) { throw "Session O borrowed field-write run expected exit 2, got $($RunSessionOBorrowField.ExitCode)" }
  if (-not $RunSessionOBorrowField.Output.Contains('H0802')) { throw "Session O borrowed field-write run expected H0802, got $($RunSessionOBorrowField.Output)" }
  if (-not $RunSessionOBorrowField.Output.Contains('point.x')) { throw "Session O borrowed field-write run expected point.x blame, got $($RunSessionOBorrowField.Output)" }
  if (-not $RunSessionOBorrowField.Output.Contains('help:')) { throw "Session O borrowed field-write run expected blame help, got $($RunSessionOBorrowField.Output)" }

  $RunSessionPBuilder = Read-NativeOutput 'run Session P builder fixture' $Hum @('run', 'examples/probes/list_builder.hum', '--entry', 'builder_demo')
  if ($RunSessionPBuilder.Trim() -ne '[parse, check, run]') { throw "Session P builder run expected [parse, check, run], got `$RunSessionPBuilder" }

  $RunSessionPBuilderContract = Read-NativeOutput 'run Session P builder contract fixture' $Hum @('run', 'examples/probes/list_builder.hum', '--entry', 'builder_contract_demo')
  if ($RunSessionPBuilderContract.Trim() -ne '3') { throw "Session P builder contract run expected 3, got `$RunSessionPBuilderContract" }

  $RunSessionPIterConflict = Read-NativeOutputWithExit 'run Session P iteration-conflict misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_p_append_during_iteration_fail.hum', '--entry', 'append_during_iteration')
  if ($RunSessionPIterConflict.ExitCode -ne 2) { throw "Session P iteration-conflict run expected exit 2, got $($RunSessionPIterConflict.ExitCode)" }
  if (-not $RunSessionPIterConflict.Output.Contains('H0806')) { throw "Session P iteration-conflict run expected H0806, got $($RunSessionPIterConflict.Output)" }
  if (-not $RunSessionPIterConflict.Output.Contains('list_append')) { throw "Session P iteration-conflict run expected list_append blame, got $($RunSessionPIterConflict.Output)" }
  if (-not $RunSessionPIterConflict.Output.Contains('for each')) { throw "Session P iteration-conflict run expected loop blame, got $($RunSessionPIterConflict.Output)" }
  if (-not $RunSessionPIterConflict.Output.Contains('help:')) { throw "Session P iteration-conflict run expected help, got $($RunSessionPIterConflict.Output)" }

  $RunSessionPAddAfterFinish = Read-NativeOutputWithExit 'run Session P add-after-finish misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_p_add_after_finish_fail.hum', '--entry', 'add_after_finish')
  if ($RunSessionPAddAfterFinish.ExitCode -ne 2) { throw "Session P add-after-finish run expected exit 2, got $($RunSessionPAddAfterFinish.ExitCode)" }
  if (-not $RunSessionPAddAfterFinish.Output.Contains('H0801')) { throw "Session P add-after-finish run expected H0801, got $($RunSessionPAddAfterFinish.Output)" }
  if (-not $RunSessionPAddAfterFinish.Output.Contains('help:')) { throw "Session P add-after-finish run expected help, got $($RunSessionPAddAfterFinish.Output)" }

  $RunSessionRDistinctFieldView = Read-NativeOutput 'run Session R distinct-field view fixture' $Hum @('run', 'examples/probes/field_views.hum', '--entry', 'distinct_field_view_survives', '--args', '{left:false,right:false}')
  if ($RunSessionRDistinctFieldView.Trim() -ne 'false') { throw "Session R distinct-field view run expected false, got $RunSessionRDistinctFieldView" }

  $RunSessionRCopyVsView = Read-NativeOutput 'run Session R copy-vs-view fixture' $Hum @('run', 'examples/probes/field_views.hum', '--entry', 'copy_survives_field_write', '--args', '{left:false,right:false}')
  if ($RunSessionRCopyVsView.Trim() -ne 'false') { throw "Session R copy-vs-view run expected false, got $RunSessionRCopyVsView" }

  $RunSessionRStalePoint = Read-NativeOutputWithExit 'run Session R stale point field-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_r_stale_point_field_view_fail.hum', '--entry', 'stale_point_field_view', '--args', '{x:1,y:2}')
  if ($RunSessionRStalePoint.ExitCode -ne 2) { throw "Session R stale point field-view run expected exit 2, got $($RunSessionRStalePoint.ExitCode)" }
  if (-not $RunSessionRStalePoint.Output.Contains('H0807')) { throw "Session R stale point field-view run expected H0807, got $($RunSessionRStalePoint.Output)" }
  if (-not $RunSessionRStalePoint.Output.Contains('x_view')) { throw "Session R stale point field-view run expected x_view blame, got $($RunSessionRStalePoint.Output)" }
  if (-not $RunSessionRStalePoint.Output.Contains('point.x')) { throw "Session R stale point field-view run expected point.x write blame, got $($RunSessionRStalePoint.Output)" }
  if (-not $RunSessionRStalePoint.Output.Contains('re-borrow after the write')) { throw "Session R stale point field-view run expected repair help, got $($RunSessionRStalePoint.Output)" }

  $RunSessionRStaleItem = Read-NativeOutputWithExit 'run Session R stale item field-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_r_stale_item_field_view_fail.hum', '--entry', 'stale_item_done_view', '--args', '{done:false,pinned:false}')
  if ($RunSessionRStaleItem.ExitCode -ne 2) { throw "Session R stale item field-view run expected exit 2, got $($RunSessionRStaleItem.ExitCode)" }
  if (-not $RunSessionRStaleItem.Output.Contains('H0807')) { throw "Session R stale item field-view run expected H0807, got $($RunSessionRStaleItem.Output)" }
  if (-not $RunSessionRStaleItem.Output.Contains('done_view')) { throw "Session R stale item field-view run expected done_view blame, got $($RunSessionRStaleItem.Output)" }
  if (-not $RunSessionRStaleItem.Output.Contains('item.done')) { throw "Session R stale item field-view run expected item.done write blame, got $($RunSessionRStaleItem.Output)" }

  $RunSessionSElementView = Read-NativeOutput 'run Session S element-view fixture' $Hum @('run', 'examples/probes/element_views.hum', '--entry', 'element_view_before_growth')
  if ($RunSessionSElementView.Trim() -ne 'parse') { throw "Session S element-view run expected parse, got $RunSessionSElementView" }

  $RunSessionSCopyVsView = Read-NativeOutput 'run Session S element copy-vs-view fixture' $Hum @('run', 'examples/probes/element_views.hum', '--entry', 'copied_element_survives_growth')
  if ($RunSessionSCopyVsView.Trim() -ne 'parse') { throw "Session S copied element run expected parse, got $RunSessionSCopyVsView" }

  $RunSessionSStaleElement = Read-NativeOutputWithExit 'run Session S stale element-view misuse fixture' $Hum @('run', 'fixtures/ownership_check/session_s_stale_element_view_fail.hum', '--entry', 'stale_element_view_after_append')
  if ($RunSessionSStaleElement.ExitCode -ne 2) { throw "Session S stale element-view run expected exit 2, got $($RunSessionSStaleElement.ExitCode)" }
  if (-not $RunSessionSStaleElement.Output.Contains('H0807')) { throw "Session S stale element-view run expected H0807, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('first_view')) { throw "Session S stale element-view run expected first_view blame, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('items[0]')) { throw "Session S stale element-view run expected items[0] source, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('list_append grew items')) { throw "Session S stale element-view run expected append site, got $($RunSessionSStaleElement.Output)" }
  if (-not $RunSessionSStaleElement.Output.Contains('re-borrow after the append')) { throw "Session S stale element-view run expected repair help, got $($RunSessionSStaleElement.Output)" }

  $RunSessionSOverlap = Read-NativeOutputWithExit 'run Session S H0806/H0807 overlap fixture' $Hum @('run', 'fixtures/ownership_check/session_s_append_iteration_view_overlap_fail.hum', '--entry', 'append_during_iteration_with_element_view')
  if ($RunSessionSOverlap.ExitCode -ne 2) { throw "Session S overlap run expected exit 2, got $($RunSessionSOverlap.ExitCode)" }
  if (-not $RunSessionSOverlap.Output.Contains('H0806')) { throw "Session S overlap run expected H0806, got $($RunSessionSOverlap.Output)" }
  if ($RunSessionSOverlap.Output.Contains('H0807')) { throw "Session S overlap run must not also report H0807, got $($RunSessionSOverlap.Output)" }
  if (-not $RunSessionSOverlap.Output.Contains('list_append')) { throw "Session S overlap run expected list_append blame, got $($RunSessionSOverlap.Output)" }
  if (-not $RunSessionSOverlap.Output.Contains('for each')) { throw "Session S overlap run expected loop blame, got $($RunSessionSOverlap.Output)" }
  $CheckJson = Read-NativeOutput 'check JSON' $Hum @('check', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'check JSON' $CheckJson
  if (-not $CheckJson.Contains('"schema": "hum.check.v0"')) { throw 'check JSON is missing hum.check.v0 schema' }

  $EvidenceJson = Read-NativeOutput 'evidence report JSON' $Hum @('evidence', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'evidence report JSON' $EvidenceJson
  if (-not $EvidenceJson.Contains('"schema": "hum.evidence.v0"')) { throw 'evidence report JSON is missing hum.evidence.v0 schema' }
  if (-not $EvidenceJson.Contains('"linked_evidence"')) { throw 'evidence report JSON is missing linked_evidence' }
  if (-not $EvidenceJson.Contains('"verification_status": "linked"')) { throw 'evidence report JSON is missing linked evidence status' }

  $MathObligationsJson = Read-NativeOutput 'math obligations JSON' $Hum @('math-obligations', '--format', 'json', 'examples/control_flow.hum')
  Assert-Json 'math obligations JSON' $MathObligationsJson
  if (-not $MathObligationsJson.Contains('"schema": "hum.math_obligations.v0"')) { throw 'math obligations JSON is missing hum.math_obligations.v0 schema' }
  if (-not $MathObligationsJson.Contains('"schema_version": "hum.math_obligation.v0"')) { throw 'math obligations JSON is missing hum.math_obligation.v0 entries' }
  if (-not $MathObligationsJson.Contains('"obligation_kind": "allocation_freedom"')) { throw 'math obligations JSON is missing allocation_freedom obligation' }
  if (-not $MathObligationsJson.Contains('"confidence_requested": "evidence_only"')) { throw 'math obligations JSON must keep V0 exports evidence_only' }

  $ResourceReportJson = Read-NativeOutput 'resource report JSON' $Hum @('resource-report', '--format', 'json', 'examples/control_flow.hum')
  Assert-Json 'resource report JSON' $ResourceReportJson
  if (-not $ResourceReportJson.Contains('"schema": "hum.resource_report.v0"')) { throw 'resource report JSON is missing hum.resource_report.v0 schema' }
  if (-not $ResourceReportJson.Contains('"claim_kind": "time_complexity"')) { throw 'resource report JSON is missing time_complexity claims' }
  if (-not $ResourceReportJson.Contains('"verification_status": "declared"')) { throw 'resource report JSON must keep V0 claims declared' }
  if (-not $ResourceReportJson.Contains('"proof_status": "not_proven"')) { throw 'resource report JSON must not pretend proofs exist' }
  if (-not $ResourceReportJson.Contains('"benchmark_status": "not_measured"')) { throw 'resource report JSON must not pretend benchmarks exist' }

  $CorePreviewJson = Read-NativeOutput 'Core preview JSON' $Hum @('core-preview', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'Core preview JSON' $CorePreviewJson
  if (-not $CorePreviewJson.Contains('"schema": "hum.core_preview.v0"')) { throw 'Core preview JSON is missing hum.core_preview.v0 schema' }
  if (-not $CorePreviewJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'Core preview JSON is missing Core Hum contract schema' }
  if (-not $CorePreviewJson.Contains('"type_check_schema": "hum.type_check.v0"')) { throw 'Core preview JSON is missing type check schema provenance' }
  if (-not $CorePreviewJson.Contains('"execution_ready": 0')) { throw 'Core preview JSON must not claim execution readiness' }
  if (-not $CorePreviewJson.Contains('"core_operation": "return"')) { throw 'Core preview JSON is missing return operation preview' }
  if (-not $CorePreviewJson.Contains('"core_operation": "store_write_deferred"')) { throw 'Core preview JSON is missing deferred store write blocker' }
  if (-not $CorePreviewJson.Contains('"expression_previews"')) { throw 'Core preview JSON is missing expression preview counts' }
  if (-not $CorePreviewJson.Contains('"name_status": "name_preview_v0"')) { throw 'Core preview JSON is missing name preview status' }
  if (-not $CorePreviewJson.Contains('"name_preview"')) { throw 'Core preview JSON is missing name_preview facts' }
  if (-not $CorePreviewJson.Contains('"scope_model": "lexical_block_scope_preview_v0"')) { throw 'Core preview JSON is missing lexical block scope model' }
  if (-not $CorePreviewJson.Contains('"checked_resolver_status": "not_run_v0"')) { throw 'Core preview JSON must keep checked resolver status separate from preview facts' }
  if (-not $CorePreviewJson.Contains('"resolver_diagnostic_status": "preview_facts_only_v0"')) { throw 'Core preview JSON must mark name facts as non-diagnostic preview facts' }
  if (-not $CorePreviewJson.Contains('"resolver_diagnostic_count": 0')) { throw 'Core preview JSON must not emit checked resolver diagnostics yet' }
  if (-not $CorePreviewJson.Contains('"scope_count"')) { throw 'Core preview JSON is missing name scope counts' }
  if (-not $CorePreviewJson.Contains('"scopes"')) { throw 'Core preview JSON is missing name scope list' }
  if (-not $CorePreviewJson.Contains('"scope_kind": "root"')) { throw 'Core preview JSON is missing root name scope' }
  if (-not $CorePreviewJson.Contains('"scope_kind": "if_statement"')) { throw 'Core preview JSON is missing if statement name scope' }
  if (-not $CorePreviewJson.Contains('"parent_scope_id"')) { throw 'Core preview JSON is missing parent scope links' }
  if (-not $CorePreviewJson.Contains('"definition_kind": "parameter"')) { throw 'Core preview JSON is missing parameter definition preview' }
  if (-not $CorePreviewJson.Contains('"definition_kind": "let_binding"')) { throw 'Core preview JSON is missing let binding definition preview' }
  if (-not $CorePreviewJson.Contains('"reference_kind": "name_ref"')) { throw 'Core preview JSON is missing name reference preview' }
  if (-not $CorePreviewJson.Contains('"resolution_status": "resolved_preview_v0"')) { throw 'Core preview JSON is missing resolved name reference preview' }
  if (-not $CorePreviewJson.Contains('"resolution_status": "external_reference_preview_v0"')) { throw 'Core preview JSON is missing external name reference preview' }
  if (-not $CorePreviewJson.Contains('"unresolved_name_references": 0')) { throw 'Core preview JSON should report zero unresolved name references for reference fixture' }
  if (-not $CorePreviewJson.Contains('global_or_type_name_resolution_not_implemented')) { throw 'Core preview JSON is missing external name resolution honesty reason' }
  if (-not $CorePreviewJson.Contains('"block_status": "block_preview_v0"')) { throw 'Core preview JSON is missing block preview status' }
  if (-not $CorePreviewJson.Contains('"block_preview"')) { throw 'Core preview JSON is missing block_preview tree' }
  if (-not $CorePreviewJson.Contains('"block_count"')) { throw 'Core preview JSON is missing block_count' }
  if (-not $CorePreviewJson.Contains('"max_block_depth"')) { throw 'Core preview JSON is missing max_block_depth summary' }
  if (-not $CorePreviewJson.Contains('"unmatched_block_closes": 0')) { throw 'Core preview JSON should report zero unmatched block closes for reference fixture' }
  if (-not $CorePreviewJson.Contains('"unclosed_blocks": 0')) { throw 'Core preview JSON should report zero unclosed blocks for reference fixture' }
  if (-not $CorePreviewJson.Contains('"node_kind": "block"')) { throw 'Core preview JSON is missing block nodes' }
  if (-not $CorePreviewJson.Contains('"node_kind": "statement_ref"')) { throw 'Core preview JSON is missing statement refs in block tree' }
  if (-not $CorePreviewJson.Contains('"block_kind": "record_construction"')) { throw 'Core preview JSON is missing record construction block preview' }
  if (-not $CorePreviewJson.Contains('"header_statement_index"')) { throw 'Core preview JSON is missing block header statement index' }
  if (-not $CorePreviewJson.Contains('"closing_statement_index"')) { throw 'Core preview JSON is missing block closing statement index' }
  if (-not $CorePreviewJson.Contains('"expression_preview"')) { throw 'Core preview JSON is missing statement expression previews' }
  if (-not $CorePreviewJson.Contains('"expression_ast_nodes"')) { throw 'Core preview JSON is missing expression AST node counts' }
  if (-not $CorePreviewJson.Contains('"atoms"')) { throw 'Core preview JSON is missing expression atoms' }
  if (-not $CorePreviewJson.Contains('"operators"')) { throw 'Core preview JSON is missing expression operators' }
  if (-not $CorePreviewJson.Contains('"ast"')) { throw 'Core preview JSON is missing expression AST previews' }
  if (-not $CorePreviewJson.Contains('"node_count"')) { throw 'Core preview JSON is missing expression AST node_count' }
  if (-not $CorePreviewJson.Contains('"form": "binary_operation_candidate"')) { throw 'Core preview JSON is missing binary operation AST form' }
  if (-not $CorePreviewJson.Contains('"typed_expression_previews"')) { throw 'Core preview JSON is missing typed expression preview counts' }
  if (-not $CorePreviewJson.Contains('"type_status": "not_type_checked_v0"')) { throw 'Core preview JSON must keep unchecked expression AST type slots where no checked return fact exists' }
  if (-not $CorePreviewJson.Contains('"type_status": "checked_trivial_return_type_v0"')) { throw 'Core preview JSON is missing checked return expression type status' }
  if (-not $CorePreviewJson.Contains('"type_text": "WorkItem"')) { throw 'Core preview JSON is missing checked return expression type text' }
  if (-not $CorePreviewJson.Contains('"type_source": "record_literal_constructor_v0"')) { throw 'Core preview JSON is missing checked return expression type source' }
  if (-not $CorePreviewJson.Contains('"effect_status": "not_effect_checked_v0"')) { throw 'Core preview JSON must keep expression AST effect status unchecked' }
  if (-not $CorePreviewJson.Contains('"status": "compound_preview_v0"')) { throw 'Core preview JSON is missing compound expression preview status' }
  if (-not $CorePreviewJson.Contains('"kind": "path_or_field_read"')) { throw 'Core preview JSON is missing path or field read expression kind' }
  if (-not $CorePreviewJson.Contains('surface_save_requires_store_lowering')) { throw 'Core preview JSON is missing store save lowering blocker' }
  if (-not $CorePreviewJson.Contains('no executable semantics')) { throw 'Core preview JSON must keep V0 non-execution claim' }
  if (-not $CorePreviewJson.Contains('no independent type checking')) { throw 'Core preview JSON must keep type-check provenance honesty claim' }
  if (-not $CorePreviewJson.Contains('no broad expression type checking')) { throw 'Core preview JSON must keep broad expression type-checking non-goal' }
  if (-not $CorePreviewJson.Contains('no module or global name resolution')) { throw 'Core preview JSON must keep V0 name-resolution non-goal' }
  if (-not $CorePreviewJson.Contains('no checked name resolution')) { throw 'Core preview JSON must keep V0 checked name-resolution non-goal' }

  $CoreLowerJson = Read-NativeOutput 'Core lower JSON' $Hum @('core-lower', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'Core lower JSON' $CoreLowerJson
  if (-not $CoreLowerJson.Contains('"schema": "hum.core_lower.v0"')) { throw 'Core lower JSON is missing hum.core_lower.v0 schema' }
  if (-not $CoreLowerJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'Core lower JSON is missing Core Hum contract schema' }
  if (-not $CoreLowerJson.Contains('"core_preview_schema": "hum.core_preview.v0"')) { throw 'Core lower JSON is missing Core preview schema provenance' }
  if (-not $CoreLowerJson.Contains('"resolve_schema": "hum.resolve.v0"')) { throw 'Core lower JSON is missing resolver schema provenance' }
  if (-not $CoreLowerJson.Contains('"type_check_schema": "hum.type_check.v0"')) { throw 'Core lower JSON is missing type check schema provenance' }
  if (-not $CoreLowerJson.Contains('"lowering_status": "unverified_core_artifact_v0"')) { throw 'Core lower JSON is missing unverified artifact status' }
  if (-not $CoreLowerJson.Contains('"verification_status": "unverified_v0"')) { throw 'Core lower JSON must mark artifacts unverified' }
  if (-not $CoreLowerJson.Contains('"execution_ready": 0')) { throw 'Core lower JSON must not claim execution readiness' }
  if (-not $CoreLowerJson.Contains('"ir_ready": 0')) { throw 'Core lower JSON must not claim IR readiness' }
  if (-not $CoreLowerJson.Contains('"core_operation": "return"')) { throw 'Core lower JSON is missing return operation' }
  if (-not $CoreLowerJson.Contains('surface_save_requires_store_lowering')) { throw 'Core lower JSON is missing store save lowering blocker' }
  if (-not $CoreLowerJson.Contains('no executable semantics')) { throw 'Core lower JSON must keep V0 non-execution claim' }
  if (-not $CoreLowerJson.Contains('no Hum IR emission')) { throw 'Core lower JSON must keep V0 non-IR-emission claim' }

  $CoreVerifyJson = Read-NativeOutput 'Core verify JSON' $Hum @('core-verify', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'Core verify JSON' $CoreVerifyJson
  if (-not $CoreVerifyJson.Contains('"schema": "hum.core_verify.v0"')) { throw 'Core verify JSON is missing hum.core_verify.v0 schema' }
  if (-not $CoreVerifyJson.Contains('"core_lower_schema": "hum.core_lower.v0"')) { throw 'Core verify JSON is missing core lower schema link' }
  if (-not $CoreVerifyJson.Contains('"verification_status": "verified_non_executing_core_artifact_v0"')) { throw 'Core verify JSON is missing verified non-executing status' }
  if (-not $CoreVerifyJson.Contains('"mode": "non_executing_artifact_invariant_check_v0"')) { throw 'Core verify JSON is missing invariant-check mode' }
  if (-not $CoreVerifyJson.Contains('"rule": "source_span_sane"')) { throw 'Core verify JSON is missing source span checks' }
  if (-not $CoreVerifyJson.Contains('"rule": "operation_family_status_consistent"')) { throw 'Core verify JSON is missing operation consistency checks' }
  if (-not $CoreVerifyJson.Contains('"failed_checks": 0')) { throw 'Core verify JSON should pass for reference fixture' }
  if (-not $CoreVerifyJson.Contains('"execution_ready": 0')) { throw 'Core verify JSON must not claim execution readiness' }
  if (-not $CoreVerifyJson.Contains('"ir_ready": 0')) { throw 'Core verify JSON must not claim IR readiness' }
  if (-not $CoreVerifyJson.Contains('no memory-safety proof')) { throw 'Core verify JSON must keep memory-safety non-claim' }

  $ResolveJson = Read-NativeOutput 'resolve JSON' $Hum @('resolve', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'resolve JSON' $ResolveJson
  if (-not $ResolveJson.Contains('"schema": "hum.resolve.v0"')) { throw 'resolve JSON is missing hum.resolve.v0 schema' }
  if (-not $ResolveJson.Contains('"mode": "source_analysis_only_no_type_or_borrow_check"')) { throw 'resolve JSON is missing source-analysis mode' }
  if (-not $ResolveJson.Contains('"status": "checked_resolver_v0"')) { throw 'resolve JSON should pass for reference fixture' }
  if (-not $ResolveJson.Contains('"resolver_errors": 0')) { throw 'resolve JSON should have zero resolver errors for reference fixture' }
  if (-not $ResolveJson.Contains('"scopes"')) { throw 'resolve JSON is missing scopes' }
  if (-not $ResolveJson.Contains('"definitions"')) { throw 'resolve JSON is missing definitions' }
  if (-not $ResolveJson.Contains('"references"')) { throw 'resolve JSON is missing references' }
  if (-not $ResolveJson.Contains('"definition_kind": "store"')) { throw 'resolve JSON is missing store definition' }
  if (-not $ResolveJson.Contains('"definition_kind": "declared_change_permission"')) { throw 'resolve JSON is missing declared change permission' }
  if (-not $ResolveJson.Contains('"reference_kind": "declared_change"')) { throw 'resolve JSON is missing declared change reference' }
  if (-not $ResolveJson.Contains('"reference_kind": "store_write_target"')) { throw 'resolve JSON is missing store write target reference' }
  if (-not $ResolveJson.Contains('"resolution_status": "resolved_v0"')) { throw 'resolve JSON is missing resolved references' }
  if (-not $ResolveJson.Contains('"duplicate_definitions": 0')) { throw 'resolve JSON should have zero duplicate definitions for reference fixture' }
  if (-not $ResolveJson.Contains('"no type checking"')) { throw 'resolve JSON must not claim type checking' }
  if (-not $ResolveJson.Contains('"no borrow checking"')) { throw 'resolve JSON must not claim borrow checking' }
  if (-not $ResolveJson.Contains('"no executable semantics"')) { throw 'resolve JSON must not claim execution' }

  $TypeEnvJson = Read-NativeOutput 'type environment JSON' $Hum @('type-env', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'type environment JSON' $TypeEnvJson
  if (-not $TypeEnvJson.Contains('"schema": "hum.type_env.v0"')) { throw 'type environment JSON is missing hum.type_env.v0 schema' }
  if (-not $TypeEnvJson.Contains('"mode": "declaration_inventory_no_type_check"')) { throw 'type environment JSON is missing declaration-inventory mode' }
  if (-not $TypeEnvJson.Contains('"resolver"')) { throw 'type environment JSON is missing resolver summary' }
  if (-not $TypeEnvJson.Contains('"schema": "hum.resolve.v0"')) { throw 'type environment JSON is missing resolver schema link' }
  if (-not $TypeEnvJson.Contains('"type_names"')) { throw 'type environment JSON is missing type_names' }
  if (-not $TypeEnvJson.Contains('"declarations"')) { throw 'type environment JSON is missing declarations' }
  if (-not $TypeEnvJson.Contains('"resolver_definition_id"')) { throw 'type environment JSON is missing resolver definition links' }
  if (-not $TypeEnvJson.Contains('"status": "type_environment_v0"')) { throw 'type environment JSON should pass for reference fixture' }
  if (-not $TypeEnvJson.Contains('"unknown_type_references": 0')) { throw 'type environment JSON should have zero unknown type references for reference fixture' }
  if (-not $TypeEnvJson.Contains('"no full type checking"')) { throw 'type environment JSON must not claim full type checking' }
  if (-not $TypeEnvJson.Contains('"no executable semantics"')) { throw 'type environment JSON must not claim execution' }

  $TypeCheckJson = Read-NativeOutput 'type check JSON' $Hum @('type-check', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'type check JSON' $TypeCheckJson
  if (-not $TypeCheckJson.Contains('"schema": "hum.type_check.v0"')) { throw 'type check JSON is missing hum.type_check.v0 schema' }
  if (-not $TypeCheckJson.Contains('"mode": "declaration_annotation_and_trivial_return_check_v0"')) { throw 'type check JSON is missing declaration-and-return mode' }
  if (-not $TypeCheckJson.Contains('"schema": "hum.type_env.v0"')) { throw 'type check JSON is missing type-env schema link' }
  if (-not $TypeCheckJson.Contains('"status": "declaration_annotations_and_trivial_returns_checked_v0"')) { throw 'type check JSON should pass for reference fixture' }
  if (-not $TypeCheckJson.Contains('"type_errors": 0')) { throw 'type check JSON should have zero type errors for reference fixture' }
  if (-not $TypeCheckJson.Contains('"checked_returns"')) { throw 'type check JSON is missing checked return rows' }
  if (-not $TypeCheckJson.Contains('"accepted_return_expression_v0"')) { throw 'type check JSON is missing accepted return check status' }
  if (-not $TypeCheckJson.Contains('"expected_value_type": "WorkItem"')) { throw 'type check JSON is missing Result success value type' }
  if (-not $TypeCheckJson.Contains('"accepted_type_reference_v0"')) { throw 'type check JSON is missing accepted type references' }
  if (-not $TypeCheckJson.Contains('"no full expression type inference"')) { throw 'type check JSON must not claim full expression inference' }
  if (-not $TypeCheckJson.Contains('"no executable semantics"')) { throw 'type check JSON must not claim execution' }

  $FullTypeCheckJson = Read-NativeOutput 'full type check JSON' $Hum @('full-type-check', '--format', 'json', 'fixtures/full_type_check/simple_pass.hum')
  Assert-Json 'full type check JSON' $FullTypeCheckJson
  if (-not $FullTypeCheckJson.Contains('"schema": "hum.full_type_check.v0"')) { throw 'full type check JSON is missing hum.full_type_check.v0 schema' }
  if (-not $FullTypeCheckJson.Contains('"status": "recognized_core_body_types_checked_v0"')) { throw 'full type check JSON should pass for simple fixture' }
  if (-not $FullTypeCheckJson.Contains('"mode": "recognized_core_body_type_gate_v0"')) { throw 'full type check JSON is missing recognized body type gate mode' }
  if (-not $FullTypeCheckJson.Contains('"blocking_issues": 0')) { throw 'full type check JSON should have zero blocking issues for simple fixture' }
  if (-not $FullTypeCheckJson.Contains('"accepted_statement_type_v0"')) { throw 'full type check JSON is missing accepted statement type facts' }
  if (-not $FullTypeCheckJson.Contains('"accepted_inferred_binding_type_v0"')) { throw 'full type check JSON is missing inferred binding type facts' }
  if (-not $FullTypeCheckJson.Contains('"execution_ready": 0')) { throw 'full type check JSON must not claim execution readiness' }
  if (-not $FullTypeCheckJson.Contains('"ir_ready": 0')) { throw 'full type check JSON must not claim IR readiness' }
  if (-not $FullTypeCheckJson.Contains('no memory-safety proof')) { throw 'full type check JSON must keep memory-safety non-claim' }
  $EffectCheckJson = Read-NativeOutput 'effect check JSON' $Hum @('effect-check', '--format', 'json', 'fixtures/effect_check/simple_pass.hum')
  Assert-Json 'effect check JSON' $EffectCheckJson
  if (-not $EffectCheckJson.Contains('"schema": "hum.effect_check.v0"')) { throw 'effect check JSON is missing hum.effect_check.v0 schema' }
  if (-not $EffectCheckJson.Contains('"status": "recognized_core_effects_checked_v0"')) { throw 'effect check JSON should pass for simple fixture' }
  if (-not $EffectCheckJson.Contains('"mode": "recognized_core_effect_gate_v0"')) { throw 'effect check JSON is missing recognized effect gate mode' }
  if (-not $EffectCheckJson.Contains('"blocking_issues": 0')) { throw 'effect check JSON should have zero blocking issues for simple fixture' }
  if (-not $EffectCheckJson.Contains('"accepted_declared_failure_v0"')) { throw 'effect check JSON is missing declared failure facts' }
  if (-not $EffectCheckJson.Contains('"accepted_local_mutation_v0"')) { throw 'effect check JSON is missing local mutation facts' }
  if (-not $EffectCheckJson.Contains('"execution_ready": 0')) { throw 'effect check JSON must not claim execution readiness' }
  if (-not $EffectCheckJson.Contains('"ir_ready": 0')) { throw 'effect check JSON must not claim IR readiness' }
  if (-not $EffectCheckJson.Contains('no memory-safety proof')) { throw 'effect check JSON must keep memory-safety non-claim' }

  $OwnershipCheckJson = Read-NativeOutput 'ownership check JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_consume_pass.hum')
  Assert-Json 'ownership check JSON' $OwnershipCheckJson
  if (-not $OwnershipCheckJson.Contains('"schema": "hum.ownership_check.v0"')) { throw 'ownership check JSON is missing hum.ownership_check.v0 schema' }
  if (-not $OwnershipCheckJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw 'ownership check JSON should pass for Session J consume fixture' }
  if (-not $OwnershipCheckJson.Contains('"accepted_consume_argument_move_v0"')) { throw 'ownership check JSON is missing accepted consume move fact' }
  if (-not $OwnershipCheckJson.Contains('"accepted_return_move_v0"')) { throw 'ownership check JSON is missing accepted return move fact' }
  if (-not $OwnershipCheckJson.Contains('"execution_ready": 0')) { throw 'ownership check JSON must not claim execution readiness' }
  if (-not $OwnershipCheckJson.Contains('"ir_ready": 0')) { throw 'ownership check JSON must not claim IR readiness' }
  if (-not $OwnershipCheckJson.Contains('no memory-safety proof')) { throw 'ownership check JSON must keep memory-safety non-claim' }

  $OwnershipCheckChangeJson = Read-NativeOutput 'ownership check Session J change JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_change_pass.hum')
  Assert-Json 'ownership check Session J change JSON' $OwnershipCheckChangeJson
  if (-not $OwnershipCheckChangeJson.Contains('"accepted_parameter_mutation_v0"')) { throw 'ownership check JSON is missing accepted parameter mutation fact' }

  $OwnershipUseAfterMoveJson = Read-NativeOutputWithExit 'ownership check Session J use-after-move JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_use_after_move_fail.hum')
  if ($OwnershipUseAfterMoveJson.ExitCode -ne 1) { throw "ownership check use-after-move expected exit 1, got $($OwnershipUseAfterMoveJson.ExitCode)" }
  Assert-Json 'ownership check Session J use-after-move JSON' $OwnershipUseAfterMoveJson.Output
  if (-not $OwnershipUseAfterMoveJson.Output.Contains('"status": "ownership_errors_v0"')) { throw "ownership check use-after-move expected ownership_errors_v0, got $($OwnershipUseAfterMoveJson.Output)" }
  if (-not $OwnershipUseAfterMoveJson.Output.Contains('"diagnostic_code": "H0801"')) { throw "ownership check use-after-move expected H0801, got $($OwnershipUseAfterMoveJson.Output)" }
  if (-not $OwnershipUseAfterMoveJson.Output.Contains('"help"')) { throw "ownership check use-after-move expected blame help, got $($OwnershipUseAfterMoveJson.Output)" }

  $OwnershipBorrowWriteJson = Read-NativeOutputWithExit 'ownership check Session J borrowed-write JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_borrow_write_fail.hum')
  if ($OwnershipBorrowWriteJson.ExitCode -ne 1) { throw "ownership check borrowed-write expected exit 1, got $($OwnershipBorrowWriteJson.ExitCode)" }
  Assert-Json 'ownership check Session J borrowed-write JSON' $OwnershipBorrowWriteJson.Output
  if (-not $OwnershipBorrowWriteJson.Output.Contains('"diagnostic_code": "H0802"')) { throw "ownership check borrowed-write expected H0802, got $($OwnershipBorrowWriteJson.Output)" }
  if (-not $OwnershipBorrowWriteJson.Output.Contains('"help"')) { throw "ownership check borrowed-write expected blame help, got $($OwnershipBorrowWriteJson.Output)" }

  $OwnershipDoubleConsumeJson = Read-NativeOutputWithExit 'ownership check Session J double-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_j_double_consume_fail.hum')
  if ($OwnershipDoubleConsumeJson.ExitCode -ne 1) { throw "ownership check double-consume expected exit 1, got $($OwnershipDoubleConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session J double-consume JSON' $OwnershipDoubleConsumeJson.Output
  if (-not $OwnershipDoubleConsumeJson.Output.Contains('"diagnostic_code": "H0801"')) { throw "ownership check double-consume expected H0801, got $($OwnershipDoubleConsumeJson.Output)" }

  $OwnershipTransactionOnceJson = Read-NativeOutput 'ownership check Session K transaction JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/transaction_once.hum')
  Assert-Json 'ownership check Session K transaction JSON' $OwnershipTransactionOnceJson
  if (-not $OwnershipTransactionOnceJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check transaction expected pass, got $OwnershipTransactionOnceJson" }
  if (-not $OwnershipTransactionOnceJson.Contains('"blocking_issues": 0')) { throw "ownership check transaction expected zero blockers, got $OwnershipTransactionOnceJson" }

  $OwnershipKMissingConsumeJson = Read-NativeOutputWithExit 'ownership check Session K missing-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_k_missing_consume_fail.hum')
  if ($OwnershipKMissingConsumeJson.ExitCode -ne 1) { throw "ownership check missing-consume expected exit 1, got $($OwnershipKMissingConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session K missing-consume JSON' $OwnershipKMissingConsumeJson.Output
  if (-not $OwnershipKMissingConsumeJson.Output.Contains('"diagnostic_code": "H0803"')) { throw "ownership check missing-consume expected H0803, got $($OwnershipKMissingConsumeJson.Output)" }
  if (-not $OwnershipKMissingConsumeJson.Output.Contains('if line 31 true')) { throw "ownership check missing-consume expected true path name, got $($OwnershipKMissingConsumeJson.Output)" }

  $OwnershipKDoubleConsumeJson = Read-NativeOutputWithExit 'ownership check Session K double-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_k_double_consume_fail.hum')
  if ($OwnershipKDoubleConsumeJson.ExitCode -ne 1) { throw "ownership check double-consume expected exit 1, got $($OwnershipKDoubleConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session K double-consume JSON' $OwnershipKDoubleConsumeJson.Output
  if (-not $OwnershipKDoubleConsumeJson.Output.Contains('"diagnostic_code": "H0804"')) { throw "ownership check double-consume expected H0804, got $($OwnershipKDoubleConsumeJson.Output)" }
  if (-not $OwnershipKDoubleConsumeJson.Output.Contains('already consumed by commit')) { throw "ownership check double-consume expected previous commit in help, got $($OwnershipKDoubleConsumeJson.Output)" }

  $OwnershipKBranchConsumeJson = Read-NativeOutputWithExit 'ownership check Session K branch-consume JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_k_branch_consume_fail.hum')
  if ($OwnershipKBranchConsumeJson.ExitCode -ne 1) { throw "ownership check branch-consume expected exit 1, got $($OwnershipKBranchConsumeJson.ExitCode)" }
  Assert-Json 'ownership check Session K branch-consume JSON' $OwnershipKBranchConsumeJson.Output
  if (-not $OwnershipKBranchConsumeJson.Output.Contains('"diagnostic_code": "H0803"')) { throw "ownership check branch-consume expected H0803, got $($OwnershipKBranchConsumeJson.Output)" }
  if (-not $OwnershipKBranchConsumeJson.Output.Contains('if line 24 false')) { throw "ownership check branch-consume expected false path name, got $($OwnershipKBranchConsumeJson.Output)" }

  $OwnershipLParameterViewJson = Read-NativeOutput 'ownership check Session L parameter returned-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_l_return_parameter_view_pass.hum')
  Assert-Json 'ownership check Session L parameter returned-view JSON' $OwnershipLParameterViewJson
  if (-not $OwnershipLParameterViewJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check returned-view expected pass, got $OwnershipLParameterViewJson" }
  if (-not $OwnershipLParameterViewJson.Contains('"return_dependencies"')) { throw "ownership check returned-view is missing return_dependencies, got $OwnershipLParameterViewJson" }
  if (-not $OwnershipLParameterViewJson.Contains('"source": "text"')) { throw "ownership check returned-view expected source text, got $OwnershipLParameterViewJson" }
  if (-not $OwnershipLParameterViewJson.Contains('"status": "accepted_return_dependency_parameter_v0"')) { throw "ownership check returned-view expected accepted dependency, got $OwnershipLParameterViewJson" }

  $OwnershipLLocalViewJson = Read-NativeOutputWithExit 'ownership check Session L local returned-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_l_return_view_local_fail.hum')
  if ($OwnershipLLocalViewJson.ExitCode -ne 1) { throw "ownership check local returned-view expected exit 1, got $($OwnershipLLocalViewJson.ExitCode)" }
  Assert-Json 'ownership check Session L local returned-view JSON' $OwnershipLLocalViewJson.Output
  if (-not $OwnershipLLocalViewJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check local returned-view expected H0805, got $($OwnershipLLocalViewJson.Output)" }
  if (-not $OwnershipLLocalViewJson.Output.Contains('"status": "rejected_return_dependency_local_v0"')) { throw "ownership check local returned-view expected local rejection, got $($OwnershipLLocalViewJson.Output)" }

  $OwnershipLInternalViewJson = Read-NativeOutputWithExit 'ownership check Session L internal-reference returned-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_l_return_view_internal_fail.hum')
  if ($OwnershipLInternalViewJson.ExitCode -ne 1) { throw "ownership check internal-reference returned-view expected exit 1, got $($OwnershipLInternalViewJson.ExitCode)" }
  Assert-Json 'ownership check Session L internal-reference returned-view JSON' $OwnershipLInternalViewJson.Output
  if (-not $OwnershipLInternalViewJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check internal-reference returned-view expected H0805, got $($OwnershipLInternalViewJson.Output)" }
  if (-not $OwnershipLInternalViewJson.Output.Contains('"status": "rejected_return_dependency_internal_reference_v0"')) { throw "ownership check internal-reference returned-view expected internal-reference rejection, got $($OwnershipLInternalViewJson.Output)" }

  $OwnershipNFirstWordJson = Read-NativeOutput 'ownership check Session N first_word JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/first_word.hum')
  Assert-Json 'ownership check Session N first_word JSON' $OwnershipNFirstWordJson
  if (-not $OwnershipNFirstWordJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check first_word expected pass, got $OwnershipNFirstWordJson" }
  if (-not $OwnershipNFirstWordJson.Contains('"source": "text"')) { throw "ownership check first_word expected source text, got $OwnershipNFirstWordJson" }
  if (-not $OwnershipNFirstWordJson.Contains('"status": "accepted_return_dependency_closed_view_derivation_v0"')) { throw "ownership check first_word expected closed derivation acceptance, got $OwnershipNFirstWordJson" }

  $OwnershipNLocalSliceJson = Read-NativeOutputWithExit 'ownership check Session N local sub-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_n_return_view_local_slice_fail.hum')
  if ($OwnershipNLocalSliceJson.ExitCode -ne 1) { throw "ownership check local sub-view expected exit 1, got $($OwnershipNLocalSliceJson.ExitCode)" }
  Assert-Json 'ownership check Session N local sub-view JSON' $OwnershipNLocalSliceJson.Output
  if (-not $OwnershipNLocalSliceJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check local sub-view expected H0805, got $($OwnershipNLocalSliceJson.Output)" }
  if (-not $OwnershipNLocalSliceJson.Output.Contains('"status": "rejected_return_dependency_local_v0"')) { throw "ownership check local sub-view expected local rejection, got $($OwnershipNLocalSliceJson.Output)" }

  $OwnershipNLostProvenanceJson = Read-NativeOutputWithExit 'ownership check Session N lost-provenance JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_n_return_view_lost_provenance_fail.hum')
  if ($OwnershipNLostProvenanceJson.ExitCode -ne 1) { throw "ownership check lost-provenance expected exit 1, got $($OwnershipNLostProvenanceJson.ExitCode)" }
  Assert-Json 'ownership check Session N lost-provenance JSON' $OwnershipNLostProvenanceJson.Output
  if (-not $OwnershipNLostProvenanceJson.Output.Contains('"diagnostic_code": "H0805"')) { throw "ownership check lost-provenance expected H0805, got $($OwnershipNLostProvenanceJson.Output)" }
  if (-not $OwnershipNLostProvenanceJson.Output.Contains('"reason": "returned_view_expression_not_closed_view_derivation_v0"')) { throw "ownership check lost-provenance expected non-closed reason, got $($OwnershipNLostProvenanceJson.Output)" }
  $OwnershipOFieldPlacesJson = Read-NativeOutput 'ownership check Session O field places JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/field_places.hum')
  Assert-Json 'ownership check Session O field places JSON' $OwnershipOFieldPlacesJson
  if (-not $OwnershipOFieldPlacesJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check field places expected pass, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"status": "accepted_disjoint_field_mutation_v0"')) { throw "ownership check field places expected disjoint field mutation acceptance, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"target": "point.x"')) { throw "ownership check field places expected point.x target, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"target": "point.y"')) { throw "ownership check field places expected point.y target, got $OwnershipOFieldPlacesJson" }
  if (-not $OwnershipOFieldPlacesJson.Contains('"target": "item.done"')) { throw "ownership check field places expected item.done target, got $OwnershipOFieldPlacesJson" }

  $OwnershipOBorrowFieldJson = Read-NativeOutputWithExit 'ownership check Session O borrowed field-write JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_o_field_write_borrow_fail.hum')
  if ($OwnershipOBorrowFieldJson.ExitCode -ne 1) { throw "ownership check borrowed field-write expected exit 1, got $($OwnershipOBorrowFieldJson.ExitCode)" }
  Assert-Json 'ownership check Session O borrowed field-write JSON' $OwnershipOBorrowFieldJson.Output
  if (-not $OwnershipOBorrowFieldJson.Output.Contains('"diagnostic_code": "H0802"')) { throw "ownership check borrowed field-write expected H0802, got $($OwnershipOBorrowFieldJson.Output)" }
  if (-not $OwnershipOBorrowFieldJson.Output.Contains('"target": "point.x"')) { throw "ownership check borrowed field-write expected point.x target, got $($OwnershipOBorrowFieldJson.Output)" }
  if (-not $OwnershipOBorrowFieldJson.Output.Contains('writes through borrowed parameter')) { throw "ownership check borrowed field-write expected field blame help, got $($OwnershipOBorrowFieldJson.Output)" }

  $OwnershipRFieldViewsJson = Read-NativeOutput 'ownership check Session R field views JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/field_views.hum')
  Assert-Json 'ownership check Session R field views JSON' $OwnershipRFieldViewsJson
  if (-not $OwnershipRFieldViewsJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check Session R field views expected pass, got $OwnershipRFieldViewsJson" }
  if (-not $OwnershipRFieldViewsJson.Contains('"status": "accepted_field_view_borrow_v0"')) { throw "ownership check Session R field views expected field-view borrow acceptance, got $OwnershipRFieldViewsJson" }
  if (-not $OwnershipRFieldViewsJson.Contains('"declaration": "borrow flags.right"')) { throw "ownership check Session R field views expected flags.right declaration, got $OwnershipRFieldViewsJson" }
  if (-not $OwnershipRFieldViewsJson.Contains('"status": "accepted_disjoint_field_mutation_v0"')) { throw "ownership check Session R field views expected disjoint field mutation, got $OwnershipRFieldViewsJson" }

  $OwnershipRStalePointJson = Read-NativeOutputWithExit 'ownership check Session R stale point field-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_r_stale_point_field_view_fail.hum')
  if ($OwnershipRStalePointJson.ExitCode -ne 1) { throw "ownership check Session R stale point field-view expected exit 1, got $($OwnershipRStalePointJson.ExitCode)" }
  Assert-Json 'ownership check Session R stale point field-view JSON' $OwnershipRStalePointJson.Output
  if (-not $OwnershipRStalePointJson.Output.Contains('"diagnostic_code": "H0807"')) { throw "ownership check Session R stale point expected H0807, got $($OwnershipRStalePointJson.Output)" }
  if (-not $OwnershipRStalePointJson.Output.Contains('"status": "rejected_stale_field_view_use_v0"')) { throw "ownership check Session R stale point expected stale view rejection, got $($OwnershipRStalePointJson.Output)" }
  if (-not $OwnershipRStalePointJson.Output.Contains('x_view borrowed point.x')) { throw "ownership check Session R stale point expected binding and source in help, got $($OwnershipRStalePointJson.Output)" }
  if (-not $OwnershipRStalePointJson.Output.Contains('point.x was written')) { throw "ownership check Session R stale point expected invalidating write in help, got $($OwnershipRStalePointJson.Output)" }

  $OwnershipRStaleItemJson = Read-NativeOutputWithExit 'ownership check Session R stale item field-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_r_stale_item_field_view_fail.hum')
  if ($OwnershipRStaleItemJson.ExitCode -ne 1) { throw "ownership check Session R stale item field-view expected exit 1, got $($OwnershipRStaleItemJson.ExitCode)" }
  Assert-Json 'ownership check Session R stale item field-view JSON' $OwnershipRStaleItemJson.Output
  if (-not $OwnershipRStaleItemJson.Output.Contains('"diagnostic_code": "H0807"')) { throw "ownership check Session R stale item expected H0807, got $($OwnershipRStaleItemJson.Output)" }
  if (-not $OwnershipRStaleItemJson.Output.Contains('done_view borrowed item.done')) { throw "ownership check Session R stale item expected binding and source in help, got $($OwnershipRStaleItemJson.Output)" }
  if (-not $OwnershipRStaleItemJson.Output.Contains('item.done was written')) { throw "ownership check Session R stale item expected invalidating write in help, got $($OwnershipRStaleItemJson.Output)" }

  $OwnershipSElementViewsJson = Read-NativeOutput 'ownership check Session S element views JSON' $Hum @('ownership-check', '--format', 'json', 'examples/probes/element_views.hum')
  Assert-Json 'ownership check Session S element views JSON' $OwnershipSElementViewsJson
  if (-not $OwnershipSElementViewsJson.Contains('"status": "recognized_core_ownership_facts_checked_v0"')) { throw "ownership check Session S element views expected pass, got $OwnershipSElementViewsJson" }
  if (-not $OwnershipSElementViewsJson.Contains('"status": "accepted_element_view_borrow_v0"')) { throw "ownership check Session S element views expected element-view borrow acceptance, got $OwnershipSElementViewsJson" }
  if (-not $OwnershipSElementViewsJson.Contains('"declaration": "borrow items[0]"')) { throw "ownership check Session S element views expected items[0] declaration, got $OwnershipSElementViewsJson" }

  $OwnershipSStaleElementJson = Read-NativeOutputWithExit 'ownership check Session S stale element-view JSON' $Hum @('ownership-check', '--format', 'json', 'fixtures/ownership_check/session_s_stale_element_view_fail.hum')
  if ($OwnershipSStaleElementJson.ExitCode -ne 1) { throw "ownership check Session S stale element-view expected exit 1, got $($OwnershipSStaleElementJson.ExitCode)" }
  Assert-Json 'ownership check Session S stale element-view JSON' $OwnershipSStaleElementJson.Output
  if (-not $OwnershipSStaleElementJson.Output.Contains('"diagnostic_code": "H0807"')) { throw "ownership check Session S stale element expected H0807, got $($OwnershipSStaleElementJson.Output)" }
  if (-not $OwnershipSStaleElementJson.Output.Contains('"status": "rejected_stale_element_view_use_v0"')) { throw "ownership check Session S stale element expected stale element rejection, got $($OwnershipSStaleElementJson.Output)" }
  if (-not $OwnershipSStaleElementJson.Output.Contains('first_view borrowed items[0]')) { throw "ownership check Session S stale element expected binding and source in help, got $($OwnershipSStaleElementJson.Output)" }
  if (-not $OwnershipSStaleElementJson.Output.Contains('list_append grew items')) { throw "ownership check Session S stale element expected append site in help, got $($OwnershipSStaleElementJson.Output)" }
  $ResourceCheckJson = Read-NativeOutput 'resource check JSON' $Hum @('resource-check', '--format', 'json', 'fixtures/resource_check/simple_pass.hum')
  Assert-Json 'resource check JSON' $ResourceCheckJson
  if (-not $ResourceCheckJson.Contains('"schema": "hum.resource_check.v0"')) { throw 'resource check JSON is missing hum.resource_check.v0 schema' }
  if (-not $ResourceCheckJson.Contains('"status": "recognized_core_resources_checked_v0"')) { throw 'resource check JSON should pass for simple fixture' }
  if (-not $ResourceCheckJson.Contains('"mode": "recognized_core_resource_gate_v0"')) { throw 'resource check JSON is missing resource gate mode' }
  if (-not $ResourceCheckJson.Contains('"ownership_check_schema": "hum.ownership_check.v0"')) { throw 'resource check JSON is missing ownership-check dependency schema' }
  if (-not $ResourceCheckJson.Contains('"resource_report_schema": "hum.resource_report.v0"')) { throw 'resource check JSON is missing resource-report dependency schema' }
  if (-not $ResourceCheckJson.Contains('"blocking_issues": 0')) { throw 'resource check JSON should have zero blocking issues for simple fixture' }
  if (-not $ResourceCheckJson.Contains('"accepted_conservative_allocation_free_claim_v0"')) { throw 'resource check JSON is missing accepted allocation-free fact' }
  if (-not $ResourceCheckJson.Contains('"proof_ready": 0')) { throw 'resource check JSON must not claim proof readiness' }
  if (-not $ResourceCheckJson.Contains('"execution_ready": 0')) { throw 'resource check JSON must not claim execution readiness' }
  if (-not $ResourceCheckJson.Contains('"ir_ready": 0')) { throw 'resource check JSON must not claim IR readiness' }
  if (-not $ResourceCheckJson.Contains('no allocation-freedom proof')) { throw 'resource check JSON must keep allocation-proof non-claim' }
  if (-not $ResourceCheckJson.Contains('no memory-safety proof')) { throw 'resource check JSON must keep memory-safety non-claim' }

  $ProfileCheckJson = Read-NativeOutput 'profile check JSON' $Hum @('profile-check', '--format', 'json', 'fixtures/profile_check/simple_pass.hum')
  Assert-Json 'profile check JSON' $ProfileCheckJson
  if (-not $ProfileCheckJson.Contains('"schema": "hum.profile_check.v0"')) { throw 'profile check JSON is missing hum.profile_check.v0 schema' }
  if (-not $ProfileCheckJson.Contains('"status": "recognized_profile_policy_checked_v0"')) { throw 'profile check JSON should pass for simple fixture' }
  if (-not $ProfileCheckJson.Contains('"mode": "recognized_profile_policy_gate_v0"')) { throw 'profile check JSON is missing profile policy gate mode' }
  if (-not $ProfileCheckJson.Contains('"resource_check_schema": "hum.resource_check.v0"')) { throw 'profile check JSON is missing resource-check dependency schema' }
  if (-not $ProfileCheckJson.Contains('"runtime_profiles_schema": "hum.runtime_profiles.v0"')) { throw 'profile check JSON is missing runtime profiles dependency schema' }
  if (-not $ProfileCheckJson.Contains('"runtime_profile_mode": "contract_only_no_profile_enforcement"')) { throw 'profile check JSON must keep runtime profile catalog mode' }
  if (-not $ProfileCheckJson.Contains('"profile_id": "normal"')) { throw 'profile check JSON is missing accepted normal profile id' }
  if (-not $ProfileCheckJson.Contains('"accepted_normal_profile_policy_v0"')) { throw 'profile check JSON is missing accepted normal profile check' }
  if (-not $ProfileCheckJson.Contains('"blocking_issues": 0')) { throw 'profile check JSON should have zero blocking issues for simple fixture' }
  if (-not $ProfileCheckJson.Contains('"proof_ready": 0')) { throw 'profile check JSON must not claim proof readiness' }
  if (-not $ProfileCheckJson.Contains('"execution_ready": 0')) { throw 'profile check JSON must not claim execution readiness' }
  if (-not $ProfileCheckJson.Contains('"ir_ready": 0')) { throw 'profile check JSON must not claim IR readiness' }
  if (-not $ProfileCheckJson.Contains('no profile enforcement')) { throw 'profile check JSON must keep profile-enforcement non-claim' }
  if (-not $ProfileCheckJson.Contains('no certification claim')) { throw 'profile check JSON must keep certification non-claim' }

  $IrReadinessJson = Read-NativeOutput 'IR readiness JSON' $Hum @('ir-readiness', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'IR readiness JSON' $IrReadinessJson
  if (-not $IrReadinessJson.Contains('"schema": "hum.ir_readiness.v0"')) { throw 'IR readiness JSON is missing hum.ir_readiness.v0 schema' }
  if (-not $IrReadinessJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'IR readiness JSON is missing Core Hum contract schema' }
  if (-not $IrReadinessJson.Contains('"ir_contract_schema": "hum.ir_contract.v0"')) { throw 'IR readiness JSON is missing Hum IR contract schema' }
  if (-not $IrReadinessJson.Contains('"resolver"')) { throw 'IR readiness JSON is missing resolver summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.resolve.v0"')) { throw 'IR readiness JSON is missing hum.resolve.v0 resolver schema' }
  if (-not $IrReadinessJson.Contains('"status": "checked_resolver_v0"')) { throw 'IR readiness JSON should include checked resolver status for reference fixture' }
  if (-not $IrReadinessJson.Contains('"mode": "source_analysis_only_no_type_or_borrow_check"')) { throw 'IR readiness JSON is missing resolver source-analysis mode' }
  if (-not $IrReadinessJson.Contains('"resolver_errors": 0')) { throw 'IR readiness JSON should have zero resolver errors for reference fixture' }
  if (-not $IrReadinessJson.Contains('"type_check"')) { throw 'IR readiness JSON is missing type_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.type_check.v0"')) { throw 'IR readiness JSON is missing hum.type_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"core_preview"')) { throw 'IR readiness JSON is missing core_preview summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.core_preview.v0"')) { throw 'IR readiness JSON is missing hum.core_preview.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"core_lower"')) { throw 'IR readiness JSON is missing core_lower summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.core_lower.v0"')) { throw 'IR readiness JSON is missing hum.core_lower.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"core_verify"')) { throw 'IR readiness JSON is missing core_verify summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.core_verify.v0"')) { throw 'IR readiness JSON is missing hum.core_verify.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"full_type_check"')) { throw 'IR readiness JSON is missing full_type_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.full_type_check.v0"')) { throw 'IR readiness JSON is missing hum.full_type_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"effect_check"')) { throw 'IR readiness JSON is missing effect_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.effect_check.v0"')) { throw 'IR readiness JSON is missing hum.effect_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"ownership_check"')) { throw 'IR readiness JSON is missing ownership_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.ownership_check.v0"')) { throw 'IR readiness JSON is missing hum.ownership_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"resource_check"')) { throw 'IR readiness JSON is missing resource_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.resource_check.v0"')) { throw 'IR readiness JSON is missing hum.resource_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"mode": "recognized_core_resource_gate_v0"')) { throw 'IR readiness JSON is missing resource-check gate mode' }
  if (-not $IrReadinessJson.Contains('"profile_check"')) { throw 'IR readiness JSON is missing profile_check summary' }
  if (-not $IrReadinessJson.Contains('"schema": "hum.profile_check.v0"')) { throw 'IR readiness JSON is missing hum.profile_check.v0 schema link' }
  if (-not $IrReadinessJson.Contains('"mode": "recognized_profile_policy_gate_v0"')) { throw 'IR readiness JSON is missing profile-check gate mode' }
  if (-not $IrReadinessJson.Contains('"mode": "non_executing_artifact_invariant_check_v0"')) { throw 'IR readiness JSON is missing core verify mode' }
  if (-not $IrReadinessJson.Contains('"status": "unverified_core_artifact_v0"')) { throw 'IR readiness JSON is missing unverified core lower status' }
  if (-not $IrReadinessJson.Contains('"typed_expression_previews": 1')) { throw 'IR readiness JSON is missing core preview typed expression count' }
  if (-not $IrReadinessJson.Contains('"status": "declaration_annotations_and_trivial_returns_checked_v0"')) { throw 'IR readiness JSON should include clean type-check status for reference fixture' }
  if (-not $IrReadinessJson.Contains('"type_errors": 0')) { throw 'IR readiness JSON should have zero type errors for reference fixture' }
  if (-not $IrReadinessJson.Contains('"unknown_type_references": 0')) { throw 'IR readiness JSON should have zero unknown type refs for reference fixture' }
  if (-not $IrReadinessJson.Contains('"checked_returns"')) { throw 'IR readiness JSON is missing type-check return counters' }
  if (-not $IrReadinessJson.Contains('"trivial_return_checks_v0"')) { throw 'IR readiness JSON is missing trivial return fact' }
  if (-not $IrReadinessJson.Contains('"type_check_summary_v0"')) { throw 'IR readiness JSON is missing type check summary fact' }
  if (-not $IrReadinessJson.Contains('"core_preview_summary_v0"')) { throw 'IR readiness JSON is missing core preview summary fact' }
  if (-not $IrReadinessJson.Contains('"core_lower_summary_v0"')) { throw 'IR readiness JSON is missing core lower summary fact' }
  if (-not $IrReadinessJson.Contains('"core_verify_summary_v0"')) { throw 'IR readiness JSON is missing core verify summary fact' }
  if (-not $IrReadinessJson.Contains('"full_type_check_summary_v0"')) { throw 'IR readiness JSON is missing full type check summary fact' }
  if (-not $IrReadinessJson.Contains('"effect_check_summary_v0"')) { throw 'IR readiness JSON is missing effect check summary fact' }
  if (-not $IrReadinessJson.Contains('"ownership_check_summary_v0"')) { throw 'IR readiness JSON is missing ownership check summary fact' }
  if (-not $IrReadinessJson.Contains('"resource_check_summary_v0"')) { throw 'IR readiness JSON is missing resource check summary fact' }
  if (-not $IrReadinessJson.Contains('"profile_check_summary_v0"')) { throw 'IR readiness JSON is missing profile check summary fact' }
  if (-not $IrReadinessJson.Contains('"unverified_core_artifact_rows_v0"')) { throw 'IR readiness JSON is missing unverified core artifact row fact' }
  if (-not $IrReadinessJson.Contains('"verified_core_artifact_rows_v0"')) { throw 'IR readiness JSON is missing verified core artifact row fact' }
  if (-not $IrReadinessJson.Contains('"checked_return_expression_type_slots_v0"')) { throw 'IR readiness JSON is missing checked return expression slot fact' }
  if (-not $IrReadinessJson.Contains('"name": "resolve"')) { throw 'IR readiness JSON is missing resolve pass status' }
  if (-not $IrReadinessJson.Contains('"checked_report_available"')) { throw 'IR readiness JSON is missing checked resolver pass availability' }
  if (-not $IrReadinessJson.Contains('"resolver_summary_v0"')) { throw 'IR readiness JSON is missing resolver summary fact' }
  if (-not $IrReadinessJson.Contains('"ready_for_ir": 0')) { throw 'IR readiness JSON must not claim IR readiness yet' }
  if (-not $IrReadinessJson.Contains('"body_grammar"')) { throw 'IR readiness JSON is missing body_grammar facts' }
  if (-not $IrReadinessJson.Contains('"name": "core_preview"')) { throw 'IR readiness JSON is missing core_preview pass status' }
  if (-not $IrReadinessJson.Contains('"body_grammar_partial_v0"')) { throw 'IR readiness JSON is missing partial body grammar fact' }
  if (-not $IrReadinessJson.Contains('"body_grammar_unsupported_lines"')) { throw 'IR readiness JSON is missing body grammar unsupported count' }
  if (-not $IrReadinessJson.Contains('"surface_save_requires_store_lowering"')) { throw 'IR readiness JSON is missing store save lowering blocker' }
  if (-not $IrReadinessJson.Contains('"core_lowering"')) { throw 'IR readiness JSON is missing core_lowering pass status' }
  if (-not $IrReadinessJson.Contains('"core_verify"')) { throw 'IR readiness JSON is missing core_verify pass status' }
  if (-not $IrReadinessJson.Contains('"blocked_by_full_type_check_errors"')) { throw 'IR readiness JSON is missing full type check blocker status' }
  if (-not $IrReadinessJson.Contains('"declaration_and_trivial_return_check_available"')) { throw 'IR readiness JSON is missing type-check pass availability' }
  if (-not $IrReadinessJson.Contains('"full_type_check"')) { throw 'IR readiness JSON is missing full_type_check blocker' }
  if (-not $IrReadinessJson.Contains('"full_type_check_errors"')) { throw 'IR readiness JSON is missing full type-check blocker reason' }
  if (-not $IrReadinessJson.Contains('"recognized_core_effect_gate_available_v0"')) { throw 'IR readiness JSON is missing effect-check pass availability' }
  if (-not $IrReadinessJson.Contains('"recognized_core_ownership_gate_available_v0"')) { throw 'IR readiness JSON is missing ownership-check pass availability' }
  if (-not $IrReadinessJson.Contains('"recognized_core_resource_gate_available_v0"')) { throw 'IR readiness JSON is missing resource-check pass availability' }
  if (-not $IrReadinessJson.Contains('"recognized_core_profile_gate_available_v0"')) { throw 'IR readiness JSON is missing profile-check pass availability' }
  if ($IrReadinessJson.Contains('"allocation_resource_check_not_implemented"')) { throw 'IR readiness JSON should not report resource check as not implemented' }
  if ($IrReadinessJson.Contains('"profile_check_not_implemented"')) { throw 'IR readiness JSON should not report profile check as not implemented' }
  if (-not $IrReadinessJson.Contains('"ir_verify_not_implemented"')) { throw 'IR readiness JSON is missing IR verifier blocker' }
  if (-not $IrReadinessJson.Contains('"not_implemented"')) { throw 'IR readiness JSON is missing not_implemented blockers' }
  if (-not $IrReadinessJson.Contains('"no IR emission"')) { throw 'IR readiness JSON must keep V0 non-emission claim' }

  $MathOutDir = Join-Path (Join-Path $RepoRoot 'target') ('hum-math-obligations-smoke-' + [System.Guid]::NewGuid().ToString('N'))
  Invoke-Native 'math obligations out-dir' $Hum @('math-obligations', '--out-dir', $MathOutDir, 'examples/control_flow.hum')
  $MathFiles = @(Get-ChildItem -LiteralPath $MathOutDir -Filter '*.json' -File)
  if ($MathFiles.Count -lt 1) { throw 'math obligations out-dir wrote no JSON files' }

  Invoke-Native 'reference fixture coverage smoke' $Hum @('test-skeletons', 'examples/reference_surface.hum')

  $GraphJson = Read-NativeOutput 'reference fixture graph JSON' $Hum @('graph', 'examples/reference_surface.hum')
  Assert-Json 'reference fixture graph JSON' $GraphJson
  $Graph = $GraphJson | ConvertFrom-Json
  if (-not $GraphJson.Contains('"folding_ranges"')) { throw 'reference fixture graph JSON is missing folding_ranges' }
  if (-not $GraphJson.Contains('"symbols"')) { throw 'reference fixture graph JSON is missing symbols' }
  if (-not $GraphJson.Contains('"portability"')) { throw 'reference fixture graph JSON is missing portability reservation' }
  if (-not $GraphJson.Contains('"status": "reserved_v0"')) { throw 'reference fixture graph JSON is missing reserved portability status' }
  if (-not $GraphJson.Contains('"mode": "source_analysis_only_no_target_selection"')) { throw 'reference fixture graph JSON must not select a target in V0' }
  if (-not $GraphJson.Contains('"target_facts_schema": "hum.target_facts.v0"')) { throw 'reference fixture graph JSON is missing target facts schema link' }
  if (-not $GraphJson.Contains('"target_fact_record_schema": "hum.target_fact_record.v0"')) { throw 'reference fixture graph JSON is missing target fact record schema link' }
  if (-not $GraphJson.Contains('"target_fact_records": ["wasm32-wasi-preview1"]')) { throw 'reference fixture graph JSON is missing source-declared target fact record' }
  if (-not $GraphJson.Contains('"required_capability_families": ["os.clock", "os.filesystem"]')) { throw 'reference fixture graph JSON is missing source-declared required capability families' }
  if (-not $GraphJson.Contains('"denied_capability_families": ["os.network"]')) { throw 'reference fixture graph JSON is missing source-declared denied capability family' }
  if (-not $GraphJson.Contains('"unavailable_capability_families": []')) { throw 'reference fixture graph JSON should have no unavailable capability families' }
  if (-not $GraphJson.Contains('"source_target_declarations"')) { throw 'reference fixture graph JSON is missing source target declarations' }
  if (-not $GraphJson.Contains('"status": "declared_not_enforced_v0"')) { throw 'reference fixture graph JSON must mark source target declarations unenforced' }
  if (-not $GraphJson.Contains('"source_section": "targets"')) { throw 'reference fixture graph JSON is missing targets source section links' }
  if (-not $GraphJson.Contains('"no target selected"')) { throw 'reference fixture graph JSON must keep portability non-claim' }
  Assert-ReferenceEvidenceCoverage $Graph

  $GraphParameterViewJson = Read-NativeOutput 'Session L parameter returned-view graph JSON' $Hum @('graph', 'fixtures/ownership_check/session_l_return_parameter_view_pass.hum')
  Assert-Json 'Session L parameter returned-view graph JSON' $GraphParameterViewJson
  if (-not $GraphParameterViewJson.Contains('"return_dependencies"')) { throw 'Session L graph JSON is missing return_dependencies' }
  if (-not $GraphParameterViewJson.Contains('"source": "text"')) { throw 'Session L graph JSON is missing returned-view source text' }
  if (-not $GraphParameterViewJson.Contains('"status": "declared_return_dependency_parameter_v0"')) { throw 'Session L graph JSON is missing returned-view parameter status' }

  $GraphFirstWordJson = Read-NativeOutput 'Session N first_word graph JSON' $Hum @('graph', 'examples/probes/first_word.hum')
  Assert-Json 'Session N first_word graph JSON' $GraphFirstWordJson
  if (-not $GraphFirstWordJson.Contains('"return_dependencies"')) { throw 'Session N graph JSON is missing return_dependencies' }
  if (-not $GraphFirstWordJson.Contains('"source": "text"')) { throw 'Session N graph JSON is missing returned-view source text' }
  if (-not $GraphFirstWordJson.Contains('"status": "declared_return_dependency_parameter_v0"')) { throw 'Session N graph JSON is missing returned-view parameter status' }
  if (-not $GraphFirstWordJson.Contains('slice_until(text, \" \"')) { throw 'Session N graph JSON is missing slice_until body evidence' }
  Invoke-RepoScript 'editor fixture recovery' 'check_editor_fixtures.ps1'

  $SyntaxJson = Read-NativeOutput 'syntax surface JSON' $Hum @('syntax')
  Assert-Json 'syntax surface JSON' $SyntaxJson
  if (-not $SyntaxJson.Contains('"section_catalog"')) { throw 'syntax surface JSON is missing section_catalog' }
  if (-not $SyntaxJson.Contains('"targets"')) { throw 'syntax surface JSON is missing targets section' }
  if (-not $SyntaxJson.Contains('"hover"')) { throw 'syntax surface JSON is missing hover metadata' }
  if (-not $SyntaxJson.Contains('"semantic_tokens"')) { throw 'syntax surface JSON is missing semantic_tokens' }
  if (-not $SyntaxJson.Contains('"token_types"')) { throw 'syntax surface JSON is missing semantic token types' }
  if (-not $SyntaxJson.Contains('"parameter_permission_modes": ["borrow", "change", "consume"]')) { throw 'syntax surface JSON is missing parameter permission modes' }

  $TextMateJson = Read-NativeOutput 'TextMate grammar JSON' $Hum @('syntax', '--format', 'textmate')
  Assert-Json 'TextMate grammar JSON' $TextMateJson
  Assert-TextMateSnapshot $TextMateJson
  Assert-ReadmeHumExamplesMatch
  Assert-SessionASurfaceRules

  Invoke-Native 'git diff --check' $Git @('diff', '--check')
  Invoke-Native 'git diff --cached --check' $Git @('diff', '--cached', '--check')

  $DecisionIndex = [System.IO.File]::ReadAllText((Join-Path (Join-Path $RepoRoot 'docs') 'decisions\README.md'))
  if (-not $DecisionIndex.Contains('0009-adopt-formal-readability-not-english-mimicry.md')) { throw 'decision index is missing formal readability ADR' }
  if (-not $DecisionIndex.Contains('0010-adopt-explicit-state-model.md')) { throw 'decision index is missing explicit state model ADR' }
  if (-not $DecisionIndex.Contains('0011-add-checked-resolver-before-execution.md')) { throw 'decision index is missing checked resolver ADR' }
  $ArchitectureText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\ARCHITECTURE.md'))
  if (-not $ArchitectureText.Contains('Formal-readability doctrine')) { throw 'architecture is missing formal-readability doctrine' }
  if (-not $ArchitectureText.Contains('Debuggability doctrine')) { throw 'architecture is missing debuggability doctrine' }
  if (-not $ArchitectureText.Contains('State-management doctrine')) { throw 'architecture is missing state-management doctrine' }
  if (-not $ArchitectureText.Contains('Resolution doctrine')) { throw 'architecture is missing resolution doctrine' }
  if (-not $ArchitectureText.Contains('STATE_MODEL.md')) { throw 'architecture is missing state model link' }
  if (-not $ArchitectureText.Contains('HUM_RESOLVE_SCHEMA.md')) { throw 'architecture is missing resolve schema link' }
  if (-not $ArchitectureText.Contains('HUM_CORE_LOWER_SCHEMA.md')) { throw 'architecture is missing core lower schema link' }
  if (-not $ArchitectureText.Contains('HUM_CORE_VERIFY_SCHEMA.md')) { throw 'architecture is missing core verify schema link' }
  if (-not $ArchitectureText.Contains('HUM_RESOURCE_CHECK_SCHEMA.md')) { throw 'architecture is missing resource check schema link' }
  if (-not $ArchitectureText.Contains('HUM_PROFILE_CHECK_SCHEMA.md')) { throw 'architecture is missing profile check schema link' }
  if (-not $ArchitectureText.Contains('recognized_core_resource_gate_available_v0')) { throw 'architecture is missing current resource-check gate' }
  if (-not $ArchitectureText.Contains('recognized_core_profile_gate_available_v0')) { throw 'architecture is missing current profile-check gate' }
  if (-not $ArchitectureText.Contains('PORTABILITY_BOUNDARY_MODEL.md')) { throw 'architecture is missing portability boundary model link' }
  $LanguageReferenceText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\LANGUAGE_REFERENCE.md'))
  if (-not $LanguageReferenceText.Contains('traditional language reference spine')) { throw 'language reference is missing reference spine marker' }
  if (-not $LanguageReferenceText.Contains('PORTABILITY_BOUNDARY_MODEL.md')) { throw 'language reference is missing portability boundary link' }
  if (-not $LanguageReferenceText.Contains('STATE_MODEL.md')) { throw 'language reference is missing state model link' }
  if (-not $LanguageReferenceText.Contains('HUM_RESOLVE_SCHEMA.md')) { throw 'language reference is missing resolve schema link' }
  if (-not $LanguageReferenceText.Contains('HUM_RESOURCE_CHECK_SCHEMA.md')) { throw 'language reference is missing resource check schema link' }
  if (-not $LanguageReferenceText.Contains('HUM_PROFILE_CHECK_SCHEMA.md')) { throw 'language reference is missing profile check schema link' }
  if (-not $LanguageReferenceText.Contains('hum resource-check --format json')) { throw 'language reference is missing resource-check command' }
  if (-not $LanguageReferenceText.Contains('hum profile-check --format json')) { throw 'language reference is missing profile-check command' }
  if (-not $LanguageReferenceText.Contains('hum state-model --format json')) { throw 'language reference is missing state-model command' }
  if (-not $LanguageReferenceText.Contains('hum resolve --format json')) { throw 'language reference is missing resolve command' }
  if (-not $LanguageReferenceText.Contains('H1205')) { throw 'language reference is missing target declaration diagnostics' }
  $PortabilityBoundaryText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\PORTABILITY_BOUNDARY_MODEL.md'))
  if (-not $PortabilityBoundaryText.Contains('Hum Portability Boundary Model')) { throw 'portability boundary model is missing title' }
  if (-not $PortabilityBoundaryText.Contains('Absence Is A First-Class Case')) { throw 'portability boundary model is missing absence rule' }
  if (-not $PortabilityBoundaryText.Contains('Artifact Evidence')) { throw 'portability boundary model is missing artifact evidence rule' }
  if (-not $PortabilityBoundaryText.Contains('TARGET_FACTS_SCHEMA.md')) { throw 'portability boundary model is missing target facts schema link' }
  $SemanticGraphSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\SEMANTIC_GRAPH_SCHEMA.md'))
  if (-not $SemanticGraphSchemaText.Contains('"portability": {}')) { throw 'semantic graph schema doc is missing portability top-level shape' }
  if (-not $SemanticGraphSchemaText.Contains('source_analysis_only_no_target_selection')) { throw 'semantic graph schema doc is missing source-analysis portability mode' }
  if (-not $SemanticGraphSchemaText.Contains('target_facts_schema')) { throw 'semantic graph schema doc is missing target_facts_schema field' }
  if (-not $SemanticGraphSchemaText.Contains('target_fact_record_schema')) { throw 'semantic graph schema doc is missing target_fact_record_schema field' }
  if (-not $SemanticGraphSchemaText.Contains('reserved_v0')) { throw 'semantic graph schema doc is missing reserved portability status' }
  if (-not $SemanticGraphSchemaText.Contains('declared_not_enforced_v0')) { throw 'semantic graph schema doc is missing source target declaration status' }
  if (-not $SemanticGraphSchemaText.Contains('unavailable_capability_families')) { throw 'semantic graph schema doc is missing unavailable capability families' }
  $TargetFactsSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\TARGET_FACTS_SCHEMA.md'))
  if (-not $TargetFactsSchemaText.Contains('hum.target_facts.v0')) { throw 'target facts schema doc is missing hum.target_facts.v0' }
  if (-not $TargetFactsSchemaText.Contains('hum.target_fact_record.v0')) { throw 'target facts schema doc is missing hum.target_fact_record.v0' }
  if (-not $TargetFactsSchemaText.Contains('contract_only_no_host_probe')) { throw 'target facts schema doc is missing no-probe mode' }
  if (-not $TargetFactsSchemaText.Contains('unknown_fails_closed')) { throw 'target facts schema doc is missing fail-closed policy' }
  if (-not $TargetFactsSchemaText.Contains('../fixtures/target_facts')) { throw 'target facts schema doc is missing fixture link' }
  if (-not $TargetFactsSchemaText.Contains('Semantic Graph Link')) { throw 'target facts schema doc is missing semantic graph link section' }
  if (-not $TargetFactsSchemaText.Contains('targets:')) { throw 'target facts schema doc is missing targets section link' }
  if (-not $TargetFactsSchemaText.Contains('H1205')) { throw 'target facts schema doc is missing target declaration diagnostics' }
  $DebugDoctrineText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\DEBUGGABILITY_DOCTRINE.md'))
  if (-not $DebugDoctrineText.Contains('hum.debug_info.v0')) { throw 'debuggability doctrine is missing debug info schema direction' }
  if (-not $DebugDoctrineText.Contains('faster and clearer than adding `printf`')) { throw 'debuggability doctrine is missing debugger speed rule' }
  if (-not $DebugDoctrineText.Contains('type-attached visualizers')) { throw 'debuggability doctrine is missing type-attached visualizer rule' }
  if (-not $DebugDoctrineText.Contains('debug probe sites')) { throw 'debuggability doctrine is missing debug probe site rule' }
  if (-not $DebugDoctrineText.Contains('DEBUG_INFO_AND_VISUALIZER_MODEL.md')) { throw 'debuggability doctrine is missing debug info model link' }
  $DebugInfoModelText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\DEBUG_INFO_AND_VISUALIZER_MODEL.md'))
  if (-not $DebugInfoModelText.Contains('Target schema: `hum.debug_info.v0`')) { throw 'debug info model is missing target schema' }
  if (-not $DebugInfoModelText.Contains('many-to-many provenance')) { throw 'debug info model is missing source-map provenance rule' }
  if (-not $DebugInfoModelText.Contains('Probe sites unify')) { throw 'debug info model is missing probe-site model' }
  if (-not $DebugInfoModelText.Contains('Visualizers must be reversible')) { throw 'debug info model is missing reversible visualizer rule' }
  if (-not $DebugInfoModelText.Contains('Native DWARF and PDB are compatibility targets')) { throw 'debug info model is missing native debug bridge rule' }
  $ResearchMapText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\RESEARCH_MAP_2026.md'))
  if (-not $ResearchMapText.Contains('2026-07-07-rad-debugger-lessons.md')) { throw 'research map is missing RAD Debugger lessons' }
  if (-not $ResearchMapText.Contains('DEBUG_INFO_AND_VISUALIZER_MODEL.md')) { throw 'research map is missing debug info model gate' }
  if (-not $ResearchMapText.Contains('2026-07-07-bellard-systems-lessons.md')) { throw 'research map is missing Bellard systems lessons' }
  $BellardResearchText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\research\2026-07-07-bellard-systems-lessons.md'))
  if (-not $BellardResearchText.Contains('Bellard Test For Hum')) { throw 'Bellard research note is missing Bellard Test' }
  if (-not $BellardResearchText.Contains('footprint')) { throw 'Bellard research note is missing footprint pressure' }
  if (-not $BellardResearchText.Contains('deterministic artifacts')) { throw 'Bellard research note is missing deterministic artifact rule' }
  $RuntimeProfilesText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\RUNTIME_PROFILES.md'))
  if (-not $RuntimeProfilesText.Contains('footprint constrained')) { throw 'runtime profiles are missing footprint constrained profile' }
  if (-not $RuntimeProfilesText.Contains('hum.runtime_profiles.v0')) { throw 'runtime profiles doc is missing runtime profiles schema' }
  if (-not $RuntimeProfilesText.Contains('contract_only_no_profile_enforcement')) { throw 'runtime profiles doc is missing contract-only mode' }
  if (-not $RuntimeProfilesText.Contains('HUM_RUNTIME_PROFILES_SCHEMA.md')) { throw 'runtime profiles doc is missing schema link' }
  $RuntimeProfilesSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_RUNTIME_PROFILES_SCHEMA.md'))
  if (-not $RuntimeProfilesSchemaText.Contains('hum.runtime_profiles.v0')) { throw 'runtime profile schema doc is missing catalog schema' }
  if (-not $RuntimeProfilesSchemaText.Contains('hum.runtime_profile.v0')) { throw 'runtime profile schema doc is missing entry schema' }
  if (-not $RuntimeProfilesSchemaText.Contains('contract_only_no_profile_enforcement')) { throw 'runtime profile schema doc is missing contract-only mode' }
  if (-not $RuntimeProfilesSchemaText.Contains('HUM_PROFILE_CHECK_SCHEMA.md')) { throw 'runtime profile schema doc is missing profile check schema link' }
  $StateModelText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\STATE_MODEL.md'))
  if (-not $StateModelText.Contains('Hum State Model')) { throw 'state model doc is missing title' }
  if (-not $StateModelText.Contains('hum.state_model.v0')) { throw 'state model doc is missing state model schema' }
  if (-not $StateModelText.Contains('contract_only_partial_declared_mutation_check')) { throw 'state model doc is missing contract-only mode' }
  if (-not $StateModelText.Contains('No new Hum feature is allowed to hide state')) { throw 'state model doc is missing brutal state rule' }
  if (-not $StateModelText.Contains('hum resolve --format json')) { throw 'state model doc is missing resolver link' }
  $ResolveSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_RESOLVE_SCHEMA.md'))
  if (-not $ResolveSchemaText.Contains('hum.resolve.v0')) { throw 'resolve schema doc is missing hum.resolve.v0' }
  if (-not $ResolveSchemaText.Contains('source_analysis_only_no_type_or_borrow_check')) { throw 'resolve schema doc is missing source-analysis mode' }
  if (-not $ResolveSchemaText.Contains('H0604')) { throw 'resolve schema doc is missing resolver diagnostics' }
  if (-not $ResolveSchemaText.Contains('blocked_by_resolver_errors')) { throw 'resolve schema doc is missing IR readiness link' }
  $TypeEnvSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_TYPE_ENV_SCHEMA.md'))
  if (-not $TypeEnvSchemaText.Contains('hum.type_env.v0')) { throw 'type environment schema doc is missing hum.type_env.v0' }
  if (-not $TypeEnvSchemaText.Contains('declaration_inventory_no_type_check')) { throw 'type environment schema doc is missing no-type-check mode' }
  if (-not $TypeEnvSchemaText.Contains('unknown_type_name_v0')) { throw 'type environment schema doc is missing unknown type status' }
  $TypeCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_TYPE_CHECK_SCHEMA.md'))
  if (-not $TypeCheckSchemaText.Contains('hum.type_check.v0')) { throw 'type check schema doc is missing hum.type_check.v0' }
  if (-not $TypeCheckSchemaText.Contains('declaration_annotation_and_trivial_return_check_v0')) { throw 'type check schema doc is missing declaration-and-return mode' }
  if (-not $TypeCheckSchemaText.Contains('H0605')) { throw 'type check schema doc is missing H0605' }
  if (-not $TypeCheckSchemaText.Contains('H0606')) { throw 'type check schema doc is missing H0606' }
  if (-not $TypeCheckSchemaText.Contains('checked_returns')) { throw 'type check schema doc is missing checked_returns' }
  $IrReadinessSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_IR_READINESS_SCHEMA.md'))
  if (-not $IrReadinessSchemaText.Contains('hum.resolve.v0')) { throw 'IR readiness schema doc is missing resolver schema link' }
  if (-not $IrReadinessSchemaText.Contains('checked_resolver_errors')) { throw 'IR readiness schema doc is missing resolver blocker' }
  if (-not $IrReadinessSchemaText.Contains('hum.type_check.v0')) { throw 'IR readiness schema doc is missing type-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.core_preview.v0')) { throw 'IR readiness schema doc is missing core-preview schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.core_lower.v0')) { throw 'IR readiness schema doc is missing core-lower schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.core_verify.v0')) { throw 'IR readiness schema doc is missing core-verify schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.full_type_check.v0')) { throw 'IR readiness schema doc is missing full type-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.effect_check.v0')) { throw 'IR readiness schema doc is missing effect-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.ownership_check.v0')) { throw 'IR readiness schema doc is missing ownership-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.resource_check.v0')) { throw 'IR readiness schema doc is missing resource-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('hum.profile_check.v0')) { throw 'IR readiness schema doc is missing profile-check schema link' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_full_type_check_errors')) { throw 'IR readiness schema doc is missing full type-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_effect_check_errors')) { throw 'IR readiness schema doc is missing effect-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_ownership_check_errors')) { throw 'IR readiness schema doc is missing ownership-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_resource_check_errors')) { throw 'IR readiness schema doc is missing resource-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_profile_check_errors')) { throw 'IR readiness schema doc is missing profile-check blocker' }
  if (-not $IrReadinessSchemaText.Contains('blocked_before_ir_verify')) { throw 'IR readiness schema doc is missing before-IR-verifier blocker' }
  if (-not $IrReadinessSchemaText.Contains('resource_check_summary_v0')) { throw 'IR readiness schema doc is missing resource check summary fact' }
  if (-not $IrReadinessSchemaText.Contains('profile_check_summary_v0')) { throw 'IR readiness schema doc is missing profile check summary fact' }
  if (-not $IrReadinessSchemaText.Contains('recognized_core_resource_gate_available_v0')) { throw 'IR readiness schema doc is missing resource-check pass status' }
  if (-not $IrReadinessSchemaText.Contains('recognized_core_profile_gate_available_v0')) { throw 'IR readiness schema doc is missing profile-check pass status' }
  if ($IrReadinessSchemaText.Contains('allocation_resource_check_not_implemented')) { throw 'IR readiness schema doc should not call resource check not implemented' }
  if ($IrReadinessSchemaText.Contains('profile_check_not_implemented')) { throw 'IR readiness schema doc should not call profile check not implemented' }
  if (-not $IrReadinessSchemaText.Contains('unverified_core_artifact_rows_v0')) { throw 'IR readiness schema doc is missing unverified core artifact row fact' }
  if (-not $IrReadinessSchemaText.Contains('core_verify')) { throw 'IR readiness schema doc is missing core_verify pass' }
  if (-not $IrReadinessSchemaText.Contains('verified_core_artifact_rows_v0')) { throw 'IR readiness schema doc is missing verified core artifact row fact' }
  if (-not $IrReadinessSchemaText.Contains('checked_return_expression_type_slots_v0')) { throw 'IR readiness schema doc is missing checked return expression slot fact' }
  if (-not $IrReadinessSchemaText.Contains('blocked_by_type_errors')) { throw 'IR readiness schema doc is missing type-error blocker' }
  if (-not $IrReadinessSchemaText.Contains('declaration_and_trivial_return_check_available')) { throw 'IR readiness schema doc is missing type-check pass status' }
  $ArchitectureText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\ARCHITECTURE.md'))
  if (-not $ArchitectureText.Contains('Current Compiler Spine')) { throw 'architecture doc is missing current compiler spine' }
  if (-not $ArchitectureText.Contains('recognized_core_body_type_gate_available_v0')) { throw 'architecture doc is missing current full type-check gate' }
  $CoreContractSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_CORE_CONTRACT_SCHEMA.md'))
  if (-not $CoreContractSchemaText.Contains('hum core-lower')) { throw 'Core contract schema doc is missing core-lower command link' }
  if (-not $CoreContractSchemaText.Contains('unverified_core_artifact_v0')) { throw 'Core contract schema doc is missing unverified core artifact gate' }
  if (-not $CoreContractSchemaText.Contains('declaration_and_trivial_return_check_available')) { throw 'Core contract schema doc is missing narrow type-check gate' }
  if (-not $CoreContractSchemaText.Contains('full_type_check')) { throw 'Core contract schema doc is missing full type-check gate' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_body_type_gate_available_v0')) { throw 'Core contract schema doc is missing full type-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_effect_gate_available_v0')) { throw 'Core contract schema doc is missing effect-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_ownership_gate_available_v0')) { throw 'Core contract schema doc is missing ownership-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_resource_gate_available_v0')) { throw 'Core contract schema doc is missing resource-check gate status' }
  if (-not $CoreContractSchemaText.Contains('recognized_core_profile_gate_available_v0')) { throw 'Core contract schema doc is missing profile-check gate status' }
  if (-not $CoreContractSchemaText.Contains('hum resource-check')) { throw 'Core contract schema doc is missing resource-check command link' }
  if (-not $CoreContractSchemaText.Contains('hum profile-check')) { throw 'Core contract schema doc is missing profile-check command link' }
  $FullTypeCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_FULL_TYPE_CHECK_SCHEMA.md'))
  if (-not $FullTypeCheckSchemaText.Contains('hum.full_type_check.v0')) { throw 'full type check schema doc is missing hum.full_type_check.v0' }
  if (-not $FullTypeCheckSchemaText.Contains('recognized_core_body_type_gate_v0')) { throw 'full type check schema doc is missing gate mode' }
  if (-not $FullTypeCheckSchemaText.Contains('hum full-type-check')) { throw 'full type check schema doc is missing command' }
  if (-not $FullTypeCheckSchemaText.Contains('no executable semantics')) { throw 'full type check schema doc must keep non-execution claim' }
  $EffectCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_EFFECT_CHECK_SCHEMA.md'))
  if (-not $EffectCheckSchemaText.Contains('hum.effect_check.v0')) { throw 'effect check schema doc is missing hum.effect_check.v0' }
  if (-not $EffectCheckSchemaText.Contains('recognized_core_effect_gate_v0')) { throw 'effect check schema doc is missing gate mode' }
  if (-not $EffectCheckSchemaText.Contains('hum effect-check')) { throw 'effect check schema doc is missing command' }
  if (-not $EffectCheckSchemaText.Contains('no executable semantics')) { throw 'effect check schema doc must keep non-execution claim' }
  $OwnershipCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_OWNERSHIP_CHECK_SCHEMA.md'))
  if (-not $OwnershipCheckSchemaText.Contains('hum.ownership_check.v0')) { throw 'ownership check schema doc is missing hum.ownership_check.v0' }
  if (-not $OwnershipCheckSchemaText.Contains('recognized_core_ownership_gate_v0')) { throw 'ownership check schema doc is missing gate mode' }
  if (-not $OwnershipCheckSchemaText.Contains('hum ownership-check')) { throw 'ownership check schema doc is missing command' }
  if (-not $OwnershipCheckSchemaText.Contains('no executable semantics')) { throw 'ownership check schema doc must keep non-execution claim' }
  $ResourceCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_RESOURCE_CHECK_SCHEMA.md'))
  if (-not $ResourceCheckSchemaText.Contains('hum.resource_check.v0')) { throw 'resource check schema doc is missing hum.resource_check.v0' }
  if (-not $ResourceCheckSchemaText.Contains('recognized_core_resource_gate_v0')) { throw 'resource check schema doc is missing gate mode' }
  if (-not $ResourceCheckSchemaText.Contains('hum resource-check')) { throw 'resource check schema doc is missing command' }
  if (-not $ResourceCheckSchemaText.Contains('no executable semantics')) { throw 'resource check schema doc must keep non-execution claim' }
  if (-not $ResourceCheckSchemaText.Contains('no allocation-freedom proof')) { throw 'resource check schema doc must keep allocation-proof non-claim' }
  $ProfileCheckSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_PROFILE_CHECK_SCHEMA.md'))
  if (-not $ProfileCheckSchemaText.Contains('hum.profile_check.v0')) { throw 'profile check schema doc is missing hum.profile_check.v0' }
  if (-not $ProfileCheckSchemaText.Contains('recognized_profile_policy_gate_v0')) { throw 'profile check schema doc is missing gate mode' }
  if (-not $ProfileCheckSchemaText.Contains('hum profile-check')) { throw 'profile check schema doc is missing command' }
  if (-not $ProfileCheckSchemaText.Contains('blocked_by_unchecked_profile_policy_v0')) { throw 'profile check schema doc is missing strict profile blocker' }
  if (-not $ProfileCheckSchemaText.Contains('no profile enforcement')) { throw 'profile check schema doc must keep profile-enforcement non-claim' }
  if (-not $ProfileCheckSchemaText.Contains('no certification claim')) { throw 'profile check schema doc must keep certification non-claim' }
  $CoreLowerSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_CORE_LOWER_SCHEMA.md'))
  if (-not $CoreLowerSchemaText.Contains('hum.core_lower.v0')) { throw 'Core lower schema doc is missing hum.core_lower.v0' }
  if (-not $CoreLowerSchemaText.Contains('unverified_core_artifact_v0')) { throw 'Core lower schema doc is missing unverified artifact status' }
  if (-not $CoreLowerSchemaText.Contains('hum.core_verify.v0')) { throw 'Core lower schema doc is missing core verify sync link' }
  if (-not $CoreLowerSchemaText.Contains('no Hum IR emission')) { throw 'Core lower schema doc must keep non-IR-emission claim' }
  $CoreVerifySchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_CORE_VERIFY_SCHEMA.md'))
  if (-not $CoreVerifySchemaText.Contains('hum.core_verify.v0')) { throw 'Core verify schema doc is missing hum.core_verify.v0' }
  if (-not $CoreVerifySchemaText.Contains('non_executing_artifact_invariant_check_v0')) { throw 'Core verify schema doc is missing invariant-check mode' }
  if (-not $CoreVerifySchemaText.Contains('verified_non_executing_core_artifact_v0')) { throw 'Core verify schema doc is missing verification status' }
  if (-not $CoreVerifySchemaText.Contains('source_span_sane')) { throw 'Core verify schema doc is missing source span rule' }
  if (-not $CoreVerifySchemaText.Contains('no memory-safety proof')) { throw 'Core verify schema doc must keep memory-safety non-claim' }
  $ResolveDecisionText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\decisions\0011-add-checked-resolver-before-execution.md'))
  if (-not $ResolveDecisionText.Contains('checked resolver')) { throw 'checked resolver ADR is missing decision language' }
  $StateModelSchemaText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\HUM_STATE_MODEL_SCHEMA.md'))
  if (-not $StateModelSchemaText.Contains('hum.state_model.v0')) { throw 'state model schema doc is missing catalog schema' }
  if (-not $StateModelSchemaText.Contains('hum.state_permission.v0')) { throw 'state model schema doc is missing permission schema' }
  if (-not $StateModelSchemaText.Contains('contract_only_partial_declared_mutation_check')) { throw 'state model schema doc is missing contract-only mode' }
  $StateModelDecisionText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\decisions\0010-adopt-explicit-state-model.md'))
  if (-not $StateModelDecisionText.Contains('Hum adopts a source-visible state model')) { throw 'state model ADR is missing decision statement' }
  $OptimizationText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\OPTIMIZATION_AND_DSA_STRATEGY.md'))
  if (-not $OptimizationText.Contains('Bellard Constraint Rule')) { throw 'optimization strategy is missing Bellard constraint rule' }
  if (-not $ResearchMapText.Contains('2026-07-07-systems-legends-lessons.md')) { throw 'research map is missing systems legends lessons' }
  $LegendsResearchText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\research\2026-07-07-systems-legends-lessons.md'))
  if (-not $LegendsResearchText.Contains('Combined Legends Test For Hum')) { throw 'Systems legends research note is missing combined test' }
  if (-not $LegendsResearchText.Contains('traditional language reference')) { throw 'Systems legends research note is missing language reference consequence' }
  if (-not $LegendsResearchText.Contains('portability-boundary')) { throw 'Systems legends research note is missing portability boundary consequence' }
  if (-not $ResearchMapText.Contains('Systems Legends And Durable Taste')) { throw 'research map is missing systems legends cluster' }

  Invoke-RepoScript 'text hygiene' 'check_text_hygiene.ps1'
  Invoke-RepoScript 'public readiness' 'check_public_readiness.ps1'
  Invoke-RepoScript 'release readiness' 'check_release_readiness.ps1'

  Write-Host 'All Hum preflight checks passed.'
} finally {
  Pop-Location
}
