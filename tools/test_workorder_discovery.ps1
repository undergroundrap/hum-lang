# Regression test for Find-ActiveWorkorder.ps1.
#
# Proves that active Work Order discovery works by marker, not by filename:
# renaming or moving the marked file must not break discovery. Also proves
# the fail-closed cases (zero marked files, two marked files).

$ErrorActionPreference = 'Stop'

. $PSScriptRoot/Find-ActiveWorkorder.ps1

$Script:Passed = 0
$Script:Failed = 0

function Assert-DiscoveryTest {
    param([string] $Name, [scriptblock] $Body)
    try {
        & $Body
        $Script:Passed++
        Write-Host "ok - $Name"
    } catch {
        $Script:Failed++
        Write-Host "FAIL - $Name : $_"
    }
}

function New-DiscoveryFixture {
    param([string] $Name)
    $Root = Join-Path ([IO.Path]::GetTempPath()) "wo-discovery-test-$Name"
    if (Test-Path -LiteralPath $Root) { Remove-Item -LiteralPath $Root -Recurse -Force }
    $Active = Join-Path $Root 'workorders/active'
    New-Item -ItemType Directory -Path $Active | Out-Null
    return @{ Root = $Root; Active = $Active }
}

function Write-WorkorderFile {
    param([string] $Directory, [string] $FileName, [bool] $Marked)
    $Marker = if ($Marked) { "<!-- hum-active-workorder:v1 -->`n" } else { '' }
    $Content = "# Work Order`nDate: 2026-09-22`n${Marker}Status: active`n"
    [IO.File]::WriteAllText((Join-Path $Directory $FileName), $Content)
}

# 1. Discovery finds the marked file under its real name.
Assert-DiscoveryTest 'finds_marked_file' {
    $Fix = New-DiscoveryFixture 'finds'
    try {
        Write-WorkorderFile -Directory $Fix.Active -FileName 'WORKORDER_25.md' -Marked $true
        $Found = Find-ActiveWorkorder -Directory $Fix.Active -RepoRoot $Fix.Root
        if ($Found.FullPath -ne (Join-Path $Fix.Active 'WORKORDER_25.md')) { throw "wrong path: $($Found.FullPath)" }
        if ($Found.RepoPath -ne 'workorders/active/WORKORDER_25.md') { throw "wrong repo path: $($Found.RepoPath)" }
    } finally {
        Remove-Item -LiteralPath $Fix.Root -Recurse -Force
    }
}

# 2. Discovery follows the marker when the file is renamed (the regression).
Assert-DiscoveryTest 'follows_marker_after_rename' {
    $Fix = New-DiscoveryFixture 'rename'
    try {
        Write-WorkorderFile -Directory $Fix.Active -FileName 'WORKORDER_99.md' -Marked $true
        $Found = Find-ActiveWorkorder -Directory $Fix.Active -RepoRoot $Fix.Root
        if ($Found.FullPath -ne (Join-Path $Fix.Active 'WORKORDER_99.md')) { throw "wrong path: $($Found.FullPath)" }
        if ($Found.RepoPath -ne 'workorders/active/WORKORDER_99.md') { throw "wrong repo path: $($Found.RepoPath)" }
    } finally {
        Remove-Item -LiteralPath $Fix.Root -Recurse -Force
    }
}

# 3. Unmarked files are ignored.
Assert-DiscoveryTest 'ignores_unmarked_files' {
    $Fix = New-DiscoveryFixture 'unmarked'
    try {
        Write-WorkorderFile -Directory $Fix.Active -FileName 'WORKORDER_24.md' -Marked $false
        Write-WorkorderFile -Directory $Fix.Active -FileName 'WORKORDER_27.md' -Marked $true
        $Found = Find-ActiveWorkorder -Directory $Fix.Active -RepoRoot $Fix.Root
        if ($Found.RepoPath -ne 'workorders/active/WORKORDER_27.md') { throw "wrong repo path: $($Found.RepoPath)" }
    } finally {
        Remove-Item -LiteralPath $Fix.Root -Recurse -Force
    }
}

# 4. Zero marked files fails closed.
Assert-DiscoveryTest 'zero_marked_fails_closed' {
    $Fix = New-DiscoveryFixture 'zero'
    try {
        Write-WorkorderFile -Directory $Fix.Active -FileName 'WORKORDER_25.md' -Marked $false
        try {
            Find-ActiveWorkorder -Directory $Fix.Active -RepoRoot $Fix.Root | Out-Null
            throw 'expected a throw for zero marked files'
        } catch {
            if ($_ -notmatch 'no active Work Order found') { throw "wrong error: $_" }
        }
    } finally {
        Remove-Item -LiteralPath $Fix.Root -Recurse -Force
    }
}

# 5. Two marked files fails closed.
Assert-DiscoveryTest 'two_marked_fails_closed' {
    $Fix = New-DiscoveryFixture 'two'
    try {
        Write-WorkorderFile -Directory $Fix.Active -FileName 'WORKORDER_25.md' -Marked $true
        Write-WorkorderFile -Directory $Fix.Active -FileName 'WORKORDER_26.md' -Marked $true
        try {
            Find-ActiveWorkorder -Directory $Fix.Active -RepoRoot $Fix.Root | Out-Null
            throw 'expected a throw for two marked files'
        } catch {
            if ($_ -notmatch 'multiple active Work Orders found') { throw "wrong error: $_" }
        }
    } finally {
        Remove-Item -LiteralPath $Fix.Root -Recurse -Force
    }
}

Write-Host ""
Write-Host "Discovery tests: $($Script:Passed) passed, $($Script:Failed) failed."
if ($Script:Failed -gt 0) { exit 1 }
