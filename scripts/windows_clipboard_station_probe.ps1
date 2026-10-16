param([Parameter(Mandatory=$true)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
if(Test-Path $OutputDirectory){throw 'Fresh owned output required'}
$sid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User
$acl=New-Object System.Security.AccessControl.DirectorySecurity
$acl.SetOwner($sid);$acl.SetAccessRuleProtection($true,$false)
$acl.AddAccessRule((New-Object System.Security.AccessControl.FileSystemAccessRule($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow')))
(New-Object System.IO.DirectoryInfo($OutputDirectory)).Create($acl)
$env:TMP=$OutputDirectory;$env:TEMP=$OutputDirectory
$source=@'
using System;using System.IO;using System.Collections.Generic;using System.Runtime.InteropServices;using System.Security.Principal;using System.Security.AccessControl;using System.Text;
class OwnedStation {
[StructLayout(LayoutKind.Sequential)]struct SA{public int length;public IntPtr descriptor;public int inherit;}
[StructLayout(LayoutKind.Sequential)]struct FLAGS{public int inherit,reserved;public uint flags;}
[DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)]static extern IntPtr CreateWindowStationW(string name,uint flags,uint rights,ref SA sa);
[DllImport("user32.dll",SetLastError=true)]static extern bool CloseWindowStation(IntPtr h);
[DllImport("user32.dll")]static extern IntPtr GetProcessWindowStation();
[DllImport("user32.dll",SetLastError=true)]static extern bool SetProcessWindowStation(IntPtr h);
[DllImport("kernel32.dll")]static extern uint GetCurrentThreadId();
[DllImport("user32.dll")]static extern IntPtr GetThreadDesktop(uint id);
[DllImport("user32.dll",SetLastError=true)]static extern bool SetThreadDesktop(IntPtr h);
[DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)]static extern IntPtr CreateDesktopW(string name,IntPtr device,IntPtr mode,uint flags,uint rights,ref SA sa);
[DllImport("user32.dll",SetLastError=true)]static extern bool CloseDesktop(IntPtr h);
[DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)]static extern bool GetUserObjectInformationW(IntPtr h,int index,IntPtr data,uint size,out uint need);
[DllImport("user32.dll",SetLastError=true)]static extern bool GetUserObjectSecurity(IntPtr h,ref uint info,byte[] sd,uint size,out uint need);
[DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)]static extern IntPtr CreateWindowExW(uint ex,string cls,string title,uint style,int x,int y,int w,int h,IntPtr parent,IntPtr menu,IntPtr instance,IntPtr param);
[DllImport("user32.dll",SetLastError=true)]static extern bool DestroyWindow(IntPtr h);
[DllImport("user32.dll",SetLastError=true)]static extern bool OpenClipboard(IntPtr h);
[DllImport("user32.dll",SetLastError=true)]static extern bool CloseClipboard();
[DllImport("user32.dll",SetLastError=true)]static extern bool EmptyClipboard();
[DllImport("user32.dll",SetLastError=true)]static extern IntPtr SetClipboardData(uint fmt,IntPtr data);
[DllImport("user32.dll",SetLastError=true)]static extern IntPtr GetClipboardData(uint fmt);
[DllImport("kernel32.dll",SetLastError=true)]static extern IntPtr GlobalAlloc(uint flags,UIntPtr size);
[DllImport("kernel32.dll",SetLastError=true)]static extern IntPtr GlobalLock(IntPtr h);
[DllImport("kernel32.dll",SetLastError=true)]static extern bool GlobalUnlock(IntPtr h);
[DllImport("kernel32.dll",SetLastError=true)]static extern UIntPtr GlobalSize(IntPtr h);
[DllImport("kernel32.dll")]static extern IntPtr GlobalFree(IntPtr h);
static List<string> fields=new List<string>();static string J(string s){return "\""+s.Replace("\\","\\\\").Replace("\"","\\\"").Replace("\r","\\r").Replace("\n","\\n")+"\"";}static void S(string k,string v){fields.Add(J(k)+":"+J(v));}static void B(string k,bool v){fields.Add(J(k)+":"+(v?"true":"false"));}static void Check(bool ok,string stage){if(!ok)throw new Exception(stage+" Win32="+Marshal.GetLastWin32Error());}
static string Name(IntPtr h){uint n;GetUserObjectInformationW(h,2,IntPtr.Zero,0,out n);if(n<2||n>4096)throw new Exception("name size invalid");IntPtr p=Marshal.AllocHGlobal((int)n);try{Check(GetUserObjectInformationW(h,2,p,n,out n),"name");return Marshal.PtrToStringUni(p);}finally{Marshal.FreeHGlobal(p);}}
static bool Nonvisible(IntPtr h){uint n;IntPtr p=Marshal.AllocHGlobal(Marshal.SizeOf(typeof(FLAGS)));try{Check(GetUserObjectInformationW(h,1,p,(uint)Marshal.SizeOf(typeof(FLAGS)),out n),"flags");return (((FLAGS)Marshal.PtrToStructure(p,typeof(FLAGS))).flags&1)==0;}finally{Marshal.FreeHGlobal(p);}}
static bool ExactAcl(IntPtr h,SecurityIdentifier sid,uint mask){uint info=5,n;GetUserObjectSecurity(h,ref info,null,0,out n);if(n==0||n>65536)throw new Exception("security size invalid");byte[] data=new byte[n];Check(GetUserObjectSecurity(h,ref info,data,n,out n),"security read");RawSecurityDescriptor sd=new RawSecurityDescriptor(data,0);if(!sid.Equals(sd.Owner)||(sd.ControlFlags&ControlFlags.DiscretionaryAclProtected)==0||sd.DiscretionaryAcl==null||sd.DiscretionaryAcl.Count!=1)return false;CommonAce ace=sd.DiscretionaryAcl[0] as CommonAce;return ace!=null&&ace.AceQualifier==AceQualifier.AccessAllowed&&sid.Equals(ace.SecurityIdentifier)&&unchecked((uint)ace.AccessMask)==mask;}
static SA Attributes(SecurityIdentifier sid,uint mask,out IntPtr memory){RawSecurityDescriptor sd=new RawSecurityDescriptor("O:"+sid.Value+"D:P(A;;0x"+mask.ToString("x")+";;;"+sid.Value+")");byte[] b=new byte[sd.BinaryLength];sd.GetBinaryForm(b,0);memory=Marshal.AllocHGlobal(b.Length);Marshal.Copy(b,0,memory,b.Length);SA sa=new SA();sa.length=Marshal.SizeOf(typeof(SA));sa.descriptor=memory;return sa;}
static void OwnedClipboard(IntPtr station,string name){IntPtr current=GetProcessWindowStation();Check(current==station,"owned station handle");Check(Name(current)==name&&name!="WinSta0"&&Nonvisible(current),"owned station evidence");}
static int Main(string[] args){IntPtr oldStation=GetProcessWindowStation(),oldDesktop=GetThreadDesktop(GetCurrentThreadId()),station=IntPtr.Zero,desktop=IntPtr.Zero,window=IntPtr.Zero,sd1=IntPtr.Zero,sd2=IntPtr.Zero,memory=IntPtr.Zero;bool clipboard=false,passed=false;string stage="create station";try{SecurityIdentifier sid=WindowsIdentity.GetCurrent().User;S("current_sid",sid.Value);const uint stationRights=0x2002f,desktopRights=0x200c7;SA sa=Attributes(sid,stationRights,out sd1);station=CreateWindowStationW(null,1,stationRights,ref sa);int createError=Marshal.GetLastWin32Error();S("create_error",createError.ToString());Check(station!=IntPtr.Zero,"CreateOnlyNULL");string name=Name(station);S("station_name",name);Check(!name.Equals("WinSta0",StringComparison.OrdinalIgnoreCase),"not WinSta0");Check(Nonvisible(station),"nonvisible");B("station_nonvisible",true);Check(ExactAcl(station,sid,stationRights),"protected current SID station ACL");B("station_acl_verified",true);stage="associate station";Check(SetProcessWindowStation(station),"SetProcessWindowStation");SA dsa=Attributes(sid,desktopRights,out sd2);string dname="owned-"+Guid.NewGuid().ToString("N");desktop=CreateDesktopW(dname,IntPtr.Zero,IntPtr.Zero,0,desktopRights,ref dsa);Check(desktop!=IntPtr.Zero,"CreateDesktop");S("desktop_name",Name(desktop));Check(ExactAcl(desktop,sid,desktopRights),"protected current SID desktop ACL");B("desktop_acl_verified",true);Check(SetThreadDesktop(desktop),"SetThreadDesktop");stage="clipboard roundtrip";window=CreateWindowExW(0,"STATIC","owned clipboard marker",0,0,0,1,1,IntPtr.Zero,IntPtr.Zero,IntPtr.Zero,IntPtr.Zero);Check(window!=IntPtr.Zero,"owned owner window");string marker="dlpad-owned-station-"+Guid.NewGuid().ToString("N");byte[] bytes=Encoding.Unicode.GetBytes(marker+"\0");OwnedClipboard(station,name);Check(OpenClipboard(window),"OpenClipboard write");clipboard=true;Check(EmptyClipboard(),"EmptyClipboard owned");memory=GlobalAlloc(2,(UIntPtr)bytes.Length);Check(memory!=IntPtr.Zero,"GlobalAlloc");IntPtr ptr=GlobalLock(memory);Check(ptr!=IntPtr.Zero,"GlobalLock write");Marshal.Copy(bytes,0,ptr,bytes.Length);GlobalUnlock(memory);Check(SetClipboardData(13,memory)!=IntPtr.Zero,"SetClipboardData owned");memory=IntPtr.Zero;Check(CloseClipboard(),"CloseClipboard write");clipboard=false;OwnedClipboard(station,name);Check(OpenClipboard(window),"OpenClipboard read");clipboard=true;IntPtr value=GetClipboardData(13);Check(value!=IntPtr.Zero,"GetClipboardData owned");ulong size=GlobalSize(value).ToUInt64();Check(size>=2&&size<=4194304&&(size%2)==0,"owned UTF16 size cap");ptr=GlobalLock(value);Check(ptr!=IntPtr.Zero,"GlobalLock read");string actual=Marshal.PtrToStringUni(ptr,(int)size/2).TrimEnd('\0');GlobalUnlock(value);Check(actual==marker,"synthetic marker equality");Check(CloseClipboard(),"CloseClipboard read");clipboard=false;B("synthetic_roundtrip",true);passed=true;}catch(Exception e){S("failure_stage",stage);S("error",e.Message);}finally{if(clipboard)B("close_clipboard_cleanup",CloseClipboard());if(memory!=IntPtr.Zero)GlobalFree(memory);if(window!=IntPtr.Zero)B("owner_window_destroyed",DestroyWindow(window));B("restore_process_station",SetProcessWindowStation(oldStation));B("restore_thread_desktop",SetThreadDesktop(oldDesktop));if(desktop!=IntPtr.Zero)B("close_owned_desktop",CloseDesktop(desktop));if(station!=IntPtr.Zero)B("close_owned_station",CloseWindowStation(station));if(sd1!=IntPtr.Zero)Marshal.FreeHGlobal(sd1);if(sd2!=IntPtr.Zero)Marshal.FreeHGlobal(sd2);B("borrowed_initial_handles_closed",false);B("WinSta0_clipboard_accessed",false);B("passed",passed);File.WriteAllText(args[0],"{"+String.Join(",",fields.ToArray())+"}");}return passed?0:1;}
}
'@
$sourcePath=Join-Path $OutputDirectory 'helper.cs';[IO.File]::WriteAllText($sourcePath,$source)
$exe=Join-Path $OutputDirectory 'helper.exe';Add-Type -TypeDefinition $source -OutputAssembly $exe -OutputType ConsoleApplication
$info=New-Object System.Diagnostics.ProcessStartInfo;$info.FileName=$exe;$info.Arguments='"'+(Join-Path $OutputDirectory 'result.json')+'"';$info.UseShellExecute=$false;$info.CreateNoWindow=$true
$child=[System.Diagnostics.Process]::Start($info)
if(-not $child.WaitForExit(60000)){$child.Kill();$child.WaitForExit(5000)|Out-Null;throw 'Owned capability helper exceeded60seconds'}
$childCode=$child.ExitCode
Get-Content (Join-Path $OutputDirectory 'result.json')
Get-FileHash -Algorithm SHA256 $sourcePath,$exe|ConvertTo-Json|Set-Content (Join-Path $OutputDirectory 'hashes.json')
$child.Dispose();exit $childCode
