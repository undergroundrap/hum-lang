$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
  param([string] $Message)
  $Failures.Add($Message)
}

function Read-RepoText {
  param([string] $RelativePath)

  $path = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $RelativePath))
  if (-not $path.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to read outside repo root: $path"
  }

  return [System.IO.File]::ReadAllText($path).Trim()
}

$version = Read-RepoText 'VERSION'
$semverPattern = '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$'
if (-not [regex]::IsMatch($version, $semverPattern)) {
  Add-Failure "VERSION is not valid SemVer: $version"
}

$cargoToml = Read-RepoText 'Cargo.toml'
$cargoVersionMatch = [regex]::Match($cargoToml, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $cargoVersionMatch.Success) {
  Add-Failure 'Cargo.toml does not declare package version'
} elseif ($cargoVersionMatch.Groups[1].Value -ne $version) {
  Add-Failure ("Cargo.toml version {0} does not match VERSION {1}" -f $cargoVersionMatch.Groups[1].Value, $version)
}

$releaseDoc = Read-RepoText 'docs/RELEASE_AND_VERSIONING.md'
$markdownTick = [string] [char] 96
$expectedVersionText = 'Current repo version: ' + $markdownTick + $version + $markdownTick
if (-not $releaseDoc.Contains($expectedVersionText)) {
  Add-Failure "docs/RELEASE_AND_VERSIONING.md does not mention current version $version"
}
if (-not $releaseDoc.Contains('hum version')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention hum version'
}
if (-not $releaseDoc.Contains('hum explain')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention hum explain'
}
if (-not $releaseDoc.Contains('hum diagnostics')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention hum diagnostics'
}
if (-not $releaseDoc.Contains('hum capabilities')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention hum capabilities'
}
if (-not $releaseDoc.Contains('hum lsp --capabilities')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention hum lsp --capabilities'
}
if (-not $releaseDoc.Contains('hum doctor')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention hum doctor'
}
if (-not $releaseDoc.Contains('check_clean_checkout.ps1')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention check_clean_checkout.ps1'
}
if (-not $releaseDoc.Contains('check_tag_readiness.ps1')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention check_tag_readiness.ps1'
}
if (-not $releaseDoc.Contains('CHANGELOG.md')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention CHANGELOG.md'
}
if (-not $releaseDoc.Contains('releases/v0.0.1.md')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention releases/v0.0.1.md'
}
if (-not $releaseDoc.Contains('RELEASE_MANIFEST_SCHEMA.md')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention RELEASE_MANIFEST_SCHEMA.md'
}
if (-not $releaseDoc.Contains('releases/v0.0.1.manifest.json')) {
  Add-Failure 'docs/RELEASE_AND_VERSIONING.md does not mention releases/v0.0.1.manifest.json'
}
$changelog = Read-RepoText 'CHANGELOG.md'
foreach ($required in @("## [$version]", 'Status: pre-alpha', '### Added', '### Known Risks', '### Verification', 'tools/check_tag_readiness.ps1')) {
  if (-not $changelog.Contains($required)) {
    Add-Failure "CHANGELOG.md does not mention $required"
  }
}

$releaseNotePath = "docs/releases/v$version.md"
$releaseNote = Read-RepoText $releaseNotePath
foreach ($required in @("Hum v$version Release Notes", 'Status: pre-alpha', 'Commit hash', 'Manifest:', 'v0.0.1.manifest.json', '## Highlights', '## Compatibility Notes', '## Known Risks', '## Verification Commands', 'tools/check_tag_readiness.ps1')) {
  if (-not $releaseNote.Contains($required)) {
    Add-Failure "$releaseNotePath does not mention $required"
  }
}

$manifestSchemaDoc = Read-RepoText 'docs/RELEASE_MANIFEST_SCHEMA.md'
foreach ($required in @('hum.release_manifest.v0', 'releases/v0.0.1.manifest.json', 'tools/check_tag_readiness.ps1', 'tag_created', 'remote_touched', 'public_package_published')) {
  if (-not $manifestSchemaDoc.Contains($required)) {
    Add-Failure "docs/RELEASE_MANIFEST_SCHEMA.md does not mention $required"
  }
}

$manifestPath = "docs/releases/v$version.manifest.json"
$manifestText = Read-RepoText $manifestPath
try {
  $manifest = $manifestText | ConvertFrom-Json
} catch {
  Add-Failure "$manifestPath is not valid JSON: $_"
  $manifest = $null
}
if ($null -ne $manifest) {
  if ($manifest.schema -ne 'hum.release_manifest.v0') { Add-Failure "$manifestPath schema is not hum.release_manifest.v0" }
  if ($manifest.version -ne $version) { Add-Failure "$manifestPath version does not match VERSION $version" }
  if ($manifest.tag -ne ("v$version")) { Add-Failure "$manifestPath tag does not match v$version" }
  if ($manifest.status -ne 'pre-alpha') { Add-Failure "$manifestPath status is not pre-alpha" }
  if ($manifest.source_artifacts.changelog -ne 'CHANGELOG.md') { Add-Failure "$manifestPath does not point at CHANGELOG.md" }
  if ($manifest.source_artifacts.release_notes -ne $releaseNotePath) { Add-Failure "$manifestPath does not point at $releaseNotePath" }
  foreach ($command in @('tools/check_all.ps1', 'tools/check_clean_checkout.ps1', 'tools/check_tag_readiness.ps1')) {
    $found = $false
    foreach ($entry in $manifest.verification) {
      if ($entry.command -eq $command -and $entry.required -eq $true) { $found = $true }
    }
    if (-not $found) { Add-Failure "$manifestPath verification does not require $command" }
  }
  if ($manifest.publishing.tag_created -ne $false) { Add-Failure "$manifestPath should say tag_created is false before tag creation" }
  if ($manifest.publishing.remote_touched -ne $false) { Add-Failure "$manifestPath should say remote_touched is false before publication" }
  if ($manifest.publishing.public_package_published -ne $false) { Add-Failure "$manifestPath should say public_package_published is false before publication" }
}
$readme = Read-RepoText 'README.md'
if (-not $readme.Contains('docs/RELEASE_AND_VERSIONING.md')) {
  Add-Failure 'README.md does not link docs/RELEASE_AND_VERSIONING.md'
}
if (-not $readme.Contains('CHANGELOG.md')) {
  Add-Failure 'README.md does not link CHANGELOG.md'
}
if (-not $readme.Contains('docs/releases/v0.0.1.md')) {
  Add-Failure 'README.md does not link docs/releases/v0.0.1.md'
}
if (-not $readme.Contains('docs/RELEASE_MANIFEST_SCHEMA.md')) {
  Add-Failure 'README.md does not link docs/RELEASE_MANIFEST_SCHEMA.md'
}
if (-not $readme.Contains('docs/releases/v0.0.1.manifest.json')) {
  Add-Failure 'README.md does not link docs/releases/v0.0.1.manifest.json'
}
if (-not $readme.Contains('docs/LSP_CAPABILITY_MATRIX.md')) {
  Add-Failure 'README.md does not link docs/LSP_CAPABILITY_MATRIX.md'
}
if (-not $readme.Contains('docs/SYNTAX_SURFACE_SCHEMA.md')) {
  Add-Failure 'README.md does not link docs/SYNTAX_SURFACE_SCHEMA.md'
}
if (-not $readme.Contains('docs/CAPABILITIES_SCHEMA.md')) {
  Add-Failure 'README.md does not link docs/CAPABILITIES_SCHEMA.md'
}
if (-not $readme.Contains('docs/LSP_CAPABILITIES_SCHEMA.md')) {
  Add-Failure 'README.md does not link docs/LSP_CAPABILITIES_SCHEMA.md'
}
if (-not $readme.Contains('docs/DOCTOR_SCHEMA.md')) {
  Add-Failure 'README.md does not link docs/DOCTOR_SCHEMA.md'
}
if (-not $readme.Contains('SECURITY.md')) {
  Add-Failure 'README.md does not link SECURITY.md'
}
if (-not $readme.Contains('hum version')) {
  Add-Failure 'README.md does not mention hum version'
}
if (-not $readme.Contains('hum explain')) {
  Add-Failure 'README.md does not mention hum explain'
}
if (-not $readme.Contains('hum diagnostics')) {
  Add-Failure 'README.md does not mention hum diagnostics'
}
if (-not $readme.Contains('hum capabilities')) {
  Add-Failure 'README.md does not mention hum capabilities'
}
if (-not $readme.Contains('hum lsp --capabilities')) {
  Add-Failure 'README.md does not mention hum lsp --capabilities'
}
if (-not $readme.Contains('hum doctor')) {
  Add-Failure 'README.md does not mention hum doctor'
}
if (-not $readme.Contains('check_clean_checkout.ps1')) {
  Add-Failure 'README.md does not mention check_clean_checkout.ps1'
}
if (-not $readme.Contains('check_tag_readiness.ps1')) {
  Add-Failure 'README.md does not mention check_tag_readiness.ps1'
}

$diagnosticsDoc = Read-RepoText 'docs/DIAGNOSTICS.md'
if (-not $diagnosticsDoc.Contains('hum explain')) {
  Add-Failure 'docs/DIAGNOSTICS.md does not mention hum explain'
}
if (-not $diagnosticsDoc.Contains('hum.diagnostic_explain.v0')) {
  Add-Failure 'docs/DIAGNOSTICS.md does not mention hum.diagnostic_explain.v0'
}
if (-not $diagnosticsDoc.Contains('hum diagnostics')) {
  Add-Failure 'docs/DIAGNOSTICS.md does not mention hum diagnostics'
}
if (-not $diagnosticsDoc.Contains('hum.diagnostic_catalog.v0')) {
  Add-Failure 'docs/DIAGNOSTICS.md does not mention hum.diagnostic_catalog.v0'
}
if (-not $diagnosticsDoc.Contains('hum check --format json')) {
  Add-Failure 'docs/DIAGNOSTICS.md does not document hum check --format json'
}
if (-not $diagnosticsDoc.Contains('hum.check.v0')) {
  Add-Failure 'docs/DIAGNOSTICS.md does not mention hum.check.v0'
}

$semanticGraphDoc = Read-RepoText 'docs/SEMANTIC_GRAPH_SCHEMA.md'
if (-not $semanticGraphDoc.Contains('source-derived handles')) {
  Add-Failure 'docs/SEMANTIC_GRAPH_SCHEMA.md does not describe source-derived node IDs'
}
if (-not $semanticGraphDoc.Contains('File `folding_ranges`')) {
  Add-Failure 'docs/SEMANTIC_GRAPH_SCHEMA.md does not describe graph folding ranges'
}
if (-not $semanticGraphDoc.Contains('File `symbols`')) {
  Add-Failure 'docs/SEMANTIC_GRAPH_SCHEMA.md does not describe graph symbols'
}

$editorFixturesDoc = Read-RepoText 'docs/EDITOR_FIXTURES.md'
if (-not $editorFixturesDoc.Contains('check_editor_fixtures.ps1')) {
  Add-Failure 'docs/EDITOR_FIXTURES.md does not mention check_editor_fixtures.ps1'
}

$editorStrategyDoc = Read-RepoText 'docs/EDITOR_AND_INTEGRATION_STRATEGY.md'
if (-not $editorStrategyDoc.Contains('section hover metadata')) {
  Add-Failure 'docs/EDITOR_AND_INTEGRATION_STRATEGY.md does not describe syntax section hover metadata'
}
if (-not $editorStrategyDoc.Contains('semantic-token legend')) {
  Add-Failure 'docs/EDITOR_AND_INTEGRATION_STRATEGY.md does not describe syntax semantic-token metadata'
}
if (-not $editorStrategyDoc.Contains('LSP_CAPABILITY_MATRIX.md')) {
  Add-Failure 'docs/EDITOR_AND_INTEGRATION_STRATEGY.md does not link docs/LSP_CAPABILITY_MATRIX.md'
}

$lspMatrixDoc = Read-RepoText 'docs/LSP_CAPABILITY_MATRIX.md'
foreach ($required in @('hum.capabilities.v0', 'hum.lsp_capabilities.v0', 'hum.check.v0', 'hum.semantic_graph.v0', 'hum.syntax_surface.v0', 'folding_ranges', 'section_catalog', 'semantic_tokens', 'VS Code', 'Visual Studio', 'JetBrains', 'Eclipse', 'Jupyter')) {
  if (-not $lspMatrixDoc.Contains($required)) {
    Add-Failure "docs/LSP_CAPABILITY_MATRIX.md does not mention $required"
  }
}

$syntaxSurfaceDoc = Read-RepoText 'docs/SYNTAX_SURFACE_SCHEMA.md'
foreach ($required in @('hum.syntax_surface.v0', 'section_catalog', 'semantic_tokens', 'TextMate', 'hum graph', 'hum check --format json')) {
  if (-not $syntaxSurfaceDoc.Contains($required)) {
    Add-Failure "docs/SYNTAX_SURFACE_SCHEMA.md does not mention $required"
  }
}

$capabilitiesDoc = Read-RepoText 'docs/CAPABILITIES_SCHEMA.md'
foreach ($required in @('hum.capabilities.v0', 'hum.lsp_capabilities.v0', 'hum.doctor.v0', 'hum capabilities --format json', 'hum lsp --capabilities --format json', 'hum doctor --format json', 'toolchain discovery', 'not Hum''s runtime authority model', 'editor_capabilities', 'hum.syntax_surface.v0')) {
  if (-not $capabilitiesDoc.Contains($required)) {
    Add-Failure "docs/CAPABILITIES_SCHEMA.md does not mention $required"
  }
}

$lspCapabilitiesDoc = Read-RepoText 'docs/LSP_CAPABILITIES_SCHEMA.md'
foreach ($required in @('hum.lsp_capabilities.v0', 'hum lsp --capabilities --format json', 'json_rpc_server', 'false', 'textDocument/publishDiagnostics', 'textDocument/documentSymbol')) {
  if (-not $lspCapabilitiesDoc.Contains($required)) {
    Add-Failure "docs/LSP_CAPABILITIES_SCHEMA.md does not mention $required"
  }
}

$doctorDoc = Read-RepoText 'docs/DOCTOR_SCHEMA.md'
foreach ($required in @('hum.doctor.v0', 'hum doctor --format json', 'current_directory', 'text_hygiene_policy', 'public_readiness_policy', 'preflight_script', 'clean_checkout_smoke', 'tag_readiness', 'tools/check_clean_checkout.ps1', 'tools/check_tag_readiness.ps1')) {
  if (-not $doctorDoc.Contains($required)) {
    Add-Failure "docs/DOCTOR_SCHEMA.md does not mention $required"
  }
}
$securityPolicy = Read-RepoText 'SECURITY.md'
foreach ($required in @('Supported Versions', 'How To Report', 'Security Claims')) {
  if (-not $securityPolicy.Contains($required)) {
    Add-Failure "SECURITY.md does not mention $required"
  }
}

$setup = Read-RepoText 'docs/SETUP.md'
foreach ($required in @('Visual Studio', 'Eclipse', 'Jupyter', 'VS Code', 'Cursor', 'PyCharm', 'hum doctor', 'check_clean_checkout.ps1', 'check_tag_readiness.ps1')) {
  if (-not $setup.Contains($required)) {
    Add-Failure "docs/SETUP.md does not mention $required"
  }
}

& (Join-Path $PSScriptRoot 'check_alpha_claims.ps1')
if (-not $?) {
  Add-Failure 'alpha claims check failed'
}

if ($Failures.Count -gt 0) {
  Write-Host 'Release readiness check failed:'
  foreach ($failure in $Failures) {
    Write-Host ("- {0}" -f $failure)
  }
  exit 1
}

Write-Host "Release readiness check passed for version $version."
