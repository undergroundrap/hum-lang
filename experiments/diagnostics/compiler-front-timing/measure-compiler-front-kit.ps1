<#
.SYNOPSIS
  Scratch-only timing kit sampling the compiler-front execution families.
  Measures; never repairs, never judges. Timeouts and incomplete captures
  are retained honestly as incomplete measurements, never defect verdicts.

.DESCRIPTION
  Context: PR #69 (head a01f4dc2) Windows validation failed when the
  compiler-front group exceeded the 3000s capture deadline. Codex compared
  runs 36822239252 and 36921557967 on Ubuntu:
    compiler-front:  449598 ms -> 1353070 ms   (3.0x)
    compiler-corpus: 528590 ms ->  515260 ms   (~1.0x)
  Both restored Cargo caches; Rust moved 1.98.1 -> 1.99. The delta is
  localized to compiler-front but toolchain causality is NOT proven.

  This kit samples the four execution families inside
  Invoke-HumCompilerFrontChecks (tools/check_all.ps1 @ pinned head),
  executing each command through the PINNED capture helper
  tools/run_fast_evidence.ps1 (Invoke-HumBinaryCapture), dot-sourced
  through its supported guard ($MyInvocation.InvocationName -ne '.'
  keeps the producer main from running). Every non-timeout capture is
  authenticated with the helper's own Assert-HumCaptureComplete before
  any completion credit; cleanup failures are never swallowed; mutated
  source is restored only after authenticated descendant quiescence.

  Families (28 measured commands: 10 + 8 + 4 + 6):
    F0 launch-only control      : hum.exe --version (spawn+init overhead ONLY)
    F1 exact Rust selectors     : cargo test <sel> -- --exact --list  +  run
    F2 compiler-mutation work   : apply needle->replacement, list, run, restore
    F3 native/CLI probes        : hum.exe <subcommand> <fixture> invocations

.PINNED INPUTS
  Repo head : a01f4dc242cd075d7aff91090280ce9c38bcc8a7 (asserted before/after)
  Repo base : 2dfe3d9c230f2d45336bbde60a526c87c6ee1c5b (asserted as HEAD^)
  Capture helper sha256:
    1b547338649e6b3e875257caa2e3f2d15f716cf7aea52182a13740e85e3ec1f6
  SourceCopy: a FRESH OWNED scratch copy of the repo at the pinned head,
    inside the dedicated -ScratchBoundary. The executor prepares it (fresh
    clone or directory copy) and dedicates it to the kit. Mutations happen
    ONLY here; real worktrees are untouched. Must be clean before the run.
  ScratchBoundary: an executor-owned directory outside every registered git
    worktree. Both SourceCopy and OutputDirectory must resolve inside it.
    The kit refuses any SourceCopy/OutputDirectory outside the boundary.
  OutputDirectory: must NOT exist; the kit creates it fresh (inside boundary).

.REQUIRED CANDIDATE BUILD (explicit; separately authorized; the kit does NOT build)
  After the separately authorized honest build inside the scratch source
  copy (cargo build), the kit asserts target/debug/hum.exe exists, copies
  it OUTSIDE Cargo's target tree to <OutputDirectory>/frozen/hum.exe,
  records its SHA-256, and uses the FROZEN binary for F0/F3. Build and
  toolchain provenance (cargo/rustc versions, git head, binary hash/mtime)
  is recorded; the frozen binary is re-hashed afterward and must match.
  No installs, downloads, or toolchain changes are performed by the kit.

.BINARY / TOOLCHAIN REQUIREMENTS
  - pwsh 7+ (this script; asserted in preflight)
  - cargo resolvable via Get-Command (absolute path recorded; production's
    default target selection is preserved: plain `cargo test <selector>`,
    never --bin/--package substitutions)
  - rustc on PATH (version recorded via bounded capture)
  - git on PATH (all invocations go through the bounded capture helper;
    exits verified, never inferred from output text alone)

.BUDGET (absolute 900s from kit start; everything is inside it)
  Deadline accounting per launch:
    remaining_for_command = (deadline - now) - reserve(60s) - grace(5s)
    launch only if remaining_for_command >= 15s
    command timeout = min(180s, floor(remaining_for_command))
  The 60s reserve covers: final git verification (3 x 10s bounded),
  frozen re-hash, abort sweep, and summary write. Termination grace is
  charged per command (the helper may take timeout+grace wall time).
  A mutation is applied only if >= 150s remain after reserve+grace
  (two clamped phases + restore margin). Preflight calls are bounded
  (git 15s, cargo/rustc --version 30s).

.STATES (exactly one terminal state per planned ID; retained in plan.json + summary.json)
  measured    transport authenticated via Assert-HumCaptureComplete, timing valid
              outcome: ok | expected_failure | unexpected_exit
  incomplete  timed_out | transport_incomplete | compile_failure
              (timing excluded from completed-sample means)
  error       pin_drift | zero_selection | restore_mismatch | capture_error
  not-run     budget_exhausted | list_incomplete | aborted

  A selector's run phase is NEVER launched when its listing did not
  complete cleanly (timeout, compile failure, or selection error).

.INTERPRETATION RULES (binding on the reader)
  1. F0 measures spawn+init overhead ONLY. Do NOT subtract a multiple of
     it from family totals; do NOT equate any family's runtime with spawn
     overhead.
  2. Do NOT assume the historical ~1752-call inventory still applies; the
     kit reports only what it executed (summary.executed_count).
  3. Family means cover measured commands with outcome ok/expected_failure
     only; incomplete timings are excluded, never averaged in.
  4. Causality (e.g., Rust 1.99 vs cache state) requires the controlled
     host/toolchain comparison, not this kit alone.

.OUTPUTS (under the fresh -OutputDirectory)
  plan.json          the 28-entry planned inventory with exact argv arrays
  timing.ndjson      append-only per-command records, flushed per command
  cap-NNN/           pinned capture-helper directory per command (retained,
                     including on timeout/failure: incomplete evidence kept honestly)
  git-<tag>/         bounded capture directories for the kit's own git calls
  frozen/hum.exe     frozen measured binary + frozen.json provenance
  summary.json       ALWAYS written, including on failure (kit_error set);
                     final head/base/cleanliness/frozen-hash verification
                     failures FAIL the kit, not merely appear in the summary

.NOTES
  Dot-sourcing this file loads its functions WITHOUT executing the
  measurement (same guard idiom as the pinned helper), enabling
  inspection and unit validation of the budget/classification logic.
#>
[CmdletBinding()]
param(
  [string] $SourceCopy = '',
  [string] $OutputDirectory = '',
  [string] $ScratchBoundary = '',
  [string] $Cargo = 'cargo',
  [int] $PerCommandTimeoutSeconds = 180,
  [int] $AbsoluteBudgetSeconds = 900,
  [int] $ReserveSeconds = 60,
  [int] $TerminationGraceSeconds = 5
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:PinnedHead = 'a01f4dc242cd075d7aff91090280ce9c38bcc8a7'
$script:PinnedBase = '2dfe3d9c230f2d45336bbde60a526c87c6ee1c5b'
$script:PinnedHelperSha256 = '1b547338649e6b3e875257caa2e3f2d15f716cf7aea52182a13740e85e3ec1f6'
$script:Utf8 = New-Object System.Text.UTF8Encoding($false, $true)
$script:KitError = $null
$script:Plan = @()
$script:Records = @()
$script:RawRecords = @()
$script:PlanSeq = 0
$script:NdjsonPath = $null
$script:CargoVersion = $null
$script:RustcVersion = $null
$script:HumSourcePath = $null
$script:HumMtime = $null

function Write-KitConsole([string] $Text) { [Console]::Out.WriteLine($Text) }

function Get-FileSha256([string] $Path) {
  $h = [System.Security.Cryptography.SHA256]::Create()
  try {
    $s = [System.IO.File]::OpenRead($Path)
    try { return ([BitConverter]::ToString($h.ComputeHash($s))).Replace('-', '').ToLowerInvariant() }
    finally { $s.Dispose() }
  } finally { $h.Dispose() }
}

function Test-PathInside([string] $Child, [string] $Parent) {
  $Sep = [IO.Path]::DirectorySeparatorChar
  $Alt = [IO.Path]::AltDirectorySeparatorChar
  $C = [IO.Path]::GetFullPath($Child).TrimEnd($Sep, $Alt)
  $P = [IO.Path]::GetFullPath($Parent).TrimEnd($Sep, $Alt)
  if ($C -ceq $P) { return $true }
  $Cmp = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) {
    [StringComparison]::OrdinalIgnoreCase
  } else { [StringComparison]::Ordinal }
  return $C.StartsWith($P + $Sep, $Cmp)
}

function Test-KitLaunchBudget {
  param([datetime] $NowUtc, [datetime] $DeadlineUtc, [int] $ReserveSeconds,
        [int] $GraceSeconds, [int] $MaxTimeoutSeconds)
  $Remaining = ($DeadlineUtc - $NowUtc).TotalSeconds - $ReserveSeconds - $GraceSeconds
  if ($Remaining -lt 15) {
    return [pscustomobject]@{
      Launch = $false; TimeoutSeconds = 0
      RemainingSeconds = [math]::Round(($DeadlineUtc - $NowUtc).TotalSeconds, 1)
    }
  }
  $Timeout = [math]::Min($MaxTimeoutSeconds, [int][math]::Floor($Remaining))
  if ($Timeout -lt 1) { $Timeout = 1 }
  return [pscustomobject]@{
    Launch = $true; TimeoutSeconds = $Timeout
    RemainingSeconds = [math]::Round(($DeadlineUtc - $NowUtc).TotalSeconds, 1)
  }
}

function Test-KitMutationBudget {
  param([datetime] $NowUtc, [datetime] $DeadlineUtc, [int] $ReserveSeconds, [int] $GraceSeconds)
  $Remaining = ($DeadlineUtc - $NowUtc).TotalSeconds - $ReserveSeconds - $GraceSeconds
  return $Remaining -ge 150
}

function New-KitPlan {
  $script:PlanSeq = 0
  $Plan = New-Object 'System.Collections.Generic.List[object]'
  $Add = {
    param($Family, $Label, $Kind, $File, $Argv, $Selector, $Mutation, $Note)
    $script:PlanSeq = $script:PlanSeq + 1
    $e = [ordered]@{
      seq = $script:PlanSeq; family = $Family; label = $Label
      kind = $Kind; file = $File; argv = @($Argv); selector = $Selector
      mutation = $Mutation; note = $Note; state = 'planned'; outcome = $null
      reason = $null; elapsed_ms = $null; exit_code = $null; timed_out = $null
    }
    $Plan.Add($e)
  }
  for ($i = 1; $i -le 10; $i++) {
    & $Add 'F0' "launch-only $i/10" 'control' 'FROZEN_HUM' @('--version') '' $null 'spawn+init overhead only; rule 1 applies'
  }
  $Selectors = @(
    'parser::tests::string_braces_and_escaped_quotes_do_not_close_items',
    'ir_verify::tests::canonical_minimal_add_artifact_corruption_matrix_is_complete',
    'backend_cranelift::tests::minimal_add_jit_probe_matrix_is_exact',
    'parser::tests::complete_canonical_seal_reaches_private_core_and_rejects_transport_corruption'
  )
  foreach ($Sel in $Selectors) {
    & $Add 'F1' "$Sel list" 'list' 'CARGO' @('test', $Sel, '--', '--exact', '--list') $Sel $null 'production list+run shape; first invocation may pay pending compile'
    & $Add 'F1' "$Sel run" 'run' 'CARGO' @('test', $Sel, '--', '--exact') $Sel $null ''
  }
  $Mutations = @(
    [ordered]@{
      label = 'Wo22-B01 capability admission'; path = 'src/backend_cranelift.rs'
      needle = 'fault_at(fault, 0) || input.schema() != crate::backend_input::BACKEND_INPUT_SCHEMA'
      replacement = 'fault_at(fault, 0) && input.schema() != crate::backend_input::BACKEND_INPUT_SCHEMA'
      selector = 'backend_cranelift::tests::backend_go_no_go_rows_are_complete_and_load_bearing'
    },
    [ordered]@{
      label = 'Wo23-M01 live artifact identity'; path = 'src/ir_verify.rs'
      needle = "if live.bytes() != artifact {`n        return Err(`"integer_sign_artifact_live_binding_mismatch_v1`");`n    }"
      replacement = "if false && live.bytes() != artifact {`n        return Err(`"integer_sign_artifact_live_binding_mismatch_v1`");`n    }"
      selector = 'ir_verify::tests::integer_sign_artifact_rejection_matrix_is_complete'
    }
  )
  foreach ($M in $Mutations) {
    & $Add 'F2' "$($M.label) list" 'mut-list' 'CARGO' @('test', $M.selector, '--', '--exact', '--list') $M.selector $M 'mutation applied before this command'
    & $Add 'F2' "$($M.label) run" 'mut-run' 'CARGO' @('test', $M.selector, '--', '--exact') $M.selector $M 'production expects exit 101 + exactly the named FAILED; recorded, not judged'
  }
  $Probes = @(
    @('backend-probe human', @('backend-probe', '--format', 'human', 'examples/core/minimal_add.hum')),
    @('check inventory', @('check', 'fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum')),
    @('check comparison-conjunction', @('check', 'fixtures/foundation/pre_ar_comparison_conjunction_pass.hum')),
    @('core-preview json', @('core-preview', '--format=json', 'fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum')),
    @('run independent_comparisons', @('run', 'fixtures/foundation/pre_ar_comparison_conjunction_pass.hum', '--entry', 'independent_comparisons')),
    @('run comparison_looking_text', @('run', 'fixtures/foundation/pre_ar_comparison_conjunction_pass.hum', '--entry', 'comparison_looking_text'))
  )
  foreach ($Probe in $Probes) {
    & $Add 'F3' $Probe[0] 'probe' 'FROZEN_HUM' @($Probe[1]) '' $null 'exit recorded, not judged'
  }
  # NOTE: do NOT use @($Plan) here; the array subexpression over a generic
  # List[object] throws "Argument types do not match" on this host.
  return $Plan.ToArray()
}

function Write-KitRecord([hashtable] $Record) {
  [IO.File]::AppendAllText($script:NdjsonPath, (($Record | ConvertTo-Json -Compress -Depth 6) + "`n"), $script:Utf8)
}

# Bounded git through the pinned capture helper (replaces all unchecked git).
# Returns trimmed stdout. Throws on timeout, incomplete capture, or nonzero exit.
function Invoke-KitGit {
  param([string] $RepoRoot, [string[]] $Arguments, [string] $OutDir,
        [string] $Tag, [int] $TimeoutSeconds)
  $CapDir = Join-Path $OutDir ("git-$Tag")
  $Cap = Invoke-HumBinaryCapture 'git' $Arguments $RepoRoot $CapDir $TimeoutSeconds 5 -CaseName ("kit-git-$Tag")
  if ($Cap.TimedOut) { throw "git $($Arguments -join ' ') timed out after ${TimeoutSeconds}s" }
  $null = Assert-HumCaptureComplete $Cap
  if ($Cap.ExitCode -ne 0) {
    $ErrText = $script:Utf8.GetString([IO.File]::ReadAllBytes($Cap.StderrPath)).Trim()
    throw "git $($Arguments -join ' ') exited $($Cap.ExitCode): $ErrText"
  }
  return $script:Utf8.GetString([IO.File]::ReadAllBytes($Cap.StdoutPath)).Trim()
}

# Executes one planned entry through the pinned capture helper.
# Non-timeout captures are authenticated with the helper's own
# Assert-HumCaptureComplete before ANY completion credit; a failed
# capture is preserved (capture dir retained) with state error and
# no measurement credit. Cleanup/termination failures are never swallowed:
# Invoke-HumBinaryCapture throws on an invalid record.
function Invoke-KitMeasuredCommand {
  param($Entry, [string] $FilePath, [string] $WorkingDirectory,
        [string] $OutDir, [int] $TimeoutSeconds, [int] $GraceSeconds)
  $CaseName = ('kit-{0}-{1:D3}' -f ($Entry.family.ToLowerInvariant() -replace '[^a-z0-9]', ''), $Entry.seq)
  if ($CaseName.Length -gt 63) { $CaseName = $CaseName.Substring(0, 63) }
  $CapDir = Join-Path $OutDir ('cap-{0:D3}' -f $Entry.seq)
  $Record = [ordered]@{
    seq = $Entry.seq; family = $Entry.family; label = $Entry.label; kind = $Entry.kind
    file = $FilePath; argv = @($Entry.argv); case_name = $CaseName
    capture_dir = $CapDir; note = $Entry.note
    started_utc = [DateTime]::UtcNow.ToString('o')
    state = 'error'; outcome = $null; reason = 'not_run_yet'
    elapsed_ms = $null; exit_code = $null; timed_out = $null
  }
  Write-KitRecord $Record
  $Cap = Invoke-HumBinaryCapture $FilePath @($Entry.argv) $WorkingDirectory $CapDir $TimeoutSeconds $GraceSeconds -CaseName $CaseName
  $ElapsedMs = [math]::Round($Cap.DurationTicks * 1000.0 / $Cap.StopwatchFrequency, 1)
  $Record['ended_utc'] = [DateTime]::UtcNow.ToString('o')
  $Record['elapsed_ms'] = $ElapsedMs
  $Record['exit_code'] = $Cap.ExitCode
  $Record['timed_out'] = $Cap.TimedOut
  $Record['quiescence'] = [ordered]@{
    primary_exit = $Cap.PrimaryExitObserved; stdout_done = $Cap.StdoutCompletionObserved
    stderr_done = $Cap.StderrCompletionObserved; job_quiescent = $Cap.JobQuiescenceObserved
    final_active = $Cap.FinalActiveProcessCount
  }
  $Record['stdout_sha256'] = $Cap.StdoutSha256; $Record['stdout_bytes'] = $Cap.StdoutBytes
  $Record['stderr_sha256'] = $Cap.StderrSha256; $Record['stderr_bytes'] = $Cap.StderrBytes
  $Record['deadline_disposition'] = $Cap.DeadlineDisposition
  $Record['termination'] = [ordered]@{
    disposition = $Cap.TerminationDisposition; result = $Cap.TerminationResult; count = $Cap.TerminationCount
  }
  if ($Cap.TimedOut) {
    # Timeout containment was authenticated by the helper's own record
    # validation (quiescence enforced). Retained as incomplete, never a defect.
    $Record['state'] = 'incomplete'; $Record['reason'] = 'timeout'
  } else {
    try {
      $null = Assert-HumCaptureComplete $Cap
    } catch {
      # Failed capture preserved (capture dir retained); no completion credit.
      $Record['state'] = 'error'; $Record['reason'] = 'capture_error'
      $Record['capture_error'] = $_.Exception.Message
      Write-KitRecord $Record
      $script:RawRecords += $Record
      return $Record
    }
    $Record['state'] = 'measured'
  }
  Write-KitRecord $Record
  $script:RawRecords += $Record
  return $Record
}

function Read-CapStdout([string] $CapDir) {
  return $script:Utf8.GetString([IO.File]::ReadAllBytes((Join-Path $CapDir 'stdout.bin')))
}
function Read-CapStderr([string] $CapDir) {
  return $script:Utf8.GetString([IO.File]::ReadAllBytes((Join-Path $CapDir 'stderr.bin')))
}

function Test-CompileFailure([string] $Text) {
  return ($Text -match '(?m)^error(\[|:)' -or $Text -match 'could not compile')
}

# Exact nonzero selector execution, mirroring production's Invoke-ExactRustTest.
# Rejects zero, ignored (no : test$ lines), and duplicate/ambiguous selection.
function Assert-ExactSelection([string] $StdoutText, [string] $Selector, [string] $Label) {
  $Listed = @($StdoutText -split "`r?`n" | Where-Object { $_ -match ': test$' })
  $Escaped = [regex]::Escape($Selector)
  $Exact = @($Listed | Where-Object { $_ -cmatch "^${Escaped}: test$" })
  if ($Listed.Count -eq 0) { throw "$Label zero selection for '$Selector' (pin drift)" }
  if ($Listed.Count -ne 1 -or $Exact.Count -ne 1) {
    throw "$Label ambiguous selection for '$Selector': $($Listed.Count) listed, $($Exact.Count) exact (pin drift)"
  }
}

# Run-phase classification from actual results, not exit zero alone:
# the named test must have run and the summary must show exactly 1 passed.
function Test-KitRunResult {
  param([string] $Text, [string] $Selector, [int] $ExitCode)
  if (Test-CompileFailure $Text) { return 'compile_failure' }
  if ($ExitCode -ne 0) { return 'unexpected_exit' }
  $Esc = [regex]::Escape($Selector)
  $NamedOk = $Text -cmatch "(?m)^test $Esc \.\.\. ok$"
  $OnePassed = $Text -match '(?m)^test result: ok\. 1 passed;'
  if ($NamedOk -and $OnePassed) { return 'ok' }
  return 'unexpected_exit'
}

# Mutation failure credit requires EXACTLY the intended failed test:
# exit 101, one FAILED line total, and it names the intended selector,
# with no compilation errors.
function Test-KitMutationRunResult {
  param([string] $Text, [string] $Selector, [int] $ExitCode)
  if (Test-CompileFailure $Text) { return 'compile_failure' }
  $Esc = [regex]::Escape($Selector)
  $FailedLines = @($Text -split "`r?`n" | Where-Object { $_ -match '^test .* \.\.\. FAILED$' })
  $Intended = @($FailedLines | Where-Object { $_ -cmatch "^test $Esc \.\.\. FAILED$" })
  if ($ExitCode -eq 101 -and $FailedLines.Count -eq 1 -and $Intended.Count -eq 1) {
    return 'expected_failure'
  }
  return 'unexpected_exit'
}

# A selector's run phase may launch only if its listing completed cleanly.
function Test-KitRunLaunchAllowed {
  param([string] $Selector, [hashtable] $ListStatus)
  return $ListStatus[$Selector] -ceq 'ok'
}

function Invoke-KitMutation {
  param($ListEntry, $RunEntry, [string] $RepoRoot,
        [string] $CargoPath, [string] $OutDir, [datetime] $DeadlineUtc,
        [int] $ReserveSeconds, [int] $GraceSeconds, [int] $MaxTimeoutSeconds)
  $M = $ListEntry.mutation
  if (-not (Test-KitMutationBudget ([DateTime]::UtcNow) $DeadlineUtc $ReserveSeconds $GraceSeconds)) {
    foreach ($E in @($ListEntry, $RunEntry)) {
      $E['state'] = 'not-run'; $E['reason'] = 'budget_exhausted'; $E['outcome'] = $null
      $script:Records += $E
      Write-KitRecord $E
    }
    Write-KitConsole "SKIP mutation $($M.label): budget insufficient for list+run+restore"
    return
  }
  # Mutation targets resolve inside the dedicated scratch copy only.
  $Path = [IO.Path]::GetFullPath((Join-Path $RepoRoot $M.path))
  if (-not (Test-PathInside $Path $RepoRoot)) { throw "mutation target escapes scratch copy: $Path" }
  $OriginalBytes = [IO.File]::ReadAllBytes($Path)
  $Original = $script:Utf8.GetString($OriginalBytes)
  if (([regex]::Matches($Original, [regex]::Escape($M.needle))).Count -ne 1) {
    throw "mutation $($M.label) needle missing or not unique in $($M.path) (pin drift)"
  }
  $Mutated = $Original.Replace($M.needle, $M.replacement)
  if ($Mutated -ceq $Original) { throw "mutation $($M.label) replacement did not change source" }
  [IO.File]::WriteAllText($Path, $Mutated, $script:Utf8)
  Write-KitConsole "mutation applied: $($M.label)"
  $LastRaw = $null
  $ListOk = $false
  $MutationError = $null
  foreach ($E in @($ListEntry, $RunEntry)) {
    if ($E.kind -ceq 'mut-run' -and -not $ListOk) {
      # Never run a selector whose listing did not complete cleanly.
      $E['state'] = 'not-run'; $E['reason'] = 'list_incomplete'; $E['outcome'] = $null
      $script:Records += $E; Write-KitRecord $E
      Write-KitConsole "SKIP [$($E.seq)] $($E.label): listing did not complete"
      continue
    }
    $B = Test-KitLaunchBudget ([DateTime]::UtcNow) $DeadlineUtc $ReserveSeconds $GraceSeconds $MaxTimeoutSeconds
    if (-not $B.Launch) {
      $E['state'] = 'not-run'; $E['reason'] = 'budget_exhausted'
      $script:Records += $E; Write-KitRecord $E; continue
    }
    try {
      $R = Invoke-KitMeasuredCommand $E $CargoPath $RepoRoot $OutDir $B.TimeoutSeconds $GraceSeconds
    } catch {
      $E['state'] = 'error'; $E['reason'] = 'capture_error'; $E['outcome'] = $null
      $E['capture_error'] = $_.Exception.Message
      $script:Records += $E; Write-KitRecord $E
      $MutationError = "mutation $($M.label) $($E.kind): capture helper failed: $($_.Exception.Message)"
      break
    }
    $LastRaw = $R
    $Text = (Read-CapStdout $R.capture_dir) + "`n" + (Read-CapStderr $R.capture_dir)
    if ($R.state -ceq 'measured') {
      if ($E.kind -ceq 'mut-list') {
        if (Test-CompileFailure $Text) {
          $R['state'] = 'incomplete'; $R['reason'] = 'compile_failure'; $R['outcome'] = $null
        } else {
          try { Assert-ExactSelection (Read-CapStdout $R.capture_dir) $E.selector $E.label }
          catch { $R['state'] = 'error'; $R['reason'] = 'zero_selection'; $R['outcome'] = $null }
          if ($R.state -ceq 'measured') { $R['outcome'] = 'ok' }
        }
        $ListOk = ($R.state -ceq 'measured' -and $R.outcome -ceq 'ok')
      } else {
        $Res = Test-KitMutationRunResult $Text $E.selector $R.exit_code
        if ($Res -ceq 'expected_failure') { $R['outcome'] = 'expected_failure' }
        elseif ($Res -ceq 'compile_failure') { $R['state'] = 'incomplete'; $R['reason'] = 'compile_failure'; $R['outcome'] = $null }
        else { $R['outcome'] = 'unexpected_exit' }
      }
    }
    $E['state'] = $R.state; $E['outcome'] = $R.outcome; $E['reason'] = $R.reason
    $E['elapsed_ms'] = $R.elapsed_ms; $E['exit_code'] = $R.exit_code; $E['timed_out'] = $R.timed_out
    $script:Records += $E
    if ($E.state -ceq 'error') { $MutationError = "mutation $($M.label) $($E.kind) error: $($E.reason)"; break }
  }
  # Restore iff quiescence is authenticated for the last launched child, or
  # no child was launched after applying (nothing can be active). NEVER
  # restore while a compiler child may remain active.
  $Restored = $false
  $CanRestore = ($null -eq $LastRaw) -or
    ($LastRaw.quiescence.job_quiescent -and $LastRaw.quiescence.final_active -eq 0)
  if ($CanRestore) {
    [IO.File]::WriteAllBytes($Path, $OriginalBytes)
    $Check = [IO.File]::ReadAllBytes($Path)
    $Equal = $Check.Length -eq $OriginalBytes.Length
    if ($Equal) { for ($i = 0; $i -lt $Check.Length; $i++) { if ($Check[$i] -ne $OriginalBytes[$i]) { $Equal = $false; break } } }
    if (-not $Equal) { throw "mutation $($M.label) restore mismatch: $($M.path) not byte-identical" }
    $Restored = $true
    Write-KitConsole "mutation restored byte-identical: $($M.label)"
  } else {
    # Honest retention: the tree may still be mutated and a compiler child
    # may still be active. Do NOT touch the source; report it.
    Write-KitConsole "WARNING: $($M.label) NOT restored (quiescence not authenticated); source left as-is for inspection"
  }
  if ($MutationError) { throw $MutationError }
}

# Abort finalization: exactly one terminal state for every planned ID.
# Entries already recorded keep their state; every unstarted ID becomes
# a definitive not-run/aborted terminal record.
function Complete-KitAbortRecords {
  param([string] $Reason = 'aborted')
  $Seen = @{}
  foreach ($R in $script:Records) { $Seen[$R.seq] = $true }
  foreach ($E in $script:Plan) {
    if (-not $Seen.ContainsKey($E.seq)) {
      $E['state'] = 'not-run'; $E['reason'] = $Reason; $E['outcome'] = $null
      $script:Records += $E
      Write-KitRecord $E
    }
  }
  $script:Records = @($script:Records | Sort-Object { $_.seq })
}

function Write-KitSummary {
  param([string] $OutDir, [string] $RepoRoot, [string] $FrozenPath, [string] $FrozenSha256,
        [datetime] $KitStartUtc, [int] $AbsoluteBudgetSeconds)
  $GitAfter = [ordered]@{ head = $null; base = $null; clean = $null; error = $null }
  if ((Get-Command Invoke-KitGit -ErrorAction SilentlyContinue) -and
      -not [string]::IsNullOrWhiteSpace($RepoRoot)) {
    try {
      # Bounded: 3 x 10s fits inside the 60s final reserve.
      $GitAfter.head = Invoke-KitGit $RepoRoot @('rev-parse', 'HEAD') $OutDir 'final-head' 10
      $GitAfter.base = Invoke-KitGit $RepoRoot @('rev-parse', 'HEAD^') $OutDir 'final-base' 10
      $Porcelain = Invoke-KitGit $RepoRoot @('status', '--porcelain') $OutDir 'final-status' 10
      $GitAfter.clean = [string]::IsNullOrEmpty($Porcelain)
    } catch { $GitAfter.error = $_.Exception.Message }
  } else {
    $GitAfter.error = 'bounded git unavailable for final verification'
  }
  $FrozenAfter = $null
  if (-not [string]::IsNullOrWhiteSpace($FrozenPath) -and [IO.File]::Exists($FrozenPath)) {
    try { $FrozenAfter = Get-FileSha256 $FrozenPath } catch { $FrozenAfter = "unreadable: $($_.Exception.Message)" }
  }
  # Final verification: drift or unavailable verification FAILS the kit,
  # it does not merely appear in the summary.
  $VerifyErrors = New-Object 'System.Collections.Generic.List[string]'
  if ($GitAfter.error) {
    $VerifyErrors.Add("final git verification unavailable: $($GitAfter.error)")
  } else {
    if ($GitAfter.head -cne $script:PinnedHead) { $VerifyErrors.Add("final head drift: $($GitAfter.head)") }
    if ($GitAfter.base -cne $script:PinnedBase) { $VerifyErrors.Add("final base drift: $($GitAfter.base)") }
    if ($GitAfter.clean -ne $true) { $VerifyErrors.Add('final worktree not clean') }
  }
  if ($null -eq $FrozenSha256 -or $FrozenAfter -cne $FrozenSha256) {
    $VerifyErrors.Add('frozen binary hash mismatch or unavailable')
  }
  if ($VerifyErrors.Count -gt 0 -and [string]::IsNullOrEmpty($script:KitError)) {
    $script:KitError = "final verification failed: $($VerifyErrors -join '; ')"
  }
  $Families = [ordered]@{}
  foreach ($E in $script:Records) {
    if (-not $Families.Contains($E.family)) {
      $Families[$E.family] = [ordered]@{ measured_clean = 0; total_ms = 0; incomplete = 0; error = 0; not_run = 0; unexpected_exit = 0 }
    }
    $F = $Families[$E.family]
    switch ($E.state) {
      'measured' {
        if ($E.outcome -ceq 'ok' -or $E.outcome -ceq 'expected_failure') {
          $F.measured_clean += 1; $F.total_ms += $E.elapsed_ms
        } elseif ($E.outcome -ceq 'unexpected_exit') { $F.unexpected_exit += 1 }
      }
      'incomplete' { $F.incomplete += 1 }
      'error' { $F.error += 1 }
      'not-run' { $F.not_run += 1 }
    }
  }
  foreach ($Fam in @($Families.Keys)) {
    $F = $Families[$Fam]
    # Means cover completed clean samples only; incomplete timings excluded.
    $F['mean_ms'] = if ($F.measured_clean -gt 0) { [math]::Round($F.total_ms / $F.measured_clean, 1) } else { $null }
  }
  $Summary = [ordered]@{
    pins = [ordered]@{ head = $script:PinnedHead; base = $script:PinnedBase; helper_sha256 = $script:PinnedHelperSha256 }
    provenance = [ordered]@{
      cargo_version = $script:CargoVersion; rustc_version = $script:RustcVersion
      hum_source = $script:HumSourcePath; hum_frozen = $FrozenPath
      hum_sha256_before = $FrozenSha256; hum_sha256_after = $FrozenAfter
      hum_frozen_verified = ($null -ne $FrozenSha256 -and $FrozenAfter -ceq $FrozenSha256)
      hum_mtime_utc = $script:HumMtime
    }
    budget = [ordered]@{
      absolute_seconds = $AbsoluteBudgetSeconds
      elapsed_ms = [math]::Round(([DateTime]::UtcNow - $KitStartUtc).TotalMilliseconds, 1)
    }
    plan_count = $script:Plan.Count
    executed_count = @($script:Records | Where-Object { $_.state -ne 'not-run' }).Count
    inventory = @($script:Records)
    family_stats = $Families
    kit_error = $script:KitError
    verification_errors = @($VerifyErrors)
    git_after = $GitAfter
  }
  [IO.File]::WriteAllText((Join-Path $OutDir 'summary.json'), ($Summary | ConvertTo-Json -Depth 8), $script:Utf8)
  Write-KitConsole '==> summary (also in summary.json)'
  foreach ($Fam in @($Families.Keys)) {
    $F = $Families[$Fam]
    Write-KitConsole ("  {0}: clean_n={1} mean={2}ms incomplete={3} error={4} notrun={5} unexpected={6}" -f `
      $Fam, $F.measured_clean, $F.mean_ms, $F.incomplete, $F.error, $F.not_run, $F.unexpected_exit)
  }
  if ($script:KitError) { Write-KitConsole "  kit_error: $($script:KitError)" }
  Write-KitConsole ("  git_after: head={0} base={1} clean={2}" -f $GitAfter.head, $GitAfter.base, $GitAfter.clean)
}

function Main-Kit {
  $KitStartUtc = [DateTime]::UtcNow
  $DeadlineUtc = $KitStartUtc.AddSeconds($AbsoluteBudgetSeconds)
  $RepoRoot = $null
  $OutDir = $null
  $FrozenPath = $null
  $FrozenSha256 = $null
  try {
    if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'kit requires PowerShell 7+' }
    if ([string]::IsNullOrWhiteSpace($SourceCopy)) { throw '-SourceCopy is required (fresh owned scratch source copy)' }
    if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { throw '-OutputDirectory is required (must not exist)' }
    if ([string]::IsNullOrWhiteSpace($ScratchBoundary)) { throw '-ScratchBoundary is required (dedicated owned boundary)' }
    $RepoRoot = [IO.Path]::GetFullPath((Resolve-Path -LiteralPath $SourceCopy).Path)
    if (-not [IO.Directory]::Exists($RepoRoot)) { throw "SourceCopy not found: $SourceCopy" }
    # Fresh output directory: refuse to reuse.
    if ([IO.Directory]::Exists($OutputDirectory) -or [IO.File]::Exists($OutputDirectory)) {
      throw "OutputDirectory already exists (must be fresh): $OutputDirectory"
    }
    [IO.Directory]::CreateDirectory($OutputDirectory) | Out-Null
    $OutDir = [IO.Path]::GetFullPath($OutputDirectory)
    $script:NdjsonPath = Join-Path $OutDir 'timing.ndjson'

    # Planned inventory first: exact argv arrays, retained before execution.
    $script:Plan = New-KitPlan
    [IO.File]::WriteAllText((Join-Path $OutDir 'plan.json'), ((@($script:Plan) | ConvertTo-Json -Depth 6)), $script:Utf8)
    Write-KitConsole "plan: $($script:Plan.Count) commands"

    # Preflight: bounded, checked.
    Write-KitConsole '==> preflight'
    $HelperPath = Join-Path $RepoRoot 'tools/run_fast_evidence.ps1'
    if (-not [IO.File]::Exists($HelperPath)) { throw "capture helper missing: $HelperPath" }
    $HelperHash = Get-FileSha256 $HelperPath
    if ($HelperHash -cne $script:PinnedHelperSha256) { throw "capture helper hash mismatch: $HelperHash" }
    Write-KitConsole "helper pinned: $HelperHash"

    # Dot-source the pinned capture helper through its supported guard
    # ($MyInvocation.InvocationName -ne '.' keeps the producer main inert).
    . $HelperPath
    if (-not (Get-Command Invoke-HumBinaryCapture -ErrorAction SilentlyContinue)) {
      throw 'Invoke-HumBinaryCapture not available after dot-sourcing helper'
    }

    # Dedicated owned scratch-source boundary: SourceCopy and OutputDirectory
    # must live inside it, and it must live outside every registered worktree.
    $BoundaryFull = [IO.Path]::GetFullPath($ScratchBoundary)
    if (-not [IO.Directory]::Exists($BoundaryFull)) { throw "ScratchBoundary not found: $ScratchBoundary" }
    if (-not (Test-PathInside $RepoRoot $BoundaryFull)) { throw "SourceCopy outside ScratchBoundary: $RepoRoot" }
    if (-not (Test-PathInside $OutDir $BoundaryFull)) { throw "OutputDirectory outside ScratchBoundary: $OutDir" }
    $WtText = Invoke-KitGit $RepoRoot @('worktree', 'list', '--porcelain') $OutDir 'boundary-worktrees' 15
    $WtPaths = @($WtText -split "`r?`n" | Where-Object { $_ -match '^worktree ' } | ForEach-Object { $_.Substring(9) })
    foreach ($W in $WtPaths) {
      $Wf = [IO.Path]::GetFullPath($W)
      if (Test-PathInside $BoundaryFull $Wf) { throw "ScratchBoundary inside registered worktree: $W" }
      if ((Test-PathInside $Wf $BoundaryFull) -and $Wf -cne $RepoRoot) {
        throw "registered worktree inside ScratchBoundary: $W"
      }
    }
    Write-KitConsole "boundary ok: $BoundaryFull"

    $Head = Invoke-KitGit $RepoRoot @('rev-parse', 'HEAD') $OutDir 'preflight-head' 15
    if ($Head -cne $script:PinnedHead) { throw "head $Head != pinned $($script:PinnedHead)" }
    $Base = Invoke-KitGit $RepoRoot @('rev-parse', 'HEAD^') $OutDir 'preflight-base' 15
    if ($Base -cne $script:PinnedBase) { throw "base $Base != pinned $($script:PinnedBase)" }
    $Porcelain = Invoke-KitGit $RepoRoot @('status', '--porcelain') $OutDir 'preflight-status' 15
    if ($Porcelain) { throw "scratch copy not clean: $Porcelain" }

    $CargoPath = (Get-Command $Cargo -ErrorAction Stop).Source
    # Bounded toolchain version capture (no unbounded native calls).
    $CargoCapDir = Join-Path $OutDir 'tool-cargo-version'
    $CargoCap = Invoke-HumBinaryCapture $CargoPath @('--version') $RepoRoot $CargoCapDir 30 5 -CaseName 'kit-tool-cargo-version'
    if ($CargoCap.TimedOut) { throw 'cargo --version timed out' }
    $null = Assert-HumCaptureComplete $CargoCap
    if ($CargoCap.ExitCode -ne 0) { throw "cargo --version exited $($CargoCap.ExitCode)" }
    $script:CargoVersion = (Read-CapStdout $CargoCapDir).Trim()
    $RustcCapDir = Join-Path $OutDir 'tool-rustc-version'
    $RustcCap = Invoke-HumBinaryCapture 'rustc' @('--version') $RepoRoot $RustcCapDir 30 5 -CaseName 'kit-tool-rustc-version'
    if ($RustcCap.TimedOut) { throw 'rustc --version timed out' }
    $null = Assert-HumCaptureComplete $RustcCap
    if ($RustcCap.ExitCode -ne 0) { throw "rustc --version exited $($RustcCap.ExitCode)" }
    $script:RustcVersion = (Read-CapStdout $RustcCapDir).Trim()
    Write-KitConsole "cargo: $($script:CargoVersion) @ $CargoPath"
    Write-KitConsole "rustc: $($script:RustcVersion)"

    # Required candidate build (separately authorized): assert, then freeze
    # the binary OUTSIDE Cargo's target tree.
    $HumSource = Join-Path $RepoRoot 'target/debug/hum.exe'
    if (-not [IO.File]::Exists($HumSource)) {
      throw "required candidate build missing: $HumSource (run 'cargo build' first; the kit does not build)"
    }
    $script:HumSourcePath = $HumSource
    $script:HumMtime = [IO.File]::GetLastWriteTimeUtc($HumSource).ToString('o')
    $FrozenDir = Join-Path $OutDir 'frozen'
    [IO.Directory]::CreateDirectory($FrozenDir) | Out-Null
    $FrozenPath = Join-Path $FrozenDir 'hum.exe'
    [IO.File]::Copy($HumSource, $FrozenPath)
    $FrozenSha256 = Get-FileSha256 $FrozenPath
    if ($FrozenSha256 -cne (Get-FileSha256 $HumSource)) { throw 'frozen binary hash != source binary hash after copy' }
    $FrozenProvenance = [ordered]@{
      source = $HumSource; frozen = $FrozenPath; sha256 = $FrozenSha256
      bytes = (Get-Item $FrozenPath).Length; mtime_utc = $script:HumMtime
      cargo_version = $script:CargoVersion; rustc_version = $script:RustcVersion
      git_head = $Head
    }
    [IO.File]::WriteAllText((Join-Path $FrozenDir 'frozen.json'), ($FrozenProvenance | ConvertTo-Json -Depth 4), $script:Utf8)
    Write-KitConsole "frozen binary: $FrozenSha256"

    foreach ($F in @('examples/core/minimal_add.hum',
        'fixtures/foundation/pre_ar_canonical_seal_inventory_pass.hum',
        'fixtures/foundation/pre_ar_comparison_conjunction_pass.hum')) {
      if (-not [IO.File]::Exists((Join-Path $RepoRoot $F))) { throw "fixture missing: $F" }
    }

    # Execute the plan in order.
    $ListStatus = @{}
    $i = 0
    while ($i -lt $script:Plan.Count) {
      $E = $script:Plan[$i]
      if ($E.kind -ceq 'mut-list') {
        $RunE = $script:Plan[$i + 1]
        if ($RunE.kind -cne 'mut-run') { throw 'plan inventory malformed: mut-list not followed by mut-run' }
        Invoke-KitMutation $E $RunE $RepoRoot $CargoPath $OutDir $DeadlineUtc $ReserveSeconds $TerminationGraceSeconds $PerCommandTimeoutSeconds
        $i += 2
        continue
      }
      if ($E.kind -ceq 'run' -and -not (Test-KitRunLaunchAllowed $E.selector $ListStatus)) {
        # Never run a selector whose listing did not complete cleanly.
        $E['state'] = 'not-run'; $E['reason'] = 'list_incomplete'; $E['outcome'] = $null
        $script:Records += $E; Write-KitRecord $E
        Write-KitConsole "SKIP [$($E.seq)] $($E.label): listing did not complete"
        $i += 1; continue
      }
      $B = Test-KitLaunchBudget ([DateTime]::UtcNow) $DeadlineUtc $ReserveSeconds $TerminationGraceSeconds $PerCommandTimeoutSeconds
      if (-not $B.Launch) {
        $E['state'] = 'not-run'; $E['reason'] = 'budget_exhausted'
        $script:Records += $E; Write-KitRecord $E
        Write-KitConsole "SKIP [$($E.seq)] $($E.label): budget exhausted"
        $i += 1; continue
      }
      $File = if ($E.file -ceq 'FROZEN_HUM') { $FrozenPath } else { $CargoPath }
      try {
        $R = Invoke-KitMeasuredCommand $E $File $RepoRoot $OutDir $B.TimeoutSeconds $TerminationGraceSeconds
      } catch {
        $E['state'] = 'error'; $E['reason'] = 'capture_error'; $E['outcome'] = $null
        $E['capture_error'] = $_.Exception.Message
        $script:Records += $E; Write-KitRecord $E
        throw
      }
      $Text = (Read-CapStdout $R.capture_dir) + "`n" + (Read-CapStderr $R.capture_dir)
      if ($R.state -ceq 'measured') {
        switch ($E.kind) {
          'list' {
            if (Test-CompileFailure $Text) {
              $R['state'] = 'incomplete'; $R['reason'] = 'compile_failure'; $R['outcome'] = $null
            } else {
              try { Assert-ExactSelection (Read-CapStdout $R.capture_dir) $E.selector $E.label }
              catch { $R['state'] = 'error'; $R['reason'] = 'zero_selection'; $R['outcome'] = $null }
              if ($R.state -ceq 'measured') { $R['outcome'] = 'ok' }
            }
          }
          'run' {
            $Res = Test-KitRunResult $Text $E.selector $R.exit_code
            if ($Res -ceq 'ok') { $R['outcome'] = 'ok' }
            elseif ($Res -ceq 'compile_failure') { $R['state'] = 'incomplete'; $R['reason'] = 'compile_failure'; $R['outcome'] = $null }
            else { $R['outcome'] = 'unexpected_exit' }
          }
          default { $R['outcome'] = 'ok' }  # control + probe: exit recorded, not judged
        }
      }
      $E['state'] = $R.state; $E['outcome'] = $R.outcome; $E['reason'] = $R.reason
      $E['elapsed_ms'] = $R.elapsed_ms; $E['exit_code'] = $R.exit_code; $E['timed_out'] = $R.timed_out
      $script:Records += $E
      if ($E.kind -ceq 'list') {
        $ListStatus[$E.selector] = if ($E.state -ceq 'measured' -and $E.outcome -ceq 'ok') { 'ok' } else { 'bad' }
      }
      $StateNote = if ($R.state -ceq 'measured') { $R.outcome } else { "$($R.state)/$($R.reason)" }
      Write-KitConsole ("[{0:D3}] {1}/{2} {3}ms {4}" -f $E.seq, $E.family, $E.label, $R.elapsed_ms, $StateNote)
      if ($E.state -ceq 'error') { throw "command [$($E.seq)] $($E.label) error: $($E.reason)" }
      $i += 1
    }
  } catch {
    $script:KitError = $_.Exception.Message
    Write-KitConsole "KIT ERROR: $($script:KitError)"
    Complete-KitAbortRecords -Reason 'aborted'
  } finally {
    # Summary is ALWAYS written, including on failure; final verification
    # failures fail the kit inside Write-KitSummary.
    if ($null -ne $OutDir) {
      Write-KitSummary -OutDir $OutDir -RepoRoot $RepoRoot -FrozenPath $FrozenPath `
        -FrozenSha256 $FrozenSha256 -KitStartUtc $KitStartUtc -AbsoluteBudgetSeconds $AbsoluteBudgetSeconds
    }
  }
}

# Dot-source guard: sourcing this file loads its functions WITHOUT executing
# the measurement (same idiom as the pinned helper).
if ($MyInvocation.InvocationName -ne '.') {
  Main-Kit
}
