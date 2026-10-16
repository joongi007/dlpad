param([Parameter(Mandatory=$true)][string]$Binary,[Parameter(Mandatory=$true)][string]$OutputDirectory,[switch]$Active,[switch]$CancelLeave,[switch]$FixtureEvents,[switch]$BurstClick,[switch]$ExistingTrace)
$ErrorActionPreference='Stop'
if (Test-Path $OutputDirectory) { throw 'OutputDirectory must be fresh' }
$sid=[System.Security.Principal.WindowsIdentity]::GetCurrent().User
$acl=New-Object System.Security.AccessControl.DirectorySecurity
$acl.SetOwner($sid); $acl.SetAccessRuleProtection($true,$false)
$rule=New-Object System.Security.AccessControl.FileSystemAccessRule($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow')
$acl.AddAccessRule($rule)
$dir=New-Object System.IO.DirectoryInfo($OutputDirectory); $dir.Create($acl)
$env:TMP=$OutputDirectory; $env:TEMP=$OutputDirectory
$env:DLPAD_TEST_DATA=Join-Path $OutputDirectory 'data'
$env:DLPAD_LIBRARY_ROOT=Join-Path $OutputDirectory 'media'
[System.IO.Directory]::CreateDirectory($env:DLPAD_TEST_DATA)|Out-Null
[System.IO.Directory]::CreateDirectory($env:DLPAD_LIBRARY_ROOT)|Out-Null
$existingTraceHash=$null
if($ExistingTrace){
if(-not $FixtureEvents){throw 'ExistingTrace requires fixture events mode'}
$existing=Join-Path $env:DLPAD_TEST_DATA 'fixture-events.jsonl'
$stream=[System.IO.File]::Open($existing,[System.IO.FileMode]::CreateNew,[System.IO.FileAccess]::Write)
try{$bytes=[System.Text.Encoding]::UTF8.GetBytes("owned-existing-trace-sentinel`n");$stream.Write($bytes,0,$bytes.Length)}finally{$stream.Dispose()}
$existingTraceHash=(Get-FileHash -Algorithm SHA256 $existing).Hash
}
if($Active){
$env:DLPAD_SYNTHETIC_ROOT=$OutputDirectory
$env:DLPAD_YTDLP=Join-Path $OutputDirectory 'worker.exe'
Add-Type -OutputAssembly $env:DLPAD_YTDLP -OutputType ConsoleApplication -TypeDefinition @'
using System;using System.IO;using System.Threading;using System.Diagnostics;
class SyntheticWorker {
static string J(string s){return "\""+s.Replace("\\","\\\\").Replace("\"","\\\"")+"\"";}
static void Main(string[] args){if(Array.IndexOf(args,"--version")>=0){Console.WriteLine("2026.10.02");return;}if(Array.IndexOf(args,"--dump-single-json")>=0){Console.WriteLine("{\"id\":\"synthetic\",\"duration\":120}");return;}
string root=Environment.GetEnvironmentVariable("DLPAD_SYNTHETIC_ROOT");File.WriteAllText(Path.Combine(root,"worker.pid"),Process.GetCurrentProcess().Id.ToString());File.WriteAllText(Path.Combine(root,"worker.start"),Process.GetCurrentProcess().StartTime.ToUniversalTime().ToString("o"));
for(int i=0;i<600;i++){File.WriteAllText(Path.Combine(root,"heartbeat"),i.ToString());Console.WriteLine("DLPAD_PROGRESS:{\"speed\":1024,\"eta\":"+(120-i/5)+"}");Console.Out.Flush();Thread.Sleep(200);}
int a=Array.IndexOf(args,"--paths"),b=Array.IndexOf(args,"--output");if(a<0||b<0)Environment.Exit(3);string path=Path.Combine(args[a+1],args[b+1].Replace("%(ext)s","mp4"));File.WriteAllBytes(path,new byte[65536]);Console.WriteLine("DLPAD_ARTIFACT:"+J(path));}
}
'@
}
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public class PrivateGui {
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct SI { public int cb; public string reserved,desktop,title; public uint x,y,xs,ys,xc,yc,fill,flags; public short show,reserved2; public IntPtr r2,input,output,error; }
 [StructLayout(LayoutKind.Sequential)] struct PI { public IntPtr process,thread; public uint pid,tid; }
 [StructLayout(LayoutKind.Sequential)] struct Basic {public long a,b;public uint flags;public UIntPtr min,max;public uint active;public UIntPtr affinity;public uint priority,schedule;}
 [StructLayout(LayoutKind.Sequential)] struct Io {public ulong a,b,c,d,e,f;}
 [StructLayout(LayoutKind.Sequential)] struct Limits {public Basic basic;public Io io;public UIntPtr p,j,peakp,peakj;}
 [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateDesktop(string n,IntPtr d,IntPtr m,uint f,uint a,IntPtr s);
 [DllImport("user32.dll")] static extern bool CloseDesktop(IntPtr d);
 [DllImport("user32.dll")] static extern IntPtr OpenInputDesktop(uint f,bool i,uint a);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern bool GetUserObjectInformation(IntPtr h,int n,StringBuilder b,int l,out int need);
 delegate bool EnumFn(IntPtr h,IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumDesktopWindows(IntPtr d,EnumFn f,IntPtr l);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h,StringBuilder b,int n);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll",SetLastError=true)] static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateProcess(string a,StringBuilder c,IntPtr p,IntPtr t,bool i,uint f,IntPtr e,string cwd,ref SI s,out PI pi);
 [DllImport("kernel32.dll")] static extern IntPtr CreateJobObject(IntPtr s,string n);
 [DllImport("kernel32.dll")] static extern bool SetInformationJobObject(IntPtr j,int n,ref Limits l,uint s);
 [DllImport("kernel32.dll")] static extern bool AssignProcessToJobObject(IntPtr j,IntPtr p);
 [DllImport("kernel32.dll")] static extern uint ResumeThread(IntPtr t);
 [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h,uint ms);
 [DllImport("kernel32.dll")] static extern bool GetExitCodeProcess(IntPtr p,out uint code);
 [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr p,uint c);
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("kernel32.dll")] static extern bool IsProcessInJob(IntPtr p,IntPtr j,out bool member);
 [DllImport("kernel32.dll")] static extern bool GetProcessTimes(IntPtr p,out long created,out long exited,out long kernel,out long user);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 static void Check(bool b){if(!b)throw new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error());}
 public static string Input(){IntPtr h=OpenInputDesktop(0,false,1);if(h==IntPtr.Zero)throw new Exception("Input desktop query failed");try{int n;var b=new StringBuilder(256);Check(GetUserObjectInformation(h,2,b,512,out n));return b.ToString();}finally{CloseDesktop(h);}}
 [StructLayout(LayoutKind.Sequential)] struct Rect {public int left,top,right,bottom;}
 [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr h,out Rect r);
 static void Owned(IntPtr h,uint pid){uint actual;GetWindowThreadProcessId(h,out actual);Check(actual==pid);}
 static bool burstClick;
 static void Click(IntPtr h,uint pid,int x,int y){Owned(h,pid);Rect r;Check(GetClientRect(h,out r));int px=x*(r.right-r.left)/960,py=y*(r.bottom-r.top)/640;IntPtr l=new IntPtr((py<<16)|(px&65535));Check(PostMessage(h,0x200,IntPtr.Zero,l));if(!burstClick){Check(PostMessage(h,15,IntPtr.Zero,IntPtr.Zero));System.Threading.Thread.Sleep(150);}Check(PostMessage(h,0x201,new IntPtr(1),l));Check(PostMessage(h,15,IntPtr.Zero,IntPtr.Zero));System.Threading.Thread.Sleep(150);if(burstClick)Check(PostMessage(h,0x200,IntPtr.Zero,new IntPtr((py<<16)|((px+1)&65535))));Check(PostMessage(h,0x202,IntPtr.Zero,l));Check(PostMessage(h,15,IntPtr.Zero,IntPtr.Zero));System.Threading.Thread.Sleep(300);}
 [DllImport("user32.dll")] static extern bool PrintWindow(IntPtr h,IntPtr dc,uint flags);
 static void Capture(IntPtr h,uint pid,string path){Owned(h,pid);Rect r;Check(GetClientRect(h,out r));using(var b=new System.Drawing.Bitmap(r.right-r.left,r.bottom-r.top))using(var g=System.Drawing.Graphics.FromImage(b)){IntPtr dc=g.GetHdc();bool ok;try{ok=PrintWindow(h,dc,3);}finally{g.ReleaseHdc(dc);}if(ok)b.Save(path,System.Drawing.Imaging.ImageFormat.Png);}}
 static int Heart(string root){try{return int.Parse(System.IO.File.ReadAllText(System.IO.Path.Combine(root,"heartbeat")));}catch{return -1;}}
 static bool WaitVisible(IntPtr h,bool v){for(int i=0;i<50;i++){if(IsWindowVisible(h)==v)return true;System.Threading.Thread.Sleep(100);}return false;}
 [StructLayout(LayoutKind.Sequential)] struct Tme {public uint size,flags;public IntPtr window;public uint hover;}
 [DllImport("user32.dll",SetLastError=true)] static extern bool TrackMouseEvent(ref Tme t);
 public static string Run(string exe,bool active,string root,bool cancelLeave,bool fixtureEvents,bool burst){burstClick=burst;string before=Input(),name="dlpad-gui-"+Guid.NewGuid().ToString("N");IntPtr desk=IntPtr.Zero,job=IntPtr.Zero,workerHandle=IntPtr.Zero;PI pi=new PI();string title="";IntPtr window=IntPtr.Zero;bool visible=false,closed=false,started=false,hidden=false,continued=false,restored=false,secondaryExited=false;uint secondaryPid=0,workerPid=0,secondaryCode=259,primaryCode=259;long workerStarted=0;int heartbeatBefore=-1,heartbeatAfter=-1;bool workerOwned=false,workerStartMatched=false,workerSame=false,workerExited=false;int cancelled=0,cancelErrors=0;System.Threading.Thread cancelThread=null;bool stopCancel=false;
 try{desk=CreateDesktop(name,IntPtr.Zero,IntPtr.Zero,0,0x1ff,IntPtr.Zero);Check(desk!=IntPtr.Zero);job=CreateJobObject(IntPtr.Zero,null);Check(job!=IntPtr.Zero);Limits limits=new Limits();limits.basic.flags=0x2000;Check(SetInformationJobObject(job,9,ref limits,(uint)Marshal.SizeOf(typeof(Limits))));SI si=new SI();si.cb=Marshal.SizeOf(typeof(SI));si.desktop=name;Check(CreateProcess(exe,new StringBuilder("\""+exe+"\" --downloads"+(fixtureEvents?" --fixture --fixture-events":"")),IntPtr.Zero,IntPtr.Zero,false,0x08000004,IntPtr.Zero,System.IO.Path.GetDirectoryName(exe),ref si,out pi));if(!AssignProcessToJobObject(job,pi.process)){TerminateProcess(pi.process,91);throw new Exception("Owned job assignment failed");}Check(ResumeThread(pi.thread)!=0xffffffff);
 for(int i=0;i<150&&window==IntPtr.Zero;i++){int count=0;EnumDesktopWindows(desk,delegate(IntPtr h,IntPtr l){if(++count>64)return false;uint pid;GetWindowThreadProcessId(h,out pid);if(pid==pi.pid){var b=new StringBuilder(256);GetWindowText(h,b,256);if(b.ToString().ToLowerInvariant().Contains("dlpad")){window=h;title=b.ToString();visible=IsWindowVisible(h);}}return true;},IntPtr.Zero);if(WaitForSingleObject(pi.process,0)==0)break;System.Threading.Thread.Sleep(100);}
 if(window!=IntPtr.Zero){System.Threading.Thread.Sleep(2000);visible=IsWindowVisible(window);
 if(active){if(cancelLeave){cancelThread=new System.Threading.Thread(delegate(){while(!System.Threading.Volatile.Read(ref stopCancel)&&WaitForSingleObject(pi.process,0)!=0){uint cancellationOwner;GetWindowThreadProcessId(window,out cancellationOwner);if(cancellationOwner!=pi.pid)break;Tme t=new Tme();t.size=(uint)Marshal.SizeOf(typeof(Tme));t.flags=0x80000002;t.window=window;if(TrackMouseEvent(ref t))System.Threading.Interlocked.Increment(ref cancelled);else System.Threading.Interlocked.Increment(ref cancelErrors);System.Threading.Thread.Sleep(2);}});cancelThread.IsBackground=true;cancelThread.Start();}Owned(window,pi.pid);Check(PostMessage(window,7,IntPtr.Zero,IntPtr.Zero));System.Threading.Thread.Sleep(300);Capture(window,pi.pid,System.IO.Path.Combine(root,"initial.png"));Click(window,pi.pid,250,160);foreach(char ch in "https://fixture.invalid/watch"){Owned(window,pi.pid);Check(PostMessage(window,0x100,new IntPtr(0x41),new IntPtr(1)));Check(PostMessage(window,0x102,new IntPtr(ch),new IntPtr(1)));Check(PostMessage(window,0x101,new IntPtr(0x41),new IntPtr(unchecked((int)0xc0000001))));Check(PostMessage(window,15,IntPtr.Zero,IntPtr.Zero));System.Threading.Thread.Sleep(50);}
 Capture(window,pi.pid,System.IO.Path.Combine(root,"typed.png"));Click(window,pi.pid,500,350);for(int i=0;i<12;i++){if(burstClick){Rect scrollRect;Check(GetClientRect(window,out scrollRect));int sx=(500+i%2)*(scrollRect.right-scrollRect.left)/960,sy=350*(scrollRect.bottom-scrollRect.top)/640;Check(PostMessage(window,0x200,IntPtr.Zero,new IntPtr((sy<<16)|(sx&65535))));}Check(PostMessage(window,0x20a,new IntPtr(unchecked((int)0xff880000)),IntPtr.Zero));Check(PostMessage(window,15,IntPtr.Zero,IntPtr.Zero));System.Threading.Thread.Sleep(80);}Capture(window,pi.pid,System.IO.Path.Combine(root,"scrolled.png"));Click(window,pi.pid,80,435);
 for(int i=0;i<100&&Heart(root)<0;i++)System.Threading.Thread.Sleep(100);started=Heart(root)>=0;
 if(started){workerPid=uint.Parse(System.IO.File.ReadAllText(System.IO.Path.Combine(root,"worker.pid")));workerHandle=OpenProcess(0x101000,false,workerPid);Check(workerHandle!=IntPtr.Zero);Check(IsProcessInJob(workerHandle,job,out workerOwned));Check(workerOwned);long exitedTime,kernelTime,userTime;Check(GetProcessTimes(workerHandle,out workerStarted,out exitedTime,out kernelTime,out userTime));DateTime expectedStart=DateTime.Parse(System.IO.File.ReadAllText(System.IO.Path.Combine(root,"worker.start")),System.Globalization.CultureInfo.InvariantCulture,System.Globalization.DateTimeStyles.RoundtripKind);workerStartMatched=DateTime.FromFileTimeUtc(workerStarted)==expectedStart.ToUniversalTime();Check(workerStartMatched);Check(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero));System.Threading.Thread.Sleep(500);Click(window,pi.pid,295,604);hidden=WaitVisible(window,false);heartbeatBefore=Heart(root);System.Threading.Thread.Sleep(1200);heartbeatAfter=Heart(root);continued=heartbeatAfter>heartbeatBefore&&WaitForSingleObject(workerHandle,0)!=0;
 PI second=new PI();SI secondSi=new SI();secondSi.cb=Marshal.SizeOf(typeof(SI));secondSi.desktop=name;
 Check(CreateProcess(exe,new StringBuilder("\""+exe+"\" --downloads"+(fixtureEvents?" --fixture --fixture-events":"")),IntPtr.Zero,IntPtr.Zero,false,0x08000004,IntPtr.Zero,System.IO.Path.GetDirectoryName(exe),ref secondSi,out second));try{secondaryPid=second.pid;if(!AssignProcessToJobObject(job,second.process)){TerminateProcess(second.process,92);throw new Exception("Secondary owned job assignment failed");}Check(ResumeThread(second.thread)!=0xffffffff);secondaryExited=WaitForSingleObject(second.process,5000)==0;Check(GetExitCodeProcess(second.process,out secondaryCode));secondaryExited=secondaryExited&&secondaryCode==0;restored=WaitVisible(window,true);Capture(window,pi.pid,System.IO.Path.Combine(root,"restored.png"));workerSame=uint.Parse(System.IO.File.ReadAllText(System.IO.Path.Combine(root,"worker.pid")))==workerPid&&WaitForSingleObject(workerHandle,0)!=0;}finally{CloseHandle(second.thread);CloseHandle(second.process);}
 }
 }
 System.Threading.Volatile.Write(ref stopCancel,true);if(cancelThread!=null)cancelThread.Join(2000);uint pid;GetWindowThreadProcessId(window,out pid);Check(pid==pi.pid);Check(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero));if(started){System.Threading.Thread.Sleep(500);Click(window,pi.pid,80,604);}closed=WaitForSingleObject(pi.process,5000)==0;Check(GetExitCodeProcess(pi.process,out primaryCode));closed=closed&&primaryCode==0;}
 }finally{System.Threading.Volatile.Write(ref stopCancel,true);if(cancelThread!=null)cancelThread.Join(2000);if(job!=IntPtr.Zero)CloseHandle(job);if(workerHandle!=IntPtr.Zero){workerExited=WaitForSingleObject(workerHandle,5000)==0;CloseHandle(workerHandle);}if(pi.process!=IntPtr.Zero){WaitForSingleObject(pi.process,5000);CloseHandle(pi.process);}if(pi.thread!=IntPtr.Zero)CloseHandle(pi.thread);if(desk!=IntPtr.Zero)CloseDesktop(desk);}
 string after=Input();string extra=",\"primary_exit_code\":"+primaryCode+",\"secondary_exit_code\":"+secondaryCode+",\"heartbeat_before_hidden_wait\":"+heartbeatBefore+",\"heartbeat_after_hidden_wait\":"+heartbeatAfter+",\"worker_pid\":"+workerPid+",\"worker_start_filetime\":"+workerStarted+",\"worker_in_owned_job\":"+(workerOwned?"true":"false")+",\"worker_start_matched\":"+(workerStartMatched?"true":"false")+",\"worker_same_after_restore\":"+(workerSame?"true":"false")+",\"worker_exited_on_cleanup\":"+(workerExited?"true":"false")+",\"cancel_leave_requested\":"+(cancelLeave?"true":"false")+",\"track_cancel_calls\":"+cancelled+",\"track_cancel_errors\":"+cancelErrors+",\"active_requested\":"+(active?"true":"false")+",\"worker_started\":"+(started?"true":"false")+",\"hidden\":"+(hidden?"true":"false")+",\"worker_continued\":"+(continued?"true":"false")+",\"restored\":"+(restored?"true":"false")+",\"secondary_exited\":"+(secondaryExited?"true":"false")+",\"secondary_pid\":"+secondaryPid;return "{\"pid\":"+pi.pid+",\"private_desktop\":\""+name+"\",\"window_found\":"+(window!=IntPtr.Zero?"true":"false")+",\"visible_on_private_desktop\":"+(visible?"true":"false")+",\"wm_close_exited\":"+(closed?"true":"false")+",\"input_desktop_unchanged\":"+(before==after?"true":"false")+extra+"}"; }
}
'@
$expected='98c48919e79f713c0ecca74882ad4f4d5ad34beec19b0a2778e9f839e3f15ff7'
$hash=(Get-FileHash -Algorithm SHA256 $Binary).Hash.ToLowerInvariant()
if($hash -ne $expected){throw 'Unexpected application snapshot'}
$result=[PrivateGui]::Run($Binary,[bool]$Active,$OutputDirectory,[bool]$CancelLeave,[bool]$FixtureEvents,[bool]$BurstClick)|ConvertFrom-Json
$trace=Join-Path $env:DLPAD_TEST_DATA 'fixture-events.jsonl'
$traceExists=Test-Path $trace
if($traceExists){
$traceFile=Get-Item $trace
if($traceFile.Length -gt 16384){throw 'Fixture trace exceeded bounded evidence limit'}
$rows=@(Get-Content $trace)
if($rows.Count -gt 64){throw 'Fixture trace exceeded 64 rows'}
Copy-Item -LiteralPath $trace -Destination (Join-Path $OutputDirectory 'fixture-events.jsonl')
$result|Add-Member fixture_trace_rows $rows.Count
}
$result|Add-Member fixture_trace_exists ([bool]$traceExists)
if($ExistingTrace){$preserved=(Get-FileHash -Algorithm SHA256 $trace).Hash -eq $existingTraceHash;$result|Add-Member existing_trace_preserved $preserved;if(-not $preserved){throw 'Existing owned trace changed'}}
$result|Add-Member burst_click_requested ([bool]$BurstClick)
$result|Add-Member fixture_events_requested ([bool]$FixtureEvents)
$result|Add-Member binary_sha256 $hash
$result|Add-Member observed_utc ([DateTime]::UtcNow.ToString('o'))
$result|Add-Member scope 'Private desktop GUI with synthetic PostMessage input; real IME and real media are unverified'
$result|ConvertTo-Json|Set-Content -Encoding UTF8 (Join-Path $OutputDirectory 'result.json')
$result|ConvertTo-Json
if(-not $result.window_found -or -not $result.wm_close_exited -or -not $result.input_desktop_unchanged){exit 1}

if($Active -and (-not $result.worker_started -or -not $result.hidden -or -not $result.worker_continued -or -not $result.restored -or -not $result.secondary_exited -or -not $result.worker_in_owned_job -or -not $result.worker_start_matched -or -not $result.worker_same_after_restore -or -not $result.worker_exited_on_cleanup)){exit 1}
