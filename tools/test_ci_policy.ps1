param([switch]$CompilerConsumerOnly, [string]$HumPath = '')
$ErrorActionPreference = 'Stop'
$Root = Split-Path $PSScriptRoot -Parent
. (Join-Path $PSScriptRoot 'check_ci_policy.ps1')
$Count = 0
function Assert-Policy([bool] $Value, [string] $Label) {
  if (-not $Value) { throw "ci_policy_test: $Label" }
  $script:Count++
}
function Assert-PolicyRejects([scriptblock] $Action, [string] $Label) {
  $Caught = $false
  try { & $Action | Out-Null } catch { $Caught = $true }
  Assert-Policy $Caught $Label
}
function Import-PolicyTestFunction([string] $Source, [string] $Name) {
  $Tokens = $null; $Errors = $null
  $Ast = [Management.Automation.Language.Parser]::ParseInput($Source,[ref]$Tokens,[ref]$Errors)
  if ($Errors.Count -ne 0) { throw 'ci_policy_test: source parse failed' }
  $Definitions = @($Ast.FindAll({ param($Node) $Node -is [Management.Automation.Language.FunctionDefinitionAst] -and $Node.Name -ceq $Name }, $true))
  if ($Definitions.Count -ne 1) { throw "ci_policy_test: function ownership $Name" }
  [scriptblock]::Create($Definitions[0].Extent.Text)
}
# Decision 0025: fixtures that exercise the real workflow health gates for
# cheap profiles install this isolated gh mock. The mock answers the two
# metadata calls the gates make (workflow runs, then --paginate jobs) with a
# successful scheduled main result, so a cheap-profile fixture exercises the
# health check honestly instead of routing Full to avoid it. Any pre-existing
# gh command and the prior health-conclusion variable are preserved and
# restored exactly; the variable is uniquely named to avoid collisions.
function Install-HumCiTestGhMock {
  $State = [pscustomobject]@{ HadCommand = $false; PriorFunction = $null; HadHealthVar = $false; PriorHealth = $null }
  $Prior = Get-Command gh -ErrorAction SilentlyContinue
  if ($null -ne $Prior) {
    $State.HadCommand = $true
    if ($Prior.CommandType -ceq 'Function') { $State.PriorFunction = $Prior.ScriptBlock }
  }
  if (Test-Path Variable:global:HumCiTestHealthConclusion) {
    $State.HadHealthVar = $true
    $State.PriorHealth = $global:HumCiTestHealthConclusion
  }
  $global:HumCiTestHealthConclusion = 'success'
  # Global scope: the workflow code runs through nested child scopes
  # (scriptblock dot-source inside & blocks), where a script-local function
  # is not reliably visible.
  function global:gh {
    $global:LASTEXITCODE = 0
    if ($args -contains '--paginate') {
      $Jobs = foreach ($Platform in @('windows','ubuntu')) {
        $Names = @('Checkout','Verify integration identity','Run Hum preflight','Close full evidence transport','Confirm selected work completion')
        if ($Platform -ceq 'ubuntu') { $Names += 'Run exhaustive canonical-seal evidence' }
        @{name="evaluate / preflight ($Platform-latest)";run_id=42;head_sha=('a'*40);status='completed';conclusion='success';steps=@($Names|ForEach-Object{@{name=$_;status='completed';conclusion='success'}})}
      }
      ConvertTo-Json -InputObject @(@{jobs=@($Jobs)}) -Depth 8 -Compress
    } else {
      @{workflow_runs=@(@{event='schedule';path='.github/workflows/validation.yml';repository=@{full_name='owner/repo'};head_branch='main';head_sha=('a'*40);status='completed';conclusion=$global:HumCiTestHealthConclusion;id=42;run_attempt=1;created_at=[datetimeoffset]::UtcNow.ToString('o')})}|ConvertTo-Json -Depth 8 -Compress
    }
  }
  Assert-Policy ((Get-Command gh -ErrorAction Stop).CommandType -ceq 'Function') 'gh resolves to the isolated mock inside the fixture'
  Assert-Policy ((Get-Command gh).Definition -match 'HumCiTestHealthConclusion') 'gh resolves to this mock, not a pre-existing function'
  $State
}
function Remove-HumCiTestGhMock([pscustomobject] $State) {
  # 'Function:\gh' resolves through the scope chain to the global mock. The
  # 'Function:global:gh' path form silently removes nothing (verified
  # 2026-09-23); the old code used it and leaked the mock, which the
  # assertion below now catches fail-closed.
  Remove-Item -Path 'Function:\gh' -ErrorAction SilentlyContinue
  if ($State.HadCommand -and $null -ne $State.PriorFunction) {
    Set-Item -Path 'function:global:gh' -Value $State.PriorFunction -Force
  }
  if ($State.HadHealthVar) { $global:HumCiTestHealthConclusion = $State.PriorHealth }
  else { Remove-Variable -Name HumCiTestHealthConclusion -Scope Global -ErrorAction SilentlyContinue }
  $After = Get-Command gh -ErrorAction SilentlyContinue
  $MockStillResolves = ($null -ne $After) -and ($After.CommandType -ceq 'Function') -and ($After.Definition -match 'HumCiTestHealthConclusion')
  Assert-Policy (-not $MockStillResolves) 'gh no longer resolves to the isolated mock after the fixture'
  $global:LASTEXITCODE = 0
}

if ($CompilerConsumerOnly) {
  if (-not [IO.Path]::IsPathRooted($HumPath)) { throw 'ci_policy_test: explicit compiler required' }
  if ($PSVersionTable.PSVersion.Major -eq 5) {
    # A fresh PS5.1 control may inherit a PS7-only module search path. Load
    # the installed host module directly; do not alter the environment.
    Import-Module (Join-Path $PSHOME 'Modules/Microsoft.PowerShell.Utility/Microsoft.PowerShell.Utility.psd1') -ErrorAction Stop
  }
  . (Join-Path $PSScriptRoot 'run_fast_evidence.ps1')
  # This is the canonical Cargo compiler, as used by Full, not an isolated
  # hum-dev copy. Cargo may hard-link it to its deps output.
  $Before = Get-HumExecutableIdentity $HumPath -AllowHardLinks
  $Text = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'check_all.ps1'))
  foreach($Name in @('Read-NativeOutputWithExit','Assert-Json','Invoke-HumUseAfterMoveRuntimeCheck','Invoke-HumUseAfterMoveProjectionCheck')) {
    . (Import-PolicyTestFunction $Text $Name)
  }
  Push-Location $Root
  try {
    Invoke-HumUseAfterMoveRuntimeCheck $HumPath
    Invoke-HumUseAfterMoveProjectionCheck $HumPath
    $After = Get-HumExecutableIdentity $HumPath -AllowHardLinks
    Assert-Policy (($Before | ConvertTo-Json -Depth 5 -Compress) -ceq ($After | ConvertTo-Json -Depth 5 -Compress)) 'compiler identity unchanged'
  } finally { Pop-Location }
  Write-Output 'Actual shared use-after-move runtime and ownership projection consumers passed; not complete-profile evidence.'
  return
}

foreach ($Case in @(
  @('language', @('examples/probes/word_count.hum')),
  @('runtime', @('examples/probes/word_count.hum','src/run.rs','docs/LANGUAGE_REFERENCE.md')),
  @('compiler', @('src/parser.rs')), @('compiler', @('src/type_check.rs')),
  @('language', @('README.md','docs/LANGUAGE_REFERENCE.md')),
  @('full', @('.github/workflows/validation.yml')),
  @('full', @('unknown.file')),
  @('full', @('src/new_unmapped.rs')), @('full', @('examples/new_unmapped.hum')),
  @('full', @('fixtures/new_unmapped.json')), @('full', @('fixtures/new_unmapped.hum')),
  @('language', @('docs/HUM_CORE_VERIFY_SCHEMA.md')),
  @('language', @('docs/bakeoff/EFFECT_POLYMORPHISM_CORPUS.md')),
  @('language', @('docs/TEXT_HYGIENE_WORKFLOW.md')),
  @('full', @('CONTRIBUTING.md')),
  # Decision 0025: owned prefixes (docs/, workorders/) classify at language
  # rank by path; unknown prefixes still classify Full. tools/ has NO prefix
  # entry: only hygiene-group scripts are pinned at language rank as literals;
  # everything else under tools/ (including check_all.ps1, the Full preflight)
  # defaults to Full.
  @('full', @('tools/run_fast_evidence.ps1')),
  @('full', @('tools/check_all.ps1')),
  @('full', @('tools/test_fast_evidence_capture.ps1')),
  @('full', @('tools/test_exact_rust_selector.ps1')),
  @('full', @('tools/check_editor_fixtures.ps1')),
  @('full', @('tools/new_unlisted_tool.ps1')),
  @('language', @('tools/check_text_hygiene.ps1')),
  @('language', @('tools/test_ci_policy.ps1')),
  @('language', @('docs/TESTING_STRATEGY.md')),
  @('language', @('docs/new_policy.md')),
  @('language', @('workorders/completed/2026-09-22-fix-validation-bootstrap-probe.md')),
  # Decision 0025 exceptions: the docs compiled into the binary via include_str!
  # in src/diagnostic_catalog.rs keep compiler rank. tools/check_ci_policy.ps1
  # is the highest-sensitivity tooling path; it keeps a code-level profile
  # whose hygiene group runs the classification's own real gates.
  @('compiler', @('docs/DIAGNOSTICS.md')),
  @('compiler', @('docs/DIAGNOSTICS_SCHEMA_0_1.md')),
  @('compiler', @('docs/EFFECT_REPORT_SCHEMA_0_1.md')),
  @('compiler', @('docs/SECURITY_MODEL.md')),
  @('compiler', @('docs/UNSAFE_POLICY.md')),
  @('compiler', @('docs/RUNTIME_PROFILES.md')),
  @('compiler', @('docs/LANGUAGE_SUBSET_0_1.md')),
  @('compiler', @('docs/PORTABILITY_BOUNDARY_MODEL.md')),
  @('language', @('tools/check_ci_policy.ps1')),
  @('runtime', @('src/run.rs','tools/check_ci_policy.ps1')),
  @('compiler', @('fixtures/ownership_check/session_j_use_after_move_fail.hum')),
  @('full', @())
)) { Assert-Policy ((Get-HumCiProfile $Case[1]) -ceq $Case[0]) "routing $($Case[1] -join ',')" }
foreach ($Paths in @(@('src/run.rs','SRC/run.rs'), @('../src/run.rs'), @('src//run.rs'), @("src/run.rs`nother"), @(('C'+':/src/run.rs')), @('src\run.rs'), @(''))) {
  Assert-PolicyRejects { Get-HumCiProfile $Paths } 'malformed inventory cannot select cheaper work'
}
# Decision 0025: workorders/ routes at language rank only because the hygiene
# group executes its consumer, the status-boundary classifier. Pin the real
# wiring in check_all.ps1, not a re-implementation.
$CheckAllText = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'check_all.ps1'))
$HygieneMatch = [regex]::Match($CheckAllText, "(?ms)^    'hygiene' \{(?<body>.*?)^    \}")
Assert-Policy $HygieneMatch.Success 'hygiene group owner exists in check_all.ps1'
Assert-Policy ($HygieneMatch.Groups['body'].Value -cmatch "test_workorder_status_boundary\.ps1") 'hygiene group executes the work-order status-boundary consumer'
# Decision 0025 amendment (2026-09-23, boolean transport): in the fixed
# profiles the boundary tests run only when the plan/push classify step
# computed boundary_required=true from the ACCEPTED BASE policy. check_all
# reads only the boolean env var; it never sees the path inventory, never
# calls the trigger function, and never dot-sources the PR's policy. Only an
# explicit 'false' skips; missing, empty, or malformed values run (fail safe).
# Each entry is @(expected, paths) for the BASE trigger function matrix.
Assert-Policy ($HygieneMatch.Groups['body'].Value -cmatch 'HUM_CI_BOUNDARY_REQUIRED') 'hygiene boundary gate reads the precomputed boolean'
Assert-Policy ($HygieneMatch.Groups['body'].Value -cnotmatch 'HUM_CI_CHANGE_PATHS') 'hygiene boundary gate carries no path inventory'
Assert-Policy ($HygieneMatch.Groups['body'].Value -cnotmatch 'Test-HumCiWorkOrderBoundaryTrigger') 'hygiene boundary gate does not call the trigger function'
foreach ($Case in @(
  @($true,  $null),
  @($true,  @()),
  @($false, @('src/main.rs')),
  @($false, @('src/main.rs', 'docs/LANGUAGE_REFERENCE.md')),
  @($true,  @('workorders/active/2026-09-23-wordfreq-text-primitives.md')),
  @($true,  @('workorders/completed/2026-09-22-fix-validation-bootstrap-probe.md')),
  @($true,  @('tools/check_workorder_status_boundary.ps1')),
  @($true,  @('tools/test_workorder_status_boundary.ps1')),
  @($true,  @('tools/Find-ActiveWorkorder.ps1')),
  @($true,  @('.github/workflows/ci.yml')),
  @($true,  @('src/main.rs', 'tools/Find-ActiveWorkorder.ps1')),
  @($false, @('docs/LANGUAGE_REFERENCE.md')),
  @($false, @('tools/check_text_hygiene.ps1')),
  @($false, @('src/main.rs', 'tools/check_text_hygiene.ps1', 'docs/DIAGNOSTICS.md'))
)) {
  $Expected, $Paths = $Case
  $Label = if ($null -eq $Paths) { '<null>' } else { ($Paths -join ', ') }
  Assert-Policy ((Test-HumCiWorkOrderBoundaryTrigger -Paths $Paths) -eq $Expected) "boundary trigger for [$Label] must be $Expected"
}
# Fail safe: blank-only inventories run the tests; non-consumer paths with
# line endings do not trigger.
Assert-Policy (Test-HumCiWorkOrderBoundaryTrigger -Paths @('', '   ')) 'blank-only inventory fails safe to true'
Assert-Policy (-not (Test-HumCiWorkOrderBoundaryTrigger -Paths @(('src/main.rs' + [char]13 + [char]10), 'docs/LANGUAGE_REFERENCE.md'))) 'non-consumer paths with line endings must not trigger'
# Boolean transport (2026-09-23): check_all must not dot-source the policy
# script — the PR's own trigger function must never decide the skip. The
# boolean is computed by the plan/push classify step from the ACCEPTED BASE
# policy before check_all runs.
$CheckAllTop = (Get-Content -Raw (Join-Path $Root 'tools/check_all.ps1'))
Assert-Policy ($CheckAllTop -cnotmatch "(?m)^\s*\.\s*\(.*check_ci_policy") 'check_all does not dot-source check_ci_policy.ps1'
# Fail-safe boolean reading: only an explicit 'false' skips. Extract the
# assignment from the hygiene gate and evaluate it against each input.
$BoundaryAssign = [regex]::Match($HygieneMatch.Groups['body'].Value, '\$BoundaryRequired\s*=\s*\$env:HUM_CI_BOUNDARY_REQUIRED\s*-cne\s*''false''').Value
Assert-Policy ($BoundaryAssign.Length -gt 0) 'hygiene gate uses explicit-false-only skip'
foreach ($Case in @(
  @($true,  $null),
  @($true,  ''),
  @($true,  '   '),
  @($true,  'true'),
  @($true,  'TRUE'),
  @($true,  'yes'),
  @($true,  '0'),
  @($false, 'false')
)) {
  $Expected, $Raw = $Case
  $Label = if ($null -eq $Raw) { '<null>' } else { "'$Raw'" }
  Assert-Policy ((($Raw -cne 'false') -eq $Expected)) "boundary boolean for [$Label] must be $Expected"
}
Assert-Policy ((('FALSE' -cne 'false') -eq $true)) "boundary boolean is case-sensitive: 'FALSE' runs (fail safe)"

# Decision 0025: every include_str!/include_bytes! target under docs/ must have
# a code-level pin in check_ci_policy.ps1, so a newly compiled-in doc can't
# silently route cheap via the docs/ prefix (language rank). Scans every
# repository .rs file — the hum binary (src/), the workspace crates (crates/),
# and the experiments (experiments/) that CI still compiles and tests — not
# just the src/ tree, and never a fixture. tools/ targets need no pin: with no
# tools/ prefix entry, unlisted tools default to Full (the safe direction).
$PolicyText = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'check_ci_policy.ps1'))
$RsFiles = @()
foreach ($Top in @('src', 'crates', 'experiments')) {
  $Dir = Join-Path $Root $Top
  if ([IO.Directory]::Exists($Dir)) { $RsFiles += [IO.Directory]::GetFiles($Dir, '*.rs', [IO.SearchOption]::AllDirectories) }
}
$Unpinned = @()
foreach ($RsFile in $RsFiles) {
  $RsText = [IO.File]::ReadAllText($RsFile)
  foreach ($M in [regex]::Matches($RsText, 'include_(?:str|bytes)!\("([^"]+)"\)')) {
    $Target = $M.Groups[1].Value
    # Resolve relative to src/ (targets look like "../docs/X.md").
    $Resolved = [IO.Path]::GetFullPath((Join-Path (Split-Path $RsFile) $Target))
    $RepoRel = [IO.Path]::GetRelativePath($Root, $Resolved).Replace([IO.Path]::DirectorySeparatorChar, '/')
    if ($RepoRel.StartsWith('docs/')) {
      # Must appear as a literal pin: $Owners.Add('<path>', <rank>)
      $Escaped = [regex]::Escape($RepoRel)
      if ($PolicyText -notmatch "\`$Owners\.Add\('$Escaped',") {
        $Unpinned += "$RepoRel (from $(([IO.Path]::GetRelativePath($Root, $RsFile)).Replace([IO.Path]::DirectorySeparatorChar, '/')))"
      }
    }
  }
}
Assert-Policy ($Unpinned.Count -eq 0) ("compiled-in docs without code-level pin: " + ($Unpinned -join '; '))

$Workflow = [IO.File]::ReadAllText((Join-Path $Root '.github/workflows/validation.yml')).Replace(([string][char]13+[char]10),[string][char]10)
$Ci = [IO.File]::ReadAllText((Join-Path $Root '.github/workflows/ci.yml')).Replace(([string][char]13+[char]10),[string][char]10)
# Real read-only Git selection against owned synthetic commit objects; no
# source checkout, publication, policy stub or manually seeded path authority.
$Fixture=[IO.Path]::GetFullPath((Join-Path ([IO.Path]::GetTempPath()) ('hum-ci-policy-'+[Guid]::NewGuid().ToString('N'))))
if(Test-Path -LiteralPath $Fixture){throw 'ci_policy_test: fixture collision'}
[IO.Directory]::CreateDirectory($Fixture)|Out-Null
try {
  $null=Read-HumCiGit $Fixture @('init','-q')
  [IO.Directory]::CreateDirectory((Join-Path $Fixture 'src'))|Out-Null
  [IO.File]::WriteAllText((Join-Path $Fixture 'src/run.rs'),"base`n")
  $null=Read-HumCiGit $Fixture @('add','--','src/run.rs')
  $TreeA=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Identity=@('-c','user.name=HumPolicyFixture','-c','user.email=fixture@example.invalid')
  $Base=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$TreeA,'-m','base'))).Trim()
  [IO.File]::WriteAllText((Join-Path $Fixture 'src/run.rs'),"changed`n")
  $null=Read-HumCiGit $Fixture @('add','--','src/run.rs')
  $TreeB=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Head=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$TreeB,'-p',$Base,'-m','head'))).Trim()
  $Merge=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$TreeB,'-p',$Base,'-p',$Head,'-m','integration'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Merge)
  $Selection=Get-HumCiSelection $Fixture $Base $Head $Merge
  Assert-Policy ($Selection.Profile -ceq 'runtime' -and $Selection.Tree -ceq $TreeB -and $Selection.Paths.Count -eq 1 -and $Selection.Paths[0] -ceq 'src/run.rs') 'actual Git inventory and integration tree select Runtime'
  Assert-PolicyRejects {Get-HumCiSelection $Fixture $Head $Base $Merge} 'substituted integration parents'
  Assert-PolicyRejects {Get-HumCiSelection $Fixture $Base $Head $Head} 'stale checkout identity'
  # Actual raw Git objects, not filesystem symlinks or a live submodule.
  $Blob=(Read-HumCiGit $Fixture @('rev-parse',"${Head}:src/run.rs")).Trim()
  foreach($Mode in @('100755','120000','160000')) {
    $Object=if($Mode -ceq '160000'){$Head}else{$Blob}
    $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
    $null=Read-HumCiGit $Fixture @('update-index','--cacheinfo',"$Mode,$Object,src/run.rs")
    $KindTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
    $KindHead=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$KindTree,'-p',$Base,'-m','kind'))).Trim()
    $KindMerge=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$KindTree,'-p',$Base,'-p',$KindHead,'-m','merge'))).Trim()
    $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$KindMerge)
    Assert-Policy ((Get-HumCiSelection $Fixture $Base $KindHead $KindMerge).Profile -ceq 'full') "actual mode $Mode selects Full"
  }
  foreach($Path in @('src/new_unmapped.rs','examples/new_unmapped.hum','fixtures/new_unmapped.hum')) {
    $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
    $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,$Path")
    $NewTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
    $NewHead=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$NewTree,'-p',$Base,'-m','addition'))).Trim()
    $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$NewHead)
    Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $NewHead).Profile -ceq 'full') 'unregistered prefix addition cannot admit itself'
  }
  # Decision 0025: additions under owned prefixes classify by path.
  $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
  $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,docs/new_policy.md")
  $DocsAddTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $DocsAddHead=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$DocsAddTree,'-p',$Base,'-m','docs-addition'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$DocsAddHead)
  Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $DocsAddHead).Profile -ceq 'language') 'owned-prefix addition classifies by path'
  $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
  $null=Read-HumCiGit $Fixture @('update-index','--force-remove','--','src/run.rs')
  $DeletedTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Deleted=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$DeletedTree,'-p',$Base,'-m','deletion'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Deleted)
  Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $Deleted).Profile -ceq 'runtime') 'registered deletion classifies by path'
  $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,src/parser.rs")
  $RenameTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Renamed=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$RenameTree,'-p',$Base,'-m','rename'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Renamed)
  $RenameSelection=Get-HumCiPushSelection $Fixture $Base $Renamed
  Assert-Policy ($RenameSelection.Profile -ceq 'compiler' -and $RenameSelection.Paths -ccontains 'src/run.rs' -and $RenameSelection.Paths -ccontains 'src/parser.rs') 'both rename sides participate and the max rank wins'
  # Decision 0025: mode/type changes stay Full even under owned prefixes.
  foreach($ModeCase in @(@('100755','executable bit'),@('120000','symlink'),@('160000','gitlink'))) {
    $Mode=$ModeCase[0]
    $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
    $ModeObject=if($Mode -ceq '160000'){$Base}else{$Blob}
    $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"$Mode,$ModeObject,docs/tool_link")
    $ModeTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
    $ModeHead=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$ModeTree,'-p',$Base,'-m','mode-add'))).Trim()
    $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$ModeHead)
    Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $ModeHead).Profile -ceq 'full') "owned-prefix $($ModeCase[1]) addition selects Full"
  }
  # Decision 0025: deletions under owned prefixes classify by path.
  $null=Read-HumCiGit $Fixture @('read-tree',$TreeB)
  $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,docs/removable.md")
  $DocsBase=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',(Read-HumCiGit $Fixture @('write-tree')).Trim(),'-p',$Base,'-m','docs-base'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-index','--force-remove','--','docs/removable.md')
  $DocsDelTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $DocsDel=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$DocsDelTree,'-p',$DocsBase,'-m','docs-deletion'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$DocsDel)
  Assert-Policy ((Get-HumCiPushSelection $Fixture $DocsBase $DocsDel).Profile -ceq 'language') 'owned-prefix deletion classifies by path'
  # Complete multi-commit range, including a policy edit reverted before head.
  # Decision 0025: the policy script keeps a code-level profile, so the range's
  # max rank comes from the src/run.rs modification, not the policy add/revert.
  $null=Read-HumCiGit $Fixture @('read-tree',$TreeB)
  $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,tools/check_ci_policy.ps1")
  $PolicyTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $PolicyCommit=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$PolicyTree,'-p',$Head,'-m','policy'))).Trim()
  $Revert=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$TreeB,'-p',$PolicyCommit,'-m','revert'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Revert)
  Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $Revert).Profile -ceq 'runtime') 'reverted policy in earlier push commit classifies by remaining changes'
  $Again=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$TreeA,'-p',$Head,'-m','ordinary'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Again)
  Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $Again).Profile -ceq 'runtime') 'multi-commit ordinary push (net empty) executes Runtime'
  Assert-Policy ((Get-HumCiPushSelection $Fixture ('0'*40) $Again).Profile -ceq 'full') 'missing pre-push history selects Full'
  Assert-Policy ((Get-HumCiPushSelection $Fixture ('f'*40) $Again).Profile -ceq 'full') 'unresolved pre-push history selects Full'
  $Raw=Read-HumCiGit $Fixture @('diff','--no-ext-diff','--no-textconv','--no-renames','--raw','--no-abbrev','-z',$Base,$Head,'--')
  $Rows=@(ConvertFrom-HumCiRawChanges $Raw)
  Assert-Policy ($Rows.Count -eq 1 -and (Get-HumCiChangeProfile $Rows) -ceq 'runtime') 'actual raw modification parsed'
  foreach($Bad in @($Raw.TrimEnd([char]0),($Raw+':malformed'+[char]0+'src/parser.rs'+[char]0),($Raw+$Raw),$Raw.Replace('100644','000000'))) {
    $Published=New-Object 'Collections.Generic.List[object]';$Caught=$false
    try {ConvertFrom-HumCiRawChanges $Bad|ForEach-Object{$Published.Add($_)}}catch{$Caught=$true}
    Assert-Policy ($Caught -and $Published.Count -eq 0) "complete raw validation precedes publication: rejected=$Caught;published=$($Published.Count)"
  }
  # Execute the actual main-push admission block with real accepted-policy
  # loading and real Git history. Only remote health metadata is substituted.
  $PushStart=$Ci.IndexOf('          $SelectedProfile = $env:HUM_SELECTED_PROFILE',[StringComparison]::Ordinal)
  $PushEnd=$Ci.IndexOf('          if ($Mode -ceq ''fast'') {',$PushStart,[StringComparison]::Ordinal)
  Assert-Policy ($PushStart -ge 0 -and $PushEnd -gt $PushStart) 'main-push workflow owner'
  $PushCode=($Ci.Substring($PushStart,$PushEnd-$PushStart) -split "`n"|ForEach-Object{if($_.StartsWith('          ')){$_.Substring(10)}else{$_}})-join "`n"
  $PolicyFile=Join-Path $Fixture 'accepted-policy.ps1'
  [IO.File]::WriteAllText($PolicyFile,[IO.File]::ReadAllText((Join-Path $PSScriptRoot 'check_ci_policy.ps1')))
  $PolicyBlob=(Read-HumCiGit $Fixture @('hash-object','-w','--',$PolicyFile)).Trim()
  $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
  $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$PolicyBlob,tools/check_ci_policy.ps1")
  $AcceptedTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Accepted=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$AcceptedTree,'-p',$Base,'-m','accepted-policy'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-index','--cacheinfo',"100644,$Blob,src/run.rs")
  $OrdinaryTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Ordinary=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$OrdinaryTree,'-p',$Accepted,'-m','ordinary-push'))).Trim()
  [IO.File]::WriteAllText($PolicyFile,"throw 'candidate policy must never execute'`n")
  $PoisonBlob=(Read-HumCiGit $Fixture @('hash-object','-w','--',$PolicyFile)).Trim()
  $null=Read-HumCiGit $Fixture @('update-index','--cacheinfo',"100644,$PoisonBlob,tools/check_ci_policy.ps1")
  $PoisonTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Poison=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$PoisonTree,'-p',$Ordinary,'-m','proposed-policy'))).Trim()
  $SavedPush=@{}
  foreach($Key in @('HUM_SELECTED_PROFILE','HUM_CI_EVENT_NAME','HUM_CI_EVENT_REF','HUM_CI_BASE_SHA','HUM_CI_HEAD_SHA','RUNNER_TEMP','GITHUB_STEP_SUMMARY','GITHUB_OUTPUT','GITHUB_REPOSITORY','GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT')){$SavedPush[$Key]=[Environment]::GetEnvironmentVariable($Key,'Process')}
  Push-Location $Fixture
  try {
    $env:HUM_SELECTED_PROFILE='full';$env:HUM_CI_EVENT_NAME='push';$env:HUM_CI_EVENT_REF='refs/heads/main'
    $env:RUNNER_TEMP=$Fixture;$env:GITHUB_REPOSITORY='owner/repo';$env:GITHUB_RUN_ID='42';$env:GITHUB_RUN_ATTEMPT='1'
    $env:GITHUB_STEP_SUMMARY=Join-Path $Fixture 'summary.txt';$env:GITHUB_OUTPUT=Join-Path $Fixture 'output.txt'
    $GhState = Install-HumCiTestGhMock
    # Decision 0025: the push range Accepted..Poison holds the Ordinary commit
    # (src/run.rs, runtime) plus the poisoned policy modification (language);
    # the max rank wins. The mechanism under test is that the accepted policy
    # is consumed, never the poisoned candidate.
    foreach($Case in @(@($Accepted,$Ordinary,'runtime'),@($Accepted,$Poison,'runtime'),@($Base,$Head,'full'),@(('0'*40),$Ordinary,'full'))){
      $env:HUM_CI_BASE_SHA=$Case[0];$env:HUM_CI_HEAD_SHA=$Case[1];$global:HumCiTestHealthConclusion='success'
      $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Case[1])
      [IO.File]::WriteAllText($env:GITHUB_OUTPUT,'')
      & { $Mode='full'; . ([scriptblock]::Create($PushCode)) }
      $Lines=[IO.File]::ReadAllLines($env:GITHUB_OUTPUT)
      Assert-Policy ($Lines -ccontains "profile=$($Case[2])") 'actual push caller consumes accepted policy or Full fallback'
      # Decision 0025 amendment (2026-09-23, boolean transport): the push
      # classify step publishes boundary_required (true/false) computed from
      # the ACCEPTED BASE policy's trigger function, never the path inventory.
      # The runtime cases' ranges hold src/run.rs (not a boundary consumer),
      # so the boolean must be false. A PR editing the trigger function cannot
      # change its own decision.
      if ($Case[2] -ceq 'runtime') {
        Assert-Policy ($Lines -ccontains 'boundary_required=false') 'push classify publishes boundary_required=false for non-consumer range'
      }
    }
    $env:HUM_CI_BASE_SHA=$Accepted;$env:HUM_CI_HEAD_SHA=$Ordinary;$global:HumCiTestHealthConclusion='failure'
    $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Ordinary)
    [IO.File]::WriteAllText($env:GITHUB_OUTPUT,'')
    Assert-PolicyRejects { & { $Mode='full'; . ([scriptblock]::Create($PushCode)) } } 'actual normal push rejects failed health'
    Assert-Policy ([IO.File]::ReadAllText($env:GITHUB_OUTPUT).Length -eq 0) 'failed health publishes no selected mode'
    Assert-Policy (@(Get-ChildItem -LiteralPath $Fixture -Filter 'hum-accepted-push-policy-*').Count -eq 0) 'accepted-policy disposable files removed'
  } finally {
    Remove-HumCiTestGhMock $GhState
    foreach($Key in $SavedPush.Keys){[Environment]::SetEnvironmentVariable($Key,$SavedPush[$Key],'Process')}
    Pop-Location
  }
} finally {
  $Temp=[IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([IO.Path]::DirectorySeparatorChar)+[IO.Path]::DirectorySeparatorChar
  if(-not $Fixture.StartsWith($Temp,[StringComparison]::OrdinalIgnoreCase)){throw 'ci_policy_test: cleanup ownership'}
  Remove-Item -LiteralPath $Fixture -Recurse -Force
}
Assert-Policy (-not(Test-Path -LiteralPath $Fixture)) 'owned Git fixture removed'
$Ci = [IO.File]::ReadAllText((Join-Path $Root '.github/workflows/ci.yml')).Replace(([string][char]13+[char]10),[string][char]10)
$Source = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'check_all.ps1')).Replace(([string][char]13+[char]10),[string][char]10)
foreach($Yaml in @($Workflow,$Ci)) {
  $Blocks=@([regex]::Matches($Yaml,'(?m)^        run: \|\n(?<body>(?:          [^\n]*\n|\n)*)'))
  Assert-Policy ($Blocks.Count -gt 0) 'workflow script owners found'
  foreach($Block in $Blocks){
    $Body=($Block.Groups['body'].Value -split "`n"|ForEach-Object{if($_.StartsWith('          ')){$_.Substring(10)}else{$_}})-join "`n"
    # Existing push-only script contains these GitHub template substitutions.
    # This checks shell syntax after representative interpolation, not API access.
    $Body=$Body.Replace('${{ github.repository }}','owner/repo')
    $Body=$Body.Replace('${{ github.workspace }}','/workspace/repo')
    $Tokens=$null;$Errors=$null
    $null=[Management.Automation.Language.Parser]::ParseInput($Body,[ref]$Tokens,[ref]$Errors)
    Assert-Policy ($Errors.Count -eq 0) 'actual workflow PowerShell body parses'
  }
}
# Exercise the actual aggregate's script, not a duplicate acceptance validator.
$AggregateMatch = [regex]::Match($Workflow, '(?ms)^      - name: Require explicit success\n.*?        run: \|\n(?<body>.*)\z')
Assert-Policy $AggregateMatch.Success 'aggregate owner exists'
$AggregateSource = ($AggregateMatch.Groups['body'].Value -split "`n" | ForEach-Object { if ($_.StartsWith('          ')) { $_.Substring(10) } else { $_ } }) -join "`n"
$Aggregate = [scriptblock]::Create($AggregateSource)
$OriginalNeeds = [Environment]::GetEnvironmentVariable('HUM_CI_NEEDS','Process')
$SavedProvenance = @{}
foreach($Key in @('GITHUB_SHA','GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT','GITHUB_EVENT_NAME')) { $SavedProvenance[$Key]=[Environment]::GetEnvironmentVariable($Key,'Process') }
try {
  $env:GITHUB_SHA='a'*40;$env:GITHUB_RUN_ID='42';$env:GITHUB_RUN_ATTEMPT='1';$env:GITHUB_EVENT_NAME='pull_request'
  foreach ($Outcome in @('success','failure','cancelled','skipped','timed_out','')) {
    $Needs = [ordered]@{ plan = @{ result='success'; outputs=@{profile='runtime';integration=('a'*40);tree=('b'*40);base=('c'*40);head=('d'*40);run='42';attempt='1'} }; evaluate = @{result=$Outcome} }
    $env:HUM_CI_NEEDS = $Needs | ConvertTo-Json -Depth 6 -Compress
    if ($Outcome -ceq 'success') { & $Aggregate; Assert-Policy $true 'honest aggregate' }
    else { Assert-PolicyRejects $Aggregate "aggregate $Outcome" }
  }
  foreach ($Mutation in @('missing','extra','plan-failed','tree-missing','profile-missing','head-missing','stale-attempt','stale-integration')) {
    $Needs = @{ plan = @{ result='success'; outputs=@{profile='runtime';integration=('a'*40);tree=('b'*40);base=('c'*40);head=('d'*40);run='42';attempt='1'} }; evaluate = @{result='success'} }
    switch ($Mutation) {
      'missing' { $Needs.Remove('evaluate') }
      'extra' { $Needs.extra = @{ result='success' } }
      'plan-failed' { $Needs.plan.result = 'failure' }
      'tree-missing' { $Needs.plan.outputs.Remove('tree') }
      'profile-missing' { $Needs.plan.outputs.Remove('profile') }
      'head-missing' { $Needs.plan.outputs.Remove('head') }
      'stale-attempt' { $Needs.plan.outputs.attempt='2' }
      'stale-integration' { $Needs.plan.outputs.integration='e'*40 }
    }
    $env:HUM_CI_NEEDS = $Needs | ConvertTo-Json -Depth 6 -Compress
    Assert-PolicyRejects $Aggregate "aggregate $Mutation"
  }
} finally {
  [Environment]::SetEnvironmentVariable('HUM_CI_NEEDS',$OriginalNeeds,'Process')
  foreach($Key in $SavedProvenance.Keys) { [Environment]::SetEnvironmentVariable($Key,$SavedProvenance[$Key],'Process') }
}

# The per-platform completion owner checks actual steps, not just a green job.
$CompletionMatch=[regex]::Match($Ci,'(?ms)^      - name: Confirm selected work completion\n.*?        run: \|\n(?<body>.*)\z')
Assert-Policy $CompletionMatch.Success 'platform completion owner exists'
$Completion=[scriptblock]::Create((($CompletionMatch.Groups['body'].Value -split "`n"|ForEach-Object{if($_.StartsWith('          ')){$_.Substring(10)}else{$_}})-join "`n"))
$SavedCompletion=@{}
foreach($Key in @('HUM_COMPLETED_STEPS','HUM_SELECTED_MODE','HUM_SELECTED_PROFILE','HUM_MATRIX_OS','GITHUB_EVENT_NAME')){$SavedCompletion[$Key]=[Environment]::GetEnvironmentVariable($Key,'Process')}
try {
  foreach($Platform in @('windows-latest','ubuntu-latest')) {
    foreach($Profile in @('language','runtime','compiler','full')) {
      $env:HUM_MATRIX_OS=$Platform;$env:HUM_SELECTED_PROFILE=$Profile;$env:GITHUB_EVENT_NAME='pull_request'
      $env:HUM_SELECTED_MODE=if($Profile -ceq 'full'){'full'}else{'normal'}
      $Rows=@{};foreach($Name in @('integration_identity','pwsh_identity','classify','normal_profile','full_preflight','close_full_transport','exhaustive')){$Rows[$Name]=@{outcome='success';conclusion='success'}}
      $env:HUM_COMPLETED_STEPS=$Rows|ConvertTo-Json -Depth 5 -Compress
      & $Completion
      Assert-Policy $true "honest $Platform/$Profile completion"
      $Required=@('integration_identity','pwsh_identity','classify')
      $Required+=if($Profile -ceq 'full'){@('full_preflight','close_full_transport')}else{@('normal_profile')}
      if($Platform -ceq 'ubuntu-latest'){$Required+='exhaustive'}
      foreach($Name in $Required){
        foreach($Outcome in @('failure','skipped','cancelled','')){
          $Rows[$Name].outcome=$Outcome
          $env:HUM_COMPLETED_STEPS=$Rows|ConvertTo-Json -Depth 5 -Compress
          Assert-PolicyRejects $Completion "masked or missing $Platform/$Profile/$Name/$Outcome"
        }
        $Rows[$Name].outcome='success'
      }
      $env:HUM_SELECTED_MODE='unknown'
      Assert-PolicyRejects $Completion 'unknown mode cannot omit execution'
    }
  }
} finally {foreach($Key in $SavedCompletion.Keys){[Environment]::SetEnvironmentVariable($Key,$SavedCompletion[$Key],'Process')}}

$Now = [datetimeoffset]'2026-09-20T12:00:00Z'
function New-HealthRun {
  [pscustomobject]@{ event='schedule';path='.github/workflows/validation.yml';repository=@{full_name='owner/repo'};head_branch='main';head_sha=('a'*40);status='completed';conclusion='success';id=42;run_attempt=1;created_at='2026-09-20T10:00:00Z' }
}
$Run = New-HealthRun
$null = Assert-HumIntegrationHealth @($Run) $Now 'owner/repo'
Assert-Policy $true 'honest health'
foreach ($Field in @('conclusion','status','head_branch','head_sha','path','event','repository','id','run_attempt','created_at')) {
  $Bad = New-HealthRun
  switch ($Field) {
    'repository' { $Bad.repository = @{full_name='foreign/repo'} }
    'id' { $Bad.id = 0 }
    'run_attempt' { $Bad.run_attempt = 0 }
    'created_at' { $Bad.created_at = '2026-09-18T01:00:00Z' }
    default { $Bad.$Field = 'invalid' }
  }
  Assert-PolicyRejects { Assert-HumIntegrationHealth @($Bad) $Now 'owner/repo' } "health $Field"
}
$Red = New-HealthRun; $Red.created_at='2026-09-20T11:00:00Z';$Red.conclusion='failure'
Assert-PolicyRejects { Assert-HumIntegrationHealth @($Run,$Red) $Now 'owner/repo' } 'latest failure must not fall back to older success'
Assert-PolicyRejects { Assert-HumIntegrationHealth @() $Now 'owner/repo' } 'missing health'
$Future = New-HealthRun; $Future.created_at='2026-09-21T10:00:00Z'
Assert-PolicyRejects { Assert-HumIntegrationHealth @($Future) $Now 'owner/repo' } 'future health'
$Dispatch = New-HealthRun; $Dispatch.event='workflow_dispatch'
$null = Assert-HumIntegrationHealth @($Dispatch) $Now 'owner/repo'
Assert-Policy $true 'dispatch health accepted'
$Older = New-HealthRun; $Older.created_at='2026-09-20T09:00:00Z'
$Newer = New-HealthRun; $Newer.event='workflow_dispatch'; $Newer.created_at='2026-09-20T11:00:00Z'
$Picked = Assert-HumIntegrationHealth @($Older,$Newer) $Now 'owner/repo'
Assert-Policy ($Picked.event -ceq 'workflow_dispatch') 'latest dispatch wins over older schedule'
$Dead = New-HealthRun; $Dead.event='workflow_dispatch'; $Dead.created_at='2026-09-20T11:00:00Z'; $Dead.conclusion='failure'
Assert-PolicyRejects { Assert-HumIntegrationHealth @($Run,$Dead) $Now 'owner/repo' } 'latest dispatch failure must not fall back to older schedule success'
$Push = New-HealthRun; $Push.event='push'
Assert-PolicyRejects { Assert-HumIntegrationHealth @($Push) $Now 'owner/repo' } 'push event never mints health'
function New-HealthJobs {
  foreach ($Platform in @('windows','ubuntu')) {
    $Names=@('Checkout','Verify integration identity','Run Hum preflight','Close full evidence transport','Confirm selected work completion')
    if ($Platform -ceq 'ubuntu') { $Names += 'Run exhaustive canonical-seal evidence' }
    [pscustomobject]@{name="evaluate / preflight ($Platform-latest)";run_id=42;head_sha=('a'*40);status='completed';conclusion='success';steps=@($Names|ForEach-Object{[pscustomobject]@{name=$_;status='completed';conclusion='success'}})}
  }
}
$Jobs=@(New-HealthJobs);Assert-HumIntegrationJobs $Jobs $Run;Assert-Policy $true 'both health platforms'
Assert-PolicyRejects { Assert-HumIntegrationJobs @($Jobs[0]) $Run } 'missing Ubuntu'
Assert-PolicyRejects { Assert-HumIntegrationJobs @($Jobs + $Jobs[0]) $Run } 'duplicate Windows'
foreach ($Platform in 0,1) {
  foreach ($Index in 0..($Jobs[$Platform].steps.Count-1)) {
    $Bad=@(New-HealthJobs);$Bad[$Platform].steps[$Index].conclusion='skipped'
    Assert-PolicyRejects { Assert-HumIntegrationJobs $Bad $Run } 'missing required nightly execution'
  }
}

# Actual dispatcher with substituted expensive groups: routing/failure evidence,
# not compiler, Full, mutation or complete-profile execution credit.
. (Import-PolicyTestFunction $Source 'Invoke-HumFixedProfile')
function Invoke-HumCoreCheck([string]$Group,[string]$Cargo) { $script:Observed.Add($Group); if($script:FailAt -ceq $Group){throw 'owned group failure'} }
function Invoke-HumCaptureSmoke { $script:Observed.Add('capture') }
function Invoke-HumLanguageProgramChecks { $script:Observed.Add('language-programs') }
function Invoke-HumRuntimeProgramChecks { $script:Observed.Add('runtime-programs') }
function Invoke-HumCompilerPrivacyChecks { $script:Observed.Add('compiler-privacy') }
function Invoke-HumCompilerFrontChecks { $script:Observed.Add('compiler-front') }
function Invoke-HumCompilerCorpusChecks { $script:Observed.Add('compiler-corpus') }
function Invoke-Wo22UnsafeBoundaryCompilerEvidence { $script:Observed.Add('unsafe') }
function Invoke-Wo22BackendPredicateMutationEvidence { $script:Observed.Add('backend-mutations') }
function Invoke-Wo23UnitAProductionMutationEvidence { $script:Observed.Add('integer-mutations') }
function Invoke-Wo24UnitAProductionMutationEvidence { $script:Observed.Add('text-mutations') }
# Regression: the capture group must bind -ProfileSmokeOnly as a literal
# switch on the real script. A [string[]] splat like @('-ProfileSmokeOnly')
# binds positionally into the script's $ShellContract ValidateSet and fails
# on every host; the old Invoke-RepoScript mock hid this by intercepting the
# call before binding ever happened.
$CaptureFn = (Import-PolicyTestFunction $Source 'Invoke-HumCaptureSmoke').ToString()
$CapTokens = $null; $CapErrors = $null
$CapAst = [Management.Automation.Language.Parser]::ParseInput($CaptureFn, [ref]$CapTokens, [ref]$CapErrors)
Assert-Policy ($CapErrors.Count -eq 0) 'capture smoke body parses'
$CapCalls = @($CapAst.FindAll({ param($Node) $Node -is [Management.Automation.Language.CommandAst] -and $Node.CommandElements.Count -gt 0 -and $Node.CommandElements[0].Extent.Text -like '*test_fast_evidence_capture.ps1*' }, $true))
Assert-Policy ($CapCalls.Count -eq 1) 'capture smoke invokes the capture script once'
$CapSwitch = @($CapCalls[0].CommandElements | Where-Object { $_ -is [Management.Automation.Language.CommandParameterAst] -and $_.ParameterName -ceq 'ProfileSmokeOnly' })
Assert-Policy ($CapSwitch.Count -eq 1) 'capture smoke binds literal -ProfileSmokeOnly switch'
$CapSplat = @($CapCalls[0].CommandElements | Where-Object { $_ -is [Management.Automation.Language.VariableExpressionAst] -and $_.Splatted })
Assert-Policy ($CapSplat.Count -eq 0) 'capture smoke has no string-array splat on the call path'
# Real-path regression: execute the actual Invoke-HumCaptureSmoke body against
# the real capture script. The AST checks above only guard the source shape;
# this exercises the exact binding path that failed under the old splat.
. (Import-PolicyTestFunction $Source 'Invoke-HumCaptureSmoke')
$global:LASTEXITCODE = 0
Invoke-HumCaptureSmoke -ToolsDir (Join-Path $Root 'tools')
Assert-Policy ($LASTEXITCODE -eq 0) 'real capture smoke executes with exit 0'
function Invoke-HumCaptureSmoke { $script:Observed.Add('capture') }
$RepoRoot=$Root
$OldReceipt=[Environment]::GetEnvironmentVariable('HUM_EVIDENCE_RECEIPT','Process')
try {
  [Environment]::SetEnvironmentVariable('HUM_EVIDENCE_RECEIPT',$null,'Process')
  foreach($Profile in @('Language','Runtime','Compiler')) {
    $script:Observed=New-Object 'System.Collections.Generic.List[string]';$script:FailAt=''
    Invoke-HumFixedProfile $Profile 'owned-cargo'
    $Expected=@('capture','format','check','tests','clippy','build')
    if($Profile -ceq 'Compiler'){$Expected+=@('compiler-front','compiler-corpus')}
    else {$Expected+='language-programs';if($Profile -ceq 'Runtime'){$Expected+='runtime-programs'}}
    $Expected+='hygiene'
    Assert-Policy (($Observed -join ',') -ceq ($Expected -join ',')) "actual $Profile route closure"
    if($Profile -ceq 'Compiler') {
      $Dispatcher=(Import-PolicyTestFunction $Source 'Invoke-HumFixedProfile').ToString()
      foreach($Removed in @('compiler-front','compiler-corpus')) {
        $Call=if($Removed -ceq 'compiler-front'){'. Invoke-HumCompilerFrontChecks $Cargo $Hum'}else{'. Invoke-HumCompilerCorpusChecks $Cargo $Hum'}
        $Broken=$Dispatcher.Replace($Call,'')
        Assert-Policy ($Broken -cne $Dispatcher) 'initialized shared group omission'
        $script:Observed.Clear()
        & { . ([scriptblock]::Create($Broken)); Invoke-HumFixedProfile 'Compiler' 'owned-cargo' }
        Assert-Policy (($Observed -join ',') -cne ($Expected -join ',')) "omitted $Removed cannot satisfy closure"
      }
    }
  }
  # Execute the actual top-level dispatch body with bounded group substitutes.
  # Removing its normal return must hit the blocked Full entry, not earn credit.
  $Tokens=$null;$Errors=$null
  $Ast=[Management.Automation.Language.Parser]::ParseInput($Source,[ref]$Tokens,[ref]$Errors)
  $Main=@($Ast.EndBlock.Statements|Where-Object{$_ -is [Management.Automation.Language.TryStatementAst]})
  Assert-Policy ($Main.Count -eq 1) 'unique main dispatch owner'
  $MainBody=$Main[0].Body.Extent.Text
  $MainCode=$MainBody.Substring(1,$MainBody.Length-2)
  $Guard="if (`$EvidenceTier -cin @('Language', 'Runtime', 'Compiler')) { Invoke-HumFixedProfile `$EvidenceTier `$Cargo; return }"
  Assert-Policy ($MainCode.Contains($Guard)) 'normal return is owned by actual dispatch'
  function Invoke-RepoScript {
    param($Label,$Path,$Arguments)
    throw 'ci_policy_test: blocked Full fallback'
  }
  foreach($EvidenceTier in @('Language','Runtime','Compiler')) {
    $script:Observed.Clear();$script:FailAt='';$Cargo='owned-cargo'
    & ([scriptblock]::Create($MainCode))
    Assert-Policy ($Observed.Contains('tests')) 'top-level route reached compiler tests'
    $script:Observed.Clear()
    Assert-PolicyRejects ([scriptblock]::Create($MainCode.Replace($Guard,''))) 'omitted dispatch rejects before Full execution'
    Assert-Policy ($Observed.Count -eq 0) 'omitted route gained no selected-work credit'
  }
  $script:Observed.Clear();$script:FailAt='tests'
  Assert-PolicyRejects { Invoke-HumFixedProfile 'Runtime' 'owned-cargo' } 'group failure propagates'
  Assert-Policy (($Observed -join ',') -ceq 'capture,format,check,tests') 'no work after failure'
  $env:HUM_EVIDENCE_RECEIPT='rejected-full-receipt'
  $script:Observed.Clear()
  Assert-PolicyRejects { Invoke-HumFixedProfile 'Language' 'owned-cargo' } 'normal cannot write Full receipt'
  Assert-Policy ($Observed.Count -eq 0) 'receipt rejection precedes all work'
} finally { [Environment]::SetEnvironmentVariable('HUM_EVIDENCE_RECEIPT',$OldReceipt,'Process') }

foreach($Name in @('Invoke-HumCoreCheck','Invoke-HumRuntimeProgramChecks','Invoke-HumCompilerPrivacyChecks','Invoke-HumLanguageProgramChecks','Invoke-HumCompilerFrontChecks','Invoke-HumCompilerCorpusChecks')) {
  $null=Import-PolicyTestFunction $Source $Name
  Assert-Policy ([regex]::Matches($Source,"(?m)^  (?:\. )?$Name ").Count -ge 1) "Full consumes shared $Name"
}
# These pins protect the mechanically shared bodies from silent omissions.
# They are source-closure evidence, never a claim that the full corpus ran.
$CompilerBodies=@{
  'Invoke-HumCompilerFrontChecks'='4d532cc6335d66a86d0de23f0cb9e4803d4e1b8119052faa6a0248d6f36ae26c'
  'Invoke-HumCompilerCorpusChecks'='9207434165c1d693e85a83a554c2d9c5031faa14b3cfc0d9bdac4c69cb56a6cf'
  'Invoke-HumUseAfterMoveRuntimeCheck'='a7c5db7146519cba950ec4ba2de9a0f15bf505bbfde0bc855b2e49c9a74a34f8'
  'Invoke-HumUseAfterMoveProjectionCheck'='d1708df1234a0cbdf4f686d48facad32ccfb2bcc7baa834281d2246b748f41f0'
}
foreach($Name in $CompilerBodies.Keys){
  $Definition=(Import-PolicyTestFunction $Source $Name).ToString()
  $Tokens=$null;$Errors=$null;$Ast=[Management.Automation.Language.Parser]::ParseInput($Definition,[ref]$Tokens,[ref]$Errors)
  $Function=@($Ast.EndBlock.Statements)[0]
  $Body=$Function.Body.Extent.Text
  $Hasher=[Security.Cryptography.SHA256]::Create()
  try {
    $Digest=-join($Hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($Body))|ForEach-Object{$_.ToString('x2')})
    $ExpectedDigest = $CompilerBodies[$Name]
    Assert-Policy ($Digest -ceq $ExpectedDigest) "complete shared body ${Name} changed: expected ${ExpectedDigest}, found ${Digest}. If the change is intended, update the pin in the `$CompilerBodies table in tools/test_ci_policy.ps1"
    $First=@($Function.Body.EndBlock.Statements)[0].Extent.Text
    $Omitted=$Body.Replace($First,'')
    $BadDigest=-join($Hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($Omitted))|ForEach-Object{$_.ToString('x2')})
    Assert-Policy ($Omitted -cne $Body -and $BadDigest -cne $CompilerBodies[$Name]) "initialized body omission $Name"
  } finally {$Hasher.Dispose()}
}
foreach($Name in @('Get-Wo25Sha256','Assert-Wo25EvidenceTierDispatcherContract','Assert-Wo25UnitBFullPreflightWorkflowRoute','Get-Wo25ExhaustiveWorkflowRouteFailure')) {
  . (Import-PolicyTestFunction $Source $Name)
}
Assert-Wo25EvidenceTierDispatcherContract $Source -SkipStaleControl
Assert-Wo25UnitBFullPreflightWorkflowRoute $Ci
Assert-Policy ($null -eq (Get-Wo25ExhaustiveWorkflowRouteFailure $Ci)) 'Full exhaustive route unchanged'
Assert-Policy ($Workflow.Contains('if: always()') -and $Workflow.Contains('needs: [plan, evaluate]')) 'aggregate runs after failure'
Assert-Policy ($Workflow.Contains('git show "$($AcceptedBase):tools/check_ci_policy.ps1"')) 'accepted-base policy selection'
Assert-Policy ($Workflow -notmatch 'git show "\$\(\$env:HUM_BASE\):tools/check_ci_policy\.ps1"') 'stale event base never feeds the policy read'
Assert-Policy ($Workflow -notmatch '(?m)^\s*(checks|statuses|id-token):\s*write\s*$' -and $Workflow -notmatch 'pull_request_target:') 'no privileged publisher'
foreach ($Step in @('Generate evidence summary','Upload evidence summary','Upload hum-dev executable')) {
  $Block=[regex]::Match($Ci,'(?ms)^      - name: '+[regex]::Escape($Step)+'\n.*?(?=^      - name: |\z)').Value
  Assert-Policy ($Block.Contains("if: steps.classify.outputs.mode == 'full' && github.event_name == 'push'")) 'push-only anchor contract'
}
# Regression: the ci.yml push classify step must reset $LASTEXITCODE after the
# accepted-policy probe. A failed git show (bootstrap: base without the policy)
# leaves a non-zero native exit code; GitHub fails the step when $LASTEXITCODE
# is non-zero at step end even though the bootstrap branch handles the case
# explicitly. This extracts the real probe block from ci.yml and executes it
# against a policy-less base, asserting exit 0 and the bootstrap path.
$ClassifyStep=[regex]::Match($Ci,'(?ms)^      - name: Classify CI evidence lane\n.*?(?=^      - name: |\z)').Value
Assert-Policy ($ClassifyStep.Length -gt 0) 'classify step found in ci.yml'
# Decision 0025 amendment (2026-09-23, boolean transport): on workflow_call
# the classify step must forward the reusable boundary_required input (the
# boolean published by validation.yml's plan step from the accepted base
# policy) to its boundary_required output; the fixed profiles read
# steps.classify.outputs.boundary_required. The forwarding is workflow_call-
# gated so it cannot overwrite the push-published boolean output (step outputs
# are last-wins); an empty input stays empty and fails safe downstream.
Assert-Policy ($ClassifyStep -match 'HUM_CI_BOUNDARY_REQUIRED_INPUT: \$\{\{ inputs\.boundary_required \}\}') 'classify maps the reusable boundary_required input'
Assert-Policy ($ClassifyStep -match "(?s)if \(\`$env:HUM_CI_EVENT_NAME -ceq 'workflow_call'\) \{.*?boundary_required=\`$env:HUM_CI_BOUNDARY_REQUIRED_INPUT") 'classify forwards the boundary_required input on workflow_call only'
$ProbeStart=$ClassifyStep.IndexOf('$SavedPreference = $ErrorActionPreference')
$ProbeEndMarker="} else { Write-Host 'Accepted pre-push policy unavailable: Full bootstrap required.' }"
$ProbeEnd=$ClassifyStep.IndexOf($ProbeEndMarker)
Assert-Policy (($ProbeStart -ge 0) -and ($ProbeEnd -gt $ProbeStart)) 'classify probe block located in ci.yml'
$ProbeBlock=$ClassifyStep.Substring($ProbeStart,$ProbeEnd-$ProbeStart+$ProbeEndMarker.Length)
Assert-Policy ($ProbeBlock -match '\$global:LASTEXITCODE = 0') 'classify probe resets LASTEXITCODE after capture'
$OldProbeBase=[Environment]::GetEnvironmentVariable('HUM_CI_BASE_SHA','Process')
Push-Location $Root
try {
  [Environment]::SetEnvironmentVariable('HUM_CI_BASE_SHA','3cd2e8c0d3abdaf7786d710c39891409128cb17e','Process')
  $global:LASTEXITCODE=0
  & ([scriptblock]::Create($ProbeBlock+"`n`$script:ProbePolicyExit = `$PolicyExit"))
  Assert-Policy ($script:ProbePolicyExit -ne 0) 'classify probe takes bootstrap path on policy-less base'
  Assert-Policy ($LASTEXITCODE -eq 0) 'classify probe leaves LASTEXITCODE 0 (no leak)'
} finally {
  [Environment]::SetEnvironmentVariable('HUM_CI_BASE_SHA',$OldProbeBase,'Process')
  Pop-Location
}
# Regression: the ci.yml classify step dot-sourced the accepted policy script,
# whose param() block ($Mode defaults to 'Library') overwrote the step's $Mode.
# The completion check then saw HUM_SELECTED_MODE=Library and threw, failing
# every main push once the base contained the policy. The fix loads the policy
# in a child scope. This extracts the real policy-load block from ci.yml and
# executes it against a policy-bearing base, asserting the step's $Mode is not
# clobbered. Fails on the pre-fix ci.yml.
$LoadStart=$ClassifyStep.IndexOf('$PolicyPath = Join-Path $env:RUNNER_TEMP')
$LoadEndMarker="} else { Write-Host 'Accepted pre-push policy unavailable: Full bootstrap required.' }"
$LoadEnd=$ClassifyStep.IndexOf($LoadEndMarker)
Assert-Policy (($LoadStart -ge 0) -and ($LoadEnd -gt $LoadStart)) 'classify policy-load block located in ci.yml'
$LoadBlock=$ClassifyStep.Substring($LoadStart,$LoadEnd-$LoadStart).TrimEnd()
Assert-Policy ($LoadBlock -match '(?s)& \{\s*\. \$PolicyPath') 'classify policy loads in a child scope'
$ScopeFixture=[IO.Path]::GetFullPath((Join-Path ([IO.Path]::GetTempPath()) ('hum-ci-scope-'+[Guid]::NewGuid().ToString('N'))))
if(Test-Path -LiteralPath $ScopeFixture){throw 'ci_policy_test: fixture collision'}
[IO.Directory]::CreateDirectory($ScopeFixture)|Out-Null
$ScopeIdentity=@('-c','user.name=HumPolicyFixture','-c','user.email=fixture@example.invalid')
$OldScopeEnv=@{}
$ScopeSummary=$null
try {
  $null=Read-HumCiGit $ScopeFixture @('init','-q','-b','main')
  [IO.Directory]::CreateDirectory((Join-Path $ScopeFixture 'tools'))|Out-Null
  [IO.File]::Copy((Join-Path $PSScriptRoot 'check_ci_policy.ps1'),(Join-Path $ScopeFixture 'tools/check_ci_policy.ps1'))
  # Decision 0025: the fixture change lives under the tools/ hygiene-script
  # pin (check_text_hygiene.ps1 selects language rank), so production selects
  # the language profile and the classify step's health gate runs. The fixture
  # installs the isolated gh mock; the regression under test is that the
  # child-scope policy load does not clobber the step's $Mode.
  [IO.File]::WriteAllText((Join-Path $ScopeFixture 'tools/check_text_hygiene.ps1'),"base`n")
  $null=Read-HumCiGit $ScopeFixture @('add','--','tools/check_ci_policy.ps1','tools/check_text_hygiene.ps1')
  $ScopeTreeA=(Read-HumCiGit $ScopeFixture @('write-tree')).Trim()
  $ScopeBase=(Read-HumCiGit $ScopeFixture ($ScopeIdentity+@('commit-tree',$ScopeTreeA,'-m','policy-base'))).Trim()
  [IO.File]::WriteAllText((Join-Path $ScopeFixture 'tools/check_text_hygiene.ps1'),"changed`n")
  $null=Read-HumCiGit $ScopeFixture @('add','--','tools/check_text_hygiene.ps1')
  $ScopeTreeB=(Read-HumCiGit $ScopeFixture @('write-tree')).Trim()
  $ScopeHead=(Read-HumCiGit $ScopeFixture ($ScopeIdentity+@('commit-tree',$ScopeTreeB,'-p',$ScopeBase,'-m','push-head'))).Trim()
  $null=Read-HumCiGit $ScopeFixture @('update-ref','HEAD',$ScopeHead)
  $null=Read-HumCiGit $ScopeFixture @('checkout','-q','--detach',$ScopeHead)
  foreach ($Name in @('HUM_CI_BASE_SHA','HUM_CI_HEAD_SHA','RUNNER_TEMP','GITHUB_STEP_SUMMARY','GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT','GITHUB_REPOSITORY')) {
    $OldScopeEnv[$Name]=[Environment]::GetEnvironmentVariable($Name,'Process')
  }
  [Environment]::SetEnvironmentVariable('HUM_CI_BASE_SHA',$ScopeBase,'Process')
  [Environment]::SetEnvironmentVariable('HUM_CI_HEAD_SHA',$ScopeHead,'Process')
  [Environment]::SetEnvironmentVariable('RUNNER_TEMP',[IO.Path]::GetTempPath(),'Process')
  [Environment]::SetEnvironmentVariable('GITHUB_REPOSITORY','owner/repo','Process')
  $ScopeSummary=Join-Path ([IO.Path]::GetTempPath()) ('hum-ci-scope-summary-'+[Guid]::NewGuid().ToString('N')+'.md')
  [Environment]::SetEnvironmentVariable('GITHUB_STEP_SUMMARY',$ScopeSummary,'Process')
  [Environment]::SetEnvironmentVariable('GITHUB_RUN_ID','0','Process')
  [Environment]::SetEnvironmentVariable('GITHUB_RUN_ATTEMPT','1','Process')
  Push-Location $ScopeFixture
  try {
    $Mode='full'
    $SelectedProfile='full'
    $Policy=git -C $ScopeFixture show "$($ScopeBase):tools/check_ci_policy.ps1" 2>$null
    Assert-Policy ($LASTEXITCODE -eq 0) 'scope fixture policy readable'
    $ScopeGhState = Install-HumCiTestGhMock
    try {
      & ([scriptblock]::Create($LoadBlock+"`n`$script:ScopeModeAfter = `$Mode`n`$script:ScopeProfileAfter = `$SelectedProfile"))
    } finally { Remove-HumCiTestGhMock $ScopeGhState }
    Assert-Policy ($script:ScopeModeAfter -ceq 'full') 'classify policy load leaves step $Mode at full (not Library)'
    Assert-Policy ($script:ScopeProfileAfter -ceq 'language') 'classify push selection honestly selects language for the owned-prefix change'
    Assert-Policy ($LASTEXITCODE -eq 0) 'classify policy load leaves LASTEXITCODE 0'
  } finally { Pop-Location }
} finally {
  foreach ($Entry in $OldScopeEnv.GetEnumerator()) { [Environment]::SetEnvironmentVariable($Entry.Key,$Entry.Value,'Process') }
  if (Test-Path -LiteralPath $ScopeFixture) { Remove-Item -LiteralPath $ScopeFixture -Recurse -Force }
  if ($ScopeSummary -and (Test-Path -LiteralPath $ScopeSummary)) { Remove-Item -LiteralPath $ScopeSummary -Force }
}
# Regression: validation.yml's plan step compared the test merge commit's parents
# against the event's pull_request.base.sha, which goes stale when the target
# branch moves after GitHub builds the merge commit. Every PR open across a
# merge then failed with 'integration parents differ', and re-running could not
# help. The fix derives the accepted base from the merge commit's first parent,
# verifies the second parent is the event head and the base is on the target
# branch. This extracts the real plan block from validation.yml and executes it
# with a stale event base behind the merge commit's first parent, asserting it
# passes and reports the verified base. Fails on the pre-fix validation.yml.
$PlanStepAt=$Workflow.IndexOf('      - name: Select accepted-policy route')
Assert-Policy ($PlanStepAt -ge 0) 'plan step found in validation.yml'
$PlanRunMarker="`n        run: |`n"
$PlanRunAt=$Workflow.IndexOf($PlanRunMarker,$PlanStepAt)
Assert-Policy ($PlanRunAt -ge 0) 'plan run block found in validation.yml'
$PlanBodyAt=$PlanRunAt+$PlanRunMarker.Length
$PlanLines=$Workflow.Substring($PlanBodyAt) -split "`n"
$PlanBody=@()
foreach ($PlanLine in $PlanLines) {
  if ($PlanLine -match '^          (.*)$') { $PlanBody+=$Matches[1] }
  elseif ($PlanLine -eq '') { $PlanBody+='' }
  else { break }
}
$PlanBlock=$PlanBody -join "`n"
Assert-Policy ($PlanBlock -match 'ParentIds\[1\]') 'plan derives the accepted base from the merge commit'
$PlanFixture=[IO.Path]::GetFullPath((Join-Path ([IO.Path]::GetTempPath()) ('hum-ci-plan-'+[Guid]::NewGuid().ToString('N'))))
$PlanRemote="$PlanFixture-remote.git"
if((Test-Path -LiteralPath $PlanFixture) -or (Test-Path -LiteralPath $PlanRemote)){throw 'ci_policy_test: fixture collision'}
[IO.Directory]::CreateDirectory($PlanFixture)|Out-Null
$PlanIdentity=@('-c','user.name=HumPolicyFixture','-c','user.email=fixture@example.invalid')
$OldPlanEnv=@{}
$PlanOutput=$null
$PlanSummary=$null
try {
  $null=Read-HumCiGit $PlanFixture @('init','-q','-b','main')
  [IO.Directory]::CreateDirectory((Join-Path $PlanFixture 'tools'))|Out-Null
  [IO.File]::Copy((Join-Path $PSScriptRoot 'check_ci_policy.ps1'),(Join-Path $PlanFixture 'tools/check_ci_policy.ps1'))
  # Decision 0025: the fixture change uses the tools/ hygiene-script pin
  # (check_text_hygiene.ps1 selects language rank), so production selects the
  # language profile and the plan step's health gate runs. The fixture
  # installs the isolated gh mock; the regression under test is that the
  # verified merge-parent base is used, not the stale event base.
  [IO.File]::WriteAllText((Join-Path $PlanFixture 'tools/check_text_hygiene.ps1'),"base`n")
  $null=Read-HumCiGit $PlanFixture @('add','--','tools/check_ci_policy.ps1','tools/check_text_hygiene.ps1')
  $PlanTreeA=(Read-HumCiGit $PlanFixture @('write-tree')).Trim()
  # A is the stale event base: the main tip the PR was opened against.
  $PlanStaleBase=(Read-HumCiGit $PlanFixture ($PlanIdentity+@('commit-tree',$PlanTreeA,'-m','stale-base'))).Trim()
  [IO.File]::WriteAllText((Join-Path $PlanFixture 'tools/check_text_hygiene.ps1'),"main tip`n")
  $null=Read-HumCiGit $PlanFixture @('add','--','tools/check_text_hygiene.ps1')
  $PlanTreeB=(Read-HumCiGit $PlanFixture @('write-tree')).Trim()
  # B is the current main tip: the merge commit's first parent.
  $PlanTip=(Read-HumCiGit $PlanFixture ($PlanIdentity+@('commit-tree',$PlanTreeB,'-p',$PlanStaleBase,'-m','main-tip'))).Trim()
  $null=Read-HumCiGit $PlanFixture @('update-ref','refs/heads/main',$PlanTip)
  $null=Read-HumCiGit $PlanFixture @('init','--bare','-q',$PlanRemote)
  $null=Read-HumCiGit $PlanFixture @('remote','add','origin',$PlanRemote)
  $null=Read-HumCiGit $PlanFixture @('push','-q','origin','main')
  # H is the PR head, branched from the stale base.
  $null=Read-HumCiGit $PlanFixture @('read-tree',$PlanTreeA)
  [IO.File]::WriteAllText((Join-Path $PlanFixture 'tools/check_text_hygiene.ps1'),"pr change`n")
  $null=Read-HumCiGit $PlanFixture @('add','--','tools/check_text_hygiene.ps1')
  $PlanTreeH=(Read-HumCiGit $PlanFixture @('write-tree')).Trim()
  $PlanHead=(Read-HumCiGit $PlanFixture ($PlanIdentity+@('commit-tree',$PlanTreeH,'-p',$PlanStaleBase,'-m','pr-head'))).Trim()
  # M is GitHub's test merge commit, built on the current main tip.
  $PlanMerge=(Read-HumCiGit $PlanFixture ($PlanIdentity+@('commit-tree',$PlanTreeH,'-p',$PlanTip,'-p',$PlanHead,'-m','integration'))).Trim()
  $null=Read-HumCiGit $PlanFixture @('checkout','-q','--detach',$PlanMerge)
  foreach ($Name in @('HUM_EVENT','HUM_BASE','HUM_BASE_REF','HUM_HEAD','HUM_INTEGRATION','HUM_FULL_REQUESTED','RUNNER_TEMP','GITHUB_OUTPUT','GITHUB_STEP_SUMMARY','GITHUB_REPOSITORY','GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT')) {
    $OldPlanEnv[$Name]=[Environment]::GetEnvironmentVariable($Name,'Process')
  }
  [Environment]::SetEnvironmentVariable('HUM_EVENT','pull_request','Process')
  [Environment]::SetEnvironmentVariable('HUM_BASE',$PlanStaleBase,'Process')
  [Environment]::SetEnvironmentVariable('HUM_BASE_REF','main','Process')
  [Environment]::SetEnvironmentVariable('HUM_HEAD',$PlanHead,'Process')
  [Environment]::SetEnvironmentVariable('HUM_INTEGRATION',$PlanMerge,'Process')
  [Environment]::SetEnvironmentVariable('HUM_FULL_REQUESTED','false','Process')
  [Environment]::SetEnvironmentVariable('RUNNER_TEMP',[IO.Path]::GetTempPath(),'Process')
  $PlanOutput=Join-Path ([IO.Path]::GetTempPath()) ('hum-ci-plan-output-'+[Guid]::NewGuid().ToString('N')+'.txt')
  $PlanSummary=Join-Path ([IO.Path]::GetTempPath()) ('hum-ci-plan-summary-'+[Guid]::NewGuid().ToString('N')+'.md')
  [Environment]::SetEnvironmentVariable('GITHUB_OUTPUT',$PlanOutput,'Process')
  [Environment]::SetEnvironmentVariable('GITHUB_STEP_SUMMARY',$PlanSummary,'Process')
  [Environment]::SetEnvironmentVariable('GITHUB_REPOSITORY','owner/repo','Process')
  [Environment]::SetEnvironmentVariable('GITHUB_RUN_ID','0','Process')
  [Environment]::SetEnvironmentVariable('GITHUB_RUN_ATTEMPT','1','Process')
  Push-Location $PlanFixture
  try {
    $PlanError=$null
    $PlanGhState = Install-HumCiTestGhMock
    try { & ([scriptblock]::Create($PlanBlock)) } catch { $PlanError=$_.Exception.Message }
    Remove-HumCiTestGhMock $PlanGhState
    Assert-Policy ($null -eq $PlanError) "plan block passes with a stale event base (error: $PlanError)"
    $PlanOutLines=@(Get-Content -LiteralPath $PlanOutput)
    Assert-Policy ($PlanOutLines -contains "base=$PlanTip") 'plan reports the verified base, not the stale event base'
    Assert-Policy ($PlanOutLines -contains 'profile=language') 'plan selects language for the owned-prefix fixture change'
    # Decision 0025 amendment (2026-09-23, boolean transport): the plan step
    # publishes boundary_required computed from the ACCEPTED BASE policy's
    # trigger function, never the path inventory. The fixture change
    # (tools/check_text_hygiene.ps1) is not a boundary consumer, so the
    # boolean must be false.
    Assert-Policy ($PlanOutLines -contains 'boundary_required=false') 'plan publishes boundary_required=false for non-consumer change'
  } finally { Pop-Location }
} finally {
  foreach ($Entry in $OldPlanEnv.GetEnumerator()) { [Environment]::SetEnvironmentVariable($Entry.Key,$Entry.Value,'Process') }
  if (Test-Path -LiteralPath $PlanFixture) { Remove-Item -LiteralPath $PlanFixture -Recurse -Force }
  if (Test-Path -LiteralPath $PlanRemote) { Remove-Item -LiteralPath $PlanRemote -Recurse -Force }
  if ($PlanOutput -and (Test-Path -LiteralPath $PlanOutput)) { Remove-Item -LiteralPath $PlanOutput -Force }
  if ($PlanSummary -and (Test-Path -LiteralPath $PlanSummary)) { Remove-Item -LiteralPath $PlanSummary -Force }
}
# StrictMode regression contract. StrictMode is deliberately NOT enabled
# globally: these scripts depend on PowerShell's lenient semantics in
# load-bearing ways (see the leniency inventory below), and flipping the
# switch globally would churn hundreds of unrelated lines for no safety gain.
# Instead, the security-critical contract assertions run inside a confined
# `& { Set-StrictMode -Version Latest; ... }` scope, which proves they never
# read an unset variable. That is exactly the failure shape of the 2026-09-23
# stale-rename incident: renaming $FastStart to $HygieneBoundary left stale
# references that silently evaluated to $null and surfaced as a confusing
# ".Replace($null,'') cannot convert newChar to System.Char" error. Under
# confined StrictMode the same staleness throws
# "The variable '$HygieneBoundary' cannot be retrieved because it has not
# been set." at the read site. The dispatcher contract, Unit B transport
# contract, and Unit C isolation contract run under confined StrictMode in
# check_all.ps1's Invoke-Wo25StrictModeContract (hygiene group, every
# profile); this file covers the classifier and the stale-rename mutation.
#
# Lenient-dependency inventory (why StrictMode stays confined). Do not add new
# instances of these patterns inside contract assertions; the confined scopes
# are the tripwire.
# - Unset-variable reads are load-bearing in check_all.ps1: the WO25
#   stale-control matrix builds needle variables ($HygieneBoundary and
#   friends) and reads them across 41 in-memory corruption cases; optional
#   env reads ($env:HUM_CANONICAL_SEAL_EVIDENCE_TIER, $env:GITHUB_RUN_ATTEMPT)
#   return $null when unset and the code branches on that.
# - $Matches is read after -match in check_all.ps1 (14 sites),
#   run_fast_evidence.ps1 (4), check_ci_policy.ps1 (3), test_ci_policy.ps1
#   (4). The safe idiom — read $Matches immediately after the matching
#   operation in the same scope — is used everywhere; separating the read
#   from the match would silently pick up a stale $Matches.
# - $null is always on the left of -eq/-ne comparisons (75 sites in
#   check_all.ps1, 19 in run_fast_evidence.ps1); the reversed form would apply
#   lenient array filtering instead of a null test.
# - Native-command probes rely on the $LASTEXITCODE reset protocol
#   ($global:LASTEXITCODE = 0 after capturing the code of interest); see the
#   regression above and the workspace AGENTS.md entry.
$StrictSource=[IO.File]::ReadAllText((Join-Path $PSScriptRoot 'check_all.ps1'))
$StrictFixtures=@(
  @(@('docs/bakeoff/EFFECT_POLYMORPHISM_CORPUS.md'),'language'),
  @(@('docs/DIAGNOSTICS.md'),'compiler'),
  @(@('src/parser.rs'),'compiler'),
  @(@('tools/check_all.ps1'),'full'),
  @(@('workorders/active/WO27.md'),'language')
)
$StrictBaseline=@($StrictFixtures | ForEach-Object { Get-HumCiProfile $_[0] })
$StrictError=$null
$StrictProfiles=& {
  Set-StrictMode -Version Latest
  try {
    @($StrictFixtures | ForEach-Object { Get-HumCiProfile $_[0] })
  } catch { $script:StrictError=$_ }
}
Assert-Policy ($null -eq $StrictError) "classifier runs under confined StrictMode (error: $($StrictError.Exception.Message))"
Assert-Policy ((@($StrictProfiles | ForEach-Object { "$_" }) -join '|') -ceq ((@($StrictBaseline | ForEach-Object { "$_" }) -join '|'))) 'classifier routes identically under confined StrictMode'
# Stale-rename regression: rename the $HygieneBoundary initialization in a copy
# of check_all.ps1, define the MUTATED contract function, and execute it under
# confined StrictMode. The stale $HygieneBoundary read must throw the
# unset-variable error instead of silently evaluating to $null. This is the
# 2026-09-23 incident shape ($FastStart renamed, stale references left
# behind); executing the mutated definition is what makes the test honest —
# passing mutated text as -Source would only analyze it as data and could
# never catch a stale read.
$StaleInitCount=([regex]::Matches($StrictSource,'\$HygieneBoundary =')).Count
Assert-Policy ($StaleInitCount -eq 1) 'stale-rename mutation target is unique'
$StaleSource=$StrictSource -creplace '\$HygieneBoundary =','$HygieneBoundaryRenamed ='
$StaleFailure=$null
& {
  Set-StrictMode -Version Latest
  try {
    $StaleTokens=$null; $StaleErrors=$null
    $StaleAst=[Management.Automation.Language.Parser]::ParseInput($StaleSource,[ref]$StaleTokens,[ref]$StaleErrors)
    $StaleDef=@($StaleAst.FindAll({ param($Node) $Node -is [Management.Automation.Language.FunctionDefinitionAst] -and $Node.Name -ceq 'Assert-Wo25EvidenceTierDispatcherContract' },$true))
    if ($StaleDef.Count -ne 1) { throw 'stale-rename mutation lost the contract function' }
    Invoke-Expression $StaleDef[0].Extent.Text
    Assert-Wo25EvidenceTierDispatcherContract -Source $StaleSource -SkipStaleControl
  } catch { $script:StaleFailure=$_ }
}
Assert-Policy (($null -ne $StaleFailure) -and ($StaleFailure.Exception.Message -match '\$HygieneBoundary') -and ($StaleFailure.Exception.Message -match 'has not been set')) 'stale variable rename fails closed with unset-variable error under StrictMode'
Write-Output "CI policy focused controls passed: $Count assertions; no Full execution credit."
