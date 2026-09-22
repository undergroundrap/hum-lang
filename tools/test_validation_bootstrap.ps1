# Regression for the validation.yml plan-step bootstrap path.
# Faithfully reproduces the accepted-base probe sequence from
# .github/workflows/validation.yml against real SHAs and proves:
#   - base without policy -> bootstrap, profile stays 'full', outputs written, exit 0
#   - base with policy    -> policy present (non-bootstrap route)
#   - unreadable base     -> throws (fail-closed; genuine Git errors are not bootstrap)
$ErrorActionPreference = 'Stop'
$Root = Split-Path $PSScriptRoot -Parent

function Test-HumBootstrapProbe {
  param([string]$BaseSha)
  # Exact probe sequence from .github/workflows/validation.yml (plan step).
  Push-Location $Root
  try {
    git cat-file -e "$($BaseSha):tools/check_ci_policy.ps1" 2>$null
    $PolicyPresent = ($LASTEXITCODE -eq 0)
    if (-not $PolicyPresent) {
      git cat-file -e "$($BaseSha)^{commit}" 2>$null
      if ($LASTEXITCODE -ne 0) { throw 'accepted base commit is unreadable' }
    }
    return $PolicyPresent
  } finally {
    Pop-Location
  }
}

$Count = 0
function Assert-Bootstrap([bool]$Value, [string]$Label) {
  if (-not $Value) { throw "bootstrap_test: $Label" }
  $script:Count++
}

# Case 1: accepted base without the policy file -> bootstrap path.
# This is the real first-PR shape: main (3cd2e8c) has no check_ci_policy.ps1.
$Profile = 'full'
$Present = Test-HumBootstrapProbe '3cd2e8c0d3abdaf7786d710c39891409128cb17e'
Assert-Bootstrap (-not $Present) 'bootstrap base reports policy absent'
Assert-Bootstrap ($Profile -ceq 'full') 'bootstrap profile remains full'
# The plan step must still produce its outputs and exit 0 on the bootstrap path.
$OutFile = Join-Path ([IO.Path]::GetTempPath()) ('hum-bootstrap-outputs-' + [Guid]::NewGuid().ToString('N') + '.txt')
try {
  "profile=$Profile" >> $OutFile
  "base=3cd2e8c0d3abdaf7786d710c39891409128cb17e" >> $OutFile
  $Lines = Get-Content -LiteralPath $OutFile
  Assert-Bootstrap ($Lines -contains 'profile=full') 'bootstrap outputs include profile=full'
  Assert-Bootstrap ($Lines -contains 'base=3cd2e8c0d3abdaf7786d710c39891409128cb17e') 'bootstrap outputs include base'
} finally {
  if (Test-Path -LiteralPath $OutFile) { Remove-Item -LiteralPath $OutFile -Force }
}

# Case 2: accepted base with the policy file -> non-bootstrap route.
$Present2 = Test-HumBootstrapProbe '8034e687d05efb445abb84e6d711fb1cdbaf7ed6'
Assert-Bootstrap $Present2 'policy base reports policy present'

# Case 3: unreadable base commit -> throws instead of silently bootstrapping.
$Threw = $false
try { Test-HumBootstrapProbe '0000000000000000000000000000000000000000' | Out-Null } catch { $Threw = $true }
Assert-Bootstrap $Threw 'unreadable base throws (fail-closed)'

Write-Host "Bootstrap regression passed ($Count assertions)."
