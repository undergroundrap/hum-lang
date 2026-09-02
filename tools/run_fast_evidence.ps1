param(
  [string] $ScratchRoot = '',
  [int] $TimeoutSeconds = 3600
)

$ErrorActionPreference = 'Stop'
$script:HumHostIsWindows = [Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT
$script:HumSuccessMarker = 'All Hum preflight checks passed.'
$script:HumCaptureFiles = @(
  'case_name.txt',
  'containment_kind.txt',
  'job_creation_attempted.txt', 'job_creation_succeeded.txt',
  'job_kill_on_close_configured.txt',
  'child_creation_attempted.txt', 'process_creation_attempted.txt',
  'process_creation_succeeded.txt', 'process_created_suspended.txt',
  'launch_succeeded.txt', 'launch_error.bin', 'capture_error.bin', 'pid.txt',
  'job_assignment_attempted.txt', 'job_assignment_succeeded.txt',
  'resume_attempted.txt', 'resume_succeeded.txt',
  'started_utc.txt', 'completed_utc.txt', 'duration_ticks.txt',
  'stopwatch_frequency.txt', 'deadline_ticks.txt',
  'termination_grace_ticks.txt', 'exit.txt', 'primary_exit_observed.txt',
  'stdout_completion_observed.txt', 'stderr_completion_observed.txt',
  'job_quiescence_observed.txt', 'final_active_process_count.txt',
  'deadline_disposition.txt', 'timed_out.txt', 'termination_requested.txt',
  'termination_disposition.txt', 'termination_result.txt',
  'termination_count.txt', 'kill_attempt_count.txt',
  'final_descendant_tree.txt', 'stdout.bin', 'stderr.bin',
  'completion_count.txt', 'terminal_stdout_line.bin',
  'success_marker_count.txt'
)

if (-not ('HumFastJobNative' -as [type])) {
  Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

public sealed class HumFastSuspendedProcess {
  public IntPtr ProcessHandle;
  public IntPtr ThreadHandle;
  public IntPtr StdoutReadHandle;
  public IntPtr StderrReadHandle;
  public UInt32 ProcessId;
}

public static class HumFastJobNative {
  private const UInt32 CreateSuspendedFlag = 0x00000004;
  private const UInt32 CreateNoWindow = 0x08000000;
  private const UInt32 ExtendedStartupInfoPresent = 0x00080000;
  private const UInt32 StartfUseStdHandles = 0x00000100;
  private const UInt32 HandleFlagInherit = 0x00000001;
  private const UInt32 JobObjectLimitKillOnClose = 0x00002000;
  private const UInt32 WaitObject0 = 0x00000000;
  private const UInt32 WaitTimeout = 0x00000102;
  private const UInt32 ProcThreadAttributeHandleList = 0x00020002;

  [StructLayout(LayoutKind.Sequential)]
  private struct SECURITY_ATTRIBUTES {
    public Int32 nLength;
    public IntPtr lpSecurityDescriptor;
    [MarshalAs(UnmanagedType.Bool)] public bool bInheritHandle;
  }

  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  private struct STARTUPINFO {
    public Int32 cb;
    public string lpReserved;
    public string lpDesktop;
    public string lpTitle;
    public Int32 dwX;
    public Int32 dwY;
    public Int32 dwXSize;
    public Int32 dwYSize;
    public Int32 dwXCountChars;
    public Int32 dwYCountChars;
    public Int32 dwFillAttribute;
    public Int32 dwFlags;
    public Int16 wShowWindow;
    public Int16 cbReserved2;
    public IntPtr lpReserved2;
    public IntPtr hStdInput;
    public IntPtr hStdOutput;
    public IntPtr hStdError;
  }

  [StructLayout(LayoutKind.Sequential)]
  private struct STARTUPINFOEX {
    public STARTUPINFO StartupInfo;
    public IntPtr lpAttributeList;
  }

  [StructLayout(LayoutKind.Sequential)]
  private struct PROCESS_INFORMATION {
    public IntPtr hProcess;
    public IntPtr hThread;
    public UInt32 dwProcessId;
    public UInt32 dwThreadId;
  }

  [StructLayout(LayoutKind.Sequential)]
  private struct IO_COUNTERS {
    public UInt64 ReadOperationCount;
    public UInt64 WriteOperationCount;
    public UInt64 OtherOperationCount;
    public UInt64 ReadTransferCount;
    public UInt64 WriteTransferCount;
    public UInt64 OtherTransferCount;
  }

  [StructLayout(LayoutKind.Sequential)]
  private struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
    public Int64 PerProcessUserTimeLimit;
    public Int64 PerJobUserTimeLimit;
    public UInt32 LimitFlags;
    public UIntPtr MinimumWorkingSetSize;
    public UIntPtr MaximumWorkingSetSize;
    public UInt32 ActiveProcessLimit;
    public UIntPtr Affinity;
    public UInt32 PriorityClass;
    public UInt32 SchedulingClass;
  }

  [StructLayout(LayoutKind.Sequential)]
  private struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
    public JOBOBJECT_BASIC_LIMIT_INFORMATION BasicLimitInformation;
    public IO_COUNTERS IoInfo;
    public UIntPtr ProcessMemoryLimit;
    public UIntPtr JobMemoryLimit;
    public UIntPtr PeakProcessMemoryUsed;
    public UIntPtr PeakJobMemoryUsed;
  }

  [StructLayout(LayoutKind.Sequential)]
  private struct JOBOBJECT_BASIC_ACCOUNTING_INFORMATION {
    public Int64 TotalUserTime;
    public Int64 TotalKernelTime;
    public Int64 ThisPeriodTotalUserTime;
    public Int64 ThisPeriodTotalKernelTime;
    public UInt32 TotalPageFaultCount;
    public UInt32 TotalProcesses;
    public UInt32 ActiveProcesses;
    public UInt32 TotalTerminatedProcesses;
  }

  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  private static extern IntPtr CreateJobObjectW(IntPtr attributes, string name);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool SetInformationJobObject(
    IntPtr job, Int32 informationClass, IntPtr information, UInt32 informationLength);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool QueryInformationJobObject(
    IntPtr job, Int32 informationClass, IntPtr information,
    UInt32 informationLength, out UInt32 returnLength);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool TerminateJobObject(IntPtr job, UInt32 exitCode);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool TerminateProcess(IntPtr process, UInt32 exitCode);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool GetExitCodeProcess(IntPtr process, out UInt32 exitCode);

  [DllImport("kernel32.dll", SetLastError = true)]
  private static extern UInt32 WaitForSingleObject(IntPtr handle, UInt32 milliseconds);

  [DllImport("kernel32.dll", SetLastError = true)]
  private static extern UInt32 ResumeThread(IntPtr thread);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  public static extern bool CloseHandle(IntPtr handle);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool CreatePipe(
    out IntPtr readPipe, out IntPtr writePipe,
    ref SECURITY_ATTRIBUTES attributes, UInt32 size);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool SetHandleInformation(
    IntPtr handle, UInt32 mask, UInt32 flags);

  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  private static extern IntPtr CreateFileW(
    string fileName, UInt32 desiredAccess, UInt32 shareMode,
    ref SECURITY_ATTRIBUTES attributes, UInt32 creationDisposition,
    UInt32 flagsAndAttributes, IntPtr templateFile);

  [DllImport("kernel32.dll", SetLastError = true)] [return: MarshalAs(UnmanagedType.Bool)] private static extern bool GetFileInformationByHandle(SafeFileHandle file, [Out] byte[] information);
  [DllImport("libc", SetLastError = true)] private static extern Int32 stat(string path, [Out] byte[] information);
  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool InitializeProcThreadAttributeList(
    IntPtr attributeList, Int32 attributeCount, UInt32 flags, ref IntPtr size);

  [DllImport("kernel32.dll", SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool UpdateProcThreadAttribute(
    IntPtr attributeList, UInt32 flags, UIntPtr attribute,
    IntPtr value, UIntPtr size, IntPtr previousValue, IntPtr returnSize);

  [DllImport("kernel32.dll")]
  private static extern void DeleteProcThreadAttributeList(IntPtr attributeList);

  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  [return: MarshalAs(UnmanagedType.Bool)]
  private static extern bool CreateProcessW(
    string applicationName, StringBuilder commandLine,
    IntPtr processAttributes, IntPtr threadAttributes,
    [MarshalAs(UnmanagedType.Bool)] bool inheritHandles,
    UInt32 creationFlags, IntPtr environment, string currentDirectory,
    ref STARTUPINFOEX startupInfo, out PROCESS_INFORMATION processInformation);

  private static Win32Exception LastError() {
    return new Win32Exception(Marshal.GetLastWin32Error());
  }

  private static void CloseIfValid(ref IntPtr handle) {
    if (handle != IntPtr.Zero && handle != new IntPtr(-1)) {
      CloseHandle(handle);
      handle = IntPtr.Zero;
    }
  }

  public static UInt64[] GetFileIdentity(string path, bool windows) { byte[] information = new byte[windows ? 52 : 144];
    if (windows) {
      using (FileStream file = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete)) {
        if (!GetFileInformationByHandle(file.SafeFileHandle, information)) throw LastError();
      }
    } else if (stat(path, information) != 0) { throw LastError(); }
    UInt64 volume = windows ? BitConverter.ToUInt32(information, 28) : BitConverter.ToUInt64(information, 0), index = windows ? ((UInt64)BitConverter.ToUInt32(information, 44) << 32) | BitConverter.ToUInt32(information, 48) : BitConverter.ToUInt64(information, 8);
    return new UInt64[] { volume, index, windows ? BitConverter.ToUInt32(information, 40) : BitConverter.ToUInt64(information, 16) };
  }
  public static IntPtr CreateConfiguredJob() {
    IntPtr job = CreateJobObjectW(IntPtr.Zero, null);
    if (job == IntPtr.Zero) throw LastError();
    return job;
  }

  public static void ConfigureKillOnClose(IntPtr job) {
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limits =
      new JOBOBJECT_EXTENDED_LIMIT_INFORMATION();
    limits.BasicLimitInformation.LimitFlags = JobObjectLimitKillOnClose;
    Int32 size = Marshal.SizeOf(typeof(JOBOBJECT_EXTENDED_LIMIT_INFORMATION));
    IntPtr memory = Marshal.AllocHGlobal(size);
    try {
      Marshal.StructureToPtr(limits, memory, false);
      if (!SetInformationJobObject(job, 9, memory, (UInt32)size)) throw LastError();
    } finally {
      Marshal.FreeHGlobal(memory);
    }
  }

  public static HumFastSuspendedProcess CreateSuspended(
    string applicationName, string commandLine, string workingDirectory) {
    SECURITY_ATTRIBUTES inheritable = new SECURITY_ATTRIBUTES();
    inheritable.nLength = Marshal.SizeOf(typeof(SECURITY_ATTRIBUTES));
    inheritable.bInheritHandle = true;
    IntPtr stdoutRead = IntPtr.Zero;
    IntPtr stdoutWrite = IntPtr.Zero;
    IntPtr stderrRead = IntPtr.Zero;
    IntPtr stderrWrite = IntPtr.Zero;
    IntPtr stdinNull = IntPtr.Zero;
    IntPtr attributeList = IntPtr.Zero;
    IntPtr handleList = IntPtr.Zero;
    PROCESS_INFORMATION processInfo = new PROCESS_INFORMATION();
    bool created = false;
    try {
      if (!CreatePipe(out stdoutRead, out stdoutWrite, ref inheritable, 0)) throw LastError();
      if (!SetHandleInformation(stdoutRead, HandleFlagInherit, 0)) throw LastError();
      if (!CreatePipe(out stderrRead, out stderrWrite, ref inheritable, 0)) throw LastError();
      if (!SetHandleInformation(stderrRead, HandleFlagInherit, 0)) throw LastError();
      stdinNull = CreateFileW("NUL", 0x80000000, 0x00000003, ref inheritable,
        3, 0x00000080, IntPtr.Zero);
      if (stdinNull == new IntPtr(-1)) throw LastError();

      IntPtr attributeBytes = IntPtr.Zero;
      InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref attributeBytes);
      if (attributeBytes == IntPtr.Zero) throw LastError();
      attributeList = Marshal.AllocHGlobal(attributeBytes);
      if (!InitializeProcThreadAttributeList(attributeList, 1, 0, ref attributeBytes)) {
        throw LastError();
      }
      handleList = Marshal.AllocHGlobal(IntPtr.Size * 3);
      Marshal.WriteIntPtr(handleList, 0, stdinNull);
      Marshal.WriteIntPtr(handleList, IntPtr.Size, stdoutWrite);
      Marshal.WriteIntPtr(handleList, IntPtr.Size * 2, stderrWrite);
      if (!UpdateProcThreadAttribute(
          attributeList, 0, new UIntPtr(ProcThreadAttributeHandleList),
          handleList, new UIntPtr((UInt64)(IntPtr.Size * 3)),
          IntPtr.Zero, IntPtr.Zero)) {
        throw LastError();
      }

      STARTUPINFOEX startup = new STARTUPINFOEX();
      startup.StartupInfo.cb = Marshal.SizeOf(typeof(STARTUPINFOEX));
      startup.StartupInfo.dwFlags = (Int32)StartfUseStdHandles;
      startup.StartupInfo.hStdInput = stdinNull;
      startup.StartupInfo.hStdOutput = stdoutWrite;
      startup.StartupInfo.hStdError = stderrWrite;
      startup.lpAttributeList = attributeList;
      StringBuilder mutableCommandLine = new StringBuilder(commandLine);
      UInt32 flags = CreateSuspendedFlag | CreateNoWindow | ExtendedStartupInfoPresent;
      if (!CreateProcessW(applicationName, mutableCommandLine, IntPtr.Zero, IntPtr.Zero,
          true, flags, IntPtr.Zero, workingDirectory, ref startup, out processInfo)) {
        throw LastError();
      }
      created = true;
      HumFastSuspendedProcess result = new HumFastSuspendedProcess();
      result.ProcessHandle = processInfo.hProcess;
      result.ThreadHandle = processInfo.hThread;
      result.ProcessId = processInfo.dwProcessId;
      result.StdoutReadHandle = stdoutRead;
      result.StderrReadHandle = stderrRead;
      processInfo.hProcess = IntPtr.Zero;
      processInfo.hThread = IntPtr.Zero;
      stdoutRead = IntPtr.Zero;
      stderrRead = IntPtr.Zero;
      return result;
    } finally {
      CloseIfValid(ref stdoutWrite);
      CloseIfValid(ref stderrWrite);
      CloseIfValid(ref stdinNull);
      if (handleList != IntPtr.Zero) Marshal.FreeHGlobal(handleList);
      if (attributeList != IntPtr.Zero) {
        DeleteProcThreadAttributeList(attributeList);
        Marshal.FreeHGlobal(attributeList);
      }
      if (!created) {
        CloseIfValid(ref processInfo.hThread);
        CloseIfValid(ref processInfo.hProcess);
      }
      CloseIfValid(ref stdoutRead);
      CloseIfValid(ref stderrRead);
    }
  }

  public static FileStream OpenAsyncReadStream(IntPtr handle) {
    return new FileStream(new SafeFileHandle(handle, true), FileAccess.Read, 65536, false);
  }

  public static void Assign(IntPtr job, IntPtr process) {
    if (!AssignProcessToJobObject(job, process)) throw LastError();
  }

  public static void Resume(IntPtr thread) {
    if (ResumeThread(thread) == UInt32.MaxValue) throw LastError();
  }

  public static bool Wait(IntPtr process, UInt32 milliseconds) {
    UInt32 result = WaitForSingleObject(process, milliseconds);
    if (result == WaitObject0) return true;
    if (result == WaitTimeout) return false;
    throw LastError();
  }

  public static Int32 ExitCode(IntPtr process) {
    UInt32 code;
    if (!GetExitCodeProcess(process, out code)) throw LastError();
    return unchecked((Int32)code);
  }

  public static UInt32 ActiveProcessCount(IntPtr job) {
    Int32 size = Marshal.SizeOf(typeof(JOBOBJECT_BASIC_ACCOUNTING_INFORMATION));
    IntPtr memory = Marshal.AllocHGlobal(size);
    try {
      UInt32 returned;
      if (!QueryInformationJobObject(job, 1, memory, (UInt32)size, out returned)) {
        throw LastError();
      }
      JOBOBJECT_BASIC_ACCOUNTING_INFORMATION info =
        (JOBOBJECT_BASIC_ACCOUNTING_INFORMATION)Marshal.PtrToStructure(
          memory, typeof(JOBOBJECT_BASIC_ACCOUNTING_INFORMATION));
      return info.ActiveProcesses;
    } finally {
      Marshal.FreeHGlobal(memory);
    }
  }
  public static UInt64[] ProcessIds(IntPtr job) {
    const Int32 capacity = 4096;
    Int32 size = 8 + (capacity * IntPtr.Size);
    IntPtr memory = Marshal.AllocHGlobal(size);
    try {
      UInt32 returned;
      if (!QueryInformationJobObject(job, 3, memory, (UInt32)size, out returned)) throw LastError();
      UInt32 assigned = (UInt32)Marshal.ReadInt32(memory, 0);
      UInt32 count = (UInt32)Marshal.ReadInt32(memory, 4);
      if (assigned > capacity || count > assigned) throw new InvalidOperationException("Job process list overflow");
      UInt64[] ids = new UInt64[count];
      for (Int32 index = 0; index < count; index++) ids[index] = (UInt64)Marshal.ReadIntPtr(memory, 8 + (index * IntPtr.Size)).ToInt64();
      return ids;
    } finally { Marshal.FreeHGlobal(memory); }
  }

  public static bool KillJob(IntPtr job) {
    return TerminateJobObject(job, 57005);
  }

  public static bool KillPrimary(IntPtr process) {
    return TerminateProcess(process, 57005);
  }
}
'@
}

function ConvertTo-HumNativeArgument {
  param([string] $Value)
  if ($Value -notmatch '[\s"]') { return $Value }
  return '"' + ($Value -replace '(\\*)"', '$1$1\"' -replace '(\\+)$', '$1$1') + '"'
}

function Get-HumFileIdentity {
  param([string] $Path)
  if (-not [System.IO.File]::Exists($Path)) { throw "capture is missing: $Path" }
  $Bytes = [System.IO.File]::ReadAllBytes($Path)
  $Hasher = [System.Security.Cryptography.SHA256]::Create()
  try { $Hash = $Hasher.ComputeHash([byte[]] $Bytes) } finally { $Hasher.Dispose() }
  [pscustomobject]@{
    Bytes = $Bytes.Length
    Sha256 = (($Hash | ForEach-Object { $_.ToString('x2') }) -join '')
  }
}

function Set-HumDurableBytes {
  param([string] $Path, [byte[]] $Bytes)
  $Stream = New-Object System.IO.FileStream(
    $Path,
    [System.IO.FileMode]::Create,
    [System.IO.FileAccess]::Write,
    [System.IO.FileShare]::Read
  )
  try {
    $Stream.Write($Bytes, 0, $Bytes.Length)
    $Stream.Flush($true)
  } finally {
    $Stream.Dispose()
  }
}

function Set-HumDurableText {
  param([string] $Path, [string] $Text)
  $Encoding = New-Object System.Text.UTF8Encoding($false, $true)
  Set-HumDurableBytes $Path $Encoding.GetBytes($Text + "`n")
}

function Set-HumCaptureFlag {
  param([string] $CaptureDirectory, [string] $Name, [bool] $Value)
  Set-HumDurableText (Join-Path $CaptureDirectory $Name) $(if ($Value) { '1' } else { '0' })
}

function Read-HumStrictUtf8 {
  param([string] $Path, [bool] $RequireFinalLf = $false)
  $Bytes = [System.IO.File]::ReadAllBytes($Path)
  if ($Bytes.Length -ge 3 -and $Bytes[0] -eq 0xef -and $Bytes[1] -eq 0xbb -and $Bytes[2] -eq 0xbf) {
    throw "UTF-8 BOM forbidden: $Path"
  }
  if ($Bytes -contains 13) { throw "CR byte forbidden: $Path" }
  if ($RequireFinalLf -and ($Bytes.Length -eq 0 -or $Bytes[$Bytes.Length - 1] -ne 10)) {
    throw "final LF required: $Path"
  }
  $Encoding = New-Object System.Text.UTF8Encoding($false, $true)
  $Encoding.GetString($Bytes)
}

function Read-HumScalar {
  param([string] $CaptureDirectory, [string] $Name)
  $Text = Read-HumStrictUtf8 (Join-Path $CaptureDirectory $Name) $true
  if ($Text.IndexOf("`n") -ne $Text.Length - 1) {
    throw "scalar record must contain one line: $Name"
  }
  $Text.Substring(0, $Text.Length - 1)
}

function ConvertFrom-HumUnsignedInteger {
  param([string] $Value, [string] $Name, [bool] $Nullable = $false)
  if ($Nullable -and $Value -ceq 'null') { return $null }
  if ($Value -notmatch '^(0|[1-9][0-9]*)$') { throw "invalid unsigned integer: $Name" }
  [UInt64]::Parse($Value, [Globalization.CultureInfo]::InvariantCulture)
}

function ConvertFrom-HumCaptureFlag {
  param([string] $Value, [string] $Name)
  if ($Value -cnotmatch '^[01]$') { throw "invalid Boolean record: $Name" }
  $Value -ceq '1'
}

function ConvertTo-HumDescendantPathToken {
  param([string] $Path)
  [Convert]::ToBase64String((New-Object Text.UTF8Encoding($false, $true)).GetBytes($Path)).TrimEnd('=').Replace('+', '-').Replace('/', '_')
}
function Get-HumActiveDescendantRecord {
  param([IntPtr] $JobHandle, [UInt64] $PrimaryPid)
  $First = @([HumFastJobNative]::ProcessIds($JobHandle) | Sort-Object)
  if ($First.Count -ne (@($First | Select-Object -Unique)).Count) { throw 'descendant_evidence: duplicate Job process identity' }
  $Members = foreach ($PidValue in $First) {
    try {
      $Process = Get-Process -Id ([int] $PidValue) -ErrorAction Stop
      $Created = [UInt64]$Process.StartTime.ToUniversalTime().Ticks
      $Image = $Process.Path
      if ([string]::IsNullOrWhiteSpace($Image) -or $(if ($script:HumHostIsWindows) { -not [IO.Path]::IsPathRooted($Image) -or [IO.Path]::GetFullPath($Image) -cne $Image } else { -not $Image.StartsWith('/') })) { throw 'image path unavailable' }
      $Item = Get-Item -LiteralPath $Image -Force -ErrorAction Stop
      if ($Item.PSIsContainer -or ($Item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'image is not an ordinary file' }
      $Identity = Get-HumFileIdentity $Item.FullName
      [pscustomobject]@{ Pid = [UInt64]$PidValue; Created = $Created; Primary = [int]([UInt64]$PidValue -eq $PrimaryPid); Bytes = $Identity.Bytes; Sha256 = $Identity.Sha256; Path = ConvertTo-HumDescendantPathToken $Item.FullName }
    } catch { throw "descendant_evidence: incomplete active Job identity acquisition for PID ${PidValue}: $($_.Exception.Message)" }
  }
  $Second = @([HumFastJobNative]::ProcessIds($JobHandle) | Sort-Object)
  $Active = [UInt64][HumFastJobNative]::ActiveProcessCount($JobHandle)
  if ($Active -eq 0 -and $Second.Count -eq 0) { return 'pretermination=quiescent_race' }
  if ($Active -ne $Second.Count -or ($First -join ',') -cne ($Second -join ',')) { throw 'descendant_evidence: active Job membership changed during identity acquisition' }
  $Encoded = @($Members | ForEach-Object { "$($_.Pid),$($_.Created),$($_.Primary),$($_.Bytes),$($_.Sha256),$($_.Path)" }) -join '|'
  "pretermination=members;active=$Active;member=$Encoded"
}
function Assert-HumVctipPath {
  param([string] $Path, [string] $ToolchainRoot)
  $Full = [IO.Path]::GetFullPath($Path); $Root = [IO.Path]::GetFullPath($ToolchainRoot).TrimEnd('\')
  if (-not $Full.StartsWith($Root + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'vctip_auxiliary: path outside authenticated MSVC root' }
  $Current = [IO.Path]::GetPathRoot($Full)
  foreach ($Part in $Full.Substring($Current.Length).Split('\')) {
    $Current = Join-Path $Current $Part; $Item = Get-Item -LiteralPath $Current -Force -ErrorAction Stop
    if (($Item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
        ($Current -cne $Full -and -not $Item.PSIsContainer) -or
        ($Current -ceq $Full -and $Item.PSIsContainer)) { throw 'vctip_auxiliary: path component is not ordinary' }
  }
  $Full
}
function Assert-HumVctipFacts {
  param([pscustomobject] $Facts)
  if ($Facts.Active -ne 1 -or $Facts.Primary -ne 0) { throw 'vctip_auxiliary: expected exactly one non-primary Job member' }
  if ($Facts.Basename -cne 'VCTIP.EXE') { throw 'vctip_auxiliary: filesystem filename mismatch' }
  if ($Facts.OriginalFilename -cne 'VCTIP.EXE') { throw 'vctip_auxiliary: signed original filename mismatch' }
  $ExpectedDescription = 'Microsoft' + [char]0x00ae + ' VC compiler and tools experience improvement data uploader'
  if ($Facts.Description -cne $ExpectedDescription) { throw 'vctip_auxiliary: signed description mismatch' }
  if ($Facts.SignatureStatus -cne 'Valid' -or $Facts.Publisher -cne 'Microsoft Corporation') { throw 'vctip_auxiliary: Microsoft signature invalid' }
  if ($Facts.Links -ne 1) { throw 'vctip_auxiliary: executable is hard linked' }
  if ($Facts.Pid -le 0 -or $Facts.Generation -le 0 -or $Facts.Bytes -le 0 -or
      $Facts.Identity -cnotmatch '^[0-9a-f]{16}:[0-9a-f]{16}$' -or
      $Facts.Sha256 -cnotmatch '^[0-9a-f]{64}$' -or $Facts.PathToken -cnotmatch '^[A-Za-z0-9_-]+$' -or
      $Facts.Certificate -cnotmatch '^[0-9A-F]{40}$') { throw 'vctip_auxiliary: identity record malformed' }
}
function Get-HumVctipFacts {
  param([IntPtr] $JobHandle, [UInt64] $PrimaryPid, [string] $ToolchainRoot)
  $Pids = @([HumFastJobNative]::ProcessIds($JobHandle) | Sort-Object)
  if ($Pids.Count -ne 1 -or $Pids[0] -eq $PrimaryPid) { throw 'vctip_auxiliary: expected exactly one non-primary Job member' }
  $Process = Get-Process -Id ([int]$Pids[0]) -ErrorAction Stop
  $Path = Assert-HumVctipPath $Process.Path $ToolchainRoot; $Item = Get-Item -LiteralPath $Path -Force
  $Native = [HumFastJobNative]::GetFileIdentity($Path, $true); $Signature = Get-AuthenticodeSignature -LiteralPath $Path
  $Version = [Diagnostics.FileVersionInfo]::GetVersionInfo($Path)
  $Publisher = if ($null -eq $Signature.SignerCertificate) { '' } else { $Signature.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false) }
  $Certificate = if ($null -eq $Signature.SignerCertificate) { '' } else { $Signature.SignerCertificate.Thumbprint }
  $Facts = [pscustomobject][ordered]@{
    Active = 1; Pid = [UInt64]$Pids[0]; Generation = [UInt64]$Process.StartTime.ToUniversalTime().Ticks
    Primary = 0; Basename = $Item.Name; OriginalFilename = $Version.OriginalFilename
    Description = $Version.FileDescription; SignatureStatus = [string]$Signature.Status
    Publisher = $Publisher; Certificate = $Certificate; Identity = ('{0:x16}:{1:x16}' -f $Native[0], $Native[1])
    Links = [UInt64]$Native[2]; Bytes = [UInt64]$Item.Length
    Sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    PathToken = ConvertTo-HumDescendantPathToken $Path
  }
  Assert-HumVctipFacts $Facts; $Facts
}
function ConvertTo-HumVctipRecord {
  param([pscustomobject] $Facts)
  "authenticated_vctip_auxiliary;pid=$($Facts.Pid);generation=$($Facts.Generation);identity=$($Facts.Identity);bytes=$($Facts.Bytes);sha256=$($Facts.Sha256);certificate=$($Facts.Certificate);path=$($Facts.PathToken)"
}
function Complete-HumAuthenticatedVctip {
  param([object] $State, [object] $NativeProcess, [IntPtr] $JobHandle, [string] $CaptureDirectory, [string] $ToolchainRoot)
  if (-not $State.PrimaryExited -or $State.ExitCode -ne 0 -or -not $State.StdoutCompleted -or -not $State.StderrCompleted) { throw 'vctip_auxiliary: primary or streams incomplete' }
  $Before = Get-HumVctipFacts $JobHandle ([UInt64]$NativeProcess.ProcessId) $ToolchainRoot
  $Record = ConvertTo-HumVctipRecord $Before
  Set-HumDurableText (Join-Path $CaptureDirectory 'final_descendant_tree.txt') ("pretermination_pending;" + $Record)
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_result.txt') $Record
  $After = Get-HumVctipFacts $JobHandle ([UInt64]$NativeProcess.ProcessId) $ToolchainRoot
  if ((ConvertTo-HumVctipRecord $After) -cne $Record) { throw 'vctip_auxiliary: identity drift before termination' }
  $State.TerminationCount = 1; Set-HumCaptureFlag $CaptureDirectory 'termination_requested.txt' $true
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_count.txt') '1'; Set-HumDurableText (Join-Path $CaptureDirectory 'kill_attempt_count.txt') '1'
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_disposition.txt') 'authenticated_vctip_requested'
  if (-not [HumFastJobNative]::KillJob($JobHandle)) { throw 'vctip_auxiliary: Job termination failed' }
  $State.TerminationResult = $Record; $Record
}
function Assert-HumFinalDescendantTree {
  param([string] $Value)
  if ($Value -cmatch '^(quiescent|suspended_unassigned_terminated|no_process_created|unconfirmed)$') { return }
  if ($Value -ceq 'terminated_quiescent;pretermination=authenticated_vctip_auxiliary') { return }
  if ($Value -ceq 'terminated_quiescent;pretermination=quiescent_race') { return }
  if ($Value -cnotmatch '^terminated_quiescent;pretermination=members;active=([1-9][0-9]*);member=(.+)$') { throw 'final descendant tree malformed' }
  $Active = [UInt64]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
  $Last = [UInt64]0; $Count = 0
  foreach ($Member in @($Matches[2] -split '\|')) {
    if ($Member -cnotmatch '^([1-9][0-9]*),([1-9][0-9]*),([01]),([1-9][0-9]*),([0-9a-f]{64}),([A-Za-z0-9_-]+)$') { throw 'final descendant member malformed' }
    $PidValue = [UInt64]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture)
    if ($PidValue -le $Last) { throw 'final descendant members duplicated or reordered' }
    $Token = $Matches[6]; $Padding = '=' * ((4 - ($Token.Length % 4)) % 4)
    try { $Path = (New-Object Text.UTF8Encoding($false, $true)).GetString([Convert]::FromBase64String(($Token.Replace('-', '+').Replace('_', '/') + $Padding))) } catch { throw 'final descendant image identity malformed' }
    if ($(if ($script:HumHostIsWindows) { -not [IO.Path]::IsPathRooted($Path) -or [IO.Path]::GetFullPath($Path) -cne $Path } else { -not $Path.StartsWith('/') }) -or (ConvertTo-HumDescendantPathToken $Path) -cne $Token) { throw 'final descendant image identity malformed' }
    $Last = $PidValue; $Count++
  }
  if ($Count -ne $Active) { throw 'final descendant identity count disagrees with Job active count' }
}

function Get-HumTerminalFacts {
  param([byte[]] $StdoutBytes)
  $Hex = [BitConverter]::ToString($StdoutBytes)
  $Lines = @(
    [regex]::Split($Hex, '0D-0A|0A|0D') | ForEach-Object { $_.Trim('-') } |
      Where-Object { $_.Length -ne 0 }
  )
  if ($Lines.Count -ne 0 -and $Lines[-1] -cnotmatch '^(?:[0-7][0-9A-F])(?:-(?:[0-7][0-9A-F]))*$') { throw 'terminal stdout line is not ASCII' }
  [pscustomobject]@{
    Terminal = if ($Lines.Count -eq 0) { '' } else { -join @($Lines[-1].Split('-') | ForEach-Object { [char][Convert]::ToByte($_, 16) }) }
    MarkerCount = @($Lines | Where-Object { $_ -ceq [BitConverter]::ToString([Text.Encoding]::ASCII.GetBytes($script:HumSuccessMarker)) }).Count
  }
}

function Write-HumCaptureManifest {
  param([string] $CaptureDirectory)
  $Lines = New-Object System.Collections.Generic.List[string]
  $Lines.Add('schema=hum.fast_evidence_capture.v3')
  foreach ($Name in $script:HumCaptureFiles) {
    $Identity = Get-HumFileIdentity (Join-Path $CaptureDirectory $Name)
    $Lines.Add("file=$Name;bytes=$($Identity.Bytes);sha256=$($Identity.Sha256)")
  }
  Set-HumDurableText (Join-Path $CaptureDirectory 'manifest.txt') ($Lines -join "`n")
}

function Read-HumCaptureRecord {
  param([string] $CaptureDirectory)
  $ExpectedNames = @(($script:HumCaptureFiles + 'manifest.txt') | Sort-Object)
  $ActualEntries = @(Get-ChildItem -LiteralPath $CaptureDirectory | Sort-Object Name)
  if (@($ActualEntries | Where-Object { -not $_.PSIsContainer }).Count -ne $ActualEntries.Count) {
    throw 'capture inventory contains a directory'
  }
  $ActualNames = @($ActualEntries | Select-Object -ExpandProperty Name)
  if (($ExpectedNames -join "`n") -cne ($ActualNames -join "`n")) {
    throw 'capture inventory mismatch'
  }

  $ManifestPath = Join-Path $CaptureDirectory 'manifest.txt'
  $ManifestText = Read-HumStrictUtf8 $ManifestPath $true
  $ManifestLines = @($ManifestText.Substring(0, $ManifestText.Length - 1) -split "`n")
  if ($ManifestLines.Count -ne $script:HumCaptureFiles.Count + 1 -or
      $ManifestLines[0] -cne 'schema=hum.fast_evidence_capture.v3') {
    throw 'capture manifest framing mismatch'
  }
  for ($Index = 0; $Index -lt $script:HumCaptureFiles.Count; $Index++) {
    $Name = $script:HumCaptureFiles[$Index]
    $Identity = Get-HumFileIdentity (Join-Path $CaptureDirectory $Name)
    $Expected = "file=$Name;bytes=$($Identity.Bytes);sha256=$($Identity.Sha256)"
    if ($ManifestLines[$Index + 1] -cne $Expected) {
      throw "capture manifest identity mismatch: $Name"
    }
  }

  $CaseName = Read-HumScalar $CaptureDirectory 'case_name.txt'
  if ($CaseName -cnotmatch '^[a-z0-9][a-z0-9_-]{0,63}$') { throw 'capture case name malformed' }
  $Kind = Read-HumScalar $CaptureDirectory 'containment_kind.txt'
  if ($Kind -cnotmatch '^(windows_job|process_tree)$') { throw 'containment kind malformed' }
  $JobAttempted = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'job_creation_attempted.txt') 'job_creation_attempted'
  $JobCreated = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'job_creation_succeeded.txt') 'job_creation_succeeded'
  $JobConfigured = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'job_kill_on_close_configured.txt') 'job_kill_on_close_configured'
  $ChildAttempted = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'child_creation_attempted.txt') 'child_creation_attempted'
  $ProcessAttempted = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'process_creation_attempted.txt') 'process_creation_attempted'
  $ProcessCreated = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'process_creation_succeeded.txt') 'process_creation_succeeded'
  $CreatedSuspended = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'process_created_suspended.txt') 'process_created_suspended'
  $Launched = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'launch_succeeded.txt') 'launch_succeeded'
  $AssignAttempted = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'job_assignment_attempted.txt') 'job_assignment_attempted'
  $Assigned = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'job_assignment_succeeded.txt') 'job_assignment_succeeded'
  $ResumeAttempted = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'resume_attempted.txt') 'resume_attempted'
  $Resumed = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'resume_succeeded.txt') 'resume_succeeded'
  $PrimaryExited = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'primary_exit_observed.txt') 'primary_exit_observed'
  $StdoutCompleted = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'stdout_completion_observed.txt') 'stdout_completion_observed'
  $StderrCompleted = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'stderr_completion_observed.txt') 'stderr_completion_observed'
  $JobQuiescent = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'job_quiescence_observed.txt') 'job_quiescence_observed'
  $TimedOut = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'timed_out.txt') 'timed_out'
  $TerminationRequested = ConvertFrom-HumCaptureFlag (Read-HumScalar $CaptureDirectory 'termination_requested.txt') 'termination_requested'
  $RetainedPid = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'pid.txt') 'pid' $true
  $Started = [DateTime]::ParseExact((Read-HumScalar $CaptureDirectory 'started_utc.txt'), 'o', [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
  $Completed = [DateTime]::ParseExact((Read-HumScalar $CaptureDirectory 'completed_utc.txt'), 'o', [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::RoundtripKind)
  $Duration = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'duration_ticks.txt') 'duration_ticks'
  $Frequency = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'stopwatch_frequency.txt') 'stopwatch_frequency'
  $DeadlineTicks = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'deadline_ticks.txt') 'deadline_ticks'
  $GraceTicks = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'termination_grace_ticks.txt') 'termination_grace_ticks'
  $ExitText = Read-HumScalar $CaptureDirectory 'exit.txt'
  $ExitCode = if ($ExitText -ceq 'null') { $null } elseif ($ExitText -match '^-?(0|[1-9][0-9]*)$') {
    [int]::Parse($ExitText, [Globalization.CultureInfo]::InvariantCulture)
  } else { throw 'exit record malformed' }
  $FinalActive = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'final_active_process_count.txt') 'final_active_process_count' $true
  $DeadlineDisposition = Read-HumScalar $CaptureDirectory 'deadline_disposition.txt'
  $TerminationDisposition = Read-HumScalar $CaptureDirectory 'termination_disposition.txt'
  $TerminationResult = Read-HumScalar $CaptureDirectory 'termination_result.txt'
  $TerminationCount = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'termination_count.txt') 'termination_count'
  $KillCount = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'kill_attempt_count.txt') 'kill_attempt_count'
  $FinalTree = Read-HumScalar $CaptureDirectory 'final_descendant_tree.txt'
  Assert-HumFinalDescendantTree $FinalTree
  $CompletionCount = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'completion_count.txt') 'completion_count'
  $MarkerCount = ConvertFrom-HumUnsignedInteger (Read-HumScalar $CaptureDirectory 'success_marker_count.txt') 'success_marker_count'
  $LaunchErrorBytes = (Get-HumFileIdentity (Join-Path $CaptureDirectory 'launch_error.bin')).Bytes
  $CaptureErrorBytes = (Get-HumFileIdentity (Join-Path $CaptureDirectory 'capture_error.bin')).Bytes

  if ($Completed -lt $Started -or $Frequency -eq 0 -or $DeadlineTicks -eq 0 -or $GraceTicks -eq 0) {
    throw 'capture timing record invalid'
  }
  if ($ChildAttempted -ne $ProcessAttempted -or $Launched -ne $ProcessCreated -or
      $TerminationCount -ne $KillCount -or $TerminationCount -gt 1 -or
      $TerminationRequested -ne ($TerminationCount -eq 1)) {
    throw 'capture lifecycle aliases disagree'
  }
  if ($Kind -ceq 'windows_job') {
    if (-not $JobAttempted -or ($JobConfigured -and -not $JobCreated) -or
        ($ProcessAttempted -and -not $JobConfigured) -or
        ($ProcessCreated -and (-not $CreatedSuspended -or -not $JobConfigured)) -or
        ($AssignAttempted -and -not $ProcessCreated) -or
        ($Assigned -and -not $AssignAttempted) -or
        ($ResumeAttempted -and -not $Assigned) -or
        ($Resumed -and -not $ResumeAttempted)) {
      throw 'Windows containment lifecycle is inconsistent'
    }
  } elseif ($JobAttempted -or $JobCreated -or $JobConfigured -or $CreatedSuspended -or
            $AssignAttempted -or $Assigned -or $ResumeAttempted -or $Resumed) {
    throw 'non-Windows capture contains Windows containment facts'
  }
  if ($ProcessCreated) {
    if ($null -eq $RetainedPid -or -not $PrimaryExited -or $CompletionCount -ne 1 -or
        -not $StdoutCompleted -or -not $StderrCompleted -or $null -eq $ExitCode -or
        $LaunchErrorBytes -ne 0 -or -not $JobQuiescent -or $FinalActive -ne 0) {
      throw 'created-process capture is incomplete'
    }
    if ($Kind -ceq 'windows_job' -and $CaptureErrorBytes -eq 0 -and
        (-not $Assigned -or -not $Resumed)) {
      throw 'successful Windows capture lacks assigned and resumed containment'
    }
  } elseif ($null -ne $RetainedPid -or $PrimaryExited -or $CompletionCount -ne 0 -or
            $StdoutCompleted -or $StderrCompleted -or $null -ne $ExitCode -or
            $TerminationRequested -or $TerminationCount -ne 0 -or
            $FinalTree -cne 'no_process_created' -or $LaunchErrorBytes -eq 0 -or
            $CaptureErrorBytes -ne 0) {
    throw 'prelaunch capture state is inconsistent'
  }
  if ($TimedOut) {
    $ExpectedTimeoutResult = if ($Kind -ceq 'windows_job' -and -not $Assigned) {
      'unassigned_primary_terminated'
    } else { 'job_terminated_quiescent' }
    $ExpectedTimeoutTree = if ($Kind -ceq 'windows_job' -and -not $Assigned) {
      '^suspended_unassigned_terminated$'
    } else { '^terminated_quiescent;pretermination=(?:quiescent_race|members;active=[1-9][0-9]*;member=.+)$' }
    if ($DeadlineDisposition -cne 'deadline_expired' -or -not $TerminationRequested -or
        $TerminationDisposition -cne 'tree_termination_confirmed' -or
        $TerminationResult -cne $ExpectedTimeoutResult -or
        $FinalTree -cnotmatch $ExpectedTimeoutTree) {
      throw 'timeout containment record invalid'
    }
  } elseif ($Resumed -and $CaptureErrorBytes -eq 0) {
    $Ordinary = $DeadlineDisposition -ceq 'completed_before_deadline' -and -not $TerminationRequested -and
      $TerminationDisposition -ceq 'not_requested' -and $TerminationResult -ceq 'not_requested' -and $FinalTree -ceq 'quiescent'
    $Vctip = $DeadlineDisposition -ceq 'completed_after_authenticated_vctip_termination' -and $TerminationRequested -and
      $TerminationDisposition -ceq 'authenticated_vctip_termination_confirmed' -and
      $TerminationResult -cmatch '^authenticated_vctip_auxiliary;pid=[1-9][0-9]*;generation=[1-9][0-9]*;identity=[0-9a-f]{16}:[0-9a-f]{16};bytes=[1-9][0-9]*;sha256=[0-9a-f]{64};certificate=[0-9A-F]{40};path=[A-Za-z0-9_-]+$' -and
      $FinalTree -ceq 'terminated_quiescent;pretermination=authenticated_vctip_auxiliary' -and $TerminationCount -eq 1
    if (-not $Ordinary -and -not $Vctip) {
      throw 'ordinary completion containment record invalid'
    }
  }

  $StdoutPath = Join-Path $CaptureDirectory 'stdout.bin'
  $StderrPath = Join-Path $CaptureDirectory 'stderr.bin'
  $StdoutRaw = [System.IO.File]::ReadAllBytes($StdoutPath)
  $TerminalFacts = Get-HumTerminalFacts $StdoutRaw
  $Encoding = New-Object System.Text.UTF8Encoding($false, $true)
  $PersistedTerminal = $Encoding.GetString([System.IO.File]::ReadAllBytes((Join-Path $CaptureDirectory 'terminal_stdout_line.bin')))
  if ($PersistedTerminal -cne $TerminalFacts.Terminal -or $MarkerCount -ne $TerminalFacts.MarkerCount) {
    throw 'capture terminal facts mismatch'
  }
  $StdoutIdentity = Get-HumFileIdentity $StdoutPath
  $StderrIdentity = Get-HumFileIdentity $StderrPath
  [pscustomobject]@{
    CaseName = $CaseName
    CaptureDirectory = [System.IO.Path]::GetFullPath($CaptureDirectory)
    ContainmentKind = $Kind
    JobCreationAttempted = $JobAttempted
    JobCreationSucceeded = $JobCreated
    JobKillOnCloseConfigured = $JobConfigured
    ChildCreationAttempted = $ChildAttempted
    ProcessCreationAttempted = $ProcessAttempted
    ProcessCreationSucceeded = $ProcessCreated
    ProcessCreatedSuspended = $CreatedSuspended
    Launched = $Launched
    LaunchErrorPath = Join-Path $CaptureDirectory 'launch_error.bin'
    CaptureErrorPath = Join-Path $CaptureDirectory 'capture_error.bin'
    CaptureErrorBytes = $CaptureErrorBytes
    Pid = $RetainedPid
    JobAssignmentAttempted = $AssignAttempted
    JobAssignmentSucceeded = $Assigned
    ResumeAttempted = $ResumeAttempted
    ResumeSucceeded = $Resumed
    StartedUtc = $Started
    CompletedUtc = $Completed
    DurationTicks = $Duration
    StopwatchFrequency = $Frequency
    DeadlineTicks = $DeadlineTicks
    TerminationGraceTicks = $GraceTicks
    ExitCode = $ExitCode
    PrimaryExitObserved = $PrimaryExited
    StdoutCompletionObserved = $StdoutCompleted
    StderrCompletionObserved = $StderrCompleted
    JobQuiescenceObserved = $JobQuiescent
    FinalActiveProcessCount = $FinalActive
    DeadlineDisposition = $DeadlineDisposition
    TimedOut = $TimedOut
    TerminationRequested = $TerminationRequested
    TerminationDisposition = $TerminationDisposition
    TerminationResult = $TerminationResult
    TerminationCount = $TerminationCount
    KillAttemptCount = $KillCount
    FinalDescendantTree = $FinalTree
    StdoutPath = $StdoutPath
    StderrPath = $StderrPath
    StdoutBytes = $StdoutIdentity.Bytes
    StdoutSha256 = $StdoutIdentity.Sha256
    StderrBytes = $StderrIdentity.Bytes
    StderrSha256 = $StderrIdentity.Sha256
    CompletionCount = $CompletionCount
    TerminalStdoutLine = $PersistedTerminal
    SuccessMarkerCount = $MarkerCount
    ManifestPath = $ManifestPath
  }
}

function Get-HumRemainingMilliseconds {
  param([System.Diagnostics.Stopwatch] $Timer, [Int64] $DeadlineTicks, [int] $Maximum = 50)
  $RemainingTicks = $DeadlineTicks - $Timer.ElapsedTicks
  if ($RemainingTicks -le 0) { return 0 }
  $Milliseconds = [Math]::Ceiling(($RemainingTicks * 1000.0) / [System.Diagnostics.Stopwatch]::Frequency)
  [int] [Math]::Min([double] $Maximum, [Math]::Max(1.0, $Milliseconds))
}

function Initialize-HumCaptureRecord {
  param([string] $Directory, [string] $CaseName, [string] $Kind, [DateTime] $Started, [Int64] $Deadline, [Int64] $Grace)
  if ($CaseName -cnotmatch '^[a-z0-9][a-z0-9_-]{0,63}$') { throw 'capture case name malformed' }
  Set-HumDurableText (Join-Path $Directory 'case_name.txt') $CaseName
  Set-HumDurableText (Join-Path $Directory 'containment_kind.txt') $Kind
  foreach ($Name in @(
    'job_creation_attempted.txt', 'job_creation_succeeded.txt',
    'job_kill_on_close_configured.txt', 'child_creation_attempted.txt',
    'process_creation_attempted.txt', 'process_creation_succeeded.txt',
    'process_created_suspended.txt', 'launch_succeeded.txt',
    'job_assignment_attempted.txt', 'job_assignment_succeeded.txt',
    'resume_attempted.txt', 'resume_succeeded.txt',
    'primary_exit_observed.txt', 'stdout_completion_observed.txt',
    'stderr_completion_observed.txt', 'job_quiescence_observed.txt',
    'timed_out.txt', 'termination_requested.txt'
  )) { Set-HumDurableText (Join-Path $Directory $Name) '0' }
  Set-HumDurableBytes (Join-Path $Directory 'launch_error.bin') ([byte[]] @())
  Set-HumDurableBytes (Join-Path $Directory 'capture_error.bin') ([byte[]] @())
  Set-HumDurableText (Join-Path $Directory 'pid.txt') 'null'
  Set-HumDurableText (Join-Path $Directory 'started_utc.txt') $Started.ToString('o', [Globalization.CultureInfo]::InvariantCulture)
  Set-HumDurableText (Join-Path $Directory 'stopwatch_frequency.txt') ([System.Diagnostics.Stopwatch]::Frequency.ToString([Globalization.CultureInfo]::InvariantCulture))
  Set-HumDurableText (Join-Path $Directory 'deadline_ticks.txt') $Deadline.ToString([Globalization.CultureInfo]::InvariantCulture)
  Set-HumDurableText (Join-Path $Directory 'termination_grace_ticks.txt') $Grace.ToString([Globalization.CultureInfo]::InvariantCulture)
}

function Update-HumCaptureObservation {
  param(
    [object] $State,
    [bool] $WindowsPlatform,
    [object] $NativeProcess,
    [System.Diagnostics.Process] $ManagedProcess,
    [IntPtr] $JobHandle,
    [System.Threading.Tasks.Task] $StdoutTask,
    [System.Threading.Tasks.Task] $StderrTask,
    [string] $CaptureDirectory
  )
  if (-not $State.PrimaryExited) {
    $Exited = if ($WindowsPlatform) { [HumFastJobNative]::Wait($NativeProcess.ProcessHandle, 0) } else { $ManagedProcess.HasExited }
    if ($Exited) {
      $State.PrimaryExited = $true
      $State.ExitCode = if ($WindowsPlatform) { [HumFastJobNative]::ExitCode($NativeProcess.ProcessHandle) } else { $ManagedProcess.ExitCode }
      Set-HumCaptureFlag $CaptureDirectory 'primary_exit_observed.txt' $true
    }
  }
  if (-not $State.StdoutCompleted -and $StdoutTask.IsCompleted) {
    if ($StdoutTask.IsFaulted -or $StdoutTask.IsCanceled) { throw 'stdout copy did not complete successfully' }
    $State.StdoutCompleted = $true
    Set-HumCaptureFlag $CaptureDirectory 'stdout_completion_observed.txt' $true
  }
  if (-not $State.StderrCompleted -and $StderrTask.IsCompleted) {
    if ($StderrTask.IsFaulted -or $StderrTask.IsCanceled) { throw 'stderr copy did not complete successfully' }
    $State.StderrCompleted = $true
    Set-HumCaptureFlag $CaptureDirectory 'stderr_completion_observed.txt' $true
  }
  if (-not $State.JobQuiescent) {
    $Active = if ($WindowsPlatform -and $State.Assigned) {
      [UInt64] [HumFastJobNative]::ActiveProcessCount($JobHandle)
    } elseif ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted) {
      [UInt64] 0
    } else {
      [UInt64] 1
    }
    $State.FinalActive = $Active
    if ($Active -eq 0) {
      $State.JobQuiescent = $true
      Set-HumCaptureFlag $CaptureDirectory 'job_quiescence_observed.txt' $true
      Set-HumDurableText (Join-Path $CaptureDirectory 'final_active_process_count.txt') '0'
    }
  }
}

function Request-HumCaptureTermination {
  param(
    [object] $State,
    [bool] $WindowsPlatform,
    [object] $NativeProcess,
    [System.Diagnostics.Process] $ManagedProcess,
    [IntPtr] $JobHandle,
    [string] $CaptureDirectory,
    [string] $Reason
  )
  if ($State.TerminationCount -ne 0) { throw 'termination requested more than once' }
  $EvidenceError = $null
  if ($Reason -ceq 'deadline' -and $WindowsPlatform -and $State.Assigned -and [HumFastJobNative]::ActiveProcessCount($JobHandle) -ne 0) {
    try {
      $State.Pretermination = Get-HumActiveDescendantRecord $JobHandle ([UInt64]$NativeProcess.ProcessId)
      Set-HumDurableText (Join-Path $CaptureDirectory 'final_descendant_tree.txt') ("pretermination_pending;" + $State.Pretermination)
    } catch { $EvidenceError = $_.Exception }
  }
  $State.TerminationCount = 1
  Set-HumCaptureFlag $CaptureDirectory 'termination_requested.txt' $true
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_count.txt') '1'
  Set-HumDurableText (Join-Path $CaptureDirectory 'kill_attempt_count.txt') '1'
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_disposition.txt') 'requested'
  $Succeeded = if ($WindowsPlatform -and $State.Assigned) {
    [HumFastJobNative]::KillJob($JobHandle)
  } elseif ($WindowsPlatform) {
    [HumFastJobNative]::KillPrimary($NativeProcess.ProcessHandle)
  } else {
    try {
      $KillTree = $ManagedProcess.GetType().GetMethod('Kill', [type[]] @([bool]))
      if ($null -ne $KillTree) { $KillTree.Invoke($ManagedProcess, @($true)) | Out-Null }
      else { $ManagedProcess.Kill() }
      $true
    } catch { $false }
  }
  $State.TerminationResult = if ($Succeeded) { "${Reason}_requested" } else { "${Reason}_request_failed" }
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_result.txt') $State.TerminationResult
  if ($null -ne $EvidenceError) { throw $EvidenceError }
  $Succeeded
}

function Invoke-HumBinaryCapture {
  param(
    [Parameter(Mandatory = $true)][string] $FilePath,
    [Parameter(Mandatory = $true)][string[]] $Arguments,
    [Parameter(Mandatory = $true)][string] $WorkingDirectory,
    [Parameter(Mandatory = $true)][string] $CaptureDirectory,
    [ValidateRange(1, 86400)][int] $TimeoutSeconds = 60,
    [ValidateRange(1, 30)][int] $TerminationGraceSeconds = 3,
    [ValidateSet('', 'job_create', 'job_configure', 'process_create', 'assignment', 'resume')]
    [string] $InjectedSetupFailure = '',
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[a-z0-9][a-z0-9_-]{0,63}$')]
    [string] $CaseName
  )
  if ([System.IO.Directory]::Exists($CaptureDirectory) -or [System.IO.File]::Exists($CaptureDirectory)) {
    throw "capture path already exists: $CaptureDirectory"
  }
  [System.IO.Directory]::CreateDirectory($CaptureDirectory) | Out-Null
  $StartedUtc = [DateTime]::UtcNow
  $Timer = [System.Diagnostics.Stopwatch]::StartNew()
  $Frequency = [Int64] [System.Diagnostics.Stopwatch]::Frequency
  $DeadlineTicks = [Int64] $TimeoutSeconds * $Frequency
  $GraceTicks = [Int64] $TerminationGraceSeconds * $Frequency
  $WindowsPlatform = $env:OS -eq 'Windows_NT'
  Initialize-HumCaptureRecord $CaptureDirectory $CaseName $(if ($WindowsPlatform) { 'windows_job' } else { 'process_tree' }) $StartedUtc $DeadlineTicks $GraceTicks

  $Encoding = New-Object System.Text.UTF8Encoding($false, $true)
  $StdoutPath = Join-Path $CaptureDirectory 'stdout.bin'
  $StderrPath = Join-Path $CaptureDirectory 'stderr.bin'
  $Share = [System.IO.FileShare]::Read
  $StdoutCapture = New-Object System.IO.FileStream($StdoutPath, 'CreateNew', 'Write', $Share)
  $StderrCapture = New-Object System.IO.FileStream($StderrPath, 'CreateNew', 'Write', $Share)
  $JobHandle = [IntPtr]::Zero
  $NativeProcess = $null
  $ManagedProcess = $null
  $StdoutRead = $null
  $StderrRead = $null
  $StdoutTask = $null
  $StderrTask = $null
  $State = [pscustomobject]@{
    ProcessCreated = $false
    Assigned = $false
    Resumed = $false
    PrimaryExited = $false
    StdoutCompleted = $false
    StderrCompleted = $false
    JobQuiescent = $false
    FinalActive = $null
    ExitCode = $null
    TerminationCount = 0
    TerminationResult = 'not_requested'
    Pretermination = ''
  }
  $TimedOut = $false
  $DeadlineDisposition = 'prelaunch_failure'
  $TerminationDisposition = 'not_launched'
  $FinalTree = 'no_process_created'
  $CaptureError = ''
  $VctipGraceDeadline = $null

  try {
    if ($WindowsPlatform) {
      Set-HumCaptureFlag $CaptureDirectory 'job_creation_attempted.txt' $true
      if ($InjectedSetupFailure -ceq 'job_create') { throw 'injected Job creation failure' }
      $JobHandle = [HumFastJobNative]::CreateConfiguredJob()
      Set-HumCaptureFlag $CaptureDirectory 'job_creation_succeeded.txt' $true
      if ($InjectedSetupFailure -ceq 'job_configure') { throw 'injected Job configuration failure' }
      [HumFastJobNative]::ConfigureKillOnClose($JobHandle)
      Set-HumCaptureFlag $CaptureDirectory 'job_kill_on_close_configured.txt' $true
    }
    if ($Timer.ElapsedTicks -ge $DeadlineTicks) { throw 'absolute deadline expired before process creation' }
    Set-HumCaptureFlag $CaptureDirectory 'child_creation_attempted.txt' $true
    Set-HumCaptureFlag $CaptureDirectory 'process_creation_attempted.txt' $true
    if ($InjectedSetupFailure -ceq 'process_create') { throw 'injected process creation failure' }

    if ($WindowsPlatform) {
      $CommandLine = ((@($FilePath) + $Arguments | ForEach-Object { ConvertTo-HumNativeArgument $_ }) -join ' ')
      $NativeProcess = [HumFastJobNative]::CreateSuspended($FilePath, $CommandLine, $WorkingDirectory)
      $State.ProcessCreated = $true
      Set-HumCaptureFlag $CaptureDirectory 'process_creation_succeeded.txt' $true
      Set-HumCaptureFlag $CaptureDirectory 'process_created_suspended.txt' $true
      Set-HumCaptureFlag $CaptureDirectory 'launch_succeeded.txt' $true
      Set-HumDurableText (Join-Path $CaptureDirectory 'pid.txt') ([string] $NativeProcess.ProcessId)
      $StdoutRead = [HumFastJobNative]::OpenAsyncReadStream($NativeProcess.StdoutReadHandle)
      $NativeProcess.StdoutReadHandle = [IntPtr]::Zero
      $StderrRead = [HumFastJobNative]::OpenAsyncReadStream($NativeProcess.StderrReadHandle)
      $NativeProcess.StderrReadHandle = [IntPtr]::Zero
      $StdoutTask = $StdoutRead.CopyToAsync($StdoutCapture)
      $StderrTask = $StderrRead.CopyToAsync($StderrCapture)

      if ($Timer.ElapsedTicks -ge $DeadlineTicks) { throw 'absolute deadline expired before Job assignment' }
      Set-HumCaptureFlag $CaptureDirectory 'job_assignment_attempted.txt' $true
      if ($InjectedSetupFailure -ceq 'assignment') { throw 'injected Job assignment failure' }
      [HumFastJobNative]::Assign($JobHandle, $NativeProcess.ProcessHandle)
      $State.Assigned = $true
      Set-HumCaptureFlag $CaptureDirectory 'job_assignment_succeeded.txt' $true
      if ($Timer.ElapsedTicks -ge $DeadlineTicks) { throw 'absolute deadline expired before primary-thread resume' }
      Set-HumCaptureFlag $CaptureDirectory 'resume_attempted.txt' $true
      if ($InjectedSetupFailure -ceq 'resume') { throw 'injected primary-thread resume failure' }
      [HumFastJobNative]::Resume($NativeProcess.ThreadHandle)
      $State.Resumed = $true
      Set-HumCaptureFlag $CaptureDirectory 'resume_succeeded.txt' $true
      [HumFastJobNative]::CloseHandle($NativeProcess.ThreadHandle) | Out-Null
      $NativeProcess.ThreadHandle = [IntPtr]::Zero
    } else {
      $ManagedProcess = New-Object System.Diagnostics.Process
      $ManagedProcess.StartInfo = New-Object System.Diagnostics.ProcessStartInfo
      $ManagedProcess.StartInfo.FileName = $FilePath
      $ManagedProcess.StartInfo.Arguments = (($Arguments | ForEach-Object { ConvertTo-HumNativeArgument $_ }) -join ' ')
      $ManagedProcess.StartInfo.WorkingDirectory = $WorkingDirectory
      $ManagedProcess.StartInfo.UseShellExecute = $false
      $ManagedProcess.StartInfo.CreateNoWindow = $true
      $ManagedProcess.StartInfo.RedirectStandardOutput = $true
      $ManagedProcess.StartInfo.RedirectStandardError = $true
      $State.ProcessCreated = $ManagedProcess.Start()
      if (-not $State.ProcessCreated) { throw 'process creation returned false' }
      Set-HumCaptureFlag $CaptureDirectory 'process_creation_succeeded.txt' $true
      Set-HumCaptureFlag $CaptureDirectory 'launch_succeeded.txt' $true
      Set-HumDurableText (Join-Path $CaptureDirectory 'pid.txt') ([string] $ManagedProcess.Id)
      $StdoutTask = $ManagedProcess.StandardOutput.BaseStream.CopyToAsync($StdoutCapture)
      $StderrTask = $ManagedProcess.StandardError.BaseStream.CopyToAsync($StderrCapture)
    }

    while ($true) {
      Update-HumCaptureObservation $State $WindowsPlatform $NativeProcess $ManagedProcess $JobHandle $StdoutTask $StderrTask $CaptureDirectory
      if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent) {
        $DeadlineDisposition = 'completed_before_deadline'
        $FinalTree = 'quiescent'
        break
      }
      if ($WindowsPlatform -and $State.Assigned -and $State.PrimaryExited -and $State.ExitCode -eq 0 -and
          $State.StdoutCompleted -and $State.StderrCompleted -and -not $State.JobQuiescent) {
        if ($null -eq $VctipGraceDeadline) { $VctipGraceDeadline = $Timer.ElapsedTicks + (5 * $Frequency) }
        if ($Timer.ElapsedTicks -ge $VctipGraceDeadline) {
          $Root = [Environment]::GetEnvironmentVariable('VCToolsInstallDir')
          if ([string]::IsNullOrWhiteSpace($Root) -or -not [IO.Path]::IsPathRooted($Root) -or [IO.Path]::GetFullPath($Root) -cne $Root) { throw 'vctip_auxiliary: authenticated MSVC root unavailable' }
          $null = Complete-HumAuthenticatedVctip $State $NativeProcess $JobHandle $CaptureDirectory $Root
          $TerminationDeadline = $Timer.ElapsedTicks + $GraceTicks
          while ($Timer.ElapsedTicks -lt $TerminationDeadline) {
            Update-HumCaptureObservation $State $WindowsPlatform $NativeProcess $ManagedProcess $JobHandle $StdoutTask $StderrTask $CaptureDirectory
            if ($State.JobQuiescent) { break }
            Start-Sleep -Milliseconds (Get-HumRemainingMilliseconds $Timer $TerminationDeadline)
          }
          if (-not $State.JobQuiescent -or $State.FinalActive -ne 0) { throw 'vctip_auxiliary: Job termination did not reach final zero' }
          $DeadlineDisposition = 'completed_after_authenticated_vctip_termination'
          $TerminationDisposition = 'authenticated_vctip_termination_confirmed'
          $FinalTree = 'terminated_quiescent;pretermination=authenticated_vctip_auxiliary'
          break
        }
      }
      $Remaining = Get-HumRemainingMilliseconds $Timer $DeadlineTicks
      if ($Remaining -eq 0) { break }
      if ($WindowsPlatform -and -not $State.PrimaryExited) {
        $null = [HumFastJobNative]::Wait($NativeProcess.ProcessHandle, [UInt32] $Remaining)
      } else { Start-Sleep -Milliseconds $Remaining }
    }

    if ($DeadlineDisposition -ne 'completed_before_deadline' -and
        $DeadlineDisposition -ne 'completed_after_authenticated_vctip_termination') {
      $TimedOut = $true
      $DeadlineDisposition = 'deadline_expired'
      Set-HumCaptureFlag $CaptureDirectory 'timed_out.txt' $true
      Set-HumDurableText (Join-Path $CaptureDirectory 'deadline_disposition.txt') $DeadlineDisposition
      $null = Request-HumCaptureTermination $State $WindowsPlatform $NativeProcess $ManagedProcess $JobHandle $CaptureDirectory 'deadline'
      $GraceDeadline = $Timer.ElapsedTicks + $GraceTicks
      while ($Timer.ElapsedTicks -lt $GraceDeadline) {
        Update-HumCaptureObservation $State $WindowsPlatform $NativeProcess $ManagedProcess $JobHandle $StdoutTask $StderrTask $CaptureDirectory
        if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent) { break }
        $Remaining = Get-HumRemainingMilliseconds $Timer $GraceDeadline
        if ($Remaining -eq 0) { break }
        if ($WindowsPlatform -and -not $State.PrimaryExited) {
          $null = [HumFastJobNative]::Wait($NativeProcess.ProcessHandle, [UInt32] $Remaining)
        } else { Start-Sleep -Milliseconds $Remaining }
      }
      if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent) {
        $TerminationDisposition = 'tree_termination_confirmed'
        $State.TerminationResult = 'job_terminated_quiescent'
        $FinalTree = if ($State.Pretermination) { 'terminated_quiescent;' + $State.Pretermination } else { 'terminated_quiescent;pretermination=quiescent_race' }
      } else {
        $TerminationDisposition = 'tree_termination_failed'
        $State.TerminationResult = 'job_termination_unconfirmed'
        $FinalTree = 'unconfirmed'
        throw 'capture did not quiesce inside termination grace'
      }
    }
  } catch {
    $CaptureError = $_.Exception.ToString()
    if (-not $State.ProcessCreated) {
      Set-HumDurableBytes (Join-Path $CaptureDirectory 'launch_error.bin') $Encoding.GetBytes($CaptureError)
      $DeadlineDisposition = if ($Timer.ElapsedTicks -ge $DeadlineTicks) { 'deadline_expired_prelaunch' } else { 'prelaunch_failure' }
      $CaptureError = ''
    } else {
      $DeadlineExpiredAfterCreation = $Timer.ElapsedTicks -ge $DeadlineTicks
      if ($DeadlineExpiredAfterCreation) {
        $TimedOut = $true
        $DeadlineDisposition = 'deadline_expired'
        Set-HumCaptureFlag $CaptureDirectory 'timed_out.txt' $true
        Set-HumDurableText (Join-Path $CaptureDirectory 'deadline_disposition.txt') $DeadlineDisposition
      }
      if ($State.TerminationCount -eq 0) {
        $TerminationReason = if ($DeadlineExpiredAfterCreation) { 'deadline' } else { 'setup_failure' }
        try { $null = Request-HumCaptureTermination $State $WindowsPlatform $NativeProcess $ManagedProcess $JobHandle $CaptureDirectory $TerminationReason }
        catch { $CaptureError += "`n" + $_.Exception.ToString() }
      }
      $GraceDeadline = $Timer.ElapsedTicks + $GraceTicks
      while ($Timer.ElapsedTicks -lt $GraceDeadline) {
        try { Update-HumCaptureObservation $State $WindowsPlatform $NativeProcess $ManagedProcess $JobHandle $StdoutTask $StderrTask $CaptureDirectory }
        catch { $CaptureError += "`n" + $_.Exception.ToString(); break }
        if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent) { break }
        $Remaining = Get-HumRemainingMilliseconds $Timer $GraceDeadline
        if ($Remaining -eq 0) { break }
        Start-Sleep -Milliseconds $Remaining
      }
      if ($State.PrimaryExited -and $State.StdoutCompleted -and $State.StderrCompleted -and $State.JobQuiescent) {
        $TerminationDisposition = 'tree_termination_confirmed'
        $State.TerminationResult = if ($State.Assigned) { 'job_terminated_quiescent' } else { 'unassigned_primary_terminated' }
        $FinalTree = if ($State.Assigned) { if ($State.Pretermination) { 'terminated_quiescent;' + $State.Pretermination } else { 'terminated_quiescent;pretermination=quiescent_race' } } else { 'suspended_unassigned_terminated' }
      } else {
        $TerminationDisposition = 'tree_termination_failed'
        $State.TerminationResult = 'setup_termination_unconfirmed'
        $FinalTree = 'unconfirmed'
      }
      if (-not $DeadlineExpiredAfterCreation) { $DeadlineDisposition = 'setup_failure' }
    }
  } finally {
    if ($null -ne $StdoutRead) { try { $StdoutRead.Dispose() } catch {} }
    if ($null -ne $StderrRead) { try { $StderrRead.Dispose() } catch {} }
    if ($null -ne $ManagedProcess) {
      try { $ManagedProcess.StandardOutput.Close() } catch {}
      try { $ManagedProcess.StandardError.Close() } catch {}
      $ManagedProcess.Dispose()
    }
    if ($null -ne $NativeProcess) {
      foreach ($Name in @('ThreadHandle', 'StdoutReadHandle', 'StderrReadHandle', 'ProcessHandle')) {
        if ($NativeProcess.$Name -ne [IntPtr]::Zero) {
          [HumFastJobNative]::CloseHandle($NativeProcess.$Name) | Out-Null
          $NativeProcess.$Name = [IntPtr]::Zero
        }
      }
    }
    if ($JobHandle -ne [IntPtr]::Zero) {
      if ($State.ProcessCreated -and $State.Assigned -and -not $State.JobQuiescent -and $State.TerminationCount -eq 0) {
        $CaptureError += "`nJob handle reached cleanup without recorded quiescence or termination."
      }
      [HumFastJobNative]::CloseHandle($JobHandle) | Out-Null
    }
    try { $StdoutCapture.Flush($true) } catch { $CaptureError += "`n" + $_.Exception.ToString() }
    try { $StderrCapture.Flush($true) } catch { $CaptureError += "`n" + $_.Exception.ToString() }
    $StdoutCapture.Dispose()
    $StderrCapture.Dispose()
    $Timer.Stop()
  }

  $CompletedUtc = [DateTime]::UtcNow
  Set-HumDurableText (Join-Path $CaptureDirectory 'completed_utc.txt') $CompletedUtc.ToString('o', [Globalization.CultureInfo]::InvariantCulture)
  Set-HumDurableText (Join-Path $CaptureDirectory 'duration_ticks.txt') $Timer.ElapsedTicks.ToString([Globalization.CultureInfo]::InvariantCulture)
  Set-HumDurableText (Join-Path $CaptureDirectory 'exit.txt') $(if ($null -eq $State.ExitCode) { 'null' } else { [string] $State.ExitCode })
  Set-HumCaptureFlag $CaptureDirectory 'primary_exit_observed.txt' $State.PrimaryExited
  Set-HumCaptureFlag $CaptureDirectory 'stdout_completion_observed.txt' $State.StdoutCompleted
  Set-HumCaptureFlag $CaptureDirectory 'stderr_completion_observed.txt' $State.StderrCompleted
  Set-HumCaptureFlag $CaptureDirectory 'job_quiescence_observed.txt' $State.JobQuiescent
  Set-HumDurableText (Join-Path $CaptureDirectory 'final_active_process_count.txt') $(if ($null -eq $State.FinalActive) { 'null' } else { [string] $State.FinalActive })
  Set-HumDurableText (Join-Path $CaptureDirectory 'deadline_disposition.txt') $DeadlineDisposition
  Set-HumCaptureFlag $CaptureDirectory 'timed_out.txt' $TimedOut
  Set-HumCaptureFlag $CaptureDirectory 'termination_requested.txt' ($State.TerminationCount -eq 1)
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_disposition.txt') $(if ($State.TerminationCount -eq 0) { if ($State.ProcessCreated) { 'not_requested' } else { 'not_launched' } } else { $TerminationDisposition })
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_result.txt') $(if ($State.TerminationCount -eq 0) { 'not_requested' } else { $State.TerminationResult })
  Set-HumDurableText (Join-Path $CaptureDirectory 'termination_count.txt') ([string] $State.TerminationCount)
  Set-HumDurableText (Join-Path $CaptureDirectory 'kill_attempt_count.txt') ([string] $State.TerminationCount)
  Set-HumDurableText (Join-Path $CaptureDirectory 'final_descendant_tree.txt') $FinalTree
  Set-HumDurableText (Join-Path $CaptureDirectory 'completion_count.txt') $(if ($State.PrimaryExited) { '1' } else { '0' })
  Set-HumDurableBytes (Join-Path $CaptureDirectory 'capture_error.bin') $Encoding.GetBytes($CaptureError.TrimStart("`r", "`n"))

  try {
    $TerminalFacts = Get-HumTerminalFacts ([System.IO.File]::ReadAllBytes($StdoutPath))
    Set-HumDurableBytes (Join-Path $CaptureDirectory 'terminal_stdout_line.bin') $Encoding.GetBytes($TerminalFacts.Terminal)
    Set-HumDurableText (Join-Path $CaptureDirectory 'success_marker_count.txt') ([string] $TerminalFacts.MarkerCount)
  } catch {
    Set-HumDurableBytes (Join-Path $CaptureDirectory 'terminal_stdout_line.bin') ([byte[]] @())
    Set-HumDurableText (Join-Path $CaptureDirectory 'success_marker_count.txt') '0'
    $Existing = [System.IO.File]::ReadAllBytes((Join-Path $CaptureDirectory 'capture_error.bin'))
    Set-HumDurableBytes (Join-Path $CaptureDirectory 'capture_error.bin') ($Existing + $Encoding.GetBytes($_.Exception.ToString()))
  }
  Write-HumCaptureManifest $CaptureDirectory
  Read-HumCaptureRecord $CaptureDirectory
}

function Assert-HumCaptureComplete {
  param([object] $Result)
  $Retained = Read-HumCaptureRecord $Result.CaptureDirectory
  if (-not $Retained.Launched -or $null -eq $Retained.ExitCode) {
    $Diagnostic = Get-HumCaptureFailureDiagnostic $Retained
    throw "capture has no launched terminal child`n$Diagnostic"
  }
  if ($Retained.CompletionCount -ne 1 -or $Retained.CaptureErrorBytes -ne 0 -or
      -not $Retained.PrimaryExitObserved -or -not $Retained.StdoutCompletionObserved -or
      -not $Retained.StderrCompletionObserved -or -not $Retained.JobQuiescenceObserved) {
    throw 'capture transport is incomplete'
  }
  if ($Retained.ContainmentKind -ceq 'windows_job' -and
      (-not $Retained.ProcessCreatedSuspended -or
       -not $Retained.JobAssignmentSucceeded -or -not $Retained.ResumeSucceeded)) {
    throw 'capture child was not resumed from durable Windows containment'
  }
  $Retained
}

function Get-HumCaptureFailureDiagnostic {
  param([object] $Result)
  $Retained = Read-HumCaptureRecord $Result.CaptureDirectory
  $LaunchIdentity = Get-HumFileIdentity $Retained.LaunchErrorPath
  $CaptureIdentity = Get-HumFileIdentity $Retained.CaptureErrorPath
  $LaunchBytes = [System.IO.File]::ReadAllBytes($Retained.LaunchErrorPath)
  $CaptureBytes = [System.IO.File]::ReadAllBytes($Retained.CaptureErrorPath)
  @(
    'hum_capture_failure_v1'
    "case_name=$($Retained.CaseName)"
    "capture_directory=$($Retained.CaptureDirectory)"
    "containment_kind=$($Retained.ContainmentKind)"
    "process_creation_attempted=$($Retained.ProcessCreationAttempted.ToString().ToLowerInvariant())"
    "process_creation_succeeded=$($Retained.ProcessCreationSucceeded.ToString().ToLowerInvariant())"
    "launch_succeeded=$($Retained.Launched.ToString().ToLowerInvariant())"
    "pid=$(if ($null -eq $Retained.Pid) { 'null' } else { $Retained.Pid })"
    "completion_count=$($Retained.CompletionCount)"
    "exit=$(if ($null -eq $Retained.ExitCode) { 'null' } else { $Retained.ExitCode })"
    "deadline_disposition=$($Retained.DeadlineDisposition)"
    "timed_out=$($Retained.TimedOut.ToString().ToLowerInvariant())"
    "termination_disposition=$($Retained.TerminationDisposition)"
    "termination_count=$($Retained.TerminationCount)"
    "launch_error_bytes=$($LaunchIdentity.Bytes)"
    "launch_error_sha256=$($LaunchIdentity.Sha256)"
    "launch_error_base64=$([Convert]::ToBase64String($LaunchBytes))"
    "capture_error_bytes=$($CaptureIdentity.Bytes)"
    "capture_error_sha256=$($CaptureIdentity.Sha256)"
    "capture_error_base64=$([Convert]::ToBase64String($CaptureBytes))"
  ) -join "`n"
}

function Remove-HumCaptureAfterAuthentication {
  param([string] $CaptureDirectory)
  $null = Read-HumCaptureRecord $CaptureDirectory
  $Resolved = [System.IO.Path]::GetFullPath($CaptureDirectory)
  Remove-Item -LiteralPath $Resolved -Recurse -Force
  if ([System.IO.Directory]::Exists($Resolved) -or [System.IO.File]::Exists($Resolved)) {
    throw "capture cleanup failed: $Resolved"
  }
}
function Invoke-HumContainedRustNativeCapture {
  param([string] $Label, [string] $Cargo, [string[]] $Arguments)
  $Root = Join-Path ([IO.Path]::GetTempPath()) ('hum-contained-rust-' + [Guid]::NewGuid().ToString('N'))
  $Capture = Join-Path $Root 'capture'; [void] [IO.Directory]::CreateDirectory($Root); $Authenticated = $false
  try {
    $Result = Invoke-HumBinaryCapture $Cargo $Arguments (Get-Location).Path $Capture 120 -CaseName ($Label.ToLowerInvariant() -replace '[^a-z0-9_-]', '-')
    $Result = Assert-HumCaptureComplete $Result
    $Authenticated = $true
    $Encoding = New-Object Text.UTF8Encoding($false, $true)
    $Text = $Encoding.GetString([IO.File]::ReadAllBytes($Result.StdoutPath)) +
      $Encoding.GetString([IO.File]::ReadAllBytes($Result.StderrPath))
    [pscustomobject] @{ Output = @([regex]::Split($Text, '\r\n|\n|\r')); ExitCode = $Result.ExitCode }
  } finally {
    if ($Authenticated) {
      Remove-HumCaptureAfterAuthentication $Capture
      Remove-Item -LiteralPath $Root -Force
      if ([IO.Directory]::Exists($Root)) { throw 'contained Rust capture cleanup failed' }
    }
  }
}
function Invoke-HumContainedExactRustTest {
  param([string] $Label, [string] $Cargo, [string] $Selector)
  Write-Host "==> $Label"
  Assert-ExactRustSelectorSyntax $Selector
  $List = Invoke-HumContainedRustNativeCapture "$Label-list" $Cargo @('test', $Selector, '--', '--exact', '--list')
  if ($List.ExitCode -ne 0) { $List.Output | ForEach-Object { Write-Host $_ }; throw "$Label could not list '$Selector'; cargo exited $($List.ExitCode)" }
  $Escaped = [regex]::Escape($Selector); $Listed = @($List.Output | Where-Object { $_ -match ': test$' }); $Exact = @($Listed | Where-Object { $_ -match "^${Escaped}: test$" })
  if ($Listed.Count -ne 1 -or $Exact.Count -ne 1) { throw "$Label must resolve '$Selector' to exactly one test before execution" }
  $Run = Invoke-HumContainedRustNativeCapture "$Label-run" $Cargo @('test', $Selector, '--', '--exact')
  $Run.Output | ForEach-Object { Write-Host $_ }
  if ($Run.ExitCode -ne 0) { throw "$Label failed with exit code $($Run.ExitCode)" }
  Assert-ExactRustSelectorEvidence $Selector $List.Output $Run.Output
  $script:ExactRustSelectorCredits.Add($Selector)
}

function Get-HumExecutableIdentity {
  param([string] $Path, [switch] $AllowHardLinks)
  $Full = [IO.Path]::GetFullPath($Path); $Item = Get-Item -LiteralPath $Full -Force -ErrorAction Stop
  if ($Item.PSIsContainer -or ($Item.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw "isolated executable is not an ordinary non-reparse file: $Full" }
  $Native = [HumFastJobNative]::GetFileIdentity($Full, $script:HumHostIsWindows)
  if ($Native[2] -ne 1 -and -not $AllowHardLinks) { throw "isolated executable is not one ordinary non-linked file: $Full" }
  if (-not $script:HumHostIsWindows -and ([IO.File]::GetUnixFileMode($Full) -band ([IO.UnixFileMode]::UserExecute -bor [IO.UnixFileMode]::GroupExecute -bor [IO.UnixFileMode]::OtherExecute)) -eq 0) { throw "isolated executable mode is not executable: $Full" }
  [pscustomobject] @{ Path = $Full; FileIdentity = ('{0:x16}:{1:x16}' -f $Native[0], $Native[1]); Links = $Native[2]; Bytes = $Item.Length; Sha256 = (Get-FileHash -LiteralPath $Full -Algorithm SHA256).Hash.ToLowerInvariant() }
}
function Assert-HumIsolatedExecutable {
  param([pscustomobject] $Record, [switch] $RequireSource)
  $Comparison = if ($script:HumHostIsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
  $Candidate = [IO.Path]::GetFullPath($Record.Directory); $Target = [IO.Path]::GetFullPath($Record.TargetRoot); if ($Candidate.Equals($Target, $Comparison) -or $Candidate.StartsWith($Target + [IO.Path]::DirectorySeparatorChar, $Comparison)) { throw 'target-tree directory identity' }
  $Directory = Get-Item -LiteralPath $Record.Directory -Force -ErrorAction Stop; $LinkType = $Directory.PSObject.Properties['LinkType']
  if (-not $Directory.PSIsContainer -or ($Directory.Attributes -band [IO.FileAttributes]::ReparsePoint) -or ($null -ne $LinkType -and -not [string]::IsNullOrEmpty([string] $LinkType.Value))) { throw 'isolated execution directory identity failed' }
  $Entries = @(Get-ChildItem -LiteralPath $Record.Directory -Force -ErrorAction Stop); if ($Entries.Count -ne 1 -or [IO.Path]::GetFullPath($Entries[0].FullName) -cne $Record.Executable) { throw 'isolated execution directory shape failed' }
  if ($RequireSource) { $Source = Get-HumExecutableIdentity $Record.Source -AllowHardLinks; if ($Source.FileIdentity -cne $Record.SourceFileIdentity -or $Source.Bytes -ne $Record.Bytes -or $Source.Sha256 -cne $Record.Sha256) { throw 'canonical executable identity changed during transport' } }
  $Copy = Get-HumExecutableIdentity $Record.Executable -AllowHardLinks
  if ($Copy.FileIdentity -ceq $Record.SourceFileIdentity) { throw 'isolated executable aliases canonical source' }
  if ($Copy.Links -ne 1) { throw 'isolated executable is not one ordinary non-linked file' }
  if ($Copy.FileIdentity -cne $Record.ExecutableFileIdentity -or $Copy.Bytes -ne $Record.Bytes -or $Copy.Sha256 -cne $Record.Sha256) { throw 'isolated executable byte identity failed' }
}
function New-HumIsolatedExecutable {
  param([string] $Source, [string] $ScratchRoot, [string] $CargoTargetRoot)
  $Target = (Get-Item -LiteralPath ([IO.Path]::GetFullPath($CargoTargetRoot)) -Force -ErrorAction Stop).FullName; $Comparison = if ($script:HumHostIsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
  $Expected = Join-Path $Target $(if ($script:HumHostIsWindows) { 'debug\hum-dev.exe' } else { 'debug/hum-dev' }); if (-not [IO.Path]::GetFullPath($Source).Equals($Expected, $Comparison)) { throw 'canonical Cargo executable path identity failed' }
  $SourceIdentity = Get-HumExecutableIdentity $Expected -AllowHardLinks; $Directory = Join-Path (Get-Item -LiteralPath ([IO.Path]::GetFullPath($ScratchRoot)) -Force -ErrorAction Stop).FullName ('hum-dev-exec-' + [Guid]::NewGuid().ToString('N'))
  if ($Directory.Equals($Target, $Comparison) -or $Directory.StartsWith($Target + [IO.Path]::DirectorySeparatorChar, $Comparison)) { throw 'target-tree directory identity' }; if ([IO.Directory]::Exists($Directory) -or [IO.File]::Exists($Directory)) { throw 'isolated execution directory already exists' }
  [void] [IO.Directory]::CreateDirectory($Directory); $Executable = Join-Path $Directory ([IO.Path]::GetFileName($SourceIdentity.Path))
  try { [IO.File]::Copy($SourceIdentity.Path, $Executable, $false); $After = Get-HumExecutableIdentity $SourceIdentity.Path -AllowHardLinks; if ($After.FileIdentity -cne $SourceIdentity.FileIdentity -or $After.Bytes -ne $SourceIdentity.Bytes -or $After.Sha256 -cne $SourceIdentity.Sha256) { throw 'canonical executable identity changed during transport' }; $Copy = Get-HumExecutableIdentity $Executable -AllowHardLinks; $Record = [pscustomobject] @{ Source = $SourceIdentity.Path; SourceFileIdentity = $SourceIdentity.FileIdentity; Executable = $Executable; ExecutableFileIdentity = $Copy.FileIdentity; Directory = $Directory; TargetRoot = $Target; Bytes = $SourceIdentity.Bytes; Sha256 = $SourceIdentity.Sha256 }; Assert-HumIsolatedExecutable $Record -RequireSource; $Record }
  catch { if ([IO.File]::Exists($Executable)) { Remove-Item -LiteralPath $Executable -Force }; if ([IO.Directory]::Exists($Directory)) { Remove-Item -LiteralPath $Directory -Force }; throw }
}
function Remove-HumIsolatedExecutable {
  param([pscustomobject] $Record); Assert-HumIsolatedExecutable $Record
  Remove-Item -LiteralPath $Record.Executable -Force; Remove-Item -LiteralPath $Record.Directory -Force
  if ([IO.File]::Exists($Record.Executable) -or [IO.Directory]::Exists($Record.Directory)) { throw 'isolated executable cleanup failed' }
}
if ($MyInvocation.InvocationName -ne '.') {
  $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
  if ($ScratchRoot -eq '') {
    $ScratchRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("hum-fast-evidence-" + [Guid]::NewGuid().ToString('N'))
  }
  $PowerShellApplications = @(Get-Command pwsh -CommandType Application -All -ErrorAction Stop)
  if ($PowerShellApplications.Count -ne 1) { throw "Fast producer requires exactly one PowerShell 7 application, found $($PowerShellApplications.Count)" }
  $PowerShell = [IO.Path]::GetFullPath([string]$PowerShellApplications[0].Source)
  if (-not [IO.File]::Exists($PowerShell) -or ([IO.File]::GetAttributes($PowerShell) -band [IO.FileAttributes]::ReparsePoint)) { throw 'Fast producer PowerShell 7 identity is not one ordinary non-reparse file' }
  Write-Output "capture_directory=$ScratchRoot"
  $Result = Invoke-HumBinaryCapture $PowerShell @(
    '-NoLogo', '-NoProfile', '-NonInteractive', '-File', 'tools/check_all.ps1',
    '-EvidenceTier', 'Fast'
  ) $RepoRoot $ScratchRoot $TimeoutSeconds -CaseName 'production-fast'
  try {
    $Result = Assert-HumCaptureComplete $Result
    Write-Output (
      "launched={0};pid={1};exit={2};started_utc={3:o};completed_utc={4:o};duration_ticks={5};stopwatch_frequency={6};deadline={7};timed_out={8};termination={9};termination_count={10};stdout_bytes={11};stdout_sha256={12};stderr_bytes={13};stderr_sha256={14};terminal={15};success_markers={16};capture_directory={17}" -f
      $Result.Launched, $Result.Pid, $Result.ExitCode, $Result.StartedUtc,
      $Result.CompletedUtc, $Result.DurationTicks, $Result.StopwatchFrequency,
      $Result.DeadlineDisposition, $Result.TimedOut, $Result.TerminationDisposition,
      $Result.TerminationCount, $Result.StdoutBytes, $Result.StdoutSha256,
      $Result.StderrBytes, $Result.StderrSha256, $Result.TerminalStdoutLine,
      $Result.SuccessMarkerCount, $Result.CaptureDirectory
    )
    if ($Result.ExitCode -ne 0 -or $Result.TimedOut -or
        $Result.SuccessMarkerCount -ne 1 -or
        $Result.TerminalStdoutLine -cne $script:HumSuccessMarker) { exit 1 }
  } catch {
    Write-Error $_
    exit 1
  }
}
