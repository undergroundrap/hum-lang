param(
  [ValidateSet('powershell', 'pwsh')][string] $ShellContract = 'pwsh',
  [string] $ScratchRoot = '',
  [switch] $EnvironmentSnapshotOnly,
  [ValidateSet('', 'preflight', 'success', 'exit23', 'empty', 'interleaved', 'unicode',
    'early-marker', 'duplicate-marker', 'nonzero-marker', 'timeout', 'descendant',
    'descendant-long', 'descendant-short', 'inherited-parent', 'redirected-parent',
    'earliest-parent', 'quiescent-parent', 'inherited-short-parent')]
  [string] $SyntheticChild = ''
)

$ErrorActionPreference = 'Stop'
$SuccessMarker = 'All Hum preflight checks passed.'
$script:HumCaptureTestPath = $MyInvocation.MyCommand.Path

function Write-ExactBytes {
  param([System.IO.Stream] $Stream, [byte[]] $Bytes)
  $Stream.Write($Bytes, 0, $Bytes.Length)
  $Stream.Flush()
}

function Write-ExactAscii {
  param([System.IO.Stream] $Stream, [string] $Text)
  Write-ExactBytes $Stream ([System.Text.Encoding]::ASCII.GetBytes($Text))
}

function Start-SyntheticDescendant {
  param([bool] $Redirect, [string] $Mode)
  $Self = $script:HumCaptureTestPath
  $Shell = (Get-Process -Id $PID).Path
  $Process = New-Object System.Diagnostics.Process
  $Process.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
  $Process.StartInfo.FileName = $Shell
  $Process.StartInfo.Arguments = "-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$Self`" -SyntheticChild $Mode"
  $Process.StartInfo.UseShellExecute = $false
  $Process.StartInfo.CreateNoWindow = $true
  if ($Redirect) {
    $Process.StartInfo.RedirectStandardOutput = $true
    $Process.StartInfo.RedirectStandardError = $true
  }
  if (-not $Process.Start()) { throw 'descendant did not launch' }
  $Process
}

if ($SyntheticChild -ne '') {
  $Stdout = [Console]::OpenStandardOutput()
  $Stderr = [Console]::OpenStandardError()
  switch ($SyntheticChild) {
    'preflight' {
      Write-ExactAscii $Stdout "$((Get-Process -Id $PID).Path)`n$($PSVersionTable.PSVersion.ToString())`n"
      exit 0
    }
    'success' {
      Write-ExactAscii $Stdout "CAPTURE_STDOUT`n$SuccessMarker`n"
      Write-ExactAscii $Stderr "CAPTURE_STDERR`n"
      exit 0
    }
    'exit23' {
      Write-ExactAscii $Stdout "EXIT23_STDOUT`n"
      Write-ExactAscii $Stderr "EXIT23_STDERR`n"
      exit 23
    }
    'empty' { exit 0 }
    'interleaved' {
      for ($Index = 0; $Index -lt 2048; $Index++) {
        $OutBytes = [Text.Encoding]::ASCII.GetBytes(("O{0:d5}:{1}`n" -f $Index, ('o' * 48)))
        $ErrBytes = [Text.Encoding]::ASCII.GetBytes(("E{0:d5}:{1}`n" -f $Index, ('e' * 48)))
        $Stdout.Write($OutBytes, 0, $OutBytes.Length)
        $Stderr.Write($ErrBytes, 0, $ErrBytes.Length)
      }
      $Stdout.Flush()
      $Stderr.Flush()
      exit 0
    }
    'unicode' {
      Write-ExactBytes $Stdout ([byte[]] (0x75, 0x74, 0x66, 0x38, 0x3d, 0xe2, 0x98, 0x83, 0x0d, 0x0a, 0x6c, 0x66, 0x0a))
      Write-ExactBytes $Stderr ([byte[]] (0x65, 0x72, 0x72, 0x3d, 0xf0, 0x9f, 0x8c, 0x8a, 0x0a, 0x63, 0x72, 0x0d))
      exit 0
    }
    'early-marker' {
      Write-ExactAscii $Stdout "$SuccessMarker`nlater assertion`n"
      exit 0
    }
    'duplicate-marker' {
      Write-ExactAscii $Stdout "$SuccessMarker`n$SuccessMarker`n"
      exit 0
    }
    'nonzero-marker' {
      Write-ExactAscii $Stdout "$SuccessMarker`n"
      exit 9
    }
    'descendant' {
      Write-ExactAscii $Stdout "descendant_alive=$PID`n"
      Write-ExactAscii $Stderr "descendant_partial_stderr`n"
      Start-Sleep -Seconds 60
      exit 0
    }
    'descendant-long' {
      Start-Sleep -Seconds 12
      exit 0
    }
    'descendant-short' {
      Start-Sleep -Milliseconds 250
      exit 0
    }
    'inherited-parent' {
      $Process = Start-SyntheticDescendant $false 'descendant-long'
      Write-ExactAscii $Stdout "inherited_parent_pid=$PID`ninherited_descendant_pid=$($Process.Id)`ninherited_parent_stdout`n"
      Write-ExactAscii $Stderr "inherited_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'inherited-short-parent' {
      $Process = Start-SyntheticDescendant $false 'descendant-short'
      Write-ExactAscii $Stdout "inherited_parent_pid=$PID`ninherited_descendant_pid=$($Process.Id)`ninherited_parent_stdout`n"
      Write-ExactAscii $Stderr "inherited_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'redirected-parent' {
      $Process = Start-SyntheticDescendant $true 'descendant-long'
      Write-ExactAscii $Stdout "redirected_parent_pid=$PID`nredirected_descendant_pid=$($Process.Id)`nredirected_parent_stdout`n"
      Write-ExactAscii $Stderr "redirected_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'earliest-parent' {
      $Process = Start-SyntheticDescendant $false 'descendant-long'
      Write-ExactAscii $Stdout "earliest_parent_pid=$PID`nearliest_descendant_pid=$($Process.Id)`nearliest_parent_stdout`n"
      Write-ExactAscii $Stderr "earliest_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'quiescent-parent' {
      $Process = Start-SyntheticDescendant $false 'descendant-short'
      Write-ExactAscii $Stdout "quiescent_parent_pid=$PID`nquiescent_descendant_pid=$($Process.Id)`nquiescent_parent_stdout`n"
      Write-ExactAscii $Stderr "quiescent_parent_stderr`n"
      [Environment]::Exit(0)
    }
    'timeout' {
      $Self = $MyInvocation.MyCommand.Path
      $Shell = (Get-Process -Id $PID).Path
      $Process = New-Object System.Diagnostics.Process
      $Process.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
      $Process.StartInfo.FileName = $Shell
      $Process.StartInfo.Arguments = "-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$Self`" -SyntheticChild descendant"
      $Process.StartInfo.UseShellExecute = $false
      $Process.StartInfo.CreateNoWindow = $true
      if (-not $Process.Start()) { throw 'descendant did not launch' }
      Write-ExactAscii $Stdout "parent_alive=$PID`ndescendant_pid=$($Process.Id)`nparent_partial_stdout`n"
      Write-ExactAscii $Stderr "parent_partial_stderr`n"
      $Process.WaitForExit()
      exit $Process.ExitCode
    }
  }
}

$RequestedScratchRoot = $ScratchRoot
. (Join-Path $PSScriptRoot 'run_fast_evidence.ps1')
$ScratchRoot = $RequestedScratchRoot

function Assert-True {
  param([bool] $Condition, [string] $Message)
  if (-not $Condition) { throw $Message }
}

function Read-Bytes {
  param([string] $Path)
  [System.IO.File]::ReadAllBytes($Path)
}

function Assert-Bytes {
  param([byte[]] $Actual, [byte[]] $Expected, [string] $Message)
  Assert-True ($Actual.Length -eq $Expected.Length) "$Message length"
  for ($Index = 0; $Index -lt $Actual.Length; $Index++) {
    Assert-True ($Actual[$Index] -eq $Expected[$Index]) "$Message byte $Index"
  }
}

function Assert-Throws {
  param([scriptblock] $Action, [string] $Message)
  $Rejected = $false
  try { & $Action } catch { $Rejected = $true }
  Assert-True $Rejected $Message
}

if (-not ('HumFastEnvironmentNative' -as [type])) {
  Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public sealed class HumFastEnvironmentProbe
{
    private readonly char[] characters;
    private readonly bool releaseResult;

    public bool Acquired { get; private set; }
    public int ReadCallCount { get; private set; }
    public int MaximumRequestedIndex { get; private set; }
    public int ReadAfterReleaseCount { get; private set; }
    public int ReleaseCallCount { get; private set; }
    public bool Released { get; private set; }
    public bool ReleaseResult { get { return releaseResult; } }

    private HumFastEnvironmentProbe(char[] characters, bool acquired, bool releaseResult)
    {
        this.characters = characters == null ? null : (char[])characters.Clone();
        this.Acquired = acquired;
        this.releaseResult = releaseResult;
        this.MaximumRequestedIndex = -1;
    }

    public static HumFastEnvironmentProbe Create(
        char[] characters,
        bool acquired,
        bool releaseResult)
    {
        return new HumFastEnvironmentProbe(characters, acquired, releaseResult);
    }

    public char ReadCharacter(int index)
    {
        if (Released)
        {
            ReadAfterReleaseCount++;
            throw new InvalidOperationException("environment block read after release");
        }
        ReadCallCount++;
        if (index > MaximumRequestedIndex) { MaximumRequestedIndex = index; }
        if (characters == null || index < 0 || index >= characters.Length)
        {
            throw new InvalidOperationException(
                "environment block ended before the terminal double NUL");
        }
        return characters[index];
    }

    public bool Release()
    {
        ReleaseCallCount++;
        if (ReleaseCallCount != 1)
        {
            throw new InvalidOperationException("environment block released more than once");
        }
        Released = true;
        return releaseResult;
    }
}

public static class HumFastEnvironmentNative
{
    private const int MaximumCharacters = 4 * 1024 * 1024;
    private const int MaximumEntries = 65536;

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern IntPtr GetEnvironmentStrings();

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool FreeEnvironmentStrings(IntPtr environmentBlock);

    private static char ReadBounded(
        Func<int, char> readCharacter,
        int index,
        int maximumCharacters)
    {
        if (index < 0 || index >= maximumCharacters)
        {
            throw new InvalidOperationException("environment block exceeds the character limit");
        }
        return readCharacter(index);
    }

    private static void ValidateEntry(string entry)
    {
        if (entry.Length == 0)
        {
            throw new InvalidOperationException("environment block contains an empty internal entry");
        }

        int firstEquals = entry.IndexOf('=');
        if (firstEquals > 0)
        {
            return;
        }
        if (firstEquals == 0 && entry.IndexOf('=', 1) > 1)
        {
            return;
        }
        throw new InvalidOperationException("environment block contains a malformed entry");
    }

    public static string[] ParseBoundedEnvironmentBlock(
        Func<int, char> readCharacter,
        int maximumCharacters,
        int maximumEntries)
    {
        if (readCharacter == null)
        {
            throw new ArgumentNullException("readCharacter");
        }
        if (maximumCharacters < 2)
        {
            throw new InvalidOperationException(
                "environment block character limit cannot authenticate double NUL");
        }
        if (maximumEntries < 0)
        {
            throw new InvalidOperationException("environment block entry limit is invalid");
        }

        List<string> entries = new List<string>();
        int cursor = 0;
        if (ReadBounded(readCharacter, cursor, maximumCharacters) == '\0')
        {
            if (ReadBounded(readCharacter, cursor + 1, maximumCharacters) != '\0')
            {
                throw new InvalidOperationException(
                    "empty environment block lacks terminal double NUL");
            }
            return entries.ToArray();
        }

        while (true)
        {
            if (entries.Count >= maximumEntries)
            {
                throw new InvalidOperationException("environment block exceeds the entry limit");
            }

            StringBuilder entry = new StringBuilder();
            while (true)
            {
                char value = ReadBounded(readCharacter, cursor, maximumCharacters);
                if (value == '\0')
                {
                    break;
                }
                entry.Append(value);
                cursor++;
            }

            string entryText = entry.ToString();
            ValidateEntry(entryText);
            entries.Add(entryText);
            cursor++;

            if (ReadBounded(readCharacter, cursor, maximumCharacters) == '\0')
            {
                return entries.ToArray();
            }
        }
    }

    public static void AuthenticateRelease(
        int releaseCallCount,
        bool reportedResult,
        bool actualResult)
    {
        if (releaseCallCount != 1)
        {
            throw new InvalidOperationException(
                "environment block release count must equal one");
        }
        if (reportedResult != actualResult)
        {
            throw new InvalidOperationException("environment block release result was altered");
        }
        if (!actualResult)
        {
            throw new InvalidOperationException("environment block release failed");
        }
    }

    private static string[] ReadAcquiredEnvironmentBlock(
        bool acquired,
        Func<int, char> readCharacter,
        int maximumCharacters,
        int maximumEntries,
        Func<bool> release,
        Func<int> releaseCallCount)
    {
        if (!acquired)
        {
            throw new InvalidOperationException("environment block acquisition returned null");
        }

        string[] result = null;
        bool reportedReleaseResult = false;
        bool actualReleaseResult = false;
        try
        {
            result = ParseBoundedEnvironmentBlock(
                readCharacter,
                maximumCharacters,
                maximumEntries);
        }
        finally
        {
            actualReleaseResult = release();
            reportedReleaseResult = actualReleaseResult;
            AuthenticateRelease(
                releaseCallCount(),
                reportedReleaseResult,
                actualReleaseResult);
        }
        return result;
    }

    public static string[] ReadInjectedEnvironmentBlock(
        HumFastEnvironmentProbe probe,
        int maximumCharacters,
        int maximumEntries)
    {
        if (probe == null)
        {
            throw new InvalidOperationException("environment block acquisition returned null");
        }
        return ReadAcquiredEnvironmentBlock(
            probe.Acquired,
            probe.ReadCharacter,
            maximumCharacters,
            maximumEntries,
            probe.Release,
            delegate { return probe.ReleaseCallCount; });
    }

    public static string[] ReadWindowsEnvironmentBlock()
    {
        IntPtr block = GetEnvironmentStrings();
        if (block == IntPtr.Zero)
        {
            throw new InvalidOperationException(
                "GetEnvironmentStrings failed with Win32 error " + Marshal.GetLastWin32Error());
        }

        int releaseCallCount = 0;
        return ReadAcquiredEnvironmentBlock(
            true,
            delegate(int index)
            {
                return (char)Marshal.ReadInt16(block, checked(index * 2));
            },
            MaximumCharacters,
            MaximumEntries,
            delegate
            {
                releaseCallCount++;
                return FreeEnvironmentStrings(block);
            },
            delegate { return releaseCallCount; });
    }
}
'@
}

function Get-ProcessEnvironmentEntries {
  if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) {
    [HumFastEnvironmentNative]::ReadWindowsEnvironmentBlock()
    return
  }

  $Entries = New-Object 'System.Collections.Generic.List[string]'
  foreach ($Entry in [Environment]::GetEnvironmentVariables('Process').GetEnumerator()) {
    $Entries.Add(([string] $Entry.Key) + '=' + ([string] $Entry.Value))
  }
  $Entries.ToArray()
}

function ConvertTo-EnvironmentSnapshotBytes {
  param([Parameter(Mandatory = $true)][AllowNull()][AllowEmptyCollection()][System.Array] $Entries)

  if ($null -eq $Entries) { throw 'environment snapshot entries array is null' }
  $Ordered = New-Object 'string[]' $Entries.Length
  for ($Index = 0; $Index -lt $Entries.Length; $Index++) {
    $Entry = $Entries.GetValue($Index)
    if ($null -eq $Entry) { throw 'environment snapshot entry is null' }
    if (-not ($Entry -is [string])) { throw 'environment snapshot entry is not a string' }
    $Ordered[$Index] = [string] $Entry
  }
  [Array]::Sort($Ordered, [StringComparer]::Ordinal)
  $Utf8 = New-Object System.Text.UTF8Encoding($false, $true)
  $Stream = New-Object System.IO.MemoryStream
  try {
    foreach ($Entry in $Ordered) {
      if ($Entry.IndexOf([char] 0) -ge 0) { throw 'environment snapshot entry contains NUL' }
      $Bytes = $Utf8.GetBytes($Entry)
      $Stream.Write($Bytes, 0, $Bytes.Length)
      $Stream.WriteByte(0)
    }
    return ,([byte[]] $Stream.ToArray())
  } finally {
    $Stream.Dispose()
  }
}

function Test-ExactBytesEqual {
  param([byte[]] $Left, [byte[]] $Right)
  if ($Left.Length -ne $Right.Length) { return $false }
  for ($Index = 0; $Index -lt $Left.Length; $Index++) {
    if ($Left[$Index] -ne $Right[$Index]) { return $false }
  }
  $true
}

function New-EnvironmentProbe {
  param(
    [char[]] $Characters,
    [bool] $Acquired = $true,
    [bool] $ReleaseResult = $true
  )
  [HumFastEnvironmentProbe]::Create($Characters, $Acquired, $ReleaseResult)
}

function Assert-EnvironmentNativeParserContract {
  $DoubleNul = [char[]] @([char] 0, [char] 0)
  $EmptyProbe = New-EnvironmentProbe $DoubleNul
  $EmptyEntries = @([HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($EmptyProbe, $DoubleNul.Length, 0))
  Assert-True ($EmptyEntries.Count -eq 0) 'valid empty environment block was rejected'
  Assert-True ($EmptyProbe.ReleaseCallCount -eq 1 -and $EmptyProbe.Released) 'empty block release lifecycle'
  Assert-True ($EmptyProbe.ReadAfterReleaseCount -eq 0) 'empty block read after release'

  $OrdinaryCharacters = [char[]] "B=2$([char] 0)A=1$([char] 0)$([char] 0)".ToCharArray()
  $OrdinaryProbe = New-EnvironmentProbe $OrdinaryCharacters
  $OrdinaryEntries = @([HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock(
      $OrdinaryProbe,
      $OrdinaryCharacters.Length,
      2))
  Assert-True ($OrdinaryEntries.Count -eq 2) 'valid ordinary environment block entry count'
  Assert-True ($OrdinaryEntries[0] -ceq 'B=2' -and $OrdinaryEntries[1] -ceq 'A=1') 'valid ordinary environment block order'
  Assert-True ($OrdinaryProbe.ReleaseCallCount -eq 1 -and $OrdinaryProbe.ReadAfterReleaseCount -eq 0) 'ordinary block release lifecycle'

  $SpecialEntry = '=C:=C:' + [char] 92 + 'probe'
  $SpecialCharacters = [char[]] "$SpecialEntry$([char] 0)$([char] 0)".ToCharArray()
  $SpecialProbe = New-EnvironmentProbe $SpecialCharacters
  $SpecialEntries = @([HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock(
      $SpecialProbe,
      $SpecialCharacters.Length,
      1))
  Assert-True ($SpecialEntries.Count -eq 1 -and $SpecialEntries[0] -ceq $SpecialEntry) 'valid leading-equals Windows entry rejected'
  Assert-True ($SpecialProbe.ReleaseCallCount -eq 1) 'leading-equals block release lifecycle'

  $NullProbe = New-EnvironmentProbe $DoubleNul $false $true
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($NullProbe, $DoubleNul.Length, 0)
  } 'null environment block acquisition was accepted'
  Assert-True ($NullProbe.ReadCallCount -eq 0 -and $NullProbe.ReleaseCallCount -eq 0) 'null block was parsed or released'

  $SingleNul = [char[]] @([char] 0)
  $SingleNulProbe = New-EnvironmentProbe $SingleNul
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($SingleNulProbe, 2, 0)
  } 'empty block with one NUL was accepted'
  Assert-True ($SingleNulProbe.ReleaseCallCount -eq 1) 'single-NUL rejection did not release'

  $OneTerminalNul = [char[]] "A=1$([char] 0)".ToCharArray()
  $OneTerminalProbe = New-EnvironmentProbe $OneTerminalNul
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($OneTerminalProbe, $OneTerminalNul.Length + 1, 1)
  } 'nonempty block with one terminal NUL was accepted'
  Assert-True ($OneTerminalProbe.ReleaseCallCount -eq 1) 'one-terminal-NUL rejection did not release'

  $TruncatedCharacters = [char[]] 'A='.ToCharArray()
  $TruncatedProbe = New-EnvironmentProbe $TruncatedCharacters
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($TruncatedProbe, $TruncatedCharacters.Length + 1, 1)
  } 'truncated environment entry was accepted'
  Assert-True ($TruncatedProbe.ReleaseCallCount -eq 1) 'truncated-entry rejection did not release'

  $MissingTerminatorCharacters = [char[]] 'A=1'.ToCharArray()
  $MissingTerminatorProbe = New-EnvironmentProbe $MissingTerminatorCharacters
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock(
      $MissingTerminatorProbe,
      $MissingTerminatorCharacters.Length + 1,
      1)
  } 'unterminated environment entry was accepted'
  Assert-True ($MissingTerminatorProbe.ReleaseCallCount -eq 1) 'unterminated-entry rejection did not release'

  $CharacterBoundProbe = New-EnvironmentProbe $OrdinaryCharacters
  $CharacterBound = $OrdinaryCharacters.Length - 1
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($CharacterBoundProbe, $CharacterBound, 2)
  } 'environment character-bound exhaustion was accepted'
  Assert-True ($CharacterBound -lt $OrdinaryCharacters.Length) 'character-bound exhaustion mutation did not initialize'
  Assert-True ($CharacterBoundProbe.MaximumRequestedIndex -lt $CharacterBound) 'environment parser read past character bound'
  Assert-True ($CharacterBoundProbe.ReleaseCallCount -eq 1) 'character-bound rejection did not release'
  $RepeatedReadProbe = New-EnvironmentProbe $DoubleNul
  for ($Index = 0; $Index -le $DoubleNul.Length; $Index++) { $null = $RepeatedReadProbe.ReadCharacter(0) }
  Assert-True ($RepeatedReadProbe.ReadCallCount -gt $DoubleNul.Length -and $RepeatedReadProbe.MaximumRequestedIndex -eq 0) 'repeated bounded-read control failed'
  Assert-True ($RepeatedReadProbe.Release()) 'repeated bounded-read control release failed'

  $EntryBoundProbe = New-EnvironmentProbe $OrdinaryCharacters
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($EntryBoundProbe, $OrdinaryCharacters.Length, 1)
  } 'environment entry-bound exhaustion was accepted'
  Assert-True ($EntryBoundProbe.ReleaseCallCount -eq 1) 'entry-bound rejection did not release'

  $MalformedCharacters = [char[]] "MALFORMED$([char] 0)$([char] 0)".ToCharArray()
  $MalformedProbe = New-EnvironmentProbe $MalformedCharacters
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($MalformedProbe, $MalformedCharacters.Length, 1)
  } 'malformed environment entry was accepted'
  Assert-True ($MalformedProbe.ReleaseCallCount -eq 1 -and $MalformedProbe.ReadAfterReleaseCount -eq 0) 'parse failure release lifecycle'

  $ReleaseFailureProbe = New-EnvironmentProbe $DoubleNul $true $false
  Assert-Throws {
    [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock($ReleaseFailureProbe, $DoubleNul.Length, 0)
  } 'environment release failure was accepted'
  Assert-True ($ReleaseFailureProbe.ReleaseCallCount -eq 1 -and $ReleaseFailureProbe.Released) 'release failure call count'

  Assert-Throws {
    [HumFastEnvironmentNative]::AuthenticateRelease(0, $true, $true)
  } 'missing environment release was accepted'
  Assert-Throws {
    [HumFastEnvironmentNative]::AuthenticateRelease(2, $true, $true)
  } 'duplicate environment release was accepted'
  Assert-Throws {
    [HumFastEnvironmentNative]::AuthenticateRelease(1, $true, $false)
  } 'forced-success environment release was accepted'

  $DuplicateReleaseProbe = New-EnvironmentProbe $DoubleNul
  $null = [HumFastEnvironmentNative]::ReadInjectedEnvironmentBlock(
    $DuplicateReleaseProbe,
    $DoubleNul.Length,
    0)
  Assert-Throws { $DuplicateReleaseProbe.Release() } 'duplicate release operation was accepted'
  Assert-True ($DuplicateReleaseProbe.ReleaseCallCount -eq 2) 'duplicate release mutation did not initialize'

  Assert-Throws { $OrdinaryProbe.ReadCharacter(0) } 'read after environment release was accepted'
  Assert-True ($OrdinaryProbe.ReadAfterReleaseCount -eq 1) 'read-after-release adversary did not initialize'
}

function Get-ProcessEnvironmentSnapshot {
  $Entries = @(Get-ProcessEnvironmentEntries)
  $Bytes = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] $Entries))
  $Hasher = [Security.Cryptography.SHA256]::Create()
  try {
    $Sha256 = ([BitConverter]::ToString($Hasher.ComputeHash($Bytes))).Replace('-', '').ToLowerInvariant()
  } finally {
    $Hasher.Dispose()
  }
  [pscustomobject] @{
    EntryCount = $Entries.Count
    Bytes = $Bytes
    ByteCount = $Bytes.Length
    Sha256 = $Sha256
  }
}

function Assert-EnvironmentSnapshotContract {
  Assert-EnvironmentNativeParserContract

  $EmptyA = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @()))
  $EmptyB = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @()))
  Assert-True (Test-ExactBytesEqual $EmptyA $EmptyB) 'empty environment snapshot is nondeterministic'

  $OrdinaryA = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('B=2', 'A=1')))
  $OrdinaryB = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('A=1', 'B=2')))
  Assert-True (Test-ExactBytesEqual $OrdinaryA $OrdinaryB) 'ordinary environment snapshot depends on input order'

  $CaseVariantsA = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('PATH=value-a', 'Path=value-b')))
  $CaseVariantsB = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('Path=value-b', 'PATH=value-a')))
  Assert-True (Test-ExactBytesEqual $CaseVariantsA $CaseVariantsB) 'case-variant environment snapshot depends on input order'
  $ChangedUpper = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('PATH=value-c', 'Path=value-b')))
  $ChangedLower = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('PATH=value-a', 'Path=value-c')))
  Assert-True (-not (Test-ExactBytesEqual $CaseVariantsA $ChangedUpper)) 'uppercase environment value change was lost'
  Assert-True (-not (Test-ExactBytesEqual $CaseVariantsA $ChangedLower)) 'mixed-case environment value change was lost'

  $Single = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('DUPLICATE=value')))
  $Duplicate = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('DUPLICATE=value', 'DUPLICATE=value')))
  Assert-True ($Duplicate.Length -eq (2 * $Single.Length)) 'exact environment duplicate multiplicity was lost'
  Assert-True (-not (Test-ExactBytesEqual $Single $Duplicate)) 'exact environment duplicate was collapsed'

  $FramedA = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @("FRAME=line-one`nline=two", 'EQUALS=a=b=c')))
  $FramedB = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('EQUALS=a=b=c', "FRAME=line-one`nline=two")))
  $AmbiguousTextShape = [byte[]] (ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @('FRAME=line-one', 'line=two', 'EQUALS=a=b=c')))
  Assert-True (Test-ExactBytesEqual $FramedA $FramedB) 'framed environment snapshot depends on input order'
  Assert-True (-not (Test-ExactBytesEqual $FramedA $AmbiguousTextShape)) 'environment entry framing is ambiguous'

  $NullOnly = [Array]::CreateInstance([string], 1)
  Assert-Throws { ConvertTo-EnvironmentSnapshotBytes -Entries $NullOnly } 'one-element null environment entry was accepted'
  $NullAmongEntries = [Array]::CreateInstance([string], 2)
  $NullAmongEntries.SetValue('A=1', 0)
  Assert-Throws { ConvertTo-EnvironmentSnapshotBytes -Entries $NullAmongEntries } 'null among environment entries was accepted'
  Assert-Throws {
    ConvertTo-EnvironmentSnapshotBytes -Entries ([string[]] @("NUL=value$([char] 0)tail"))
  } 'embedded NUL environment entry was accepted'

  $Before = Get-ProcessEnvironmentSnapshot
  $ProbeName = 'HUM_FAST_CAPTURE_ENV_SNAPSHOT_' + [Guid]::NewGuid().ToString('N')
  Assert-True ($null -eq [Environment]::GetEnvironmentVariable($ProbeName, [EnvironmentVariableTarget]::Process)) 'environment probe name collision'
  try {
    [Environment]::SetEnvironmentVariable($ProbeName, "value=one`nvalue-two", [EnvironmentVariableTarget]::Process)
    $Changed = Get-ProcessEnvironmentSnapshot
    Assert-True (-not (Test-ExactBytesEqual $Before.Bytes $Changed.Bytes)) 'initialized environment mutation was not detected'
  } finally {
    [Environment]::SetEnvironmentVariable(
      $ProbeName,
      [System.Management.Automation.Language.NullString]::Value,
      [EnvironmentVariableTarget]::Process)
  }
  $Restored = Get-ProcessEnvironmentSnapshot
  Assert-True (Test-ExactBytesEqual $Before.Bytes $Restored.Bytes) 'environment probe was not restored byte-exactly'
}

function Assert-CaptureRejected {
  param([string] $CaptureDirectory, [string] $Message)
  $Rejected = $false
  try { $null = Read-HumCaptureRecord $CaptureDirectory } catch { $Rejected = $true }
  Assert-True $Rejected $Message
}

function Assert-DescendantRecordRejected {
  param([string] $Record, [string] $Name)
  $Rejected = $false
  try { Assert-HumFinalDescendantTree $Record } catch { $Rejected = $true }
  Assert-True $Rejected "descendant record corruption accepted: $Name"
}

function New-DescendantMember {
  param([UInt64] $ProcessId, [UInt64] $Generation, [int] $Primary, [UInt64] $Bytes, [string] $Hash, [string] $PathToken)
  "$ProcessId,$Generation,$Primary,$Bytes,$Hash,$PathToken"
}

function Assert-LaunchedCapture {
  param([object] $Capture)
  try {
    Assert-HumCaptureComplete $Capture
  } catch {
    $Diagnostic = Get-HumCaptureFailureDiagnostic $Capture
    [Console]::Error.WriteLine($Diagnostic)
    [Console]::Error.Flush()
    throw
  }
}

function Assert-LaunchedTimeoutCapture {
  param([object] $Capture, [int] $ExpectedDeadlineSeconds)
  $Capture = Assert-LaunchedCapture $Capture
  $ExpectedDeadlineTicks = [Int64] $ExpectedDeadlineSeconds * [Int64] $Capture.StopwatchFrequency
  Assert-True ($Capture.DeadlineTicks -eq $ExpectedDeadlineTicks) 'launched-timeout absolute deadline'
  Assert-True ($Capture.ProcessCreationAttempted -and $Capture.ProcessCreationSucceeded -and
    $Capture.Launched -and $Capture.PrimaryExitObserved -and
    $Capture.StdoutCompletionObserved -and $Capture.StderrCompletionObserved -and
    $Capture.JobQuiescenceObserved -and $Capture.FinalActiveProcessCount -eq 0) 'launched-timeout lifecycle'
  Assert-True ($Capture.TimedOut -and $Capture.DeadlineDisposition -ceq 'deadline_expired' -and
    $Capture.TerminationRequested -and $Capture.TerminationCount -eq 1 -and
    $Capture.KillAttemptCount -eq 1 -and
    $Capture.TerminationDisposition -ceq 'tree_termination_confirmed' -and
    $Capture.FinalDescendantTree -cmatch '^terminated_quiescent;pretermination=(members|quiescent_race)') 'launched-timeout disposition'
  if ($env:OS -eq 'Windows_NT') {
    Assert-WindowsContainmentLifecycle $Capture 'launched-timeout'
  }
  $Capture
}

function Test-LaunchedTimeoutCaptureDirectory {
  param([string] $CaptureDirectory, [int] $ExpectedDeadlineSeconds)
  try {
    $Capture = Read-HumCaptureRecord $CaptureDirectory
    $null = Assert-LaunchedTimeoutCapture $Capture $ExpectedDeadlineSeconds
    $true
  } catch { $false }
}

function Assert-PrelaunchDiagnostic {
  param([object] $Capture, [string] $Diagnostic)
  $LaunchIdentity = Get-HumFileIdentity $Capture.LaunchErrorPath
  $CaptureIdentity = Get-HumFileIdentity $Capture.CaptureErrorPath
  $Expected = @(
    'hum_capture_failure_v1'
    "case_name=$($Capture.CaseName)"
    "capture_directory=$($Capture.CaptureDirectory)"
    "containment_kind=$($Capture.ContainmentKind)"
    "process_creation_attempted=$($Capture.ProcessCreationAttempted.ToString().ToLowerInvariant())"
    "process_creation_succeeded=$($Capture.ProcessCreationSucceeded.ToString().ToLowerInvariant())"
    "launch_succeeded=$($Capture.Launched.ToString().ToLowerInvariant())"
    "pid=$(if ($null -eq $Capture.Pid) { 'null' } else { $Capture.Pid })"
    "completion_count=$($Capture.CompletionCount)"
    "exit=$(if ($null -eq $Capture.ExitCode) { 'null' } else { $Capture.ExitCode })"
    "deadline_disposition=$($Capture.DeadlineDisposition)"
    "timed_out=$($Capture.TimedOut.ToString().ToLowerInvariant())"
    "termination_disposition=$($Capture.TerminationDisposition)"
    "termination_count=$($Capture.TerminationCount)"
    "launch_error_bytes=$($LaunchIdentity.Bytes)"
    "launch_error_sha256=$($LaunchIdentity.Sha256)"
    "launch_error_base64=$([Convert]::ToBase64String([byte[]] [System.IO.File]::ReadAllBytes($Capture.LaunchErrorPath)))"
    "capture_error_bytes=$($CaptureIdentity.Bytes)"
    "capture_error_sha256=$($CaptureIdentity.Sha256)"
    "capture_error_base64=$([Convert]::ToBase64String([byte[]] [System.IO.File]::ReadAllBytes($Capture.CaptureErrorPath)))"
  ) -join "`n"
  Assert-True ($Diagnostic -ceq $Expected) 'prelaunch diagnostic does not bind exact retained evidence'
  Assert-True (-not $Diagnostic.Contains('stdout_base64=') -and -not $Diagnostic.Contains('stderr_base64=')) 'prelaunch diagnostic exposed child streams'
}

function Test-PrelaunchDiagnostic {
  param([object] $Capture, [string] $Diagnostic)
  try { Assert-PrelaunchDiagnostic $Capture $Diagnostic; $true } catch { $false }
}

function Assert-WindowsContainmentLifecycle {
  param([object] $Capture, [string] $Message)
  Assert-True ($Capture.ContainmentKind -ceq 'windows_job') "$Message containment kind"
  Assert-True ($Capture.JobCreationAttempted -and $Capture.JobCreationSucceeded) "$Message Job creation"
  Assert-True $Capture.JobKillOnCloseConfigured "$Message kill-on-close"
  Assert-True ($Capture.ProcessCreationAttempted -and $Capture.ProcessCreationSucceeded) "$Message process creation"
  Assert-True $Capture.ProcessCreatedSuspended "$Message suspended creation"
  Assert-True ($Capture.JobAssignmentAttempted -and $Capture.JobAssignmentSucceeded) "$Message Job assignment"
  Assert-True ($Capture.ResumeAttempted -and $Capture.ResumeSucceeded) "$Message resume"
  Assert-True ($Capture.PrimaryExitObserved -and $Capture.StdoutCompletionObserved -and $Capture.StderrCompletionObserved) "$Message terminal channels"
  Assert-True ($Capture.JobQuiescenceObserved -and $Capture.FinalActiveProcessCount -eq 0) "$Message Job quiescence"
}

function Get-WitnessPid {
  param([string] $Text, [string] $Name)
  $Match = [regex]::Match($Text, [regex]::Escape($Name) + '=([0-9]+)')
  Assert-True $Match.Success "missing PID witness: $Name"
  [int] $Match.Groups[1].Value
}

function New-ContainmentProjection {
  [pscustomobject]@{
    CreateSuspended = $true
    KillOnClose = $true
    Assignment = $true
    AssignmentOrdinal = 1
    ResumeOrdinal = 2
    JobQuiescence = $true
    StreamsUseRemainingDeadline = $true
    SingleAbsoluteDeadline = $true
    TerminationCount = 1
    DescendantAbsence = $true
    PersistedFacts = $true
  }
}

function Test-ContainmentProjection {
  param([object] $Projection)
  $Projection.CreateSuspended -and $Projection.KillOnClose -and
    $Projection.Assignment -and
    $Projection.AssignmentOrdinal -lt $Projection.ResumeOrdinal -and
    $Projection.JobQuiescence -and $Projection.StreamsUseRemainingDeadline -and
    $Projection.SingleAbsoluteDeadline -and $Projection.TerminationCount -eq 1 -and
    $Projection.DescendantAbsence -and $Projection.PersistedFacts
}

function Assert-ContainmentWeakeningRejected {
  param([string] $Property, [object] $Value)
  $Projection = New-ContainmentProjection
  $Projection.$Property = $Value
  Assert-True (-not (Test-ContainmentProjection $Projection)) "containment weakening accepted: $Property"
}

function New-VctipFacts {
  [pscustomobject][ordered]@{
    Active = 1; Pid = [UInt64]101; Generation = [UInt64]638000000000000001; Primary = 0
    Basename = 'VCTIP.EXE'; OriginalFilename = 'VCTIP.EXE'
    Description = 'Microsoft' + [char]0x00ae + ' VC compiler and tools experience improvement data uploader'
    SignatureStatus = 'Valid'; Publisher = 'Microsoft Corporation'; Certificate = 'A' * 40
    Identity = '0000000000000001:0000000000000002'; Links = [UInt64]1; Bytes = [UInt64]514488
    Sha256 = 'a' * 64; PathToken = 'QzpcVkNUSVAuRVhF'
  }
}
function Assert-VctipFactRejected {
  param([string] $Name, [string] $Property, [object] $Value, [string] $Owner)
  $Facts = New-VctipFacts; $Facts.$Property = $Value
  try { Assert-HumVctipFacts $Facts; throw "VCTIP corruption earned credit: $Name" }
  catch { Assert-True ($_.Exception.Message -ceq $Owner) "VCTIP corruption owner drifted: $Name => $($_.Exception.Message)" }
}
function Assert-VctipFactMatrix {
  $Honest = New-VctipFacts; Assert-HumVctipFacts $Honest
  $Utf8 = [Text.UTF8Encoding]::new($false, $true).GetBytes($Honest.Description)
  Assert-True (($Utf8[9..10] -join ',') -ceq '194,174') 'VCTIP description does not contain UTF-8 C2 AE'
  foreach ($Case in @(
    @{ Name='ascii-description'; Property='Description'; Value='Microsoft VC compiler and tools experience improvement data uploader'; Owner='vctip_auxiliary: signed description mismatch' },
    @{ Name='wrong-description'; Property='Description'; Value=('Microsoft' + [char]0x00ae + ' VC compiler uploader'); Owner='vctip_auxiliary: signed description mismatch' },
    @{ Name='filesystem-name'; Property='Basename'; Value='renamed.exe'; Owner='vctip_auxiliary: filesystem filename mismatch' },
    @{ Name='original-name'; Property='OriginalFilename'; Value='renamed.exe'; Owner='vctip_auxiliary: signed original filename mismatch' },
    @{ Name='publisher'; Property='Publisher'; Value='Contoso'; Owner='vctip_auxiliary: Microsoft signature invalid' },
    @{ Name='signature'; Property='SignatureStatus'; Value='HashMismatch'; Owner='vctip_auxiliary: Microsoft signature invalid' },
    @{ Name='hard-link'; Property='Links'; Value=2; Owner='vctip_auxiliary: executable is hard linked' },
    @{ Name='multiple'; Property='Active'; Value=2; Owner='vctip_auxiliary: expected exactly one non-primary Job member' },
    @{ Name='primary'; Property='Primary'; Value=1; Owner='vctip_auxiliary: expected exactly one non-primary Job member' },
    @{ Name='generation'; Property='Generation'; Value=0; Owner='vctip_auxiliary: identity record malformed' },
    @{ Name='identity'; Property='Identity'; Value='invalid'; Owner='vctip_auxiliary: identity record malformed' },
    @{ Name='length'; Property='Bytes'; Value=0; Owner='vctip_auxiliary: identity record malformed' },
    @{ Name='digest'; Property='Sha256'; Value=('0' * 63); Owner='vctip_auxiliary: identity record malformed' },
    @{ Name='certificate'; Property='Certificate'; Value=('A' * 39); Owner='vctip_auxiliary: identity record malformed' },
    @{ Name='path'; Property='PathToken'; Value='bad/path'; Owner='vctip_auxiliary: identity record malformed' }
  )) { Assert-VctipFactRejected $Case.Name $Case.Property $Case.Value $Case.Owner }
}

function Assert-RunnerSourceContract {
  param([string] $Source)
  foreach ($Literal in @(
    'private const UInt32 CreateSuspendedFlag = 0x00000004;',
    'ProcThreadAttributeHandleList',
    'ConfigureKillOnClose($JobHandle)', 'CreateSuspended(',
    "'process_creation_succeeded.txt'", 'Assign(',
    "'job_assignment_succeeded.txt'", 'Resume(',
    "'resume_succeeded.txt'", 'ActiveProcessCount',
    'Get-HumRemainingMilliseconds', "'termination_count.txt'",
    'Read-HumCaptureRecord $Result.CaptureDirectory',
    "'case_name.txt'", 'Get-HumCaptureFailureDiagnostic $Retained',
    'launch_error_base64=', 'capture_error_base64=',
    '[regex]::Split($Hex, ''0D-0A|0A|0D'')',
    'terminal stdout line is not ASCII',
    'if ($State.TerminationCount -ne 0) { throw ''termination requested more than once'' }',
    'if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent)',
    '-not $JobQuiescent -or $FinalActive -ne 0',
    '$Remaining = Get-HumRemainingMilliseconds $Timer $DeadlineTicks',
    '[HumFastJobNative]::ProcessIds($JobHandle)', 'StartTime.ToUniversalTime().Ticks',
    'Get-HumFileIdentity $Item.FullName', '$Second = @([HumFastJobNative]::ProcessIds($JobHandle) | Sort-Object)',
    '$Active -ne $Second.Count', 'pretermination=quiescent_race',
    'descendant_evidence: active Job membership changed during identity acquisition',
    '"pretermination_pending;" + $State.Pretermination',
    'completed_after_authenticated_vctip_termination',
    "`$DeadlineDisposition -ne 'completed_before_deadline' -and`n        `$DeadlineDisposition -ne 'completed_after_authenticated_vctip_termination'",
    'terminated_quiescent;pretermination=authenticated_vctip_auxiliary',
    'Get-AuthenticodeSignature -LiteralPath $Path',
    "`$ExpectedDescription = 'Microsoft' + [char]0x00ae + ' VC compiler and tools experience improvement data uploader'",
    '$Before = Get-HumVctipFacts', '$After = Get-HumVctipFacts',
    "if (`$Facts.Basename -cne 'VCTIP.EXE')", "if (`$Facts.OriginalFilename -cne 'VCTIP.EXE')",
    'if ($Facts.Description -cne $ExpectedDescription)',
    'Set-HumDurableText (Join-Path $CaptureDirectory ''termination_result.txt'') $Record',
    '[HumFastJobNative]::KillJob($JobHandle)'
    'function Invoke-HumContainedRustNativeCapture'
    'function Invoke-HumContainedExactRustTest'
    'Invoke-HumBinaryCapture $Cargo $Arguments'
    'Assert-HumCaptureComplete $Result'
    'Remove-HumCaptureAfterAuthentication $Capture'
  )) { Assert-True ($Source.Contains($Literal)) "runner source contract missing: $Literal" }
  Assert-True ([regex]::Matches($Source, 'pretermination=quiescent_race').Count -eq 4) 'descendant race ownership count'
  $Create = $Source.IndexOf('$NativeProcess = [HumFastJobNative]::CreateSuspended', [StringComparison]::Ordinal)
  $PersistCreate = $Source.IndexOf("'process_creation_succeeded.txt'", $Create, [StringComparison]::Ordinal)
  $Assign = $Source.IndexOf('[HumFastJobNative]::Assign(', $PersistCreate, [StringComparison]::Ordinal)
  $PersistAssign = $Source.IndexOf("'job_assignment_succeeded.txt'", $Assign, [StringComparison]::Ordinal)
  $Resume = $Source.IndexOf('[HumFastJobNative]::Resume(', $PersistAssign, [StringComparison]::Ordinal)
  $PersistResume = $Source.IndexOf("'resume_succeeded.txt'", $Resume, [StringComparison]::Ordinal)
  Assert-True ($Create -ge 0 -and $Create -lt $PersistCreate -and $PersistCreate -lt $Assign -and
    $Assign -lt $PersistAssign -and $PersistAssign -lt $Resume -and $Resume -lt $PersistResume) 'create/assign/resume source order'
  Assert-True (-not $Source.Contains('Task]::WaitAll')) 'unbounded Task.WaitAll returned'
  Assert-True (-not $Source.Contains('.WaitForExit()')) 'unbounded WaitForExit returned'
  Assert-True (-not $Source.Contains('$StdoutTask.Wait()')) 'unbounded stream wait returned'
  Assert-True (-not $Source.Contains('$Timer.ElapsedTicks + $DeadlineTicks')) 'execution deadline can restart'
  $VctipPersist = $Source.IndexOf("Set-HumDurableText (Join-Path `$CaptureDirectory 'termination_result.txt') `$Record", [StringComparison]::Ordinal)
  $VctipRecheck = $Source.IndexOf('$After = Get-HumVctipFacts', $VctipPersist, [StringComparison]::Ordinal)
  $VctipKill = $Source.IndexOf('[HumFastJobNative]::KillJob($JobHandle)', $VctipRecheck, [StringComparison]::Ordinal)
  Assert-True ($VctipPersist -ge 0 -and $VctipPersist -lt $VctipRecheck -and $VctipRecheck -lt $VctipKill) 'VCTIP evidence/recheck/termination order'
}

function Test-RunnerSourceContract {
  param([string] $Source)
  try { Assert-RunnerSourceContract $Source; $true } catch { $false }
}

function Assert-SourceWeakeningRejected {
  param([string] $Source, [string] $Old, [string] $New, [string] $Name)
  $Mutated = $Source.Replace($Old, $New)
  Assert-True ($Mutated -cne $Source) "source mutation did not initialize: $Name"
  $Tokens = $null
  $Errors = $null
  [Management.Automation.Language.Parser]::ParseInput($Mutated, [ref] $Tokens, [ref] $Errors) | Out-Null
  Assert-True ($Errors.Count -eq 0) "source mutation failed to parse: $Name"
  Assert-True (-not (Test-RunnerSourceContract $Mutated)) "source weakening accepted: $Name"
}

function Copy-CaptureForMutation {
  param([string] $Source, [string] $Destination)
  Copy-Item -LiteralPath $Source -Destination $Destination -Recurse
  $Destination
}

function Rewrite-Manifest {
  param([string] $CaptureDirectory)
  Remove-Item -LiteralPath (Join-Path $CaptureDirectory 'manifest.txt')
  Write-HumCaptureManifest $CaptureDirectory
}

function Get-GitConfigurationIdentity {
  $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
  $Paths = @(
    (Join-Path $RepoRoot '.git\config'),
    (Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)) '.gitconfig'),
    (Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::UserProfile)) '.config\git\config')
  )
  foreach ($Variable in @('GIT_CONFIG_GLOBAL', 'GIT_CONFIG_SYSTEM')) {
    $Value = [Environment]::GetEnvironmentVariable($Variable)
    if (-not [string]::IsNullOrEmpty($Value)) { $Paths += $Value }
  }
  if ($env:OS -eq 'Windows_NT') { $Paths += (Join-Path $env:ProgramFiles 'Git\etc\gitconfig') }
  else { $Paths += '/etc/gitconfig' }
  @($Paths | Sort-Object -Unique | ForEach-Object {
    $Resolved = [System.IO.Path]::GetFullPath($_)
    if (Test-Path -LiteralPath $Resolved -PathType Leaf) {
      "$Resolved=$((Get-HumFileIdentity $Resolved).Sha256)"
    } else {
      "$Resolved=<absent>"
    }
  }) -join "`n"
}

if ($EnvironmentSnapshotOnly) {
  Assert-EnvironmentSnapshotContract
  Write-Output "Environment snapshot tests passed for $ShellContract."
  exit 0
}

if ($ScratchRoot -eq '') {
  $ScratchRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("hum-fast-capture-test-" + [Guid]::NewGuid().ToString('N'))
}
Assert-True (-not (Test-Path -LiteralPath $ScratchRoot)) 'scratch root must be absent before test'
Assert-EnvironmentSnapshotContract
$BeforeEnvironment = Get-ProcessEnvironmentSnapshot
$BeforeDirectory = (Get-Location).Path
$BeforeConfig = Get-GitConfigurationIdentity
$RunnerSource = [System.IO.File]::ReadAllText((Join-Path $PSScriptRoot 'run_fast_evidence.ps1'))
Assert-RunnerSourceContract $RunnerSource
Assert-VctipFactMatrix
$DurableTree = $RunnerSource.IndexOf('Set-HumDurableText (Join-Path $CaptureDirectory ''final_descendant_tree.txt'') ("pretermination_pending;" + $State.Pretermination)', [StringComparison]::Ordinal)
$TerminateTree = $RunnerSource.IndexOf('[HumFastJobNative]::KillJob($JobHandle)', $DurableTree, [StringComparison]::Ordinal)
Assert-True ($DurableTree -ge 0 -and $DurableTree -lt $TerminateTree) 'descendant identities were not durable before termination'
Assert-SourceWeakeningRejected $RunnerSource `
  'private const UInt32 CreateSuspendedFlag = 0x00000004;' `
  'private const UInt32 CreateSuspendedFlag = 0x00000000;' 'CREATE_SUSPENDED removal'
Assert-SourceWeakeningRejected $RunnerSource `
  '[HumFastJobNative]::Assign($JobHandle, $NativeProcess.ProcessHandle)' `
  '[HumFastJobNative]::Resume($NativeProcess.ThreadHandle)' 'resume before assignment'
Assert-SourceWeakeningRejected $RunnerSource `
  '[HumFastJobNative]::Assign($JobHandle, $NativeProcess.ProcessHandle)' `
  '$null = $JobHandle' 'Job assignment omission'
Assert-SourceWeakeningRejected $RunnerSource `
  '[HumFastJobNative]::ConfigureKillOnClose($JobHandle)' `
  '$null = $JobHandle' 'kill-on-close omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent)' `
  'if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $true)' 'Job quiescence omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'Start-Sleep -Milliseconds $Remaining' '$StdoutTask.Wait()' 'unbounded stream wait'
Assert-SourceWeakeningRejected $RunnerSource `
  '$Remaining = Get-HumRemainingMilliseconds $Timer $DeadlineTicks' `
  '$Remaining = Get-HumRemainingMilliseconds $Timer ($Timer.ElapsedTicks + $DeadlineTicks)' 'deadline restart'
Assert-SourceWeakeningRejected $RunnerSource `
  'if ($State.TerminationCount -ne 0) { throw ''termination requested more than once'' }' `
  '$null = $State.TerminationCount' 'double termination'
Assert-SourceWeakeningRejected $RunnerSource `
  "`$DeadlineDisposition -ne 'completed_before_deadline' -and`n        `$DeadlineDisposition -ne 'completed_after_authenticated_vctip_termination'" `
  "`$DeadlineDisposition -ne 'completed_before_deadline'" 'VCTIP success fallthrough'
Assert-SourceWeakeningRejected $RunnerSource `
  "`$DeadlineDisposition -ne 'completed_after_authenticated_vctip_termination'" `
  "`$DeadlineDisposition -ne 'completed_after_authenticated_vctip_cleanup'" 'VCTIP success disposition substitution'
Assert-SourceWeakeningRejected $RunnerSource `
  "`$DeadlineDisposition -ne 'completed_after_authenticated_vctip_termination'" `
  "`$DeadlineDisposition -ne 'deadline_expired'" 'deadline exclusion broadening'
Assert-SourceWeakeningRejected $RunnerSource `
  '-not $JobQuiescent -or $FinalActive -ne 0' '$false' 'descendant absence omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'Read-HumCaptureRecord $Result.CaptureDirectory' '$Result' 'in-memory fact trust'
Assert-SourceWeakeningRejected $RunnerSource `
  'StartTime.ToUniversalTime().Ticks' '0' 'descendant generation omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'Get-HumFileIdentity $Item.FullName' '$null' 'descendant image omission'
Assert-SourceWeakeningRejected $RunnerSource `
  '$Second = @([HumFastJobNative]::ProcessIds($JobHandle) | Sort-Object)' '$Second = $First' 'descendant requery omission'
Assert-SourceWeakeningRejected $RunnerSource `
  '$Active -ne $Second.Count' '$false' 'descendant count agreement omission'
Assert-SourceWeakeningRejected $RunnerSource `
  "return 'pretermination=quiescent_race'" "return 'pretermination=members'" 'descendant race disposition substitution'
Assert-SourceWeakeningRejected $RunnerSource `
  "if (`$Facts.Basename -cne 'VCTIP.EXE')" "if (`$false)" 'VCTIP filesystem filename omission'
Assert-SourceWeakeningRejected $RunnerSource `
  "if (`$Facts.OriginalFilename -cne 'VCTIP.EXE')" "if (`$false)" 'VCTIP signed original filename omission'
Assert-SourceWeakeningRejected $RunnerSource `
  'if ($Facts.Description -cne $ExpectedDescription)' 'if ($false)' 'VCTIP signed description omission'
Assert-SourceWeakeningRejected $RunnerSource `
  '$After = Get-HumVctipFacts $JobHandle ([UInt64]$NativeProcess.ProcessId) $ToolchainRoot' '$After = $Before' 'VCTIP immediate reauthentication omission'
Assert-True (Test-ContainmentProjection (New-ContainmentProjection)) 'honest containment projection rejected'
Assert-ContainmentWeakeningRejected 'CreateSuspended' $false
Assert-ContainmentWeakeningRejected 'KillOnClose' $false
Assert-ContainmentWeakeningRejected 'Assignment' $false
Assert-ContainmentWeakeningRejected 'AssignmentOrdinal' 3
Assert-ContainmentWeakeningRejected 'JobQuiescence' $false
Assert-ContainmentWeakeningRejected 'StreamsUseRemainingDeadline' $false
Assert-ContainmentWeakeningRejected 'SingleAbsoluteDeadline' $false
Assert-ContainmentWeakeningRejected 'TerminationCount' 2
Assert-ContainmentWeakeningRejected 'DescendantAbsence' $false
Assert-ContainmentWeakeningRejected 'PersistedFacts' $false
$MemberA = New-DescendantMember 101 638000000000000001 1 4096 ('a' * 64) 'QzpcYmluXGFwcC5leGU'
$MemberB = New-DescendantMember 202 638000000000000002 0 8192 ('b' * 64) 'QzpcYmluXGNoaWxkLmV4ZQ'
$One = "terminated_quiescent;pretermination=members;active=1;member=$MemberA"
$Many = "terminated_quiescent;pretermination=members;active=2;member=$MemberA|$MemberB"
foreach ($Record in @('quiescent', 'terminated_quiescent;pretermination=quiescent_race', $One, $Many)) { Assert-HumFinalDescendantTree $Record }
Assert-DescendantRecordRejected "terminated_quiescent;pretermination=members;active=2;member=$MemberB|$MemberA" 'ordering'
Assert-DescendantRecordRejected "terminated_quiescent;pretermination=members;active=2;member=$MemberA|$MemberA" 'duplicate'
Assert-DescendantRecordRejected "terminated_quiescent;pretermination=members;active=3;member=$MemberA|$MemberB" 'count'
Assert-DescendantRecordRejected "terminated_quiescent;pretermination=members;active=1;member=101,0,1,4096,$('a' * 64),QzpcYmluXGFwcC5leGU" 'generation'
Assert-DescendantRecordRejected "terminated_quiescent;pretermination=members;active=1;member=101,638000000000000001,1,4096,$('a' * 64)," 'image'
Assert-DescendantRecordRejected "terminated_quiescent;pretermination=members;active=1;member=101,638000000000000001,1,4096,$('a' * 63),QzpcYmluXGFwcC5leGU" 'image-hash'
Assert-DescendantRecordRejected ($Many.Substring(0, $Many.Length - 1)) 'truncation'
$CreatedPids = New-Object System.Collections.Generic.List[int]
$ValidCaptures = New-Object System.Collections.Generic.List[object]

try {
  if($ShellContract -eq 'powershell'){$Shell="$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"}else{$ShellApplications=@(Get-Command pwsh -CommandType Application -All -ErrorAction Stop);Assert-True ($ShellApplications.Count -eq 1) 'capture contract requires exactly one PowerShell 7 application';$Shell=[IO.Path]::GetFullPath([string]$ShellApplications[0].Source)}
  Assert-True ([IO.File]::Exists($Shell)) 'selected PowerShell executable is missing'
  Assert-True (-not ([IO.File]::GetAttributes($Shell) -band [IO.FileAttributes]::ReparsePoint)) 'selected PowerShell executable is a reparse point'
  $Self = $MyInvocation.MyCommand.Path
  $BaseArguments = @('-NoLogo', '-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', $Self)

  $Preflight = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'preflight')) $BeforeDirectory (Join-Path $ScratchRoot 'preflight') 30 -CaseName 'preflight'
  $Preflight = Assert-LaunchedCapture $Preflight
  Assert-True ($Preflight.ExitCode -eq 0 -and $Preflight.StderrBytes -eq 0) 'shell preflight failed'
  $PreflightLines = @([Text.Encoding]::UTF8.GetString((Read-Bytes $Preflight.StdoutPath)) -split "\r?\n" | Where-Object { $_ -ne '' })
  Assert-True ($PreflightLines.Count -eq 2) 'shell preflight output shape'
  Assert-True ([System.IO.Path]::GetFullPath($PreflightLines[0]) -eq [System.IO.Path]::GetFullPath($Shell)) 'fresh child resolved a different shell'

  $Success = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'success')) $BeforeDirectory (Join-Path $ScratchRoot 'success') 30 -CaseName 'success'
  $Success = Assert-LaunchedCapture $Success
  $Success = Read-HumCaptureRecord $Success.CaptureDirectory
  Assert-True ($Success.ExitCode -eq 0 -and $Success.CompletionCount -eq 1) 'success terminal metadata'
  Assert-Bytes (Read-Bytes $Success.StdoutPath) ([Text.Encoding]::ASCII.GetBytes("CAPTURE_STDOUT`n$SuccessMarker`n")) 'success stdout'
  Assert-Bytes (Read-Bytes $Success.StderrPath) ([Text.Encoding]::ASCII.GetBytes("CAPTURE_STDERR`n")) 'success stderr'
  Assert-True ($Success.SuccessMarkerCount -eq 1 -and $Success.TerminalStdoutLine -ceq $SuccessMarker) 'success terminal marker'

  $Exit23 = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'exit23')) $BeforeDirectory (Join-Path $ScratchRoot 'exit23') 30 -CaseName 'exit23'
  $Exit23 = Assert-LaunchedCapture $Exit23
  Assert-True ($Exit23.ExitCode -eq 23) 'exit 23 must be retained'
  Assert-Bytes (Read-Bytes $Exit23.StdoutPath) ([Text.Encoding]::ASCII.GetBytes("EXIT23_STDOUT`n")) 'exit23 stdout'
  Assert-Bytes (Read-Bytes $Exit23.StderrPath) ([Text.Encoding]::ASCII.GetBytes("EXIT23_STDERR`n")) 'exit23 stderr'

  $Empty = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'empty')) $BeforeDirectory (Join-Path $ScratchRoot 'empty') 30 -CaseName 'empty'
  $Empty = Assert-LaunchedCapture $Empty
  Assert-True ($Empty.Launched -and $Empty.ExitCode -eq 0 -and $Empty.StdoutBytes -eq 0 -and $Empty.StderrBytes -eq 0) 'launched empty-stream child'

  $Missing = Invoke-HumBinaryCapture (Join-Path $ScratchRoot 'missing-executable.exe') @('--unused') $BeforeDirectory (Join-Path $ScratchRoot 'prelaunch') 30 -CaseName 'known-missing-executable'
  $Missing = Read-HumCaptureRecord $Missing.CaptureDirectory
  Assert-True (-not $Missing.Launched -and $null -eq $Missing.ExitCode -and $Missing.CompletionCount -eq 0) 'prelaunch failure state'
  Assert-True ($Missing.StdoutBytes -eq 0 -and $Missing.StderrBytes -eq 0) 'prelaunch durable empty streams'
  Assert-True ((Get-HumFileIdentity $Missing.LaunchErrorPath).Bytes -gt 0) 'prelaunch error must be retained'
  $MissingDiagnostic = Get-HumCaptureFailureDiagnostic $Missing
  Assert-PrelaunchDiagnostic $Missing $MissingDiagnostic
  $MissingDiagnosticMutation = $MissingDiagnostic -replace '(?m)^launch_error_base64=.*$', 'launch_error_base64='
  Assert-True ($MissingDiagnosticMutation -cne $MissingDiagnostic) 'prelaunch diagnostic mutation did not initialize'
  Assert-True (-not (Test-PrelaunchDiagnostic $Missing $MissingDiagnosticMutation)) 'launch-error diagnostic omission accepted'

  $Interleaved = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'interleaved')) $BeforeDirectory (Join-Path $ScratchRoot 'interleaved') 60 -CaseName 'interleaved'
  $Interleaved = Assert-LaunchedCapture $Interleaved
  $ExpectedOut = New-Object System.Text.StringBuilder
  $ExpectedErr = New-Object System.Text.StringBuilder
  for ($Index = 0; $Index -lt 2048; $Index++) {
    $null = $ExpectedOut.Append(("O{0:d5}:{1}`n" -f $Index, ('o' * 48)))
    $null = $ExpectedErr.Append(("E{0:d5}:{1}`n" -f $Index, ('e' * 48)))
  }
  Assert-Bytes (Read-Bytes $Interleaved.StdoutPath) ([Text.Encoding]::ASCII.GetBytes($ExpectedOut.ToString())) 'interleaved stdout order'
  Assert-Bytes (Read-Bytes $Interleaved.StderrPath) ([Text.Encoding]::ASCII.GetBytes($ExpectedErr.ToString())) 'interleaved stderr order'

  $Unicode = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'unicode')) $BeforeDirectory (Join-Path $ScratchRoot 'unicode') 30 -CaseName 'unicode'
  $Unicode = Assert-LaunchedCapture $Unicode
  Assert-Bytes (Read-Bytes $Unicode.StdoutPath) ([byte[]] (0x75, 0x74, 0x66, 0x38, 0x3d, 0xe2, 0x98, 0x83, 0x0d, 0x0a, 0x6c, 0x66, 0x0a)) 'Unicode stdout'
  Assert-Bytes (Read-Bytes $Unicode.StderrPath) ([byte[]] (0x65, 0x72, 0x72, 0x3d, 0xf0, 0x9f, 0x8c, 0x8a, 0x0a, 0x63, 0x72, 0x0d)) 'Unicode stderr'

  $Early = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'early-marker')) $BeforeDirectory (Join-Path $ScratchRoot 'early-marker') 30 -CaseName 'early-marker')
  Assert-True ($Early.SuccessMarkerCount -eq 1 -and $Early.TerminalStdoutLine -ceq 'later assertion') 'early marker must not be terminal success'
  $Duplicate = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'duplicate-marker')) $BeforeDirectory (Join-Path $ScratchRoot 'duplicate-marker') 30 -CaseName 'duplicate-marker')
  Assert-True ($Duplicate.SuccessMarkerCount -eq 2) 'duplicate marker must be visible'
  $Nonzero = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'nonzero-marker')) $BeforeDirectory (Join-Path $ScratchRoot 'nonzero-marker') 30 -CaseName 'nonzero-marker')
  Assert-True ($Nonzero.ExitCode -eq 9 -and $Nonzero.SuccessMarkerCount -eq 1 -and $Nonzero.TerminalStdoutLine -ceq $SuccessMarker) 'nonzero marker case'

  $LaunchedTimeoutDeadlineSeconds = 10
  $Timeout = Assert-LaunchedTimeoutCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'timeout')) $BeforeDirectory (Join-Path $ScratchRoot 'timeout') $LaunchedTimeoutDeadlineSeconds -CaseName 'timeout') $LaunchedTimeoutDeadlineSeconds
  Assert-True ($Timeout.FinalDescendantTree -cmatch '^terminated_quiescent;pretermination=members;active=([2-9]|[1-9][0-9]+);') 'multiple survivors were not retained deterministically'
  Assert-HumFinalDescendantTree $Timeout.FinalDescendantTree
  $TimeoutOut = [Text.Encoding]::UTF8.GetString((Read-Bytes $Timeout.StdoutPath))
  $TimeoutErr = [Text.Encoding]::UTF8.GetString((Read-Bytes $Timeout.StderrPath))
  Assert-True ($TimeoutOut -match 'parent_alive=([0-9]+)' -and $TimeoutOut -match 'descendant_pid=([0-9]+)' -and $TimeoutOut.Contains('parent_partial_stdout')) 'timeout PID and stdout witnesses'
  $ParentPid = [int] ([regex]::Match($TimeoutOut, 'parent_alive=([0-9]+)').Groups[1].Value)
  $DescendantPid = [int] ([regex]::Match($TimeoutOut, 'descendant_pid=([0-9]+)').Groups[1].Value)
  Assert-True ($TimeoutErr.Contains('parent_partial_stderr')) 'timeout stderr witness'
  Assert-True ($null -eq (Get-Process -Id $ParentPid -ErrorAction SilentlyContinue)) 'timed-out parent survived'
  Assert-True ($null -eq (Get-Process -Id $DescendantPid -ErrorAction SilentlyContinue)) 'timed-out descendant survived'

  $WindowsCaptures = @()
  $SetupCaptures = @()
  if ($env:OS -eq 'Windows_NT') {
    $ContainedDescendantDeadlineSeconds = 4
    $Inherited = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'inherited-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'inherited-parent') $ContainedDescendantDeadlineSeconds 2 -CaseName 'inherited-parent')
    Assert-True ($Inherited.FinalDescendantTree -cmatch '^terminated_quiescent;pretermination=members;active=[1-9][0-9]*;') 'controlled survivor set was not retained'
    Assert-HumFinalDescendantTree $Inherited.FinalDescendantTree
    Assert-WindowsContainmentLifecycle $Inherited 'inherited-pipe'
    Assert-True ($Inherited.DeadlineTicks -eq [Int64] $ContainedDescendantDeadlineSeconds * $Inherited.StopwatchFrequency) 'inherited-pipe absolute deadline'
    Assert-True ($Inherited.TimedOut -and $Inherited.DeadlineDisposition -ceq 'deadline_expired' -and
      $Inherited.TerminationRequested -and $Inherited.TerminationCount -eq 1 -and
      $Inherited.KillAttemptCount -eq 1 -and $Inherited.FinalDescendantTree -cmatch '^terminated_quiescent;pretermination=(members|quiescent_race)') 'inherited-pipe timeout disposition'
    $InheritedText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Inherited.StdoutPath))
    $InheritedDescendant = Get-WitnessPid $InheritedText 'inherited_descendant_pid'
    Assert-True ($Inherited.FinalDescendantTree -cmatch "(?:member=|\|)$InheritedDescendant,") 'controlled survivor identity was not retained'
    $InheritedExpected = [Text.Encoding]::ASCII.GetBytes("inherited_parent_pid=$($Inherited.Pid)`ninherited_descendant_pid=$InheritedDescendant`ninherited_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Inherited.StdoutPath) $InheritedExpected 'inherited-pipe stdout'
    Assert-Bytes (Read-Bytes $Inherited.StderrPath) ([Text.Encoding]::ASCII.GetBytes("inherited_parent_stderr`n")) 'inherited-pipe stderr'
    Assert-True ($null -eq (Get-Process -Id $InheritedDescendant -ErrorAction SilentlyContinue)) 'inherited-pipe descendant survived'

    $InheritedNatural = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'inherited-short-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'inherited-parent-natural') 20 2 -CaseName 'inherited-parent-natural')
    Assert-WindowsContainmentLifecycle $InheritedNatural 'inherited natural completion'
    Assert-True ($InheritedNatural.DeadlineTicks -eq [Int64] 20 * $InheritedNatural.StopwatchFrequency) 'inherited natural-completion deadline'
    Assert-True ($InheritedNatural.ExitCode -eq 0 -and -not $InheritedNatural.TimedOut -and
      -not $InheritedNatural.TerminationRequested -and $InheritedNatural.TerminationCount -eq 0 -and
      $InheritedNatural.KillAttemptCount -eq 0 -and $InheritedNatural.DeadlineDisposition -ceq 'completed_before_deadline' -and
      $InheritedNatural.FinalDescendantTree -ceq 'quiescent') 'inherited natural-completion disposition'
    $InheritedNaturalText = [Text.Encoding]::UTF8.GetString((Read-Bytes $InheritedNatural.StdoutPath))
    $InheritedNaturalDescendant = Get-WitnessPid $InheritedNaturalText 'inherited_descendant_pid'
    $InheritedNaturalExpected = [Text.Encoding]::ASCII.GetBytes("inherited_parent_pid=$($InheritedNatural.Pid)`ninherited_descendant_pid=$InheritedNaturalDescendant`ninherited_parent_stdout`n")
    Assert-Bytes (Read-Bytes $InheritedNatural.StdoutPath) $InheritedNaturalExpected 'inherited natural-completion stdout'
    Assert-Bytes (Read-Bytes $InheritedNatural.StderrPath) ([Text.Encoding]::ASCII.GetBytes("inherited_parent_stderr`n")) 'inherited natural-completion stderr'
    Assert-True ($null -eq (Get-Process -Id $InheritedNaturalDescendant -ErrorAction SilentlyContinue)) 'naturally completed inherited descendant survived'

    $Redirected = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'redirected-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'redirected-parent') $ContainedDescendantDeadlineSeconds 2 -CaseName 'redirected-parent')
    Assert-WindowsContainmentLifecycle $Redirected 'redirected-descendant'
    Assert-True ($Redirected.DeadlineTicks -eq [Int64] $ContainedDescendantDeadlineSeconds * $Redirected.StopwatchFrequency) 'redirected-descendant absolute deadline'
    Assert-True ($Redirected.TimedOut -and $Redirected.TerminationCount -eq 1) 'redirected descendant escaped Job deadline'
    $RedirectedText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Redirected.StdoutPath))
    $RedirectedDescendant = Get-WitnessPid $RedirectedText 'redirected_descendant_pid'
    $RedirectedExpected = [Text.Encoding]::ASCII.GetBytes("redirected_parent_pid=$($Redirected.Pid)`nredirected_descendant_pid=$RedirectedDescendant`nredirected_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Redirected.StdoutPath) $RedirectedExpected 'redirected descendant stdout'
    Assert-Bytes (Read-Bytes $Redirected.StderrPath) ([Text.Encoding]::ASCII.GetBytes("redirected_parent_stderr`n")) 'redirected descendant stderr'
    Assert-True ($null -eq (Get-Process -Id $RedirectedDescendant -ErrorAction SilentlyContinue)) 'redirected descendant survived'

    $Earliest = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'earliest-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'earliest-parent') $ContainedDescendantDeadlineSeconds 2 -CaseName 'earliest-parent')
    Assert-WindowsContainmentLifecycle $Earliest 'earliest-descendant'
    Assert-True ($Earliest.DeadlineTicks -eq [Int64] $ContainedDescendantDeadlineSeconds * $Earliest.StopwatchFrequency) 'earliest-descendant absolute deadline'
    Assert-True ($Earliest.TimedOut -and $Earliest.TerminationCount -eq 1) 'earliest descendant did not remain contained'
    $EarliestText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Earliest.StdoutPath))
    $EarliestDescendant = Get-WitnessPid $EarliestText 'earliest_descendant_pid'
    $EarliestExpected = [Text.Encoding]::ASCII.GetBytes("earliest_parent_pid=$($Earliest.Pid)`nearliest_descendant_pid=$EarliestDescendant`nearliest_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Earliest.StdoutPath) $EarliestExpected 'earliest descendant stdout'
    Assert-Bytes (Read-Bytes $Earliest.StderrPath) ([Text.Encoding]::ASCII.GetBytes("earliest_parent_stderr`n")) 'earliest descendant stderr'
    Assert-True ($null -eq (Get-Process -Id $EarliestDescendant -ErrorAction SilentlyContinue)) 'earliest descendant survived'

    $OrdinaryCompletionDeadlineSeconds = 20
    $Quiescent = Assert-LaunchedCapture (Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'quiescent-parent')) $BeforeDirectory (Join-Path $ScratchRoot 'quiescent-parent') $OrdinaryCompletionDeadlineSeconds 2 -CaseName 'quiescent-parent')
    Assert-WindowsContainmentLifecycle $Quiescent 'ordinary-quiescent'
    Assert-True ($Quiescent.DeadlineTicks -eq [Int64] $OrdinaryCompletionDeadlineSeconds * $Quiescent.StopwatchFrequency) 'ordinary quiescence absolute deadline'
    Assert-True ($Quiescent.ExitCode -eq 0 -and -not $Quiescent.TimedOut -and
      -not $Quiescent.TerminationRequested -and $Quiescent.TerminationCount -eq 0 -and
      $Quiescent.KillAttemptCount -eq 0 -and $Quiescent.DeadlineDisposition -ceq 'completed_before_deadline' -and
      $Quiescent.FinalDescendantTree -ceq 'quiescent') 'ordinary quiescence disposition'
    $QuiescentText = [Text.Encoding]::UTF8.GetString((Read-Bytes $Quiescent.StdoutPath))
    $QuiescentDescendant = Get-WitnessPid $QuiescentText 'quiescent_descendant_pid'
    $QuiescentExpected = [Text.Encoding]::ASCII.GetBytes("quiescent_parent_pid=$($Quiescent.Pid)`nquiescent_descendant_pid=$QuiescentDescendant`nquiescent_parent_stdout`n")
    Assert-Bytes (Read-Bytes $Quiescent.StdoutPath) $QuiescentExpected 'ordinary quiescence stdout'
    Assert-Bytes (Read-Bytes $Quiescent.StderrPath) ([Text.Encoding]::ASCII.GetBytes("quiescent_parent_stderr`n")) 'ordinary quiescence stderr'
    Assert-True ($null -eq (Get-Process -Id $QuiescentDescendant -ErrorAction SilentlyContinue)) 'ordinary short descendant survived'

    foreach ($FailureName in @('job_create', 'job_configure', 'process_create', 'assignment', 'resume')) {
      $Failure = Invoke-HumBinaryCapture $Shell ($BaseArguments + @('-SyntheticChild', 'empty')) $BeforeDirectory (Join-Path $ScratchRoot "setup-$FailureName") 10 2 $FailureName -CaseName "setup-$FailureName"
      $Failure = Read-HumCaptureRecord $Failure.CaptureDirectory
      Assert-True ($Failure.CaptureErrorBytes -gt 0 -or -not $Failure.ProcessCreationSucceeded) "setup failure not retained: $FailureName"
      if ($FailureName -in @('job_create', 'job_configure', 'process_create')) {
        Assert-True (-not $Failure.Launched -and $null -eq $Failure.Pid) "prelaunch setup failure launched: $FailureName"
      } else {
        Assert-True ($Failure.Launched -and $Failure.ProcessCreatedSuspended -and $Failure.TerminationCount -eq 1) "post-create setup failure not contained: $FailureName"
        Assert-True ($null -eq (Get-Process -Id $Failure.Pid -ErrorAction SilentlyContinue)) "setup-failure child survived: $FailureName"
      }
      $SetupCaptures += $Failure
    }
    Assert-True (-not $SetupCaptures[3].JobAssignmentSucceeded -and -not $SetupCaptures[3].ResumeAttempted) 'assignment-failure lifecycle'
    Assert-True ($SetupCaptures[4].JobAssignmentSucceeded -and $SetupCaptures[4].ResumeAttempted -and -not $SetupCaptures[4].ResumeSucceeded) 'resume-failure lifecycle'
    $VctipProjectedPath = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'vctip-projected')
    $VctipRecord = 'authenticated_vctip_auxiliary;pid=101;generation=638000000000000001;identity=0000000000000001:0000000000000002;bytes=514488;sha256=' + ('a' * 64) + ';certificate=' + ('A' * 40) + ';path=QzpcVkNUSVAuRVhF'
    foreach ($Fact in @{
      'deadline_disposition.txt' = 'completed_after_authenticated_vctip_termination'
      'termination_requested.txt' = '1'; 'termination_count.txt' = '1'; 'kill_attempt_count.txt' = '1'
      'termination_disposition.txt' = 'authenticated_vctip_termination_confirmed'
      'termination_result.txt' = $VctipRecord
      'final_descendant_tree.txt' = 'terminated_quiescent;pretermination=authenticated_vctip_auxiliary'
    }.GetEnumerator()) { Set-HumDurableText (Join-Path $VctipProjectedPath $Fact.Key) $Fact.Value }
    Rewrite-Manifest $VctipProjectedPath
    $VctipProjected = Read-HumCaptureRecord $VctipProjectedPath
    Assert-True ($VctipProjected.DeadlineDisposition -ceq 'completed_after_authenticated_vctip_termination' -and
      $VctipProjected.TerminationCount -eq 1 -and $VctipProjected.FinalActiveProcessCount -eq 0) 'honest VCTIP terminal projection rejected'
    foreach ($Mutation in @(
      @{ Name = 'truncated'; Value = $VctipRecord.Substring(0, $VctipRecord.LastIndexOf(';path=') + 6) },
      @{ Name = 'reordered'; Value = $VctipRecord.Replace(';pid=101;generation=', ';generation=').Replace(';identity=', ';pid=101;identity=') },
      @{ Name = 'malformed'; Value = $VctipRecord + ';extra=true' },
      @{ Name = 'contradictory'; Value = $VctipRecord.Replace('pid=101', 'pid=0') }
    )) {
      $Copy = Copy-CaptureForMutation $VctipProjectedPath (Join-Path $ScratchRoot "vctip-$($Mutation.Name)")
      Set-HumDurableText (Join-Path $Copy 'termination_result.txt') $Mutation.Value; Rewrite-Manifest $Copy
      Assert-CaptureRejected $Copy "VCTIP $($Mutation.Name) record accepted"
    }
    $DuplicateVctip = Copy-CaptureForMutation $VctipProjectedPath (Join-Path $ScratchRoot 'vctip-duplicated')
    Set-HumDurableBytes (Join-Path $DuplicateVctip 'termination_result.txt') ([Text.Encoding]::UTF8.GetBytes("$VctipRecord`n$VctipRecord`n")); Rewrite-Manifest $DuplicateVctip
    Assert-CaptureRejected $DuplicateVctip 'duplicated VCTIP record accepted'
    $TerminationFailure = Copy-CaptureForMutation $VctipProjectedPath (Join-Path $ScratchRoot 'vctip-termination-failure')
    Set-HumDurableText (Join-Path $TerminationFailure 'termination_disposition.txt') 'tree_termination_failed'; Rewrite-Manifest $TerminationFailure
    Assert-CaptureRejected $TerminationFailure 'VCTIP termination failure accepted'
    $WindowsCaptures = @($Inherited, $InheritedNatural, $Redirected, $Earliest, $Quiescent, $VctipProjected)
  }

  $MissingFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-missing')
  Remove-Item -LiteralPath (Join-Path $MissingFact 'completion_count.txt')
  Assert-CaptureRejected $MissingFact 'missing completion fact accepted'

  $MalformedFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-malformed')
  Set-HumDurableText (Join-Path $MalformedFact 'completion_count.txt') 'malformed'
  Rewrite-Manifest $MalformedFact
  Assert-CaptureRejected $MalformedFact 'malformed completion fact accepted'

  $DuplicateFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-duplicate')
  Set-HumDurableBytes (Join-Path $DuplicateFact 'completion_count.txt') ([Text.Encoding]::UTF8.GetBytes("1`n1`n"))
  Rewrite-Manifest $DuplicateFact
  Assert-CaptureRejected $DuplicateFact 'duplicate completion fact accepted'

  $InconsistentFact = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-inconsistent')
  Set-HumDurableText (Join-Path $InconsistentFact 'completion_count.txt') '2'
  Rewrite-Manifest $InconsistentFact
  Assert-CaptureRejected $InconsistentFact 'inconsistent completion fact accepted'

  $HashMismatch = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-hash')
  Set-HumDurableBytes (Join-Path $HashMismatch 'stdout.bin') ([Text.Encoding]::ASCII.GetBytes("changed`n"))
  Assert-CaptureRejected $HashMismatch 'hash mismatch accepted'

  $Reordered = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-reordered')
  $ManifestLines = [System.Collections.Generic.List[string]] (Get-Content -LiteralPath (Join-Path $Reordered 'manifest.txt'))
  $Swap = $ManifestLines[1]
  $ManifestLines[1] = $ManifestLines[2]
  $ManifestLines[2] = $Swap
  Set-HumDurableText (Join-Path $Reordered 'manifest.txt') ($ManifestLines -join "`n")
  Assert-CaptureRejected $Reordered 'reordered manifest accepted'

  $TerminalMismatch = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-terminal')
  Set-HumDurableBytes (Join-Path $TerminalMismatch 'terminal_stdout_line.bin') ([Text.Encoding]::UTF8.GetBytes('forged terminal'))
  Rewrite-Manifest $TerminalMismatch
  Assert-CaptureRejected $TerminalMismatch 'terminal mismatch accepted'

  $OpaqueBody = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-opaque-body')
  $OpaqueBytes = ([byte[]] (0x82, 0x0a, 0xe1, 0x0d, 0x0a)) + (Read-Bytes $Success.StdoutPath)
  Set-HumDurableBytes (Join-Path $OpaqueBody 'stdout.bin') $OpaqueBytes
  Rewrite-Manifest $OpaqueBody
  $OpaqueRecord = Read-HumCaptureRecord $OpaqueBody
  Assert-True ($OpaqueRecord.TerminalStdoutLine -ceq $SuccessMarker -and $OpaqueRecord.SuccessMarkerCount -eq 1) 'opaque nonterminal bytes changed terminal facts'
  Assert-Bytes (Read-Bytes $OpaqueRecord.StdoutPath) $OpaqueBytes 'opaque stdout bytes were rewritten'

  $OpaqueTerminal = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-opaque-terminal')
  Set-HumDurableBytes (Join-Path $OpaqueTerminal 'stdout.bin') ((Read-Bytes $Success.StdoutPath) + [byte[]] (0x82, 0x0a))
  Set-HumDurableBytes (Join-Path $OpaqueTerminal 'terminal_stdout_line.bin') ([byte[]] (0x82))
  Rewrite-Manifest $OpaqueTerminal
  Assert-CaptureRejected $OpaqueTerminal 'non-ASCII terminal line accepted'

  $QuiescenceMismatch = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-quiescence')
  Set-HumDurableText (Join-Path $QuiescenceMismatch 'job_quiescence_observed.txt') '0'
  Rewrite-Manifest $QuiescenceMismatch
  Assert-CaptureRejected $QuiescenceMismatch 'quiescence corruption accepted'

  $ContainmentOmission = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-containment-omission')
  Remove-Item -LiteralPath (Join-Path $ContainmentOmission 'job_assignment_succeeded.txt')
  Assert-CaptureRejected $ContainmentOmission 'containment record omission accepted'

  $TimeoutDeadlineMismatch = Copy-CaptureForMutation $Timeout.CaptureDirectory (Join-Path $ScratchRoot 'mutation-timeout-deadline')
  Set-HumDurableText (Join-Path $TimeoutDeadlineMismatch 'deadline_ticks.txt') ([string] ($Timeout.DeadlineTicks + $Timeout.StopwatchFrequency))
  Rewrite-Manifest $TimeoutDeadlineMismatch
  Assert-True (-not (Test-LaunchedTimeoutCaptureDirectory $TimeoutDeadlineMismatch $LaunchedTimeoutDeadlineSeconds)) 'launched-timeout deadline corruption accepted'

  if ($env:OS -eq 'Windows_NT') {
    $ContainmentInconsistent = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-containment-inconsistent')
    Assert-True $Success.JobAssignmentSucceeded 'initialized assignment witness missing'
    Set-HumDurableText (Join-Path $ContainmentInconsistent 'job_assignment_succeeded.txt') '0'
    Rewrite-Manifest $ContainmentInconsistent
    Assert-CaptureRejected $ContainmentInconsistent 'inconsistent assigned/resumed state accepted'

    $MemoryTrust = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-memory-trust')
    Assert-True $Success.ResumeSucceeded 'initialized in-memory witness missing'
    Set-HumDurableText (Join-Path $MemoryTrust 'resume_succeeded.txt') '0'
    Rewrite-Manifest $MemoryTrust
    Assert-CaptureRejected $MemoryTrust 'in-memory containment fact overrode disk'
  } else {
    $NonWindowsContainment = Copy-CaptureForMutation $Success.CaptureDirectory (Join-Path $ScratchRoot 'mutation-non-windows-containment')
    Assert-True (-not $Success.JobAssignmentSucceeded) 'non-Windows assignment witness was not false'
    Set-HumDurableText (Join-Path $NonWindowsContainment 'job_assignment_succeeded.txt') '1'
    Rewrite-Manifest $NonWindowsContainment
    Assert-CaptureRejected $NonWindowsContainment 'non-Windows capture accepted Windows assignment fact'
  }

  if ($env:OS -eq 'Windows_NT') {
    $TreeRecord = Read-HumScalar $Inherited.CaptureDirectory 'final_descendant_tree.txt'
    Assert-True ($TreeRecord -cmatch '^terminated_quiescent;pretermination=members;') 'retained pre-termination record missing after final zero'
    Assert-True ((Read-HumScalar $Inherited.CaptureDirectory 'final_active_process_count.txt') -ceq '0') 'retained identities displaced final zero'
    $TreeMatch = [regex]::Match($TreeRecord, '^terminated_quiescent;pretermination=members;active=([0-9]+);member=(.+)$')
    $TreeMembers = $TreeMatch.Groups[2].Value
    $DuplicatedTree = "terminated_quiescent;pretermination=members;active=$([int]$TreeMatch.Groups[1].Value + 1);member=$TreeMembers|$(($TreeMembers -split '\|')[0])"
    foreach ($TreeMutation in @(
      @{ Name = 'malformed'; Value = $TreeRecord + ';extra' },
      @{ Name = 'truncated'; Value = $TreeRecord.Substring(0, $TreeRecord.Length - 1) },
      @{ Name = 'duplicated'; Value = $DuplicatedTree },
      @{ Name = 'reordered'; Value = $Timeout.FinalDescendantTree -replace 'member=([^|]+)\|(.+)$', 'member=$2|$1' }
    )) {
      $TreeCopy = Copy-CaptureForMutation $Inherited.CaptureDirectory (Join-Path $ScratchRoot "mutation-tree-$($TreeMutation.Name)")
      Set-HumDurableText (Join-Path $TreeCopy 'final_descendant_tree.txt') $TreeMutation.Value
      Rewrite-Manifest $TreeCopy
      Assert-CaptureRejected $TreeCopy "descendant tree $($TreeMutation.Name) accepted"
    }
    $TimeoutDisposition = Copy-CaptureForMutation $Inherited.CaptureDirectory (Join-Path $ScratchRoot 'mutation-timeout-disposition')
    Set-HumDurableText (Join-Path $TimeoutDisposition 'timed_out.txt') '0'
    Rewrite-Manifest $TimeoutDisposition
    Assert-CaptureRejected $TimeoutDisposition 'timeout disposition corruption accepted'

    $DoubleTermination = Copy-CaptureForMutation $Inherited.CaptureDirectory (Join-Path $ScratchRoot 'mutation-double-termination')
    Set-HumDurableText (Join-Path $DoubleTermination 'termination_count.txt') '2'
    Set-HumDurableText (Join-Path $DoubleTermination 'kill_attempt_count.txt') '2'
    Rewrite-Manifest $DoubleTermination
    Assert-CaptureRejected $DoubleTermination 'double termination accepted'

    $ActiveDescendant = Copy-CaptureForMutation $Inherited.CaptureDirectory (Join-Path $ScratchRoot 'mutation-descendant-absence')
    Set-HumDurableText (Join-Path $ActiveDescendant 'final_active_process_count.txt') '1'
    Rewrite-Manifest $ActiveDescendant
    Assert-CaptureRejected $ActiveDescendant 'live descendant witness accepted'
  }

  foreach ($Capture in @($Preflight, $Success, $Exit23, $Empty, $Missing, $Interleaved, $Unicode, $Early, $Duplicate, $Nonzero, $Timeout) + $WindowsCaptures + $SetupCaptures) {
    $null = Read-HumCaptureRecord $Capture.CaptureDirectory
    $ValidCaptures.Add($Capture)
    if ($null -ne $Capture.Pid) { $CreatedPids.Add([int] $Capture.Pid) }
  }
  foreach ($CreatedPid in @($CreatedPids | Sort-Object -Unique)) {
    Assert-True ($null -eq (Get-Process -Id $CreatedPid -ErrorAction SilentlyContinue)) "test-created process survived: $CreatedPid"
  }
} finally {
  if (Test-Path -LiteralPath $ScratchRoot) { Remove-Item -LiteralPath $ScratchRoot -Recurse -Force }
}

Assert-True (-not (Test-Path -LiteralPath $ScratchRoot)) 'scratch root cleanup'
Assert-True ((Get-Location).Path -eq $BeforeDirectory) 'current directory changed'
$AfterEnvironment = Get-ProcessEnvironmentSnapshot
Assert-True (Test-ExactBytesEqual $BeforeEnvironment.Bytes $AfterEnvironment.Bytes) 'parent environment changed'
$AfterConfig = Get-GitConfigurationIdentity
Assert-True ($AfterConfig -eq $BeforeConfig) 'local/global/system Git configuration changed'
Write-Output "Fast evidence capture tests passed for $ShellContract."
