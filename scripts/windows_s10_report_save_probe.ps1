param(
 [Parameter(Mandatory=$true)][string]$Binary,
 [Parameter(Mandatory=$true)][string]$ExpectedBinarySha,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [ValidateSet('capability','success','metadata-failure','download-failure','cancel')][string]$Mode='metadata-failure',
 [ValidateSet('single','all')][string]$LogOmission='single',
 [string]$FixtureTool,[string]$ExpectedToolSha,[switch]$VerifyReport=$true,[switch]$VerifyReportReachability,[ValidateRange(0,2048)][int]$ReportClientHeight=0
)
$ErrorActionPreference='Stop'
if($VerifyReportReachability -and (-not $VerifyReport -or $ReportClientHeight -ne 0)){throw 'Reachability requires VerifyReport at original client size'}
if($ReportClientHeight -ne 0 -and ($ReportClientHeight -lt 640 -or -not $VerifyReport)){throw 'Report resize needs VerifyReport and client height 640..2048'}
if(-not $VerifyReport){throw 'S10 requires report verification'}
if($Mode -notin @('metadata-failure','download-failure')){throw 'VerifyReport requires a synthetic failure mode'}
# No desktop switch, clipboard, global keyboard/mouse input or TEMP change.
function Assert-PlainPath([string]$Path) {
 if(-not [IO.Path]::IsPathRooted($Path)){throw 'An absolute path is required'}
 $current=[IO.Path]::GetFullPath($Path)
 while($current){
  if(Test-Path -LiteralPath $current){if(([IO.File]::GetAttributes($current) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Reparse path rejected'}}
  $next=[IO.Path]::GetDirectoryName($current);if($next -eq $current){break};$current=$next
 }
}
function Assert-Hash([string]$Path,[string]$Expected){
 Assert-PlainPath $Path
 if($Expected -notmatch '^[a-fA-F0-9]{64}$' -or (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash -ne $Expected){throw 'Immutable input hash mismatch'}
}
function Private-Directory([string]$Path){
 Assert-PlainPath $Path
 if(Test-Path -LiteralPath $Path){throw 'Fresh directory required'}
 $sid=[Security.Principal.WindowsIdentity]::GetCurrent().User
 $acl=New-Object Security.AccessControl.DirectorySecurity
 $acl.SetOwner($sid);$acl.SetAccessRuleProtection($true,$false)
 $acl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($sid,'FullControl','ContainerInherit,ObjectInherit','None','Allow')))
 (New-Object IO.DirectoryInfo($Path)).Create($acl)
}
Assert-Hash $Binary $ExpectedBinarySha
if($Mode -ne 'capability'){Assert-Hash $FixtureTool $ExpectedToolSha}
Private-Directory $OutputDirectory
$root=Join-Path ([IO.Path]::GetTempPath()) ('dlpad-windows-cookie-download-owned-'+[Guid]::NewGuid().ToString('N'))
Private-Directory $root
$utf8=New-Object Text.UTF8Encoding($false)
$source=Join-Path $root 'source.cookies'
$literal="# Netscape HTTP Cookie File`n.fixture.invalid`tTRUE`t/`tTRUE`t0`tSESSION`twindows-download-fixture-only`n"
[IO.File]::WriteAllBytes($source,$utf8.GetBytes($literal))
[IO.File]::WriteAllBytes((Join-Path $root 'fixture.cap'),$utf8.GetBytes('dlpad-windows-session-cookie-synthetic-fixture-v1'))
[IO.File]::WriteAllBytes((Join-Path $root 'mode'),$utf8.GetBytes($Mode))
$sentinel=Join-Path $root 'sentinel';[IO.File]::WriteAllBytes($sentinel,$utf8.GetBytes('owned foreign sentinel'))
$sourceSha=(Get-FileHash $source -Algorithm SHA256).Hash
$sentinelSha=(Get-FileHash $sentinel -Algorithm SHA256).Hash
$env:DLPAD_TEST_DATA=Join-Path $root 'private-cache'
$env:DLPAD_LIBRARY_ROOT=Join-Path $root 'media'
[IO.Directory]::CreateDirectory($env:DLPAD_TEST_DATA)|Out-Null
[IO.Directory]::CreateDirectory($env:DLPAD_LIBRARY_ROOT)|Out-Null
# Product uses the ordinary registered application guardian, not a helper override.
foreach($key in @('DLPAD_GUARDIAN_BACKEND','DLPAD_TEST_SYSTEMD_BUS_PATH','DLPAD_TEST_SYSTEMD_STARTUP_FAULT','DLPAD_TEST_GUARDIAN_EXECUTABLE')){[Environment]::SetEnvironmentVariable($key,$null,'Process')}
if($Mode -ne 'capability'){
 $env:DLPAD_YTDLP=Join-Path $root 'synthetic-ytdlp.exe'
 [IO.File]::Copy($FixtureTool,$env:DLPAD_YTDLP,$false);Assert-Hash $env:DLPAD_YTDLP $ExpectedToolSha
}else{[Environment]::SetEnvironmentVariable('DLPAD_YTDLP',$null,'Process')}
Add-Type -ReferencedAssemblies System.Drawing,UIAutomationClient,UIAutomationTypes,WindowsBase,System.Web.Extensions -TypeDefinition @'
using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Automation;
using System.Web.Script.Serialization;
public static class CookiePrivateGui {
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct SI {public int cb;public string reserved,desktop,title;public uint x,y,xs,ys,xc,yc,fill,flags;public short show,reserved2;public IntPtr r2,input,output,error;}
 [StructLayout(LayoutKind.Sequential)] struct PI {public IntPtr process,thread;public uint pid,tid;}
 [StructLayout(LayoutKind.Sequential)] struct Basic {public long a,b;public uint flags;public UIntPtr min,max;public uint active;public UIntPtr affinity;public uint priority,schedule;}
 [StructLayout(LayoutKind.Sequential)] struct Io {public ulong a,b,c,d,e,f;}
 [StructLayout(LayoutKind.Sequential)] struct Limits {public Basic basic;public Io io;public UIntPtr p,j,peakp,peakj;}
 [StructLayout(LayoutKind.Sequential)] struct Accounting {public long user,kernel,periodUser,periodKernel;public uint faults,total,active,terminated;}
 [StructLayout(LayoutKind.Sequential)] struct Point {public int x,y;}
 [StructLayout(LayoutKind.Sequential)] struct Rect {public int left,top,right,bottom;}
 [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateDesktop(string name,IntPtr device,IntPtr mode,uint flags,uint access,IntPtr security);
 [DllImport("user32.dll",SetLastError=true)] static extern bool CloseDesktop(IntPtr h);
 [DllImport("user32.dll",SetLastError=true)] static extern bool SetThreadDesktop(IntPtr h);
 [DllImport("user32.dll")] static extern IntPtr GetThreadDesktop(uint tid);
 [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] static extern IntPtr OpenInputDesktop(uint flags,bool inherit,uint access);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern bool GetUserObjectInformation(IntPtr h,int index,StringBuilder value,int length,out int needed);
 delegate bool EnumFn(IntPtr h,IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumDesktopWindows(IntPtr desktop,EnumFn callback,IntPtr l);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h,StringBuilder text,int length);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h,StringBuilder text,int length);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll",SetLastError=true)] static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll",SetLastError=true)] static extern IntPtr SendMessageTimeout(IntPtr h,uint m,IntPtr w,IntPtr l,uint flags,uint timeout,out IntPtr result);
 [DllImport("user32.dll")] static extern bool ScreenToClient(IntPtr h,ref Point p);
 [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr h,ref Point p);
 [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr h,out Rect rect);
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h,out Rect rect);
 [DllImport("user32.dll",SetLastError=true)] static extern bool SetWindowPos(IntPtr h,IntPtr after,int x,int y,int width,int height,uint flags);
 [DllImport("user32.dll")] static extern bool PrintWindow(IntPtr h,IntPtr dc,uint flags);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateProcess(string app,StringBuilder command,IntPtr sa,IntPtr ta,bool inherit,uint flags,IntPtr env,string cwd,ref SI si,out PI pi);
 [DllImport("kernel32.dll")] static extern IntPtr CreateJobObject(IntPtr sa,string name);
 [DllImport("kernel32.dll")] static extern bool SetInformationJobObject(IntPtr h,int info,ref Limits limits,uint size);
 [DllImport("kernel32.dll")] static extern bool AssignProcessToJobObject(IntPtr job,IntPtr process);
 [DllImport("kernel32.dll")] static extern bool QueryInformationJobObject(IntPtr job,int info,out Accounting accounting,uint size,IntPtr needed);
 [DllImport("kernel32.dll")] static extern bool TerminateJobObject(IntPtr job,uint code);
 [DllImport("kernel32.dll")] static extern uint ResumeThread(IntPtr thread);
 [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h,uint ms);
 [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr process,uint code);
 [DllImport("kernel32.dll")] static extern bool GetProcessTimes(IntPtr h,out long created,out long exited,out long kernel,out long user);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 static void Require(bool ok,string reason){if(!ok)throw new Exception(reason+" (Win32 "+Marshal.GetLastWin32Error()+")");}
 static string Input(){IntPtr h=OpenInputDesktop(0,false,1);Require(h!=IntPtr.Zero,"Input desktop query");try{int n;StringBuilder b=new StringBuilder(256);Require(GetUserObjectInformation(h,2,b,512,out n),"Input desktop name");return b.ToString();}finally{CloseDesktop(h);}}
 static void Owned(IntPtr window,PI pi,long start){uint pid;GetWindowThreadProcessId(window,out pid);long actual,e,k,u;Require(pid==pi.pid && WaitForSingleObject(pi.process,0)==258,"Owned HWND/process identity");Require(GetProcessTimes(pi.process,out actual,out e,out k,out u)&&actual==start,"Owned process start identity");}
 static AutomationElement[] Nodes(AutomationElement root){List<AutomationElement> items=new List<AutomationElement>();Queue<AutomationElement> todo=new Queue<AutomationElement>();todo.Enqueue(root);TreeWalker walker=TreeWalker.RawViewWalker;while(todo.Count>0){AutomationElement e=todo.Dequeue();items.Add(e);if(items.Count>256)throw new Exception("UIA subtree node limit");for(AutomationElement child=walker.GetFirstChild(e);child!=null;child=walker.GetNextSibling(child)){todo.Enqueue(child);if(todo.Count+items.Count>256)throw new Exception("UIA subtree queue limit");}}return items.ToArray();}
 static AutomationElement Find(AutomationElement root,string name,ControlType type){AutomationElement found=null;foreach(AutomationElement e in Nodes(root)){if(e.Current.Name==name&&e.Current.ControlType==type){Require(found==null,"Duplicate accessible field "+name);found=e;}}Require(found!=null,"Missing accessible field "+name);return found;}
 static string Read(AutomationElement e){object pattern;string value;if(e.TryGetCurrentPattern(ValuePattern.Pattern,out pattern))value=((ValuePattern)pattern).Current.Value;else if(e.TryGetCurrentPattern(TextPattern.Pattern,out pattern))value=((TextPattern)pattern).DocumentRange.GetText(65537);else throw new Exception("No readable text pattern");Require(value.Length<=65536,"UIA text limit");return value;}
 static AutomationElement Activate(IntPtr window,PI pi,long start,string output){List<Dictionary<string,object>> probes=new List<Dictionary<string,object>>();foreach(int objectId in new int[]{-25,-4}){Owned(window,pi,start);IntPtr response;IntPtr delivered=SendMessageTimeout(window,0x003d,IntPtr.Zero,new IntPtr(objectId),0x0002,2000,out response);var row=new Dictionary<string,object>();row["object_id"]=objectId;row["delivered"]=delivered!=IntPtr.Zero;row["response"]=response.ToInt64();row["last_error"]=Marshal.GetLastWin32Error();probes.Add(row);Save(Path.Combine(output,"provider-probes.json"),probes);Require(delivered!=IntPtr.Zero,"Owned WM_GETOBJECT timed out/failed");}AutomationElement root=null;List<int> counts=new List<int>();for(int i=0;i<50;i++){Owned(window,pi,start);root=AutomationElement.FromHandle(window);Require(root!=null&&root.Current.ProcessId==pi.pid,"UIA root identity");counts.Add(Nodes(root).Length);if(counts[counts.Count-1]>1)break;Thread.Sleep(100);}Save(Path.Combine(output,"provider-subtree-counts.json"),counts);return root;}
 static string Hash(string text){using(var sha=System.Security.Cryptography.SHA256.Create()){return BitConverter.ToString(sha.ComputeHash(Encoding.UTF8.GetBytes(text))).Replace("-","").ToLowerInvariant();}}
 static void Pump(IntPtr window,PI pi,long start,int delay){Owned(window,pi,start);Require(PostMessage(window,15,IntPtr.Zero,IntPtr.Zero),"Owned WM_PAINT");Thread.Sleep(delay);}
 static void InputEvidence(AutomationElement e,string expected,string observed,string output,string stage,IntPtr window,PI pi,long start){var row=new Dictionary<string,object>();row["field"]=e.Current.Name;row["stage"]=stage;row["expected_length"]=expected.Length;row["actual_length"]=observed.Length;row["expected_sha256"]=Hash(expected);row["actual_sha256"]=Hash(observed);row["actual_synthetic_uia_value"]=observed.Length<=1024?observed:"[bounded omission]";row["synthetic_only"]=true;var bounds=e.Current.BoundingRectangle;row["bounds_left"]=bounds.Left;row["bounds_top"]=bounds.Top;row["bounds_width"]=bounds.Width;row["bounds_height"]=bounds.Height;string field=expected.StartsWith("https://",StringComparison.Ordinal)?"url":"cookie";Save(Path.Combine(output,"input-"+field+"-"+stage+".json"),row);Capture(window,pi,start,Path.Combine(output,"input-"+field+"-"+stage+".png"));}
 static string Fill(AutomationElement e,string value,IntPtr window,PI pi,long start,string output,bool replace=false){Require(value.Length<=1024,"Input bound");Require(replace||Read(e)=="","Expected initially empty field");if(replace||e.Current.Name=="\uc9c4\ub2e8 \uc790\ub8cc \uc800\uc7a5 \uacbd\ub85c")EnsureContent(e,window,pi,start,output);object pattern;bool setter=e.TryGetCurrentPattern(ValuePattern.Pattern,out pattern);if(setter){try{((ValuePattern)pattern).SetValue(value);}catch(InvalidOperationException){}Pump(window,pi,start,300);string observed=Read(e);InputEvidence(e,value,observed,output,"setter",window,pi,start);if(observed==value)return "ValuePattern/readback";}
 if(e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out pattern)){((ScrollItemPattern)pattern).ScrollIntoView();Pump(window,pi,start,250);}
 var bounds=e.Current.BoundingRectangle;Require(!bounds.IsEmpty&&bounds.Width>0&&bounds.Height>0&&!e.Current.IsOffscreen,"Field bounds unavailable");Point p=new Point();p.x=(int)(bounds.Left+bounds.Width/2);p.y=(int)(bounds.Top+bounds.Height/2);Owned(window,pi,start);Require(ScreenToClient(window,ref p),"Field coordinate conversion");Rect r;Require(GetClientRect(window,out r)&&p.x>=0&&p.y>=0&&p.x<r.right&&p.y<r.bottom,"Field outside owned client");IntPtr xy=new IntPtr((p.y<<16)|(p.x&65535));Require(PostMessage(window,7,IntPtr.Zero,IntPtr.Zero),"Owned focus event");Pump(window,pi,start,200);Require(PostMessage(window,0x200,IntPtr.Zero,xy),"Owned mouse move");Pump(window,pi,start,150);Require(PostMessage(window,0x201,new IntPtr(1),xy),"Owned mouse down");Pump(window,pi,start,150);Require(PostMessage(window,0x202,IntPtr.Zero,xy),"Owned mouse up");Pump(window,pi,start,200);
 string partial=Read(e);if(partial!=""){Require(partial.Length<=1024,"Partial input clear bound");Require(PostMessage(window,0x100,new IntPtr(0x23),new IntPtr(1)),"Owned End down");Require(PostMessage(window,0x101,new IntPtr(0x23),new IntPtr(unchecked((int)0xc0000001))),"Owned End up");Pump(window,pi,start,150);for(int i=0;i<partial.Length;i++){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(8),new IntPtr(1)),"Owned Backspace down");Require(PostMessage(window,0x101,new IntPtr(8),new IntPtr(unchecked((int)0xc0000001))),"Owned Backspace up");Pump(window,pi,start,35);}string cleared=Read(e);InputEvidence(e,value,cleared,output,"clear",window,pi,start);Require(cleared=="","Bounded local backspace did not clear field; no append fallback");}
 foreach(char c in value){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(0x87),new IntPtr(1)),"Owned non-text F24 envelope down");Require(PostMessage(window,0x102,new IntPtr(c),new IntPtr(1)),"Owned WM_CHAR");Require(PostMessage(window,0x101,new IntPtr(0x87),new IntPtr(unchecked((int)0xc0000001))),"Owned non-text F24 envelope up");Pump(window,pi,start,35);}string actual=Read(e);for(int i=0;i<40&&actual!=value;i++){Pump(window,pi,start,50);actual=Read(e);}InputEvidence(e,value,actual,output,"readback",window,pi,start);Require(actual==value,"Owned input readback mismatch: "+e.Current.Name+" expected_length="+value.Length+" actual_length="+actual.Length);return "owned-F24-envelope-WM_CHAR/readback (UIA setter unavailable/unapplied)";}
 static void Invoke(AutomationElement e){object pattern=null;Require(e.Current.IsEnabled&&e.TryGetCurrentPattern(InvokePattern.Pattern,out pattern),"Invoke pattern unavailable/disabled");((InvokePattern)pattern).Invoke();}
 static void Capture(IntPtr window,PI pi,long start,string path){Owned(window,pi,start);Rect r;Require(GetClientRect(window,out r)&&r.right>0&&r.bottom>0&&r.right<=4096&&r.bottom<=4096,"Screenshot dimensions");using(var bitmap=new System.Drawing.Bitmap(r.right,r.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool ok;try{ok=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(ok,"Private PrintWindow unsupported");bitmap.Save(path,System.Drawing.Imaging.ImageFormat.Png);}}
 static void Save(string path,object value){string text=new JavaScriptSerializer().Serialize(value);Require(Encoding.UTF8.GetByteCount(text)<=65536,"Evidence JSON limit");File.WriteAllText(path,text,new UTF8Encoding(false));}

 static Dictionary<string,object> Patterns(AutomationElement e){object p;var row=new Dictionary<string,object>();row["name"]=e.Current.Name;row["type"]=e.Current.ControlType.ProgrammaticName;row["enabled"]=e.Current.IsEnabled;row["keyboard_focusable"]=e.Current.IsKeyboardFocusable;row["has_keyboard_focus"]=e.Current.HasKeyboardFocus;row["invoke"]=e.TryGetCurrentPattern(InvokePattern.Pattern,out p);row["scroll_item"]=e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out p);row["text"]=e.TryGetCurrentPattern(TextPattern.Pattern,out p);AutomationProperty text2=AutomationProperty.LookupById(30119);if(text2!=null){object availability=e.GetCurrentPropertyValue(text2,true);row["text_pattern2_available"]=availability is bool?(object)(bool)availability:"unsupported";}bool hasValue=e.TryGetCurrentPattern(ValuePattern.Pattern,out p);row["value"]=hasValue;if(hasValue)row["read_only"]=((ValuePattern)p).Current.IsReadOnly;return row;}
 static bool InContent(AutomationElement e,IntPtr window,PI pi,long start,out Point point){Owned(window,pi,start);point=new Point();var b=e.Current.BoundingRectangle;if(b.IsEmpty||b.Width<=0||b.Height<=0||e.Current.IsOffscreen)return false;point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);Rect r;if(!ScreenToClient(window,ref point)||!GetClientRect(window,out r)||point.x<0||point.y<0||point.x>=r.right||point.y>=r.bottom)return false;AutomationElement app=AutomationElement.FromHandle(window);foreach(AutomationElement n in Nodes(app)){if(n.Current.ControlType==ControlType.Button&&n.Current.Name=="\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791"){var footer=n.Current.BoundingRectangle;Point top=new Point();top.x=(int)footer.Left;top.y=(int)footer.Top;if(!ScreenToClient(window,ref top)||point.y>=top.y)return false;}}return true;}
 static void EnsureContent(AutomationElement target,IntPtr window,PI pi,long start,string output){
  var rows=new List<Dictionary<string,object>>();
  for(int attempt=0;attempt<32;attempt++){
   Point targetPoint;if(InContent(target,window,pi,start,out targetPoint)){Save(Path.Combine(output,"s10-scroll.json"),rows);return;}
   var app=AutomationElement.FromHandle(window);AutomationElement anchor=null;Point local=new Point();
   foreach(var node in Nodes(app)){Point point;if(node.Current.ControlType==ControlType.Edit&&InContent(node,window,pi,start,out point)){anchor=node;local=point;break;}}
   Require(anchor!=null,"No actual visible owned scroll anchor");
   Point targetCenter=new Point();var bounds=target.Current.BoundingRectangle;targetCenter.x=(int)(bounds.Left+bounds.Width/2);targetCenter.y=(int)(bounds.Top+bounds.Height/2);
   Require(ScreenToClient(window,ref targetCenter),"S10 target coordinate");int delta=targetCenter.y<local.y?120:-120;
   Point screen=local;Require(ClientToScreen(window,ref screen),"S10 wheel coordinate");
   rows.Add(new Dictionary<string,object>{{"attempt",attempt},{"target",target.Current.Name},{"wheel_delta",delta},{"target_client_y",targetCenter.y},{"anchor_client_y",local.y}});
   Save(Path.Combine(output,"s10-scroll.json"),rows);Owned(window,pi,start);
   Require(PostMessage(window,0x200,IntPtr.Zero,new IntPtr((local.y<<16)|(local.x&65535))),"S10 owned mouse move");
   Require(PostMessage(window,0x20a,new IntPtr(delta<<16),new IntPtr((screen.y<<16)|(screen.x&65535))),"S10 owned wheel");Pump(window,pi,start,150);
  }
  Capture(window,pi,start,Path.Combine(output,"s10-scroll-failed.png"));throw new Exception("S10 control not visibly reachable within bounded scroll");
 }
 static void BoundsClick(AutomationElement e,IntPtr window,PI pi,long start,string output){EnsureContent(e,window,pi,start,output);object pattern;if(e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out pattern)){((ScrollItemPattern)pattern).ScrollIntoView();Pump(window,pi,start,300);}var b=e.Current.BoundingRectangle;Require(!b.IsEmpty&&!e.Current.IsOffscreen&&b.Width>0&&b.Height>0,"Report control bounds unavailable");Point point=new Point();point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);Owned(window,pi,start);Rect r;Require(ScreenToClient(window,ref point)&&GetClientRect(window,out r)&&point.x>=0&&point.y>=0&&point.x<r.right&&point.y<r.bottom,"Report control outside owned client");IntPtr xy=new IntPtr((point.y<<16)|(point.x&65535));Require(PostMessage(window,0x200,IntPtr.Zero,xy),"Report owned move");Pump(window,pi,start,100);Require(PostMessage(window,0x201,new IntPtr(1),xy),"Report owned down");Pump(window,pi,start,100);Require(PostMessage(window,0x202,IntPtr.Zero,xy),"Report owned up");Pump(window,pi,start,300);}
 static bool HasLabel(AutomationElement app,string name){foreach(AutomationElement e in Nodes(app))if(e.Current.Name==name)return true;return false;}
 static void Privacy(string text,string root){var forbidden=new List<string>();forbidden.Add("windows-download-fixture-only");forbidden.Add(Path.Combine(root,"source.cookies"));string calls=Path.Combine(root,"calls.jsonl");Require(File.Exists(calls)&&new FileInfo(calls).Length<=16384,"Report toolcalls evidence missing/bounded");foreach(string line in File.ReadAllLines(calls)){if(line.Length==0)continue;var call=new JavaScriptSerializer().Deserialize<Dictionary<string,object>>(line);object path;if(call.TryGetValue("cookie_path",out path)&&path is string)forbidden.Add((string)path);}foreach(string value in forbidden){Require(!text.Contains(value),"Report synthetic secret/private path leaked");string escaped=new JavaScriptSerializer().Serialize(value);Require(!text.Contains(escaped.Substring(1,escaped.Length-2)),"Report escaped private value leaked");}}
 static Dictionary<string,object> Geometry(AutomationElement target,IntPtr window,PI pi,long start){Owned(window,pi,start);var b=target.Current.BoundingRectangle;Point rawPoint=new Point();rawPoint.x=(int)(b.Left+b.Width/2);rawPoint.y=(int)(b.Top+b.Height/2);Require(ScreenToClient(window,ref rawPoint),"Owned geometry coordinate conversion");Point visiblePoint;var row=new Dictionary<string,object>();row["name"]=target.Current.Name;row["screen_top"]=b.Top;row["screen_left"]=b.Left;row["width"]=b.Width;row["height"]=b.Height;row["client_x"]=rawPoint.x;row["client_y"]=rawPoint.y;row["is_offscreen"]=target.Current.IsOffscreen;row["in_owned_content"]=InContent(target,window,pi,start,out visiblePoint);row["has_keyboard_focus"]=target.Current.HasKeyboardFocus;return row;}
 static bool PreviewIntersection(AutomationElement preview,AutomationElement app,IntPtr window,PI pi,long start,string output){Owned(window,pi,start);Rect client;Require(GetClientRect(window,out client),"Owned preview clip client");var bounds=preview.Current.BoundingRectangle;Point origin=new Point();origin.x=(int)Math.Floor(bounds.Left);origin.y=(int)Math.Floor(bounds.Top);Require(ScreenToClient(window,ref origin),"Owned preview clip transform");int bottom=client.bottom;foreach(AutomationElement n in Nodes(app)){if(n.Current.ControlType==ControlType.Button&&n.Current.Name=="\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791"){var footer=n.Current.BoundingRectangle;Point edge=new Point();edge.x=(int)footer.Left;edge.y=(int)footer.Top;Require(ScreenToClient(window,ref edge),"Owned footer clip transform");bottom=Math.Min(bottom,edge.y);}}
 int left=Math.Max(0,origin.x),top=Math.Max(0,origin.y),right=Math.Min(client.right,(int)Math.Ceiling(origin.x+bounds.Width)),lower=Math.Min(bottom,(int)Math.Ceiling(origin.y+bounds.Height));bool intersects=!preview.Current.IsOffscreen&&right>left&&lower>top;var evidence=new Dictionary<string,object>();evidence["uia_is_offscreen"]=preview.Current.IsOffscreen;evidence["uia_width"]=bounds.Width;evidence["uia_height"]=bounds.Height;evidence["intersection_left"]=left;evidence["intersection_top"]=top;evidence["intersection_width"]=Math.Max(0,right-left);evidence["intersection_height"]=Math.Max(0,lower-top);evidence["geometry_intersects"]=intersects;evidence["actual_glyph_visibility_verified"]=false;evidence["requires_root_private_image_review"]=true;Save(Path.Combine(output,"report-preview-visible-intersection.json"),evidence);if(intersects){Require(client.right<=4096&&client.bottom<=4096,"Owned preview screenshot limit");using(var bitmap=new System.Drawing.Bitmap(client.right,client.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool printed;try{printed=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(printed,"Owned preview clip PrintWindow");using(var slice=bitmap.Clone(new System.Drawing.Rectangle(left,top,right-left,lower-top),bitmap.PixelFormat)){slice.Save(Path.Combine(output,"report-preview-visible-intersection.png"),System.Drawing.Imaging.ImageFormat.Png);}}}return intersects;}
 static void FocusProbe(AutomationElement target,IntPtr window,PI pi,long start,string output,string slug,bool endKey){Save(Path.Combine(output,slug+"-before.json"),Geometry(target,window,pi,start));Capture(window,pi,start,Path.Combine(output,slug+"-before.png"));Require(target.Current.IsKeyboardFocusable,"Owned reachability target not focusable");target.SetFocus();Pump(window,pi,start,300);Require(target.Current.HasKeyboardFocus,"Owned reachability focus not reflected");if(endKey){Require(PostMessage(window,0x100,new IntPtr(0x23),new IntPtr(1)),"Reachability owned End down");Require(PostMessage(window,0x101,new IntPtr(0x23),new IntPtr(unchecked((int)0xc0000001))),"Reachability owned End up");Pump(window,pi,start,500);}Save(Path.Combine(output,slug+"-after.json"),Geometry(target,window,pi,start));Capture(window,pi,start,Path.Combine(output,slug+"-after.png"));}
 static void S10Click(AutomationElement app,string label,IntPtr window,PI pi,long start,string output){
  Owned(window,pi,start);var button=Find(app,label,ControlType.Button);object action;
  if(button.TryGetCurrentPattern(ScrollItemPattern.Pattern,out action)){((ScrollItemPattern)action).ScrollIntoView();Pump(window,pi,start,120);}
  if(button.Current.IsKeyboardFocusable){button.SetFocus();Pump(window,pi,start,120);}
  EnsureContent(button,window,pi,start,output);Invoke(button);Pump(window,pi,start,160);
 }
 static void S10Field(AutomationElement app,string label,string text,IntPtr window,PI pi,long start,string output){
  var field=Find(app,label,ControlType.Edit);Require(field.Current.IsKeyboardFocusable,"S10 field focus unsupported");
  field.SetFocus();Pump(window,pi,start,160);string stage=Path.Combine(output,"input-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(stage);Fill(field,text,window,pi,start,stage,true);Require(Read(field)==text,"S10 exact field readback");
 }
 static string S10Export(AutomationElement app,string root,string output,string slug,string format,IntPtr window,PI pi,long start){
  S10Click(app,format,window,pi,start,output);
  string path=Path.Combine(root,slug+(format=="JSON"?".json":".md"));Require(!File.Exists(path),"S10 fresh export required");
  S10Field(app,"진단 자료 저장 경로",path,window,pi,start,output);
  S10Click(app,"새 파일로 저장",window,pi,start,output);
  for(int i=0;i<80&&!File.Exists(path);i++)Pump(window,pi,start,100);
  Require(File.Exists(path),"S10 export absent");
  // Worker completes sync and returns report before the saved label is published.
  for(int i=0;i<80&&!HasLabel(app,"선택한 새 파일로 저장했습니다.");i++)Pump(window,pi,start,100);
  for(int i=0;i<80&&!Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled;i++)Pump(window,pi,start,100);
  Require(HasLabel(app,"선택한 새 파일로 저장했습니다.")&&Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled,"S10 worker completion absent");
  Require(new FileInfo(path).Length>0&&new FileInfo(path).Length<=2097152,"S10 export bound");
  string text=File.ReadAllText(path,new UTF8Encoding(false,true));Privacy(text,root);
  var evidence=new Dictionary<string,object>();evidence["format"]=format;evidence["bytes"]=new FileInfo(path).Length;evidence["sha256"]=Hash(text);
  evidence["clipboard_invoked"]=false;Save(Path.Combine(output,slug+"-export.json"),evidence);
  return text;
 }
 static Dictionary<string,object> S10Json(string text){return new JavaScriptSerializer(){MaxJsonLength=2097152}.Deserialize<Dictionary<string,object>>(text);}
 static int S10Count(Dictionary<string,object> report,string key){object value;Require(report.TryGetValue(key,out value),"S10 expected JSON field missing");var list=value as System.Collections.ICollection;Require(list!=null,"S10 JSON array shape");return list.Count;}
 static void S10WriteNew(string path,byte[] bytes){using(var f=new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.None)){f.Write(bytes,0,bytes.Length);f.Flush(true);}}
 static void VerifyDiagnostic(AutomationElement app,IntPtr window,PI pi,long start,string root,string output,Dictionary<string,object> result,bool reachability,string logOmission){
  result["clipboard_invoked"]=false;result["paste_invoked"]=false;result["closewait_inflight_verified"]=false;
  S10Click(app,"실패 원인 보기",window,pi,start,output);
  var initial=S10Json(S10Export(app,root,output,"s10-initial","JSON",window,pi,start));
  int initialLogs=S10Count(initial,"logs");Require(initialLogs>=1,"S10 fixture must expose at least one sanitized log");
  string md=S10Export(app,root,output,"s10-initial-md","Markdown",window,pi,start);Require(md.Length>0,"S10 Markdown empty");
  S10Field(app,"로그 번호 (0부터)","0",window,pi,start,output);
  S10Click(app,"선택한 로그 수정",window,pi,start,output);
  const string edited="S10 public observation https://fixture.invalid/?signature=synthetic-value";
  S10Field(app,"선택한 로그 수정",edited,window,pi,start,output);
  S10Click(app,"정제 후 반영",window,pi,start,output);
  string editedRaw=S10Export(app,root,output,"s10-edited","JSON",window,pi,start);
  Require(editedRaw.Contains("S10 public observation")&&!editedRaw.Contains("fixture.invalid")&&!editedRaw.Contains("synthetic-value"),"S10 edited URL not resanitized");
  Require(S10Count(S10Json(editedRaw),"logs")==initialLogs,"S10 safe edit changed count");
  result["log_omission_case"]=logOmission;
  if(logOmission=="single"){
   S10Click(app,"선택한 로그 제외",window,pi,start,output);
   Require(S10Count(S10Json(S10Export(app,root,output,"s10-omit-one-log","JSON",window,pi,start)),"logs")==initialLogs-1,"S10 single log omit mismatch");
  }
  S10Click(app,"로그 제외",window,pi,start,output);
  Require(S10Count(S10Json(S10Export(app,root,output,"s10-omit-all-logs","JSON",window,pi,start)),"logs")==0,"S10 all log omit mismatch");
  result["all_logs_nonempty_before_action_verified"]=logOmission=="all"||initialLogs>1;
  string mpd=Path.Combine(root,"s10-fixture.mpd");byte[] mpdBytes=Encoding.UTF8.GetBytes("<MPD><BaseURL>https://fixture.invalid/?signature=mpd-fixture</BaseURL><SegmentList><Initialization range=\"0-10\"/><SegmentURL mediaRange=\"11-20\"/></SegmentList></MPD>");
  S10WriteNew(mpd,mpdBytes);
  S10Field(app,"추가할 로컬 MPD 경로 (최대 1 MiB)",mpd,window,pi,start,output);
  S10Click(app,"구조만 첨부",window,pi,start,output);
  // Save controls are disabled while attachment worker owns the report; wait actual enabled state.
  for(int i=0;i<80&&!Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled;i++)Pump(window,pi,start,100);
  string attachedRaw=S10Export(app,root,output,"s10-attached","JSON",window,pi,start);var attached=S10Json(attachedRaw);
  Require(S10Count(attached,"mpd_attachments")==1&&!attachedRaw.Contains("mpd-fixture"),"S10 MPD structure/privacy mismatch");
  S10Click(app,"MPD 0",window,pi,start,output);S10Click(app,"선택한 MPD 구조 제외",window,pi,start,output);
  Require(S10Count(S10Json(S10Export(app,root,output,"s10-omit-one-mpd","JSON",window,pi,start)),"mpd_attachments")==0,"S10 single MPD omit mismatch");
  S10Click(app,"구조만 첨부",window,pi,start,output);
  for(int i=0;i<80&&!Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled;i++)Pump(window,pi,start,100);
  Require(S10Count(S10Json(S10Export(app,root,output,"s10-reattached","JSON",window,pi,start)),"mpd_attachments")==1,"S10 reattach mismatch");
  foreach(bool oversized in new[]{false,true}){
   string bad=Path.Combine(root,oversized?"s10-oversize.mpd":"s10-unsafe.mpd");byte[] data=oversized?new byte[1048577]:Encoding.UTF8.GetBytes("<!DOCTYPE MPD [<!ENTITY x SYSTEM 'file:///SYNTHETIC'>]><MPD>&x;</MPD>");S10WriteNew(bad,data);
   S10Field(app,"추가할 로컬 MPD 경로 (최대 1 MiB)",bad,window,pi,start,output);S10Click(app,"구조만 첨부",window,pi,start,output);
   for(int i=0;i<80&&!Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled;i++)Pump(window,pi,start,100);
   var rejected=S10Json(S10Export(app,root,output,oversized?"s10-rejected-large":"s10-rejected-xml","JSON",window,pi,start));
   Require(S10Count(rejected,"mpd_attachments")==1&&S10Count(rejected,"attachment_omissions")>0,"S10 unsafe input not rejected");
   Require(Convert.ToBase64String(File.ReadAllBytes(bad))==Convert.ToBase64String(data),"S10 rejected input modified");
  }
  S10Click(app,"MPD 구조 모두 제외",window,pi,start,output);
  Require(S10Count(S10Json(S10Export(app,root,output,"s10-omit-all-mpd","JSON",window,pi,start)),"mpd_attachments")==0,"S10 all MPD omit mismatch");
  Require(Convert.ToBase64String(File.ReadAllBytes(mpd))==Convert.ToBase64String(mpdBytes),"S10 source MPD modified");
  string sentinel=Path.Combine(root,"sentinel"),before=Hash(File.ReadAllText(sentinel));
  S10Field(app,"진단 자료 저장 경로",sentinel,window,pi,start,output);S10Click(app,"새 파일로 저장",window,pi,start,output);
  for(int i=0;i<80&&!HasLabel(app,"진단 자료 작업을 완료하지 못했습니다. 입력 파일과 기존 저장 파일은 유지됩니다.");i++)Pump(window,pi,start,100);
  Require(HasLabel(app,"진단 자료 작업을 완료하지 못했습니다. 입력 파일과 기존 저장 파일은 유지됩니다.")&&Hash(File.ReadAllText(sentinel))==before,"S10 create-only collision not preserved");
  Capture(window,pi,start,Path.Combine(output,"s10-final.png"));
  result["s10_save_edit_omit_attach_assertions"]=true;result["s10_report_requires_root_db_and_image_review"]=true;
 }
 public static string Run(string exe,string root,string output,string mode,bool verifyReport,int reportHeight,bool reachability,string logOmission){Dictionary<string,object> result=new Dictionary<string,object>();result["mode"]=mode;result["status"]="FAIL";result["input_before"]=Input();string name="dlpad-cookie-"+Guid.NewGuid().ToString("N");result["private_desktop"]=name;IntPtr desktop=IntPtr.Zero,job=IntPtr.Zero,window=IntPtr.Zero;PI pi=new PI();long start=0;bool restored=false;Thread worker=null;Exception workerError=null;
 try{desktop=CreateDesktop(name,IntPtr.Zero,IntPtr.Zero,0,0x1ff,IntPtr.Zero);Require(desktop!=IntPtr.Zero,"Create private desktop");job=CreateJobObject(IntPtr.Zero,null);Require(job!=IntPtr.Zero,"Create owned Job");Limits limits=new Limits();limits.basic.flags=0x2000;Require(SetInformationJobObject(job,9,ref limits,(uint)Marshal.SizeOf(typeof(Limits))),"Set owned Job kill-on-close");SI si=new SI();si.cb=Marshal.SizeOf(typeof(SI));si.desktop=name;Require(CreateProcess(exe,new StringBuilder("\""+exe+"\" --fixture"),IntPtr.Zero,IntPtr.Zero,false,0x08000004,IntPtr.Zero,Path.GetDirectoryName(exe),ref si,out pi),"Create suspended app");if(!AssignProcessToJobObject(job,pi.process)){TerminateProcess(pi.process,91);throw new Exception("Owned Job binding failed before resume");}long e,k,u;Require(GetProcessTimes(pi.process,out start,out e,out k,out u),"App start identity");result["pid"]=pi.pid;result["start_filetime"]=start;Require(ResumeThread(pi.thread)!=0xffffffff,"Resume owned app");for(int i=0;i<150&&window==IntPtr.Zero;i++){int count=0;EnumDesktopWindows(desktop,delegate(IntPtr h,IntPtr l){if(++count>64)return false;uint id;GetWindowThreadProcessId(h,out id);if(id==pi.pid&&IsWindowVisible(h)){StringBuilder title=new StringBuilder(256);GetWindowText(h,title,256);if(title.ToString()=="dlpad"){Require(window==IntPtr.Zero||window==h,"Duplicate owned dlpad windows");window=h;}}return true;},IntPtr.Zero);if(WaitForSingleObject(pi.process,0)==0)break;Thread.Sleep(100);}Require(window!=IntPtr.Zero,"Owned visible HWND missing");result["hwnd"]=window.ToInt64();Owned(window,pi,start);StringBuilder windowClass=new StringBuilder(256);GetClassName(window,windowClass,256);result["window_class"]=windowClass.ToString();Rect windowRect;Require(GetClientRect(window,out windowRect),"Owned client metadata");result["client_width"]=windowRect.right;result["client_height"]=windowRect.bottom;
 worker=new Thread(delegate(){IntPtr previous=GetThreadDesktop(GetCurrentThreadId());try{Require(SetThreadDesktop(desktop),"MTA private desktop binding");Owned(window,pi,start);Capture(window,pi,start,Path.Combine(output,"before-uia.png"));AutomationElement app=Activate(window,pi,start,output);Require(app!=null&&app.Current.ProcessId==pi.pid,"UIA root identity");var tree=new List<Dictionary<string,object>>();foreach(AutomationElement n in Nodes(app)){string label=n.Current.Name;Require(label.Length<=4096,"UIA name bound");object p;var row=new Dictionary<string,object>();row["name"]=label;row["native_window_handle"]=n.Current.NativeWindowHandle;row["process_id"]=n.Current.ProcessId;row["framework_id"]=n.Current.FrameworkId;row["automation_id"]=n.Current.AutomationId;row["control_type"]=n.Current.ControlType.ProgrammaticName;row["enabled"]=n.Current.IsEnabled;row["value"]=n.TryGetCurrentPattern(ValuePattern.Pattern,out p);row["text"]=n.TryGetCurrentPattern(TextPattern.Pattern,out p);row["invoke"]=n.TryGetCurrentPattern(InvokePattern.Pattern,out p);tree.Add(row);}Save(Path.Combine(output,"uia-tree.json"),tree);AutomationElement url=Find(app,"\uc601\uc0c1 URL",ControlType.Edit);AutomationElement cookie=Find(app,"\uc774\ubc88 \uc791\uc5c5\uc758 Netscape \ucfe0\ud0a4 \ud30c\uc77c (\uc120\ud0dd)",ControlType.Edit);AutomationElement button=Find(app,"\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791",ControlType.Button);Require(Read(url)==""&&Read(cookie)=="","Initial field text mismatch");object invoke;Require(button.TryGetCurrentPattern(InvokePattern.Pattern,out invoke),"Start Invoke unsupported");result["field_labels_verified"]=true;Capture(window,pi,start,Path.Combine(output,"initial.png"));if(mode!="capability"){result["url_input_route"]=Fill(url,"https://fixture.invalid/watch",window,pi,start,output);result["cookie_input_route"]=Fill(cookie,Path.Combine(root,"source.cookies"),window,pi,start,output);Invoke(button);for(int i=0;i<100&&Read(cookie)!="";i++)Thread.Sleep(100);Require(Read(cookie)=="","Cookie source field not consumed");result["cookie_field_cleared"]=true;if(mode=="cancel"){for(int i=0;i<100&&!File.Exists(Path.Combine(root,"worker-ready"));i++)Thread.Sleep(100);Require(File.Exists(Path.Combine(root,"worker-ready")),"Actual download worker readiness missing");Invoke(Find(app,"\uc791\uc5c5 \ucde8\uc18c",ControlType.Button));}Thread.Sleep(mode=="cancel"?2500:5000);if(verifyReport){if(reportHeight!=0){Owned(window,pi,start);Rect client=new Rect(),outer=new Rect();Require(GetClientRect(window,out client)&&GetWindowRect(window,out outer),"Owned resize metadata");result["report_original_client_width"]=client.right;result["report_original_client_height"]=client.bottom;Require(SetWindowPos(window,IntPtr.Zero,0,0,outer.right-outer.left,reportHeight+(outer.bottom-outer.top-client.bottom),0x0016),"Owned window resize without activation");Pump(window,pi,start,500);Require(GetClientRect(window,out client)&&client.bottom>=reportHeight,"Owned requested client height not reached");result["report_resized_client_height"]=client.bottom;result["original_960x640_report_scroll_verified"]=false;}VerifyDiagnostic(app,window,pi,start,root,output,result,reachability,logOmission);}Capture(window,pi,start,Path.Combine(output,"terminal.png"));Save(Path.Combine(output,"terminal-uia.json"),Array.ConvertAll(Nodes(app),delegate(AutomationElement n){return n.Current.Name;}));result["download_assertions_pending_root_db_review"]=true;}
 Owned(window,pi,start);Require(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero),"Owned WM_CLOSE");Require(WaitForSingleObject(pi.process,5000)==0,"WM_CLOSE did not exit (no global input fallback)");result["wm_close_exited"]=true;result["closewait_inflight_verified"]=false;result["status"]=mode=="capability"?"PASS":"REVIEW_REQUIRED";
 }catch(Exception error){workerError=error;}finally{restored=SetThreadDesktop(previous);}});worker.IsBackground=true;worker.SetApartmentState(ApartmentState.MTA);worker.Start();Require(worker.Join(90000),"UIA worker deadline exceeded");if(workerError!=null)throw workerError;Require(restored,"MTA desktop restoration failed");result["mta_desktop_restored"]=restored;
 }catch(Exception error){result["error"]=error.GetType().FullName+": "+error.Message;result["status"]="FAIL";}
 finally{if(job!=IntPtr.Zero){Accounting accounting;bool queried=QueryInformationJobObject(job,1,out accounting,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero);result["owned_job_active_before_cleanup"]=queried?(object)accounting.active:null;if(!queried||accounting.active!=0){result["owned_job_terminated"]=TerminateJobObject(job,92);}bool empty=false;for(int i=0;i<100;i++){if(QueryInformationJobObject(job,1,out accounting,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero)&&accounting.active==0){empty=true;break;}Thread.Sleep(50);}result["owned_job_empty_after_cleanup"]=empty;if(!empty)result["status"]="FAIL";CloseHandle(job);}if(pi.process!=IntPtr.Zero){bool gone=WaitForSingleObject(pi.process,5000)==0;result["owned_process_exited_after_job_close"]=gone;if(!gone)result["status"]="FAIL";CloseHandle(pi.process);}if(pi.thread!=IntPtr.Zero)CloseHandle(pi.thread);if(desktop!=IntPtr.Zero){bool closed=CloseDesktop(desktop);result["private_desktop_closed"]=closed;if(!closed)result["status"]="FAIL";}try{string after=Input();result["input_after"]=after;result["input_desktop_unchanged"]=after==(string)result["input_before"];if(!(bool)result["input_desktop_unchanged"])result["status"]="FAIL";}catch(Exception error){result["input_query_error"]=error.Message;result["status"]="FAIL";}}
 return new JavaScriptSerializer().Serialize(result);}
}
'@
$result=$null
try{
 $result=[CookiePrivateGui]::Run($Binary,$root,$OutputDirectory,$Mode,[bool]$VerifyReport,$ReportClientHeight,[bool]$VerifyReportReachability,$LogOmission)|ConvertFrom-Json
 Assert-Hash $Binary $ExpectedBinarySha
 if($Mode -ne 'capability'){Assert-Hash $FixtureTool $ExpectedToolSha}
 if((Get-FileHash $source -Algorithm SHA256).Hash -ne $sourceSha -or (Get-FileHash $sentinel -Algorithm SHA256).Hash -ne $sentinelSha){throw 'Synthetic source/sentinel changed'}
 $result|Add-Member -NotePropertyName source_sha256 -NotePropertyValue $sourceSha
 $result|Add-Member -NotePropertyName sentinel_sha256 -NotePropertyValue $sentinelSha
 $result|Add-Member -NotePropertyName binary_sha256 -NotePropertyValue $ExpectedBinarySha
 $result|Add-Member -NotePropertyName owned_root -NotePropertyValue $root
 # Retain bounded raw files after confirmed process termination, never a live DB copy.
 if($result.owned_process_exited_after_job_close){
  $files=@(Get-ChildItem -LiteralPath $root -Recurse -File)
  if($files.Count -gt 128){throw 'Owned evidence file count exceeded'}
  $manifest=@();$total=0
  foreach($file in $files){
   Assert-PlainPath $file.FullName
   if($file.Extension -eq '.exe'){continue}
   if($file.Length -gt 2097152){throw 'Owned file evidence bound exceeded'}
   $total+=$file.Length;if($total -gt 8388608){throw 'Owned evidence total exceeded'}
   $relative=$file.FullName.Substring($root.Length+1)
   $target=Join-Path (Join-Path $OutputDirectory 'owned-files') $relative
   [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target))|Out-Null
   [IO.File]::Copy($file.FullName,$target,$false)
   $manifest+=@{path=$relative;size=$file.Length;sha256=(Get-FileHash $file.FullName -Algorithm SHA256).Hash}
  }
  $manifest|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $OutputDirectory 'owned-files.json') -Encoding UTF8
 }
 $result|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $OutputDirectory 'result.json') -Encoding UTF8
}catch{
 @{status='FAIL';error=$_.Exception.Message;owned_root=$root;runtime_result=$result}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $OutputDirectory 'result.json') -Encoding UTF8
 throw
}
if($result.status -eq 'FAIL'){throw 'Private GUI capability/action failed; see result.json. No visible desktop fallback.'}
# REVIEW_REQUIRED is deliberately not an authenticated-download PASS. Root must
# inspect the preserved DB/Attempt/artifact/calls/privacy and cleanup records.
Write-Output ($result|ConvertTo-Json -Depth 8)
