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

    $Marker = '<!-- hum-active-workorder:v1 -->'
    $ResolvedDir = [IO.Path]::GetFullPath($Directory)
    $ResolvedRoot = [IO.Path]::GetFullPath($RepoRoot)

    $Candidates = @(Get-ChildItem -LiteralPath $ResolvedDir -File -Filter '*.md' | Where-Object {
        $_.Name -match '^WORKORDER_[0-9]+\.md$'
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
