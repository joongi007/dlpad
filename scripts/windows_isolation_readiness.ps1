$ErrorActionPreference = 'Stop'
$report = [ordered]@{ schema=1; scope='read-only capability discovery; no GUI/session/VM/system changes'; features=@(); get_vm_command=$false; existing_vm_count=$null; sessions=$null }
foreach ($name in @('Containers-DisposableClientVM','Microsoft-Hyper-V-All')) {
    try {
        $feature = Get-WindowsOptionalFeature -Online -FeatureName $name -ErrorAction Stop
        $report.features += @{name=$name; state=[string]$feature.State; query='observed'}
    } catch { $report.features += @{name=$name; query='unavailable'; state=$null} }
}
$command = Get-Command Get-VM -ErrorAction SilentlyContinue
$report.get_vm_command = $null -ne $command
if ($null -ne $command) {
    try { $report.existing_vm_count = @(Get-VM -ErrorAction Stop).Count }
    catch { $report.existing_vm_count = $null }
}
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class DlpadReadOnlySessions {
 [StructLayout(LayoutKind.Sequential)] public struct Session { public int Id; public IntPtr Name; public int State; }
 [DllImport("wtsapi32.dll", SetLastError=true)] static extern bool WTSEnumerateSessionsW(IntPtr server,int reserved,int version,out IntPtr data,out int count);
 [DllImport("wtsapi32.dll")] static extern void WTSFreeMemory(IntPtr data);
 public static int[] Counts() {
  IntPtr data;int count;
  if(!WTSEnumerateSessionsW(IntPtr.Zero,0,1,out data,out count)) return new int[]{-1,Marshal.GetLastWin32Error(),0};
  try { if(count<0||count>128)return new int[]{-2,0,0}; int active=0,connected=0;int size=Marshal.SizeOf(typeof(Session));
   for(int i=0;i<count;i++){ var s=(Session)Marshal.PtrToStructure(IntPtr.Add(data,i*size),typeof(Session));if(s.State==0)active++;if(s.State==1)connected++; }
   return new int[]{count,active,connected};
  } finally { WTSFreeMemory(data); }
 }
}
'@
$counts = [DlpadReadOnlySessions]::Counts()
$report.sessions = @{total=$counts[0]; active=$counts[1]; connected=$counts[2]; names_retained=$false; attached=$false}
$report | ConvertTo-Json -Depth 5 -Compress
