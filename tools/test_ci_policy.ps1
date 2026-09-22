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
  @('full', @('tools/run_fast_evidence.ps1')), @('full', @('.github/workflows/validation.yml')),
  @('full', @('docs/TESTING_STRATEGY.md')), @('full', @('unknown.file')),
  @('full', @('src/new_unmapped.rs')), @('full', @('examples/new_unmapped.hum')),
  @('full', @('fixtures/new_unmapped.json')), @('full', @('fixtures/new_unmapped.hum')),
  @('full', @('docs/new_policy.md')), @('full', @('docs/HUM_CORE_VERIFY_SCHEMA.md')),
  @('full', @('docs/TEXT_HYGIENE_WORKFLOW.md')), @('full', @('CONTRIBUTING.md')),
  @('compiler', @('fixtures/ownership_check/session_j_use_after_move_fail.hum')),
  @('full', @('src/run.rs','tools/check_ci_policy.ps1')), @('full', @())
)) { Assert-Policy ((Get-HumCiProfile $Case[1]) -ceq $Case[0]) "routing $($Case[1] -join ',')" }
foreach ($Paths in @(@('src/run.rs','SRC/run.rs'), @('../src/run.rs'), @('src//run.rs'), @("src/run.rs`nother"), @(('C'+':/src/run.rs')), @('src\run.rs'), @(''))) {
  Assert-PolicyRejects { Get-HumCiProfile $Paths } 'malformed inventory cannot select cheaper work'
}

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
  foreach($Path in @('src/new_unmapped.rs','examples/new_unmapped.hum','fixtures/new_unmapped.hum','docs/new_policy.md')) {
    $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
    $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,$Path")
    $NewTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
    $NewHead=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$NewTree,'-p',$Base,'-m','addition'))).Trim()
    $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$NewHead)
    Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $NewHead).Profile -ceq 'full') 'new path cannot admit itself'
  }
  $null=Read-HumCiGit $Fixture @('read-tree',$TreeA)
  $null=Read-HumCiGit $Fixture @('update-index','--force-remove','--','src/run.rs')
  $DeletedTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Deleted=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$DeletedTree,'-p',$Base,'-m','deletion'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Deleted)
  Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $Deleted).Profile -ceq 'full') 'registered deletion selects Full'
  $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,src/parser.rs")
  $RenameTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $Renamed=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$RenameTree,'-p',$Base,'-m','rename'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Renamed)
  $RenameSelection=Get-HumCiPushSelection $Fixture $Base $Renamed
  Assert-Policy ($RenameSelection.Profile -ceq 'full' -and $RenameSelection.Paths -ccontains 'src/run.rs' -and $RenameSelection.Paths -ccontains 'src/parser.rs') 'both rename sides participate'
  # Complete multi-commit range, including a policy edit reverted before head.
  $null=Read-HumCiGit $Fixture @('read-tree',$TreeB)
  $null=Read-HumCiGit $Fixture @('update-index','--add','--cacheinfo',"100644,$Blob,tools/check_ci_policy.ps1")
  $PolicyTree=(Read-HumCiGit $Fixture @('write-tree')).Trim()
  $PolicyCommit=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$PolicyTree,'-p',$Head,'-m','policy'))).Trim()
  $Revert=(Read-HumCiGit $Fixture ($Identity+@('commit-tree',$TreeB,'-p',$PolicyCommit,'-m','revert'))).Trim()
  $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Revert)
  Assert-Policy ((Get-HumCiPushSelection $Fixture $Base $Revert).Profile -ceq 'full') 'reverted policy in earlier push commit remains Full'
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
    function gh {
      $global:LASTEXITCODE=0
      if($args -contains '--paginate'){
        $Jobs=foreach($Platform in @('windows','ubuntu')){
          $Names=@('Checkout','Verify integration identity','Run Hum preflight','Close full evidence transport','Confirm selected work completion')
          if($Platform -ceq 'ubuntu'){$Names+='Run exhaustive canonical-seal evidence'}
          @{name="evaluate / preflight ($Platform-latest)";run_id=42;head_sha=('a'*40);status='completed';conclusion='success';steps=@($Names|ForEach-Object{@{name=$_;status='completed';conclusion='success'}})}
        }
        ConvertTo-Json -InputObject @(@{jobs=@($Jobs)}) -Depth 8 -Compress
      }else{
        @{workflow_runs=@(@{event='schedule';path='.github/workflows/validation.yml';repository=@{full_name='owner/repo'};head_branch='main';head_sha=('a'*40);status='completed';conclusion=$script:HealthConclusion;id=42;run_attempt=1;created_at=[datetimeoffset]::UtcNow.ToString('o')})}|ConvertTo-Json -Depth 8 -Compress
      }
    }
    foreach($Case in @(@($Accepted,$Ordinary,'runtime'),@($Accepted,$Poison,'full'),@($Base,$Head,'full'),@(('0'*40),$Ordinary,'full'))){
      $env:HUM_CI_BASE_SHA=$Case[0];$env:HUM_CI_HEAD_SHA=$Case[1];$script:HealthConclusion='success'
      $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Case[1])
      [IO.File]::WriteAllText($env:GITHUB_OUTPUT,'')
      & { $Mode='full'; . ([scriptblock]::Create($PushCode)) }
      $Lines=[IO.File]::ReadAllLines($env:GITHUB_OUTPUT)
      Assert-Policy ($Lines -ccontains "profile=$($Case[2])") 'actual push caller consumes accepted policy or Full fallback'
    }
    $env:HUM_CI_BASE_SHA=$Accepted;$env:HUM_CI_HEAD_SHA=$Ordinary;$script:HealthConclusion='failure'
    $null=Read-HumCiGit $Fixture @('update-ref','HEAD',$Ordinary)
    [IO.File]::WriteAllText($env:GITHUB_OUTPUT,'')
    Assert-PolicyRejects { & { $Mode='full'; . ([scriptblock]::Create($PushCode)) } } 'actual normal push rejects failed health'
    Assert-Policy ([IO.File]::ReadAllText($env:GITHUB_OUTPUT).Length -eq 0) 'failed health publishes no selected mode'
    Assert-Policy (@(Get-ChildItem -LiteralPath $Fixture -Filter 'hum-accepted-push-policy-*').Count -eq 0) 'accepted-policy disposable files removed'
  } finally {
    Remove-Item Function:gh -ErrorAction SilentlyContinue
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
    # Existing push-only script contains this GitHub template substitution.
    # This checks shell syntax after representative interpolation, not API access.
    $Body=$Body.Replace('${{ github.repository }}','owner/repo')
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
  'Invoke-HumCompilerFrontChecks'='f705e7a73254db2626ea9a0b0a536de493417a15380622745b4941ed6d4a1dda'
  'Invoke-HumCompilerCorpusChecks'='c07fbd4f30d9d811584d5f4ea11bd4490f6cb48537bf87c43ff2e160bc7bc1d5'
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
    Assert-Policy ($Digest -ceq $CompilerBodies[$Name]) "complete shared body $Name"
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
Assert-Policy ($Workflow.Contains('git show "$($env:HUM_BASE):tools/check_ci_policy.ps1"')) 'accepted-base policy selection'
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
Write-Output "CI policy focused controls passed: $Count assertions; no Full execution credit."
