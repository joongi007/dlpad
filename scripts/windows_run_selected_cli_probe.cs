using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Automation;
using System.Web.Script.Serialization;
public static class LaunchSelectedPrivateGui {
 static volatile bool ObserverStopping;
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
 [DllImport("user32.dll",EntryPoint="PostMessageW",ExactSpelling=true,CharSet=CharSet.Unicode,SetLastError=true)] static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
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
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool QueryInformationJobObject(IntPtr job,int info,out Accounting accounting,uint size,IntPtr needed);
 [DllImport("kernel32.dll")] static extern bool TerminateJobObject(IntPtr job,uint code);
 [DllImport("kernel32.dll")] static extern uint ResumeThread(IntPtr thread);
 [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h,uint ms);
 [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr process,uint code);
 [DllImport("kernel32.dll")] static extern bool GetProcessTimes(IntPtr h,out long created,out long exited,out long kernel,out long user);
 [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
 static void Require(bool ok,string reason){if(!ok)throw new Exception(reason+" (Win32 "+Marshal.GetLastWin32Error()+")");}
 static string Input(){IntPtr h=OpenInputDesktop(0,false,1);Require(h!=IntPtr.Zero,"Input desktop query");try{int n;StringBuilder b=new StringBuilder(256);Require(GetUserObjectInformation(h,2,b,512,out n),"Input desktop name");return b.ToString();}finally{CloseDesktop(h);}}
 static void Owned(IntPtr window,PI pi,long start){Require(!ObserverStopping,"Observer cooperative stop");uint pid;GetWindowThreadProcessId(window,out pid);long actual,e,k,u;Require(pid==pi.pid && WaitForSingleObject(pi.process,0)==258,"Owned HWND/process identity");Require(GetProcessTimes(pi.process,out actual,out e,out k,out u)&&actual==start,"Owned process start identity");}
 static AutomationElement[] Nodes(AutomationElement root){List<AutomationElement> items=new List<AutomationElement>();Queue<AutomationElement> todo=new Queue<AutomationElement>();todo.Enqueue(root);TreeWalker walker=TreeWalker.RawViewWalker;while(todo.Count>0){AutomationElement e=todo.Dequeue();items.Add(e);if(items.Count>256)throw new Exception("UIA subtree node limit");for(AutomationElement child=walker.GetFirstChild(e);child!=null;child=walker.GetNextSibling(child)){todo.Enqueue(child);if(todo.Count+items.Count>256)throw new Exception("UIA subtree queue limit");}}return items.ToArray();}
 static AutomationElement Activate(IntPtr window,PI pi,long start,string output){List<Dictionary<string,object>> probes=new List<Dictionary<string,object>>();foreach(int objectId in new int[]{-25,-4}){Owned(window,pi,start);IntPtr response;IntPtr delivered=SendMessageTimeout(window,0x003d,IntPtr.Zero,new IntPtr(objectId),0x0002,2000,out response);var row=new Dictionary<string,object>();row["object_id"]=objectId;row["delivered"]=delivered!=IntPtr.Zero;row["response"]=response.ToInt64();row["last_error"]=Marshal.GetLastWin32Error();probes.Add(row);Save(Path.Combine(output,"provider-probes.json"),probes);Require(delivered!=IntPtr.Zero,"Owned WM_GETOBJECT timed out/failed");}AutomationElement root=null;List<int> counts=new List<int>();for(int i=0;i<50;i++){Owned(window,pi,start);root=AutomationElement.FromHandle(window);Require(root!=null&&root.Current.ProcessId==pi.pid,"UIA root identity");counts.Add(Nodes(root).Length);if(counts[counts.Count-1]>1)break;Thread.Sleep(100);}Save(Path.Combine(output,"provider-subtree-counts.json"),counts);return root;}
 static void Pump(IntPtr window,PI pi,long start,int delay){Owned(window,pi,start);Require(PostMessage(window,15,IntPtr.Zero,IntPtr.Zero),"Owned WM_PAINT");Thread.Sleep(delay);}
 static void Capture(IntPtr window,PI pi,long start,string path){Owned(window,pi,start);Rect r;Require(GetClientRect(window,out r)&&r.right>0&&r.bottom>0&&r.right<=4096&&r.bottom<=4096,"Screenshot dimensions");using(var bitmap=new System.Drawing.Bitmap(r.right,r.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool ok;try{ok=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(ok,"Private PrintWindow unsupported");bitmap.Save(path,System.Drawing.Imaging.ImageFormat.Png);}}
 static void S10Plain(string path){
  Require(Path.IsPathRooted(path)&&Path.GetFullPath(path)==path,"Absolute normalized fixture path required");
  string current=path;int count=0;
  while(!String.IsNullOrEmpty(current)){
   Require(++count<=512,"Path ancestor bound");
   if(File.Exists(current)||Directory.Exists(current))Require((File.GetAttributes(current)&FileAttributes.ReparsePoint)==0,"Reparse fixture path rejected");
   string next=Path.GetDirectoryName(current);if(next==current)break;current=next;
  }
 }
 static void S10PrivateNewDirectory(string path){
  S10Plain(path);Require(!File.Exists(path)&&!Directory.Exists(path),"Fresh private directory required");
  var sid=System.Security.Principal.WindowsIdentity.GetCurrent().User;Require(sid!=null,"Current SID unavailable");
  var security=new System.Security.AccessControl.DirectorySecurity();security.SetOwner(sid);security.SetAccessRuleProtection(true,false);
  security.AddAccessRule(new System.Security.AccessControl.FileSystemAccessRule(sid,System.Security.AccessControl.FileSystemRights.FullControl,
   System.Security.AccessControl.InheritanceFlags.ContainerInherit|System.Security.AccessControl.InheritanceFlags.ObjectInherit,
   System.Security.AccessControl.PropagationFlags.None,System.Security.AccessControl.AccessControlType.Allow));
  byte[] bytes=security.GetSecurityDescriptorBinaryForm();GCHandle pinned=GCHandle.Alloc(bytes,GCHandleType.Pinned);
  try{S10SA sa=new S10SA();sa.length=Marshal.SizeOf(typeof(S10SA));sa.descriptor=pinned.AddrOfPinnedObject();
   Require(CreateDirectoryW(path,ref sa),"Create-only private directory failed");
  }finally{pinned.Free();}
  S10Plain(path);var observed=new DirectoryInfo(path).GetAccessControl();Require(observed.AreAccessRulesProtected,"Private ACL not protected");
  Require(observed.GetOwner(typeof(System.Security.Principal.SecurityIdentifier)).Equals(sid),"Private owner mismatch");
  var rules=observed.GetAccessRules(true,true,typeof(System.Security.Principal.SecurityIdentifier));Require(rules.Count==1,"Private ACL rule count");
  var rule=(System.Security.AccessControl.FileSystemAccessRule)rules[0];Require(rule.IdentityReference.Equals(sid)&&!rule.IsInherited&&rule.AccessControlType==System.Security.AccessControl.AccessControlType.Allow&&rule.FileSystemRights==System.Security.AccessControl.FileSystemRights.FullControl,"Private ACL exact rule mismatch");
 }
 static AutomationElement ThemeCombo(AutomationElement app){
  AutomationElement found=null;
  foreach(var node in Nodes(app)){
   if(node.Current.ControlType!=ControlType.ComboBox&&node.Current.ControlType!=ControlType.Button)continue;
   Require(node.Current.Name.Length<=4096,"Theme name bound");var label=node.Current.LabeledBy;
   if(node.Current.Name!="화면 테마"&&(label==null||label.Current.Name!="화면 테마"))continue;
   Require(found==null,"Ambiguous labelled theme control");found=node;
  }
  Require(found!=null,"Labelled theme control unavailable");return found;
 }
 static void SettingsAction(AutomationElement app,string name,IntPtr window,PI pi,long start){
  Owned(window,pi,start);AutomationElement target=null;
  foreach(var node in Nodes(app)){
   if(node.Current.Name!=name||!node.Current.IsEnabled)continue;
   var role=node.Current.ControlType;
   if(role!=ControlType.Button&&role!=ControlType.TabItem&&role!=ControlType.RadioButton&&role!=ControlType.CheckBox&&role!=ControlType.ListItem)continue;
   Require(target==null,"Ambiguous settings action");target=node;
  }
  Require(target!=null,"Missing enabled settings action");
  var b=target.Current.BoundingRectangle;Require(!b.IsEmpty&&b.Width>0&&b.Height>0&&!target.Current.IsOffscreen,"Settings action bounds unavailable");
  Point point=new Point();point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);
  Owned(window,pi,start);Require(ScreenToClient(window,ref point),"Settings action client conversion");
  Rect client;Require(GetClientRect(window,out client)&&point.x>=0&&point.y>=0&&point.x<client.right&&point.y<client.bottom,"Settings action outside owned client");
  Require(target.Current.IsKeyboardFocusable,"Settings action not focusable");target.SetFocus();Pump(window,pi,start,100);
  Require(target.Current.HasKeyboardFocus,"Settings action focus not observed");Owned(window,pi,start);
  Require(PostMessage(window,0x100,new IntPtr(0x20),new IntPtr(1)),"Settings owned Space down");
  Require(PostMessage(window,0x101,new IntPtr(0x20),new IntPtr(unchecked((int)0xc0000001))),"Settings owned Space up");
  Pump(window,pi,start,150);
 }
 static void ThemeObserve(AutomationElement app,IntPtr window,PI pi,long start,string output,Dictionary<string,object> result,string mode){
  // Existing Run captures before-uia.png before accessibility activation. That is
  // an observed capture only; this draft has NO first-presentation claim.
  SettingsAction(app,"설정",window,pi,start);
  var combo=ThemeCombo(app);result["theme_control_type"]=combo.Current.ControlType.ProgrammaticName;
  result["initial_theme_control_name"]=combo.Current.Name;
  object value;result["initial_theme_value_pattern"]=combo.TryGetCurrentPattern(ValuePattern.Pattern,out value);
  if(value is ValuePattern){string actual=((ValuePattern)value).Current.Value;Require(actual.Length<=4096,"Theme actual value bound");result["initial_theme_actual_value"]=actual;}
  Capture(window,pi,start,Path.Combine(output,"settings-initial.png"));
  if(Desired!=null){
   object expand;Require(combo.Current.IsEnabled,"Theme control disabled");
   if(combo.TryGetCurrentPattern(ExpandCollapsePattern.Pattern,out expand))((ExpandCollapsePattern)expand).Expand();
   else if(combo.TryGetCurrentPattern(InvokePattern.Pattern,out expand))((InvokePattern)expand).Invoke();
   else throw new Exception("Theme expand action unsupported");
   Pump(window,pi,start,150);
   SettingsAction(app,Desired,window,pi,start);
   Pump(window,pi,start,250);
   Capture(window,pi,start,Path.Combine(output,"settings-selected.png"));
   result["requested_choice"]=Desired;
  }
  var rows=new List<Dictionary<string,object>>();
  foreach(var n in Nodes(app))rows.Add(new Dictionary<string,object>{{"name",n.Current.Name},{"type",n.Current.ControlType.ProgrammaticName},{"enabled",n.Current.IsEnabled}});
  Save(Path.Combine(output,"settings-tree.json"),rows);
  result["actual_os_change_verified"]=false;result["first_presentation_verified"]=false;
 }

 static void Save(string path,object value){string text=new JavaScriptSerializer().Serialize(value);Require(Encoding.UTF8.GetByteCount(text)<=65536,"Evidence JSON limit");File.WriteAllText(path,text,new UTF8Encoding(false));}
 [DllImport("kernel32.dll",EntryPoint="SetLastError",ExactSpelling=true)] static extern void SetLastError(uint error);
 static string Desired;
 static string ExpectedCpu;
 [StructLayout(LayoutKind.Sequential)] struct S10SA {public int length;public IntPtr descriptor;public int inherit;}
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateDirectoryW(string path,ref S10SA attributes);
 [StructLayout(LayoutKind.Sequential)] struct SX {public SI si;public IntPtr attributes;}
 [StructLayout(LayoutKind.Sequential)] struct SA {public int size;public IntPtr descriptor;public int inherit;}
 [StructLayout(LayoutKind.Sequential)] struct ID {public ulong volume;[MarshalAs(UnmanagedType.ByValArray,SizeConst=16)]public byte[] value;}
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetFileInformationByHandleEx(IntPtr h,int type,out ID id,uint size);
 [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr OpenProcess(uint access,bool inherit,uint pid);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool QueryFullProcessImageNameW(IntPtr h,uint flags,StringBuilder name,ref uint size);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool CreatePipe(out IntPtr read,out IntPtr write,ref SA sa,uint size);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool SetHandleInformation(IntPtr h,uint mask,uint flags);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateFileW(string path,uint access,uint share,ref SA sa,uint creation,uint flags,IntPtr template);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr list,int count,uint flags,ref UIntPtr bytes);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr list,uint flags,UIntPtr attribute,IntPtr value,UIntPtr bytes,IntPtr previous,IntPtr returned);
 [DllImport("kernel32.dll")] static extern void DeleteProcThreadAttributeList(IntPtr list);
 [DllImport("kernel32.dll",EntryPoint="CreateProcessW",ExactSpelling=true,CharSet=CharSet.Unicode,SetLastError=true)] static extern bool StartProducer(string app,StringBuilder command,IntPtr sa,IntPtr ta,bool inherit,uint flags,IntPtr env,string cwd,ref SX startup,out PI child);
 static string Quote(string value){Require(value.IndexOf('\0')<0&&value.Length<=32767,"Arg bound");var b=new StringBuilder("\"");int slash=0;foreach(char c in value){if(c=='\\'){slash++;continue;}b.Append('\\',c=='"'?slash*2+1:slash);b.Append(c);slash=0;}b.Append('\\',slash*2);return b.Append('"').ToString();}
 static string Hash(FileStream file){long size=file.Length;Require(size>0&&size<=134217728,"File bound");file.Position=0;byte[] buffer=new byte[65536];long total=0;using(var hash=System.Security.Cryptography.SHA256.Create()){int n;while((n=file.Read(buffer,0,buffer.Length))>0){Require(n<=size-total,"File growth rejected");total+=n;hash.TransformBlock(buffer,0,n,null,0);}Require(total==size&&file.Length==size,"File size changed");hash.TransformFinalBlock(new byte[0],0,0);return BitConverter.ToString(hash.Hash).Replace("-","").ToLowerInvariant();}}
 static FileStream Pin(string path,string sha){S10Plain(path);Require(sha.Length==64,"SHA format");foreach(char c in sha)Require((c>='0'&&c<='9')||(c>='a'&&c<='f'),"SHA format");var file=new FileStream(path,FileMode.Open,FileAccess.Read,FileShare.Read);try{Require(Hash(file)==sha,"File digest mismatch");return file;}catch{file.Dispose();throw;}}
 static ID Identity(FileStream file){ID id;Require(GetFileInformationByHandleEx(file.SafeFileHandle.DangerousGetHandle(),18,out id,(uint)Marshal.SizeOf(typeof(ID))),"File128 identity");return id;}
 static bool Same(ID a,ID b){if(a.volume!=b.volume)return false;for(int i=0;i<16;i++)if(a.value[i]!=b.value[i])return false;return true;}
 sealed class Records {public readonly System.Collections.Concurrent.BlockingCollection<Dictionary<string,object>> queue=new System.Collections.Concurrent.BlockingCollection<Dictionary<string,object>>(2);public Exception error;public Thread thread;public void Start(IntPtr h){thread=new Thread(delegate(){try{using(var file=new FileStream(new Microsoft.Win32.SafeHandles.SafeFileHandle(h,true),FileAccess.Read)){byte[] line=new byte[4097];int count=0,records=0;for(;;){int c=file.ReadByte();if(c<0){Require(count==0,"Truncated producer JSON");break;}if(c==10){Require(++records<=1,"Producer record count");string raw=new UTF8Encoding(false,true).GetString(line,0,count).TrimEnd('\r');count=0;var row=new JavaScriptSerializer(){MaxJsonLength=4096}.Deserialize<Dictionary<string,object>>(raw);Require(queue.TryAdd(row,1000),"Producer queue bound");}else{Require(count<4096,"Producer record size");line[count++]=(byte)c;}}}}catch(Exception e){error=e;}finally{queue.CompleteAdding();}});thread.IsBackground=true;thread.Start();}public Dictionary<string,object> Next(int ms){Dictionary<string,object> row;Require(queue.TryTake(out row,ms),"Producer record deadline");return row;}}
 static string Text(Dictionary<string,object> row,string key){object x;Require(row.TryGetValue(key,out x)&&x is string,"Missing bounded string");string value=(string)x;Require(value.Length<=32767&&value.IndexOf('\0')<0,"String bound");return value;}
 static ulong Number(Dictionary<string,object> row,string key){object x;Require(row.TryGetValue(key,out x),"Missing numeric field");return Convert.ToUInt64(x,System.Globalization.CultureInfo.InvariantCulture);}
 static void Flag(Dictionary<string,object> row,string key){object x;Require(row.TryGetValue(key,out x)&&x is bool&&(bool)x,"Required producer guard");}
 static Dictionary<string,object> One(string fixture,string[] args,string output,string mode,string installed,string dataRoot){
  ObserverStopping=false;var result=new Dictionary<string,object>{{"status","FAIL"},{"input_before",Input()},{"child_handle_is_observation_only",true}};
  IntPtr env=IntPtr.Zero,previousDesktop=IntPtr.Zero;bool hiddenConsole=false;IntPtr desktop=IntPtr.Zero,job=IntPtr.Zero,read=IntPtr.Zero,write=IntPtr.Zero,nul=IntPtr.Zero,list=IntPtr.Zero,handles=IntPtr.Zero,observed=IntPtr.Zero;PI producer=new PI();Records records=null;Thread worker=null;bool restored=false,forced=false,attributesInitialized=false;Exception workerError=null;
  try{string name="dlpad-cli-"+Guid.NewGuid().ToString("N");result["private_desktop"]=name;desktop=CreateDesktop(name,IntPtr.Zero,IntPtr.Zero,0,0x1ff,IntPtr.Zero);Require(desktop!=IntPtr.Zero,"Private desktop");previousDesktop=GetThreadDesktop(GetCurrentThreadId());Require(SetThreadDesktop(desktop),"Console original private desktop bind");if(GetConsoleCP()!=0)Require(FreeConsole(),"Detach original host console");AllocateHidden(result);hiddenConsole=true;Require(SetThreadDesktop(previousDesktop),"Console thread desktop restore");env=EnvironmentBlock(dataRoot);job=CreateJobObject(IntPtr.Zero,null);Require(job!=IntPtr.Zero,"Parent Job");Limits limits=new Limits();limits.basic.flags=0x2000;Require(SetInformationJobObject(job,9,ref limits,(uint)Marshal.SizeOf(typeof(Limits))),"Parent Job limit");SA sa=new SA();sa.size=Marshal.SizeOf(typeof(SA));sa.inherit=1;Require(CreatePipe(out read,out write,ref sa,0)&&SetHandleInformation(read,1,0),"Bounded stdout pipe");nul=CreateFileW("NUL",0x80000000,3,ref sa,3,0,IntPtr.Zero);Require(nul!=new IntPtr(-1)&&nul!=IntPtr.Zero&&nul!=write,"Original NUL/stdout handles");UIntPtr bytes=UIntPtr.Zero;InitializeProcThreadAttributeList(IntPtr.Zero,1,0,ref bytes);Require(bytes.ToUInt64()>0&&bytes.ToUInt64()<=65536,"Attribute size");list=Marshal.AllocHGlobal((int)bytes.ToUInt64());Require(InitializeProcThreadAttributeList(list,1,0,ref bytes),"Attributes initialize");attributesInitialized=true;handles=Marshal.AllocHGlobal(IntPtr.Size*2);Marshal.WriteIntPtr(handles,0,nul);Marshal.WriteIntPtr(handles,IntPtr.Size,write);Require(UpdateProcThreadAttribute(list,0,new UIntPtr(0x20002),handles,new UIntPtr((uint)(IntPtr.Size*2)),IntPtr.Zero,IntPtr.Zero),"Exact HANDLE_LIST2");SX sx=new SX();sx.si.cb=Marshal.SizeOf(typeof(SX));sx.si.desktop=name;sx.si.flags=0x100;sx.si.input=nul;sx.si.output=write;sx.si.error=write;sx.attributes=list;var command=new StringBuilder(Quote(fixture));foreach(string a in args)command.Append(" ").Append(Quote(a));Require(StartProducer(fixture,command,IntPtr.Zero,IntPtr.Zero,true,0x00080604,env,Path.GetDirectoryName(fixture),ref sx,out producer),"Suspended original producer");if(!AssignProcessToJobObject(job,producer.process)){TerminateProcess(producer.process,91);throw new Exception("Bind failed before resume");}long pc,e,k,u;Require(GetProcessTimes(producer.process,out pc,out e,out k,out u),"Producer creation");result["producer_pid"]=producer.pid;result["producer_creation_filetime"]=pc;CloseHandle(write);write=IntPtr.Zero;CloseHandle(nul);nul=IntPtr.Zero;records=new Records();records.Start(read);read=IntPtr.Zero;Require(ResumeThread(producer.thread)!=0xffffffff,"Resume producer");var runtimeClock=System.Diagnostics.Stopwatch.StartNew();uint pid=0;IntPtr window=IntPtr.Zero;var findClock=System.Diagnostics.Stopwatch.StartNew();while(window==IntPtr.Zero&&findClock.ElapsedMilliseconds<10000){bool overflow=false,duplicate=false,candidateQueryFailed=false;int count=0;SetLastError(0);bool ok=EnumDesktopWindows(desktop,delegate(IntPtr h,IntPtr l){if(++count>64){overflow=true;return false;}uint id;GetWindowThreadProcessId(h,out id);var title=new StringBuilder(256);GetWindowText(h,title,256);if(id!=producer.pid&&IsWindowVisible(h)&&title.ToString()=="dlpad"){IntPtr candidate=OpenProcess(0x101000,false,id);if(candidate==IntPtr.Zero)return true;try{bool inJob;if(!IsProcessInJob(candidate,job,out inJob)){candidateQueryFailed=true;return false;}if(!inJob)return true;if(window!=IntPtr.Zero&&window!=h){duplicate=true;return false;}window=h;pid=id;}finally{CloseHandle(candidate);}}return true;},IntPtr.Zero);int er=Marshal.GetLastWin32Error();Require(!overflow&&!duplicate&&!candidateQueryFailed&&(ok||er==0),"Bounded private main enumeration");Require(WaitForSingleObject(producer.process,0)==258,"CLI exited before HWND ACK");if(window==IntPtr.Zero)Thread.Sleep(20);}Require(window!=IntPtr.Zero&&pid!=0,"Original private main HWND deadline");observed=OpenProcess(0x101000,false,pid);Require(observed!=IntPtr.Zero,"Read-only child HANDLE");long creation;Require(GetProcessTimes(observed,out creation,out e,out k,out u)&&WaitForSingleObject(observed,0)==258,"Child creation/live");bool member;Require(IsProcessInJob(observed,job,out member)&&member,"Original parent Job membership");var image=new StringBuilder(32768);uint chars=32768;Require(QueryFullProcessImageNameW(observed,0,image,ref chars),"Child original image");using(var expected=Pin(installed,ExpectedCpu))using(var actualImage=Pin(image.ToString(),ExpectedCpu)){Require(Same(Identity(expected),Identity(actualImage)),"Original installed image nativeID");}result["child_actual_image_id_sha_matched"]=true;result["child_parent_job_verified"]=true;result["resume_ack_observed_by_original_hwnd"]=true;
   PI child=new PI();child.process=observed;child.pid=pid;Owned(window,child,creation);result["child_pid"]=pid;result["child_creation_filetime"]=creation;result["hwnd"]=window.ToInt64();
   // Observe owned runtime beyond the two-second startup deadline without global input.
   while(runtimeClock.ElapsedMilliseconds<4000){Owned(window,child,creation);Require(WaitForSingleObject(producer.process,0)==258&&WaitForSingleObject(observed,0)==258,"Original supervisor/GUI alive beyond startup");Accounting alive;Require(QueryInformationJobObject(job,1,out alive,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero)&&alive.active>=2,"Original live parent Job before action");Thread.Sleep(20);}
   Owned(window,child,creation);Require(WaitForSingleObject(producer.process,0)==258&&WaitForSingleObject(observed,0)==258,"Runtime remains live after startup deadline");result["runtime_observed_elapsed_ms"]=runtimeClock.ElapsedMilliseconds;result["startup_budget_seconds"]=2;result["runtime_survived_startup_budget"]=true;
   worker=new Thread(delegate(){IntPtr previous=GetThreadDesktop(GetCurrentThreadId());try{Require(SetThreadDesktop(desktop),"MTA private desktop");if(mode=="first"||mode=="restart"){var app=Activate(window,child,creation,output);Desired=mode=="first"?"밝게":null;ThemeObserve(app,window,child,creation,output,result,"settings");Owned(window,child,creation);Require(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero),"Original HWND WM_CLOSE");Require(WaitForSingleObject(observed,10000)==0,"GUI natural exit deadline");uint code;Require(GetExitCodeProcess(observed,out code)&&code==0,"GUI natural exit0");result["child_natural_exit_code"]=code;}else if(mode=="cancel"){Require(GetProcessTimes(producer.process,out pc,out e,out k,out u)&&pc==Convert.ToInt64(result["producer_creation_filetime"])&&WaitForSingleObject(producer.process,0)==258,"Original CLI group live before CTRL_BREAK");Require(producer.pid!=0&&GenerateConsoleCtrlEvent(1,producer.pid),"Original nonzero CLI group CTRL_BREAK");result["ctrl_break_group_pid"]=producer.pid;}}catch(Exception err){workerError=err;}finally{restored=SetThreadDesktop(previous);}});worker.IsBackground=true;worker.SetApartmentState(ApartmentState.MTA);worker.Start();Require(worker.Join(90000),"UIA deadline");if(workerError!=null)throw workerError;Require(restored,"MTA restore");result["mta_desktop_restored"]=true;var terminal=records.Next(25000);Save(Path.Combine(output,"cli-receipt.json"),terminal);bool normal=mode=="first"||mode=="restart";Require(Text(terminal,"phase")=="run-wait","CLI terminal phase");Require(Text(terminal,"status")== (normal?"RAN_SELECTED_DEVELOPMENT_ONLY":"FAIL"),"CLI expected status");Require(Text(terminal,"reason")== (normal?"Natural":mode=="cancel"?"Cancelled":"TimedOut"),"CLI expected reason");if(!normal)Require(Text(terminal,"kind")== (mode=="cancel"?"Interrupted":"TimedOut"),"CLI expected typed error");else Require(Number(terminal,"exit_code")==0,"GUI natural0 receipt");Require(Number(terminal,"startup_timeout_seconds")==2&&terminal.ContainsKey("runtime_deadline_seconds")&&terminal["runtime_deadline_seconds"]==null,"Startup-only deadline receipt");Flag(terminal,"runtime_until_cancel_or_exit");Require(terminal.ContainsKey("permanent_detach")&&!Convert.ToBoolean(terminal["permanent_detach"]),"No detached runtime");Flag(terminal,"job_terminal_confirmed");Flag(terminal,"child_created");Flag(terminal,"resume_attempted");Flag(terminal,"may_have_executed");Require(Number(terminal,"child_creation_filetime")==checked((ulong)creation),"Receipt original child creation");Require(WaitForSingleObject(observed,5000)==0,"Original GUI terminal");Require(WaitForSingleObject(producer.process,5000)==0,"CLI natural exit");uint exit;Require(GetExitCodeProcess(producer.process,out exit)&&exit==(normal?0u:1u),"CLI expected natural exit");result["producer_natural_exit_code"]=exit;Require(records.thread.Join(5000)&&records.error==null&&records.queue.Count==0,"Bounded stdout complete");Require(NaturalJobEmpty(job,result),"Parent natural Job0 deadline/query failure");result["parent_job_naturally_empty"]=true;result["status"]="REVIEW_REQUIRED";
  }catch(Exception err){result["error"]=err.GetType().Name+": "+err.Message;}
  finally{ObserverStopping=true;if(job!=IntPtr.Zero){Accounting a;bool natural=QueryInformationJobObject(job,1,out a,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero)&&a.active==0;result["parent_job_active_before_cleanup"]=natural?0:-1;if(!natural){forced=true;TerminateJobObject(job,94);result["status"]="FAIL";}bool empty=false;for(int i=0;i<100;i++){if(QueryInformationJobObject(job,1,out a,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero)&&a.active==0){empty=true;break;}Thread.Sleep(50);}result["parent_job_empty_after_cleanup"]=empty;if(!empty)result["status"]="FAIL";}
   bool drained=worker==null||!worker.IsAlive||worker.Join(5000);bool readerDone=records==null||records.thread.Join(5000);result["worker_drained"]=drained;result["stdout_reader_drained"]=readerDone;result["forced_cleanup"]=forced;if(!drained||!readerDone)result["status"]="FAIL";
   if(previousDesktop!=IntPtr.Zero&&!SetThreadDesktop(previousDesktop))result["status"]="FAIL";
   if(drained){bool consoleDetached=!hiddenConsole||FreeConsole();result["hidden_console_detached"]=consoleDetached;if(!consoleDetached)result["status"]="FAIL";if(observed!=IntPtr.Zero)CloseHandle(observed);if(producer.thread!=IntPtr.Zero)CloseHandle(producer.thread);if(producer.process!=IntPtr.Zero)CloseHandle(producer.process);if(job!=IntPtr.Zero)CloseHandle(job);if(desktop!=IntPtr.Zero){if(consoleDetached){bool closed=CloseDesktop(desktop);result["private_desktop_closed"]=closed;if(!closed)result["status"]="FAIL";}else{result["private_desktop_closed"]=false;result["original_desktop_handle_retained_until_process_exit"]=true;}}}
   else{result["status"]="FAIL";result["private_desktop_closed"]=false;result["hidden_console_detached"]=false;result["original_desktop_handle_retained_until_process_exit"]=desktop!=IntPtr.Zero;result["original_observation_handle_retained_until_process_exit"]=observed!=IntPtr.Zero;result["original_producer_process_handle_retained_until_process_exit"]=producer.process!=IntPtr.Zero;result["original_producer_thread_handle_retained_until_process_exit"]=producer.thread!=IntPtr.Zero;result["original_parent_job_handle_retained_until_process_exit"]=job!=IntPtr.Zero;result["hidden_console_retained_until_process_exit"]=hiddenConsole;}
   if(env!=IntPtr.Zero)Marshal.FreeHGlobal(env);if(read!=IntPtr.Zero)CloseHandle(read);if(write!=IntPtr.Zero)CloseHandle(write);if(nul!=IntPtr.Zero)CloseHandle(nul);if(list!=IntPtr.Zero){if(attributesInitialized)DeleteProcThreadAttributeList(list);Marshal.FreeHGlobal(list);}if(handles!=IntPtr.Zero)Marshal.FreeHGlobal(handles);try{string after=Input();result["input_after"]=after;result["input_desktop_unchanged"]=after==(string)result["input_before"];if(!(bool)result["input_desktop_unchanged"])result["status"]="FAIL";}catch{result["status"]="FAIL";}}
  Save(Path.Combine(output,"result.json"),result);return result;
 }
    [DllImport("kernel32.dll")] static extern uint GetConsoleCP();
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool FreeConsole();
    [StructLayout(LayoutKind.Sequential)] struct ConsoleOptions { public int mode; public int useShowWindow; public ushort showWindow; }
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] delegate int AllocHidden(ref ConsoleOptions options,out int result);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] static extern IntPtr GetModuleHandleW(string name);
    [DllImport("kernel32.dll",CharSet=CharSet.Ansi,ExactSpelling=true)] static extern IntPtr GetProcAddress(IntPtr module,string name);
    static void AllocateHidden(Dictionary<string,object> receipt) { IntPtr fn=GetProcAddress(GetModuleHandleW("kernel32.dll"),"AllocConsoleWithOptions");receipt["hidden_console_api_available"]=fn!=IntPtr.Zero;if(fn==IntPtr.Zero)throw new Exception("hidden_console_api_unavailable");var alloc=(AllocHidden)Marshal.GetDelegateForFunctionPointer(fn,typeof(AllocHidden));ConsoleOptions options=new ConsoleOptions();options.mode=2;options.useShowWindow=1;options.showWindow=0;int result;int hr=alloc(ref options,out result);receipt["hidden_console_hresult"]=hr;receipt["hidden_console_result"]=result;receipt["hidden_console_mode"]=2;receipt["hidden_console_show_window"]=0;if(hr!=0||result!=1)throw new Exception("hidden_console_allocation_failed"); }
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool SetStdHandle(int kind,IntPtr value);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool GenerateConsoleCtrlEvent(uint kind,uint group);
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool IsProcessInJob(IntPtr process,IntPtr job,out bool member);
 static IntPtr EnvironmentBlock(string root){var env=new SortedDictionary<string,string>(StringComparer.OrdinalIgnoreCase);foreach(System.Collections.DictionaryEntry row in Environment.GetEnvironmentVariables()){string key=(string)row.Key;if(!key.StartsWith("=",StringComparison.Ordinal))env[key]=(string)row.Value;}env["DLPAD_TEST_DATA"]=Path.Combine(root,"data");env["DLPAD_LIBRARY_ROOT"]=Path.Combine(root,"media");env["TEMP"]=Path.Combine(root,"tmp");env["TMP"]=Path.Combine(root,"tmp");var b=new StringBuilder();foreach(var row in env)b.Append(row.Key).Append('=').Append(row.Value).Append('\0');b.Append('\0');Require(b.Length<=32767,"Environment bound");return Marshal.StringToHGlobalUni(b.ToString());}
 static bool NaturalJobEmpty(IntPtr job,Dictionary<string,object> result){var clock=System.Diagnostics.Stopwatch.StartNew();int iterations=0;uint active=UInt32.MaxValue,total=0,terminated=0;int lastError=0;bool queried=false,empty=false;do{Accounting a;iterations++;queried=QueryInformationJobObject(job,1,out a,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero);lastError=queried?0:Marshal.GetLastWin32Error();if(!queried)break;active=a.active;total=a.total;terminated=a.terminated;if(active==0){empty=true;break;}Thread.Sleep(20);}while(clock.ElapsedMilliseconds<5000);result["parent_natural_job_poll"]=new Dictionary<string,object>{{"query_succeeded",queried},{"last_os_error",lastError},{"active_processes",active},{"total_processes",total},{"terminated_processes",terminated},{"iterations",iterations},{"elapsed_ms",clock.ElapsedMilliseconds},{"deadline_ms",5000},{"natural_empty_confirmed",empty}};return empty;}
 public static int Main(string[] args){Console.OutputEncoding=new UTF8Encoding(false);var summary=new Dictionary<string,object>{{"status","FAIL"},{"pixel_theme_verified",false},{"installer_complete",false}};bool fresh=false;try{Require(args.Length==9,"Expected nine explicit arguments");ExpectedCpu=args[3];foreach(int i in new int[]{0,2,4,5,6,7,8})S10Plain(args[i]);string root=args[7];Require(Path.GetFileName(root).StartsWith("dlpad-launch-cli-owned-",StringComparison.Ordinal),"Owned data namespace");foreach(string leaf in new string[]{"data","media","tmp"}){string p=Path.Combine(root,leaf);S10Plain(p);Require(Directory.Exists(p),"Owned app fixture child");}using(var cli=Pin(args[0],args[1]))using(var cpu=Pin(args[2],args[3])){S10PrivateNewDirectory(args[8]);fresh=true;foreach(string mode in new string[]{"first","restart","cancel"}){string output=Path.Combine(args[8],mode);S10PrivateNewDirectory(output);string[] command={"run-selected","--target",args[4],"--receipt-target",args[5],"--active-target",args[6],"--startup-timeout-seconds","2"};var row=One(args[0],command,output,mode,args[2],root);summary[mode]=row;Require(Text(row,"status")=="REVIEW_REQUIRED","CLI GUI case failed");}using(var c=Pin(args[0],args[1]))using(var p=Pin(args[2],args[3]))Require(Same(Identity(cli),Identity(c))&&Same(Identity(cpu),Identity(p)),"Original path IDs after suite");Require(Hash(cli)==args[1]&&Hash(cpu)==args[3],"Source pins after suite");summary["status"]="REVIEW_REQUIRED";summary["sqlite_and_pixels_require_root_review"]=true;}}catch(Exception e){summary["error"]=e.GetType().Name+": "+e.Message;}if(fresh)Save(Path.Combine(args[8],"run-selected-cli-result.json"),summary);return Text(summary,"status")=="REVIEW_REQUIRED"?0:1;}
}
