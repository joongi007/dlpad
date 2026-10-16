using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Security.AccessControl;
using System.Security.Principal;
using System.Web.Script.Serialization;
public static class FirstThemeProbe {
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr process,out uint code);
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
 [DllImport("user32.dll",SetLastError=true)] static extern bool EnumDesktopWindows(IntPtr desktop,EnumFn callback,IntPtr l);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h,StringBuilder text,int length);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h,StringBuilder text,int length);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll",SetLastError=true)] static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] static extern bool ScreenToClient(IntPtr h,ref Point p);
 [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr h,ref Point p);
 [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr h,out Rect rect);
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h,out Rect rect);
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
 [DllImport("kernel32.dll",EntryPoint="SetLastError")] static extern void ClearLastError(uint value);
 [DllImport("user32.dll")] static extern IntPtr GetDC(IntPtr window);
 [DllImport("user32.dll")] static extern int ReleaseDC(IntPtr window,IntPtr dc);
 [DllImport("gdi32.dll")] static extern uint GetPixel(IntPtr dc,int x,int y);
 [StructLayout(LayoutKind.Sequential)] struct CWEX {public SI si;public IntPtr attributes;}
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateEventW(IntPtr sa,bool manual,bool initial,string name);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetHandleInformation(IntPtr handle,out uint flags);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool SetEvent(IntPtr handle);
 [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool DuplicateHandle(IntPtr sourceProcess,IntPtr source,IntPtr targetProcess,out IntPtr target,uint access,bool inherit,uint options);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr list,int count,uint flags,ref IntPtr bytes);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr list,uint flags,IntPtr attribute,IntPtr value,IntPtr size,IntPtr previous,IntPtr returned);
 [DllImport("kernel32.dll")] static extern void DeleteProcThreadAttributeList(IntPtr list);
 [DllImport("kernel32.dll",EntryPoint="CreateProcessW",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CWCreateProcess(string exe,StringBuilder command,IntPtr sa,IntPtr ta,bool inherit,uint flags,IntPtr env,string cwd,ref CWEX info,out PI process);
 static void CWLaunch(string exe,string desktop,IntPtr[] fixtures,IntPtr environment,out PI process){
  Require(fixtures.Length==3,"Exactly three fixture capabilities required");IntPtr[] caps=new IntPtr[3];IntPtr list=IntPtr.Zero,values=IntPtr.Zero;bool initialized=false;process=new PI();
  try{
   for(int i=0;i<3;i++){Require(fixtures[i]!=IntPtr.Zero,"Missing original fixture");uint flags;Require(GetHandleInformation(fixtures[i],out flags)&&(flags&1)==0,"Original fixture must be non-inheritable");Require(DuplicateHandle(GetCurrentProcess(),fixtures[i],GetCurrentProcess(),out caps[i],0,true,2),"Owned fixture inheritance duplicate");Require(GetHandleInformation(caps[i],out flags)&&(flags&1)==1,"Only duplicate fixture is inheritable");for(int prior=0;prior<i;prior++)Require(caps[prior]!=caps[i],"Duplicate fixture cap alias");}
   IntPtr bytes=IntPtr.Zero;InitializeProcThreadAttributeList(IntPtr.Zero,1,0,ref bytes);Require(bytes.ToInt64()>0&&bytes.ToInt64()<=65536,"Attribute-list size bound");
   list=Marshal.AllocHGlobal(bytes);Require(InitializeProcThreadAttributeList(list,1,0,ref bytes),"Attribute-list initialize");initialized=true;
   values=Marshal.AllocHGlobal(IntPtr.Size*3);for(int i=0;i<3;i++)Marshal.WriteIntPtr(values,i*IntPtr.Size,caps[i]);
   Require(UpdateProcThreadAttribute(list,0,new IntPtr(0x20002),values,new IntPtr(IntPtr.Size*3),IntPtr.Zero,IntPtr.Zero),"Exact fixture HANDLE_LIST");
   CWEX info=new CWEX();info.si.cb=Marshal.SizeOf(typeof(CWEX));info.si.desktop=desktop;info.attributes=list;
   string argv="\""+exe+"\" --fixture --settings --fixture-first-paint";foreach(IntPtr cap in caps)argv+=" "+cap.ToInt64().ToString(System.Globalization.CultureInfo.InvariantCulture);
   Require(CWCreateProcess(exe,new StringBuilder(argv),IntPtr.Zero,IntPtr.Zero,true,0x08080404,environment,Path.GetDirectoryName(exe),ref info,out process),"Exact fixture-cap suspended app launch");
  }finally{if(initialized)DeleteProcThreadAttributeList(list);if(list!=IntPtr.Zero)Marshal.FreeHGlobal(list);if(values!=IntPtr.Zero)Marshal.FreeHGlobal(values);foreach(IntPtr cap in caps)if(cap!=IntPtr.Zero)CloseHandle(cap);}
 }

 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateFileMappingW(IntPtr file,IntPtr security,uint protect,uint high,uint low,string name);
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr MapViewOfFile(IntPtr mapping,uint access,uint high,uint low,UIntPtr bytes);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool UnmapViewOfFile(IntPtr view);

 static void NewDir(string p){Require(!File.Exists(p)&&!Directory.Exists(p),"Fresh directory required");var acl=new DirectorySecurity();acl.SetAccessRuleProtection(true,false);var sid=WindowsIdentity.GetCurrent().User;acl.SetOwner(sid);acl.AddAccessRule(new FileSystemAccessRule(sid,FileSystemRights.FullControl,InheritanceFlags.ContainerInherit|InheritanceFlags.ObjectInherit,PropagationFlags.None,AccessControlType.Allow));Directory.CreateDirectory(p,acl);Require((File.GetAttributes(p)&FileAttributes.ReparsePoint)==0,"Directory reparse rejected");}
 static void Save(string p,object value){byte[] bytes=Encoding.UTF8.GetBytes(new JavaScriptSerializer(){MaxJsonLength=1048576}.Serialize(value));Require(bytes.Length<=1048576,"Result limit");using(var f=new FileStream(p,FileMode.CreateNew,FileAccess.Write,FileShare.None)){f.Write(bytes,0,bytes.Length);f.Flush(true);}}
 static string Sha(string path){using(var f=File.OpenRead(path))using(var h=System.Security.Cryptography.SHA256.Create())return BitConverter.ToString(h.ComputeHash(f)).Replace("-","").ToLowerInvariant();}
 static void Write(string p,string s){using(var f=new FileStream(p,FileMode.CreateNew,FileAccess.Write,FileShare.None)){byte[] b=Encoding.UTF8.GetBytes(s);f.Write(b,0,b.Length);f.Flush(true);}}
 public static int Main(string[] args){
  // exact executable SHA, NEW native TEMP root, theme. No arbitrary app argv.
  if(args.Length!=4)return 64;
  IntPtr desktop=IntPtr.Zero,job=IntPtr.Zero,request=IntPtr.Zero,ack=IntPtr.Zero,mapping=IntPtr.Zero,mapped=IntPtr.Zero;PI pi=new PI();IntPtr original=GetThreadDesktop(GetCurrentThreadId());bool switched=false;var row=new Dictionary<string,object>();int code=1;
  string before=null;string root=args[2];
  try{
   Require(args[1].Length==64&&Sha(args[0])==args[1],"Binary pin mismatch");Require(args[3]=="Light"||args[3]=="Dark"||args[3]=="System","Theme choice");
   string temp=Path.GetFullPath(Path.GetTempPath()).TrimEnd('\\');root=Path.GetFullPath(root);Require(String.Equals(Path.GetDirectoryName(root),temp,StringComparison.OrdinalIgnoreCase)&&Path.GetFileName(root).StartsWith("dlpad-first-theme-",StringComparison.Ordinal),"Owned TEMP namespace");
   NewDir(root);NewDir(Path.Combine(root,"data"));NewDir(Path.Combine(root,"library"));NewDir(Path.Combine(root,"evidence"));Write(Path.Combine(root,"data","theme.json"),"\""+args[3]+"\"");
   before=Input();string name="dlpad-first-theme-"+Guid.NewGuid().ToString("N");desktop=CreateDesktop(name,IntPtr.Zero,IntPtr.Zero,0,0x1ff,IntPtr.Zero);Require(desktop!=IntPtr.Zero,"Private desktop create");Require(SetThreadDesktop(desktop),"Own observer desktop");switched=true;
   job=CreateJobObject(IntPtr.Zero,null);Require(job!=IntPtr.Zero,"Job create");Limits limits=new Limits();limits.basic.flags=0x2000;Require(SetInformationJobObject(job,9,ref limits,(uint)Marshal.SizeOf(typeof(Limits))),"Job kill-on-close");
   var env=new SortedDictionary<string,string>(StringComparer.OrdinalIgnoreCase);foreach(System.Collections.DictionaryEntry entry in Environment.GetEnvironmentVariables())env[(string)entry.Key]=(string)entry.Value;
   env["DLPAD_TEST_DATA"]=Path.Combine(root,"data");env["DLPAD_LIBRARY_ROOT"]=Path.Combine(root,"library");env.Remove("DLPAD_YTDLP");env.Remove("DLPAD_FFMPEG");env.Remove("DLPAD_DENO");env.Remove("DLPAD_TEST_REPORT_STRESS");
   var block=new StringBuilder();foreach(var e in env)block.Append(e.Key).Append('=').Append(e.Value).Append('\0');block.Append('\0');IntPtr environment=Marshal.StringToHGlobalUni(block.ToString());
   SI si=new SI();si.cb=Marshal.SizeOf(typeof(SI));si.desktop=name;
   try{
    request=CreateEventW(IntPtr.Zero,true,false,null);ack=CreateEventW(IntPtr.Zero,true,false,null);mapping=CreateFileMappingW(new IntPtr(-1),IntPtr.Zero,4,0,4096,null);
    Require(request!=IntPtr.Zero&&ack!=IntPtr.Zero&&mapping!=IntPtr.Zero,"Original receipt capabilities");
    mapped=MapViewOfFile(mapping,6,0,0,new UIntPtr(4096));Require(mapped!=IntPtr.Zero,"Receipt mapping view");
    byte[] initial=new byte[4096];Marshal.Copy(initial,0,mapped,4096);
    CWLaunch(args[0],name,new IntPtr[]{request,ack,mapping},environment,out pi);
   }finally{Marshal.FreeHGlobal(environment);}
   Require(AssignProcessToJobObject(job,pi.process),"Child assign before resume");long start,end,kernel,user;Require(GetProcessTimes(pi.process,out start,out end,out kernel,out user),"Creation identity");
   var clock=Stopwatch.StartNew();Require(ResumeThread(pi.thread)!=0xffffffff,"Resume child");IntPtr window=IntPtr.Zero;
   var enumRows=new List<object>();int enumDropped=0;
   while(clock.ElapsedMilliseconds<5000&&window==IntPtr.Zero){
    int count=0;bool overflow=false,duplicate=false;var candidates=new List<object>();
    EnumFn callback=delegate(IntPtr h,IntPtr unused){
     if(++count>128){overflow=true;return false;}
     uint pid;GetWindowThreadProcessId(h,out pid);
     if(pid==pi.pid&&IsWindowVisible(h)){
      StringBuilder title=new StringBuilder(256),klass=new StringBuilder(256);
      GetWindowText(h,title,256);GetClassName(h,klass,256);
      if(candidates.Count<8)candidates.Add(new {hwnd=h.ToInt64(),title=title.ToString(),window_class=klass.ToString()});
      if(title.ToString()=="dlpad"&&klass.ToString()=="Window Class"){
       if(window!=IntPtr.Zero&&window!=h){duplicate=true;return false;}
       window=h;
      }
     }return true;
    };
    ClearLastError(0);bool returned=EnumDesktopWindows(desktop,callback,IntPtr.Zero);int osError=Marshal.GetLastWin32Error();
    if(enumRows.Count==32){enumRows.RemoveAt(0);enumDropped++;}
    enumRows.Add(new {ms=clock.Elapsed.TotalMilliseconds,returned=returned,os_error=osError,count=count,overflow=overflow,duplicate=duplicate,owned_visible=candidates});
    row["enumeration"]=enumRows;row["enumeration_dropped"]=enumDropped;
    Require(!overflow&&!duplicate,"Desktop enumeration bound or duplicate main HWND");
    // EnumDesktopWindows documents zero for failure or no windows. Accept only
    // the exact no-callback/no-error case as empty, preserving raw observations.
    Require(returned||(count==0&&osError==0),"Desktop enumeration native failure");
    if(WaitForSingleObject(pi.process,0)==0)throw new Exception("Owned app exited before first visible main window");
    if(window==IntPtr.Zero)Thread.Sleep(1);
   }
   Require(window!=IntPtr.Zero,"First visible observation deadline");Owned(window,pi,start);row["first_visible_ms"]=clock.Elapsed.TotalMilliseconds;row["pid"]=pi.pid;row["creation_filetime"]=start;row["hwnd"]=window.ToInt64();
   var samples=new List<object>();
   var samplingClock=Stopwatch.StartNew();
   for(int i=0;i<32;i++){Require(samplingClock.ElapsedMilliseconds<15000,"First-visible sampling deadline");Owned(window,pi,start);Rect r;Require(GetClientRect(window,out r)&&r.right>=320&&r.bottom>=160&&r.right<=4096&&r.bottom<=4096,"Client bounds");IntPtr dc=GetDC(window);Require(dc!=IntPtr.Zero,"Client DC");double sampleBegin=clock.Elapsed.TotalMilliseconds;var colors=new List<uint>(3200);try{using(var image=new Bitmap(80,40)){for(int y=0;y<40;y++)for(int x=0;x<80;x++){Require(samplingClock.ElapsedMilliseconds<15000,"First-visible sampling deadline");uint c=GetPixel(dc,x*4,y*4);Require(c!=0xffffffff,"Actual client pixel unavailable");colors.Add(c);image.SetPixel(x,y,Color.FromArgb((int)(c&255),(int)((c>>8)&255),(int)((c>>16)&255)));}if(i<8){string png=Path.Combine(root,"evidence","sample-"+i+".png");using(var f=new FileStream(png,FileMode.CreateNew,FileAccess.Write,FileShare.None))image.Save(f,ImageFormat.Png);}}}finally{Require(ReleaseDC(window,dc)!=0,"Release owned DC");}samples.Add(new {index=i,begin_ms=sampleBegin,end_ms=clock.Elapsed.TotalMilliseconds,width=r.right,height=r.bottom,tile_x=0,tile_y=0,tile_width=320,tile_height=160,stride=4,colorref=colors});Thread.Sleep(5);}
   row["samples"]=samples;row["first_presentation_verified"]=false;row["classification_requires_pixel_review"]=true;row["sampling_scope"]="top-left 320x160 client, stride4; can miss fine glyph pixels";row["theme_pass_verified"]=false;row["theme_seed"]=args[3];
   // Sampling precedes this request; the owner receipt was latched at ORIGINAL
   // first successful present, not generated from this later induced redraw.
   Owned(window,pi,start);Require(SetEvent(request),"Original receipt request");
   var receiptClock=Stopwatch.StartNew();bool ready=false;
   while(receiptClock.ElapsedMilliseconds<5000){Owned(window,pi,start);Require(PostMessage(window,15,IntPtr.Zero,IntPtr.Zero),"Owned post-observation redraw");uint wait=WaitForSingleObject(ack,50);Require(wait==0||wait==258,"Receipt ACK result");if(wait==0){ready=true;break;}}
   Require(ready,"Original receipt ACK deadline");int bytesCount=Marshal.ReadInt32(mapped);Require(bytesCount>0&&bytesCount<=4088&&Marshal.ReadInt32(mapped,4)==0,"Receipt header bound");byte[] bytes=new byte[bytesCount];Marshal.Copy(IntPtr.Add(mapped,8),bytes,0,bytesCount);
   var receipt=new JavaScriptSerializer(){MaxJsonLength=4088}.Deserialize<Dictionary<string,object>>(new UTF8Encoding(false,true).GetString(bytes));
   uint ownerPid;uint ownerTid=GetWindowThreadProcessId(window,out ownerPid);
   Require(Convert.ToInt32(receipt["version"])==1&&Convert.ToInt32(receipt["sequence"])==1&&Convert.ToUInt32(receipt["pid"])==pi.pid&&ownerPid==pi.pid&&Convert.ToUInt32(receipt["thread"])==ownerTid&&Convert.ToInt64(receipt["hwnd"])==window.ToInt64(),"Original receipt owner identity");
   row["owner_first_present_receipt"]=receipt;
   Require((bool)receipt["successful_original_present"]&&!(bool)receipt["before_present_visible"]&&!(bool)receipt["after_present_visible"]&&(bool)receipt["initial_show_performed"]&&(bool)receipt["after_show_native_visible"],"Hidden-first-present-show order");
   if(args[3]!="System")Require((bool)receipt["dark_mode"]==(args[3]=="Dark"),"Actual first-present preference");
   else {string nativeTheme=receipt["system_theme"] as string;Require(nativeTheme==null||nativeTheme=="Light"||nativeTheme=="Dark","Native system theme domain");if(nativeTheme!=null){Require((bool)receipt["dark_mode"]==(nativeTheme=="Dark"),"Actual native system first-present consistency");row["system_theme_consistency_verified"]=true;}else row["system_theme_consistency_verified"]=false;}
   row["owner_present_order_verified"]=true;row["compositor_scanout_verified"]=false;
   Owned(window,pi,start);Require(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero),"Owned close request");Require(WaitForSingleObject(pi.process,20000)==0,"Natural exit deadline");uint exit;Require(GetExitCodeProcess(pi.process,out exit)&&exit==0,"Natural exit0");row["natural_exit_code"]=exit;code=0;
  }catch(Exception ex){row["error_type"]=ex.GetType().Name;row["error"]=ex.Message.Length<=256?ex.Message:"bounded error";row["os_error"]=Marshal.GetLastWin32Error();}
  finally{
   if(job!=IntPtr.Zero){Accounting a;bool empty=QueryInformationJobObject(job,1,out a,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero)&&a.active==0;if(!empty){code=1;row["forced_owned_job_cleanup"]=true;TerminateJobObject(job,125);if(pi.process!=IntPtr.Zero)WaitForSingleObject(pi.process,5000);empty=QueryInformationJobObject(job,1,out a,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero)&&a.active==0;}row["owned_job_empty"]=empty;if(!empty)code=1;row["owned_job_closed"]=CloseHandle(job);}
   if(mapped!=IntPtr.Zero)UnmapViewOfFile(mapped);foreach(IntPtr cap in new IntPtr[]{request,ack,mapping})if(cap!=IntPtr.Zero)CloseHandle(cap);if(pi.thread!=IntPtr.Zero)CloseHandle(pi.thread);if(pi.process!=IntPtr.Zero)CloseHandle(pi.process);bool restored=!switched||SetThreadDesktop(original);row["observer_desktop_restored"]=restored;bool closed=desktop==IntPtr.Zero||CloseDesktop(desktop);row["private_desktop_closed"]=closed;try{row["input_desktop_unchanged"]=before!=null&&before==Input();row["binary_sha_after"]=Sha(args[0]);if((string)row["binary_sha_after"]!=args[1])code=1;if(!restored||!closed||(bool)row["input_desktop_unchanged"]==false)code=1;if(Directory.Exists(Path.Combine(root,"evidence")))Save(Path.Combine(root,"evidence","result.json"),row);}catch{code=1;}
  }return code;
 }
}
