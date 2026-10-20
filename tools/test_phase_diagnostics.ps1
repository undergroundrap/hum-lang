# Focused fixture for phase diagnostic markers.
# Tests the Write-HumPhaseMarker and Invoke-HumPhase functions from check_all.ps1
# without running the full evidence suite.
#
# Covers:
# - Completed phases with real ordered timings
# - Phase records available before producer completion (start marker written first)
# - Interruption between phase start/end retains previous completion and marks current incomplete
# - Absent diagnostics directory (PhaseDiagnostics empty) is a no-op
# - Malformed phase names are rejected
# - Sink failure does not mask the phase action

$ErrorActionPreference = 'Stop'

# Dot-source the functions from check_all.ps1 without running the main body.
# We extract the function definitions by line numbers.
$CheckAllPath = Join-Path $PSScriptRoot 'check_all.ps1'
$Lines = [IO.File]::ReadAllLines($CheckAllPath)

# Find function boundaries (Write-HumPhaseMarker: line 20 to 68, Invoke-HumPhase: line 69 to 82)
# Use 1-based line numbers from grep, convert to 0-based indices
$MarkerStart = 20 - 1; $MarkerEnd = 68 - 1  # inclusive
$PhaseStart = 69 - 1; $PhaseEnd = 82 - 1    # inclusive

$MarkerLines = $Lines[$MarkerStart..$MarkerEnd] -join "`n"
$PhaseLines = $Lines[$PhaseStart..$PhaseEnd] -join "`n"

# Set up the PhaseDiagnostics variable in script scope
$PhaseDiagnostics = ''

Invoke-Expression $MarkerLines
Invoke-Expression $PhaseLines

function Invoke-OwnedRetainedHelperBindingControls {
  param([string]$TestText)
  # Binding-only: keep current param/caller/export-hook AST text, replace all other body work.
  $Tokens=$null;$Errors=$null
  $Ast=[Management.Automation.Language.Parser]::ParseInput($TestText,[ref]$Tokens,[ref]$Errors)
  if($Errors.Count -ne 0){throw 'retained binding source parse refused'}
  $Assertions=[Collections.Generic.List[object]]::new();$Cases=[Collections.Generic.List[object]]::new();$Pins=[Collections.Generic.List[object]]::new()
  function Require-RetainedBinding([bool]$Ok,[string]$Message){$Assertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message});if(-not $Ok){throw ('retained binding assertion: '+$Message)}}
  function Pin-RetainedBinding($Node){
    $Hash=[Security.Cryptography.SHA256]::Create();try{$Digest=[BitConverter]::ToString($Hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Node.Extent.Text))).Replace('-','').ToLowerInvariant()}finally{$Hash.Dispose()}
    return [pscustomobject]@{StartUTF16=$Node.Extent.StartOffset;EndUTF16=$Node.Extent.EndOffset;UTF8TextSha256=$Digest;Text=$Node.Extent.Text}
  }
  foreach($Pair in @(
    @('Invoke-OwnedWo24StageTimingControls','Invoke-OwnedWo24StageTimingControls $CheckAllText $EvidenceText'),
    @('Invoke-OwnedWo23AuthorityProofControls','Invoke-OwnedWo23AuthorityProofControls $CheckAllText'),
    @('Invoke-OwnedWo22AuthorityProofControls','Invoke-OwnedWo22AuthorityProofControls $CheckAllText'),
    @('Invoke-OwnedWo24AuthorityProofControls','Invoke-OwnedWo24AuthorityProofControls $CheckAllText'),
    @('Invoke-OwnedCaptureStartupAttributionControls','Invoke-OwnedCaptureStartupAttributionControls $EvidenceText'),
    @('Invoke-OwnedWo22TargetRoutingControls','Invoke-OwnedWo22TargetRoutingControls $CheckAllText ([IO.File]::ReadAllText((Join-Path $PSScriptRoot ''test_exact_rust_selector.ps1''))) ([IO.File]::ReadAllBytes((Join-Path $PSScriptRoot ''../src/backend_cranelift.rs'')))')
  )){
    $Name=$Pair[0];$Call=$Pair[1]
    $Functions=@($Ast.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq $Name},$true))
    $Calls=@($Ast.FindAll({param($N)$N -is [Management.Automation.Language.CommandAst] -and $N.GetCommandName() -ceq $Name},$true))
    if($Functions.Count -ne 1 -or $Calls.Count -ne 1 -or $Calls[0].Extent.Text -cne $Call){throw ('retained binding unique current function/caller refused: '+$Name)}
    $Function=$Functions[0];$ParamBlock=$Function.Body.ParamBlock
    $Selected=@($ParamBlock.Parameters|Where-Object{$_.Name.VariablePath.UserPath -ceq 'SelectedProgram'})
    if($Selected.Count -ne 1 -or $Selected[0].Attributes.Count -ne 0 -or $Selected[0].Extent.Text -cne '$SelectedProgram = $null'){throw ('retained binding current optional parameter refused: '+$Name)}
    $Exports=@($Function.Body.EndBlock.Statements|Where-Object{$_ -is [Management.Automation.Language.IfStatementAst] -and [regex]::IsMatch($_.Extent.Text,'\$SelectedProgram\.Value\s*=\s*\$Program')})
    if($Exports.Count -ne 1){throw ('retained binding actual export-hook cardinality refused: '+$Name)}
    $CurrentParams=$ParamBlock.Extent.Text
    $ProjectedCall=$Call;$InputSubstitutions=0
    if($Name -ceq 'Invoke-OwnedWo22TargetRoutingControls'){
      $TextRead='([IO.File]::ReadAllText((Join-Path $PSScriptRoot ''test_exact_rust_selector.ps1'')))'
      $BytesRead='([IO.File]::ReadAllBytes((Join-Path $PSScriptRoot ''../src/backend_cranelift.rs'')))'
      if($Calls[0].CommandElements.Count -ne 4 -or $Calls[0].CommandElements[2].Extent.Text -cne $TextRead -or $Calls[0].CommandElements[3].Extent.Text -cne $BytesRead){throw 'last binding exact source argument projection refused'}
      $ProjectedCall=$Call.Replace($TextRead,'$SelectorText').Replace($BytesRead,'$BackendBytes');$InputSubstitutions=2
    }
    $RefPrefix=if($Name -ceq 'Invoke-OwnedWo22TargetRoutingControls'){'[ref] '}else{'[ref]'}
    $BaselineParams=$CurrentParams.Replace($Selected[0].Extent.Text,$RefPrefix+$Selected[0].Extent.Text)
    $Inputs=[Collections.Generic.List[string]]::new();$ExpectedInputs=[Collections.Generic.List[string]]::new()
    foreach($Parameter in $ParamBlock.Parameters){
      $InputName=$Parameter.Name.VariablePath.UserPath
      if($InputName -ceq 'SelectedProgram'){continue}
      if($InputName -cnotin @('CheckAllText','EvidenceText','SelectorText','BackendBytes')){throw 'retained binding unexpected input parameter'}
      $Inputs.Add('$'+$InputName);$ExpectedInputs.Add('owned-'+$InputName)
    }
    $Pins.Add([pscustomobject]@{Name=$Name;ParameterBlock=(Pin-RetainedBinding $ParamBlock);OmittedCaller=(Pin-RetainedBinding $Calls[0]);ActualExport=(Pin-RetainedBinding $Exports[0]);BaselineParameterProjection=$BaselineParams})
    foreach($Mode in @('baseline-refusal','omitted-argument','explicit-reference-export')){
      $Params=if($Mode -ceq 'baseline-refusal'){$BaselineParams}else{$CurrentParams}
      $Invocation=$ProjectedCall+$(if($Mode -ceq 'explicit-reference-export'){' ([ref]$Export)'}else{''})
      $Sentinel="`$script:BindingSentinelHits++`n`$Program='owned-selected-program'`n"+$Exports[0].Extent.Text+"`n[pscustomobject]@{Name='"+$Name+"';Public='owned-binding-ok';Inputs=@("+[string]::Join(',',$Inputs)+")}`n"
      $Program="`$CheckAllText='owned-CheckAllText';`$EvidenceText='owned-EvidenceText';`$SelectorText='owned-SelectorText';`$BackendBytes=[byte[]]@(17,29);`$Export='';`$script:BindingSentinelHits=0`nfunction "+$Name+" {`n"+$Params+"`n"+$Sentinel+"}`n`$Fact=`$null;`$Caught=''`ntry{`$Fact="+$Invocation+"}catch{`$Caught=`$_.Exception.Message}`n[pscustomobject]@{Fact=`$Fact;Caught=`$Caught;SentinelHits=`$script:BindingSentinelHits;Export=`$Export}"
      $GT=$null;$GE=$null;$GA=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$GT,[ref]$GE)
      $Unknown=[Collections.Generic.List[string]]::new()
      if($GE.Count -ne 0){$Unknown.Add('parse')}
      foreach($N in $GA.FindAll({param($N)$true},$true)){
        if(($N -is [Management.Automation.Language.TypeExpressionAst] -or $N -is [Management.Automation.Language.TypeConstraintAst]) -and $N.TypeName.FullName -cnotin @('string','ref','pscustomobject','byte[]')){$Unknown.Add('type:'+$N.TypeName.FullName)}
        if($N -is [Management.Automation.Language.CommandAst] -and $N.GetCommandName() -cne $Name){$Unknown.Add('command:'+$N.Extent.Text)}
        if($N -is [Management.Automation.Language.InvokeMemberExpressionAst]){$Unknown.Add('method:'+$N.Member.Extent.Text)}
        elseif($N -is [Management.Automation.Language.MemberExpressionAst] -and ($N.Static -or $N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $N.Member.Value -cnotin @('Value','Exception','Message'))){$Unknown.Add('property:'+$N.Member.Extent.Text)}
        if($N -is [Management.Automation.Language.VariableExpressionAst] -and $N.VariablePath.IsDriveQualified -and $N.VariablePath.UserPath -cne 'script:BindingSentinelHits'){$Unknown.Add('drive:'+$N.VariablePath.UserPath)}
        if($N -is [Management.Automation.Language.FileRedirectionAst]){$Unknown.Add('redirection')}
      }
      if($Unknown.Count -ne 0){throw ('retained binding fatal selected ZERO refused: '+[string]::Join('|',$Unknown))}
      $Observed=& ([scriptblock]::Create($Program))
      if($Mode -ceq 'baseline-refusal'){
        Require-RetainedBinding ($Observed.Caught.Contains('Reference type is expected in argument.')) ($Name+' baseline public binding refusal')
        Require-RetainedBinding ($Observed.SentinelHits -eq 0 -and $null -eq $Observed.Fact) ($Name+' baseline refuses before sentinel')
      }else{
        Require-RetainedBinding ($Observed.Caught -ceq '' -and $Observed.SentinelHits -eq 1) ($Name+'/'+$Mode+' reaches exactly one sentinel')
        Require-RetainedBinding ($Observed.Fact.Name -ceq $Name -and $Observed.Fact.Public -ceq 'owned-binding-ok') ($Name+'/'+$Mode+' correct public returned fact')
        if($Name -ceq 'Invoke-OwnedWo22TargetRoutingControls'){
          Require-RetainedBinding ($Observed.Fact.Inputs.Count -eq 3 -and $Observed.Fact.Inputs[0] -ceq 'owned-CheckAllText' -and $Observed.Fact.Inputs[1] -ceq 'owned-SelectorText' -and $Observed.Fact.Inputs[2] -is [byte[]] -and [string]::Join(',',$Observed.Fact.Inputs[2]) -ceq '17,29') ($Name+'/'+$Mode+' actual ordered string/string/byte-array inputs consumed')
        }else{
        Require-RetainedBinding ([string]::Join('|',$Observed.Fact.Inputs) -ceq [string]::Join('|',$ExpectedInputs)) ($Name+'/'+$Mode+' actual parameter inputs consumed')
        }
      }
      Require-RetainedBinding ($Observed.Export -ceq $(if($Mode -ceq 'explicit-reference-export'){'owned-selected-program'}else{''})) ($Name+'/'+$Mode+' actual export hook result')
      $Cases.Add([pscustomobject]@{Name=$Name;Mode=$Mode;SourceCaller=$Call;Caller=$Invocation;OwnedReadArgumentSubstitutions=$InputSubstitutions;Caught=$Observed.Caught;SentinelHits=$Observed.SentinelHits;Fact=$Observed.Fact;Export=$Observed.Export;Zero=[pscustomobject]@{Unknown=0;RealAccessors=0;BeforeCreate=$true};Projection='actual params/caller/optional export hook; all remaining body replaced by pure sentinel'})
    }
  }
  return [pscustomobject]@{Assertions=$Assertions.ToArray();Cases=$Cases.ToArray();Pins=$Pins.ToArray();Gate=[pscustomobject]@{CompiledBindingPrograms=$Cases.Count;RealAccessors=0;FullHelperBodiesExecuted=0};EvidenceKind='binding-only owned sentinel projections'}
}

function Invoke-OwnedWo22TargetRoutingControls {
  param([string] $CheckAllText, [string] $SelectorText, [byte[]] $BackendBytes, $SelectedProgram = $null)
  function Select-Wo22OwnedFunction([string] $Text, [string] $Name) {
    $Tokens = $null; $Errors = $null
    $Ast = [Management.Automation.Language.Parser]::ParseInput($Text,[ref]$Tokens,[ref]$Errors)
    $Found = @($Ast.FindAll({param($Node) $Node -is [Management.Automation.Language.FunctionDefinitionAst] -and $Node.Name -ceq $Name},$true))
    if ($Errors.Count -ne 0 -or $Found.Count -ne 1) { throw "WO22 owned source selector refused: $Name" }
    return $Found[0]
  }
  $Names = @('Get-ExactRustGenerationDigest','Reset-ExactRustSelectorCredits','Get-ExactRustSelectorCredits','Assert-ExactRustSelectorSyntax','Assert-ExactRustSelectorEvidence','Assert-Wo22RustNamespace','New-Wo22RustTargetAdmission','Assert-Wo22RustTargetAdmission','Close-Wo22RustTargetAdmission','Invoke-ExactRustTest')
  $Parts = [Collections.Generic.List[string]]::new()
  $Pins = [Collections.Generic.List[object]]::new()
  foreach ($Name in $Names + @('Test-Wo22ByteArraysEqual','Invoke-Wo22BackendPredicateMutationEvidence')) {
    $InputText = if ($Name -cin @('Test-Wo22ByteArraysEqual','Invoke-Wo22BackendPredicateMutationEvidence')) { $CheckAllText } else { $SelectorText }
    $Function = Select-Wo22OwnedFunction $InputText $Name
    $Pins.Add([pscustomobject]@{Name=$Name;StartUTF16=$Function.Extent.StartOffset;EndUTF16=$Function.Extent.EndOffset;Text=$Function.Extent.Text})
    $Text = $Function.Extent.Text
    if ($Name -ceq 'New-Wo22RustTargetAdmission') {
      $RootExpression = '[IO.Path]::GetDirectoryName([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($BackendPath)))'
      if ([regex]::Matches($Text,[regex]::Escape($RootExpression)).Count -ne 1 -or [regex]::Matches($Text,'return \$Admission').Count -ne 1) { throw 'WO22 owned root/return projection refused' }
      $Text = $Text.Replace($RootExpression,"'owned-root'").Replace('return $Admission','return (Complete-Wo22OwnedAdmission $Admission)')
    }
    $Text = $Text.Replace('Read-Wo22RustTargetNamespace','Read-Wo22OwnedNamespace').Replace('Write-Host','Write-Wo22OwnedMessage').Replace('Join-Path','Join-Wo22OwnedPath')
    $Text = $Text.Replace('[System.IO.File]::ReadAllBytes($BackendPath)','(Read-Wo22OwnedBytes $BackendPath)').Replace('[System.IO.File]::WriteAllText($BackendPath, $Mutated, $Utf8)','Write-Wo22OwnedBytes $BackendPath $Mutated $Utf8').Replace('[System.IO.File]::WriteAllBytes($BackendPath, $OriginalBytes)','Restore-Wo22OwnedBytes $BackendPath $OriginalBytes')
    $Text = $Text.Replace('New-Object System.Text.UTF8Encoding($false, $true)','[Text.UTF8Encoding]::new($false, $true)').Replace("New-Object 'System.Collections.Generic.List[string]'",'[Collections.Generic.List[string]]::new()')
    $Parts.Add($Text)
  }
  # The source collector remains static-only: its real filesystem/environment
  # boundary is replaced by an owned namespace return. Native capture is an owned
  # argv/result sink; the selected admission, list/run, mutation and main guards are real.
  $Tokens=$null; $Errors=$null
  $Tree=[Management.Automation.Language.Parser]::ParseInput($CheckAllText,[ref]$Tokens,[ref]$Errors)
  $Guards=@($Tree.FindAll({param($Node) $Node -is [Management.Automation.Language.IfStatementAst] -and $Node.Extent.Text.Contains("throw 'Work Order 22 mutation selector must use the one guarded exact-selector route'")},$true))
  $Assignments=@($Tree.FindAll({param($Node) $Node -is [Management.Automation.Language.AssignmentStatementAst] -and $Node.Left.Extent.Text -ceq '$MutationFunctions'},$true))
  if ($Errors.Count -ne 0 -or $Guards.Count -ne 1 -or $Assignments.Count -ne 1) { throw 'WO22 actual main guard source selector refused' }
  $MainGuard=$CheckAllText.Substring($Assignments[0].Extent.StartOffset,$Guards[0].Extent.EndOffset-$Assignments[0].Extent.StartOffset)
  $Owned = @'
param([byte[]] $BackendBytes,[string] $CheckAllText)
function Add-Wo22OwnedHook([string] $Name) { if (-not $script:W22Hooks.ContainsKey($Name)) { $script:W22Hooks.Add($Name,0) }; $script:W22Hooks[$Name]++ }
function Assert-Wo22Owned([bool] $Ok,[string] $Message) { $script:W22Assertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message}); if (-not $Ok) { throw "WO22 owned assertion refused: $Message" } }
function Read-Wo22OwnedBytes($Path) { Add-Wo22OwnedHook 'read'; return ,([byte[]] $script:W22Bytes.Clone()) }
function Write-Wo22OwnedBytes($Path,$Text,$Encoding) { Add-Wo22OwnedHook 'write'; $script:W22Bytes=$Encoding.GetBytes($Text); if ($script:W22Mode -ceq 'variant-write-corruption') { $script:W22Bytes[0]=0 } }
function Restore-Wo22OwnedBytes($Path,[byte[]] $Bytes) { Add-Wo22OwnedHook 'restore'; $script:W22Bytes=[byte[]] $Bytes.Clone() }
function Write-Wo22OwnedMessage([string] $Message) { Add-Wo22OwnedHook 'public'; $script:W22Messages.Add($Message); return $Message }
function Join-Wo22OwnedPath($Parent,$Leaf) { Add-Wo22OwnedHook 'join'; return "$Parent/$Leaf" }
function Write-HumPhaseMarker { param($PhaseName,$Event,$Outcome) Add-Wo22OwnedHook 'marker' }
function Assert-ExactRustMutationGeneration { throw 'owned unexpected generation route' }
function Read-Wo22OwnedNamespace($Root,$Cargo) {
  Add-Wo22OwnedHook 'namespace'
  $Files=[Collections.Generic.SortedDictionary[string,string]]::new([StringComparer]::Ordinal)
  foreach($Name in @('Cargo.toml','Cargo.lock','src/main.rs','tests/cli_decision0030.rs','tests/cli_wo29_trust_locality.rs')) { $Files.Add($Name,'owned-'+$Name) }
  $Files.Add('src/backend_cranelift.rs',(Get-ExactRustGenerationDigest $script:W22Bytes))
  if($script:W22NamespaceDrift){$Files['tests/cli_decision0030.rs']='changed-test-namespace'}
  if($script:W22Mode -ceq 'namespace-missing'){ $null=$Files.Remove('Cargo.toml') }
  return [pscustomobject]@{Root='owned-root';Files=$Files;Context=$script:W22Context;Target=$(if($script:W22Mode -ceq 'owner-return-corruption'){'other'}else{'hum'});TargetSource='src/main.rs'}
}
function Complete-Wo22OwnedAdmission($Admission) {
  Add-Wo22OwnedHook 'admission_return'
  if($script:W22Mode -ceq 'admission-missing'){return $null}
  if($script:W22Mode -ceq 'admission-substitution'){return [pscustomobject]@{Active=$true;Root=$Admission.Root;Cargo=$Admission.Cargo;Context=$Admission.Context;Target=$Admission.Target;Selectors=$Admission.Selectors}}
  if($script:W22Mode -ceq 'admission-fact-corruption'){$Admission.Selectors[$script:W22Selector]=@('wrong::test: test')}
  if($script:W22Mode -ceq 'admission-target-corruption'){$Admission.Target='other'}
  return $Admission
}
function Invoke-ExactRustNativeCapture([string] $Cargo,[string[]] $Arguments) {
  Add-Wo22OwnedHook 'capture'
  $script:W22Calls.Add([pscustomobject]@{Cargo=$Cargo;Arguments=[string[]]$Arguments.Clone()})
  $Selector=$Arguments[1];$Qualified=$Arguments -ccontains '--bin';$List=$Arguments -ccontains '--list'
  if($Qualified){Add-Wo22OwnedHook 'qualified'}else{Add-Wo22OwnedHook 'broad'}
  if($Qualified){
    $Expected=@('test',$Selector,'--bin','hum','--','--exact');if($List){$Expected+=@('--list')}
    Assert-Wo22Owned ([string]::Join('|',$Arguments) -ceq [string]::Join('|',$Expected)) 'exact qualified argv'
  } else {
    $Expected=@('test',$Selector,'--','--exact');if($List){$Expected+=@('--list')}
    Assert-Wo22Owned ([string]::Join('|',$Arguments) -ceq [string]::Join('|',$Expected)) 'exact default argv'
  }
  if($List){
    if(-not $Qualified -and $script:W22Mode -cin @('broad-duplicate','generic-duplicate')){return [pscustomobject]@{ExitCode=0;Output=[string[]]@("${Selector}: test","${Selector}: test")}}
    if($Qualified -and $script:W22Mode -ceq 'qualified-empty'){return [pscustomobject]@{ExitCode=0;Output=[string[]]@()}}
    if($Qualified -and $script:W22Mode -ceq 'namespace-drift'){$script:W22NamespaceDrift=$true}
    if($Qualified -and $script:W22Mode -ceq 'context-drift'){$script:W22Context='changed-context'}
    return [pscustomobject]@{ExitCode=0;Output=[string[]]@("${Selector}: test")}
  }
  if($script:W22Generic -or $script:W22Mode -ceq 'green-mutant'){
    return [pscustomobject]@{ExitCode=0;Output=[string[]]@('running 1 test',"test $Selector ... ok",'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')}
  }
  $Fragment=$Mutation.PSObject.Properties['RequiredFailureFragment'];$Cause=if($null -eq $Fragment){'owned expected assertion'}else{$Fragment.Value}
  if($script:W22Mode -ceq 'wrong-cause'){$Cause='owned unrelated assertion'}
  $Name=if($script:W22Mode -ceq 'wrong-test'){'wrong::test'}else{$Selector}
  $Code=if($script:W22Mode -ceq 'wrong-exit'){1}else{101}
  return [pscustomobject]@{ExitCode=$Code;Output=[string[]]@('running 1 test',"test $Name ... FAILED",$Cause,'test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')}
}
function Reset-Wo22OwnedCase([string] $Mode) {
  $script:W22Mode=$Mode;$script:W22Generic=$false;$script:W22NamespaceDrift=$false;$script:W22Context='owned-context';$script:W22Bytes=[byte[]]$BackendBytes.Clone()
  $script:W22Calls.Clear();$script:W22Messages.Clear();$script:ExactRustSelectorCredits.Clear();$script:ExactRustSelectorCredits.Add('ambient::credit');$script:Wo22RustTargetAdmissionOwner=$null
}
'@
  $Driver = @'
$script:W22Hooks=[Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
$script:W22Assertions=[Collections.Generic.List[object]]::new();$script:W22Cases=[Collections.Generic.List[object]]::new();$script:W22Calls=[Collections.Generic.List[object]]::new();$script:W22Messages=[Collections.Generic.List[string]]::new()
$script:ExactRustSelectorCredits=[Collections.Generic.List[string]]::new();$script:Wo22RustTargetAdmissionOwner=$null
$script:W22Selector='backend_cranelift::tests::backend_go_no_go_rows_are_complete_and_load_bearing'
foreach($Mode in @('positive-all16','broad-duplicate','namespace-missing','owner-return-corruption','admission-missing','admission-substitution','admission-fact-corruption','admission-target-corruption','variant-write-corruption','namespace-drift','context-drift','qualified-empty','wrong-test','wrong-cause','wrong-exit','green-mutant')) {
  Reset-Wo22OwnedCase $Mode
  $Caught='';try{$null=Invoke-Wo22BackendPredicateMutationEvidence 'owned-cargo' 'owned-backend' $(if($Mode -ceq 'positive-all16'){''}else{'B13 ordinary execution'})}catch{$Caught=$_.Exception.Message}
  $Expected= switch -CaseSensitive ($Mode) {
    'positive-all16' {''}
    'broad-duplicate' {'WO22 broad honest singleton admission failed'}
    'namespace-missing' {'WO22 target admission namespace incomplete'}
    'owner-return-corruption' {'WO22 target admission ownership changed'}
    'admission-fact-corruption' {'Work Order 22 B13 ordinary execution did not fail the intended permanent selector'}
    'variant-write-corruption' {'Work Order 22 B13 ordinary execution did not fail the intended permanent selector'}
    'wrong-cause' {'Work Order 22 B13 ordinary execution did not reach its required escaped disposition'}
    default {'Work Order 22 B13 ordinary execution did not fail the intended permanent selector'}
  }
  Assert-Wo22Owned ($Caught -ceq $Expected) "$Mode exact public outcome"
  Assert-Wo22Owned (Test-Wo22ByteArraysEqual $BackendBytes $script:W22Bytes) "$Mode exact byte restoration"
  Assert-Wo22Owned ($null -eq $script:Wo22RustTargetAdmissionOwner) "$Mode invocation closed"
  if($Mode -ceq 'positive-all16'){
    Assert-Wo22Owned ($script:W22Calls.Count -eq 34) 'all16: two broad honest lists plus32 qualified calls'
    Assert-Wo22Owned (@($script:W22Calls|Where-Object{$_.Arguments -ccontains '--bin'}).Count -eq 32) 'all16 fresh qualified lists and runs'
    Assert-Wo22Owned (@($script:W22Messages|Where-Object{$_ -cmatch '^ok - Work Order 22 .+ production mutation rejected$'}).Count -eq 16) 'all16 public rejection rows'
  } elseif($Mode -cin @('broad-duplicate','namespace-missing','owner-return-corruption')) {
    Assert-Wo22Owned (@($script:W22Calls|Where-Object{$_.Arguments -ccontains '--bin'}).Count -eq 0) "$Mode no mutant capture"
  }
  $ExpectedCredit=if($Mode -ceq 'green-mutant'){'ambient::credit|'+$script:W22Selector}else{'ambient::credit'}
  Assert-Wo22Owned ([string]::Join('|',$script:ExactRustSelectorCredits) -ceq $ExpectedCredit) "$Mode actual credit behavior"
  $script:W22Cases.Add([pscustomobject]@{Mode=$Mode;Calls=$script:W22Calls.Count;Primary=$Caught;Credits=[string[]]$script:ExactRustSelectorCredits.ToArray()})
}
foreach($Mode in @('generic','generic-duplicate')) {
  Reset-Wo22OwnedCase $Mode;$script:W22Generic=$true;$Caught=''
  try{$null=Invoke-ExactRustTest 'owned generic default' 'owned-cargo' $script:W22Selector}catch{$Caught=$_.Exception.Message}
  Assert-Wo22Owned (($Mode -ceq 'generic' -and $Caught -ceq '') -or ($Mode -ceq 'generic-duplicate' -and $Caught.Contains('must resolve'))) "$Mode actual default outcome"
  Assert-Wo22Owned (@($script:W22Calls|Where-Object{$_.Arguments -ccontains '--bin'}).Count -eq 0) "$Mode keeps unqualified target route"
  Assert-Wo22Owned ($script:W22Calls.Count -eq $(if($Mode -ceq 'generic'){2}else{1})) "$Mode exact capture count"
  Assert-Wo22Owned ($script:ExactRustSelectorCredits.Count -eq $(if($Mode -ceq 'generic'){2}else{1})) "$Mode exact credits"
  $script:W22Cases.Add([pscustomobject]@{Mode=$Mode;Calls=$script:W22Calls.Count;Primary=$Caught})
}
$CurrentCall='Invoke-ExactRustTest $MutationExecutionLabel $Cargo $Mutation.Selector -Wo22Admission $Wo22Admission -Wo22MutationBytes ($Utf8.GetBytes($Mutated))'
$At=$CheckAllText.IndexOf($CurrentCall,[StringComparison]::Ordinal)
Assert-Wo22Owned ($At -ge 0) 'current guarded call exists'
foreach($Mode in @('current','old','missing','bypass','malformed')) {
  $Replacement=switch -CaseSensitive($Mode){'current'{$CurrentCall};'old'{'Invoke-ExactRustTest $MutationExecutionLabel $Cargo $Mutation.Selector'};'missing'{"Write-Host 'missing'"};'bypass'{'Invoke-ExactRustNativeCapture $Cargo @()'};'malformed'{$CurrentCall.Replace('-Wo22Admission','-WrongAdmission')}}
  $Text=$CheckAllText.Substring(0,$At)+$Replacement+$CheckAllText.Substring($At+$CurrentCall.Length)
  $Caught='';try{Invoke-Wo22OwnedMainGuard $Text}catch{$Caught=$_.Exception.Message}
  Assert-Wo22Owned (($Mode -ceq 'current' -and $Caught -ceq '') -or ($Mode -cne 'current' -and $Caught -ceq 'Work Order 22 mutation selector must use the one guarded exact-selector route')) "$Mode actual main guard outcome"
  $script:W22Cases.Add([pscustomobject]@{Mode='main-'+$Mode;Calls=0;Primary=$Caught})
}
foreach($Hook in @('capture','broad','qualified','namespace','admission_return','read','write','restore','public')){Assert-Wo22Owned ($script:W22Hooks.ContainsKey($Hook) -and $script:W22Hooks[$Hook] -gt 0) "$Hook nonzero actual boundary"}
[pscustomobject]@{Assertions=$script:W22Assertions.ToArray();Cases=$script:W22Cases.ToArray();Hooks=$script:W22Hooks;StaticOnlyCollector='Read-Wo22RustTargetNamespace';NoRealAccessors=$true}
'@
  $Program=$Owned+"`n"+[string]::Join("`n",$Parts)+"`nfunction Invoke-Wo22OwnedMainGuard { param([string]`$Text)`n`$CheckAllTokens=`$null;`$CheckAllParseErrors=`$null;`$CheckAllAst=[Management.Automation.Language.Parser]::ParseInput(`$Text,[ref]`$CheckAllTokens,[ref]`$CheckAllParseErrors)`n"+$MainGuard+"`n}`n"+$Driver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  $Tokens=$null;$Errors=$null;$Gate=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$Tokens,[ref]$Errors)
  $Types=@('byte[]','string','string[]','bool','int','object','object[]','pscustomobject','ref','Text.Encoding','Text.UTF8Encoding','System.StringComparison','StringComparison','StringComparer','regex','string','array','object','BitConverter','Security.Cryptography.SHA256','Collections.Generic.List[string]','Collections.Generic.List[object]','Collections.Generic.Dictionary[string,string]','Collections.Generic.Dictionary[string,int]','Collections.Generic.SortedDictionary[string,string[]]','Collections.Generic.SortedDictionary[string,string]','Management.Automation.Language.Parser','System.Management.Automation.Language.FunctionDefinitionAst','System.Management.Automation.Language.CommandAst','Management.Automation.Language.FunctionDefinitionAst')
  $Methods=@('Add','ContainsKey','GetEnumerator','Clear','Clone','Equals','ReferenceEquals','Join','IsNullOrEmpty','IsNullOrWhiteSpace','GetBytes','GetString','ToString','Create','ComputeHash','Dispose','Remove','IsMatch','Matches','Escape','Replace','IndexOf','Substring','StartsWith','EndsWith','Contains','ToArray','new','ParseInput','FindAll','GetCommandName')
  $Properties=@('Length','Count','Key','Value','Root','Cargo','Target','Context','Files','TargetSource','Namespace','Variants','Selectors','Instance','Active','Output','ExitCode','PSObject','Properties','Needle','Replacement','AdditionalNeedle','AdditionalReplacement','AdditionalChanges','OwnedSites','Label','Selector','Exception','Message','Ok','Arguments','Name','Body','Extent','Text','Sum','Keys','MutableRoot')
  $Commands=@('Where-Object','ForEach-Object','Measure-Object')+@($Gate.FindAll({param($Node)$Node -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
  $Unknown=[Collections.Generic.List[string]]::new()
  if($Errors.Count -ne 0){$Unknown.Add('parse')}
  foreach($Node in $Gate.FindAll({param($Node)$true},$true)) {
    if($Node -is [Management.Automation.Language.TypeExpressionAst] -or $Node -is [Management.Automation.Language.TypeConstraintAst]){if($Node.TypeName.FullName -cnotin $Types){$Unknown.Add('type:'+$Node.TypeName.FullName)}}
    if($Node -is [Management.Automation.Language.CommandAst]){ $Name=$Node.GetCommandName();if($null -eq $Name){if($Node.CommandElements[0] -isnot [Management.Automation.Language.ScriptBlockExpressionAst] -and $Node.CommandElements[0].Extent.Text -cne '$WriteSelectorPhaseMarker'){$Unknown.Add('dynamic:'+ $Node.Extent.Text)}}elseif($Name -cnotin $Commands){$Unknown.Add('command:'+$Name)} }
    if($Node -is [Management.Automation.Language.InvokeMemberExpressionAst]){if($Node.Member.Value -cnotin $Methods){$Unknown.Add('method:'+$Node.Member.Extent.Text)}}
    elseif($Node -is [Management.Automation.Language.MemberExpressionAst]){if($Node.Static){if($Node.Member.Value -cnotin @('UTF8','Ordinal')){$Unknown.Add('static-property:'+$Node.Member.Extent.Text)}}elseif($Node.Member.Value -cnotin $Properties){$Unknown.Add('property:'+$Node.Member.Extent.Text)}}
    if($Node -is [Management.Automation.Language.VariableExpressionAst] -and $Node.VariablePath.IsDriveQualified -and -not $Node.VariablePath.UserPath.StartsWith('script:',[StringComparison]::Ordinal)){$Unknown.Add('drive:'+$Node.VariablePath.UserPath)}
    if($Node -is [Management.Automation.Language.FileRedirectionAst]){$Unknown.Add('file-redirection')}
  }
  if($Unknown.Count -ne 0){throw ('WO22 fatal whole-selected-program ZERO gate refused: '+[string]::Join('|',$Unknown))}
  $Result=& ([scriptblock]::Create($Program)) $BackendBytes $CheckAllText
  return [pscustomobject]@{Assertions=$Result.Assertions;Cases=$Result.Cases;Hooks=$Result.Hooks;Pins=$Pins.ToArray();Gate=@{Unknown=0;ParseErrors=0;RealAccessors=0;CompiledPrograms=1};StaticOnlyCollector=$Result.StaticOnlyCollector}
}

function Invoke-OwnedWo24IncrementalRequestControls {
  param([string] $SelectorText, $SelectedProgram = $null)
  function Select-W24I([string] $Name) {
    $T=$null;$E=$null;$A=[Management.Automation.Language.Parser]::ParseInput($SelectorText,[ref]$T,[ref]$E)
    $F=@($A.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq $Name},$true))
    if($E.Count -ne 0 -or $F.Count -ne 1){throw ('WO24 incremental source selector refused: '+$Name)}
    return $F[0]
  }
  function Assert-W24IZero([string] $Text) {
    $T=$null;$E=$null;$A=[Management.Automation.Language.Parser]::ParseInput($Text,[ref]$T,[ref]$E)
    $Types=@('string','string[]','byte[]','bool','int','object','object[]','pscustomobject','ref','regex','StringComparison','StringComparer','Text.Encoding','BitConverter','Security.Cryptography.SHA256','Collections.Generic.List[string]','Collections.Generic.List[object]','Collections.Generic.Dictionary[string,byte[]]','Collections.Generic.SortedDictionary[string,string]','Management.Automation.Language.Parser','Management.Automation.Language.FunctionDefinitionAst','Management.Automation.Language.CommandAst','Management.Automation.Language.ScriptBlockExpressionAst','Management.Automation.Language.TypeExpressionAst','Management.Automation.Language.TypeConstraintAst','Management.Automation.Language.InvokeMemberExpressionAst','Management.Automation.Language.MemberExpressionAst','Management.Automation.Language.StringConstantExpressionAst','Management.Automation.Language.VariableExpressionAst','Management.Automation.Language.FileRedirectionAst','Management.Automation.Language.UsingExpressionAst','Management.Automation.Language.TypeDefinitionAst')
    $Methods=@('new','Add','Clear','Clone','Contains','ContainsKey','GetEnumerator','ToArray','Join','Equals','ReferenceEquals','IsNullOrEmpty','IsNullOrWhiteSpace','Escape','IsMatch','Matches','Replace','StartsWith','GetBytes','ToString','ComputeHash','Create','Dispose','ParseInput','FindAll','GetCommandName')
    $Properties=@('Count','Length','Keys','Key','Value','Name','Root','MutableRoot','OwnedRoot','Target','Cargo','Files','Active','Instance','Output','ExitCode','Arguments','Exception','Message','Sum','Passed','Failed','Ignored','Measured','Ok','Assertions','Cases','Hooks','Calls','Credits','Caught','Route','Mode','PSObject','Properties','TypeName','FullName','Extent','Text','CommandElements','Member','Static','VariablePath','IsDriveQualified','UserPath')
    # PowerShell variables are case-insensitive: admit only these owned bindings/safe automatic values.
    $Variables=@('_','A','ArgvOk','ExpectedPoison','Admission','Arguments','Baseline','Bound','Bytes','Canonical','CanonicalContents','Cargo','CargoArguments','Cases','Caught','Commands','Comparison','Config','Digest','Driver','E','EscapedSelector','Event','ExactListings','ExactResults','Expected','F','Failed','false','Files','Gate','Generation','Hasher','Ignored','IncrementalAssertion','Isolated','Label','Leaf','List','ListedTests','ListOutput','ListPhaseOutcome','ListResult','Matches','Measured','Message','Methods','Mode','MutableExpected','MutationBytes','MutationPath','N','Name','NamedResults','null','ObsCases','Ok','Original','Outcome','Output','Owned','Owner','Pair','Parent','Part','Parts','Passed','Path','PhaseName','Pins','Program','Properties','PSBoundParameters','Relative','Result','Role','Roles','Route','Row','Rows','RunningCounts','RunOutput','RunPhaseOutcome','RunResult','SavedGenerationOwner','script:ExactRustMutationGenerationOwner','script:ExactRustSelectorCredits','script:W24IAssertions','script:W24IBytes','script:W24ICalls','script:W24IGeneration','script:W24IHooks','script:W24IMode','script:W24IMutation','script:W24IRoute','script:W24ISelector','script:W24ISeparator','script:W24IWo22','script:W24IWorkingRoot','SelectedProgram','Selector','SelectorPhasePrefix','SelectorText','Spec','SummaryCounts','T','Text','Total','true','Types','Unknown','UseWo22Target','V','Variables','Wo22Admission','Wo22MutationBytes','Wo24Incremental','WorkingRoot','WriteSelectorPhaseMarker','Wrong')
    $Commands=@('Where-Object','ForEach-Object','Measure-Object')+@($A.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
    $Unknown=[Collections.Generic.List[string]]::new()
    if($E.Count -ne 0){$Unknown.Add('parse')}
    foreach($N in $A.FindAll({param($N)$true},$true)) {
      if(($N -is [Management.Automation.Language.TypeExpressionAst] -or $N -is [Management.Automation.Language.TypeConstraintAst]) -and $N.TypeName.FullName -cnotin $Types){$Unknown.Add('type:'+$N.TypeName.FullName)}
      if($N -is [Management.Automation.Language.CommandAst]) {
        $Name=$N.GetCommandName()
        if($null -eq $Name){if($N.CommandElements[0].Extent.Text -cne '$WriteSelectorPhaseMarker'){$Unknown.Add('dynamic:'+$N.Extent.Text)}}
        elseif($Name -cnotin $Commands){$Unknown.Add('command:'+$Name)}
      }
      if($N -is [Management.Automation.Language.InvokeMemberExpressionAst]){if($N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $N.Member.Value -cnotin $Methods){$Unknown.Add('method:'+$N.Member.Extent.Text)}}
      elseif($N -is [Management.Automation.Language.MemberExpressionAst]){
        if($N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst]){$Unknown.Add('dynamic-member')}
        elseif($N.Static){if($N.Member.Value -cnotin @('UTF8','Ordinal','OrdinalIgnoreCase')){$Unknown.Add('static-property:'+$N.Member.Value)}}
        elseif($N.Member.Value -cnotin $Properties){$Unknown.Add('property:'+$N.Member.Value)}
      }
      if($N -is [Management.Automation.Language.VariableExpressionAst]) {
        $V=$N.VariablePath.UserPath
        if($V -notin $Variables){$Unknown.Add('variable:'+$V)}
      }
      if($N -is [Management.Automation.Language.FileRedirectionAst] -or $N -is [Management.Automation.Language.UsingExpressionAst] -or $N -is [Management.Automation.Language.TypeDefinitionAst]){$Unknown.Add('boundary-node:'+$N.GetType().Name)}
    }
    if($Unknown.Count -ne 0){throw ('WO24 incremental fatal whole-program ZERO refused: '+[string]::Join('|',$Unknown))}
    return [pscustomobject]@{Unknown=0;ParseErrors=0;RealAccessors=0;BeforeCreate=$true}
  }
  $Parts=[Collections.Generic.List[string]]::new();$Pins=[Collections.Generic.List[object]]::new()
  foreach($Name in @('Get-ExactRustGenerationDigest','Assert-ExactRustSelectorSyntax','Assert-ExactRustSelectorEvidence','Assert-ExactRustMutationGeneration','Invoke-ExactRustTest')) {
    $F=Select-W24I $Name;$Text=$F.Extent.Text
    $Pins.Add([pscustomobject]@{Name=$Name;StartUTF16=$F.Extent.StartOffset;EndUTF16=$F.Extent.EndOffset;Text=$Text})
    if($Name -ceq 'Assert-ExactRustMutationGeneration') {
      foreach($Pair in @(
        @('[IO.Path]::GetFullPath((Get-Location).Path)','$script:W24IWorkingRoot'),
        @('[IO.Path]::DirectorySeparatorChar','$script:W24ISeparator'),
        @('[IO.File]::ReadAllBytes((Join-Path $Generation.Root $Relative))','(Read-W24IBytes (Join-W24IPath $Generation.Root $Relative))'),
        @('[IO.File]::ReadAllBytes((Join-Path $Generation.MutableRoot $Relative))','(Read-W24IBytes (Join-W24IPath $Generation.MutableRoot $Relative))')
      )){if([regex]::Matches($Text,[regex]::Escape($Pair[0])).Count -ne 1){throw 'WO24 incremental exact generation accessor projection refused'};$Text=$Text.Replace($Pair[0],$Pair[1])}
    }
    if($Name -ceq 'Invoke-ExactRustTest') {
      $Config=", '--config', 'build.incremental=true'"
      if([regex]::Matches($Text,[regex]::Escape($Config)).Count -ne 2){throw 'WO24 incremental fixed request sites refused'}
      $Baseline=$Text.Replace('function Invoke-ExactRustTest {','function Invoke-W24IBaselineTest {').Replace($Config,'')
      $Wrong=$Text.Replace('function Invoke-ExactRustTest {','function Invoke-W24IWrongConfigTest {').Replace('build.incremental=true','build.incremental=false')
      $Parts.Add($Baseline.Replace('Write-Host','Write-W24IMessage').Replace('Join-Path','Join-W24IPath'))
      $Parts.Add($Wrong.Replace('Write-Host','Write-W24IMessage').Replace('Join-Path','Join-W24IPath'))
    }
    $Parts.Add($Text.Replace('Write-Host','Write-W24IMessage').Replace('Join-Path','Join-W24IPath'))
  }
  $Owned=@'
function Require-W24I([bool]$Ok,[string]$Message){$script:W24IAssertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message});if(-not $Ok){throw ('WO24 incremental assertion refused: '+$Message)}}
function Hook-W24I([string]$Name){if(-not $script:W24IHooks.ContainsKey($Name)){$script:W24IHooks[$Name]=0};$script:W24IHooks[$Name]++}
function Write-W24IMessage([string]$Message){Hook-W24I 'message'}
function Write-HumPhaseMarker {param($PhaseName,$Event,$Outcome) Hook-W24I 'marker'}
function Join-W24IPath([string]$Parent,[string]$Leaf){Hook-W24I 'join';if($Parent -cnotin @('owned-a','owned-b')){throw 'owned join refused'};return ($Parent+'/'+$Leaf)}
function Read-W24IBytes([string]$Path){Hook-W24I 'read';if(-not $script:W24IBytes.ContainsKey($Path)){throw 'owned byte fact absent'};return ,([byte[]]$script:W24IBytes[$Path].Clone())}
function Assert-Wo22RustTargetAdmission {param($Admission,$Label,$Cargo,$Selector,$MutationBytes) Hook-W24I 'wo22-admission';if(-not [object]::ReferenceEquals($Admission,$script:W24IWo22) -or $Cargo -cne 'owned-cargo' -or $Selector -cne $script:W24ISelector){throw 'owned WO22 admission refused'}}
function Invoke-ExactRustNativeCapture([string]$Cargo,[string[]]$Arguments) {
  Hook-W24I 'capture';$script:W24ICalls.Add([pscustomobject]@{Cargo=$Cargo;Arguments=[string[]]$Arguments.Clone()})
  $List=$Arguments -ccontains '--list';$Expected=@('test',$script:W24ISelector)
  if($script:W24IRoute -ceq 'b'){$Expected+=@('--manifest-path','owned-b/Cargo.toml','--target-dir','owned-target','--config','build.incremental=true')}
  if($script:W24IRoute -ceq 'wo22'){$Expected+=@('--bin','hum')}
  $Expected+=@('--','--exact');if($List){$Expected+=@('--list')}
  $ArgvOk=$Cargo -ceq 'owned-cargo' -and [string]::Join('|',$Arguments) -ceq [string]::Join('|',$Expected)
  if(-not $ArgvOk){Hook-W24I 'argv-refusal';throw 'WO24 incremental actual exact scoped argv refused'}
  Require-W24I $ArgvOk 'actual exact scoped argv'
  if($script:W24IMode -ceq 'capture-throw' -and -not $List){throw 'owned capture refused'}
  if($List){
    if($script:W24IMode -ceq 'post-list-context'){$script:W24IWorkingRoot='foreign-root'}
    $Output=@($script:W24ISelector+': test');if($script:W24IMode -ceq 'list-duplicate'){$Output+=@('foreign::test: test')}
    return [pscustomobject]@{ExitCode=$(if($script:W24IMode -ceq 'list-exit'){9}else{0});Output=[string[]]$Output}
  }
  $Output=@('running 1 test',('test '+$script:W24ISelector+' ... ok'),'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s')
  if($script:W24IMode -ceq 'run-footer'){$Output[0]='running 0 tests'}
  return [pscustomobject]@{ExitCode=$(if($script:W24IMode -ceq 'run-exit'){101}else{0});Output=[string[]]$Output}
}
function Reset-W24IFacts([string]$Route,[string]$Mode) {
  $script:W24IRoute=$Route;$script:W24IMode=$Mode;$script:W24IWorkingRoot='owned-a';$script:W24ISeparator='/'
  $script:W24ICalls=[Collections.Generic.List[object]]::new();$script:W24IHooks=@{}
  $Original=[byte[]]@(1,2,3);$script:W24IMutation=[byte[]]@(1,2,4);$Digest=Get-ExactRustGenerationDigest $Original
  $Files=[Collections.Generic.SortedDictionary[string,string]]::new([StringComparer]::Ordinal);$Files.Add('src/owned.rs',$Digest)
  $Bound=[Collections.Generic.SortedDictionary[string,string]]::new([StringComparer]::Ordinal);$Bound.Add('src/owned.rs',$Digest)
  $script:W24IGeneration=[pscustomobject]@{Active=$true;Root='owned-a';MutableRoot='owned-b';OwnedRoot='owned-root';Target='owned-target';Cargo='owned-cargo';Files=$Files}
  $script:ExactRustMutationGenerationOwner=[pscustomobject]@{Instance=$script:W24IGeneration;Root='owned-a';MutableRoot='owned-b';OwnedRoot='owned-root';Target='owned-target';Cargo='owned-cargo';Files=$Bound}
  $script:W24IBytes=[Collections.Generic.Dictionary[string,byte[]]]::new([StringComparer]::Ordinal);$script:W24IBytes.Add('owned-a/src/owned.rs',$Original);$script:W24IBytes.Add('owned-b/src/owned.rs',$(if($Route -ceq 'b'){$script:W24IMutation}else{$Original}))
  $script:W24IWo22=[pscustomobject]@{Target='hum'}
  if($Mode -ceq 'foreign-context'){$script:W24IWorkingRoot='foreign-root'}
  if($Mode -ceq 'inactive'){$script:W24IGeneration.Active=$false}
  if($Mode -ceq 'foreign-owner'){$script:ExactRustMutationGenerationOwner.Instance=[pscustomobject]@{Active=$true}}
  if($Mode -ceq 'source-drift'){$script:W24IBytes['owned-b/src/owned.rs']=[byte[]]@(9)}
}
function Invoke-W24ISelection {
  if($script:W24IMode -ceq 'baseline-config'){Invoke-W24IBaselineTest 'owned B mutation' 'owned-cargo' $script:W24ISelector -Generation $script:W24IGeneration -MutationPath 'src/owned.rs' -MutationBytes $script:W24IMutation;return}
  if($script:W24IMode -ceq 'wrong-config'){Invoke-W24IWrongConfigTest 'owned B mutation' 'owned-cargo' $script:W24ISelector -Generation $script:W24IGeneration -MutationPath 'src/owned.rs' -MutationBytes $script:W24IMutation;return}
  if($script:W24IRoute -ceq 'b'){Invoke-ExactRustTest 'Work Order 24 N01 program-neutral feature identity initialized mutation' 'owned-cargo' $script:W24ISelector -Generation $script:W24IGeneration -MutationPath 'src/owned.rs' -MutationBytes $script:W24IMutation;return}
  if($script:W24IRoute -cin @('a-before','a-after')){Invoke-ExactRustTest ('owned '+$script:W24IRoute) 'owned-cargo' $script:W24ISelector -Generation $script:W24IGeneration;return}
  if($script:W24IRoute -ceq 'wo22'){Invoke-ExactRustTest 'owned WO22 mutation' 'owned-cargo' $script:W24ISelector -Wo22Admission $script:W24IWo22 -Wo22MutationBytes ([byte[]]@(7));return}
  Invoke-ExactRustTest 'owned ordinary fallback' 'owned-cargo' $script:W24ISelector
}
'@
  $Driver=@'
$script:W24ISelector='native_program::tests::native_feature_discrimination_is_typed_and_load_bearing'
$script:W24IAssertions=[Collections.Generic.List[object]]::new();$Cases=[Collections.Generic.List[object]]::new()
$Canonical=$script:ExactRustSelectorCredits;$CanonicalContents=if($null -eq $Canonical){@()}else{[string[]]$Canonical.ToArray()}
$SavedGenerationOwner=$script:ExactRustMutationGenerationOwner
try {
  foreach($Spec in @(
    @('b','honest','',2,1),@('a-before','honest','',2,1),@('a-after','honest','',2,1),@('null','honest','',2,1),@('wo22','honest','',2,1),
    @('b','foreign-context','WO24 Cargo generation lifetime or context changed',0,0),@('b','inactive','WO24 Cargo generation lifetime or context changed',0,0),@('b','foreign-owner','WO24 Cargo generation lifetime or context changed',0,0),
    @('b','source-drift','WO24 mutable Cargo source changed',0,0),@('b','post-list-context','WO24 Cargo generation lifetime or context changed',1,0),
    @('b','list-duplicate','must resolve',1,0),@('b','list-exit','cargo exited 9',1,0),@('b','run-exit','failed with exit code 101',2,0),@('b','run-footer','must run exactly one test',2,0),@('b','capture-throw','owned capture refused',2,0),
    @('b','baseline-config','WO24 incremental actual exact scoped argv refused',1,0),@('b','wrong-config','WO24 incremental actual exact scoped argv refused',1,0)
  )) {
    Reset-W24IFacts $Spec[0] $Spec[1];$Isolated=[Collections.Generic.List[string]]::new();$Isolated.Add('owned-ambient-credit');$script:ExactRustSelectorCredits=$Isolated;$Caught=''
    try{Invoke-W24ISelection}catch{$Caught=$_.Exception.Message}finally{$script:ExactRustSelectorCredits=$Canonical}
    Require-W24I (($Spec[2] -ceq '' -and $Caught -ceq '') -or ($Spec[2] -cne '' -and $Caught.Contains($Spec[2]))) ($Spec[1]+' actual public cause')
    Require-W24I ($script:W24ICalls.Count -eq $Spec[3] -and $Isolated.Count -eq (1+$Spec[4])) ($Spec[1]+' fail-first capture/real-credit counts')
    Require-W24I ([object]::ReferenceEquals($script:ExactRustSelectorCredits,$Canonical) -and [string]::Join('|',$CanonicalContents) -ceq $(if($null -eq $Canonical){''}else{[string]::Join('|',$Canonical.ToArray())})) ($Spec[1]+' canonical credit identity/order restored')
    if($Spec[1] -ceq 'honest' -and $Spec[0] -ceq 'b'){Require-W24I ($script:W24IHooks['read'] -eq 8 -and $script:W24IHooks['capture'] -eq 2 -and $script:W24IHooks['marker'] -gt 0) 'actual four generation guards and phase calls reached'}
    if($Spec[1] -cin @('baseline-config','wrong-config')) {
      $ExpectedPoison=@('test',$script:W24ISelector,'--manifest-path','owned-b/Cargo.toml','--target-dir','owned-target')
      if($Spec[1] -ceq 'wrong-config'){$ExpectedPoison+=@('--config','build.incremental=false')}
      $ExpectedPoison+=@('--','--exact','--list')
      Require-W24I ($Caught -ceq 'WO24 incremental actual exact scoped argv refused' -and $script:W24IHooks['argv-refusal'] -eq 1) ($Spec[1]+' expected actual argv-consumer refusal')
      Require-W24I ($script:W24ICalls[0].Cargo -ceq 'owned-cargo' -and [string]::Join('|',$script:W24ICalls[0].Arguments) -ceq [string]::Join('|',$ExpectedPoison)) ($Spec[1]+' exact single config corruption reached actual consumer')
    }
    $Cases.Add([pscustomobject]@{Route=$Spec[0];Mode=$Spec[1];Caught=$Caught;Calls=$script:W24ICalls.ToArray();Credits=$Isolated.ToArray();Hooks=$script:W24IHooks})
  }
  foreach($Mode in @('partial-credit','recovery')) {
    Reset-W24IFacts 'b' 'honest';$Isolated=[Collections.Generic.List[string]]::new();$Isolated.Add('owned-ambient-credit');$script:ExactRustSelectorCredits=$Isolated;$Caught=''
    try {
      if($Mode -ceq 'partial-credit'){Invoke-W24ISelection;$script:W24IMode='run-exit';try{Invoke-W24ISelection}catch{$Caught=$_.Exception.Message}}
      else{$script:W24IMode='list-duplicate';try{Invoke-W24ISelection}catch{$Caught=$_.Exception.Message};$script:W24IMode='honest';Invoke-W24ISelection}
    }finally{$script:ExactRustSelectorCredits=$Canonical}
    Require-W24I ($Caught.Contains($(if($Mode -ceq 'partial-credit'){'failed with exit code 101'}else{'must resolve'})) -and $Isolated.Count -eq 2 -and $Isolated[1] -ceq $script:W24ISelector) ($Mode+' actual failure/success/credit consumption')
    Require-W24I ($script:W24ICalls.Count -eq $(if($Mode -ceq 'partial-credit'){4}else{3})) ($Mode+' no extra requests')
    Require-W24I ([object]::ReferenceEquals($script:ExactRustSelectorCredits,$Canonical) -and [string]::Join('|',$CanonicalContents) -ceq $(if($null -eq $Canonical){''}else{[string]::Join('|',$Canonical.ToArray())})) ($Mode+' canonical identity/order restored')
    $Cases.Add([pscustomobject]@{Route='b';Mode=$Mode;Caught=$Caught;Calls=$script:W24ICalls.ToArray();Credits=$Isolated.ToArray();Hooks=$script:W24IHooks})
  }
}finally{$script:ExactRustSelectorCredits=$Canonical;$script:ExactRustMutationGenerationOwner=$SavedGenerationOwner}
[pscustomobject]@{Assertions=$script:W24IAssertions.ToArray();Cases=$Cases.ToArray()}
'@
  $Program=$Owned+"`n"+[string]::Join("`n",$Parts)+"`n"+$Driver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  $Gate=Assert-W24IZero $Program
  $Result=& ([scriptblock]::Create($Program))
  return [pscustomobject]@{Assertions=$Result.Assertions;Cases=$Result.Cases;Pins=$Pins.ToArray();Gate=$Gate;EvidenceKind='closed owned scalar/argv/source-guard evidence; no real native diagnostics';Wo22Limit='unchanged WO22 admission uses typed owned stand-in; no new WO22 admission credit'}
}


function Invoke-OwnedWo24StageTimingControls {
  param([string]$CheckAllText, [string]$EvidenceText, $SelectedProgram = $null)
  # Select current owned statements, not the mutation runner or native helper.
  function Select-Wo24Function([string]$Text, [string]$Name) {
    $Tokens = $null; $Errors = $null
    $Tree = [Management.Automation.Language.Parser]::ParseInput($Text, [ref]$Tokens, [ref]$Errors)
    if ($Errors.Count -ne 0) { throw 'WO24 focused source parse refused' }
    $Definitions = @($Tree.FindAll({ param($Node) $Node -is [Management.Automation.Language.FunctionDefinitionAst] -and $Node.Name -ceq $Name }, $true))
    if ($Definitions.Count -ne 1) { throw "WO24 focused definition refused: $Name" }
    return $Definitions[0]
  }
  $Runner = Select-Wo24Function $CheckAllText 'Invoke-Wo24UnitAProductionMutationEvidence'
  $Rows = @($Runner.Body.EndBlock.Statements | Where-Object { $_ -is [Management.Automation.Language.AssignmentStatementAst] -and $_.Left.Extent.Text -ceq '$Mutations' })
  $Maps = @($Runner.Body.EndBlock.Statements | Where-Object { $_ -is [Management.Automation.Language.AssignmentStatementAst] -and $_.Left.Extent.Text -ceq '$Wo24StageIds' })
  $Lifetimes = @($Runner.Body.EndBlock.Statements | Where-Object { $_ -is [Management.Automation.Language.TryStatementAst] })
  if ($Lifetimes.Count -ne 1 -or $Lifetimes[0].Finally.Statements.Count -ne 1 -or $Lifetimes[0].Finally.Statements[0] -isnot [Management.Automation.Language.TryStatementAst]) { throw 'WO24 focused generation lifetime ownership refused' }
  $Loops = @($Lifetimes[0].Body.Statements | Where-Object { $_ -is [Management.Automation.Language.ForEachStatementAst] -and $_.Variable.Extent.Text -ceq '$Mutation' })
  if ($Rows.Count -ne 1 -or $Maps.Count -ne 1 -or $Loops.Count -ne 1) { throw 'WO24 focused row/map/loop ownership refused' }
  $Loop = $Loops[0]
  $MapAdds = @($Runner.Body.EndBlock.Statements | Where-Object { $_.Extent.Text.StartsWith('$Wo24StageIds.Add(', [StringComparison]::Ordinal) })
  if ($MapAdds.Count -ne 8) { throw 'WO24 focused stage map count refused' }
  $MapText = $Maps[0].Extent.Text + "`n" + [string]::Join("`n", @($MapAdds | ForEach-Object { $_.Extent.Text }))
  $Bindings = @($Loop.Body.Statements | Where-Object { ($_.Extent.Text.StartsWith('if (-not $Wo24StageIds.ContainsKey', [StringComparison]::Ordinal)) -or ($_ -is [Management.Automation.Language.AssignmentStatementAst] -and $_.Left.Extent.Text -ceq '$Wo24StagePrefix') })
  if ($Bindings.Count -ne 2) { throw 'WO24 focused exact row binding refused' }
  $Stages = @($Loop.Body.Statements | Where-Object { $_ -is [Management.Automation.Language.TryStatementAst] })
  if ($Stages.Count -ne 3) { throw 'WO24 focused stage owner count refused' }
  $Commands = @($Runner.Body.FindAll({ param($Node) $Node -is [Management.Automation.Language.CommandAst] -and $Node.GetCommandName() -ceq 'Invoke-ExactRustTest' }, $true))
  $ExpectedCalls = @('Invoke-ExactRustTest $HonestBeforeLabel $Cargo $Mutation.Selector -Generation $Wo24Generation', 'Invoke-ExactRustTest $MutationExecutionLabel $Cargo $Mutation.Selector -Generation $Wo24Generation -MutationPath $Mutation.Path -MutationBytes ($Utf8.GetBytes($Mutated))', 'Invoke-ExactRustTest $HonestAfterLabel $Cargo $Mutation.Selector -Generation $Wo24Generation')
  if ($Commands.Count -ne 3) { throw 'WO24 focused guarded call count refused' }
  $StageText = [Collections.Generic.List[string]]::new()
  $StageKinds = @('before','mutation','after')
  for ($Index = 0; $Index -lt 3; $Index++) {
    $Stage = $Stages[$Index]
    $ActualCalls = @($Stage.Body.FindAll({ param($Node) $Node -is [Management.Automation.Language.CommandAst] -and $Node.GetCommandName() -ceq 'Invoke-ExactRustTest' }, $true))
    if ($ActualCalls.Count -ne 1 -or $ActualCalls[0].Extent.Text -cne $ExpectedCalls[$Index]) { throw 'WO24 focused stage action ownership refused' }
    $StageIndex = [array]::IndexOf($Loop.Body.Statements, $Stage)
    if ($StageIndex -lt 1) { throw 'WO24 focused start ownership refused' }
    $Start = $Loop.Body.Statements[$StageIndex - 1]
    $ExpectedStart = "Write-HumPhaseMarker (`$Wo24StagePrefix + '-$($StageKinds[$Index])') 'start'"
    $ExpectedEnd = "Write-HumPhaseMarker (`$Wo24StagePrefix + '-$($StageKinds[$Index])') 'end' 'completed'"
    $ExpectedFailed = "Write-HumPhaseMarker (`$Wo24StagePrefix + '-$($StageKinds[$Index])') 'end' 'failed'"
    if ($Start.Extent.Text -cne $ExpectedStart -or $Stage.Body.Statements[-1].Extent.Text -cne $ExpectedEnd -or $Stage.CatchClauses.Count -ne 1 -or $Stage.CatchClauses[0].Body.Statements.Count -ne 2 -or $Stage.CatchClauses[0].Body.Statements[0].Extent.Text -cne $ExpectedFailed -or $Stage.CatchClauses[0].Body.Statements[1].Extent.Text -cne 'throw') { throw 'WO24 focused marker/outcome ownership refused' }
    $StageText.Add($Start.Extent.Text + "`n" + $Stage.Extent.Text)
  }
  $ActualMarker = (Select-Wo24Function $CheckAllText 'Write-HumPhaseMarker').Extent.Text
  $ActualEqual = (Select-Wo24Function $CheckAllText 'Test-Wo22ByteArraysEqual').Extent.Text
  $ActualObserver = (Select-Wo24Function $EvidenceText 'Update-HumPhaseProgress').Extent.Text
  $ActualMarker = $ActualMarker.Replace('Test-Path','Test-Wo24OwnedPath').Replace('[IO.Directory]::CreateDirectory($PhaseDiagnostics)','(New-Wo24OwnedDirectory $PhaseDiagnostics)').Replace("[DateTime]::UtcNow.ToString('o')","'2026-10-13T00:00:00.0000000Z'").Replace('[Diagnostics.Stopwatch]::GetTimestamp()','(Get-Wo24OwnedTicks)').Replace('[Diagnostics.Stopwatch]::Frequency','1000').Replace('[IO.File]::ReadAllText($BindingPath)','(Read-Wo24OwnedText $BindingPath)').Replace('Join-Path','Join-Wo24OwnedPath').Replace('[IO.File]::WriteAllText($TempPath, $Content, [Text.Encoding]::UTF8)','Write-Wo24OwnedText $TempPath $Content').Replace('[IO.File]::Move($TempPath, $Path, $true)','Move-Wo24OwnedText $TempPath $Path').Replace('Write-Warning','Write-Wo24OwnedWarning')
  $ActualObserver = $ActualObserver.Replace('Test-Path','Test-Wo24OwnedPath').Replace('Get-ChildItem','Get-Wo24OwnedChildren').Replace('Join-Path','Join-Wo24OwnedPath').Replace('[IO.File]::ReadAllText($BindingPath)','(Read-Wo24OwnedText $BindingPath)').Replace('[IO.File]::ReadAllText($File.FullName)','(Read-Wo24OwnedText $File.FullName)').Replace('Write-Host','Write-Wo24OwnedHost')
  $ActualStages = [string]::Join("`n", $StageText).Replace('[System.IO.File]::WriteAllText($Path, $Mutated, $Utf8)','Write-Wo24OwnedMutation $Path $Mutated $Utf8').Replace('[System.IO.File]::WriteAllBytes($Path, $OriginalBytes)','Restore-Wo24OwnedMutation $Path $OriginalBytes').Replace('[System.IO.File]::ReadAllBytes($Path)','(Read-Wo24OwnedMutation $Path)').Replace("New-Object 'System.Collections.Generic.List[string]'",'[Collections.Generic.List[string]]::new()').Replace('$script:ExactRustSelectorCredits','$script:Wo24OwnedCredits').Replace('Write-Host','Write-Wo24OwnedHost')
  $OwnedHelpers = @'
function Add-Wo24Hook([string]$Name) { if (-not $script:Wo24Hooks.ContainsKey($Name)) { $script:Wo24Hooks[$Name] = 0 }; $script:Wo24Hooks[$Name]++ }
function Assert-Wo24Owned([bool]$Ok, [string]$Message) { $script:Wo24Assertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message}); if (-not $Ok) { throw "WO24 focused assertion refused: $Message" } }
function Test-Wo24OwnedPath { param([string]$LiteralPath) Add-Wo24Hook 'exists'; return $script:Wo24Dirs.Contains($LiteralPath) -or $script:Wo24Files.ContainsKey($LiteralPath) }
function Join-Wo24OwnedPath([string]$Parent,[string]$Leaf) { Add-Wo24Hook 'join'; return "$Parent/$Leaf" }
function New-Wo24OwnedDirectory([string]$Path) { Add-Wo24Hook 'mkdir'; $null=$script:Wo24Dirs.Add($Path) }
function Read-Wo24OwnedText([string]$Path) { Add-Wo24Hook 'read'; if ([string]::Equals($Path,$script:Wo24CorruptPath,[StringComparison]::Ordinal)) { Add-Wo24Hook 'read_corruption'; return $script:Wo24CorruptValue }; return $script:Wo24Files[$Path] }
function Write-Wo24OwnedText([string]$Path,[string]$Content) { Add-Wo24Hook 'write'; if ($script:Wo24Mode.StartsWith('sink-', [StringComparison]::Ordinal)) { Add-Wo24Hook 'write_refusal'; throw 'owned_marker_sink_refusal' }; $script:Wo24Files[$Path]=$Content }
function Move-Wo24OwnedText([string]$Source,[string]$Destination) { Add-Wo24Hook 'move'; $script:Wo24Files[$Destination]=$script:Wo24Files[$Source]; $null=$script:Wo24Files.Remove($Source) }
function Get-Wo24OwnedTicks { Add-Wo24Hook 'ticks'; $script:Wo24Ticks+=1000; return $script:Wo24Ticks }
function Get-Wo24OwnedChildren { param([string]$LiteralPath,[switch]$File) Add-Wo24Hook 'children'; foreach($Key in $script:Wo24Files.Keys) { if ($Key.StartsWith($LiteralPath+'/',[StringComparison]::Ordinal)) { [pscustomobject]@{Name=$Key.Substring($LiteralPath.Length+1);FullName=$Key} } } }
function Write-Wo24OwnedWarning([string]$Message) { Add-Wo24Hook 'warning'; $script:Wo24Warnings.Add($Message) }
function Write-Wo24OwnedHost { param([string]$Message,[switch]$NoNewline) Add-Wo24Hook 'host'; $script:Wo24Messages.Add($Message) }
function Get-ExactRustSelectorCredits { return @($script:Wo24OwnedCredits) }
function Write-Wo24OwnedMutation($Path,$Text,$Encoding) { Add-Wo24Hook 'mutate'; $script:Wo24SourceBytes=[Text.Encoding]::UTF8.GetBytes($Text) }
function Restore-Wo24OwnedMutation($Path,[byte[]]$Bytes) { Add-Wo24Hook 'restore'; $script:Wo24SourceBytes=$Bytes.Clone() }
function Read-Wo24OwnedMutation($Path) { Add-Wo24Hook 'source_read'; return ,$script:Wo24SourceBytes }
function Invoke-ExactRustTest([string]$Label,[string]$Cargo,[string]$Selector,$Generation,[string]$MutationPath,[byte[]]$MutationBytes) {
  Add-Wo24Hook 'selector'; $script:Wo24Actions.Add("$Label|$Cargo|$Selector")
  if ($Label.EndsWith('honest selector before mutation',[StringComparison]::Ordinal)) {
    if ($script:Wo24Mode -cin @('before-refusal','sink-before-refusal')) { throw 'owned_before_refusal' }
    $script:Wo24OwnedCredits.Add($Selector); return
  }
  if ($Label.EndsWith('honest selector after restoration',[StringComparison]::Ordinal)) {
    if ($script:Wo24Mode -ceq 'after-refusal') { throw 'owned_after_refusal' }
    if ($script:Wo24Mode -ceq 'after-credit') { $script:Wo24OwnedCredits.Add('wrong::credit'); return }
    $script:Wo24OwnedCredits.Add($Selector); return
  }
  if ($script:Wo24Mode -ceq 'green-mutant') { return }
  "test $Selector ... FAILED"
  if ($script:Wo24Mode -ceq 'wrong-disposition') { 'owned wrong disposition' } else { $Mutation.AssertionMessage }
  "$($Mutation.AssertionPath):10:1"
  throw "$Label failed with exit code 101"
}
'@
  $OwnedDriver = @'
function Invoke-Wo24OwnedCase([object]$Mutation,[string]$Mode) {
  $script:Wo24Mode=$Mode; $script:Wo24Files.Clear(); $script:Wo24Dirs.Clear(); $script:Wo24Actions.Clear(); $script:Wo24Messages.Clear(); $script:Wo24Warnings.Clear()
  $script:Wo24CorruptPath=''; $script:Wo24CorruptValue=''
  $null=$script:Wo24Dirs.Add('/wo24'); $script:Wo24Files['/wo24/run-binding.txt']="binding_guid=ownedwo24`n"
  $script:Wo24OwnedCredits.Clear(); $script:Wo24OwnedCredits.Add('ambient::credit')
  $OriginalBytes=[byte[]]@(11,22,33); $script:Wo24SourceBytes=$OriginalBytes.Clone()
  $Path='/owned/source'; $Mutated='owned-mutated'; $Utf8=[Text.Encoding]::UTF8; $Wo24Generation=$null; $Cargo='owned-cargo'; $AssertionStartLine=10; $AssertionMessageLine=10
  $Caught=''; try { Invoke-Wo24OwnedStages $Mutation } catch { $Caught=$_.Exception.Message }
  $Prefix='compiler-front-wo24-'+$Wo24StageIds[$Mutation.Label]
  $ExpectedError = if($Mode -cin @('before-refusal','sink-before-refusal')){'owned_before_refusal'} elseif($Mode -ceq 'after-refusal'){'owned_after_refusal'} elseif($Mode -ceq 'after-credit'){"Work Order 24 $($Mutation.Label) honest-after selector credit append drifted"} elseif($Mode -cin @('green-mutant','wrong-disposition')){"Work Order 24 $($Mutation.Label) did not fail its exact selector and restore bytes"} else {''}
  Assert-Wo24Owned ($Caught -ceq $ExpectedError) "$Mode/$($Mutation.Label): exact primary outcome"
  $ExpectedCount=if($Mode -cin @('before-refusal','sink-before-refusal')){1}elseif($Mode -cin @('green-mutant','wrong-disposition')){2}else{3}
  Assert-Wo24Owned ($script:Wo24Actions.Count -eq $ExpectedCount) "$Mode/$($Mutation.Label): exact action count"
  $Labels=@("Work Order 24 $($Mutation.Label) honest selector before mutation", "Work Order 24 $($Mutation.Label) initialized mutation", "Work Order 24 $($Mutation.Label) honest selector after restoration")
  for($Index=0;$Index -lt $ExpectedCount;$Index++){ Assert-Wo24Owned ($script:Wo24Actions[$Index] -ceq "$($Labels[$Index])|owned-cargo|$($Mutation.Selector)") "$Mode/$($Mutation.Label): owned action order $Index" }
  Assert-Wo24Owned (Test-Wo22ByteArraysEqual $OriginalBytes $script:Wo24SourceBytes) "$Mode/$($Mutation.Label): original bytes restored"
  $ExpectedCredits=if($Mode -ceq 'after-credit'){'ambient::credit|wrong::credit'}else{'ambient::credit'}
  Assert-Wo24Owned ([string]::Join('|',$script:Wo24OwnedCredits) -ceq $ExpectedCredits) "$Mode/$($Mutation.Label): original credit behavior"
  if($Mode.StartsWith('sink-',[StringComparison]::Ordinal)) {
    Assert-Wo24Owned ($script:Wo24Files.Count -eq 1 -and $script:Wo24Warnings.Count -eq (2*$ExpectedCount)) "${Mode}: marker sink failure is unavailable/nonfatal"
  } else {
    for($Index=0;$Index -lt $ExpectedCount;$Index++) {
      $Name=$Prefix+'-'+@('before','mutation','after')[$Index]
      $Start=$script:Wo24Files["/wo24/phase-$Name-start.txt"]; $End=$script:Wo24Files["/wo24/phase-$Name-end.txt"]
      $Outcome=if($Index -eq $ExpectedCount-1 -and $Caught.Length -gt 0){'failed'}else{'completed'}
      Assert-Wo24Owned ($Start.Contains("phase=$Name`n") -and $Start.Contains("binding=ownedwo24`n") -and $End.Contains("outcome=$Outcome`n")) "$Mode/${Name}: exact bound stage outcome"
      $ST=[int64]([regex]::Match($Start,'ticks=([0-9]+)').Groups[1].Value); $ET=[int64]([regex]::Match($End,'ticks=([0-9]+)').Groups[1].Value)
      Assert-Wo24Owned ($ST -lt $ET) "$Mode/${Name}: ordered actual producer ticks"
    }
    Assert-Wo24Owned ($script:Wo24Files.Count -eq (1+2*$ExpectedCount)) "$Mode/$($Mutation.Label): no future stage files"
    Update-HumPhaseProgress '/wo24' @{Binding='';Reported=[Collections.Generic.Dictionary[string,bool]]::new([StringComparer]::Ordinal)}
    for($Index=0;$Index -lt $ExpectedCount;$Index++) {
      $Name=$Prefix+'-'+@('before','mutation','after')[$Index]; $Outcome=if($Index -eq $ExpectedCount-1 -and $Caught.Length -gt 0){'failed'}else{'completed'}
      Assert-Wo24Owned ($script:Wo24Messages.Contains("phase_progress: started $Name") -and $script:Wo24Messages.Contains("phase_progress: $Name $Outcome")) "$Mode/${Name}: actual observer consumes outcome"
    }
  }
  $script:Wo24Cases.Add([pscustomobject]@{Label=$Mutation.Label;Mode=$Mode;Calls=$script:Wo24Actions.Count;Primary=$Caught;MarkerFiles=$script:Wo24Files.Count-1})
}
$script:Wo24Files=[Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
$script:Wo24Dirs=[Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
$script:Wo24Hooks=[Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
$script:Wo24OwnedCredits=[Collections.Generic.List[string]]::new()
$script:Wo24Actions=[Collections.Generic.List[string]]::new(); $script:Wo24Warnings=[Collections.Generic.List[string]]::new(); $script:Wo24Messages=[Collections.Generic.List[string]]::new()
$script:Wo24Assertions=[Collections.Generic.List[object]]::new(); $script:Wo24Cases=[Collections.Generic.List[object]]::new()
$script:Wo24Ticks=[int64]0; $PhaseDiagnostics='/wo24'
Assert-Wo24Owned ($Mutations.Count -eq 8 -and $Wo24StageIds.Count -eq 8) 'eight actual rows and exact row bindings'
$Names=[Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
$Saved=[Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
for($RowIndex=0;$RowIndex -lt 8;$RowIndex++) {
  $Mutation=$Mutations[$RowIndex]; $ExpectedId='n'+($RowIndex+1).ToString('00')
  Assert-Wo24Owned ($Wo24StageIds[$Mutation.Label] -ceq $ExpectedId) 'exact current row-to-phase identity'
  foreach($Stage in @('before','mutation','after')){ Assert-Wo24Owned ($Names.Add('compiler-front-wo24-'+$ExpectedId+'-'+$Stage)) 'fixed phase name is unique' }
  Invoke-Wo24OwnedCase $Mutation 'honest'
  if($RowIndex -eq 0){foreach($Pair in $script:Wo24Files.GetEnumerator()){$Saved[$Pair.Key]=$Pair.Value}}
}
Assert-Wo24Owned ($Names.Count -eq 24) 'exactly 24 fixed phase names'
foreach($Mode in @('before-refusal','green-mutant','wrong-disposition','after-refusal','after-credit','sink-honest','sink-before-refusal')){Invoke-Wo24OwnedCase $Mutations[0] $Mode}
$script:Wo24Files.Clear();foreach($Pair in $Saved.GetEnumerator()){$script:Wo24Files[$Pair.Key]=$Pair.Value}
foreach($Fact in @('binding','ticks')) {
  $script:Wo24Messages.Clear()
  $script:Wo24CorruptPath=if($Fact -ceq 'binding'){'/wo24/run-binding.txt'}else{'/wo24/phase-compiler-front-wo24-n01-before-end.txt'}
  $Raw=$script:Wo24Files[$script:Wo24CorruptPath]
  $script:Wo24CorruptValue=if($Fact -ceq 'binding'){"binding_guid=foreignwo24`n"}else{[regex]::Replace($Raw,'(?m)^ticks=[0-9]+$','ticks=-1')}
  Assert-Wo24Owned ($script:Wo24CorruptValue -cne $Raw) "${Fact}: authoritative return changed"
  $Before=if($script:Wo24Hooks.ContainsKey('read_corruption')){$script:Wo24Hooks['read_corruption']}else{0}
  Update-HumPhaseProgress '/wo24' @{Binding='';Reported=[Collections.Generic.Dictionary[string,bool]]::new([StringComparer]::Ordinal)}
  $Cause=if($Fact -ceq 'binding'){'binding_mismatch_foreign'}else{'ticks_not_numeric'}
  Assert-Wo24Owned ($script:Wo24Hooks['read_corruption'] -gt $Before -and $script:Wo24Files[$script:Wo24CorruptPath] -ceq $Raw) "${Fact}: nonzero actual reader corruption/raw preservation"
  Assert-Wo24Owned ($script:Wo24Messages.Contains("phase_progress: compiler-front-wo24-n01-before invalid ($Cause)") -and -not $script:Wo24Messages.Contains('phase_progress: compiler-front-wo24-n01-before completed')) "${Fact}: actual observer refuses exact public cause"
}
[pscustomobject]@{Assertions=$script:Wo24Assertions.ToArray();Cases=$script:Wo24Cases.ToArray();Hooks=$script:Wo24Hooks;UniqueNames=$Names.Count;StageCalls=$script:Wo24Hooks['selector'];ObserverCalls=$script:Wo24Hooks['children'];ProgramConsumer='actual three owned stage blocks plus unchanged actual producer/observer'}
'@
  $Program = $OwnedHelpers + "`n" + $ActualMarker + "`n" + $ActualEqual + "`n" + $ActualObserver + "`n" + $Rows[0].Extent.Text + "`n" + $MapText + "`nfunction Invoke-Wo24OwnedStages { param([object]`$Mutation)`n" + [string]::Join("`n", @($Bindings | ForEach-Object { $_.Extent.Text })) + "`n" + $ActualStages + "`n}`n" + $OwnedDriver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  # Fatal closed gate over the entire executable projection and every owned helper.
  $GateTokens=$null; $GateErrors=$null
  $GateTree=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$GateTokens,[ref]$GateErrors)
  if($GateErrors.Count -ne 0){throw 'WO24 whole-program parse gate refused'}
  $AllowedTypes=@('string','string[]','bool','int','int64','byte[]','object','pscustomobject','hashtable','switch','array','StringComparison','StringComparer','System.StringComparison','regex','Collections.Generic.List[string]','Collections.Generic.List[object]','Collections.Generic.Dictionary[string,string]','Collections.Generic.Dictionary[string,int]','Collections.Generic.Dictionary[string,bool]','Collections.Generic.HashSet[string]','Text.Encoding')
  $AllowedMethods=@('Add','Contains','ContainsKey','Remove','RemoveAt','Clear','Clone','GetEnumerator','StartsWith','EndsWith','Substring','Equals','Join','IsNullOrWhiteSpace','GetBytes','Match','Matches','Escape','IsMatch','Replace','Trim','ToString','new','IndexOf','ToArray')
  $AllowedCommands=@('Where-Object','ForEach-Object','Sort-Object') + @($GateTree.FindAll({param($Node)$Node -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
  foreach($Node in $GateTree.FindAll({param($Node)$true},$true)) {
    if($Node -is [Management.Automation.Language.VariableExpressionAst] -and ($Node.VariablePath.IsDriveQualified -and $Node.VariablePath.DriveName -cne 'script')){throw 'WO24 whole-program environment/provider gate refused'}
    if(($Node -is [Management.Automation.Language.TypeExpressionAst] -or $Node -is [Management.Automation.Language.TypeConstraintAst]) -and $Node.TypeName.FullName -cnotin $AllowedTypes){throw "WO24 whole-program closed type gate refused: $($Node.TypeName.FullName)"}
    if($Node -is [Management.Automation.Language.AttributeAst] -and $Node.TypeName.FullName -cne 'AllowEmptyCollection'){throw 'WO24 whole-program closed attribute gate refused'}
    if($Node -is [Management.Automation.Language.FileRedirectionAst]){throw 'WO24 whole-program file redirection gate refused'}
    if($Node -is [Management.Automation.Language.InvokeMemberExpressionAst] -and ($Node.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $Node.Member.Value -cnotin $AllowedMethods)){throw 'WO24 whole-program closed member gate refused'}
    if($Node -is [Management.Automation.Language.CommandAst]) {
      $CommandName=$Node.GetCommandName()
      if($null -eq $CommandName){if($Node.CommandElements[0] -isnot [Management.Automation.Language.ScriptBlockExpressionAst]){throw 'WO24 whole-program dynamic command gate refused'}}
      elseif($CommandName -cnotin $AllowedCommands){throw "WO24 whole-program closed command gate refused: $CommandName"}
    }
  }
  $Result = . ([scriptblock]::Create($Program))
  if(@($Result).Count -ne 1){throw 'WO24 focused result cardinality refused'}
  $Result | Add-Member -NotePropertyName WholeProgramGate -NotePropertyValue 'ZERO real environment/FS/provider/process/native/console/unknown accessors; closed AST types/commands/members'
  $Result | Add-Member -NotePropertyName SelectedProgram -NotePropertyValue $Program
  return $Result
}

function Invoke-OwnedWo23AuthorityProofControls {
  param([string]$CheckAllText, $SelectedProgram = $null)
  # Metadata is source text; only the closed, accessor-free projection executes.
  $Tokens=$null; $Errors=$null
  $Ast=[Management.Automation.Language.Parser]::ParseInput($CheckAllText,[ref]$Tokens,[ref]$Errors)
  if($Errors.Count -ne 0){throw 'WO23 authority source parse refused'}
  $Functions=@($Ast.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true))
  $Privacy=@($Functions|Where-Object{$_.Name -ceq 'Invoke-HumCompilerPrivacyChecks'})
  $Reader=@($Functions|Where-Object{$_.Name -ceq 'Read-NativeOutputWithExit'})
  if($Privacy.Count -ne 1 -or $Reader.Count -ne 1){throw 'WO23 authority source owner cardinality refused'}
  $Statements=@($Privacy[0].Body.EndBlock.Statements)
  $First=@($Statements|Where-Object{$_.Extent.Text -ceq "Write-Host '==> Work Order 23 VerifiedIntegerSignBackendInput compiler-owned authority proof'"})
  $Last=@($Statements|Where-Object{$_.Extent.Text -ceq "Invoke-Native 'Work Order 23 normal compiler check after authority proof' `$Cargo @('check', '--all-targets')"})
  if($First.Count -ne 1 -or $Last.Count -ne 1){throw 'WO23 authority statement cardinality refused'}
  $Start=$First[0].Extent.StartOffset; $End=$Last[0].Extent.EndOffset
  $Actual=$CheckAllText.Substring($Start,$End-$Start)
  $Pins=[Collections.Generic.List[object]]::new()
  foreach($Part in @([pscustomobject]@{Name='WO23 authority stanza';Start=$Start;End=$End;Text=$Actual},[pscustomobject]@{Name='Read-NativeOutputWithExit';Start=$Reader[0].Extent.StartOffset;End=$Reader[0].Extent.EndOffset;Text=$Reader[0].Extent.Text})){
    $Hash=[Security.Cryptography.SHA256]::Create();try{$Digest=[BitConverter]::ToString($Hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Part.Text))).Replace('-','').ToLowerInvariant()}finally{$Hash.Dispose()}
    $Pins.Add([pscustomobject]@{Name=$Part.Name;StartUTF16=$Part.Start;EndUTF16=$Part.End;UTF8TextSha256=$Digest})
  }
  $Projected=$Actual.Replace("`$env:RUSTFLAGS = ''","Set-Wo23OwnedFlags ''").Replace('$env:RUSTFLAGS = $Wo23PriorRustFlags','Set-Wo23OwnedFlags $Wo23PriorRustFlags').Replace('Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue','Remove-Wo23OwnedFlags').Replace('$env:RUSTFLAGS','(Get-Wo23OwnedFlags)')
  $ActualReader=$Reader[0].Extent.Text.Replace('$Output = & $FilePath @Arguments 2>&1','$Output = Invoke-Wo23OwnedCapture $FilePath $Arguments').Replace('return [pscustomobject] @{','$Fact = [pscustomobject] @{')
  $ActualReader=$ActualReader.Substring(0,$ActualReader.Length-1)+"  return Complete-Wo23OwnedFact `$Fact`n}"
  $Owned=@'
$script:W23Assertions=[Collections.Generic.List[object]]::new()
$script:W23Cases=[Collections.Generic.List[object]]::new()
$script:W23Hooks=[Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
function Hit-Wo23Owned([string]$Name){if(-not $script:W23Hooks.ContainsKey($Name)){$script:W23Hooks.Add($Name,0)};$script:W23Hooks[$Name]++}
function Assert-Wo23Owned([bool]$Ok,[string]$Message){$script:W23Assertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message});if(-not $Ok){throw ('WO23 owned assertion: '+$Message)}}
function Write-Host {param([string]$Message) Hit-Wo23Owned 'public'}
function Get-Wo23OwnedFlags {Hit-Wo23Owned 'context-read';return $script:W23Context['RUSTFLAGS']}
function Set-Wo23OwnedFlags([AllowEmptyString()][string]$Value){
  Hit-Wo23Owned 'context-write';$script:W23Writes++
  if($script:W23Mode -ceq 'clear-throws' -and $script:W23Writes -eq 1){throw 'owned-context-set-failure'}
  if($script:W23Writes -gt 1 -and $script:W23Mode -ceq 'restore-throws'){throw 'owned-restoration-failure'}
  $script:W23Context['RUSTFLAGS']=if($script:W23Writes -gt 1 -and $script:W23Mode -ceq 'restore-corruption'){'corrupted-restoration'}else{$Value}
}
function Remove-Wo23OwnedFlags {
  Hit-Wo23Owned 'context-remove'
  if($script:W23Mode -ceq 'restore-corruption'){$script:W23Context['RUSTFLAGS']='corrupted-restoration'}else{$null=$script:W23Context.Remove('RUSTFLAGS')}
}
function Invoke-Wo23OwnedCapture([string]$FilePath,[string[]]$Arguments){
  Hit-Wo23Owned 'native-request';$script:W23Requests.Add([pscustomobject]@{File=$FilePath;Arguments=$Arguments;Flags=$script:W23Context['RUSTFLAGS']})
  $Index=$script:W23Requests.Count-1
  $Roles=@('construction','lifetime','substitution')
  $Expected='rustc|--profile|check|--bin|hum|--|--cfg|hum_compile_fail_verified_integer_sign_backend_input_'+$Roles[$Index]
  if($FilePath -cne 'owned-cargo' -or [string]::Join('|',$Arguments) -cne $Expected -or $script:W23Context['RUSTFLAGS'] -cne ''){throw 'owned-route-refusal'}
  if($script:W23Mode -ceq ('producer-'+[string]$Index) -or ($script:W23Mode -ceq 'restore-throws' -and $Index -eq 0)){throw 'owned-producer-failure'}
  Hit-Wo23Owned 'capture-output';$global:LASTEXITCODE=101
  $Text=switch($Index){
    0 {'prefix error[E0451]: fields `projection` and `_artifact` of struct `VerifiedIntegerSignBackendInput` are private'+"`n"+' projection: None.unwrap() trailing'}
    1 {'lifetime may not live long enough'+"`n"+'|capability| capability'+"`n"+' return type of closure is VerifiedIntegerSignBackendInput'}
    2 {'error[E0308] first'+"`n"+'found reference `&[u8]`'+"`n"+'error[E0308] second'+"`n"+'found reference `&VerifiedBackendInput'+"`n"+'execute_integer_sign execute_integer_sign execute_integer_sign execute_integer_sign'}
  }
  return $Text -split "`n"
}
function Complete-Wo23OwnedFact([object]$Fact){
  Hit-Wo23Owned 'returned-fact';$Index=$script:W23Requests.Count-1
  Assert-Wo23Owned ($Fact.Output -is [string] -and $Fact.Output.Contains("`n")) 'actual reader emits scalar joined diagnostic'
  if($script:W23Mode -ceq 'exit-corruption' -and $Index -eq 0){$Fact.ExitCode=0}
  if($script:W23Mode -ceq 'construction-cause' -and $Index -eq 0){$Fact.Output=$Fact.Output.Replace('projection: None.unwrap()','wrong span')}
  if($script:W23Mode -ceq 'lifetime-cause' -and $Index -eq 1){$Fact.Output='unrelated lifetime result'}
  if($script:W23Mode -ceq 'substitution-count' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('error[E0308] second','no second error')}
  if($script:W23Mode -ceq 'unrelated-error' -and $Index -eq 2){$Fact.Output+="`nerror[E0433]: unresolved import"}
  if($script:W23Mode -ceq 'array-output' -and $Index -eq 0){$Fact.Output=@($Fact.Output)}
  return $Fact
}
function Invoke-Native([string]$Label,[string]$FilePath,[string[]]$Arguments){
  Hit-Wo23Owned 'honest-check'
  Assert-Wo23Owned ($Label -ceq 'Work Order 23 normal compiler check after authority proof' -and $FilePath -ceq 'owned-cargo' -and [string]::Join('|',$Arguments) -ceq 'check|--all-targets') 'unchanged honest all-target argv'
  Assert-Wo23Owned ([string]::Equals($script:W23Context['RUSTFLAGS'],$script:W23Prior,[StringComparison]::Ordinal) -and $script:W23Context.ContainsKey('RUSTFLAGS') -eq $script:W23HadPrior) 'honest check sees exact prior context'
}
'@
  $Driver=@'
foreach($Case in @(
  @('absent','positive'),@('empty','positive'),@('present','positive'),
  @('absent','producer-0'),@('empty','producer-1'),@('present','producer-2'),
  @('present','clear-throws'),@('present','restore-throws'),
  @('absent','restore-corruption'),@('empty','restore-corruption'),@('present','restore-corruption'),
  @('absent','construction-cause'),@('empty','lifetime-cause'),@('present','substitution-count'),
  @('present','exit-corruption'),@('present','unrelated-error'),@('present','array-output'),
  @('present','wrong-flag'),@('present','extra-argv'),@('present','wrong-target')
)){
  $PriorKind=$Case[0];$script:W23Mode=$Case[1]
  $script:W23Context=[Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
  $script:W23HadPrior=$PriorKind -cne 'absent';$script:W23Prior=if($PriorKind -ceq 'empty'){''}elseif($PriorKind -ceq 'present'){'--cfg inherited_parent --codegen debuginfo=1'}else{$null}
  if($script:W23HadPrior){$script:W23Context['RUSTFLAGS']=$script:W23Prior}
  $script:W23Writes=0;$script:W23Requests=[Collections.Generic.List[object]]::new()
  $Caught='';$BeforeHonest=if($script:W23Hooks.ContainsKey('honest-check')){$script:W23Hooks['honest-check']}else{0}
  try{
    if($script:W23Mode -ceq 'wrong-flag'){Invoke-Wo23OwnedWrongFlag}
    elseif($script:W23Mode -ceq 'extra-argv'){Invoke-Wo23OwnedExtraArgv}
    elseif($script:W23Mode -ceq 'wrong-target'){Invoke-Wo23OwnedWrongTarget}
    else{Invoke-Wo23OwnedActual}
  }catch{$Caught=$_.Exception.Message}
  $Expected=switch -CaseSensitive($script:W23Mode){
    'positive' {''}
    {$_ -cin @('producer-0','producer-1','producer-2')} {'owned-producer-failure'}
    'clear-throws' {'owned-context-set-failure'}
    'restore-throws' {'owned-restoration-failure'}
    'restore-corruption' {'Work Order 23 integer-sign authority proof did not restore RUSTFLAGS'}
    {$_ -cin @('construction-cause','exit-corruption','array-output')} {'Work Order 23 integer-sign construction proof lost its field-privacy diagnostic or primary span'}
    'lifetime-cause' {'Work Order 23 integer-sign lifetime proof lost its callback-owned escape diagnostic or primary span'}
    'substitution-count' {'Work Order 23 integer-sign substitution proof lost its two backend-consumer type diagnostics or primary spans'}
    'unrelated-error' {'Work Order 23 integer-sign authority proof failed for an unrelated symbol, cfg, import, or syntax reason'}
    default {'owned-route-refusal'}
  }
  Assert-Wo23Owned ($Caught -ceq $Expected) ($PriorKind+'/'+$script:W23Mode+' exact public outcome')
  if($script:W23Mode -cnotin @('restore-corruption','restore-throws')){
    Assert-Wo23Owned ([string]::Equals($script:W23Context['RUSTFLAGS'],$script:W23Prior,[StringComparison]::Ordinal) -and $script:W23Context.ContainsKey('RUSTFLAGS') -eq $script:W23HadPrior) ($PriorKind+'/'+$script:W23Mode+' exact finally restoration')
  }
  $AfterHonest=if($script:W23Hooks.ContainsKey('honest-check')){$script:W23Hooks['honest-check']}else{0}
  Assert-Wo23Owned ($AfterHonest-$BeforeHonest -eq $(if($script:W23Mode -ceq 'positive'){1}else{0})) ($script:W23Mode+' honest check not credited after refusal')
  $ExpectedCalls=if($script:W23Mode -ceq 'clear-throws'){0}elseif($script:W23Mode -ceq 'producer-0' -or $script:W23Mode -ceq 'restore-throws' -or $script:W23Mode -cin @('wrong-flag','extra-argv','wrong-target')){1}elseif($script:W23Mode -ceq 'producer-1'){2}else{3}
  Assert-Wo23Owned ($script:W23Requests.Count -eq $ExpectedCalls) ($script:W23Mode+' exact proof request prefix')
  $script:W23Cases.Add([pscustomobject]@{Prior=$PriorKind;Mode=$script:W23Mode;Primary=$Caught;Requests=$script:W23Requests.ToArray();HonestChecks=$AfterHonest-$BeforeHonest;OutcomeOwner=if($Expected.StartsWith('owned-',[StringComparison]::Ordinal)){'owned boundary sink'}elseif($Expected -ceq ''){'actual complete stanza'}else{'actual production refusal'}})
}
foreach($Name in @('public','context-read','context-write','context-remove','native-request','capture-output','returned-fact','honest-check')){Assert-Wo23Owned ($script:W23Hooks.ContainsKey($Name) -and $script:W23Hooks[$Name] -gt 0) ($Name+' nonzero actual boundary')}
[pscustomobject]@{Assertions=$script:W23Assertions.ToArray();Cases=$script:W23Cases.ToArray();Hooks=$script:W23Hooks}
'@
  $FunctionText="function Invoke-Wo23OwnedActual { `$Cargo='owned-cargo'`n"+$Projected+"`n}`n"
  $WrongFlag=$FunctionText.Replace('Invoke-Wo23OwnedActual','Invoke-Wo23OwnedWrongFlag').Replace('hum_compile_fail_verified_integer_sign_backend_input_construction','unowned_cfg')
  $ExtraArgv=$FunctionText.Replace('Invoke-Wo23OwnedActual','Invoke-Wo23OwnedExtraArgv').Replace("'--cfg', 'hum_compile_fail_verified_integer_sign_backend_input_construction'","'--cfg', 'hum_compile_fail_verified_integer_sign_backend_input_construction', '--release'")
  $WrongTarget=$FunctionText.Replace('Invoke-Wo23OwnedActual','Invoke-Wo23OwnedWrongTarget').Replace("'--bin', 'hum'","'--bin', 'other'")
  $Program=$Owned+"`n"+$ActualReader+"`n"+$FunctionText+$WrongFlag+$ExtraArgv+$WrongTarget+$Driver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  $GateTokens=$null;$GateErrors=$null;$Gate=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$GateTokens,[ref]$GateErrors)
  $Types=@('string','string[]','object','bool','pscustomobject','StringComparison','StringComparer','System.StringComparison','regex','Collections.Generic.List[object]','Collections.Generic.Dictionary[string,int]','Collections.Generic.Dictionary[string,string]')
  $Methods=@('Add','ContainsKey','Remove','Contains','Equals','Join','Replace','StartsWith','ToArray','Matches','IsMatch','new')
  $Properties=@('Count','Output','ExitCode','Exception','Message')
  $Commands=@($Gate.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
  $Unknown=[Collections.Generic.List[string]]::new()
  if($GateErrors.Count -ne 0){$Unknown.Add('parse')}
  foreach($N in $Gate.FindAll({param($N)$true},$true)){
    if(($N -is [Management.Automation.Language.TypeExpressionAst] -or $N -is [Management.Automation.Language.TypeConstraintAst]) -and $N.TypeName.FullName -cnotin $Types){$Unknown.Add('type:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.AttributeAst] -and $N.TypeName.FullName -cne 'AllowEmptyString'){$Unknown.Add('attribute:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.CommandAst] -and $N.GetCommandName() -cnotin $Commands){$Unknown.Add('command:'+$N.Extent.Text)}
    if($N -is [Management.Automation.Language.InvokeMemberExpressionAst]){if($N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $N.Member.Value -cnotin $Methods){$Unknown.Add('method:'+$N.Member.Extent.Text)}}
    elseif($N -is [Management.Automation.Language.MemberExpressionAst]){if($N.Static){if($N.Member.Value -cne 'Ordinal'){$Unknown.Add('static:'+ $N.Member.Extent.Text)}}elseif($N.Member.Value -cnotin $Properties){$Unknown.Add('property:'+$N.Member.Extent.Text)}}
    if($N -is [Management.Automation.Language.VariableExpressionAst] -and $N.VariablePath.IsDriveQualified -and -not $N.VariablePath.UserPath.StartsWith('script:',[StringComparison]::Ordinal) -and $N.VariablePath.UserPath -cne 'global:LASTEXITCODE'){$Unknown.Add('drive:'+$N.VariablePath.UserPath)}
    if($N -is [Management.Automation.Language.FileRedirectionAst]){$Unknown.Add('redirection')}
  }
  if($Unknown.Count -ne 0){throw ('WO23 fatal selected-program ZERO gate refused: '+[string]::Join('|',$Unknown))}
  $Result=& ([scriptblock]::Create($Program))
  return [pscustomobject]@{Assertions=$Result.Assertions;Cases=$Result.Cases;Hooks=$Result.Hooks;Pins=$Pins.ToArray();Gate=[pscustomobject]@{ParseErrors=0;Unknown=0;RealAccessors=0;CompiledPrograms=1}}
}

function Invoke-OwnedWo22AuthorityProofControls {
  param([string]$CheckAllText, $SelectedProgram = $null)
  # Metadata is source text; only the closed, accessor-free projection executes.
  $Tokens=$null; $Errors=$null
  $Ast=[Management.Automation.Language.Parser]::ParseInput($CheckAllText,[ref]$Tokens,[ref]$Errors)
  if($Errors.Count -ne 0){throw 'WO22 authority source parse refused'}
  $Functions=@($Ast.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true))
  $Privacy=@($Functions|Where-Object{$_.Name -ceq 'Invoke-HumCompilerPrivacyChecks'})
  $Reader=@($Functions|Where-Object{$_.Name -ceq 'Read-NativeOutputWithExit'})
  if($Privacy.Count -ne 1 -or $Reader.Count -ne 1){throw 'WO22 authority source owner cardinality refused'}
  $Statements=@($Privacy[0].Body.EndBlock.Statements)
  $First=@($Statements|Where-Object{$_.Extent.Text -ceq "Write-Host '==> Work Order 22 source-bound VerifiedBackendInput authority proof'"})
  $Last=@($Statements|Where-Object{$_.Extent.Text -ceq "Invoke-Native 'Work Order 22 Unit B normal compiler check after compile proofs' `$Cargo @('check', '--all-targets')"})
  if($First.Count -ne 1 -or $Last.Count -ne 1){throw 'WO22 authority statement cardinality refused'}
  $Start=$First[0].Extent.StartOffset; $End=$Last[0].Extent.EndOffset
  $Actual=$CheckAllText.Substring($Start,$End-$Start)
  $Pins=[Collections.Generic.List[object]]::new()
  foreach($Part in @([pscustomobject]@{Name='WO22 authority stanza';Start=$Start;End=$End;Text=$Actual},[pscustomobject]@{Name='Read-NativeOutputWithExit';Start=$Reader[0].Extent.StartOffset;End=$Reader[0].Extent.EndOffset;Text=$Reader[0].Extent.Text})){
    $Hash=[Security.Cryptography.SHA256]::Create();try{$Digest=[BitConverter]::ToString($Hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Part.Text))).Replace('-','').ToLowerInvariant()}finally{$Hash.Dispose()}
    $Pins.Add([pscustomobject]@{Name=$Part.Name;StartUTF16=$Part.Start;EndUTF16=$Part.End;UTF8TextSha256=$Digest})
  }
  $Projected=$Actual.Replace("`$env:RUSTFLAGS = ''","Set-Wo22ProofOwnedFlags ''").Replace('$env:RUSTFLAGS = $Wo22PriorRustFlags','Restore-Wo22ProofOwnedFlags $Wo22PriorRustFlags').Replace('$env:RUSTFLAGS = $Wo22UnitBPriorRustFlags','Restore-Wo22ProofOwnedFlags $Wo22UnitBPriorRustFlags').Replace('Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue','Remove-Wo22ProofOwnedFlags').Replace('$env:RUSTFLAGS','(Get-Wo22ProofOwnedFlags)')
  $ActualReader=$Reader[0].Extent.Text.Replace('$Output = & $FilePath @Arguments 2>&1','$Output = Invoke-Wo22ProofOwnedCapture $FilePath $Arguments').Replace('return [pscustomobject] @{','$Fact = [pscustomobject] @{')
  $ActualReader=$ActualReader.Substring(0,$ActualReader.Length-1)+"  return Complete-Wo22ProofOwnedFact `$Fact`n}"
  $Owned=@'
$script:W22ProofAssertions=[Collections.Generic.List[object]]::new()
$script:W22ProofCases=[Collections.Generic.List[object]]::new()
$script:W22ProofHooks=[Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
function Hit-Wo22ProofOwned([string]$Name){if(-not $script:W22ProofHooks.ContainsKey($Name)){$script:W22ProofHooks.Add($Name,0)};$script:W22ProofHooks[$Name]++}
function Assert-Wo22ProofOwned([bool]$Ok,[string]$Message){$script:W22ProofAssertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message});if(-not $Ok){throw ('WO22 proof owned assertion: '+$Message)}}
function Write-Host {param([string]$Message) Hit-Wo22ProofOwned 'public'}
function Get-Wo22ProofOwnedFlags {Hit-Wo22ProofOwned 'context-read';return $script:W22ProofContext['RUSTFLAGS']}
function Set-Wo22ProofOwnedFlags([AllowEmptyString()][string]$Value){
  Hit-Wo22ProofOwned 'context-clear'
  if($script:W22ProofMode -ceq 'clear-throws'){throw 'owned-context-set-failure'}
  Assert-Wo22ProofOwned ($Value -ceq '') 'proof-local flags overwrite empty'
  $script:W22ProofContext['RUSTFLAGS']=$Value
}
function Restore-Wo22ProofOwnedFlags([AllowEmptyString()][string]$Value){
  Hit-Wo22ProofOwned 'context-restore'
  if($script:W22ProofMode -ceq 'restore-throws'){throw 'owned-restoration-failure'}
  $script:W22ProofContext['RUSTFLAGS']=if($script:W22ProofMode -ceq 'restore-corruption' -and $script:W22ProofRequests.Count -eq 2){'corrupted-restoration'}else{$Value}
}
function Remove-Wo22ProofOwnedFlags {Hit-Wo22ProofOwned 'context-remove';$null=$script:W22ProofContext.Remove('RUSTFLAGS')}
function Invoke-Wo22ProofOwnedCapture([string]$FilePath,[string[]]$Arguments){
  Hit-Wo22ProofOwned 'native-request';$script:W22ProofRequests.Add([pscustomobject]@{File=$FilePath;Arguments=$Arguments;Flags=$script:W22ProofContext['RUSTFLAGS']})
  $Index=$script:W22ProofRequests.Count-1
  $Flags=@('hum_compile_fail_verified_backend_input_construction','hum_compile_fail_verified_backend_input_authority','hum_compile_fail_backend_adapter_raw_inputs','hum_compile_fail_backend_fault_seam_in_production')
  $Expected='rustc|--profile|check|--bin|hum|--|--cfg|'+$Flags[$Index]
  if($FilePath -cne 'owned-cargo' -or [string]::Join('|',$Arguments) -cne $Expected -or $script:W22ProofContext['RUSTFLAGS'] -cne ''){throw 'owned-route-refusal'}
  if($script:W22ProofMode -ceq ('producer-'+[string]$Index) -or $script:W22ProofMode -ceq 'restore-throws'){throw 'owned-producer-failure'}
  Hit-Wo22ProofOwned 'capture-output';$global:LASTEXITCODE=101
  $Text=switch($Index){
    0 {'prefix error[E0451] first'+"`n"+'fields `projection` and `_artifact` of struct `VerifiedBackendInput` are private'+"`n"+'error[E0451] second'+"`n"+'fields `expected`, `observed` and `program_identity` of struct `LiveIdentityRequest` are private trailing'}
    1 {'prefix struct `CanonicalMinimalAddBackendFacts` is private'+"`n"+'field `projection` of struct `VerifiedBackendInput` is private'+"`n"+'the trait bound `VerifiedBackendInput<''_>: From<&[u8]>` is not satisfied'+"`n"+'found `CanonicalBackendInputArtifact`'+"`n"+'found `IrVerifyReport`'+"`n"+'found `&[u8]`'+"`n"+'expected `&mut LiveIdentityRequest<''_>`, found `&mut ()`'+"`n"+'borrowed data escapes outside of closure'+"`n"+'error[E0308] one error[E0308] two error[E0308] three error[E0308] four'+"`n"+'error[E0521] trailing'}
    2 {'prefix error[E0308] one error[E0308] two error[E0308] three'+"`n"+'found reference `&Vec<u8>`'+"`n"+'found reference `&Program`'+"`n"+'found reference `&(i64, i64)` trailing'}
    3 {'prefix error[E0433]'+"`n"+'could not find `BackendProbeFault` in `super` trailing'}
  }
  return $Text -split "`n"
}
function Complete-Wo22ProofOwnedFact([object]$Fact){
  Hit-Wo22ProofOwned 'returned-fact';$Index=$script:W22ProofRequests.Count-1
  Assert-Wo22ProofOwned ($Fact.Output -is [string] -and $Fact.Output.Contains("`n")) 'actual reader emits scalar joined diagnostic'
  if($script:W22ProofMode -ceq 'construction-exit' -and $Index -eq 0){$Fact.ExitCode=0}
  if($script:W22ProofMode -ceq 'construction-count' -and $Index -eq 0){$Fact.Output=$Fact.Output.Replace('error[E0451] second','wrong code')}
  if($script:W22ProofMode -ceq 'construction-fields' -and $Index -eq 0){$Fact.Output=$Fact.Output.Replace('fields `projection` and `_artifact` of struct `VerifiedBackendInput` are private','wrong fields')}
  if($script:W22ProofMode -ceq 'construction-unrelated' -and $Index -eq 0){$Fact.Output+="`nerror[E0433]: unresolved import"}
  if($script:W22ProofMode -ceq 'authority-cause' -and $Index -eq 1){$Fact.Output=$Fact.Output.Replace('borrowed data escapes outside of closure','wrong cause')}
  if($script:W22ProofMode -ceq 'authority-count' -and $Index -eq 1){$Fact.Output=$Fact.Output.Replace('error[E0308] four','wrong code')}
  if($script:W22ProofMode -ceq 'authority-lifetime' -and $Index -eq 1){$Fact.Output=$Fact.Output.Replace('error[E0521]','wrong code')}
  if($script:W22ProofMode -ceq 'authority-unrelated' -and $Index -eq 1){$Fact.Output+="`nerror[E0433]: unresolved import"}
  if($script:W22ProofMode -ceq 'raw-count' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('error[E0308] three','wrong code')}
  if($script:W22ProofMode -ceq 'raw-type' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('found reference `&Program`','wrong type')}
  if($script:W22ProofMode -ceq 'fault-cause' -and $Index -eq 3){$Fact.Output=$Fact.Output.Replace('could not find `BackendProbeFault` in `super`','wrong cause')}
  if($script:W22ProofMode -ceq 'fault-count' -and $Index -eq 3){$Fact.Output+=' error[E0433]'}
  if($script:W22ProofMode -ceq 'array-output' -and $Index -eq 0){$Fact.Output=@($Fact.Output)}
  return $Fact
}
function Invoke-Native([string]$Label,[string]$FilePath,[string[]]$Arguments){
  Hit-Wo22ProofOwned 'honest-check'
  $Labels=@('Work Order 22 normal compiler check before authority proof','Work Order 22 normal compiler check after authority proof','Work Order 22 Unit B normal compiler check after compile proofs')
  Assert-Wo22ProofOwned ($Label -ceq $Labels[$script:W22ProofHonest.Count] -and $FilePath -ceq 'owned-cargo' -and [string]::Join('|',$Arguments) -ceq 'check|--all-targets') 'unchanged honest all-target argv and order'
  Assert-Wo22ProofOwned ([string]::Equals($script:W22ProofContext['RUSTFLAGS'],$script:W22ProofPrior,[StringComparison]::Ordinal) -and $script:W22ProofContext.ContainsKey('RUSTFLAGS') -eq $script:W22ProofHadPrior) 'honest check sees exact prior context'
  $script:W22ProofHonest.Add($Label)
}
'@
  $Driver=@'
$Cases=[Collections.Generic.List[object]]::new()
foreach($Kind in @('absent','empty','present')){
  $Cases.Add(@($Kind,'positive',4,3,''))
  foreach($Index in @(0,1,2,3)){$Cases.Add(@($Kind,('producer-'+$Index),($Index+1),$(if($Index -lt 2){1}else{2}),'owned-producer-failure'))}
}
foreach($Case in @(
  @('present','clear-throws',0,1,'owned-context-set-failure'),
  @('present','restore-throws',1,1,'owned-restoration-failure'),
  @('present','restore-corruption',2,1,'Work Order 22 authority proof did not restore RUSTFLAGS'),
  @('present','construction-exit',1,1,'Work Order 22 private capability/request construction proof must produce exactly two E0451 failures'),
  @('present','construction-count',1,1,'Work Order 22 private capability/request construction proof must produce exactly two E0451 failures'),
  @('present','construction-fields',1,1,'Work Order 22 construction proof lost: fields `projection` and `_artifact` of struct `VerifiedBackendInput` are private'),
  @('present','construction-unrelated',1,1,'Work Order 22 construction proof failed for an unrelated cfg, import, symbol, or syntax reason'),
  @('present','authority-cause',2,1,'Work Order 22 source-bound authority proof lost: borrowed data escapes outside of closure'),
  @('present','authority-count',2,1,'Work Order 22 substitution/lifetime compiler evidence count drifted'),
  @('present','authority-lifetime',2,1,'Work Order 22 substitution/lifetime compiler evidence count drifted'),
  @('present','authority-unrelated',2,1,'Work Order 22 authority proof failed for an unrelated cfg, import, symbol, or syntax reason'),
  @('present','raw-count',3,2,'Work Order 22 Unit B raw inputs must produce exactly three type mismatches'),
  @('present','raw-type',3,2,'Work Order 22 Unit B verified-only proof lost: found reference `&Program`'),
  @('present','fault-cause',4,2,'Work Order 22 Unit B test fault seam must be absent in production'),
  @('present','fault-count',4,2,'Work Order 22 Unit B test fault seam must be absent in production'),
  @('present','array-output',1,1,'Work Order 22 construction proof lost: fields `projection` and `_artifact` of struct `VerifiedBackendInput` are private'),
  @('present','wrong-flag',1,1,'owned-route-refusal'),@('present','extra-argv',1,1,'owned-route-refusal'),@('present','wrong-target',1,1,'owned-route-refusal')
)){$Cases.Add($Case)}
foreach($Case in $Cases){
  $PriorKind=$Case[0];$script:W22ProofMode=$Case[1]
  $script:W22ProofContext=[Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
  $script:W22ProofHadPrior=$PriorKind -cne 'absent';$script:W22ProofPrior=if($PriorKind -ceq 'empty'){''}elseif($PriorKind -ceq 'present'){'--cfg inherited_parent --codegen debuginfo=1'}else{$null}
  if($script:W22ProofHadPrior){$script:W22ProofContext['RUSTFLAGS']=$script:W22ProofPrior}
  $script:W22ProofRequests=[Collections.Generic.List[object]]::new();$script:W22ProofHonest=[Collections.Generic.List[object]]::new();$Caught=''
  try{
    if($script:W22ProofMode -ceq 'wrong-flag'){Invoke-Wo22ProofOwnedWrongFlag}
    elseif($script:W22ProofMode -ceq 'extra-argv'){Invoke-Wo22ProofOwnedExtraArgv}
    elseif($script:W22ProofMode -ceq 'wrong-target'){Invoke-Wo22ProofOwnedWrongTarget}
    else{Invoke-Wo22ProofOwnedActual}
  }catch{$Caught=$_.Exception.Message}
  $Expected=$Case[4]
  Assert-Wo22ProofOwned ($Caught -ceq $Expected) ($PriorKind+'/'+$script:W22ProofMode+' exact public outcome: '+$Caught)
  if($script:W22ProofMode -cnotin @('restore-corruption','restore-throws')){
    Assert-Wo22ProofOwned ([string]::Equals($script:W22ProofContext['RUSTFLAGS'],$script:W22ProofPrior,[StringComparison]::Ordinal) -and $script:W22ProofContext.ContainsKey('RUSTFLAGS') -eq $script:W22ProofHadPrior) ($PriorKind+'/'+$script:W22ProofMode+' exact finally restoration')
  }
  Assert-Wo22ProofOwned ($script:W22ProofHonest.Count -eq $Case[3]) ($script:W22ProofMode+' exact honest prefix')
  Assert-Wo22ProofOwned ($script:W22ProofRequests.Count -eq $Case[2]) ($script:W22ProofMode+' exact proof request prefix')
  $script:W22ProofCases.Add([pscustomobject]@{Prior=$PriorKind;Mode=$script:W22ProofMode;Primary=$Caught;Requests=$script:W22ProofRequests.ToArray();HonestChecks=$script:W22ProofHonest.Count;OutcomeOwner=if($Expected.StartsWith('owned-',[StringComparison]::Ordinal)){'owned boundary sink'}elseif($Expected -ceq ''){'actual complete stanza'}else{'actual production refusal'}})
}
foreach($Name in @('public','context-read','context-clear','context-restore','context-remove','native-request','capture-output','returned-fact','honest-check')){Assert-Wo22ProofOwned ($script:W22ProofHooks.ContainsKey($Name) -and $script:W22ProofHooks[$Name] -gt 0) ($Name+' nonzero actual boundary')}
[pscustomobject]@{Assertions=$script:W22ProofAssertions.ToArray();Cases=$script:W22ProofCases.ToArray();Hooks=$script:W22ProofHooks}
'@
  $FunctionText="function Invoke-Wo22ProofOwnedActual { `$Cargo='owned-cargo'`n"+$Projected+"`n}`n"
  $WrongFlag=$FunctionText.Replace('Invoke-Wo22ProofOwnedActual','Invoke-Wo22ProofOwnedWrongFlag').Replace('hum_compile_fail_verified_backend_input_construction','unowned_cfg')
  $ExtraArgv=$FunctionText.Replace('Invoke-Wo22ProofOwnedActual','Invoke-Wo22ProofOwnedExtraArgv').Replace("'--cfg', 'hum_compile_fail_verified_backend_input_construction'","'--cfg', 'hum_compile_fail_verified_backend_input_construction', '--release'")
  $WrongTarget=$FunctionText.Replace('Invoke-Wo22ProofOwnedActual','Invoke-Wo22ProofOwnedWrongTarget').Replace("'--bin', 'hum'","'--bin', 'other'")
  $Program=$Owned+"`n"+$ActualReader+"`n"+$FunctionText+$WrongFlag+$ExtraArgv+$WrongTarget+$Driver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  $GateTokens=$null;$GateErrors=$null;$Gate=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$GateTokens,[ref]$GateErrors)
  $Types=@('string','string[]','object','bool','pscustomobject','StringComparison','StringComparer','System.StringComparison','regex','Collections.Generic.List[object]','Collections.Generic.Dictionary[string,int]','Collections.Generic.Dictionary[string,string]')
  $Methods=@('Add','ContainsKey','Remove','Contains','Equals','Join','Replace','StartsWith','ToArray','Matches','IsMatch','new')
  $Properties=@('Count','Output','ExitCode','Exception','Message')
  $Commands=@($Gate.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
  $Unknown=[Collections.Generic.List[string]]::new()
  if($GateErrors.Count -ne 0){$Unknown.Add('parse')}
  foreach($N in $Gate.FindAll({param($N)$true},$true)){
    if(($N -is [Management.Automation.Language.TypeExpressionAst] -or $N -is [Management.Automation.Language.TypeConstraintAst]) -and $N.TypeName.FullName -cnotin $Types){$Unknown.Add('type:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.AttributeAst] -and $N.TypeName.FullName -cne 'AllowEmptyString'){$Unknown.Add('attribute:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.CommandAst] -and $N.GetCommandName() -cnotin $Commands){$Unknown.Add('command:'+$N.Extent.Text)}
    if($N -is [Management.Automation.Language.InvokeMemberExpressionAst]){if($N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $N.Member.Value -cnotin $Methods){$Unknown.Add('method:'+$N.Member.Extent.Text)}}
    elseif($N -is [Management.Automation.Language.MemberExpressionAst]){if($N.Static){if($N.Member.Value -cne 'Ordinal'){$Unknown.Add('static:'+ $N.Member.Extent.Text)}}elseif($N.Member.Value -cnotin $Properties){$Unknown.Add('property:'+$N.Member.Extent.Text)}}
    if($N -is [Management.Automation.Language.VariableExpressionAst] -and $N.VariablePath.IsDriveQualified -and -not $N.VariablePath.UserPath.StartsWith('script:',[StringComparison]::Ordinal) -and $N.VariablePath.UserPath -cne 'global:LASTEXITCODE'){$Unknown.Add('drive:'+$N.VariablePath.UserPath)}
    if($N -is [Management.Automation.Language.FileRedirectionAst]){$Unknown.Add('redirection')}
  }
  if($Unknown.Count -ne 0){throw ('WO22 fatal selected-program ZERO gate refused: '+[string]::Join('|',$Unknown))}
  $Result=& ([scriptblock]::Create($Program))
  return [pscustomobject]@{Assertions=$Result.Assertions;Cases=$Result.Cases;Hooks=$Result.Hooks;Pins=$Pins.ToArray();Gate=[pscustomobject]@{ParseErrors=0;Unknown=0;RealAccessors=0;CompiledPrograms=1}}
}

function Invoke-OwnedRemainingAuthorityProofControls {
  param([string]$CheckAllText, $SelectedProgram = $null)
  # Select only the four changed source stanzas; metadata parsing/hashing is pure.
  $Tokens=$null; $Errors=$null
  $Ast=[Management.Automation.Language.Parser]::ParseInput($CheckAllText,[ref]$Tokens,[ref]$Errors)
  if($Errors.Count -ne 0){throw 'remaining authority source parse refused'}
  $Functions=@($Ast.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true))
  $Privacy=@($Functions|Where-Object{$_.Name -ceq 'Invoke-HumCompilerPrivacyChecks'})
  $Reader=@($Functions|Where-Object{$_.Name -ceq 'Read-NativeOutputWithExit'})
  if($Privacy.Count -ne 1 -or $Reader.Count -ne 1){throw 'remaining authority source owner cardinality refused'}
  $Statements=@($Privacy[0].Body.EndBlock.Statements)
  $Actual='';$Pins=[Collections.Generic.List[object]]::new()
  foreach($Range in @(
    @("Write-Host '==> Replacement F4 compiler-enforced validated body construction proof'","Invoke-Native 'Work Order 19 normal compiler check after combined proof' `$Cargo @('check', '--all-targets')"),
    @("Write-Host '==> Work Order 17 sole minimal-add outcome producer proof'","Invoke-Native 'Work Order 17 normal compiler check after sole-producer proof' `$Cargo @('check', '--all-targets')")
  )){
    $First=@($Statements|Where-Object{$_.Extent.Text -ceq $Range[0]});$Last=@($Statements|Where-Object{$_.Extent.Text -ceq $Range[1]})
    if($First.Count -ne 1 -or $Last.Count -ne 1 -or $First[0].Extent.StartOffset -ge $Last[0].Extent.EndOffset){throw 'remaining authority statement cardinality refused'}
    $Start=$First[0].Extent.StartOffset;$End=$Last[0].Extent.EndOffset
    $Part=$CheckAllText.Substring($Start,$End-$Start);$Actual+=$Part+"`n"
    $Hash=[Security.Cryptography.SHA256]::Create();try{$Digest=[BitConverter]::ToString($Hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Part))).Replace('-','').ToLowerInvariant()}finally{$Hash.Dispose()}
    $Pins.Add([pscustomobject]@{Name=$Range[0];StartUTF16=$Start;EndUTF16=$End;UTF8TextSha256=$Digest})
  }
  $Hash=[Security.Cryptography.SHA256]::Create();try{$Digest=[BitConverter]::ToString($Hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Reader[0].Extent.Text))).Replace('-','').ToLowerInvariant()}finally{$Hash.Dispose()}
  $Pins.Add([pscustomobject]@{Name='Read-NativeOutputWithExit';StartUTF16=$Reader[0].Extent.StartOffset;EndUTF16=$Reader[0].Extent.EndOffset;UTF8TextSha256=$Digest})
  $Projected=$Actual.Replace("`$env:RUSTFLAGS = ''","Set-RemainingOwnedFlags ''")
  foreach($Name in @('F4PriorRustFlags','Wo17PriorRustFlags','Wo19PriorRustFlags','Wo17OutcomePriorRustFlags')){$Projected=$Projected.Replace('$env:RUSTFLAGS = $'+$Name,'Restore-RemainingOwnedFlags $'+$Name)}
  $Projected=$Projected.Replace('Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue','Remove-RemainingOwnedFlags').Replace('$env:RUSTFLAGS','(Get-RemainingOwnedFlags)')
  $ActualReader=$Reader[0].Extent.Text.Replace('$Output = & $FilePath @Arguments 2>&1','$Output = Invoke-RemainingOwnedCapture $FilePath $Arguments').Replace('return [pscustomobject] @{','$Fact = [pscustomobject] @{')
  $ActualReader=$ActualReader.Substring(0,$ActualReader.Length-1)+"  return Complete-RemainingOwnedFact `$Fact`n}"
  $Owned=@'
$script:R4Assertions=[Collections.Generic.List[object]]::new()
$script:R4Cases=[Collections.Generic.List[object]]::new()
$script:R4Hooks=[Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
function Hit-RemainingOwned([string]$Name){if(-not $script:R4Hooks.ContainsKey($Name)){$script:R4Hooks.Add($Name,0)};$script:R4Hooks[$Name]++}
function Assert-RemainingOwned([bool]$Ok,[string]$Message){$script:R4Assertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message});if(-not $Ok){throw ('remaining proof owned assertion: '+$Message)}}
function Write-Host {param([string]$Message) Hit-RemainingOwned 'public'}
function Get-RemainingOwnedFlags {Hit-RemainingOwned 'context-read';return $script:R4Context['RUSTFLAGS']}
function Set-RemainingOwnedFlags([AllowEmptyString()][string]$Value){
  Hit-RemainingOwned 'context-clear'
  if($script:R4Mode -ceq 'clear-throws'){throw 'owned-context-set-failure'}
  Assert-RemainingOwned ($Value -ceq '') 'proof-local flags overwrite empty'
  $script:R4Context['RUSTFLAGS']=$Value
}
function Restore-RemainingOwnedFlags([AllowEmptyString()][string]$Value){
  Hit-RemainingOwned 'context-restore'
  if($script:R4Mode -ceq 'restore-throws'){throw 'owned-restoration-failure'}
  $script:R4Context['RUSTFLAGS']=if($script:R4Mode -ceq 'restore-corruption'){'corrupted-restoration'}else{$Value}
}
function Remove-RemainingOwnedFlags {Hit-RemainingOwned 'context-remove';$null=$script:R4Context.Remove('RUSTFLAGS')}
function Invoke-RemainingOwnedCapture([string]$FilePath,[string[]]$Arguments){
  Hit-RemainingOwned 'native-request';$script:R4Requests.Add([pscustomobject]@{File=$FilePath;Arguments=$Arguments;Flags=$script:R4Context['RUSTFLAGS']})
  $Index=$script:R4Requests.Count-1
  $Flags=@('hum_compile_fail_validated_body_grammar_construction','hum_compile_fail_verified_canonical_minimal_add_type_escape','hum_compile_fail_canonical_minimal_add_backend_facts_escape','hum_compile_fail_canonical_minimal_add_type_outcome_foreign_issue')
  $Expected='rustc|--profile|check|--bin|hum|--|--cfg|'+$Flags[$Index]
  if($FilePath -cne 'owned-cargo' -or [string]::Join('|',$Arguments) -cne $Expected -or $script:R4Context['RUSTFLAGS'] -cne ''){throw 'owned-route-refusal'}
  if($script:R4Mode -ceq ('producer-'+[string]$Index) -or $script:R4Mode -ceq 'restore-throws'){throw 'owned-producer-failure'}
  Hit-RemainingOwned 'capture-output';$global:LASTEXITCODE=101
  $Text=switch($Index){
    0 {'prefix body_grammar_report_foreign_literal_must_not_compile canonical_body_grammar_report_foreign_literal_must_not_compile canonical_body_statement_foreign_literal_must_not_compile validated_body_grammar_permit_from_raw_section_must_not_compile'+"`n"+'cannot construct `BodyGrammarReport` with struct literal syntax due to private fields'+"`n"+'cannot construct `CanonicalBodyGrammarReport` with struct literal syntax due to private fields'+"`n"+'cannot construct `CanonicalBodyStatement` with struct literal syntax due to private fields'+"`n"+'error[E0308] expected `ValidatedCoreSection<''_>` found `&Section` trailing'}
    1 {'prefix verified_canonical_minimal_add_access_cannot_outlive_verify_artifact verified_canonical_minimal_add_result_cannot_be_collected verified_canonical_minimal_add_result_cannot_become_static core_verify_full_type_report_access_cannot_escape core_verify_diagnostic_occurrence_access_cannot_escape'+"`n"+'lifetime may not live long enough trailing'}
    2 {'prefix no `CanonicalMinimalAddBackendFactsAccess` in `backend_input`'+"`n"+'cannot find function `with_canonical_minimal_add_backend_facts`'+"`n"+'cannot find function `issue_assembled`'+"`n"+'error[E0603]: tuple struct constructor `VerifiedMinimalAddFullType` is private'+"`n"+'error[E0603]: tuple struct constructor `VerifiedMinimalAddEffect` is private'+"`n"+'error[E0603]: tuple struct constructor `VerifiedMinimalAddOwnership` is private'+"`n"+'error[E0603]: tuple struct constructor `VerifiedMinimalAddResource` is private'+"`n"+'error[E0603]: tuple struct constructor `VerifiedMinimalAddProfile` is private trailing'}
    3 {'prefix canonical_minimal_add_type_outcome_foreign_issue_must_not_compile'+"`n"+'error[E0624] associated function `non_target` is private trailing'}
  }
  return $Text -split "`n"
}
function Complete-RemainingOwnedFact([object]$Fact){
  Hit-RemainingOwned 'returned-fact';$Index=$script:R4Requests.Count-1
  Assert-RemainingOwned ($Fact.Output -is [string] -and $Fact.Output.Contains("`n")) 'actual reader emits scalar joined diagnostic'
  if($script:R4Corruptions.ContainsKey($script:R4Mode)){
    $Mutation=$script:R4Corruptions[$script:R4Mode]
    if($Index -eq $Mutation[0]){
      if($Mutation[1] -ceq 'exit'){$Fact.ExitCode=0}
      elseif($Mutation[1] -ceq 'append'){$Fact.Output+=$Mutation[2]}
      elseif($Mutation[1] -ceq 'array'){$Fact.Output=@($Fact.Output)}
      else{$Fact.Output=$Fact.Output.Replace($Mutation[1],$Mutation[2])}
    }
  }
  return $Fact
}
function Invoke-Native([string]$Label,[string]$FilePath,[string[]]$Arguments){
  Hit-RemainingOwned 'honest-check'
  $Labels=@('Replacement F4 normal compiler check before construction proof','Replacement F4 normal compiler check after construction proof','Work Order 17 normal compiler check before lifetime proof','Work Order 17 normal compiler check after lifetime proof','Work Order 19 normal compiler check before combined proof','Work Order 19 normal compiler check after combined proof','Work Order 17 normal compiler check after sole-producer proof')
  Assert-RemainingOwned ($Label -ceq $Labels[$script:R4Honest.Count] -and $FilePath -ceq 'owned-cargo' -and [string]::Join('|',$Arguments) -ceq 'check|--all-targets') 'unchanged seven honest all-target argv and order'
  Assert-RemainingOwned ([string]::Equals($script:R4Context['RUSTFLAGS'],$script:R4Prior,[StringComparison]::Ordinal) -and $script:R4Context.ContainsKey('RUSTFLAGS') -eq $script:R4HadPrior) 'honest check sees exact prior context'
  $script:R4Honest.Add($Label)
}
'@
  $Driver=@'
$Cases=[Collections.Generic.List[object]]::new()
$HonestPrefixes=@(1,3,5,6)
foreach($Kind in @('absent','empty','present')){
  $Cases.Add(@($Kind,'positive',4,7,''))
  foreach($Index in @(0,1,2,3)){$Cases.Add(@($Kind,('producer-'+$Index),($Index+1),$HonestPrefixes[$Index],'owned-producer-failure'))}
}
$script:R4Corruptions=[Collections.Generic.Dictionary[string,object]]::new([StringComparer]::Ordinal)
foreach($Mutation in @(
  @('f4-exit',0,'exit','','Replacement F4 forbidden construction compiler proof must exit 101, found 0'),
  @('f4-probe',0,'canonical_body_statement_foreign_literal_must_not_compile','missing','Replacement F4 forbidden construction compiler proof did not identify canonical_body_statement_foreign_literal_must_not_compile'),
  @('f4-private',0,'cannot construct `BodyGrammarReport` with struct literal syntax due to private fields','missing','Replacement F4 compiler proof did not reject foreign BodyGrammarReport construction through Rust privacy'),
  @('f4-substitution',0,'found `&Section`','missing','Replacement F4 compiler proof did not reject raw Section substitution for the validated capability'),
  @('f4-unrelated',0,'append',' error[E0432] unresolved import','Replacement F4 forbidden construction proof failed for an unrelated symbol, cfg, import, or syntax reason'),
  @('lifetime-exit',1,'exit','','Work Order 17 lifetime proof must exit 101, found 0'),
  @('lifetime-probe',1,'core_verify_diagnostic_occurrence_access_cannot_escape','missing','Work Order 17 lifetime proof did not identify core_verify_diagnostic_occurrence_access_cannot_escape'),
  @('lifetime-cause',1,'lifetime may not live long enough','missing','Work Order 17 lifetime proof did not fail for a borrow-check lifetime reason'),
  @('lifetime-unrelated',1,'append',' error[E0603] private struct','Work Order 17 lifetime proof failed for an unrelated privacy, symbol, cfg, or import reason'),
  @('wo19-exit',2,'exit','','Work Order 19 combined proof must exit 101, found 0'),
  @('wo19-retired',2,'cannot find function `issue_assembled`','missing','Work Order 19 proof did not establish removal of every retired backend-facts authority route'),
  @('wo19-wrapper',2,'tuple struct constructor `VerifiedMinimalAddProfile` is private','missing','Work Order 19 combined proof did not reject sibling construction of VerifiedMinimalAddProfile'),
  @('wo19-count',2,'append',' error[E0603]: tuple struct constructor `VerifiedMinimalAddEffect` is private','Work Order 19 combined proof must contain exactly five wrapper-constructor privacy failures'),
  @('wo19-unrelated',2,'append',' error[E0382]','Work Order 19 combined proof failed for an unrelated move, symbol, cfg, import, or syntax reason'),
  @('outcome-exit',3,'exit','','Work Order 17 foreign outcome issuance proof must exit 101, found 0'),
  @('outcome-cause',3,'associated function `non_target` is private','missing','Work Order 17 foreign outcome issuance proof did not fail at the private sole-producer constructor'),
  @('outcome-unrelated',3,'append',' error[E0433]','Work Order 17 foreign outcome issuance proof failed for an unrelated symbol, cfg, import, privacy-type, or syntax reason'),
  @('array-output',0,'array','','Replacement F4 forbidden construction compiler proof did not identify body_grammar_report_foreign_literal_must_not_compile')
)){
  $script:R4Corruptions.Add($Mutation[0],@($Mutation[1],$Mutation[2],$Mutation[3]))
  $Cases.Add(@('present',$Mutation[0],($Mutation[1]+1),$HonestPrefixes[$Mutation[1]],$Mutation[4]))
}
foreach($Case in @(
  @('present','clear-throws',0,1,'owned-context-set-failure'),
  @('present','restore-throws',1,1,'owned-restoration-failure'),
  @('present','restore-corruption',1,1,'Replacement F4 compiler proof did not restore RUSTFLAGS'),
  @('present','wrong-flag',1,1,'owned-route-refusal'),
  @('present','extra-argv',1,1,'owned-route-refusal'),
  @('present','wrong-target',1,1,'owned-route-refusal')
)){$Cases.Add($Case)}
foreach($Case in $Cases){
  $PriorKind=$Case[0];$script:R4Mode=$Case[1]
  $script:R4Context=[Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
  $script:R4HadPrior=$PriorKind -cne 'absent';$script:R4Prior=if($PriorKind -ceq 'empty'){''}elseif($PriorKind -ceq 'present'){'--cfg inherited_parent --codegen debuginfo=1'}else{$null}
  if($script:R4HadPrior){$script:R4Context['RUSTFLAGS']=$script:R4Prior}
  $script:R4Requests=[Collections.Generic.List[object]]::new();$script:R4Honest=[Collections.Generic.List[object]]::new();$Caught=''
  try{
    if($script:R4Mode -ceq 'wrong-flag'){Invoke-RemainingOwnedWrongFlag}
    elseif($script:R4Mode -ceq 'extra-argv'){Invoke-RemainingOwnedExtraArgv}
    elseif($script:R4Mode -ceq 'wrong-target'){Invoke-RemainingOwnedWrongTarget}
    else{Invoke-RemainingOwnedActual}
  }catch{$Caught=$_.Exception.Message}
  $Expected=$Case[4]
  Assert-RemainingOwned ($Caught -ceq $Expected) ($PriorKind+'/'+$script:R4Mode+' exact public outcome: '+$Caught)
  if($script:R4Mode -cnotin @('restore-corruption','restore-throws')){
    Assert-RemainingOwned ([string]::Equals($script:R4Context['RUSTFLAGS'],$script:R4Prior,[StringComparison]::Ordinal) -and $script:R4Context.ContainsKey('RUSTFLAGS') -eq $script:R4HadPrior) ($PriorKind+'/'+$script:R4Mode+' exact finally restoration')
  }
  Assert-RemainingOwned ($script:R4Honest.Count -eq $Case[3]) ($script:R4Mode+' exact honest prefix')
  Assert-RemainingOwned ($script:R4Requests.Count -eq $Case[2]) ($script:R4Mode+' exact proof request prefix')
  $script:R4Cases.Add([pscustomobject]@{Prior=$PriorKind;Mode=$script:R4Mode;Primary=$Caught;Requests=$script:R4Requests.ToArray();HonestChecks=$script:R4Honest.Count;OutcomeOwner=if($Expected.StartsWith('owned-',[StringComparison]::Ordinal)){'owned boundary sink'}elseif($Expected -ceq ''){'actual complete selected stanzas'}else{'actual production refusal'}})
}
foreach($Name in @('public','context-read','context-clear','context-restore','context-remove','native-request','capture-output','returned-fact','honest-check')){Assert-RemainingOwned ($script:R4Hooks.ContainsKey($Name) -and $script:R4Hooks[$Name] -gt 0) ($Name+' nonzero actual boundary')}
[pscustomobject]@{Assertions=$script:R4Assertions.ToArray();Cases=$script:R4Cases.ToArray();Hooks=$script:R4Hooks}
'@
  $FunctionText="function Invoke-RemainingOwnedActual { `$Cargo='owned-cargo'`n"+$Projected+"`n}`n"
  $WrongFlag=$FunctionText.Replace('Invoke-RemainingOwnedActual','Invoke-RemainingOwnedWrongFlag').Replace('hum_compile_fail_validated_body_grammar_construction','unowned_cfg')
  $ExtraArgv=$FunctionText.Replace('Invoke-RemainingOwnedActual','Invoke-RemainingOwnedExtraArgv').Replace("'--cfg', 'hum_compile_fail_validated_body_grammar_construction'","'--cfg', 'hum_compile_fail_validated_body_grammar_construction', '--release'")
  $WrongTarget=$FunctionText.Replace('Invoke-RemainingOwnedActual','Invoke-RemainingOwnedWrongTarget').Replace("'--bin', 'hum'","'--bin', 'other'")
  $Program=$Owned+"`n"+$ActualReader+"`n"+$FunctionText+$WrongFlag+$ExtraArgv+$WrongTarget+$Driver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  $GateTokens=$null;$GateErrors=$null;$Gate=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$GateTokens,[ref]$GateErrors)
  $Types=@('string','string[]','object','bool','pscustomobject','StringComparison','StringComparer','System.StringComparison','regex','Collections.Generic.List[object]','Collections.Generic.Dictionary[string,int]','Collections.Generic.Dictionary[string,string]','Collections.Generic.Dictionary[string,object]')
  $Methods=@('Add','ContainsKey','Remove','Contains','Equals','Join','Replace','StartsWith','ToArray','Matches','IsMatch','new')
  $Properties=@('Count','Output','ExitCode','Exception','Message')
  $Commands=@($Gate.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
  $Unknown=[Collections.Generic.List[string]]::new()
  if($GateErrors.Count -ne 0){$Unknown.Add('parse')}
  foreach($N in $Gate.FindAll({param($N)$true},$true)){
    if(($N -is [Management.Automation.Language.TypeExpressionAst] -or $N -is [Management.Automation.Language.TypeConstraintAst]) -and $N.TypeName.FullName -cnotin $Types){$Unknown.Add('type:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.AttributeAst] -and $N.TypeName.FullName -cne 'AllowEmptyString'){$Unknown.Add('attribute:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.CommandAst] -and $N.GetCommandName() -cnotin $Commands){$Unknown.Add('command:'+$N.Extent.Text)}
    if($N -is [Management.Automation.Language.InvokeMemberExpressionAst]){if($N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $N.Member.Value -cnotin $Methods){$Unknown.Add('method:'+$N.Member.Extent.Text)}}
    elseif($N -is [Management.Automation.Language.MemberExpressionAst]){if($N.Static){if($N.Member.Value -cne 'Ordinal'){$Unknown.Add('static:'+ $N.Member.Extent.Text)}}elseif($N.Member.Value -cnotin $Properties){$Unknown.Add('property:'+$N.Member.Extent.Text)}}
    if($N -is [Management.Automation.Language.VariableExpressionAst] -and $N.VariablePath.IsDriveQualified -and -not $N.VariablePath.UserPath.StartsWith('script:',[StringComparison]::Ordinal) -and $N.VariablePath.UserPath -cne 'global:LASTEXITCODE'){$Unknown.Add('drive:'+$N.VariablePath.UserPath)}
    if($N -is [Management.Automation.Language.FileRedirectionAst]){$Unknown.Add('redirection')}
  }
  if($Unknown.Count -ne 0){throw ('remaining authority fatal selected-program ZERO gate refused: '+[string]::Join('|',$Unknown))}
  $Result=& ([scriptblock]::Create($Program))
  return [pscustomobject]@{Assertions=$Result.Assertions;Cases=$Result.Cases;Hooks=$Result.Hooks;Pins=$Pins.ToArray();Gate=[pscustomobject]@{ParseErrors=0;Unknown=0;RealAccessors=0;CompiledPrograms=1}}
}

function Invoke-OwnedWo24AuthorityProofControls {
  param([string]$CheckAllText, $SelectedProgram = $null)
  # Metadata is source text; only the closed, accessor-free projection executes.
  $Tokens=$null; $Errors=$null
  $Ast=[Management.Automation.Language.Parser]::ParseInput($CheckAllText,[ref]$Tokens,[ref]$Errors)
  if($Errors.Count -ne 0){throw 'WO24 authority source parse refused'}
  $Functions=@($Ast.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true))
  $Privacy=@($Functions|Where-Object{$_.Name -ceq 'Invoke-HumCompilerPrivacyChecks'})
  $Reader=@($Functions|Where-Object{$_.Name -ceq 'Read-NativeOutputWithExit'})
  if($Privacy.Count -ne 1 -or $Reader.Count -ne 1){throw 'WO24 authority source owner cardinality refused'}
  $Statements=@($Privacy[0].Body.EndBlock.Statements)
  $First=@($Statements|Where-Object{$_.Extent.Text -ceq "Write-Host '==> Work Order 24 VerifiedConstantTextBackendInput compiler-owned authority proof'"})
  $Last=@($Statements|Where-Object{$_.Extent.Text -ceq "Invoke-Native 'Work Order 24 normal compiler check after authority proof' `$Cargo @('check', '--all-targets')"})
  if($First.Count -ne 1 -or $Last.Count -ne 1){throw 'WO24 authority statement cardinality refused'}
  $Start=$First[0].Extent.StartOffset; $End=$Last[0].Extent.EndOffset
  $Actual=$CheckAllText.Substring($Start,$End-$Start)
  $Pins=[Collections.Generic.List[object]]::new()
  foreach($Part in @([pscustomobject]@{Name='WO24 authority stanza';Start=$Start;End=$End;Text=$Actual},[pscustomobject]@{Name='Read-NativeOutputWithExit';Start=$Reader[0].Extent.StartOffset;End=$Reader[0].Extent.EndOffset;Text=$Reader[0].Extent.Text})){
    $Hash=[Security.Cryptography.SHA256]::Create();try{$Digest=[BitConverter]::ToString($Hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Part.Text))).Replace('-','').ToLowerInvariant()}finally{$Hash.Dispose()}
    $Pins.Add([pscustomobject]@{Name=$Part.Name;StartUTF16=$Part.Start;EndUTF16=$Part.End;UTF8TextSha256=$Digest})
  }
  $Projected=$Actual.Replace("`$env:RUSTFLAGS = ''","Set-Wo24OwnedFlags ''").Replace('$env:RUSTFLAGS = $Wo24PriorRustFlags','Set-Wo24OwnedFlags $Wo24PriorRustFlags').Replace('Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue','Remove-Wo24OwnedFlags').Replace('$env:RUSTFLAGS','(Get-Wo24OwnedFlags)')
  $ActualReader=$Reader[0].Extent.Text.Replace('$Output = & $FilePath @Arguments 2>&1','$Output = Invoke-Wo24OwnedCapture $FilePath $Arguments').Replace('return [pscustomobject] @{','$Fact = [pscustomobject] @{')
  $ActualReader=$ActualReader.Substring(0,$ActualReader.Length-1)+"  return Complete-Wo24OwnedFact `$Fact`n}"
  $Owned=@'
$script:W24Assertions=[Collections.Generic.List[object]]::new()
$script:W24Cases=[Collections.Generic.List[object]]::new()
$script:W24Hooks=[Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
function Hit-Wo24Owned([string]$Name){if(-not $script:W24Hooks.ContainsKey($Name)){$script:W24Hooks.Add($Name,0)};$script:W24Hooks[$Name]++}
function Assert-Wo24Owned([bool]$Ok,[string]$Message){$script:W24Assertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message});if(-not $Ok){throw ('WO24 owned assertion: '+$Message)}}
function Write-Host {param([string]$Message) Hit-Wo24Owned 'public'}
function Get-Wo24OwnedFlags {Hit-Wo24Owned 'context-read';return $script:W24Context['RUSTFLAGS']}
function Set-Wo24OwnedFlags([AllowEmptyString()][string]$Value){
  Hit-Wo24Owned 'context-write';$script:W24Writes++
  if($script:W24Mode -ceq 'clear-throws' -and $script:W24Writes -eq 1){throw 'owned-context-set-failure'}
  if($script:W24Writes -gt 1 -and $script:W24Mode -ceq 'restore-throws'){throw 'owned-restoration-failure'}
  $script:W24Context['RUSTFLAGS']=if($script:W24Writes -gt 1 -and $script:W24Mode -ceq 'restore-corruption'){'corrupted-restoration'}else{$Value}
}
function Remove-Wo24OwnedFlags {
  Hit-Wo24Owned 'context-remove'
  if($script:W24Mode -ceq 'restore-corruption'){$script:W24Context['RUSTFLAGS']='corrupted-restoration'}else{$null=$script:W24Context.Remove('RUSTFLAGS')}
}
function Invoke-Wo24OwnedCapture([string]$FilePath,[string[]]$Arguments){
  Hit-Wo24Owned 'native-request';$script:W24Requests.Add([pscustomobject]@{File=$FilePath;Arguments=$Arguments;Flags=$script:W24Context['RUSTFLAGS']})
  $Index=$script:W24Requests.Count-1
  $Roles=@('construction','lifetime','substitution')
  $Flags=@('hum_compile_fail_verified_constant_text_backend_input_construction','hum_compile_fail_verified_constant_text_backend_input_lifetime','hum_compile_fail_verified_backend_input_cross_substitution')
  $Expected='rustc|--profile|check|--bin|hum|--|--cfg|'+$Flags[$Index]
  if($FilePath -cne 'owned-cargo' -or [string]::Join('|',$Arguments) -cne $Expected -or $script:W24Context['RUSTFLAGS'] -cne ''){throw 'owned-route-refusal'}
  if($script:W24Mode -ceq ('producer-'+[string]$Index) -or ($script:W24Mode -ceq 'restore-throws' -and $Index -eq 0)){throw 'owned-producer-failure'}
  Hit-Wo24Owned 'capture-output';$global:LASTEXITCODE=101
  $Text=switch($Index){
    0 {'prefix error[E0451]: fields `projection` and `_artifact` of struct `VerifiedConstantTextBackendInput` are private; type `VerifiedConstantTextProjection` is private'+"`n"+' projection: None.unwrap() trailing'}
    1 {'lifetime may not live long enough'+"`n"+'|capability| capability'+"`n"+' return type of closure is VerifiedConstantTextBackendInput'}
    2 {'error[E0308] first'+"`n"+'expected reference `&VerifiedConstantTextBackendInput'+"`n"+'found reference `&VerifiedIntegerSignBackendInput'+"`n"+'error[E0308] second'+"`n"+'expected reference `&VerifiedIntegerSignBackendInput'+"`n"+'found reference `&VerifiedConstantTextBackendInput'+"`n"+'execute_constant_text execute_integer_sign execute_constant_text execute_integer_sign'}
  }
  return $Text -split "`n"
}
function Complete-Wo24OwnedFact([object]$Fact){
  Hit-Wo24Owned 'returned-fact';$Index=$script:W24Requests.Count-1
  Assert-Wo24Owned ($Fact.Output -is [string] -and $Fact.Output.Contains("`n")) 'actual reader emits scalar joined diagnostic'
  if($script:W24Mode -ceq 'exit-corruption' -and $Index -eq 0){$Fact.ExitCode=0}
  if($script:W24Mode -ceq 'construction-cause' -and $Index -eq 0){$Fact.Output=$Fact.Output.Replace('projection: None.unwrap()','wrong span')}
  if($script:W24Mode -ceq 'construction-type' -and $Index -eq 0){$Fact.Output=$Fact.Output.Replace('type `VerifiedConstantTextProjection` is private','missing private type')}
  if($script:W24Mode -ceq 'construction-fields' -and $Index -eq 0){$Fact.Output=$Fact.Output.Replace('fields `projection` and `_artifact` of struct `VerifiedConstantTextBackendInput` are private','missing private fields')}
  if($script:W24Mode -ceq 'construction-code' -and $Index -eq 0){$Fact.Output=$Fact.Output.Replace('error[E0451]','error[E0308]')}
  if($script:W24Mode -ceq 'lifetime-cause' -and $Index -eq 1){$Fact.Output=$Fact.Output.Replace('lifetime may not live long enough','borrowed data escapes outside of closure error[E0521]')}
  if($script:W24Mode -ceq 'substitution-count' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('error[E0308] second','no second error')}
  if($script:W24Mode -ceq 'substitution-expected-text' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('expected reference `&VerifiedConstantTextBackendInput','wrong expected Text')}
  if($script:W24Mode -ceq 'substitution-found-integer' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('found reference `&VerifiedIntegerSignBackendInput','wrong found Integer')}
  if($script:W24Mode -ceq 'substitution-expected-integer' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('expected reference `&VerifiedIntegerSignBackendInput','wrong expected Integer')}
  if($script:W24Mode -ceq 'substitution-found-text' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('found reference `&VerifiedConstantTextBackendInput','wrong found Text')}
  if($script:W24Mode -ceq 'substitution-span' -and $Index -eq 2){$Fact.Output=$Fact.Output.Replace('execute_constant_text','missing span')}
  if($script:W24Mode -ceq 'unrelated-error' -and $Index -eq 2){$Fact.Output+="`nerror[E0433]: unresolved import"}
  if($script:W24Mode -ceq 'array-output' -and $Index -eq 0){$Fact.Output=@($Fact.Output)}
  return $Fact
}
function Invoke-Native([string]$Label,[string]$FilePath,[string[]]$Arguments){
  Hit-Wo24Owned 'honest-check'
  Assert-Wo24Owned ($Label -ceq 'Work Order 24 normal compiler check after authority proof' -and $FilePath -ceq 'owned-cargo' -and [string]::Join('|',$Arguments) -ceq 'check|--all-targets') 'unchanged honest all-target argv'
  Assert-Wo24Owned ([string]::Equals($script:W24Context['RUSTFLAGS'],$script:W24Prior,[StringComparison]::Ordinal) -and $script:W24Context.ContainsKey('RUSTFLAGS') -eq $script:W24HadPrior) 'honest check sees exact prior context'
}
'@
  $Driver=@'
foreach($Case in @(
  @('absent','positive'),@('empty','positive'),@('present','positive'),
  @('absent','producer-0'),@('absent','producer-1'),@('absent','producer-2'),
  @('empty','producer-0'),@('empty','producer-1'),@('empty','producer-2'),
  @('present','producer-0'),@('present','producer-1'),@('present','producer-2'),
  @('present','clear-throws'),@('present','restore-throws'),
  @('absent','restore-corruption'),@('empty','restore-corruption'),@('present','restore-corruption'),
  @('absent','construction-cause'),@('empty','lifetime-cause'),@('present','substitution-count'),
  @('present','construction-type'),@('present','construction-fields'),@('present','construction-code'),
  @('present','substitution-expected-text'),@('present','substitution-found-integer'),@('present','substitution-expected-integer'),@('present','substitution-found-text'),@('present','substitution-span'),
  @('present','exit-corruption'),@('present','unrelated-error'),@('present','array-output'),
  @('present','wrong-flag'),@('present','extra-argv'),@('present','wrong-target')
)){
  $PriorKind=$Case[0];$script:W24Mode=$Case[1]
  $script:W24Context=[Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
  $script:W24HadPrior=$PriorKind -cne 'absent';$script:W24Prior=if($PriorKind -ceq 'empty'){''}elseif($PriorKind -ceq 'present'){'--cfg inherited_parent --codegen debuginfo=1'}else{$null}
  if($script:W24HadPrior){$script:W24Context['RUSTFLAGS']=$script:W24Prior}
  $script:W24Writes=0;$script:W24Requests=[Collections.Generic.List[object]]::new()
  $Caught='';$BeforeHonest=if($script:W24Hooks.ContainsKey('honest-check')){$script:W24Hooks['honest-check']}else{0}
  try{
    if($script:W24Mode -ceq 'wrong-flag'){Invoke-Wo24OwnedWrongFlag}
    elseif($script:W24Mode -ceq 'extra-argv'){Invoke-Wo24OwnedExtraArgv}
    elseif($script:W24Mode -ceq 'wrong-target'){Invoke-Wo24OwnedWrongTarget}
    else{Invoke-Wo24OwnedActual}
  }catch{$Caught=$_.Exception.Message}
  $Expected=switch -CaseSensitive($script:W24Mode){
    'positive' {''}
    {$_ -cin @('producer-0','producer-1','producer-2')} {'owned-producer-failure'}
    'clear-throws' {'owned-context-set-failure'}
    'restore-throws' {'owned-restoration-failure'}
    'restore-corruption' {'Work Order 24 constant-Text authority proof did not restore RUSTFLAGS'}
    {$_ -cin @('construction-cause','construction-type','construction-fields','construction-code','exit-corruption','array-output')} {'Work Order 24 constant-Text construction proof lost its private authority state or primary span'}
    'lifetime-cause' {'Work Order 24 constant-Text lifetime proof lost its callback-owned escape diagnostic or primary span'}
    {$_ -cin @('substitution-count','substitution-expected-text','substitution-found-integer','substitution-expected-integer','substitution-found-text','substitution-span')} {'Work Order 24 capability substitution proof lost its two E0308 owner diagnostics or primary spans'}
    'unrelated-error' {'Work Order 24 constant-Text authority proof failed for an unrelated cfg, import, symbol, or syntax reason'}
    default {'owned-route-refusal'}
  }
  Assert-Wo24Owned ($Caught -ceq $Expected) ($PriorKind+'/'+$script:W24Mode+' exact public outcome')
  if($script:W24Mode -cnotin @('restore-corruption','restore-throws')){
    Assert-Wo24Owned ([string]::Equals($script:W24Context['RUSTFLAGS'],$script:W24Prior,[StringComparison]::Ordinal) -and $script:W24Context.ContainsKey('RUSTFLAGS') -eq $script:W24HadPrior) ($PriorKind+'/'+$script:W24Mode+' exact finally restoration')
  }
  $AfterHonest=if($script:W24Hooks.ContainsKey('honest-check')){$script:W24Hooks['honest-check']}else{0}
  Assert-Wo24Owned ($AfterHonest-$BeforeHonest -eq $(if($script:W24Mode -ceq 'positive'){1}else{0})) ($script:W24Mode+' honest check not credited after refusal')
  $ExpectedCalls=if($script:W24Mode -ceq 'clear-throws'){0}elseif($script:W24Mode -ceq 'producer-0' -or $script:W24Mode -ceq 'restore-throws' -or $script:W24Mode -cin @('wrong-flag','extra-argv','wrong-target')){1}elseif($script:W24Mode -ceq 'producer-1'){2}else{3}
  Assert-Wo24Owned ($script:W24Requests.Count -eq $ExpectedCalls) ($script:W24Mode+' exact proof request prefix')
  $script:W24Cases.Add([pscustomobject]@{Prior=$PriorKind;Mode=$script:W24Mode;Primary=$Caught;Requests=$script:W24Requests.ToArray();HonestChecks=$AfterHonest-$BeforeHonest;OutcomeOwner=if($Expected.StartsWith('owned-',[StringComparison]::Ordinal)){'owned boundary sink'}elseif($Expected -ceq ''){'actual complete stanza'}else{'actual production refusal'}})
}
foreach($Name in @('public','context-read','context-write','context-remove','native-request','capture-output','returned-fact','honest-check')){Assert-Wo24Owned ($script:W24Hooks.ContainsKey($Name) -and $script:W24Hooks[$Name] -gt 0) ($Name+' nonzero actual boundary')}
[pscustomobject]@{Assertions=$script:W24Assertions.ToArray();Cases=$script:W24Cases.ToArray();Hooks=$script:W24Hooks}
'@
  $FunctionText="function Invoke-Wo24OwnedActual { `$Cargo='owned-cargo'`n"+$Projected+"`n}`n"
  $WrongFlag=$FunctionText.Replace('Invoke-Wo24OwnedActual','Invoke-Wo24OwnedWrongFlag').Replace('hum_compile_fail_verified_constant_text_backend_input_construction','unowned_cfg')
  $ExtraArgv=$FunctionText.Replace('Invoke-Wo24OwnedActual','Invoke-Wo24OwnedExtraArgv').Replace("'--cfg', 'hum_compile_fail_verified_constant_text_backend_input_construction'","'--cfg', 'hum_compile_fail_verified_constant_text_backend_input_construction', '--release'")
  $WrongTarget=$FunctionText.Replace('Invoke-Wo24OwnedActual','Invoke-Wo24OwnedWrongTarget').Replace("'--bin', 'hum'","'--bin', 'other'")
  $Program=$Owned+"`n"+$ActualReader+"`n"+$FunctionText+$WrongFlag+$ExtraArgv+$WrongTarget+$Driver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  $GateTokens=$null;$GateErrors=$null;$Gate=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$GateTokens,[ref]$GateErrors)
  $Types=@('string','string[]','object','bool','pscustomobject','StringComparison','StringComparer','System.StringComparison','regex','Collections.Generic.List[object]','Collections.Generic.Dictionary[string,int]','Collections.Generic.Dictionary[string,string]')
  $Methods=@('Add','ContainsKey','Remove','Contains','Equals','Join','Replace','StartsWith','ToArray','Matches','IsMatch','new')
  $Properties=@('Count','Output','ExitCode','Exception','Message')
  $Commands=@($Gate.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
  $Unknown=[Collections.Generic.List[string]]::new()
  if($GateErrors.Count -ne 0){$Unknown.Add('parse')}
  foreach($N in $Gate.FindAll({param($N)$true},$true)){
    if(($N -is [Management.Automation.Language.TypeExpressionAst] -or $N -is [Management.Automation.Language.TypeConstraintAst]) -and $N.TypeName.FullName -cnotin $Types){$Unknown.Add('type:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.AttributeAst] -and $N.TypeName.FullName -cne 'AllowEmptyString'){$Unknown.Add('attribute:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.CommandAst] -and $N.GetCommandName() -cnotin $Commands){$Unknown.Add('command:'+$N.Extent.Text)}
    if($N -is [Management.Automation.Language.InvokeMemberExpressionAst]){if($N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $N.Member.Value -cnotin $Methods){$Unknown.Add('method:'+$N.Member.Extent.Text)}}
    elseif($N -is [Management.Automation.Language.MemberExpressionAst]){if($N.Static){if($N.Member.Value -cne 'Ordinal'){$Unknown.Add('static:'+ $N.Member.Extent.Text)}}elseif($N.Member.Value -cnotin $Properties){$Unknown.Add('property:'+$N.Member.Extent.Text)}}
    if($N -is [Management.Automation.Language.VariableExpressionAst] -and $N.VariablePath.IsDriveQualified -and -not $N.VariablePath.UserPath.StartsWith('script:',[StringComparison]::Ordinal) -and $N.VariablePath.UserPath -cne 'global:LASTEXITCODE'){$Unknown.Add('drive:'+$N.VariablePath.UserPath)}
    if($N -is [Management.Automation.Language.FileRedirectionAst]){$Unknown.Add('redirection')}
  }
  if($Unknown.Count -ne 0){throw ('WO24 fatal selected-program ZERO gate refused: '+[string]::Join('|',$Unknown))}
  $Result=& ([scriptblock]::Create($Program))
  return [pscustomobject]@{Assertions=$Result.Assertions;Cases=$Result.Cases;Hooks=$Result.Hooks;Pins=$Pins.ToArray();Gate=[pscustomobject]@{ParseErrors=0;Unknown=0;RealAccessors=0;CompiledPrograms=1}}
}

function Invoke-OwnedCaptureStartupAttributionControls {
  param([string]$EvidenceText, $SelectedProgram = $null)
  $Tokens=$null;$Errors=$null
  $Ast=[Management.Automation.Language.Parser]::ParseInput($EvidenceText,[ref]$Tokens,[ref]$Errors)
  if($Errors.Count -ne 0){throw 'startup attribution source parse refused'}
  function Select-StartupOwner([string]$Name){
    $Matches=@($Ast.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst] -and $N.Name -ceq $Name},$true))
    if($Matches.Count -ne 1){throw ('startup attribution unique owner refused: '+$Name)}
    return $Matches[0]
  }
  $Capture=Select-StartupOwner 'Invoke-HumBinaryCapture';$Complete=Select-StartupOwner 'Assert-HumCaptureComplete'
  $Statements=@($Capture.Body.EndBlock.Statements)
  function Select-StartupStatement([string]$Pattern){
    $Matches=@($Statements|Where-Object{$_.Extent.Text -cmatch $Pattern})
    if($Matches.Count -ne 1){throw ('startup attribution statement refused: '+$Pattern)}
    return $Matches[0]
  }
  $Timer=Select-StartupStatement '^\$Timer = '
  $Frequency=Select-StartupStatement '^\$Frequency = '
  $Deadline=Select-StartupStatement '^\$DeadlineTicks = '
  $Grace=Select-StartupStatement '^\$GraceTicks = '
  $List=Select-StartupStatement '^\$StartupCheckpoints = '
  $Sampler=Select-StartupStatement '^\$RecordStartupCheckpoint = '
  $InitStart=Select-StartupStatement "^& \`$RecordStartupCheckpoint 'record-init-start'$"
  $Init=Select-StartupStatement '^Initialize-HumCaptureRecord '
  $InitEnd=Select-StartupStatement "^& \`$RecordStartupCheckpoint 'record-init-end'$"
  $State=Select-StartupStatement '^\$State = '
  $ErrorInit=Select-StartupStatement "^\`$CaptureError = ''$"
  $Append=Select-StartupStatement '^if \(-not \[string\]::IsNullOrEmpty\(\$CaptureError\)\)'
  $Writer=Select-StartupStatement '^Set-HumDurableBytes .*capture_error.bin.*TrimStart'
  $Try=@($Statements|Where-Object{$_ -is [Management.Automation.Language.TryStatementAst] -and $_.Body.Extent.Text.Contains('[HumFastJobNative]::CreateSuspended')})
  if($Try.Count -ne 1){throw 'startup attribution try owner refused'}
  $Native=@($Try[0].Body.Statements|Where-Object{$_ -is [Management.Automation.Language.IfStatementAst] -and $_.Clauses[0].Item2.Extent.Text.Contains('[HumFastJobNative]::CreateSuspended')})
  if($Native.Count -ne 1){throw 'startup attribution native branch refused'}
  $Setup=@($Try[0].Body.Statements|Where-Object{$_.Extent.StartOffset -lt $Native[0].Extent.StartOffset}|ForEach-Object{$_.Extent.Text})
  $Catch=$Try[0].CatchClauses[0].Body.Statements[0].Extent.Text
  if($Catch -cne '$CaptureError = $_.Exception.ToString()'){throw 'startup attribution original error owner refused'}
  $Marks=@($Capture.FindAll({param($N)$N -is [Management.Automation.Language.CommandAst] -and $N.CommandElements[0].Extent.Text -ceq '$RecordStartupCheckpoint'},$true))
  if($Marks.Count -ne 14){throw 'startup attribution checkpoint cardinality refused'}
  $Pins=[Collections.Generic.List[object]]::new()
  foreach($Part in @($Timer,$Frequency,$Deadline,$Grace,$List,$Sampler,$InitStart,$Init,$InitEnd,$Native[0],$Append,$Writer,$Complete)){
    $Hash=[Security.Cryptography.SHA256]::Create();try{$Digest=[BitConverter]::ToString($Hash.ComputeHash([Text.Encoding]::UTF8.GetBytes($Part.Extent.Text))).Replace('-','').ToLowerInvariant()}finally{$Hash.Dispose()}
    $Pins.Add([pscustomobject]@{StartUTF16=$Part.Extent.StartOffset;EndUTF16=$Part.Extent.EndOffset;UTF8TextSha256=$Digest})
  }
  $Clock=$Timer.Extent.Text.Replace('[System.Diagnostics.Stopwatch]::StartNew()','(New-StartupOwnedTimer)')+"`n"+$Frequency.Extent.Text.Replace('[System.Diagnostics.Stopwatch]::Frequency','1000')+"`n"+$Deadline.Extent.Text+"`n"+$Grace.Extent.Text
  $Sample=$Sampler.Extent.Text.Replace('$Timer.ElapsedTicks','(Read-StartupOwnedTicks $Timer)')
  $NativeBody=$Native[0].Clauses[0].Item2.Extent.Text
  if(-not $NativeBody.StartsWith('{',[StringComparison]::Ordinal) -or -not $NativeBody.EndsWith('}',[StringComparison]::Ordinal)){throw 'startup attribution native body extent refused'}
  $Startup=[string]::Join("`n",$Setup)+"`n"+$NativeBody.Substring(1,$NativeBody.Length-2)
  $Replacements=@(
    @('[HumFastJobNative]::CreateConfiguredJob()','(New-StartupOwnedJob)'),
    @('[HumFastJobNative]::ConfigureKillOnClose($JobHandle)','Set-StartupOwnedKillOnClose $JobHandle'),
    @('[HumFastJobNative]::CreateSuspended($FilePath, $CommandLine, $WorkingDirectory)','(New-StartupOwnedProcess $FilePath $CommandLine $WorkingDirectory)'),
    @('[HumFastJobNative]::OpenAsyncReadStream($NativeProcess.StdoutReadHandle)',"(Open-StartupOwnedPipe 'stdout')"),
    @('[HumFastJobNative]::OpenAsyncReadStream($NativeProcess.StderrReadHandle)',"(Open-StartupOwnedPipe 'stderr')"),
    @('$StdoutRead.CopyToAsync($StdoutCapture)',"(Start-StartupOwnedCopy 'stdout')"),
    @('$StderrRead.CopyToAsync($StderrCapture)',"(Start-StartupOwnedCopy 'stderr')"),
    @('[HumFastJobNative]::Assign($JobHandle, $NativeProcess.ProcessHandle)','Set-StartupOwnedAssignment $JobHandle'),
    @('[HumFastJobNative]::Resume($NativeProcess.ThreadHandle)','Set-StartupOwnedResume $NativeProcess.ThreadHandle'),
    @('[HumFastJobNative]::CloseHandle($NativeProcess.ThreadHandle)','Close-StartupOwnedHandle $NativeProcess.ThreadHandle'),
    @('[IntPtr]::Zero','0')
  )
  foreach($Pair in $Replacements){if(-not $Startup.Contains($Pair[0])){throw 'startup attribution accessor replacement absent'};$Startup=$Startup.Replace($Pair[0],$Pair[1])}
  $AppendText=$Append.Extent.Text.Replace("      if (`$StartupAttribution.Length -le 2048)","      `$StartupAttribution = Complete-StartupOwnedAttribution `$StartupAttribution`n      if (`$StartupAttribution.Length -le 2048)")
  $Owned=@'
$script:StartupAssertions=[Collections.Generic.List[object]]::new();$script:StartupCases=[Collections.Generic.List[object]]::new();$script:StartupHooks=[Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
function Hit-StartupOwned([string]$Name){if(-not $script:StartupHooks.ContainsKey($Name)){$script:StartupHooks[$Name]=0};$script:StartupHooks[$Name]++}
function Require-StartupOwned([bool]$Ok,[string]$Message){$script:StartupAssertions.Add([pscustomobject]@{Ok=$Ok;Message=$Message});if(-not $Ok){throw ('startup owned assertion: '+$Message)}}
function New-StartupOwnedTimer {Hit-StartupOwned 'timer-fact';$script:StartupTimer=[pscustomobject]@{ElapsedTicks=[int64]0};return $script:StartupTimer}
function Read-StartupOwnedTicks([object]$Timer){Hit-StartupOwned 'sample-clock';if($script:StartupMode -ceq 'sample-failure'){throw 'owned sample failure'};if($script:StartupMode -ceq 'sample-corruption'){return 'malformed-owned-ticks'};return $Timer.ElapsedTicks}
function Join-Path([string]$Root,[string]$Name){return $Root+'/'+$Name}
function ConvertTo-HumNativeArgument([string]$Value){return $Value}
function Initialize-HumCaptureRecord {param($Directory,$CaseName,$Kind,$Started,$Deadline,$Grace);Hit-StartupOwned 'initialize';$script:StartupTimer.ElapsedTicks=if($script:StartupMode -ceq 'before-create'){8000}else{1000}}
function Set-HumCaptureFlag {param($Directory,[string]$Name,[bool]$Value);Hit-StartupOwned 'flag';$script:StartupTimer.ElapsedTicks+=50}
function Set-HumDurableText {param([string]$Path,[string]$Value);Hit-StartupOwned 'text';$script:StartupTimer.ElapsedTicks+=50}
function Set-HumDurableBytes {param([string]$Path,[byte[]]$Bytes);Hit-StartupOwned 'error-write';$script:StartupErrorBytes=[byte[]]$Bytes.Clone()}
function New-StartupOwnedJob {Hit-StartupOwned 'job';$script:StartupTimer.ElapsedTicks+=100;return 5}
function Set-StartupOwnedKillOnClose {param($Handle);Hit-StartupOwned 'job-configure';$script:StartupTimer.ElapsedTicks+=100}
function New-StartupOwnedProcess {param($File,$Arguments,$Root);Hit-StartupOwned 'create';$script:StartupTimer.ElapsedTicks+=$(if($script:StartupMode -ceq 'before-assignment'){7000}else{1000});return [pscustomobject]@{ProcessId=101;ProcessHandle=1;ThreadHandle=2;StdoutReadHandle=3;StderrReadHandle=4}}
function Open-StartupOwnedPipe {param($Channel);Hit-StartupOwned 'pipe';$script:StartupTimer.ElapsedTicks+=50;return [pscustomobject]@{Channel=$Channel}}
function Start-StartupOwnedCopy {param($Channel);Hit-StartupOwned 'copy';$script:StartupTimer.ElapsedTicks+=50;return [pscustomobject]@{Channel=$Channel}}
function Set-StartupOwnedAssignment {param($Handle);Hit-StartupOwned 'assign';$script:StartupTimer.ElapsedTicks+=$(if($script:StartupMode -ceq 'before-resume'){6000}else{100})}
function Set-StartupOwnedResume {param($Handle);Hit-StartupOwned 'resume';$script:StartupTimer.ElapsedTicks+=100}
function Close-StartupOwnedHandle {param($Handle);Hit-StartupOwned 'close'}
function Complete-StartupOwnedAttribution([string]$Text){Hit-StartupOwned 'attribution-return';switch -CaseSensitive($script:StartupMode){'oversized-attribution'{return ('x'*2049)};'null-attribution'{return $null};'throw-attribution'{throw 'owned attribution failure'};default{return $Text}}}
function Read-HumCaptureRecord {param($Directory);Hit-StartupOwned 'record-return';return $script:StartupRetained}
function Get-HumCaptureFailureDiagnostic {param($Result);Hit-StartupOwned 'failure-diagnostic';return 'owned retained diagnostic'}
'@
  $Actual="function Invoke-StartupOwnedActual {`n"+@'
$FilePath='owned-shell';$Arguments=@('owned-argument');$WorkingDirectory='owned-root';$CaptureDirectory='owned-capture';$TimeoutSeconds=8;$TerminationGraceSeconds=2;$StartedUtc='owned-start';$WindowsPlatform=$true
$InjectedSetupFailure=if($script:StartupMode -ceq 'injected-assignment'){'assignment'}elseif($script:StartupMode -ceq 'injected-resume'){'resume'}else{''}
$Encoding=[Text.UTF8Encoding]::new($false,$true)
'@+"`n"+$Clock+"`n"+$List.Extent.Text+"`n"+$Sample+"`n"+$State.Extent.Text+"`n"+$ErrorInit.Extent.Text+"`ntry {`n"+$InitStart.Extent.Text+"`n"+$Init.Extent.Text+"`n"+$InitEnd.Extent.Text+"`n"+$Startup+"`n} catch {`n"+$Catch+"`n}`n"+@'
if($CaptureError -ceq '' -and $script:StartupMode -cne 'positive'){$CaptureError='original-primary-error'}
$OriginalError=$CaptureError
'@+"`n"+$AppendText+"`n"+$Writer.Extent.Text+"`n"+@'
$script:StartupRetained=[pscustomobject]@{CaptureDirectory=$CaptureDirectory;Launched=$State.ProcessCreated;ExitCode=$(if($State.ProcessCreated){0}else{$null});CompletionCount=1;CaptureErrorBytes=$script:StartupErrorBytes.Length;PrimaryExitObserved=$true;StdoutCompletionObserved=$true;StderrCompletionObserved=$true;JobQuiescenceObserved=$true;ContainmentKind='windows_job';ProcessCreatedSuspended=$true;JobAssignmentSucceeded=$State.Assigned;ResumeSucceeded=$State.Resumed}
$Primary='';$Credit=0
try{$null=Assert-HumCaptureComplete $script:StartupRetained;$Credit=1}catch{$Primary=$_.Exception.Message}
[pscustomobject]@{Original=$OriginalError;Serialized=[Text.Encoding]::UTF8.GetString($script:StartupErrorBytes);Checkpoints=$StartupCheckpoints.ToArray();State=$State;Primary=$Primary;Credit=$Credit;DeadlineTicks=$DeadlineTicks;GraceTicks=$GraceTicks}
'@+"`n}`n"
  $Driver=@'
foreach($Mode in @('positive','preexisting-error','before-create','before-assignment','before-resume','injected-assignment','injected-resume','sample-failure','sample-corruption','oversized-attribution','null-attribution','throw-attribution')){
  $script:StartupMode=$Mode;$script:StartupErrorBytes=[byte[]]@();$Result=Invoke-StartupOwnedActual
  Require-StartupOwned ($Result.DeadlineTicks -eq 8000 -and $Result.GraceTicks -eq 2000) "$Mode original deadline/grace"
  Require-StartupOwned ($Result.Checkpoints.Count -le 14) "$Mode bounded checkpoint count"
  if($Mode -ceq 'positive'){
    Require-StartupOwned ($Result.Serialized -ceq '' -and $Result.Primary -ceq '' -and $Result.Credit -eq 1) 'success remains error-free/accepted'
    Require-StartupOwned ($Result.Checkpoints.Count -eq 14 -and $Result.Checkpoints[0] -ceq 'record-init-start=0' -and $Result.Checkpoints[1] -ceq 'record-init-end=1000') 'actual sampler reads real owned clock transitions'
    Require-StartupOwned ($Result.State.Assigned -and $Result.State.Resumed) 'positive actual assign/resume path'
  }else{
    Require-StartupOwned ($Result.Original.Length -gt 0 -and $Result.Serialized.StartsWith($Result.Original.TrimStart("`r","`n"),[StringComparison]::Ordinal)) "$Mode original error prefix survives attribution"
    Require-StartupOwned ($Result.Serialized.Length -gt 0 -and $Result.Credit -eq 0) "$Mode no false completion credit"
    if($Mode -ceq 'before-create'){Require-StartupOwned ($Result.Primary.StartsWith("capture has no launched terminal child`n",[StringComparison]::Ordinal) -and -not $Result.State.ProcessCreated) 'pre-create expiry remains prelaunch refusal'}
    else{Require-StartupOwned ($Result.Primary -ceq 'capture transport is incomplete') "$Mode actual nonempty-error consumer refusal"}
  }
  $Cause=switch -CaseSensitive($Mode){'before-create'{'absolute deadline expired before process creation'};'before-assignment'{'absolute deadline expired before Job assignment'};'before-resume'{'absolute deadline expired before primary-thread resume'};'injected-assignment'{'injected Job assignment failure'};'injected-resume'{'injected primary-thread resume failure'};default{''}}
  if($Cause -cne ''){Require-StartupOwned ($Result.Original.Contains($Cause)) "$Mode exact unchanged original cause"}
  if($Mode -cin @('before-assignment','injected-assignment')){Require-StartupOwned (-not $Result.State.Assigned -and -not $Result.State.Resumed) "$Mode assignment/resume never credited"}
  if($Mode -cin @('before-resume','injected-resume')){Require-StartupOwned ($Result.State.Assigned -and -not $Result.State.Resumed) "$Mode no resume credit"}
  if($Mode -ceq 'sample-failure'){Require-StartupOwned ($Result.Checkpoints.Count -eq 0) 'sample getter failure is best-effort only'}
  if($Mode -ceq 'sample-corruption'){Require-StartupOwned ($Result.Serialized.Contains('record-init-start=malformed-owned-ticks')) 'corrupt clock fact reaches diagnostic without granting credit'}
  if($Mode -cin @('oversized-attribution','throw-attribution')){Require-StartupOwned ($Result.Serialized -ceq $Result.Original) "$Mode malformed diagnostic cannot erase primary"}
  $script:StartupCases.Add([pscustomobject]@{Mode=$Mode;Original=$Result.Original;Primary=$Result.Primary;Credit=$Result.Credit;Checkpoints=$Result.Checkpoints;Serialized=$Result.Serialized})
}
foreach($Name in @('timer-fact','sample-clock','initialize','job','create','flag','text','pipe','copy','assign','resume','close','attribution-return','error-write','record-return','failure-diagnostic')){Require-StartupOwned ($script:StartupHooks.ContainsKey($Name) -and $script:StartupHooks[$Name] -gt 0) ($Name+' nonzero actual boundary')}
[pscustomobject]@{Assertions=$script:StartupAssertions.ToArray();Cases=$script:StartupCases.ToArray();Hooks=$script:StartupHooks}
'@
  $Program=$Owned+"`n"+$Complete.Extent.Text+"`n"+$Actual+"`n"+$Driver
  if($null -ne $SelectedProgram){$SelectedProgram.Value=$Program}
  $GT=$null;$GE=$null;$Gate=[Management.Automation.Language.Parser]::ParseInput($Program,[ref]$GT,[ref]$GE)
  $Types=@('string','string[]','byte[]','bool','int64','Int64','object','pscustomobject','Text.UTF8Encoding','Text.Encoding','StringComparer','StringComparison','Collections.Generic.List[object]','Collections.Generic.List[string]','Collections.Generic.Dictionary[string,int]')
  $Methods=@('Add','ContainsKey','ToArray','Contains','StartsWith','TrimStart','Join','IsNullOrEmpty','ToString','Clone','GetBytes','GetString','new')
  $Properties=@('Count','ElapsedTicks','ProcessCreated','Assigned','Resumed','ProcessId','ProcessHandle','ThreadHandle','StdoutReadHandle','StderrReadHandle','CaptureDirectory','Launched','ExitCode','CompletionCount','CaptureErrorBytes','PrimaryExitObserved','StdoutCompletionObserved','StderrCompletionObserved','JobQuiescenceObserved','ContainmentKind','ProcessCreatedSuspended','JobAssignmentSucceeded','ResumeSucceeded','Exception','Message','Original','Serialized','Credit','Primary','Checkpoints','State','DeadlineTicks','GraceTicks','Length')
  $Commands=@('ForEach-Object','Out-Null')+@($Gate.FindAll({param($N)$N -is [Management.Automation.Language.FunctionDefinitionAst]},$true)|ForEach-Object{$_.Name})
  $Unknown=[Collections.Generic.List[string]]::new()
  if($GE.Count -ne 0){$Unknown.Add('parse')}
  foreach($N in $Gate.FindAll({param($N)$true},$true)){
    if(($N -is [Management.Automation.Language.TypeExpressionAst] -or $N -is [Management.Automation.Language.TypeConstraintAst]) -and $N.TypeName.FullName -cnotin $Types){$Unknown.Add('type:'+$N.TypeName.FullName)}
    if($N -is [Management.Automation.Language.CommandAst]){if($N.GetCommandName() -cnotin $Commands -and $N.CommandElements[0].Extent.Text -cne '$RecordStartupCheckpoint'){$Unknown.Add('command:'+$N.Extent.Text)}}
    if($N -is [Management.Automation.Language.InvokeMemberExpressionAst]){if($N.Member -isnot [Management.Automation.Language.StringConstantExpressionAst] -or $N.Member.Value -cnotin $Methods){$Unknown.Add('method:'+$N.Member.Extent.Text)}}
    elseif($N -is [Management.Automation.Language.MemberExpressionAst]){if($N.Static){if($N.Member.Value -cnotin @('UTF8','Ordinal')){$Unknown.Add('static:'+$N.Member.Extent.Text)}}elseif($N.Member.Value -cnotin $Properties){$Unknown.Add('property:'+$N.Member.Extent.Text)}}
    if($N -is [Management.Automation.Language.VariableExpressionAst] -and $N.VariablePath.IsDriveQualified -and -not $N.VariablePath.UserPath.StartsWith('script:',[StringComparison]::Ordinal)){$Unknown.Add('drive:'+$N.VariablePath.UserPath)}
    if($N -is [Management.Automation.Language.FileRedirectionAst]){$Unknown.Add('redirection')}
  }
  if($Unknown.Count -ne 0){throw ('startup whole-selected-program ZERO refused: '+[string]::Join('|',$Unknown))}
  $Result=& ([scriptblock]::Create($Program))
  return [pscustomobject]@{Assertions=$Result.Assertions;Cases=$Result.Cases;Hooks=$Result.Hooks;Pins=$Pins.ToArray();Gate=[pscustomobject]@{ParseErrors=0;RealAccessors=0;Unknown=0;CompiledPrograms=1};Scope='actual startup/sampler/error append/serialization/complete consumer; unchanged native lifecycle/cleanup main excluded and byte-conserved'}
}

$TestRoot = Join-Path ([IO.Path]::GetTempPath()) ('hum-phase-test-' + [Guid]::NewGuid().ToString('N'))
$null = [IO.Directory]::CreateDirectory($TestRoot)

try {
  $Passed = 0
  $Failed = 0

  function Assert-True([bool]$Condition, [string]$Message) {
    if ($Condition) { $script:Passed++; Write-Host "PASS: $Message" }
    else { $script:Failed++; Write-Host "FAIL: $Message" }
  }

  # Test 1: Completed phases with ordered timings
  Write-Host "`n=== Test 1: Completed phases ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases1'
  Invoke-HumPhase 'alpha' { Start-Sleep -Milliseconds 50 }
  Invoke-HumPhase 'beta' { Start-Sleep -Milliseconds 50 }

  $AlphaStart = Join-Path $PhaseDiagnostics 'phase-alpha-start.txt'
  $AlphaEnd = Join-Path $PhaseDiagnostics 'phase-alpha-end.txt'
  $BetaStart = Join-Path $PhaseDiagnostics 'phase-beta-start.txt'
  $BetaEnd = Join-Path $PhaseDiagnostics 'phase-beta-end.txt'

  Assert-True (Test-Path $AlphaStart) 'alpha start marker exists'
  Assert-True (Test-Path $AlphaEnd) 'alpha end marker exists'
  Assert-True (Test-Path $BetaStart) 'beta start marker exists'
  Assert-True (Test-Path $BetaEnd) 'beta end marker exists'

  # Verify ordering via ticks: alpha start < alpha end < beta start < beta end
  function Get-Ticks([string]$Path) {
    $Content = [IO.File]::ReadAllText($Path)
    if ($Content -match 'ticks=(\d+)') { return [int64]$Matches[1] }
    throw "no ticks in $Path"
  }
  $AS = Get-Ticks $AlphaStart; $AE = Get-Ticks $AlphaEnd
  $BS = Get-Ticks $BetaStart; $BE = Get-Ticks $BetaEnd
  Assert-True ($AS -lt $AE) 'alpha start before alpha end'
  Assert-True ($AE -lt $BS) 'alpha end before beta start'
  Assert-True ($BS -lt $BE) 'beta start before beta end'

  # Test 2: Interruption retains previous completion, marks current incomplete
  Write-Host "`n=== Test 2: Interruption ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases2'
  Invoke-HumPhase 'first' { Start-Sleep -Milliseconds 10 }
  try {
    Invoke-HumPhase 'second' { throw 'simulated phase failure' }
  } catch { $Caught = $_.Exception.Message }
  Assert-True ($Caught -eq 'simulated phase failure') 'exception propagated'

  $FirstEnd = Join-Path $PhaseDiagnostics 'phase-first-end.txt'
  $SecondStart = Join-Path $PhaseDiagnostics 'phase-second-start.txt'
  $SecondEnd = Join-Path $PhaseDiagnostics 'phase-second-end.txt'
  Assert-True (Test-Path $FirstEnd) 'first phase completed before interruption'
  Assert-True (Test-Path $SecondStart) 'second phase start recorded'
  Assert-True (Test-Path $SecondEnd) 'second phase end recorded even on failure'

  # Test 3: Empty PhaseDiagnostics is a no-op (no exception)
  Write-Host "`n=== Test 3: Empty diagnostics (no-op) ==="
  $PhaseDiagnostics = ''
  try {
    Invoke-HumPhase 'noop' { $script:NoopRan = $true }
    Assert-True $script:NoopRan 'phase action ran with empty diagnostics'
    Assert-True $true 'no exception with empty PhaseDiagnostics'
  } catch {
    Assert-True $false "unexpected exception: $($_.Exception.Message)"
  }

  # Test 4: Invalid phase names rejected
  Write-Host "`n=== Test 4: Invalid phase names ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases4'
  try { Write-HumPhaseMarker '../evil' 'start'; Assert-True $false 'path traversal accepted' }
  catch { Assert-True ($_.Exception.Message -like '*invalid phase name*') 'path traversal rejected' }
  try { Write-HumPhaseMarker 'ok-name_123' 'start'; Assert-True $true 'valid name accepted' }
  catch { Assert-True $false "valid name rejected: $($_.Exception.Message)" }

  # Test 5: Marker content format
  Write-Host "`n=== Test 5: Marker content ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases5'
  Write-HumPhaseMarker 'content' 'start'
  $MarkerPath = Join-Path $PhaseDiagnostics 'phase-content-start.txt'
  $Content = [IO.File]::ReadAllText($MarkerPath)
  Assert-True ($Content -match 'phase=content') 'marker has phase name'
  Assert-True ($Content -match 'event=start') 'marker has event'
  Assert-True ($Content -match 'utc=\d{4}-\d{2}-\d{2}T') 'marker has UTC timestamp'
  Assert-True ($Content -match 'ticks=\d+') 'marker has ticks'
  Assert-True ($Content -match 'frequency=\d+') 'marker has frequency'

  # Test 6: Outcome distinction (completed vs failed)
  Write-Host "`n=== Test 6: Outcome distinction ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases6'
  Invoke-HumPhase 'okphase' { $true | Out-Null }
  try { Invoke-HumPhase 'failphase' { throw 'test failure' } } catch { }
  $OkEnd = [IO.File]::ReadAllText((Join-Path $PhaseDiagnostics 'phase-okphase-end.txt'))
  $FailEnd = [IO.File]::ReadAllText((Join-Path $PhaseDiagnostics 'phase-failphase-end.txt'))
  Assert-True ($OkEnd -match 'outcome=completed') 'completed phase marked completed'
  Assert-True ($FailEnd -match 'outcome=failed') 'failed phase marked failed'

  # Test 7: Parent binding GUID in markers
  Write-Host "`n=== Test 7: Parent binding ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases7'
  $null = [IO.Directory]::CreateDirectory($PhaseDiagnostics)
  $TestGuid = 'abc123testguid456'
  [IO.File]::WriteAllText((Join-Path $PhaseDiagnostics 'run-binding.txt'), "binding_guid=$TestGuid`n", [Text.Encoding]::UTF8)
  Write-HumPhaseMarker 'bound' 'start'
  $BoundContent = [IO.File]::ReadAllText((Join-Path $PhaseDiagnostics 'phase-bound-start.txt'))
  Assert-True ($BoundContent -match "binding=$TestGuid") 'marker carries parent binding GUID'

  # Test 8: Outcome required on end (no empty outcome)
  Write-Host "`n=== Test 8: Outcome required ==="
  $PhaseDiagnostics = Join-Path $TestRoot 'phases8'
  try { Write-HumPhaseMarker 'nooutcome' 'end' ''; Assert-True $false 'empty outcome accepted' }
  catch { Assert-True ($_.Exception.Message -like '*outcome must be*') 'empty outcome rejected' }

  # === Test 9: R5 actual-consumer regression fixture ===
  # Selects the ACTUAL workflow preflight code from the repository source via AST
  # (not line slices, not a model): the parent diagnostic setup try/catch, the
  # actual capture caller (Invoke-HumPreflightCapture) and the actual diagnostic
  # consumer (Save-HumPreflightDiagnostics) from the Run Hum preflight run block
  # of .github/workflows/ci.yml, plus the actual producer functions
  # (Write-HumPhaseMarker, Invoke-HumPhase) from tools/check_all.ps1.
  # Every real environment/native/FS/clock/capture accessor in the selected
  # source is replaced with ordinal owned facts/sinks BEFORE execution; the
  # selection is rejected if any real accessor remains. Selector identities are
  # pinned to the R4-reviewed bytes and the fixture fails on drift.
  # The 24 legacy assertions above are retained producer-only controls (labeled
  # as such in the receipt); the assertions below execute the actual consumer.
  Write-Host "`n=== Test 9: R5 actual-consumer regression fixture ==="
  $NewPassed = 0; $NewFailed = 0; $NewAssertions = 0
  $NewSelection = [Collections.Generic.List[object]]::new()
  $NewCases = [Collections.Generic.List[object]]::new()
  function Test-Require([bool]$Ok, [string]$Message) {
    $script:NewAssertions++
    if ($Ok) { $script:NewPassed++; Write-Host "PASS(new): $Message" }
    else { $script:NewFailed++; Write-Host "FAIL(new): $Message" }
  }
  try {
    # --- 9a: locate and extract the actual preflight run block from ci.yml ---
    $WorkflowPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../.github/workflows/ci.yml'))
    Test-Require (Test-Path -LiteralPath $WorkflowPath) "workflow source exists at $WorkflowPath"
    $WorkflowLines = [IO.File]::ReadAllLines($WorkflowPath)
    $RunBlockIndex = -1
    $RunCandidates = [Collections.Generic.List[int]]::new()
    for ($i = 0; $i -lt $WorkflowLines.Count; $i++) {
      if ($WorkflowLines[$i] -cmatch '^        run: \|[ \t]*$') { $RunCandidates.Add($i) }
    }
    Test-Require ($RunCandidates.Count -gt 0) 'found run blocks in ci.yml'
    foreach ($Candidate in $RunCandidates) {
      $ProbeLines = [Collections.Generic.List[string]]::new()
      for ($j = $Candidate + 1; $j -lt $WorkflowLines.Count; $j++) {
        $ProbeLine = $WorkflowLines[$j]
        if ($ProbeLine.Trim() -eq '') { $ProbeLines.Add(''); continue }
        if (($ProbeLine.Length - $ProbeLine.TrimStart(' ').Length) -le 8) { break }
        $ProbeLines.Add($ProbeLine)
      }
      if (($ProbeLines -join "`n").Contains('function Save-HumPreflightDiagnostics')) { $RunBlockIndex = $Candidate; break }
    }
    Test-Require ($RunBlockIndex -ge 0) 'located the Run Hum preflight run block in ci.yml'
    $PreflightLines = [Collections.Generic.List[string]]::new()
    for ($j = $RunBlockIndex + 1; $j -lt $WorkflowLines.Count; $j++) {
      $Line = $WorkflowLines[$j]
      if ($Line.Trim() -eq '') { $PreflightLines.Add(''); continue }
      $Indent = $Line.Length - $Line.TrimStart(' ').Length
      if ($Indent -le 8) { break }
      if ($Line.Length -lt 10) { Test-Require $false "run block line $j shorter than content indent"; break }
      $PreflightLines.Add($Line.Substring(10))
    }
    $PreflightScript = $PreflightLines -join "`n"
    Test-Require ($PreflightScript.Contains('function Invoke-HumPreflightCapture')) 'run block contains the actual capture caller'
    $WTokens = $null; $WErrors = $null
    $WAst = [System.Management.Automation.Language.Parser]::ParseInput($PreflightScript, [ref]$WTokens, [ref]$WErrors)
    Test-Require ($WErrors.Count -eq 0) 'preflight run block parses with zero AST errors'
    $CheckAllText = [IO.File]::ReadAllText($CheckAllPath)
    $CTokens = $null; $CErrors = $null
    $CAst = [System.Management.Automation.Language.Parser]::ParseInput($CheckAllText, [ref]$CTokens, [ref]$CErrors)
    Test-Require ($CErrors.Count -eq 0) 'check_all.ps1 parses with zero AST errors'

    # --- 9b: unique AST selection with R4-pinned selector identities ---
    function Select-ActualSource([object]$Tree, [string]$Name, [string]$SourceLabel, [string]$ExpectedSha256) {
      $Found = @($Tree.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq $Name }, $true))
      if ($Found.Count -ne 1) { Test-Require $false "selector '$Name' in $SourceLabel is not unique (found $($Found.Count))"; return $null }
      $Ext = $Found[0].Extent
      $Sha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($Ext.Text))).Replace('-','')).ToLowerInvariant()
      $script:NewSelection.Add([pscustomobject]@{ name=$Name; source=$SourceLabel; start_utf16=$Ext.StartOffset; end_utf16=$Ext.EndOffset; line=$Ext.StartLineNumber; sha256=$Sha })
      Test-Require ($Sha -ceq $ExpectedSha256) "selector '$Name' matches R4-reviewed bytes"
      return $Ext.Text
    }
    $ActualMarker = Select-ActualSource $CAst 'Write-HumPhaseMarker' 'tools/check_all.ps1' 'e601afa49bbeb0f6d43bd3164fe4100a0aa17feac2b75f008d8d639a8d093ca6'
    $ActualPhase = Select-ActualSource $CAst 'Invoke-HumPhase' 'tools/check_all.ps1' '4f11c16095dd105da5986dd0ff98a482500a37dfca4cc49097daee35c3cf0c60'
    $ActualSave = Select-ActualSource $WAst 'Save-HumPreflightDiagnostics' 'ci.yml run block' 'fd32618e07be9e1d9fc0efca3eb9ce9493c5d4f8174902b98339553e6bc2b207'
    $ActualCaller = Select-ActualSource $WAst 'Invoke-HumPreflightCapture' 'ci.yml run block' '8ba6ec9ca424a00279b645e7e0f2aaad9dc624ea148d49ca5cf6a8d2f6c97258'
    $ParentNodes = @($WAst.EndBlock.Statements | Where-Object {
      $_ -is [System.Management.Automation.Language.TryStatementAst] -and
      $_.Body.Extent.Text.Contains('run-binding.txt') -and
      $_.Body.Extent.Text.Contains('$PhaseDiagnosticsAvailable = $true')
    })
    Test-Require ($ParentNodes.Count -eq 1) "parent setup try selector unique (found $($ParentNodes.Count))"
    $ParentTryText = $null
    if ($ParentNodes.Count -eq 1) {
      Test-Require ($ParentNodes[0].CatchClauses.Count -ge 1) 'parent setup guard has a catch clause'
      Test-Require ($ParentNodes[0].CatchClauses[0].Extent.Text.Contains('setup_failed_unavailable')) 'parent setup catch reports unavailable'
      $ParentTryText = $ParentNodes[0].Extent.Text
      $ParentSha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($ParentTryText))).Replace('-','')).ToLowerInvariant()
      $script:NewSelection.Add([pscustomobject]@{ name='parent diagnostic setup TryStatementAst'; source='ci.yml run block'; start_utf16=$ParentNodes[0].Extent.StartOffset; end_utf16=$ParentNodes[0].Extent.EndOffset; line=$ParentNodes[0].Extent.StartLineNumber; sha256=$ParentSha })
      Test-Require ($ParentSha -ceq 'd8ebac332d0c95fe4984b1623b0c3789ba9f92c260a79ee2fb29be19684ead62') 'parent setup selector matches R4-reviewed bytes'
    }

    # --- 9c: static proofs on the selected (pre-substitution) source ---
    $SavePre = $ActualSave; $CallerPre = $ActualCaller
    Test-Require (([regex]::Matches($CallerPre, 'Invoke-HumBinaryCapture')).Count -eq 1) 'actual capture call present exactly once in selected caller'
    Test-Require (([regex]::Matches($CallerPre, 'Assert-HumCaptureComplete')).Count -eq 1) 'capture completion/authentication present exactly once in selected caller'
    Test-Require (([regex]::Matches($CallerPre, 'Save-HumPreflightDiagnostics')).Count -eq 2) 'caller wires consumer on both success and failure paths'
    Test-Require ($CallerPre.Contains("Save-HumPreflightDiagnostics `$CaptureDirectory `$DiagnosticDirectory `$PhaseDiagnosticDirectory 'capture_succeeded'")) 'normal-success consumer wiring present in selected caller'
    Test-Require (([regex]::Matches($SavePre, '\$script:HumCaptureFiles')).Count -eq 1) 'capture-v3 exact inventory reference unchanged in selected consumer'

    # --- 9d: replace every real accessor with ordinal owned facts/sinks ---
    $ActualMarker = $ActualMarker.Replace('Test-Path','Test-OwnedPath').Replace('[IO.Directory]::CreateDirectory($PhaseDiagnostics)','(New-OwnedDirectory $PhaseDiagnostics)').Replace("[DateTime]::UtcNow.ToString('o')","'2026-10-09T00:00:00.0000000Z'").Replace('[Diagnostics.Stopwatch]::GetTimestamp()','(Get-OwnedTicks)').Replace('[Diagnostics.Stopwatch]::Frequency','1000').Replace('[IO.File]::ReadAllText($BindingPath)','(Read-OwnedText $BindingPath)').Replace('Join-Path','Join-OwnedPath').Replace('[IO.File]::WriteAllText($TempPath, $Content, [Text.Encoding]::UTF8)','Write-OwnedText $TempPath $Content').Replace('[IO.File]::Move($TempPath, $Path, $true)','Move-OwnedText $TempPath $Path')
    $ActualSave = $ActualSave.Replace('Test-Path','Test-OwnedPath').Replace('Get-Item','Get-OwnedItem').Replace('Get-ChildItem','Get-OwnedChildren').Replace('Join-Path','Join-OwnedPath').Replace('Get-HumFileIdentity','Get-OwnedIdentity').Replace('[IO.File]::ReadAllText($PhaseFile.FullName)','(Read-OwnedText $PhaseFile.FullName)').Replace('[IO.File]::ReadAllText($BindingFile)','(Read-OwnedText $BindingFile)').Replace('[IO.File]::WriteAllText("$Dest.invalid_reason.txt", "reason=$InvalidReason`n", [Text.Encoding]::UTF8)','Write-OwnedText ("$Dest.invalid_reason.txt") ("reason=$InvalidReason`n")').Replace('Set-HumDurableText','Write-OwnedText').Replace('[IO.Directory]::CreateDirectory($DiagnosticDirectory)','(New-OwnedDirectory $DiagnosticDirectory)').Replace('[IO.File]::Copy($Source, $Destination, $false)','Copy-OwnedText $Source $Destination').Replace('[IO.File]::Copy($PhaseFile.FullName, $Dest, $false)','Copy-OwnedText $PhaseFile.FullName $Dest').Replace('[IO.File]::ReadAllBytes($Destination)','([Text.Encoding]::UTF8.GetBytes($script:OwnedFiles[$Destination]))')
    $ActualCaller = $ActualCaller.Replace('Invoke-HumBinaryCapture','Invoke-OwnedCapture').Replace('Assert-HumCaptureComplete','Assert-OwnedCaptureComplete')
    $ActualParent = '$PhaseDiagnosticsAvailable = $false' + "`n" + $ParentTryText
    $ActualParent = $ActualParent.Replace("[Guid]::NewGuid().ToString('N')","'ownedparent123'").Replace('[IO.Directory]::CreateDirectory($PhaseDiagnosticDirectory)','(New-OwnedDirectory $PhaseDiagnosticDirectory)').Replace('${env:GITHUB_RUN_ID}','$OwnedRunId').Replace("[DateTime]::UtcNow.ToString('o')","'2026-10-09T00:00:00.0000000Z'").Replace('[IO.File]::WriteAllText((Join-Path $PhaseDiagnosticDirectory ''run-binding.txt''), $BindingContent, [Text.Encoding]::UTF8)','Write-OwnedText (Join-OwnedPath $PhaseDiagnosticDirectory ''run-binding.txt'') $BindingContent')
    $RealPattern = '\$(?:env:|\{env:)|\[(?:System\.)?Environment\]|GetEnvironment|\[(?:System\.)?IO\.(?:File|Directory|Path)\]::|\[DateTime\]::UtcNow|\[Diagnostics\.Stopwatch\]::|(?<![A-Za-z-])(?:Test-Path|Get-Item|Get-ChildItem|Join-Path|Get-HumFileIdentity|Invoke-HumBinaryCapture|Assert-HumCaptureComplete|Add-Type|Invoke-Native)\b'
    $SelectedAll = $ActualMarker + "`n" + $ActualPhase + "`n" + $ActualSave + "`n" + $ActualCaller + "`n" + $ActualParent
    Test-Require (-not [regex]::IsMatch($SelectedAll, $RealPattern)) 'selected projection retains zero real accessors'
    # R6 P1: Safety gate is FATAL. A nonfatal Test-Require must never permit
    # dynamic compilation/execution. Abort before ScriptBlock.Create if any
    # real accessor remains.
    if ([regex]::IsMatch($SelectedAll, $RealPattern)) {
      throw 'FATAL: selected projection retains real accessors; refusing dynamic execution'
    }
    # R6 P2: Register functions separately from running parent setup.
    # $SelectedAll includes $ActualParent executable code; dot-sourcing it
    # would run parent setup immediately with no case path bound.
    $SelectedFunctions = $ActualMarker + "`n" + $ActualPhase + "`n" + $ActualSave + "`n" + $ActualCaller
    if ([regex]::IsMatch($SelectedFunctions, $RealPattern)) {
      throw 'FATAL: selected functions retain real accessors; refusing dynamic execution'
    }

    # --- 9e: owned facts/sinks ---
    $script:OwnedFiles = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
    $script:OwnedDirectories = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $script:OwnedHooks = [Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
    $script:OwnedMode = ''; $script:OwnedTicks = 0; $script:CaptureExit = 0; $script:CaptureThrow = $false
    $OwnedRunId = 'owned-run'
    function Register-OwnedHook([string]$Name) { if (-not $script:OwnedHooks.ContainsKey($Name)) { $script:OwnedHooks[$Name] = 0 }; $script:OwnedHooks[$Name]++ }
    function Get-OwnedHookCount([string]$Name) { if ($script:OwnedHooks.ContainsKey($Name)) { return $script:OwnedHooks[$Name] }; return 0 }
    function Test-OwnedPath { param([string]$LiteralPath) Register-OwnedHook 'exists'; return $script:OwnedDirectories.Contains($LiteralPath) -or $script:OwnedFiles.ContainsKey($LiteralPath) }
    function Join-OwnedPath { param([string]$Parent, [string]$Leaf) Register-OwnedHook 'join'; return "$Parent/$Leaf" }
    function New-OwnedDirectory([string]$Path) { Register-OwnedHook 'mkdir'; if ($script:OwnedMode -eq 'parent-create-fail') { throw 'owned_directory_failure' }; $null = $script:OwnedDirectories.Add($Path) }
    function Write-OwnedText([string]$Path, [string]$Content) { Register-OwnedHook 'write'; if ($script:OwnedMode -in @('parent-write-fail','consumer-write-fail')) { throw 'owned_write_failure' }; $script:OwnedFiles[$Path] = $Content }
    function Move-OwnedText([string]$Source, [string]$Destination) { Register-OwnedHook 'move'; $script:OwnedFiles[$Destination] = $script:OwnedFiles[$Source]; $null = $script:OwnedFiles.Remove($Source) }
    function Read-OwnedText([string]$Path) { Register-OwnedHook 'read'; return $script:OwnedFiles[$Path] }
    function Copy-OwnedText([string]$Source, [string]$Destination) { Register-OwnedHook 'copy'; if ($script:OwnedMode -eq 'consumer-copy-fail') { throw 'owned_copy_failure' }; $script:OwnedFiles[$Destination] = $script:OwnedFiles[$Source] }
    function Get-OwnedTicks { Register-OwnedHook 'clock'; $script:OwnedTicks += 100; return $script:OwnedTicks }
    function Get-OwnedChildren { param([string]$LiteralPath, [switch]$File) Register-OwnedHook 'children'; foreach ($Key in @($script:OwnedFiles.Keys)) { if ($Key.StartsWith($LiteralPath + '/', [StringComparison]::Ordinal)) { [pscustomobject]@{ Name = $Key.Substring($LiteralPath.Length + 1); FullName = $Key } } } }
    function Get-OwnedItem { param([string]$LiteralPath, [switch]$Force) Register-OwnedHook 'item'; return [pscustomobject]@{ PSIsContainer = $script:OwnedDirectories.Contains($LiteralPath); Attributes = 0 } }
    function Get-OwnedIdentity([string]$Path) { Register-OwnedHook 'identity'; $B = [Text.Encoding]::UTF8.GetBytes($script:OwnedFiles[$Path]); return [pscustomobject]@{ Bytes = $B.Length; Sha256 = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($B)) } }
    function Invoke-OwnedCapture { param($Executable, $Arguments, $WorkingDirectory, $CaptureDirectory, $DeadlineSeconds, $CaseName) Register-OwnedHook 'capture'; if ($script:CaptureThrow) { throw 'owned_capture_refusal' }; return [pscustomobject]@{ ExitCode = $script:CaptureExit; TimedOut = $false; StdoutSha256 = 'owned-stdout'; StderrSha256 = 'owned-stderr' } }
    function Assert-OwnedCaptureComplete { param($Capture) Register-OwnedHook 'capture_authentication'; return $Capture }
    # R6 P2: Register only the function definitions here. Parent setup runs
    # per-case in 9f with $PhaseDiagnosticDirectory bound to the case path.
    . ([scriptblock]::Create($SelectedFunctions))

    # --- 9f: actual parent setup cases (success / create-fail / write-fail) ---
    foreach ($CaseMode in @('', 'parent-create-fail', 'parent-write-fail')) {
      $script:OwnedMode = $CaseMode
      $SetupDir = '/owned/setup-' + $NewCases.Count
      # R6 P2: Bind the actual parent diagnostic variable to this case's
      # fresh path BEFORE running parent setup, so parent-to-caller path
      # linkage is genuinely tested.
      $PhaseDiagnosticDirectory = $SetupDir
      $CaptureHooksBefore = Get-OwnedHookCount 'capture'
      $SetupPublic = @(. ([scriptblock]::Create($ActualParent)) 6>&1 3>&1)
      $SetupAvailable = $PhaseDiagnosticsAvailable
      # R6 P2: On success, assert the parent binding was produced at the case
      # path. On failure modes the binding is absent by design (setup failed).
      if ($CaseMode -eq '') {
        Test-Require (Test-OwnedPath (Join-OwnedPath $SetupDir 'run-binding.txt')) 'parent binding produced at case path'
      }
      $script:OwnedMode = ''; $script:CaptureExit = 0
      $ReportDir = '/owned/setup-output-' + $NewCases.Count
      $CaptureOut = @(Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' $ReportDir $SetupDir 3000 6>&1 3>&1)
      $CaptureResult = $CaptureOut[-1]
      Test-Require (($CaptureResult.ExitCode -eq 0) -and ((Get-OwnedHookCount 'capture') -gt $CaptureHooksBefore)) 'validation boundary runs after parent setup'
      Test-Require ($SetupAvailable -eq ($CaseMode -eq '')) 'parent availability flag correct'
      if ($CaseMode -ne '') { Test-Require ([bool]($SetupPublic -match 'phase_diagnostics=setup_failed_unavailable')) 'parent failure public cause present' }
      else { Test-Require ([bool]($SetupPublic -match 'phase_diagnostics=available')) 'parent success public label present' }
      $NewCases.Add([pscustomobject]@{ name = 'actual parent setup ' + $(if ($CaseMode -eq '') { 'success' } else { $CaseMode }) })
    }

    # --- 9g: actual producer creates fresh facts; actual caller selects actual consumer ---
    function New-OwnedProduce([string]$Path, [string]$Kind) {
      $script:OwnedMode = ''; $script:PhaseDiagnostics = $Path
      $null = $script:OwnedDirectories.Add($Path)
      $script:OwnedFiles["$Path/run-binding.txt"] = "binding_guid=ownedparent123`n"
      if ($Kind -eq 'start-only') { Write-HumPhaseMarker 'probe' 'start'; return }
      if ($Kind -eq 'failed') { try { Invoke-HumPhase 'probe' { throw 'owned_phase_failed' } } catch { Test-Require ($_.Exception.Message -eq 'owned_phase_failed') 'producer refusal changed' }; return }
      $Out = @(Invoke-HumPhase 'probe' { 'owned_phase_output' })
      Test-Require ($Out[0] -ceq 'owned_phase_output') 'producer output changed'
    }
    $Controls = @(
      @{ Name='normal complete'; Kind='normal'; Exit=0; Mode=''; Expected='phase_timing=probe' },
      @{ Name='failed complete'; Kind='failed'; Exit=7; Mode=''; Expected='phase_timing=probe' },
      @{ Name='interrupted start-only'; Kind='start-only'; Exit=7; Mode=''; Expected='phase_incomplete=probe;last_event=start' },
      @{ Name='absent diagnostics'; Kind='absent'; Exit=0; Mode=''; Expected='phase_diagnostics=directory_missing' },
      @{ Name='foreign binding'; Kind='normal'; Exit=0; Mode=''; Expected='reason=binding_mismatch_foreign_run' },
      @{ Name='missing outcome'; Kind='normal'; Exit=0; Mode=''; Expected='reason=missing_or_invalid_outcome' },
      @{ Name='zero frequency'; Kind='normal'; Exit=0; Mode=''; Expected='reason=frequency_not_usable' },
      @{ Name='reversed ticks'; Kind='normal'; Exit=0; Mode=''; Expected='reason=ticks_reversed' },
      @{ Name='parent fact mutation'; Kind='normal'; Exit=0; Mode=''; Expected='reason=binding_mismatch_foreign_run' },
      @{ Name='consumer write failure on success'; Kind='normal'; Exit=0; Mode='consumer-write-fail'; Expected='owned_write_failure' },
      @{ Name='consumer copy failure on failure'; Kind='failed'; Exit=7; Mode='consumer-copy-fail'; Expected='owned_copy_failure' }
    )
    foreach ($Control in $Controls) {
      $InputPath = '/owned/records-' + $NewCases.Count
      $OutputPath = '/owned/report-' + $NewCases.Count
      if ($Control.Kind -ne 'absent') { New-OwnedProduce $InputPath $Control.Kind }
      switch ($Control.Name) {
        'foreign binding' { $script:OwnedFiles["$InputPath/phase-probe-end.txt"] = $script:OwnedFiles["$InputPath/phase-probe-end.txt"].Replace('binding=ownedparent123','binding=foreignrun') }
        'missing outcome' { $script:OwnedFiles["$InputPath/phase-probe-end.txt"] = $script:OwnedFiles["$InputPath/phase-probe-end.txt"].Replace("outcome=completed`n",'') }
        'zero frequency' { foreach ($Ev in @('start','end')) { $P = "$InputPath/phase-probe-$Ev.txt"; $script:OwnedFiles[$P] = $script:OwnedFiles[$P].Replace('frequency=1000','frequency=0') } }
        'reversed ticks' {
          $StartTicks = [regex]::Match($script:OwnedFiles["$InputPath/phase-probe-start.txt"], 'ticks=(\d+)').Groups[1].Value
          $script:OwnedFiles["$InputPath/phase-probe-end.txt"] = [regex]::Replace($script:OwnedFiles["$InputPath/phase-probe-end.txt"], 'ticks=\d+', ('ticks=' + ([int64]$StartTicks - 1)))
        }
        'parent fact mutation' { $script:OwnedFiles["$InputPath/run-binding.txt"] = "binding_guid=changedparent`n" }
      }
      $KeysBefore = @($script:OwnedFiles.Keys)
      $script:OwnedMode = $Control.Mode; $script:CaptureExit = $Control.Exit; $ReadsBefore = Get-OwnedHookCount 'read'
      $Public = @(Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' $OutputPath $InputPath 3000 6>&1 3>&1)
      $CaptureResult = $Public[-1]
      $Messages = if ($Public.Count -gt 1) { @($Public[0..($Public.Count - 2)] | ForEach-Object { $_.ToString() }) } else { @() }
      $Reads = (Get-OwnedHookCount 'read') - $ReadsBefore
      Test-Require ($CaptureResult.ExitCode -eq $Control.Exit) "original capture exit preserved ($($Control.Name))"
      Test-Require (($CaptureResult.StdoutSha256 -ceq 'owned-stdout') -and ($CaptureResult.StderrSha256 -ceq 'owned-stderr')) "capture streams unchanged ($($Control.Name))"
      Test-Require ([bool]($Messages -match [regex]::Escape($Control.Expected))) "public outcome present ($($Control.Name))"
      if ($Control.Name -in @('foreign binding','missing outcome','zero frequency','reversed ticks','parent fact mutation')) {
        Test-Require (-not [bool]($Messages -match '^phase_timing=')) "corruption not promoted to valid timing ($($Control.Name))"
      }
      if ($Control.Kind -in @('normal','failed') -and $Control.Mode -ne 'consumer-write-fail') { Test-Require ($Reads -ge 2) "consumer reads parent binding + records ($($Control.Name))" }
      $AddedKeys = @($script:OwnedFiles.Keys | Where-Object { $KeysBefore -notcontains $_ })
      Test-Require ((@($AddedKeys | Where-Object { -not $_.StartsWith($OutputPath + '/', [StringComparison]::Ordinal) })).Count -eq 0) "consumer writes stay inside report directory ($($Control.Name))"
      $Status = if ($script:OwnedFiles.ContainsKey("$OutputPath/diagnostic_status.txt")) { $script:OwnedFiles["$OutputPath/diagnostic_status.txt"] } else { 'unavailable' }
      if ($Control.Mode -eq '') { Test-Require ($Status.Contains('not_success_evidence')) "diagnostic status grants no success evidence ($($Control.Name))" }
      $script:OwnedMode = ''; $script:CaptureExit = 0
      $NewCases.Add([pscustomobject]@{ name = $Control.Name })
    }

    # --- 9h: original capture refusal is preserved ---
    $script:OwnedMode = ''; $script:CaptureThrow = $true
    $RefusalInput = '/owned/refusal-input'; New-OwnedProduce $RefusalInput 'normal'
    $Refusal = ''
    try { Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' '/owned/refusal-output' $RefusalInput 3000 6>$null 3>$null } catch { $Refusal = $_.Exception.Message }
    $script:CaptureThrow = $false
    Test-Require ($Refusal -ceq 'owned_capture_refusal') 'original capture refusal preserved'
    $NewCases.Add([pscustomobject]@{ name = 'original capture refusal preserved' })

    # --- 9i: negative controls — guard/wiring removal must be detected ---
    # Note: TryStatementAst.Body.Extent.Text includes the surrounding braces, so
    # strip them to obtain the bare guard-stripped statement list.
    $MutantParentBody = $ParentNodes[0].Body.Extent.Text.Trim()
    Test-Require ($MutantParentBody.StartsWith('{') -and $MutantParentBody.EndsWith('}')) 'mutant body brace shape as expected'
    $MutantParentBody = $MutantParentBody.Substring(1, $MutantParentBody.Length - 2)
    $MutantParentBody = $MutantParentBody.Replace("[Guid]::NewGuid().ToString('N')","'ownedparent123'").Replace('[IO.Directory]::CreateDirectory($PhaseDiagnosticDirectory)','(New-OwnedDirectory $PhaseDiagnosticDirectory)').Replace('${env:GITHUB_RUN_ID}','$OwnedRunId').Replace("[DateTime]::UtcNow.ToString('o')","'2026-10-09T00:00:00.0000000Z'").Replace('[IO.File]::WriteAllText((Join-Path $PhaseDiagnosticDirectory ''run-binding.txt''), $BindingContent, [Text.Encoding]::UTF8)','Write-OwnedText (Join-OwnedPath $PhaseDiagnosticDirectory ''run-binding.txt'') $BindingContent')
    $script:OwnedMode = 'parent-write-fail'
    $MutantThrew = $false
    # R6 P2: Bind the parent path for the mutant too, so the guard-stripped
    # body executes against a real case path.
    $PhaseDiagnosticDirectory = '/owned/mutant-parent'
    try { . ([scriptblock]::Create('$PhaseDiagnosticsAvailable = $false' + "`n" + $MutantParentBody)) 6>$null 3>$null } catch { $MutantThrew = $true }
    $script:OwnedMode = ''
    Test-Require $MutantThrew 'parent guard removal is detected (guard-stripped mutant throws)'
    $SuccessTryNodes = @($WAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.TryStatementAst] -and $n.Extent.Text.Contains('Save-HumPreflightDiagnostics $CaptureDirectory $DiagnosticDirectory $PhaseDiagnosticDirectory ''capture_succeeded''') }, $true))
    Test-Require ($SuccessTryNodes.Count -eq 1) 'normal-success consumer try selector unique'
    $MutantCaller = $ActualCaller.Replace($SuccessTryNodes[0].Extent.Text, '')
    Test-Require ($MutantCaller -cne $ActualCaller) 'consumer-wiring mutant constructed (success-branch try/catch removed)'
    . ([scriptblock]::Create($MutantCaller))
    $MutantInput = '/owned/mutant-input'; New-OwnedProduce $MutantInput 'normal'
    $MutantOut = @(Invoke-HumPreflightCapture 'owned-executable' @('evidence','full') '/owned/work' '/owned/no-capture-files' '/owned/mutant-output' $MutantInput 3000 6>$null 3>$null)
    $MutantMessages = @($MutantOut | ForEach-Object { $_.ToString() })
    Test-Require (-not [bool]($MutantMessages -match 'phase_timing=probe')) 'consumer-wiring removal is detected (no timing without consumer)'
    Test-Require (-not [bool]($MutantMessages -match 'phase_diagnostic_retained=')) 'consumer-wiring removal retains nothing'
    . ([scriptblock]::Create($ActualCaller))

    Write-Host "`nR5 selector identities (UTF-16 offsets within parsed source):"
    foreach ($S in $NewSelection) { Write-Host ("  {0} [{1}] start={2} end={3} line={4} sha256={5}" -f $S.name, $S.source, $S.start_utf16, $S.end_utf16, $S.line, $S.sha256) }
    Write-Host ("R5 distinct execution cases: {0}" -f $NewCases.Count)
  } catch {
    $script:OwnedMode = ''; $script:CaptureThrow = $false
    Test-Require $false "R5 actual-consumer section aborted: $($_.Exception.Message)"
  }

  # === Test 10: Live phase-progress observer (actual-loop, owned) ===
  # Selects ACTUAL Update-HumPhaseProgress + ACTUAL while-loop + ACTUAL caller
  # via pinned AST selectors with explicit UTF16 offsets. ALL real accessors
  # replaced with owned facts/sinks; FATAL gates before each dynamic execution.
  # Executes the real loop on BOTH platform routes with owned observation/wait.
  Write-Host "`n=== Test 10: Live phase-progress observer (actual-loop) ==="
  $ObsPassed = 0; $ObsFailed = 0; $ObsAssertions = 0; $ObsCases = [Collections.Generic.List[object]]::new()
  function Test-ObsRequire([bool]$Ok, [string]$Message) {
    $script:ObsAssertions++
    if ($Ok) { $script:ObsPassed++; Write-Host "PASS(obs): $Message" }
    else { $script:ObsFailed++; Write-Host "FAIL(obs): $Message" }
  }
  function Throw-ObsFatal([string]$Message) { throw "FATAL: $Message; refusing dynamic execution" }
  try {
    $EvidencePath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot 'run_fast_evidence.ps1'))
    $EvidenceText = [IO.File]::ReadAllText($EvidencePath)
    $ETokens = $null; $EErrors = $null
    $EAst = [System.Management.Automation.Language.Parser]::ParseInput($EvidenceText, [ref]$ETokens, [ref]$EErrors)
    if ($EErrors.Count -ne 0) { Throw-ObsFatal 'run_fast_evidence.ps1 has AST errors' }
    Test-ObsRequire ($EErrors.Count -eq 0) 'run_fast_evidence.ps1 parses with zero AST errors'

    # --- 10a: unique pinned selections with explicit offsets ---
    $ObsSel = @($EAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq 'Update-HumPhaseProgress' }, $true))
    if ($ObsSel.Count -ne 1) { Throw-ObsFatal "observer selector nonunique (found $($ObsSel.Count))" }
    $ActualObs = $ObsSel[0].Extent.Text
    $ObsSha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($ActualObs))).Replace('-','')).ToLowerInvariant()
    # Pinned expected hash for R3 observer (with unavailable-cause reporting)
    $ExpectedObsSha = '34c833eb8c61c4994286f59c584908f19e364a472e2dbc45ea8e4e8569673ff7'
    if ($ObsSha -cne $ExpectedObsSha) { Throw-ObsFatal "observer hash mismatch: expected $ExpectedObsSha, got $ObsSha" }
    Write-Host ("  observer [run_fast_evidence.ps1] UTF16 [{0},{1}) line={2} sha256={3}" -f $ObsSel[0].Extent.StartOffset, $ObsSel[0].Extent.EndOffset, $ObsSel[0].Extent.StartLineNumber, $ObsSha)
    Test-ObsRequire ($ObsSha -ceq $ExpectedObsSha) 'observer matches pinned R3 identity'

    $CapSel = @($EAst.FindAll({ param($n) $n -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $n.Name -ceq 'Invoke-HumBinaryCapture' }, $true))
    if ($CapSel.Count -ne 1) { Throw-ObsFatal 'capture selector nonunique' }
    $LoopSel = @($CapSel[0].FindAll({ param($n) $n -is [System.Management.Automation.Language.WhileStatementAst] }, $true))
    if ($LoopSel.Count -lt 1) { Throw-ObsFatal 'no while loop found in capture' }
    # Select the main loop (the one containing the observer invocation) - must be unique
    $MatchingLoops = @($LoopSel | Where-Object { $_.Extent.Text.Contains('Update-HumPhaseProgress') })
    if ($MatchingLoops.Count -ne 1) { Throw-ObsFatal "matching loop count=$($MatchingLoops.Count), expected exactly 1" }
    $MainLoop = $MatchingLoops[0]
    $ActualLoop = $MainLoop.Extent.Text
    $LoopSha = ([BitConverter]::ToString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($ActualLoop))).Replace('-','')).ToLowerInvariant()
    $ExpectedLoopSha = 'fa6c26b775a6c6c93aac88915489dc06d210d00219dbb69eac60ca6b61bddc3b'
    if ($LoopSha -cne $ExpectedLoopSha) { Throw-ObsFatal "loop hash mismatch: expected $ExpectedLoopSha, got $LoopSha" }
    Write-Host ("  loop [run_fast_evidence.ps1] UTF16 [{0},{1}) line={2} sha256={3}" -f $MainLoop.Extent.StartOffset, $MainLoop.Extent.EndOffset, $MainLoop.Extent.StartLineNumber, $LoopSha)
    Test-ObsRequire ($LoopSha -ceq $ExpectedLoopSha) 'loop matches pinned R3 identity'

    # Static: caller wiring
    $CapText = $CapSel[0].Extent.Text
    $InvCount = ([regex]::Matches($CapText, 'Update-HumPhaseProgress \$PhaseDiagnosticDirectory \$PhaseObserverState')).Count
    if ($InvCount -ne 1) { Throw-ObsFatal "observer invocation count=$InvCount, expected exactly 1" }
    Test-ObsRequire ($InvCount -eq 1) 'loop observer invocation present exactly once'
    Test-ObsRequire ($CapText.Contains('observer unavailable')) 'caller-side nonfatal boundary present'

    # --- 10b: owned accessor replacements ---
    # Observer: replace FS/console accessors
    $ActualObsOwned = $ActualObs.Replace('Test-Path','Test-OwnedObsPath').Replace('Get-ChildItem','Get-OwnedObsChildren').Replace('[IO.File]::ReadAllText($BindingPath)','(Read-OwnedObsText $BindingPath)').Replace('[IO.File]::ReadAllText($File.FullName)','(Read-OwnedObsText $File.FullName)').Replace("Join-Path `$PhaseDiagnosticDirectory 'run-binding.txt'","(Join-OwnedObsPath `$PhaseDiagnosticDirectory 'run-binding.txt')").Replace('Write-Host','Write-OwnedObsHost')
    # Loop: replace observation/wait/clock/native/console accessors
    $ActualLoopOwned = $ActualLoop.Replace('Update-HumCaptureObservation','Update-OwnedCaptureObservation').Replace('Update-HumPhaseProgress','Update-HumPhaseProgress').Replace('Get-HumRemainingMilliseconds','Get-OwnedRemainingMilliseconds').Replace('[HumFastJobNative]::Wait($NativeProcess.ProcessHandle, [UInt32] $Remaining)','(Invoke-OwnedNativeWait $Remaining)').Replace('Start-Sleep','Start-OwnedSleep').Replace('Write-Host','Write-OwnedObsHost').Replace("[Environment]::GetEnvironmentVariable('VCToolsInstallDir')","'owned-vc-root'").Replace('[IO.Path]::IsPathRooted($Root)','$true').Replace('[IO.Path]::GetFullPath($Root)','$Root').Replace('Complete-HumAuthenticatedVctip','Complete-OwnedVctip')

    # Fatal zero-accessor gates
    $RealMarkers = @('[IO.File]::', '[IO.Directory]::', '[IO.Path]::', '[Environment]::', '$env:', 'HumFastJobNative]::')
    $RealCmds = @('Test-Path', 'Get-ChildItem', 'Join-Path', 'Write-Host', 'Start-Sleep', 'Update-HumCaptureObservation', 'Get-HumRemainingMilliseconds', 'Complete-HumAuthenticatedVctip')
    function Test-ZeroAccessors([string]$Text, [string]$Label) {
      $Stripped = $Text
      # Remove owned replacements before checking
      foreach ($O in @('Test-OwnedObsPath','Get-OwnedObsChildren','Join-OwnedObsPath','Write-OwnedObsHost','Read-OwnedObsText','Update-OwnedCaptureObservation','Get-OwnedRemainingMilliseconds','Invoke-OwnedNativeWait','Start-OwnedSleep','Complete-OwnedVctip')) {
        $Stripped = $Stripped.Replace($O, '')
      }
      foreach ($M in $RealMarkers) { if ($Stripped.Contains($M)) { Throw-ObsFatal "$Label retains $M" } }
      foreach ($C in $RealCmds) {
        # Word-boundary check via regex
        if ([regex]::IsMatch($Stripped, "(?<![A-Za-z-])$C(?![A-Za-z-])")) { Throw-ObsFatal "$Label retains $C" }
      }
    }
    Test-ZeroAccessors $ActualObsOwned 'observer'
    Test-ZeroAccessors $ActualLoopOwned 'loop'
    Test-ObsRequire $true 'zero real accessors in observer and loop (fatal gates passed)'

    # --- 10c: owned facts/sinks ---
    $script:ObsFiles = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::Ordinal)
    $script:ObsDirs = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $script:ObsHooks = [Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
    $script:ObsHostOut = [Collections.Generic.List[pscustomobject]]::new()
    $script:ObsReadThrow = $false; $script:ObsPathThrow = $false
    $script:ObsHostThrow = $false; $script:ObsHostNewline = "`n"
    $script:ObsThrowObserver = $false
    $script:LoopTurns = 0
    $script:LoopMaxTurns = 10
    function Register-ObsHook([string]$Name) { if (-not $script:ObsHooks.ContainsKey($Name)) { $script:ObsHooks[$Name] = 0 }; $script:ObsHooks[$Name]++ }
    function Get-ObsHookCount([string]$Name) { if ($script:ObsHooks.ContainsKey($Name)) { return $script:ObsHooks[$Name] }; return 0 }
    function Test-OwnedObsPath { param([string]$LiteralPath) if ($script:ObsPathThrow) { Register-ObsHook 'obs_path_refusal'; throw 'owned_discovery_refusal' }; Register-ObsHook 'obs_exists'; return $script:ObsDirs.Contains($LiteralPath) -or $script:ObsFiles.ContainsKey($LiteralPath) }
    function Join-OwnedObsPath { param([string]$Parent, [string]$Leaf) Register-ObsHook 'obs_join'; return "$Parent/$Leaf" }
    function Get-OwnedObsChildren { param([string]$LiteralPath, [switch]$File) Register-ObsHook 'obs_children'; foreach ($Key in @($script:ObsFiles.Keys)) { if ($Key.StartsWith($LiteralPath + '/', [StringComparison]::Ordinal)) { [pscustomobject]@{ Name = $Key.Substring($LiteralPath.Length + 1); FullName = $Key } } } }
    function Read-OwnedObsText([string]$Path) { Register-ObsHook 'obs_read'; if ($script:ObsReadThrow) { throw 'owned_obs_read_refusal' }
      # Fact corruption: return corrupted data while preserving raw bytes
      if ($script:ObsCorruptPath -ceq $Path) { Register-ObsHook 'obs_corrupt'; return $script:ObsCorruptValue }
      # Read-return delay: return empty until delay turns reached (for late-progress test)
      if (($script:ObsDelayPath -ceq $Path) -and ($script:LoopTurns -lt $script:ObsDelayUntil)) { Register-ObsHook 'obs_delayed_read'; return '' }
      if ($script:ObsDelayPath -ceq $Path) { Register-ObsHook 'obs_delayed_read' }
      return $script:ObsFiles[$Path] }
    function Write-OwnedObsHost([string]$Message, [switch]$NoNewline) { Register-ObsHook 'obs_host'
      if ($script:ObsHostThrow) { Register-ObsHook 'obs_host_refusal'; throw 'owned_obs_sink_refusal' }
      # Record exact framing separately from message/turn/capture-active facts.
      $Framed = if ($NoNewline) { $Message } else { $Message + $script:ObsHostNewline }
      $null = $script:ObsHostOut.Add([pscustomobject]@{ Message = $Message; Framed = $Framed; NoNewline = [bool]$NoNewline; Turn = $script:LoopTurns; Active = -not $script:LoopState.PrimaryExited })
    }
    function Update-OwnedCaptureObservation { param($State, $Win, $NP, $MP, $JH, $ST, $ET, $CD) Register-ObsHook 'obs_observe'; $script:LoopTurns++; $Timer.ElapsedTicks += 1000
      # Script-controlled completion: set State flags when turns reached
      if ($script:LoopTurns -ge $script:LoopCompleteAfter) { $State.PrimaryExited = $true; $State.StdoutCompleted = $true; $State.StderrCompleted = $true; $State.JobQuiescent = $true }
    }
    function Get-OwnedRemainingMilliseconds { param($T, $D) Register-ObsHook 'obs_remaining'; if ($script:LoopTurns -ge $script:LoopMaxTurns) { return 0 }; return 50 }
    function Invoke-OwnedNativeWait { param($Ms) Register-ObsHook 'obs_native_wait'; }
    function Start-OwnedSleep { param([int]$Milliseconds) Register-ObsHook 'obs_managed_wait'; }
    function Complete-OwnedVctip { param($S, $NP, $JH, $CD, $R) Register-ObsHook 'obs_vctip'; }
    # Register actual observer with owned FS (fatal guard before compile)
    Test-ZeroAccessors $ActualObsOwned 'observer-precompile'
    . ([scriptblock]::Create($ActualObsOwned))

    function Reset-ObsCase {
      $script:ObsFiles.Clear(); $script:ObsDirs.Clear(); $script:ObsHostOut.Clear()
      $script:ObsReadThrow = $false; $script:ObsPathThrow = $false
      $script:ObsHostThrow = $false; $script:ObsHostNewline = "`n"
      $script:ObsCorruptPath = ''; $script:ObsCorruptValue = ''; $script:ObsDelayPath = ''; $script:ObsDelayUntil = 0
      $script:ObsHooks.Clear(); $script:LoopTurns = 0; $script:LoopState = $null
    }
    function New-ObsBinding([string]$Dir, [string]$Guid) {
      $null = $script:ObsDirs.Add($Dir)
      $script:ObsFiles["$Dir/run-binding.txt"] = "binding_guid=$Guid`n"
    }
    function New-ObsMarker([string]$Dir, [string]$Name, [string]$Content) {
      $script:ObsFiles["$Dir/$Name"] = $Content
    }
    function Invoke-OwnedLoop([bool]$Windows, [string]$PhaseDir, [int]$CompleteAfter) {
      $script:LoopCompleteAfter = $CompleteAfter
      $script:LoopTurns = 0
      # Owned loop state (script scope for the loop text)
      $State = @{ PrimaryExited = $false; StdoutCompleted = $false; StderrCompleted = $false; JobQuiescent = $false; Assigned = $false; ExitCode = 0; FinalActive = 0 }
      $script:LoopState = $State
      $WindowsPlatform = $Windows
      $NativeProcess = $null; $ManagedProcess = $null; $JobHandle = $null; $StdoutTask = $null; $StderrTask = $null
      $CaptureDirectory = '/owned/capture'
      $PhaseDiagnosticDirectory = $PhaseDir
      $PhaseObserverState = @{ Binding = ''; Reported = @{} }
      $Timer = [pscustomobject]@{ ElapsedTicks = [int64]0 }
      $Frequency = [int64]1000
      $DeadlineTicks = [int64]3000000
      $DeadlineDisposition = 'pending'
      $FinalTree = ''
      $VctipGraceDeadline = $null; $TerminationDeadline = 0; $GraceTicks = 3000; $TerminationDisposition = ''
      Test-ZeroAccessors $ActualLoopOwned 'loop-precompile'
      . ([scriptblock]::Create($ActualLoopOwned))
      return [pscustomobject]@{ Disposition = $DeadlineDisposition; Turns = $script:LoopTurns; Tree = $FinalTree }
    }
    function Get-ObsMessages { ($script:ObsHostOut | ForEach-Object { $_.Message }) -join '|' }
    function Get-ObsEvent([string]$Pattern) { $script:ObsHostOut | Where-Object { $_.Message -match $Pattern } | Select-Object -First 1 }

    # --- 10d: actual loop executions ---
    # Case L1: Windows route, start before completion
    Reset-ObsCase; New-ObsBinding '/obs/l1' 'guidl1'
    New-ObsMarker '/obs/l1' 'phase-alpha-start.txt' "phase=alpha`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl1`n"
    $R1 = Invoke-OwnedLoop $true '/obs/l1' 3
    $J1 = Get-ObsMessages
    $E1 = Get-ObsEvent 'phase_progress: started alpha'
    Test-ObsRequire (($null -ne $E1) -and $E1.Active -and ($E1.Turn -lt $R1.Turns) -and ($R1.Disposition -eq 'completed_before_deadline')) 'L1: Windows route reports start while capture active (turn < completion)'
    Test-ObsRequire ((Get-ObsHookCount 'obs_native_wait') -gt 0) 'L1: Windows route uses owned native wait'
    $ObsCases.Add('loop-windows-start-before-completion')

    # Case L2: Non-Windows route, start before completion
    Reset-ObsCase; New-ObsBinding '/obs/l2' 'guidl2'
    New-ObsMarker '/obs/l2' 'phase-beta-start.txt' "phase=beta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl2`n"
    $R2 = Invoke-OwnedLoop $false '/obs/l2' 3
    $J2 = Get-ObsMessages
    $E2 = Get-ObsEvent 'phase_progress: started beta'
    Test-ObsRequire (($null -ne $E2) -and $E2.Active -and ($E2.Turn -lt $R2.Turns) -and ($R2.Disposition -eq 'completed_before_deadline')) 'L2: managed route reports start while capture active (turn < completion)'
    Test-ObsRequire ((Get-ObsHookCount 'obs_managed_wait') -gt 0) 'L2: managed route uses owned sleep wait'
    $ObsCases.Add('loop-managed-start-before-completion')

    # Case L3: chronology — start+end together, start first
    Reset-ObsCase; New-ObsBinding '/obs/l3' 'guidl3'
    New-ObsMarker '/obs/l3' 'phase-gamma-start.txt' "phase=gamma`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl3`n"
    New-ObsMarker '/obs/l3' 'phase-gamma-end.txt' "phase=gamma`nevent=end`nutc=2026-10-10T00:00:01Z`nticks=1100`nfrequency=1000`nbinding=guidl3`noutcome=completed`n"
    $R3 = Invoke-OwnedLoop $true '/obs/l3' 2
    $J3 = Get-ObsMessages
    $SI = $J3.IndexOf('started gamma'); $EI = $J3.IndexOf('gamma completed')
    Test-ObsRequire (($SI -ge 0) -and ($EI -gt $SI)) 'L3: loop emits start before end when both present'
    $ObsCases.Add('loop-chronology')

    # Case L4: corrupt read-return preserves input, public invalid, no completion
    Reset-ObsCase; New-ObsBinding '/obs/l4' 'guidl4'
    $RawInput = "phase=delta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl4`n"
    New-ObsMarker '/obs/l4' 'phase-delta-start.txt' $RawInput
    # Corrupt the authoritative READ RETURN (not the input): simulate refusal
    $script:ObsReadThrow = $true
    $R4 = Invoke-OwnedLoop $true '/obs/l4' 2
    $J4 = Get-ObsMessages
    # Input preserved (not modified by observer)
    Test-ObsRequire ($script:ObsFiles['/obs/l4/phase-delta-start.txt'] -ceq $RawInput) 'L4: corrupt read preserves raw owned input'
    Test-ObsRequire ($J4 -match 'observer unavailable \(parent_read_failed\)') 'L4: read refusal yields public unavailable cause'
    Test-ObsRequire (-not ($J4 -match 'delta (completed|failed)')) 'L4: no false completion on read failure'
    Test-ObsRequire ($R4.Disposition -eq 'completed_before_deadline') 'L4: loop completes normally despite observer refusal'
    $ObsCases.Add('loop-read-refusal-unavailable')

    # Case L5: throwing observer → caller boundary reports unavailable once, loop continues
    Reset-ObsCase; New-ObsBinding '/obs/l5' 'guidl5'
    New-ObsMarker '/obs/l5' 'phase-eps-start.txt' "phase=eps`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl5`n"
    # Temporarily redefine observer to throw (owned redefinition, restored after)
    function Update-HumPhaseProgress { param([string]$PhaseDiagnosticDirectory, [hashtable]$ObserverState) throw 'owned_observer_refusal' }
    $R5 = Invoke-OwnedLoop $true '/obs/l5' 2
    # Restore observer by dot-sourcing in CURRENT scope (not via helper - scope matters)
    Test-ZeroAccessors $ActualObsOwned 'observer-restore'
    . ([scriptblock]::Create($ActualObsOwned))
    $J5 = Get-ObsMessages
    Test-ObsRequire (($J5 -match 'phase_progress: observer unavailable') -and ($R5.Disposition -eq 'completed_before_deadline')) 'L5: throwing observer isolated, loop continues to completion'
    # Only once
    Test-ObsRequire (([regex]::Matches($J5, 'observer unavailable')).Count -eq 1) 'L5: unavailable reported exactly once'
    $ObsCases.Add('loop-throwing-observer-isolated')

    # Case L6: hookup removal kills progress, loop continues (negative control)
    Reset-ObsCase; New-ObsBinding '/obs/l6' 'guidl6'
    New-ObsMarker '/obs/l6' 'phase-zeta-start.txt' "phase=zeta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl6`n"
    $MutantLoop = $ActualLoopOwned.Replace('Update-HumPhaseProgress $PhaseDiagnosticDirectory $PhaseObserverState', '# observer invocation removed')
    Test-ZeroAccessors $MutantLoop 'mutant-loop'
    $script:LoopCompleteAfter = 2; $script:LoopTurns = 0
    $State = @{ PrimaryExited = $false; StdoutCompleted = $false; StderrCompleted = $false; JobQuiescent = $false; Assigned = $false; ExitCode = 0; FinalActive = 0 }
    $WindowsPlatform = $true
    $NativeProcess = $null; $ManagedProcess = $null; $JobHandle = $null; $StdoutTask = $null; $StderrTask = $null
    $CaptureDirectory = '/owned/capture'
    $PhaseDiagnosticDirectory = '/obs/l6'
    $PhaseObserverState = @{ Binding = ''; Reported = @{} }
    $Timer = [pscustomobject]@{ ElapsedTicks = [int64]0 }
    $Frequency = [int64]1000; $DeadlineTicks = [int64]3000000; $DeadlineDisposition = 'pending'; $FinalTree = ''
    $VctipGraceDeadline = $null; $TerminationDeadline = 0; $GraceTicks = 3000; $TerminationDisposition = ''
    Test-ZeroAccessors $MutantLoop 'mutant-precompile'
    . ([scriptblock]::Create($MutantLoop))
    $J6 = Get-ObsMessages
    Test-ObsRequire ((-not ($J6 -match 'phase_progress:')) -and ($DeadlineDisposition -eq 'completed_before_deadline')) 'L6: hookup removal kills progress, loop still completes'
    $ObsCases.Add('loop-hookup-removal-negative')

    # Case L7: absent observer → zero FS hooks, loop completes
    Reset-ObsCase
    $R7 = Invoke-OwnedLoop $true '' 2
    Test-ObsRequire (((Get-ObsHookCount 'obs_exists') -eq 0) -and ((Get-ObsHookCount 'obs_children') -eq 0) -and ($R7.Disposition -eq 'completed_before_deadline')) 'L7: absent observer does zero FS work'
    $ObsCases.Add('loop-absent-observer-quiet')

    # Case L8: deadline path — remaining returns 0
    Reset-ObsCase; New-ObsBinding '/obs/l8' 'guidl8'
    $script:LoopMaxTurns = 0  # Force immediate deadline
    $R8 = Invoke-OwnedLoop $true '/obs/l8' 999
    Test-ObsRequire ($R8.Turns -eq 1) 'L8: deadline breaks loop after one observation turn'
    $script:LoopMaxTurns = 10
    $ObsCases.Add('loop-deadline')

    # Case L9: authoritative parent-binding READ RETURN corruption
    # Corrupt the read return (not the raw bytes); assert public invalid + preserved input
    Reset-ObsCase; New-ObsBinding '/obs/l9' 'guidl9'
    $RawBinding = $script:ObsFiles['/obs/l9/run-binding.txt']
    New-ObsMarker '/obs/l9' 'phase-theta-start.txt' "phase=theta`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl9`n"
    $script:ObsCorruptPath = '/obs/l9/run-binding.txt'
    $script:ObsCorruptValue = "binding_guid=CORRUPTED`n"
    $R9 = Invoke-OwnedLoop $true '/obs/l9' 2
    $J9 = Get-ObsMessages
    Test-ObsRequire ($script:ObsFiles['/obs/l9/run-binding.txt'] -ceq $RawBinding) 'L9: corruption preserves raw owned bytes'
    Test-ObsRequire ((Get-ObsHookCount 'obs_corrupt') -gt 0) 'L9: nonzero corruption hooks (read return corrupted)'
    Test-ObsRequire (($J9 -match 'theta invalid \(binding_mismatch_foreign\)') -and -not ($J9 -match 'theta (completed|failed)')) 'L9: corrupt binding yields public invalid, no false completion'
    $script:ObsCorruptPath = ''
    $ObsCases.Add('loop-binding-corruption')

    # Case L10: authoritative marker-phase READ RETURN corruption
    Reset-ObsCase; New-ObsBinding '/obs/l10' 'guidl10'
    $RawMarker = "phase=iota`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl10`n"
    New-ObsMarker '/obs/l10' 'phase-iota-start.txt' $RawMarker
    $script:ObsCorruptPath = '/obs/l10/phase-iota-start.txt'
    $script:ObsCorruptValue = "phase=WRONG`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl10`n"
    $R10 = Invoke-OwnedLoop $true '/obs/l10' 2
    $J10 = Get-ObsMessages
    Test-ObsRequire ($script:ObsFiles['/obs/l10/phase-iota-start.txt'] -ceq $RawMarker) 'L10: corruption preserves raw marker bytes'
    Test-ObsRequire ((Get-ObsHookCount 'obs_corrupt') -gt 0) 'L10: nonzero corruption hooks'
    Test-ObsRequire (($J10 -match 'iota invalid \(name_event_mismatch\)') -and -not ($J10 -match 'started iota')) 'L10: corrupt phase yields public invalid, no progress'
    $script:ObsCorruptPath = ''
    $ObsCases.Add('loop-marker-corruption')

    # Case L11: late-progress sensitivity — actual read-return delay until completion
    # The start marker's READ RETURN is delayed until the final completion turn.
    # Raw bytes are present from turn 1; only the authoritative return is delayed.
    # The emitted late event must FAIL the early-progress predicate (Active=false).
    Reset-ObsCase; New-ObsBinding '/obs/l11' 'guidl11'
    $RawKappa = "phase=kappa`nevent=start`nutc=2026-10-10T00:00:00Z`nticks=100`nfrequency=1000`nbinding=guidl11`n"
    New-ObsMarker '/obs/l11' 'phase-kappa-start.txt' $RawKappa
    $script:ObsDelayPath = '/obs/l11/phase-kappa-start.txt'
    $script:ObsDelayUntil = 3  # Delay until turn 3 (completion turn)
    $R11 = Invoke-OwnedLoop $true '/obs/l11' 3
    $E11 = Get-ObsEvent 'phase_progress: started kappa'
    Test-ObsRequire ($script:ObsFiles['/obs/l11/phase-kappa-start.txt'] -ceq $RawKappa) 'L11: delay preserves raw owned bytes'
    Test-ObsRequire ((Get-ObsHookCount 'obs_delayed_read') -gt 0) 'L11: nonzero delayed-reader hooks'
    Test-ObsRequire ((Get-ObsHookCount 'obs_host') -gt 0) 'L11: nonzero public-sink hooks'
    # The late event exists but must FAIL the early-progress predicate
    $EarlyPredicate = ($null -ne $E11) -and $E11.Active -and ($E11.Turn -lt $R11.Turns)
    Test-ObsRequire (-not $EarlyPredicate) 'L11: late event fails early-progress predicate (not while active)'
    # Confirm the event was late: turn == completion turn, Active=false
    if ($null -ne $E11) {
      Test-ObsRequire ((-not $E11.Active) -and ($E11.Turn -eq $R11.Turns)) 'L11: late event has Active=false at completion turn'
    } else {
      Test-ObsRequire $false 'L11: expected late event was emitted'
    }
    $script:ObsDelayPath = ''
    $ObsCases.Add('loop-late-progress-sensitive')

    # Case L12: discovery refusal (Test-Path throws) yields unavailable cause
    # Uses owned $script:ObsPathThrow flag (no provider commands)
    Reset-ObsCase; New-ObsBinding '/obs/l12' 'guidl12'
    $script:ObsPathThrow = $true
    $R12 = Invoke-OwnedLoop $true '/obs/l12' 2
    $J12 = Get-ObsMessages
    Test-ObsRequire ((Get-ObsHookCount 'obs_path_refusal') -gt 0) 'L12: discovery refusal hooks nonzero'
    Test-ObsRequire ($J12 -match 'observer unavailable \(directory_discovery_failed\)') 'L12: discovery refusal yields public unavailable cause'
    Test-ObsRequire ($R12.Disposition -eq 'completed_before_deadline') 'L12: loop completes normally despite discovery refusal'
    $script:ObsPathThrow = $false
    $ObsCases.Add('loop-discovery-refusal')


    function Invoke-OwnedObsUnavailableFramingControls {
      # Actual unavailable reporter under both owned host terminators, including the observed missing-parent path.
      foreach ($HostNewline in @([string][char]10, ([string][char]13 + [char]10))) {
        Reset-ObsCase; $script:ObsHostNewline = $HostNewline
        $null = $script:ObsDirs.Add('/obs/framing'); $script:LoopState = @{ PrimaryExited = $false }
        $S = @{ Binding = ''; Reported = [Collections.Generic.Dictionary[string,bool]]::new([StringComparer]::Ordinal) }
        Update-HumPhaseProgress '/obs/framing' $S; Update-HumPhaseProgress '/obs/framing' $S
        Test-ObsRequire ($script:ObsHostOut.Count -eq 1) 'framing: missing parent is reported once per cause'
        Test-ObsRequire ($script:ObsHostOut[0].NoNewline -and $script:ObsHostOut[0].Framed -ceq "phase_progress: observer unavailable (parent_binding_missing)`n") 'framing: explicit LF independent of owned host terminator'
        Test-ObsRequire ((Get-ObsHookCount 'obs_exists') -eq 4 -and (Get-ObsHookCount 'obs_host') -eq 1) 'framing: actual discovery and output hooks consumed'
        $ObsCases.Add("unavailable-framing-host-$($HostNewline.Length)")
      }
      Reset-ObsCase; New-ObsBinding '/obs/framing-corrupt' 'honestguid'; $script:LoopState = @{ PrimaryExited = $false }
      $RawBinding = $script:ObsFiles['/obs/framing-corrupt/run-binding.txt']
      $script:ObsCorruptPath = '/obs/framing-corrupt/run-binding.txt'; $script:ObsCorruptValue = 'binding_guid='
      $S = @{ Binding = ''; Reported = [Collections.Generic.Dictionary[string,bool]]::new([StringComparer]::Ordinal) }
      Update-HumPhaseProgress '/obs/framing-corrupt' $S
      Test-ObsRequire ((Get-ObsHookCount 'obs_corrupt') -eq 1 -and $script:ObsFiles['/obs/framing-corrupt/run-binding.txt'] -ceq $RawBinding) 'framing: authoritative read corruption consumed with raw binding unchanged'
      Test-ObsRequire ($script:ObsHostOut.Count -eq 1 -and $script:ObsHostOut[0].NoNewline -and $script:ObsHostOut[0].Framed -ceq "phase_progress: observer unavailable (parent_binding_malformed)`n") 'framing: other sanitized cause retains exact body and LF'
      $ObsCases.Add('unavailable-framing-parent-corruption')
      Reset-ObsCase; $null = $script:ObsDirs.Add('/obs/framing-sink'); $script:LoopState = @{ PrimaryExited = $false }
      $S = @{ Binding = ''; Reported = [Collections.Generic.Dictionary[string,bool]]::new([StringComparer]::Ordinal) }
      $script:ObsHostThrow = $true; $Escaped = $false
      try { Update-HumPhaseProgress '/obs/framing-sink' $S } catch { $Escaped = $true }
      Test-ObsRequire (-not $Escaped -and (Get-ObsHookCount 'obs_host_refusal') -eq 1 -and $script:ObsHostOut.Count -eq 0) 'framing: sink failure remains nonfatal and does not claim emission'
      $script:ObsHostThrow = $false
      Update-HumPhaseProgress '/obs/framing-sink' $S; Update-HumPhaseProgress '/obs/framing-sink' $S
      Test-ObsRequire ($script:ObsHostOut.Count -eq 1 -and (Get-ObsHookCount 'obs_host') -eq 2 -and $script:ObsHostOut[0].Framed -ceq "phase_progress: observer unavailable (parent_binding_missing)`n") 'framing: recovered sink reports once without changing unavailable body'
      $ObsCases.Add('unavailable-framing-sink-refusal')
    }
    Invoke-OwnedObsUnavailableFramingControls
    $RetainedBindings = Invoke-OwnedRetainedHelperBindingControls ([IO.File]::ReadAllText($PSCommandPath))
    foreach ($BindingAssertion in $RetainedBindings.Assertions) { Test-ObsRequire $BindingAssertion.Ok $BindingAssertion.Message }
    $ObsCases.Add('retained-six-optional-binding-owned-controls')
    $Wo22Routing = Invoke-OwnedWo22TargetRoutingControls $CheckAllText ([IO.File]::ReadAllText((Join-Path $PSScriptRoot 'test_exact_rust_selector.ps1'))) ([IO.File]::ReadAllBytes((Join-Path $PSScriptRoot '../src/backend_cranelift.rs')))
    foreach ($RoutingAssertion in $Wo22Routing.Assertions) { Test-ObsRequire $RoutingAssertion.Ok $RoutingAssertion.Message }
    $ObsCases.Add('wo22-target-routing-owned-controls')
    $Wo23Authority = Invoke-OwnedWo23AuthorityProofControls $CheckAllText
    foreach ($AuthorityAssertion in $Wo23Authority.Assertions) { Test-ObsRequire $AuthorityAssertion.Ok $AuthorityAssertion.Message }
    $ObsCases.Add('wo23-final-crate-authority-owned-controls')
    $Wo22ProofAuthority = Invoke-OwnedWo22AuthorityProofControls $CheckAllText
    foreach ($AuthorityAssertion in $Wo22ProofAuthority.Assertions) { Test-ObsRequire $AuthorityAssertion.Ok $AuthorityAssertion.Message }
    $ObsCases.Add('wo22-four-final-crate-authority-owned-controls')
    $RemainingAuthority = Invoke-OwnedRemainingAuthorityProofControls $CheckAllText
    foreach ($AuthorityAssertion in $RemainingAuthority.Assertions) { Test-ObsRequire $AuthorityAssertion.Ok $AuthorityAssertion.Message }
    $ObsCases.Add('remaining-four-final-crate-authority-owned-controls')
    $Wo24Authority = Invoke-OwnedWo24AuthorityProofControls $CheckAllText
    foreach ($AuthorityAssertion in $Wo24Authority.Assertions) { Test-ObsRequire $AuthorityAssertion.Ok $AuthorityAssertion.Message }
    $ObsCases.Add('wo24-final-crate-authority-owned-controls')
    $StartupAttribution = Invoke-OwnedCaptureStartupAttributionControls $EvidenceText
    foreach ($StartupAssertion in $StartupAttribution.Assertions) { Test-ObsRequire $StartupAssertion.Ok $StartupAssertion.Message }
    $ObsCases.Add('capture-startup-attribution-owned-controls')
    $Wo24Incremental = Invoke-OwnedWo24IncrementalRequestControls ([IO.File]::ReadAllText((Join-Path $PSScriptRoot 'test_exact_rust_selector.ps1')))
    foreach ($IncrementalAssertion in $Wo24Incremental.Assertions) { Test-ObsRequire $IncrementalAssertion.Ok $IncrementalAssertion.Message }
    $ObsCases.Add('wo24-incremental-request-owned-controls')
    $Wo24Timing = Invoke-OwnedWo24StageTimingControls $CheckAllText $EvidenceText
    foreach ($StageAssertion in $Wo24Timing.Assertions) { Test-ObsRequire $StageAssertion.Ok $StageAssertion.Message }
    $ObsCases.Add('wo24-stage-owned-controls')
    Write-Host ("`nObs selector identities:")
    Write-Host ("  observer UTF16 [{0},{1}) sha256={2}" -f $ObsSel[0].Extent.StartOffset, $ObsSel[0].Extent.EndOffset, $ObsSha)
    Write-Host ("  loop UTF16 [{0},{1}) sha256={2}" -f $MainLoop.Extent.StartOffset, $MainLoop.Extent.EndOffset, $LoopSha)
    Write-Host ("Obs distinct execution cases: {0}" -f $ObsCases.Count)
  } catch {
    $script:ObsReadThrow = $false; $script:ObsPathThrow = $false; $script:LoopMaxTurns = 10
    Test-ObsRequire $false "observer test aborted: $($_.Exception.Message)"
  }

  Write-Host "`n=== Results (legacy producer-only marker tests, retained): $Passed passed, $Failed failed ==="
  Write-Host "=== Results (R5 actual-consumer fixture): $NewAssertions assertions across $($NewCases.Count) distinct execution cases: $NewPassed passed, $NewFailed failed ==="
  Write-Host "=== Results (Test 10 live observer): $ObsAssertions assertions: $ObsPassed passed, $ObsFailed failed ==="
  if (($Failed + $NewFailed + $ObsFailed) -gt 0) { exit 1 }
  exit 0
} finally {
  if (Test-Path $TestRoot) { Remove-Item -Recurse -Force $TestRoot }
}
