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

  $ExplainJson = Read-NativeOutput 'diagnostic explain JSON' $Hum @('explain', 'H0201', '--format', 'json')
  Assert-Json 'diagnostic explain JSON' $ExplainJson

  $DiagnosticsJson = Read-NativeOutput 'diagnostic catalog JSON' $Hum @('diagnostics', '--format', 'json')
  Assert-Json 'diagnostic catalog JSON' $DiagnosticsJson

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
  if (-not $CapabilitiesJson.Contains('"ir_readiness"')) { throw 'capabilities JSON is missing ir_readiness schema' }
  if (-not $CapabilitiesJson.Contains('"core_contract"')) { throw 'capabilities JSON is missing core_contract schema' }
  if (-not $CapabilitiesJson.Contains('"ir_contract"')) { throw 'capabilities JSON is missing ir_contract schema' }
  if (-not $CapabilitiesJson.Contains('"backend_contract"')) { throw 'capabilities JSON is missing backend_contract schema' }
  if (-not $CapabilitiesJson.Contains('"doctor"')) { throw 'capabilities JSON is missing doctor schema' }

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
  if (-not $CoreContractJson.Contains('"no executable semantics"')) { throw 'Core contract JSON must keep V0 non-execution claim' }

  $IrContractJson = Read-NativeOutput 'IR contract JSON' $Hum @('ir-contract', '--format', 'json')
  Assert-Json 'IR contract JSON' $IrContractJson
  if (-not $IrContractJson.Contains('"schema": "hum.ir_contract.v0"')) { throw 'IR contract JSON is missing hum.ir_contract.v0 schema' }
  if (-not $IrContractJson.Contains('"semantic_owner": "hum_ir"')) { throw 'IR contract JSON is missing semantic owner' }
  if (-not $IrContractJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'IR contract JSON is missing Core Hum contract schema' }
  if (-not $IrContractJson.Contains('"id": "core_hum"')) { throw 'IR contract JSON is missing core_hum layer' }
  if (-not $IrContractJson.Contains('"id": "hum_ir"')) { throw 'IR contract JSON is missing hum_ir layer' }
  if (-not $IrContractJson.Contains('"typed_failure_edges"')) { throw 'IR contract JSON is missing typed failure facts' }
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

  Invoke-Native 'hum check examples' $Hum @('check', 'examples')

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
  if (-not $CorePreviewJson.Contains('"type_status": "not_type_checked_v0"')) { throw 'Core preview JSON must keep expression AST type status unchecked' }
  if (-not $CorePreviewJson.Contains('"effect_status": "not_effect_checked_v0"')) { throw 'Core preview JSON must keep expression AST effect status unchecked' }
  if (-not $CorePreviewJson.Contains('"status": "compound_preview_v0"')) { throw 'Core preview JSON is missing compound expression preview status' }
  if (-not $CorePreviewJson.Contains('"kind": "path_or_field_read"')) { throw 'Core preview JSON is missing path or field read expression kind' }
  if (-not $CorePreviewJson.Contains('surface_save_requires_store_lowering')) { throw 'Core preview JSON is missing store save lowering blocker' }
  if (-not $CorePreviewJson.Contains('no executable semantics')) { throw 'Core preview JSON must keep V0 non-execution claim' }
  if (-not $CorePreviewJson.Contains('no module or global name resolution')) { throw 'Core preview JSON must keep V0 name-resolution non-goal' }
  if (-not $CorePreviewJson.Contains('no checked name resolution')) { throw 'Core preview JSON must keep V0 checked name-resolution non-goal' }
  $IrReadinessJson = Read-NativeOutput 'IR readiness JSON' $Hum @('ir-readiness', '--format', 'json', 'examples/reference_surface.hum')
  Assert-Json 'IR readiness JSON' $IrReadinessJson
  if (-not $IrReadinessJson.Contains('"schema": "hum.ir_readiness.v0"')) { throw 'IR readiness JSON is missing hum.ir_readiness.v0 schema' }
  if (-not $IrReadinessJson.Contains('"core_contract_schema": "hum.core_contract.v0"')) { throw 'IR readiness JSON is missing Core Hum contract schema' }
  if (-not $IrReadinessJson.Contains('"ir_contract_schema": "hum.ir_contract.v0"')) { throw 'IR readiness JSON is missing Hum IR contract schema' }
  if (-not $IrReadinessJson.Contains('"ready_for_ir": 0')) { throw 'IR readiness JSON must not claim IR readiness yet' }
  if (-not $IrReadinessJson.Contains('"body_grammar"')) { throw 'IR readiness JSON is missing body_grammar facts' }
  if (-not $IrReadinessJson.Contains('"body_grammar_partial_v0"')) { throw 'IR readiness JSON is missing partial body grammar fact' }
  if (-not $IrReadinessJson.Contains('"body_grammar_unsupported_lines"')) { throw 'IR readiness JSON is missing body grammar unsupported count' }
  if (-not $IrReadinessJson.Contains('"surface_save_requires_store_lowering"')) { throw 'IR readiness JSON is missing store save lowering blocker' }
  if (-not $IrReadinessJson.Contains('"core_lowering"')) { throw 'IR readiness JSON is missing core_lowering pass status' }
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
  Assert-ReferenceEvidenceCoverage $Graph

  Invoke-RepoScript 'editor fixture recovery' 'check_editor_fixtures.ps1'

  $SyntaxJson = Read-NativeOutput 'syntax surface JSON' $Hum @('syntax')
  Assert-Json 'syntax surface JSON' $SyntaxJson
  if (-not $SyntaxJson.Contains('"section_catalog"')) { throw 'syntax surface JSON is missing section_catalog' }
  if (-not $SyntaxJson.Contains('"hover"')) { throw 'syntax surface JSON is missing hover metadata' }
  if (-not $SyntaxJson.Contains('"semantic_tokens"')) { throw 'syntax surface JSON is missing semantic_tokens' }
  if (-not $SyntaxJson.Contains('"token_types"')) { throw 'syntax surface JSON is missing semantic token types' }

  $TextMateJson = Read-NativeOutput 'TextMate grammar JSON' $Hum @('syntax', '--format', 'textmate')
  Assert-Json 'TextMate grammar JSON' $TextMateJson
  Assert-TextMateSnapshot $TextMateJson

  Invoke-Native 'git diff --check' $Git @('diff', '--check')
  Invoke-Native 'git diff --cached --check' $Git @('diff', '--cached', '--check')

  $DecisionIndex = [System.IO.File]::ReadAllText((Join-Path (Join-Path $RepoRoot 'docs') 'decisions\README.md'))
  if (-not $DecisionIndex.Contains('0009-adopt-formal-readability-not-english-mimicry.md')) { throw 'decision index is missing formal readability ADR' }
  $ArchitectureText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\ARCHITECTURE.md'))
  if (-not $ArchitectureText.Contains('Formal-readability doctrine')) { throw 'architecture is missing formal-readability doctrine' }
  if (-not $ArchitectureText.Contains('Debuggability doctrine')) { throw 'architecture is missing debuggability doctrine' }
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
  $OptimizationText = [System.IO.File]::ReadAllText((Join-Path $RepoRoot 'docs\OPTIMIZATION_AND_DSA_STRATEGY.md'))
  if (-not $OptimizationText.Contains('Bellard Constraint Rule')) { throw 'optimization strategy is missing Bellard constraint rule' }

  Invoke-RepoScript 'text hygiene' 'check_text_hygiene.ps1'
  Invoke-RepoScript 'public readiness' 'check_public_readiness.ps1'
  Invoke-RepoScript 'release readiness' 'check_release_readiness.ps1'

  Write-Host 'All Hum preflight checks passed.'
} finally {
  Pop-Location
}
