# The single definition of the Work Order discovery rule. The status-boundary
# classifier (tools/check_workorder_status_boundary.ps1) dot-sources this file
# and derives its marker and numbered-path patterns from these functions, so
# the two scripts can never disagree about which Work Order is active.

function Get-HumActiveWorkOrderMarker {
    # The literal marker line that designates the active Work Order.
    return '<!-- hum-active-workorder:v1 -->'
}

function Get-HumWorkOrderNumberPattern {
    # The numbered suffix shared by canonical Work Order paths, e.g. the
    # "_25" in workorders/active/WORKORDER_25.md. Numbers start at 1; there is
    # no WORKORDER_0.md.
    return '_[1-9][0-9]*'
}

function Test-HumWorkOrderNumberedLeafName {
    param([string] $Name)

    return $Name -cmatch ('^WORKORDER' + (Get-HumWorkOrderNumberPattern) + '\.md$')
}

# Finds the active Work Order by its marker, not by filename.
#
# The active Work Order is the sole regular numbered Markdown file under
# workorders/active carrying <!-- hum-active-workorder:v1 -->. This function
# implements that discovery rule so callers never hard-code a Work Order
# filename. Exactly one marked file must exist, or the function throws
# (fail closed).
#
# Returns a PSCustomObject with:
#   FullPath - the absolute path to the marked file
#   RepoPath - the path relative to the repository root, using forward slashes
#              (suitable for the GitHub contents API)

function Find-ActiveWorkorder {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Directory,

        [Parameter(Mandatory = $true)]
        [string] $RepoRoot
    )

    $Marker = Get-HumActiveWorkOrderMarker
    $ResolvedDir = [IO.Path]::GetFullPath($Directory)
    $ResolvedRoot = [IO.Path]::GetFullPath($RepoRoot)

    $Candidates = @(Get-ChildItem -LiteralPath $ResolvedDir -File | Where-Object {
        Test-HumWorkOrderNumberedLeafName $_.Name
    })

    $Marked = @($Candidates | Where-Object {
        $Content = [IO.File]::ReadAllText($_.FullName)
        $Content.Contains($Marker)
    })

    if ($Marked.Count -eq 0) {
        throw "no active Work Order found: no file under '$ResolvedDir' carries the marker '$Marker'"
    }
    if ($Marked.Count -gt 1) {
        $Names = ($Marked | ForEach-Object { $_.Name }) -join ', '
        throw "multiple active Work Orders found: $Names"
    }

    $FullPath = $Marked[0].FullName
    $Relative = [IO.Path]::GetRelativePath($ResolvedRoot, $FullPath)
    $RepoPath = $Relative -replace '\\', '/'

    return [PSCustomObject]@{
        FullPath = $FullPath
        RepoPath = $RepoPath
    }
}
