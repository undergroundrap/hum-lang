$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$ClaimsPath = Join-Path (Join-Path $RepoRoot 'docs') (Join-Path 'alpha' 'claims-matrix.v0.1.json')
$Failures = New-Object System.Collections.Generic.List[string]

function Add-Failure {
  param([string] $Message)
  $Failures.Add($Message)
}

function Test-RepoRelativePath {
  param([string] $Path)

  if ([string]::IsNullOrWhiteSpace($Path)) {
    return $false
  }

  if ([System.IO.Path]::IsPathRooted($Path)) {
    return $false
  }

  if ($Path.Contains('..')) {
    return $false
  }

  return $true
}

if (-not (Test-Path -LiteralPath $ClaimsPath)) {
  Add-Failure 'docs/alpha/claims-matrix.v0.1.json is missing'
} else {
  $claimsJson = [System.IO.File]::ReadAllText($ClaimsPath, [System.Text.Encoding]::UTF8)
  $matrix = $claimsJson | ConvertFrom-Json

  if ($matrix.schema -ne 'hum.alpha_claims.v0') {
    Add-Failure 'claims matrix schema must be hum.alpha_claims.v0'
  }

  if ($matrix.alpha -ne 'offline-tool@0.1') {
    Add-Failure 'claims matrix alpha must be offline-tool@0.1'
  }

  $allowedStatuses = @('planned', 'chartered', 'in_progress', 'blocked', 'proven', 'deferred')
  $ids = New-Object System.Collections.Generic.HashSet[string]

  if ($null -eq $matrix.claims -or $matrix.claims.Count -eq 0) {
    Add-Failure 'claims matrix must contain at least one claim'
  } else {
    foreach ($claim in $matrix.claims) {
      if (-not ($claim.id -match '^HA[0-9][0-9]$')) {
        Add-Failure ("claim id is invalid: {0}" -f $claim.id)
      } elseif (-not $ids.Add($claim.id)) {
        Add-Failure ("duplicate claim id: {0}" -f $claim.id)
      }

      if ($allowedStatuses -notcontains $claim.status) {
        Add-Failure ("claim {0} has invalid status: {1}" -f $claim.id, $claim.status)
      }

      foreach ($field in @('scope', 'claim')) {
        if ([string]::IsNullOrWhiteSpace($claim.$field)) {
          Add-Failure ("claim {0} is missing {1}" -f $claim.id, $field)
        }
      }

      if ($null -eq $claim.current_evidence -or $claim.current_evidence.Count -eq 0) {
        Add-Failure ("claim {0} has no current_evidence" -f $claim.id)
      } else {
        foreach ($path in $claim.current_evidence) {
          if (-not (Test-RepoRelativePath $path)) {
            Add-Failure ("claim {0} has invalid current evidence path: {1}" -f $claim.id, $path)
            continue
          }

          $resolved = [System.IO.Path]::GetFullPath((Join-Path $RepoRoot $path))
          if (-not $resolved.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
            Add-Failure ("claim {0} evidence escapes repo: {1}" -f $claim.id, $path)
          } elseif (-not (Test-Path -LiteralPath $resolved)) {
            Add-Failure ("claim {0} current evidence does not exist: {1}" -f $claim.id, $path)
          }
        }
      }

      if ($null -eq $claim.acceptance -or $claim.acceptance.Count -eq 0) {
        Add-Failure ("claim {0} has no acceptance criteria" -f $claim.id)
      }
    }
  }
}

if ($Failures.Count -gt 0) {
  Write-Host 'Alpha claims check failed:'
  foreach ($failure in $Failures) {
    Write-Host ("- {0}" -f $failure)
  }
  exit 1
}

Write-Host 'Alpha claims check passed.'
