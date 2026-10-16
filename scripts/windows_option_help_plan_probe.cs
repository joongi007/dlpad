using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Automation;
using System.Web.Script.Serialization;
public static class UiReworkPrivateGui {
 static volatile bool ObserverStopping;
 static string PinnedExe;
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
 [DllImport("user32.dll")] static extern bool EnumDesktopWindows(IntPtr desktop,EnumFn callback,IntPtr l);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h,StringBuilder text,int length);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h,StringBuilder text,int length);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll",EntryPoint="PostMessageW",ExactSpelling=true,SetLastError=true)] static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
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
 static void Owned(IntPtr window,PI pi,long start){Require(!ObserverStopping,"Observer cooperative stop");uint pid;GetWindowThreadProcessId(window,out pid);long actual,e,k,u;Require(pid==pi.pid && WaitForSingleObject(pi.process,0)==258,"Owned HWND/process identity");Require(GetProcessTimes(pi.process,out actual,out e,out k,out u)&&actual==start,"Owned process start identity");}
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
 string partial=Read(e);if(partial!=""){Require(partial.Length<=1024,"Partial input clear bound");Require(PostMessage(window,0x100,new IntPtr(0x23),new IntPtr(1)),"Owned End down");Require(PostMessage(window,0x101,new IntPtr(0x23),new IntPtr(unchecked((int)0xc0000001))),"Owned End up");Pump(window,pi,start,150);for(int i=0;i<partial.Length;i++){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(8),new IntPtr(1)),"Owned Backspace down");Require(PostMessage(window,0x101,new IntPtr(8),new IntPtr(unchecked((int)0xc0000001))),"Owned Backspace up");Pump(window,pi,start,150);}string cleared=Read(e);InputEvidence(e,value,cleared,output,"clear",window,pi,start);Require(cleared=="","Bounded local backspace did not clear field; no append fallback");}
 foreach(char c in value){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(0x87),new IntPtr(1)),"Owned non-text F24 envelope down");Require(PostMessage(window,0x102,new IntPtr(c),new IntPtr(1)),"Owned WM_CHAR");Require(PostMessage(window,0x101,new IntPtr(0x87),new IntPtr(unchecked((int)0xc0000001))),"Owned non-text F24 envelope up");Pump(window,pi,start,150);}string actual=Read(e);for(int i=0;i<40&&actual!=value;i++){Pump(window,pi,start,50);actual=Read(e);}InputEvidence(e,value,actual,output,"readback",window,pi,start);Require(actual==value,"Owned input readback mismatch: "+e.Current.Name+" expected_length="+value.Length+" actual_length="+actual.Length);return "owned-F24-envelope-WM_CHAR/readback (UIA setter unavailable/unapplied)";}
 static void Invoke(AutomationElement e){object pattern=null;Require(e.Current.IsEnabled&&e.TryGetCurrentPattern(InvokePattern.Pattern,out pattern),"Invoke pattern unavailable/disabled");((InvokePattern)pattern).Invoke();}
 static void Capture(IntPtr window,PI pi,long start,string path){Owned(window,pi,start);Rect r;Require(GetClientRect(window,out r)&&r.right>0&&r.bottom>0&&r.right<=4096&&r.bottom<=4096,"Screenshot dimensions");using(var bitmap=new System.Drawing.Bitmap(r.right,r.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool ok;try{ok=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(ok,"Private PrintWindow unsupported");using(var f=new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.None))bitmap.Save(f,System.Drawing.Imaging.ImageFormat.Png);Require(new FileInfo(path).Length<=16777216,"PNG byte bound");}}
 static void Save(string path,object value){string text=new JavaScriptSerializer().Serialize(value);Require(Encoding.UTF8.GetByteCount(text)<=65536,"Evidence JSON limit");File.WriteAllText(path,text,new UTF8Encoding(false));}

 static Dictionary<string,object> Patterns(AutomationElement e){object p;var row=new Dictionary<string,object>();row["name"]=e.Current.Name;row["type"]=e.Current.ControlType.ProgrammaticName;row["enabled"]=e.Current.IsEnabled;row["keyboard_focusable"]=e.Current.IsKeyboardFocusable;row["has_keyboard_focus"]=e.Current.HasKeyboardFocus;row["invoke"]=e.TryGetCurrentPattern(InvokePattern.Pattern,out p);row["scroll_item"]=e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out p);row["text"]=e.TryGetCurrentPattern(TextPattern.Pattern,out p);AutomationProperty text2=AutomationProperty.LookupById(30119);if(text2!=null){object availability=e.GetCurrentPropertyValue(text2,true);row["text_pattern2_available"]=availability is bool?(object)(bool)availability:"unsupported";}bool hasValue=e.TryGetCurrentPattern(ValuePattern.Pattern,out p);row["value"]=hasValue;if(hasValue)row["read_only"]=((ValuePattern)p).Current.IsReadOnly;return row;}
 static bool InContent(AutomationElement e,IntPtr window,PI pi,long start,out Point point){Owned(window,pi,start);point=new Point();var b=e.Current.BoundingRectangle;if(b.IsEmpty||b.Width<=0||b.Height<=0||e.Current.IsOffscreen)return false;point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);Rect r;if(!ScreenToClient(window,ref point)||!GetClientRect(window,out r)||point.x<0||point.y<0||point.x>=r.right||point.y>=r.bottom)return false;AutomationElement app=AutomationElement.FromHandle(window);foreach(AutomationElement n in Nodes(app)){
 if(n.Current.ControlType==ControlType.Button&&(n.Current.Name=="다운로드"||n.Current.Name=="설정"||n.Current.Name=="파일 목록"||n.Current.Name=="도구"||n.Current.Name=="작업 이력")){
  var tabs=n.Current.BoundingRectangle;Point edge=new Point();edge.x=(int)tabs.Left;edge.y=(int)Math.Ceiling(tabs.Bottom);
  if(!ScreenToClient(window,ref edge)||point.y<=edge.y)return false;
 }
 if(n.Current.ControlType==ControlType.Button&&(n.Current.Name=="\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791"||n.Current.Name=="선택 구간 저장")){var footer=n.Current.BoundingRectangle;Point top=new Point();top.x=(int)footer.Left;top.y=(int)footer.Top;if(!ScreenToClient(window,ref top)||point.y>=top.y)return false;}}return true;}
 static void EnsureContent(AutomationElement target,IntPtr window,PI pi,long start,string output){
  var rows=new List<Dictionary<string,object>>();
  Owned(window,pi,start);Point visible;
  if(InContent(target,window,pi,start,out visible)){Save(Path.Combine(output,"s10-scroll.json"),rows);return;}
  object scroll;
  if(target.TryGetCurrentPattern(ScrollItemPattern.Pattern,out scroll)){
   var before=Geometry(target,window,pi,start);
   ((ScrollItemPattern)scroll).ScrollIntoView();Pump(window,pi,start,300);
   rows.Add(new Dictionary<string,object>{{"route","exact-target-ScrollItemPattern"},{"before",before},{"after",Geometry(target,window,pi,start)}});
   Save(Path.Combine(output,"s10-scroll.json"),rows);
   if(InContent(target,window,pi,start,out visible))return;
  }
  // Wheel only inside the actual content viewport, never over the first editable field.
  // Pump move separately so egui receives a painted hover position before wheel delivery.
  for(int attempt=0;attempt<32;attempt++){
   if(InContent(target,window,pi,start,out visible)){Save(Path.Combine(output,"s10-scroll.json"),rows);return;}
   Owned(window,pi,start);Rect client;Require(GetClientRect(window,out client),"Owned scroll client");
   int top=0,bottom=client.bottom;
   var app=AutomationElement.FromHandle(window);
   foreach(var node in Nodes(app)){
    if(node.Current.ControlType!=ControlType.Button)continue;
    string label=node.Current.Name;var rectangle=node.Current.BoundingRectangle;
    Point edge=new Point();edge.x=(int)rectangle.Left;
    if(label=="다운로드"||label=="설정"||label=="파일 목록"||label=="도구"||label=="작업 이력"){
     edge.y=(int)Math.Ceiling(rectangle.Bottom);Require(ScreenToClient(window,ref edge),"Owned navigation edge");top=Math.Max(top,edge.y);
    }
    if((label=="다운로드 시작"||label=="선택 구간 저장")){
     edge.y=(int)Math.Floor(rectangle.Top);Require(ScreenToClient(window,ref edge),"Owned action edge");bottom=Math.Min(bottom,edge.y);
    }
   }
   Require(bottom-top>48&&client.right>64,"Actual content viewport unavailable");
   Point local=new Point();local.x=24;local.y=top+(bottom-top)/2;
   Point targetCenter=new Point();var bounds=target.Current.BoundingRectangle;
   Require(!bounds.IsEmpty,"Target scroll geometry absent");
   targetCenter.x=(int)(bounds.Left+bounds.Width/2);targetCenter.y=(int)(bounds.Top+bounds.Height/2);
   Require(ScreenToClient(window,ref targetCenter),"S10 target coordinate");int delta=targetCenter.y<local.y?120:-120;
   Point screen=local;Require(ClientToScreen(window,ref screen),"S10 wheel coordinate");
   var row=new Dictionary<string,object>{{"attempt",attempt},{"route","owned-content-wheel"},{"target",target.Current.Name},{"wheel_delta",delta},{"target_client_y_before",targetCenter.y},{"anchor_client_x",local.x},{"anchor_client_y",local.y},{"viewport_top",top},{"viewport_bottom",bottom}};
   rows.Add(row);Save(Path.Combine(output,"s10-scroll.json"),rows);Owned(window,pi,start);
   Require(PostMessage(window,0x200,IntPtr.Zero,new IntPtr((local.y<<16)|(local.x&65535))),"S10 owned mouse move");Pump(window,pi,start,150);
   Require(PostMessage(window,0x20a,new IntPtr(delta<<16),new IntPtr((screen.y<<16)|(screen.x&65535))),"S10 owned wheel");Pump(window,pi,start,200);
   row["after"]=Geometry(target,window,pi,start);Save(Path.Combine(output,"s10-scroll.json"),rows);
  }
  Capture(window,pi,start,Path.Combine(output,"s10-scroll-failed.png"));throw new Exception("S10 control not visibly reachable within bounded scroll");
 }
 static void BoundsClick(AutomationElement e,IntPtr window,PI pi,long start,string output){EnsureContent(e,window,pi,start,output);object pattern;if(e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out pattern)){((ScrollItemPattern)pattern).ScrollIntoView();Pump(window,pi,start,300);}var b=e.Current.BoundingRectangle;Require(!b.IsEmpty&&!e.Current.IsOffscreen&&b.Width>0&&b.Height>0,"Report control bounds unavailable");Point point=new Point();point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);Owned(window,pi,start);Rect r;Require(ScreenToClient(window,ref point)&&GetClientRect(window,out r)&&point.x>=0&&point.y>=0&&point.x<r.right&&point.y<r.bottom,"Report control outside owned client");IntPtr xy=new IntPtr((point.y<<16)|(point.x&65535));Require(PostMessage(window,0x200,IntPtr.Zero,xy),"Report owned move");Require(PostMessage(window,0x201,new IntPtr(1),xy),"Report owned down");Require(PostMessage(window,0x202,IntPtr.Zero,xy),"Report owned up");Pump(window,pi,start,300);}
 static bool HasLabel(AutomationElement app,string name){foreach(AutomationElement e in Nodes(app))if(e.Current.Name==name)return true;return false;}
 static void Privacy(string text,string root){var forbidden=new List<string>();forbidden.Add("windows-download-fixture-only");forbidden.Add(Path.Combine(root,"source.cookies"));string calls=Path.Combine(root,"calls.jsonl");Require(File.Exists(calls)&&new FileInfo(calls).Length<=16384,"Report toolcalls evidence missing/bounded");foreach(string line in File.ReadAllLines(calls)){if(line.Length==0)continue;var call=new JavaScriptSerializer().Deserialize<Dictionary<string,object>>(line);object path;if(call.TryGetValue("cookie_path",out path)&&path is string)forbidden.Add((string)path);}foreach(string value in forbidden){Require(!text.Contains(value),"Report synthetic secret/private path leaked");string escaped=new JavaScriptSerializer().Serialize(value);Require(!text.Contains(escaped.Substring(1,escaped.Length-2)),"Report escaped private value leaked");}}
 static Dictionary<string,object> Geometry(AutomationElement target,IntPtr window,PI pi,long start){Owned(window,pi,start);var b=target.Current.BoundingRectangle;Point rawPoint=new Point();rawPoint.x=(int)(b.Left+b.Width/2);rawPoint.y=(int)(b.Top+b.Height/2);Require(ScreenToClient(window,ref rawPoint),"Owned geometry coordinate conversion");Point visiblePoint;var row=new Dictionary<string,object>();row["name"]=target.Current.Name;row["screen_top"]=b.Top;row["screen_left"]=b.Left;row["width"]=b.Width;row["height"]=b.Height;row["client_x"]=rawPoint.x;row["client_y"]=rawPoint.y;row["is_offscreen"]=target.Current.IsOffscreen;row["in_owned_content"]=InContent(target,window,pi,start,out visiblePoint);row["has_keyboard_focus"]=target.Current.HasKeyboardFocus;return row;}
 static bool PreviewIntersection(AutomationElement preview,AutomationElement app,IntPtr window,PI pi,long start,string output){Owned(window,pi,start);Rect client=new Rect();Require(GetClientRect(window,out client),"Owned preview clip client");var bounds=preview.Current.BoundingRectangle;Point origin=new Point();origin.x=(int)Math.Floor(bounds.Left);origin.y=(int)Math.Floor(bounds.Top);Require(ScreenToClient(window,ref origin),"Owned preview clip transform");int bottom=client.bottom;foreach(AutomationElement n in Nodes(app)){if(n.Current.ControlType==ControlType.Button&&(n.Current.Name=="\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791"||n.Current.Name=="선택 구간 저장")){var footer=n.Current.BoundingRectangle;Point edge=new Point();edge.x=(int)footer.Left;edge.y=(int)footer.Top;Require(ScreenToClient(window,ref edge),"Owned footer clip transform");bottom=Math.Min(bottom,edge.y);}}
 int left=Math.Max(0,origin.x),top=Math.Max(0,origin.y),right=Math.Min(client.right,(int)Math.Ceiling(origin.x+bounds.Width)),lower=Math.Min(bottom,(int)Math.Ceiling(origin.y+bounds.Height));bool intersects=!preview.Current.IsOffscreen&&right>left&&lower>top;var evidence=new Dictionary<string,object>();evidence["uia_is_offscreen"]=preview.Current.IsOffscreen;evidence["uia_width"]=bounds.Width;evidence["uia_height"]=bounds.Height;evidence["intersection_left"]=left;evidence["intersection_top"]=top;evidence["intersection_width"]=Math.Max(0,right-left);evidence["intersection_height"]=Math.Max(0,lower-top);evidence["geometry_intersects"]=intersects;evidence["actual_glyph_visibility_verified"]=false;evidence["requires_root_private_image_review"]=true;Save(Path.Combine(output,"report-preview-visible-intersection.json"),evidence);if(intersects){Require(client.right<=4096&&client.bottom<=4096,"Owned preview screenshot limit");using(var bitmap=new System.Drawing.Bitmap(client.right,client.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool printed;try{printed=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(printed,"Owned preview clip PrintWindow");using(var slice=bitmap.Clone(new System.Drawing.Rectangle(left,top,right-left,lower-top),bitmap.PixelFormat)){slice.Save(Path.Combine(output,"report-preview-visible-intersection.png"),System.Drawing.Imaging.ImageFormat.Png);}}}return intersects;}
 static void FocusProbe(AutomationElement target,IntPtr window,PI pi,long start,string output,string slug,bool endKey){Save(Path.Combine(output,slug+"-before.json"),Geometry(target,window,pi,start));Capture(window,pi,start,Path.Combine(output,slug+"-before.png"));Require(target.Current.IsKeyboardFocusable,"Owned reachability target not focusable");target.SetFocus();Pump(window,pi,start,300);Require(target.Current.HasKeyboardFocus,"Owned reachability focus not reflected");if(endKey){Require(PostMessage(window,0x100,new IntPtr(0x23),new IntPtr(1)),"Reachability owned End down");Require(PostMessage(window,0x101,new IntPtr(0x23),new IntPtr(unchecked((int)0xc0000001))),"Reachability owned End up");Pump(window,pi,start,500);}Save(Path.Combine(output,slug+"-after.json"),Geometry(target,window,pi,start));Capture(window,pi,start,Path.Combine(output,slug+"-after.png"));}
 static void S10Click(AutomationElement app,string label,IntPtr window,PI pi,long start,string output){
  S10Step(output,"click:"+label,"begin");
  Owned(window,pi,start);
  var snapshot=new List<Dictionary<string,object>>();AutomationElement button=null;
  foreach(var node in Nodes(app)){
   snapshot.Add(new Dictionary<string,object>{{"name",node.Current.Name},{"type",node.Current.ControlType.ProgrammaticName},{"enabled",node.Current.IsEnabled}});
   if(node.Current.Name==label&&(node.Current.ControlType==ControlType.Button||node.Current.ControlType==ControlType.RadioButton||node.Current.ControlType==ControlType.CheckBox||node.Current.ControlType==ControlType.TabItem)){
    Require(button==null,"Duplicate S10 action "+label);button=node;
   }
  }
  Save(Path.Combine(output,"s10-action-tree.json"),snapshot);
  Capture(window,pi,start,Path.Combine(output,"s10-action-before.png"));
  Require(button!=null,"Missing S10 action "+label);object action;
  if(button.TryGetCurrentPattern(ScrollItemPattern.Pattern,out action)){((ScrollItemPattern)action).ScrollIntoView();Pump(window,pi,start,120);}
  if(button.Current.IsKeyboardFocusable){button.SetFocus();Pump(window,pi,start,120);}
  EnsureContent(button,window,pi,start,output);
  Require(button.Current.IsEnabled,"S10 control disabled: "+label);
  object invoke;
  if(button.TryGetCurrentPattern(InvokePattern.Pattern,out invoke)){((InvokePattern)invoke).Invoke();}
  else if(button.Current.IsKeyboardFocusable){
   button.SetFocus();Pump(window,pi,start,300);Require(button.Current.HasKeyboardFocus,"S10 owned focus not reflected: "+label);
   EnsureContent(button,window,pi,start,output);Owned(window,pi,start);
   Require(PostMessage(window,0x100,new IntPtr(0x20),new IntPtr(1)),"S10 owned Space down");
   Require(PostMessage(window,0x101,new IntPtr(0x20),new IntPtr(unchecked((int)0xc0000001))),"S10 owned Space up");
  }
  else{BoundsClick(button,window,pi,start,output);}
  Pump(window,pi,start,300);
  S10Step(output,"click:"+label,"end");
 }
 static void S10Field(AutomationElement app,string label,string text,IntPtr window,PI pi,long start,string output){
  S10Step(output,"field:"+label,"begin");
  var field=Find(app,label,ControlType.Edit);Require(field.Current.IsKeyboardFocusable,"S10 field focus unsupported");
  field.SetFocus();Pump(window,pi,start,160);string stage=Path.Combine(output,"input-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(stage);Fill(field,text,window,pi,start,stage,true);Require(Read(field)==text,"S10 exact field readback");
  S10Step(output,"field:"+label,"end");
 }
 static string S10Export(AutomationElement app,string root,string output,string slug,string format,IntPtr window,PI pi,long start){
  S10Step(output,"export:"+slug,"begin");
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
  S10CheckInvariant(text,format);S10Step(output,"export:"+slug,"end");
  return text;
 }
 static Dictionary<string,object> S10Json(string text){return new JavaScriptSerializer(){MaxJsonLength=2097152}.Deserialize<Dictionary<string,object>>(text);}
 static int S10Count(Dictionary<string,object> report,string key){object value;Require(report.TryGetValue(key,out value),"S10 expected JSON field missing");var list=value as System.Collections.ICollection;Require(list!=null,"S10 JSON array shape");return list.Count;}
 static void S10WriteNew(string path,byte[] bytes){using(var f=new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.None)){f.Write(bytes,0,bytes.Length);f.Flush(true);}}
 static string S10Scenario="save";
 static string S10FailureMode;
 static string S10SourceHash;
 static readonly System.Diagnostics.Stopwatch S10Clock=System.Diagnostics.Stopwatch.StartNew();
 static readonly List<Dictionary<string,object>> S10Steps=new List<Dictionary<string,object>>();
 static void S10Step(string output,string operation,string phase){
  lock(S10Steps){Require(S10Steps.Count<256,"Step checkpoint bound");S10Steps.Add(new Dictionary<string,object>{{"operation",operation},{"phase",phase},{"elapsed_ms",S10Clock.ElapsedMilliseconds},{"utc",DateTime.UtcNow.ToString("o")}});Save(Path.Combine(output,"s10-steps.json"),S10Steps);}
 }
 static string S10Invariant;
 static void S10CheckInvariant(string text,string format){
  if(format!="JSON"){if(S10Invariant!=null){var fields=S10Json(S10Invariant);Require(text.Contains((string)fields["report_id"]),"Markdown report identity changed");}return;}
  var report=S10Json(text);var immutable=new Dictionary<string,object>();
  foreach(string key in new[]{"report_id","attempt_id","stage","exit_code","tools"}){object value;Require(report.TryGetValue(key,out value),"Report immutable field absent");immutable[key]=value;}
  string actual=new JavaScriptSerializer().Serialize(immutable);if(S10Invariant==null)S10Invariant=actual;else Require(actual==S10Invariant,"Report identity/stage/attempt/tools changed during GUI scenario");
 }
 static void S10Attach(AutomationElement app,string path,IntPtr window,PI pi,long start,string output){
  S10Field(app,"추가할 로컬 MPD 경로 (최대 1 MiB)",path,window,pi,start,output);S10Click(app,"구조만 첨부",window,pi,start,output);
  for(int i=0;i<80&&!Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled;i++)Pump(window,pi,start,100);
  Require(Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled,"Attachment worker did not return report");
 }
 static void VerifyDiagnostic(AutomationElement app,IntPtr window,PI pi,long start,string root,string output,Dictionary<string,object> result,bool reachability){
  result["scenario"]=S10Scenario;result["clipboard_invoked"]=false;result["paste_invoked"]=false;result["closewait_inflight_verified"]=false;
  result["s10_full_suite_verified"]=false;S10Step(output,"scenario:"+S10Scenario,"begin");
  S10Click(app,"실패 원인 보기",window,pi,start,output);
  var initial=S10Json(S10Export(app,root,output,"s10-initial","JSON",window,pi,start));int initialLogs=S10Count(initial,"logs");Require(initialLogs>=1,"Fixture must expose sanitized omission log");
  Require((string)initial["stage"]==(S10FailureMode=="metadata-failure"?"Metadata":"Download")&&Convert.ToInt32(initial["exit_code"])==(S10FailureMode=="metadata-failure"?7:9),"Expected synthetic execution stage/exit not reached");
  int versions=0,metadata=0,downloads=0;string calls=Path.Combine(root,"calls.jsonl");Require(File.Exists(calls)&&new FileInfo(calls).Length<=16384,"Actual fixture calls missing/bounded");
  foreach(string line in File.ReadAllLines(calls)){if(line.Length==0)continue;var call=S10Json(line);string stage=(string)call["stage"];if(stage=="version")versions++;else if(stage=="metadata")metadata++;else if(stage=="download")downloads++;else throw new Exception("Unexpected fixture stage");if(stage!="version")Require((string)call["literal_sha256"]==S10SourceHash,"Fixture did not inspect original synthetic cookie bytes");}
  Require(versions>=1&&metadata==1&&(S10FailureMode=="metadata-failure"?downloads==0:downloads==1),"Actual synthetic call sequence mismatch");
  Require(S10Count(initial,"mpd_attachments")==0,"Fresh failure report already has attachments");result["actual_fixture_stage_exit_verified"]=true;
  if(S10Scenario=="save"){
   S10Export(app,root,output,"s10-markdown","Markdown",window,pi,start);
  }else if(S10Scenario=="save-collision"){
   string sentinel=Path.Combine(root,"sentinel"),before=Hash(File.ReadAllText(sentinel));S10Field(app,"진단 자료 저장 경로",sentinel,window,pi,start,output);S10Click(app,"새 파일로 저장",window,pi,start,output);
   for(int i=0;i<80&&!HasLabel(app,"진단 자료 작업을 완료하지 못했습니다. 입력 파일과 기존 저장 파일은 유지됩니다.");i++)Pump(window,pi,start,100);
   Require(HasLabel(app,"진단 자료 작업을 완료하지 못했습니다. 입력 파일과 기존 저장 파일은 유지됩니다.")&&Hash(File.ReadAllText(sentinel))==before,"Create-only sentinel changed");
   Require(Find(app,"새 파일로 저장",ControlType.Button).Current.IsEnabled,"Collision worker did not return report");
  }else if(S10Scenario=="edit"){
   S10Field(app,"로그 번호 (0부터)","0",window,pi,start,output);S10Click(app,"선택한 로그 수정",window,pi,start,output);
   S10Field(app,"선택한 로그 수정","S10 public observation https://fixture.invalid/?signature=synthetic-value",window,pi,start,output);S10Click(app,"정제 후 반영",window,pi,start,output);
   string edited=S10Export(app,root,output,"s10-edited","JSON",window,pi,start);
   Require(edited.Contains("S10 public observation")&&!edited.Contains("fixture.invalid")&&!edited.Contains("synthetic-value")&&S10Count(S10Json(edited),"logs")==initialLogs,"Edited log resanitization/count mismatch");
  }else if(S10Scenario=="omit-one"||S10Scenario=="omit-all"){
   bool one=S10Scenario=="omit-one";if(one)S10Field(app,"로그 번호 (0부터)","0",window,pi,start,output);
   S10Click(app,one?"선택한 로그 제외":"로그 제외",window,pi,start,output);
   var omitted=S10Json(S10Export(app,root,output,"s10-omitted","JSON",window,pi,start));Require(S10Count(omitted,"logs")== (one?initialLogs-1:0),"Nonempty log omission mismatch");
   result["all_logs_nonempty_before_action_verified"]=!one;result["log_count_before_action"]=initialLogs;
  }else{
   string mpd=Path.Combine(root,"s10-fixture.mpd");byte[] bytes=Encoding.UTF8.GetBytes("<MPD><BaseURL>https://fixture.invalid/?signature=mpd-fixture</BaseURL><SegmentList><Initialization range=\"0-10\"/><SegmentURL mediaRange=\"11-20\"/></SegmentList></MPD>");S10WriteNew(mpd,bytes);S10Attach(app,mpd,window,pi,start,output);Require(HasLabel(app,"MPD 0"),"Original attachment structure absent before action");
   if(S10Scenario=="mpd-attach"){
    string attached=S10Export(app,root,output,"s10-attached","JSON",window,pi,start);Require(S10Count(S10Json(attached),"mpd_attachments")==1&&!attached.Contains("mpd-fixture"),"MPD structure/privacy mismatch");
   }else if(S10Scenario=="mpd-reattach"){
    S10Click(app,"MPD 0",window,pi,start,output);S10Click(app,"선택한 MPD 구조 제외",window,pi,start,output);Require(!HasLabel(app,"MPD 0"),"Original MPD not omitted before reattach");
    S10Attach(app,mpd,window,pi,start,output);string reattached=S10Export(app,root,output,"s10-reattached","JSON",window,pi,start);Require(S10Count(S10Json(reattached),"mpd_attachments")==1,"MPD reattach count mismatch");
   }else if(S10Scenario=="mpd-remove"||S10Scenario=="mpd-all"){
    if(S10Scenario=="mpd-remove"){S10Click(app,"MPD 0",window,pi,start,output);S10Click(app,"선택한 MPD 구조 제외",window,pi,start,output);}else S10Click(app,"MPD 구조 모두 제외",window,pi,start,output);
    Require(S10Count(S10Json(S10Export(app,root,output,"s10-mpd-omitted","JSON",window,pi,start)),"mpd_attachments")==0,"MPD omission mismatch");
   }else{
    bool large=S10Scenario=="mpd-large";Require(large||S10Scenario=="mpd-unsafe","Scenario mismatch");string bad=Path.Combine(root,large?"s10-large.mpd":"s10-unsafe.mpd");byte[] data=large?new byte[1048577]:Encoding.UTF8.GetBytes("<!DOCTYPE MPD [<!ENTITY x SYSTEM 'file:///SYNTHETIC'>]><MPD>&x;</MPD>");S10WriteNew(bad,data);S10Attach(app,bad,window,pi,start,output);
    var rejected=S10Json(S10Export(app,root,output,"s10-rejected","JSON",window,pi,start));Require(S10Count(rejected,"mpd_attachments")==1&&S10Count(rejected,"attachment_omissions")>0,"Unsafe/large input rejection missing");
    Require(Convert.ToBase64String(File.ReadAllBytes(bad))==Convert.ToBase64String(data),"Rejected source modified");
   }
   Require(Convert.ToBase64String(File.ReadAllBytes(mpd))==Convert.ToBase64String(bytes),"Original MPD changed");
  }
  Capture(window,pi,start,Path.Combine(output,"s10-scenario-final.png"));S10Step(output,"scenario:"+S10Scenario,"asserted");result["s10_scenario_assertions_verified"]=true;result["root_db_and_image_review_required"]=true;
 }
 public static string Run(string exe,string root,string output,string mode,bool verifyReport,int reportHeight,bool reachability,string logOmission){Dictionary<string,object> result=new Dictionary<string,object>();ObserverStopping=false;result["mode"]=mode;result["status"]="FAIL";result["input_before"]=Input();string name="dlpad-cookie-"+Guid.NewGuid().ToString("N");result["private_desktop"]=name;IntPtr desktop=IntPtr.Zero,job=IntPtr.Zero,window=IntPtr.Zero;PI pi=new PI();long start=0;bool restored=false;Thread worker=null;Exception workerError=null;
 try{desktop=CreateDesktop(name,IntPtr.Zero,IntPtr.Zero,0,0x1ff,IntPtr.Zero);Require(desktop!=IntPtr.Zero,"Create private desktop");job=CreateJobObject(IntPtr.Zero,null);Require(job!=IntPtr.Zero,"Create owned Job");Limits limits=new Limits();limits.basic.flags=0x2000;Require(SetInformationJobObject(job,9,ref limits,(uint)Marshal.SizeOf(typeof(Limits))),"Set owned Job kill-on-close");SI si=new SI();si.cb=Marshal.SizeOf(typeof(SI));si.desktop=name;Require(CreateProcess(exe,new StringBuilder("\""+exe+"\" --fixture"),IntPtr.Zero,IntPtr.Zero,false,0x08000004,IntPtr.Zero,Path.GetDirectoryName(exe),ref si,out pi),"Create suspended app");if(!AssignProcessToJobObject(job,pi.process)){TerminateProcess(pi.process,91);throw new Exception("Owned Job binding failed before resume");}long e,k,u;Require(GetProcessTimes(pi.process,out start,out e,out k,out u),"App start identity");result["pid"]=pi.pid;result["start_filetime"]=start;Require(ResumeThread(pi.thread)!=0xffffffff,"Resume owned app");for(int i=0;i<150&&window==IntPtr.Zero;i++){int count=0;bool duplicate=false,overflow=false;EnumDesktopWindows(desktop,delegate(IntPtr h,IntPtr l){if(++count>64){overflow=true;return false;}uint id;GetWindowThreadProcessId(h,out id);if(id==pi.pid&&IsWindowVisible(h)){StringBuilder title=new StringBuilder(256);GetWindowText(h,title,256);if(title.ToString()=="dlpad"){if(window!=IntPtr.Zero&&window!=h){duplicate=true;return false;}window=h;}}return true;},IntPtr.Zero);Require(!duplicate&&!overflow,"Bounded unique original window enumeration");if(WaitForSingleObject(pi.process,0)==0)break;Thread.Sleep(100);}Require(window!=IntPtr.Zero,"Owned visible HWND missing");result["hwnd"]=window.ToInt64();Owned(window,pi,start);StringBuilder windowClass=new StringBuilder(256);GetClassName(window,windowClass,256);result["window_class"]=windowClass.ToString();Rect windowRect;Require(GetClientRect(window,out windowRect),"Owned client metadata");result["client_width"]=windowRect.right;result["client_height"]=windowRect.bottom;
 worker=new Thread(delegate(){S10Step(output,"uia-worker","begin");IntPtr previous=GetThreadDesktop(GetCurrentThreadId());try{Require(SetThreadDesktop(desktop),"MTA private desktop binding");Owned(window,pi,start);Capture(window,pi,start,Path.Combine(output,"before-uia.png"));AutomationElement app=Activate(window,pi,start,output);Require(app!=null&&app.Current.ProcessId==pi.pid,"UIA root identity");OptionObserve(app,window,pi,start,output,result);

 Owned(window,pi,start);Require(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero),"Owned WM_CLOSE");Require(WaitForSingleObject(pi.process,5000)==0,"WM_CLOSE did not exit (no global input fallback)");uint naturalExit;Require(GetExitCodeProcess(pi.process,out naturalExit),"Read original process exit code");result["natural_exit_code"]=naturalExit;Require(naturalExit==0,"Natural app exit nonzero");result["wm_close_exited"]=true;result["closewait_inflight_verified"]=false;result["status"]="REVIEW_REQUIRED";
 }catch(Exception error){workerError=error;}finally{S10Step(output,"uia-worker","end");restored=SetThreadDesktop(previous);}});worker.IsBackground=true;worker.SetApartmentState(ApartmentState.MTA);worker.Start();Require(worker.Join(180000),"UIA worker deadline exceeded");if(workerError!=null)throw workerError;Require(restored,"MTA desktop restoration failed");result["mta_desktop_restored"]=restored;
 }catch(Exception error){result["error"]=error.GetType().FullName+": "+error.Message;result["status"]="FAIL";}
 finally{ObserverStopping=true;bool safeClose=worker==null||!worker.IsAlive;if(!safeClose){result["status"]="FAIL";if(job!=IntPtr.Zero)result["owned_job_terminated_for_worker_drain"]=TerminateJobObject(job,94);safeClose=worker.Join(5000);}result["original_handles_close_safe"]=safeClose;if(!safeClose){result["status"]="FAIL";result["original_handles_retained_until_process_exit"]=true;}if(safeClose){if(job!=IntPtr.Zero){Accounting accounting;bool queried=QueryInformationJobObject(job,1,out accounting,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero);result["owned_job_active_before_cleanup"]=queried?(object)accounting.active:null;result["owned_job_naturally_empty"]=queried&&accounting.active==0;if(!queried||accounting.active!=0){result["status"]="FAIL";result["owned_job_terminated"]=TerminateJobObject(job,92);}bool empty=false;for(int i=0;i<100;i++){if(QueryInformationJobObject(job,1,out accounting,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero)&&accounting.active==0){empty=true;break;}Thread.Sleep(50);}result["owned_job_empty_after_cleanup"]=empty;if(!empty)result["status"]="FAIL";CloseHandle(job);}if(pi.process!=IntPtr.Zero){bool gone=WaitForSingleObject(pi.process,5000)==0;result["owned_process_exited_after_job_close"]=gone;if(!gone)result["status"]="FAIL";CloseHandle(pi.process);}if(pi.thread!=IntPtr.Zero)CloseHandle(pi.thread);if(desktop!=IntPtr.Zero){bool closed=CloseDesktop(desktop);result["private_desktop_closed"]=closed;if(!closed)result["status"]="FAIL";}}try{string after=Input();result["input_after"]=after;result["input_desktop_unchanged"]=after==(string)result["input_before"];if(!(bool)result["input_desktop_unchanged"])result["status"]="FAIL";}catch(Exception error){result["input_query_error"]=error.Message;result["status"]="FAIL";}}
 return new JavaScriptSerializer().Serialize(result);}
 [StructLayout(LayoutKind.Sequential)] struct S10SA {public int length;public IntPtr descriptor;public int inherit;}
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateDirectoryW(string path,ref S10SA attributes);
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
 static string S10FileHash(FileStream file){
  Require(file.Length<=268435456,"Pinned source file size bound");file.Position=0;
  using(var sha=System.Security.Cryptography.SHA256.Create()){return BitConverter.ToString(sha.ComputeHash(file)).Replace("-","").ToLowerInvariant();}
 }
 static FileStream S10Pin(string path,string expected){
  S10Plain(path);Require(expected.Length==64,"SHA length");foreach(char c in expected)Require((c>='0'&&c<='9')||(c>='a'&&c<='f')||(c>='A'&&c<='F'),"SHA syntax");
  var file=new FileStream(path,FileMode.Open,FileAccess.Read,FileShare.Read);try{Require(S10FileHash(file)==expected.ToLowerInvariant(),"Pinned immutable SHA mismatch");return file;}catch{file.Dispose();throw;}
 }
 static void S10CopyNew(FileStream source,string target){S10Plain(target);source.Position=0;using(var output=new FileStream(target,FileMode.CreateNew,FileAccess.Write,FileShare.None)){byte[] buffer=new byte[65536];int n;long total=0;while((n=source.Read(buffer,0,buffer.Length))>0){total+=n;Require(total<=268435456,"Copy bound");output.Write(buffer,0,n);}output.Flush(true);}}
 static void S10SaveNew(string path,object value){string text=new JavaScriptSerializer(){MaxJsonLength=65536}.Serialize(value);byte[] bytes=Encoding.UTF8.GetBytes(text);Require(bytes.Length<=65536,"Summary JSON bound");S10WriteNew(path,bytes);}
 static void S10Archive(string root,string output){
  var queue=new Queue<string>();queue.Enqueue(root);var rows=new List<Dictionary<string,object>>();int entries=0;long total=0;
  string destination=Path.Combine(output,"owned-files");S10PrivateNewDirectory(destination);
  while(queue.Count>0){string parent=queue.Dequeue();S10Plain(parent);
   foreach(string path in Directory.EnumerateFileSystemEntries(parent)){
    Require(++entries<=256,"Owned evidence entry bound");S10Plain(path);
    string relative=path.Substring(root.Length+1);Require(relative.Length<=1024,"Evidence relative path bound");string target=Path.Combine(destination,relative);
    if(Directory.Exists(path)){Require(queue.Count<128,"Owned evidence queue bound");queue.Enqueue(path);S10PrivateNewDirectory(target);continue;}
    Require(File.Exists(path),"Evidence file disappeared");if(String.Equals(Path.GetExtension(path),".exe",StringComparison.OrdinalIgnoreCase))continue;
    using(var input=new FileStream(path,FileMode.Open,FileAccess.Read,FileShare.Read)){
     Require(input.Length<=2097152,"Evidence file size bound");total+=input.Length;Require(total<=8388608&&rows.Count<128,"Evidence aggregate bound");
     string hash=S10FileHash(input);S10CopyNew(input,target);rows.Add(new Dictionary<string,object>{{"path",relative},{"size",input.Length},{"sha256",hash}});
    }
   }
  }
  S10SaveNew(Path.Combine(output,"owned-files.json"),rows);
 }

 static string Desired=null;
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
 static void SettingsAction(AutomationElement app,string name,IntPtr window,PI pi,long start){Owned(window,pi,start);AutomationElement target=null;foreach(var node in Nodes(app)){if(node.Current.Name!=name||!node.Current.IsEnabled)continue;var type=node.Current.ControlType;if(type!=ControlType.Button&&type!=ControlType.TabItem&&type!=ControlType.RadioButton&&type!=ControlType.ListItem)continue;Require(target==null,"Unique typed navigation/action");target=node;}Require(target!=null,"Owned typed action missing");object p;if(target.TryGetCurrentPattern(InvokePattern.Pattern,out p))((InvokePattern)p).Invoke();else if(target.TryGetCurrentPattern(SelectionItemPattern.Pattern,out p))((SelectionItemPattern)p).Select();else if(target.TryGetCurrentPattern(TogglePattern.Pattern,out p)){var toggle=(TogglePattern)p;if(toggle.Current.ToggleState!=ToggleState.On)toggle.Toggle();}else throw new Exception("No-focus action pattern unsupported");Pump(window,pi,start,150);}
// Insert inside published UiReworkPrivateGui. Invoke ZoomCapture once at the
// end of each LayoutCapture phase, before its final SettingsAction/icon query.
static AutomationElement ScaleCombo(AutomationElement app){AutomationElement found=null;foreach(var node in Nodes(app)){if(node.Current.ControlType!=ControlType.ComboBox&&node.Current.ControlType!=ControlType.Button)continue;var label=node.Current.LabeledBy;if(node.Current.Name!="화면 배율"&&(label==null||label.Current.Name!="화면 배율"))continue;Require(found==null,"Unique labelled scale combo");found=node;}Require(found!=null,"Labelled scale combo missing");return found;}
static void ChooseScale(AutomationElement app,string caption,IntPtr window,PI pi,long start){SettingsAction(app,"설정",window,pi,start);var combo=ScaleCombo(app);object pattern=null;Require(combo.Current.IsEnabled,"Scale control enabled");if(combo.TryGetCurrentPattern(ExpandCollapsePattern.Pattern,out pattern))((ExpandCollapsePattern)pattern).Expand();else if(combo.TryGetCurrentPattern(InvokePattern.Pattern,out pattern))((InvokePattern)pattern).Invoke();else throw new Exception("Scale open pattern unsupported");Pump(window,pi,start,150);SettingsAction(app,caption,window,pi,start);Pump(window,pi,start,250);combo=ScaleCombo(app);bool observed=combo.Current.Name==caption;object value;if(combo.TryGetCurrentPattern(ValuePattern.Pattern,out value))observed=observed||((ValuePattern)value).Current.Value==caption;foreach(var node in Nodes(combo))if(node.Current.Name==caption&&!node.Current.IsOffscreen)observed=true;Require(observed,"Selected scale caption actually observed");}
static void ZoomCapture(AutomationElement app,IntPtr window,PI pi,long start,string output,string phase,Dictionary<string,object> result){foreach(string caption in new[]{"150%","200%"}){ChooseScale(app,caption,window,pi,start);string slug=phase+"-zoom-"+caption.Replace("%","");Capture(window,pi,start,Path.Combine(output,slug+"-settings.png"));SettingsAction(app,"다운로드",window,pi,start);var url=Find(app,"영상 URL",ControlType.Edit);var action=Find(app,"다운로드 시작",ControlType.Button);Save(Path.Combine(output,slug+"-url-before.json"),Geometry(url,window,pi,start));Save(Path.Combine(output,slug+"-start-before.json"),Geometry(action,window,pi,start));Point point;bool urlInitial=InContent(url,window,pi,start,out point),startInitial=WithinClient(action,window,pi,start);bool urlReached=urlInitial,startReached=startInitial;if(!urlReached){FocusProbe(url,window,pi,start,output,slug+"-url-reachability",false);urlReached=InContent(url,window,pi,start,out point);}if(!startReached){if(action.Current.IsKeyboardFocusable)FocusProbe(action,window,pi,start,output,slug+"-start-reachability",false);else EnsureContent(action,window,pi,start,output);startReached=WithinClient(action,window,pi,start);}Require(urlReached&&startReached,"Actual zoom controls reachable");result[slug+"_url_initial_content_visible"]=urlInitial;result[slug+"_start_initial_content_visible"]=startInitial;result[slug+"_url_focus_scroll_reachable"]=urlReached;result[slug+"_start_focus_scroll_reachable"]=startReached;Save(Path.Combine(output,slug+"-url-after.json"),Geometry(url,window,pi,start));Save(Path.Combine(output,slug+"-start-after.json"),Geometry(action,window,pi,start));Capture(window,pi,start,Path.Combine(output,slug+"-download.png"));}ChooseScale(app,"100%",window,pi,start);result[phase+"_zoom_reset_100_caption_observed"]=true;result["OS_DPI_change_verified"]=false;result["physical_keyboard_verified"]=false;result["zoom_control_activation_verified"]=false;}

static bool WithinClient(AutomationElement e,IntPtr window,PI pi,long start){Owned(window,pi,start);Rect client=new Rect();Require(GetClientRect(window,out client),"Original client");var b=e.Current.BoundingRectangle;Point p=new Point();p.x=(int)b.Left;p.y=(int)b.Top;Require(ScreenToClient(window,ref p),"Original screen transform");return !e.Current.IsOffscreen&&b.Width>0&&b.Height>0&&p.x>=0&&p.y>=0&&p.x+b.Width<=client.right&&p.y+b.Height<=client.bottom;}
 static void NativeInteraction(AutomationElement app,IntPtr window,PI pi,long start,string output,string phase){
  SettingsAction(app,"다운로드",window,pi,start);
  var target=Find(app,"설정",ControlType.Button);var bounds=target.Current.BoundingRectangle;
  Point point=new Point();point.x=(int)(bounds.Left+bounds.Width/2);point.y=(int)(bounds.Top+bounds.Height/2);
  Require(ScreenToClient(window,ref point),"Native hover coordinate");Owned(window,pi,start);
  Require(PostMessage(window,0x200,IntPtr.Zero,new IntPtr((120<<16)|8)),"Native idle pointer move");Pump(window,pi,start,150);
  string idle=Path.Combine(output,phase+"-interaction-idle.png"),hover=Path.Combine(output,phase+"-interaction-hover.png");Capture(window,pi,start,idle);
  IntPtr xy=new IntPtr((point.y<<16)|(point.x&65535));Require(PostMessage(window,0x200,IntPtr.Zero,xy),"Native hover move");Pump(window,pi,start,180);Capture(window,pi,start,hover);
  Require(PostMessage(window,0x201,new IntPtr(1),xy),"Native pointer down");Pump(window,pi,start,120);Capture(window,pi,start,Path.Combine(output,phase+"-interaction-pressed.png"));
  Require(PostMessage(window,0x202,IntPtr.Zero,xy),"Native pointer up");Pump(window,pi,start,250);ThemeCombo(app);Capture(window,pi,start,Path.Combine(output,phase+"-interaction-selected.png"));
  int changed=0;using(var a=new System.Drawing.Bitmap(idle))using(var b=new System.Drawing.Bitmap(hover)){
   Require(a.Width==b.Width&&a.Height==b.Height,"Native hover raster dimensions");
   for(int y=Math.Max(0,point.y-12);y<Math.Min(a.Height,point.y+12);y++)for(int x=Math.Max(0,point.x-30);x<Math.Min(a.Width,point.x+30);x++)if(a.GetPixel(x,y)!=b.GetPixel(x,y))changed++;
  }
  Require(changed>20,"Actual native hover surface pixels changed");
  Save(Path.Combine(output,phase+"-interaction.json"),new Dictionary<string,object>{{"native_window_mouse_event_route_verified",true},{"actual_hover_changed_pixels",changed},{"actual_pointer_release_selected_settings",true},{"physical_desktop_pointer_verified",false}});
 }
 static void LayoutCapture(AutomationElement app,IntPtr window,PI pi,long start,string output,string phase,Dictionary<string,object> result){NativeInteraction(app,window,pi,start,output,phase);Rect outer=new Rect(),client=new Rect();Require(GetWindowRect(window,out outer)&&GetClientRect(window,out client),"Original size");int dx=outer.right-outer.left-client.right,dy=outer.bottom-outer.top-client.bottom;Require(dx>=0&&dy>=0&&dx<256&&dy<256,"Decoration bound");foreach(int width in new[]{960,800}){int height=width==960?640:600;Owned(window,pi,start);Require(SetWindowPos(window,IntPtr.Zero,0,0,width+dx,height+dy,0x16),"Owned noactivate resize");Pump(window,pi,start,250);Require(GetClientRect(window,out client)&&client.right==width&&client.bottom==height,"Actual exact client");SettingsAction(app,"설정",window,pi,start);ThemeCombo(app);Capture(window,pi,start,Path.Combine(output,phase+"-settings-"+width+".png"));SettingsAction(app,"다운로드",window,pi,start);var field=Find(app,"영상 URL",ControlType.Edit);object value;Require(field.TryGetCurrentPattern(ValuePattern.Pattern,out value),"URL ValuePattern");string synthetic="https://fixture.invalid/한글-영상";if(Read(field)!=synthetic){string inputOutput=Path.Combine(output,phase+"-input-"+width);S10PrivateNewDirectory(inputOutput);result[phase+"_url_transport_"+width]=Fill(field,synthetic,window,pi,start,inputOutput,Read(field)!="");}Require(Read(field)==synthetic,"Actual URL readback");var bounds=field.Current.BoundingRectangle;Point point=new Point();point.x=(int)bounds.Left;point.y=(int)bounds.Top;Require(ScreenToClient(window,ref point)&&point.x>=0&&point.y>=0&&point.x+bounds.Width<=width&&point.y+bounds.Height<=height&&!field.Current.IsOffscreen,"URL actual content containment");Capture(window,pi,start,Path.Combine(output,phase+"-download-"+width+".png"));VerifyNativeTitle(app,window,pi,start);VerifyPathRow(app,window,pi,start,output,phase+"-download-"+width,result);FullWindowCapture(window,pi,start,Path.Combine(output,phase+"-download-full-"+width+".png"));foreach(string page in new[]{"파일 목록","작업 이력","도구"}){SettingsAction(app,page,window,pi,start);Pump(window,pi,start,600);if(page=="파일 목록"){VerifyNativeTitle(app,window,pi,start);VerifyPathRow(app,window,pi,start,output,phase+"-library-"+width,result);FullWindowCapture(window,pi,start,Path.Combine(output,phase+"-library-full-"+width+".png"));}Capture(window,pi,start,Path.Combine(output,phase+"-"+(page=="파일 목록"?"library":page=="작업 이력"?"history":"tools")+"-"+width+".png"));}}Require(SetWindowPos(window,IntPtr.Zero,0,0,960+dx,640+dy,0x16),"Restore960");Pump(window,pi,start,150);ZoomCapture(app,window,pi,start,output,phase,result);SettingsAction(app,"설정",window,pi,start);var icons=NativeIconObservation.Observe(window,PinnedExe,delegate(){Owned(window,pi,start);});Require((long)icons["wm_geticon_1_handle"]!=0||(long)icons["class_big_icon"]!=0,"Window big icon supplied");result[phase+"_icons"]=icons;result["layout_url_readback_verified"]=true;result["actual_download_executed"]=false;}

 static void FullWindowCapture(IntPtr window,PI pi,long start,string path){Owned(window,pi,start);Rect r;Require(GetWindowRect(window,out r),"Original full-window bounds");int width=r.right-r.left,height=r.bottom-r.top;Require(width>0&&height>0&&width<=4096&&height<=4096,"Bounded full-window pixels");using(var bitmap=new System.Drawing.Bitmap(width,height))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();try{Require(PrintWindow(window,dc,0),"Full owned PrintWindow supported");}finally{graphics.ReleaseHdc(dc);}Owned(window,pi,start);using(var f=new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.None))bitmap.Save(f,System.Drawing.Imaging.ImageFormat.Png);Require(new FileInfo(path).Length<=16777216,"Full PNG byte bound");}}
 static void VerifyNativeTitle(AutomationElement app,IntPtr window,PI pi,long start){Owned(window,pi,start);var title=new StringBuilder(256);Require(GetWindowText(window,title,256)>0&&title.ToString()=="dlpad","Original native title dlpad");foreach(var node in Nodes(app))Require(node.Current.ControlType!=ControlType.Text||node.Current.Name!="dlpad","No duplicate client dlpad Text");}
 static void VerifyPathRow(AutomationElement app,IntPtr window,PI pi,long start,string output,string slug,Dictionary<string,object> result){Owned(window,pi,start);var edit=Find(app,"저장 폴더",ControlType.Edit);EnsureContent(edit,window,pi,start,output);var button=Find(app,"폴더 선택…",ControlType.Button);var e=edit.Current.BoundingRectangle;var b=button.Current.BoundingRectangle;Point ep=new Point(),bp=new Point();ep.x=(int)e.Left;ep.y=(int)e.Top;bp.x=(int)b.Left;bp.y=(int)b.Top;Rect client=new Rect();Require(ScreenToClient(window,ref ep)&&ScreenToClient(window,ref bp)&&GetClientRect(window,out client),"Path row client coordinates");Require(!edit.Current.IsOffscreen&&!button.Current.IsOffscreen&&e.Width>0&&b.Width>0&&Math.Abs(e.Height-32)<=0.5&&Math.Abs(b.Height-32)<=0.5&&Math.Abs(e.Top-b.Top)<=1,"Actual same32px path row");Require(ep.x>=0&&bp.x>=0&&ep.y>=0&&bp.y>=0&&ep.x+e.Width<=client.right-16&&bp.x+b.Width<=client.right-16&&ep.y+e.Height<=client.bottom&&bp.y+b.Height<=client.bottom,"Both path row controls within original content margins");Save(Path.Combine(output,slug+"-path-edit.json"),Geometry(edit,window,pi,start));Save(Path.Combine(output,slug+"-path-button.json"),Geometry(button,window,pi,start));result[slug+"_path_row_32px_and_margin_verified"]=true;result[slug+"_path_length_observed"]=Read(edit).Length;}
 static void ThemeObserve(AutomationElement app,IntPtr window,PI pi,long start,string output,Dictionary<string,object> result,string mode){
  // Existing Run captures before-uia.png before accessibility activation. That is
  // an observed capture only; this draft has NO first-presentation claim.
  SettingsAction(app,"설정",window,pi,start);
  var combo=ThemeCombo(app);result["theme_control_type"]=combo.Current.ControlType.ProgrammaticName;
  result["initial_theme_control_name"]=combo.Current.Name;
  object value;result["initial_theme_value_pattern"]=combo.TryGetCurrentPattern(ValuePattern.Pattern,out value);
  if(value is ValuePattern){string actual=((ValuePattern)value).Current.Value;Require(actual.Length<=4096,"Theme actual value bound");result["initial_theme_actual_value"]=actual;}
  Capture(window,pi,start,Path.Combine(output,"settings-initial.png"));
  LayoutCapture(app,window,pi,start,output,"initial",result);
  if(Desired!=null){
   combo=ThemeCombo(app);
   object expand;Require(combo.Current.IsEnabled,"Theme control disabled");
   if(combo.TryGetCurrentPattern(ExpandCollapsePattern.Pattern,out expand))((ExpandCollapsePattern)expand).Expand();
   else if(combo.TryGetCurrentPattern(InvokePattern.Pattern,out expand))((InvokePattern)expand).Invoke();
   else throw new Exception("Theme expand action unsupported");
   Pump(window,pi,start,150);
   SettingsAction(app,Desired,window,pi,start);
   Pump(window,pi,start,250);
   Capture(window,pi,start,Path.Combine(output,"settings-selected.png"));
   result["requested_choice"]=Desired;LayoutCapture(app,window,pi,start,output,"selected",result);
  }
  var rows=new List<Dictionary<string,object>>();
  foreach(var n in Nodes(app))rows.Add(new Dictionary<string,object>{{"name",n.Current.Name},{"type",n.Current.ControlType.ProgrammaticName},{"enabled",n.Current.IsEnabled}});
  Save(Path.Combine(output,"settings-tree.json"),rows);
  result["actual_os_change_verified"]=false;result["first_presentation_verified"]=false;
 }
 static string PreviewUrl;
 static long[] Colors(string path){long red=0,blue=0;using(var b=new System.Drawing.Bitmap(path)){Require(b.Width<=4096&&b.Height<=4096,"Color PNG bounds");for(int y=0;y<b.Height;y+=2)for(int x=0;x<b.Width;x+=2){var c=b.GetPixel(x,y);if(c.R>160&&c.G<80&&c.B<80)red++;if(c.B>160&&c.R<80&&c.G<80)blue++;}}return new[]{red,blue};}
 static string ExportScenario;
 static AutomationElement Optional(AutomationElement app,string label,ControlType type){AutomationElement found=null;foreach(var n in Nodes(app))if(n.Current.Name==label&&n.Current.ControlType==type){Require(found==null,"Duplicate exact local action");found=n;}return found;}
 static void LocalInput(AutomationElement app,string label,string text,IntPtr window,PI pi,long start,string output,string slug){
  var field=Find(app,label,ControlType.Edit);string evidence=Path.Combine(output,slug);S10PrivateNewDirectory(evidence);
  Fill(field,text,window,pi,start,evidence,true);Require(Read(field)==text,"Actual local range readback");
 }
 [StructLayout(LayoutKind.Sequential)] struct FileId {public ulong volume;[MarshalAs(UnmanagedType.ByValArray,SizeConst=16)]public byte[] id;}
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetFileInformationByHandleEx(IntPtr file,int kind,out FileId info,uint size);
 static string Identity(FileStream stream){FileId id;Require(GetFileInformationByHandleEx(stream.SafeFileHandle.DangerousGetHandle(),18,out id,(uint)Marshal.SizeOf(typeof(FileId))),"Original file ID");Require(id.id!=null&&id.id.Length==16,"File ID bound");return id.volume.ToString("x16")+":"+BitConverter.ToString(id.id).Replace("-","").ToLowerInvariant();}
 static void ExportFiles(string root,string output,Dictionary<string,object> result,bool success){
  string media=Path.Combine(root,"media");var directories=Directory.GetDirectories(media);Require(directories.Length<=1,"Single accepted export fresh root");
  var rows=new List<object>();
  foreach(string directory in directories){S10Plain(directory);string[] files=Directory.GetFiles(directory);Require(files.Length<=2,"Export namespace bound");string clip=Path.Combine(directory,"clip.mp4"),stage=Path.Combine(directory,"stage.mp4");
   if(success)Require(files.Length==2&&File.Exists(clip)&&File.Exists(stage),"Final and retained stage exactly two files");
   string clipIdentity=null,stageIdentity=null;
   foreach(string path in files){S10Plain(path);Require(path==clip||path==stage,"Unexpected foreign export filename");using(var file=new FileStream(path,FileMode.Open,FileAccess.Read,FileShare.Read)){
    Require(file.Length>=0&&file.Length<=268435456&&(!success||file.Length>0),"Export file bounded bytes");string id=Identity(file),hash=S10FileHash(file);rows.Add(new Dictionary<string,object>{{"path",path},{"bytes",file.Length},{"sha256",hash},{"native_id",id}});
    if(path==clip)clipIdentity=id;else stageIdentity=id;
   }}
   if(success)Require(clipIdentity==stageIdentity,"Stage/final original same-file hardlink ID");
  }
  if(success)Require(directories.Length==1,"Successful export root absent");
  result["export_files"]=rows;result["same_file_stage_final_verified"]=success;Save(Path.Combine(output,"export-files.json"),rows);
 }
 static T FreshLocalRead<T>(IntPtr window,PI pi,long start,Func<AutomationElement,T> read){
  for(int attempt=0;attempt<12;attempt++){
   Owned(window,pi,start);
   try {var fresh=AutomationElement.FromHandle(window);Require(fresh!=null&&fresh.Current.ProcessId==pi.pid,"Fresh original local root identity");return read(fresh);}
   catch(ElementNotAvailableException){if(attempt==11)throw;Pump(window,pi,start,50);}
  }
  throw new Exception("Unreachable bounded local requery");
 }
 static void MediaObserve(AutomationElement app,IntPtr window,PI pi,long start,string root,string output,Dictionary<string,object> result,string mode){
  result["source_route"]="actual library selectable row and selected-video button; no WM_DROPFILES/HDROP";
  SettingsAction(app,"파일 목록",window,pi,start);
  AutomationElement row=null;var ready=System.Diagnostics.Stopwatch.StartNew();
  while(ready.ElapsedMilliseconds<15000&&row==null){foreach(var node in Nodes(app))if(node.Current.ControlType==ControlType.Button&&node.Current.Name.StartsWith("synthetic-red-blue.mkv · ",StringComparison.Ordinal)){Require(row==null,"Unique synthetic library row");row=node;}if(row==null)Pump(window,pi,start,150);}
  Require(row!=null&&row.Current.IsEnabled,"Actual synthetic library scan row ready");Invoke(row);Pump(window,pi,start,150);
  SettingsAction(app,"선택한 영상 확인",window,pi,start);
  bool prepared=false;ready.Restart();int samples=0;
  while(ready.ElapsedMilliseconds<40000&&samples<160){Owned(window,pi,start);var button=Optional(app,"선택 구간 저장",ControlType.Button);bool slider=false;foreach(var node in Nodes(app))if(node.Current.ControlType==ControlType.Slider)slider=true;
   if(button!=null&&slider&&button.Current.IsEnabled){string image=Path.Combine(output,"local-ready-"+(samples++).ToString("D3")+".png");Capture(window,pi,start,image);var colors=Colors(image);if(colors[0]>1024){prepared=true;break;}}
   Pump(window,pi,start,200);
  }
  Require(prepared,"Actual local Prepared/range/footer/red decoded frame ready");result["actual_local_prepared_red_pixels_verified"]=true;
  LocalInput(app,"시작 (시작과 끝을 모두 비우면 전체 영상)","00:00:01.100",window,pi,start,output,"range-start");
  LocalInput(app,"끝, HH:MM:SS.mmm","00:00:01.800",window,pi,start,output,"range-end");
  result["requested_start_ms"]=1100;result["requested_end_ms"]=1800;result["actual_range_field_readback_verified"]=true;
  Capture(window,pi,start,Path.Combine(output,"local-before-export.png"));
  var export=Find(app,"선택 구간 저장",ControlType.Button);Require(export.Current.IsEnabled,"Valid range export enabled");Invoke(export);
  bool running=false,finished=false;ready.Restart();int runningSamples=0;
  while(ready.ElapsedMilliseconds<140000){Owned(window,pi,start);
   bool saved=FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return HasLabel(fresh,"선택한 구간을 새 파일로 저장했습니다.");});
   bool cancelEnabled=FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){var action=Optional(fresh,"구간 저장 취소",ControlType.Button);return action!=null&&action.Current.IsEnabled;});
   if(cancelEnabled){running=true;if(runningSamples<16)Capture(window,pi,start,Path.Combine(output,"export-running-"+(runningSamples++).ToString("D3")+".png"));
    foreach(string label in new[]{"파일 목록","도구","설정"}){bool enabled=FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){var navigation=Optional(fresh,label,ControlType.Button);return navigation!=null&&navigation.Current.IsEnabled;});Require(!enabled,"Export-active navigation/source/settings gate");}
    Require(!FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return Find(fresh,"다른 영상 열기",ControlType.Button).Current.IsEnabled;}),"Export-active local source change gate");
    if(ExportScenario=="cancel"){var cancel=FreshLocalRead<AutomationElement>(window,pi,start,delegate(AutomationElement fresh){return Find(fresh,"구간 저장 취소",ControlType.Button);});Invoke(cancel);result["cancel_invoked_while_actual_running"]=true;ExportScenario="cancel-wait";}
    else if(ExportScenario=="close"){Require(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero),"Owned active WM_CLOSE");Pump(window,pi,start,150);Require(WaitForSingleObject(pi.process,0)==258,"CloseWait premature original process exit");Capture(window,pi,start,Path.Combine(output,"export-close-prompt.png"));SettingsAction(app,"돌아가기",window,pi,start);result["close_prompt_visible_and_return_verified"]=true;result["closewait_inflight_verified"]=false;ExportScenario="close-wait";}
   }
   if(saved){finished=true;break;}
   if(ExportScenario=="cancel-wait"&&FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return HasLabel(fresh,"구간 저장을 완료하지 못했습니다. 원본과 생성된 파일은 보존됩니다.");})){finished=true;break;}
   Require(!FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return HasLabel(fresh,"작업 종료 확인이 남아 새 작업과 정상 종료를 중단했습니다.");}),"Cleanup uncertainty; do not force clean close");
   Pump(window,pi,start,100);
  }
  Require(finished,"Final success/cancel preservation label absent");
  Require(ExportScenario=="success"||running,"In-flight cancel/close not observed; short export cannot prove that scenario");
  bool cleanSuccess=FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return HasLabel(fresh,"선택한 구간을 새 파일로 저장했습니다.");});
  if(ExportScenario=="success"||ExportScenario=="close-wait")Require(cleanSuccess&&FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return HasLabel(fresh,"검증용 임시 링크도 보존했습니다.");}),"Actual success and retained-stage labels");
  result["export_running_observed"]=running;result["final_success_label_verified"]=cleanSuccess;
  Capture(window,pi,start,Path.Combine(output,"export-final.png"));ExportFiles(root,output,result,cleanSuccess);
  result["source_and_glyph_private_image_review_required"]=true;result["physical_pointer_verified"]=false;
  // Wait for worker thread ACK by observing shared navigation unblocked before natural close.
  ready.Restart();while(ready.ElapsedMilliseconds<10000){if(FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return Find(fresh,"파일 목록",ControlType.Button).Current.IsEnabled;}))break;Pump(window,pi,start,100);}
  Require(FreshLocalRead<bool>(window,pi,start,delegate(AutomationElement fresh){return Find(fresh,"파일 목록",ControlType.Button).Current.IsEnabled;}),"Export worker ACK/shared mutation gate not released");
 }
 static Dictionary<string,object> PlanViewport(AutomationElement plan,IntPtr window,PI pi,long start){
  Owned(window,pi,start);Require(plan.Current.ProcessId==pi.pid,"Original plan provider PID");Rect client;Require(GetClientRect(window,out client),"Owned plan viewport");int top=0,bottom=client.bottom;
  var app=AutomationElement.FromHandle(window);
  foreach(var node in Nodes(app)){
   if(node.Current.ControlType!=ControlType.Button)continue;
   string name=node.Current.Name;var rect=node.Current.BoundingRectangle;Point edge=new Point();edge.x=(int)rect.Left;
   if(name=="다운로드"||name=="설정"||name=="파일 목록"||name=="도구"||name=="작업 이력"){edge.y=(int)Math.Ceiling(rect.Bottom);Require(ScreenToClient(window,ref edge),"Owned navigation edge");top=Math.Max(top,edge.y);}
   if(name=="다운로드 시작"||name=="선택 구간 저장"){edge.y=(int)Math.Floor(rect.Top);Require(ScreenToClient(window,ref edge),"Owned footer edge");bottom=Math.Min(bottom,edge.y);}
  }
  Require(bottom-top>=120&&client.right>=160,"Bounded actual content viewport available");
  var bounds=plan.Current.BoundingRectangle;Point origin=new Point();origin.x=(int)Math.Floor(bounds.Left);origin.y=(int)Math.Floor(bounds.Top);Require(ScreenToClient(window,ref origin),"Plan client origin");
  int left=Math.Max(0,origin.x),right=Math.Min(client.right,(int)Math.Ceiling(origin.x+bounds.Width));int upper=Math.Max(top,origin.y),lower=Math.Min(bottom,(int)Math.Ceiling(origin.y+bounds.Height));
  var ancestors=new List<object>();var parent=TreeWalker.ControlViewWalker.GetParent(plan);bool reached=false;
  for(int i=0;i<16&&parent!=null;i++){
   if(Automation.Compare(parent,app)){reached=true;break;}
   Require(parent.Current.ProcessId==pi.pid,"Plan ancestor original PID");var rectangle=parent.Current.BoundingRectangle;
   if(!rectangle.IsEmpty&&rectangle.Width>0&&rectangle.Height>0){Point p=new Point();p.x=(int)Math.Floor(rectangle.Left);p.y=(int)Math.Floor(rectangle.Top);Require(ScreenToClient(window,ref p),"Owned ancestor clip");left=Math.Max(left,p.x);right=Math.Min(right,(int)Math.Ceiling(p.x+rectangle.Width));upper=Math.Max(upper,p.y);lower=Math.Min(lower,(int)Math.Ceiling(p.y+rectangle.Height));ancestors.Add(new{name=parent.Current.Name,type=parent.Current.ControlType.ProgrammaticName,client_left=p.x,client_top=p.y,width=rectangle.Width,height=rectangle.Height});}
   parent=TreeWalker.ControlViewWalker.GetParent(parent);
  }
  Require(reached,"Bounded plan ancestry reaches original HWND");
  return new Dictionary<string,object>{{"target",Geometry(plan,window,pi,start)},{"viewport_top",top},{"viewport_bottom",bottom},{"viewport_width",client.right},{"clip_ancestors",ancestors},{"intersection_left",left},{"intersection_top",upper},{"intersection_width",Math.Max(0,right-left)},{"intersection_height",Math.Max(0,lower-upper)},{"uia_is_offscreen",plan.Current.IsOffscreen},{"physical_pointer_verified",false},{"actual_glyph_visibility_verified",false},{"root_private_image_review_required",true}};
 }
 static AutomationElement ScrollPlan(AutomationElement plan,IntPtr window,PI pi,long start,string output,Dictionary<string,object> result){
  string original=Read(plan);Require(original.Length<=65536,"Bounded readonly plan text");Require(!plan.Current.IsKeyboardFocusable,"Readonly plan not focus target");object value;if(plan.TryGetCurrentPattern(ValuePattern.Pattern,out value))Require(((ValuePattern)value).Current.IsReadOnly,"Readonly provider value");
  var before=PlanViewport(plan,window,pi,start);Save(Path.Combine(output,"plan-scroll-before.json"),before);Capture(window,pi,start,Path.Combine(output,"plan-scroll-before.png"));
  object scroll;Require(plan.TryGetCurrentPattern(ScrollItemPattern.Pattern,out scroll),"Readonly plan advertised ScrollItemPattern required");((ScrollItemPattern)scroll).ScrollIntoView();Pump(window,pi,start,700);
  plan=Find(AutomationElement.FromHandle(window),"실행 계획 내용",ControlType.Edit);Require(Read(plan)==original,"Native scroll preserves exact readonly argv");Require(!plan.Current.IsKeyboardFocusable&&!plan.Current.HasKeyboardFocus,"Scroll route cannot pretend keyboard focus");
  var after=PlanViewport(plan,window,pi,start);Save(Path.Combine(output,"plan-scroll-after.json"),after);Capture(window,pi,start,Path.Combine(output,"plan-scroll-after.png"));
  Require(Object.Equals(after["uia_is_offscreen"],false)&&(int)after["intersection_width"]>=128&&(int)after["intersection_height"]>=120,"Actual bounded plan preview visible inside content and ancestor viewport");
  Rect client;Require(GetClientRect(window,out client)&&client.right<=4096&&client.bottom<=4096,"Bounded owned preview image");
  using(var bitmap=new System.Drawing.Bitmap(client.right,client.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool printed;try{printed=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(printed,"Owned plan preview PrintWindow");using(var slice=bitmap.Clone(new System.Drawing.Rectangle((int)after["intersection_left"],(int)after["intersection_top"],(int)after["intersection_width"],(int)after["intersection_height"]),bitmap.PixelFormat)){slice.Save(Path.Combine(output,"plan-scroll-visible-preview.png"),System.Drawing.Imaging.ImageFormat.Png);}}
  result["plan_scroll_route"]="ScrollItemPattern.ScrollIntoView";result["plan_scroll_before_geometry"]=before;result["plan_scroll_after_geometry"]=after;result["plan_native_scroll_item_verified"]=true;result["plan_bounded_viewport_geometry_verified"]=true;result["plan_readonly_text_preserved"]=true;result["physical_pointer_verified"]=false;return plan;
 }
 static void OptionObserve(AutomationElement app,IntPtr window,PI pi,long start,string output,Dictionary<string,object> result){
  var url=Find(app,"영상 URL",ControlType.Edit);Fill(url,"https://fixture.invalid/watch",window,pi,start,output,true);
  Require(Read(Find(AutomationElement.FromHandle(window),"영상 URL",ControlType.Edit))=="https://fixture.invalid/watch","Synthetic URL readback");
  SettingsAction(AutomationElement.FromHandle(window),"고급 옵션 펼치기",window,pi,start);
  string[] labels={"속도 제한","재시도","조각 동시성"},bad={"0","21","0"},good={"1048576","0","1"};
  string[] errors={"속도 제한은 1~1000000000000 B/s 정수입니다","재시도는 0~20 정수입니다","조각 동시성은 1~16 정수입니다"};
  var rows=new List<object>();
  for(int i=0;i<3;i++){
   foreach(string value in new[]{bad[i],good[i]}){
    var field=Find(AutomationElement.FromHandle(window),labels[i],ControlType.Edit);field.SetFocus();Pump(window,pi,start,250);field=Find(AutomationElement.FromHandle(window),labels[i],ControlType.Edit);Require(field.Current.HasKeyboardFocus,"Actual exact numeric provider focus");
    string folder=Path.Combine(output,"numeric-"+i+"-"+value);S10PrivateNewDirectory(folder);Fill(field,value,window,pi,start,folder,true);
    field=Find(AutomationElement.FromHandle(window),labels[i],ControlType.Edit);Require(Read(field)==value&&Math.Abs(field.Current.BoundingRectangle.Height-32)<0.5,"Numeric32px exact readback");
    if(value==bad[i])Require(HasLabel(AutomationElement.FromHandle(window),errors[i]),"Actual invalid numeric validation label");else Require(!HasLabel(AutomationElement.FromHandle(window),errors[i]),"Valid numeric clears exact error");
    rows.Add(new{label=labels[i],value=value,invalid=value==bad[i],focus_route="exact-private-provider",physical_pointer_verified=false});
   }
  }
  string[] help={"기본값: 제한 없음 · 예: 1048576","기본값: 3 · 예: 3","기본값: 1 · 예: 1","충돌: 없음 · 설치 버전: 검증되지 않음","충돌: 앱 재시도 정책과 별개 · 설치 버전: 검증되지 않음"};
  foreach(string label in help)Require(HasLabel(AutomationElement.FromHandle(window),label),"Exact default/example/conflict/version help");
  SettingsAction(AutomationElement.FromHandle(window),"실행 계획 미리보기 (옵션 버전은 검증되지 않음)",window,pi,start);Pump(window,pi,start,250);
  var plan=Find(AutomationElement.FromHandle(window),"실행 계획 내용",ControlType.Edit);plan=ScrollPlan(plan,window,pi,start,output,result);string text=Read(plan);Require(text.Length<=262144,"Bounded readonly execution plan");
  var argv=new JavaScriptSerializer().Deserialize<string[]>(text);
  string[] flags={"--limit-rate","--retries","--concurrent-fragments"};
  for(int i=0;i<flags.Length;i++){int count=0,index=-1;for(int n=0;n<argv.Length;n++)if(argv[n]==flags[i]){count++;index=n;}Require(count==1&&index+1<argv.Length&&argv[index+1]==good[i],"Exact plan flag once including replaced default retries");}
  Require(Array.IndexOf(argv,"https://fixture.invalid/watch")<0,"Plan source omitted");Save(Path.Combine(output,"plan-argv.json"),argv);Capture(window,pi,start,Path.Combine(output,"plan.png"));
  result["numeric_fields"]=rows;result["help_labels_verified"]=true;result["plan_flags_verified_once"]=true;result["actual_download_verified"]=false;result["cookies_used"]=false;result["network_used"]=false;
 }
 static int SyntheticMetadata(string[] argv){
  string direct=Environment.GetEnvironmentVariable("DLPAD_PREVIEW_FIXTURE_DIRECT_URL"),page=Environment.GetEnvironmentVariable("DLPAD_PREVIEW_FIXTURE_PAGE_URL");
  Uri parsed;Require(direct!=null&&Uri.TryCreate(direct,UriKind.Absolute,out parsed)&&parsed.Scheme=="http"&&parsed.Host=="127.0.0.1"&&parsed.AbsolutePath=="/media.mkv"&&parsed.Port>0,"Original synthetic loopback URL");
  Require(argv.Length>1&&argv[argv.Length-1]==page&&Array.IndexOf(argv,"--ignore-config")>=0&&Array.IndexOf(argv,"--skip-download")>=0,"Synthetic metadata exact operation");
  Console.OutputEncoding=new UTF8Encoding(false);Console.WriteLine(new JavaScriptSerializer().Serialize(new{title="Synthetic red blue",duration=2.0,vcodec="h264",acodec="none",url=direct}));return 0;
 }
 public static int Main(string[] argv){
  // Synthetic tool requests never perform network, decode, or downloads.
  if(argv.Length>0&&argv[0]=="--version"){Console.WriteLine("2026.01.01");return 0;}
  string output=null;bool owned=false;var result=new Dictionary<string,object>{{"status","FAIL"},{"global_input",false},{"clipboard_invoked",false},{"actual_download_verified",false}};
  try{Require(argv.Length==3,"CPU SHA freshOutput");string exe=Path.GetFullPath(argv[0]);output=Path.GetFullPath(argv[2]);S10Plain(exe);S10Plain(output);
   using(var cpu=S10Pin(exe,argv[1])){S10PrivateNewDirectory(output);owned=true;string root=Path.Combine(Path.GetTempPath(),"dlpad-option-plan-owned-"+Guid.NewGuid().ToString("N"));S10PrivateNewDirectory(root);
    foreach(string name in new[]{"data","media","tmp"})S10PrivateNewDirectory(Path.Combine(root,name));
    string sentinel=Path.Combine(root,"sentinel");S10SaveNew(sentinel,"original owned sentinel");using(var pin=new FileStream(sentinel,FileMode.Open,FileAccess.Read,FileShare.Read)){string hash=S10FileHash(pin);
     Environment.SetEnvironmentVariable("DLPAD_TEST_DATA",Path.Combine(root,"data"));Environment.SetEnvironmentVariable("DLPAD_LIBRARY_ROOT",Path.Combine(root,"media"));Environment.SetEnvironmentVariable("TMP",Path.Combine(root,"tmp"));Environment.SetEnvironmentVariable("TEMP",Path.Combine(root,"tmp"));
     string synthetic=System.Reflection.Assembly.GetExecutingAssembly().Location;Environment.SetEnvironmentVariable("DLPAD_YTDLP",synthetic);Environment.SetEnvironmentVariable("DLPAD_FFMPEG",synthetic);
     foreach(string key in new[]{"DLPAD_DENO","DLPAD_GUARDIAN_BACKEND","DLPAD_TEST_GUARDIAN_EXECUTABLE","DLPAD_PREVIEW_FIXTURE_PAGE_URL","DLPAD_PREVIEW_FIXTURE_DIRECT_URL"})Environment.SetEnvironmentVariable(key,null);
     string gui=Path.Combine(output,"gui");S10PrivateNewDirectory(gui);var native=new JavaScriptSerializer(){MaxJsonLength=65536}.Deserialize<Dictionary<string,object>>(Run(exe,root,gui,"options-plan",false,0,false,"single"));result["native"]=native;Require(Object.Equals(native["status"],"REVIEW_REQUIRED"),"Native option observation failed");Require(S10FileHash(cpu)==argv[1]&&S10FileHash(pin)==hash,"Original CPU/sentinel immutable");result["sentinel_sha256"]=hash;result["owned_root"]=root;result["status"]="REVIEW_REQUIRED";
    }
   }
  }catch(Exception error){result["error"]=error.GetType().Name+": "+error.Message;}
  if(owned)S10SaveNew(Path.Combine(output,"option-plan-result.json"),result);return Object.Equals(result["status"],"REVIEW_REQUIRED")?0:1;
 }

}

static class NativeIconObservation {
 [DllImport("user32.dll",EntryPoint="SendMessageTimeoutW",SetLastError=true)] static extern IntPtr SendMessageTimeout(IntPtr hwnd,uint msg,IntPtr wp,IntPtr lp,uint flags,uint timeout,out IntPtr answer);
 [DllImport("user32.dll",EntryPoint="GetClassLongPtrW",SetLastError=true)] static extern IntPtr GetClassLongPtr(IntPtr hwnd,int index);
 [DllImport("shell32.dll",EntryPoint="ExtractIconExW",CharSet=CharSet.Unicode,SetLastError=true)] static extern uint ExtractIconEx(string path,int index,IntPtr[] large,IntPtr[] small,uint count);
 [DllImport("user32.dll",SetLastError=true)] static extern bool DestroyIcon(IntPtr icon);
 public static Dictionary<string,object> Observe(IntPtr hwnd,string originalPinnedExe,Action guard){guard();var r=new Dictionary<string,object>();foreach(int kind in new[]{0,1,2}){IntPtr icon;bool delivered=SendMessageTimeout(hwnd,0x7f,new IntPtr(kind),IntPtr.Zero,3,1000,out icon)!=IntPtr.Zero;r["wm_geticon_"+kind+"_delivered"]=delivered;r["wm_geticon_"+kind+"_handle"]=icon.ToInt64();if(!delivered)throw new InvalidOperationException("Owned WM_GETICON timed out");}r["class_big_icon"]=GetClassLongPtr(hwnd,-14).ToInt64();r["class_small_icon"]=GetClassLongPtr(hwnd,-34).ToInt64();var big=new IntPtr[1];var small=new IntPtr[1];try{uint count=ExtractIconEx(originalPinnedExe,0,big,small,1);r["exe_extract_count"]=count;r["exe_big_icon_present"]=big[0]!=IntPtr.Zero;r["exe_small_icon_present"]=small[0]!=IntPtr.Zero;if(count!=2||big[0]==IntPtr.Zero||small[0]==IntPtr.Zero)throw new InvalidOperationException("Pinned executable icon absent: count="+count+" big="+big[0].ToInt64()+" small="+small[0].ToInt64()+" error="+Marshal.GetLastWin32Error());guard();r["taskbar_visible_icon_pixels_verified"]=false;r["icon_visual_identity_verified"]=false;return r;}finally{bool ok=true;if(big[0]!=IntPtr.Zero)ok=DestroyIcon(big[0])&&ok;if(small[0]!=IntPtr.Zero&&small[0]!=big[0])ok=DestroyIcon(small[0])&&ok;if(!ok)throw new InvalidOperationException("Owned extracted icon cleanup failed");}}
}

sealed class SyntheticMediaHttp:IDisposable {
 readonly System.Net.Sockets.TcpListener listener;readonly string file;readonly long length;readonly Thread worker;volatile bool stop;
 public long Requests,Bytes,Ranges;public bool Failed,Drained;public string Origin;
 public SyntheticMediaHttp(string path){file=path;length=new FileInfo(path).Length;if(length<=0||length>1048576)throw new Exception("Media size");listener=new System.Net.Sockets.TcpListener(System.Net.IPAddress.Loopback,0);listener.Start(1);Origin="http://127.0.0.1:"+((System.Net.IPEndPoint)listener.LocalEndpoint).Port;worker=new Thread(Loop);worker.IsBackground=true;worker.Start();}
 void Loop(){var lifespan=System.Diagnostics.Stopwatch.StartNew();try{while(!stop){if(lifespan.ElapsedMilliseconds>180000){Failed=true;break;}if(!listener.Pending()){Thread.Sleep(20);continue;}using(var client=listener.AcceptTcpClient()){if(++Requests>128){Failed=true;break;}client.SendTimeout=2000;client.ReceiveTimeout=2000;using(var stream=client.GetStream()){var deadline=System.Diagnostics.Stopwatch.StartNew();var header=new List<byte>();int last=0;while(header.Count<8192){int remain=2000-(int)deadline.ElapsedMilliseconds;if(remain<=0)throw new Exception("HTTP absolute header deadline");stream.ReadTimeout=remain;int b=stream.ReadByte();if(b<0)throw new Exception("HTTP incomplete");header.Add((byte)b);last=(last<<8)|b;if(last==0x0d0a0d0a)break;}if(last!=0x0d0a0d0a)throw new Exception("HTTP header cap");string text=Encoding.ASCII.GetString(header.ToArray());var lines=text.Split(new[]{"\r\n"},StringSplitOptions.None);bool head=lines[0]=="HEAD /media.mkv HTTP/1.1";if(!head&&lines[0]!="GET /media.mkv HTTP/1.1")throw new Exception("HTTP fixed path/method");long start=0,end=length-1;bool range=false;foreach(string line in lines)if(line.StartsWith("Range: bytes=",StringComparison.OrdinalIgnoreCase)){if(range)throw new Exception("HTTP duplicate range");range=true;Ranges++;string[] parts=line.Substring(13).Split('-');if(parts.Length!=2||!long.TryParse(parts[0],out start)||(parts[1].Length!=0&&!long.TryParse(parts[1],out end))||start<0||start>=length||end<start||end>=length)throw new Exception("HTTP range bounds");}
 string response="HTTP/1.1 "+(range?"206 Partial Content":"200 OK")+"\r\nContent-Type: video/x-matroska\r\nAccept-Ranges: bytes\r\nContent-Length: "+(end-start+1)+"\r\nConnection: close\r\n"+(range?"Content-Range: bytes "+start+"-"+end+"/"+length+"\r\n":"")+"\r\n";byte[] answer=Encoding.ASCII.GetBytes(response);stream.Write(answer,0,answer.Length);
 if(!head)using(var source=new FileStream(file,FileMode.Open,FileAccess.Read,FileShare.Read)){source.Position=start;long remaining=end-start+1;byte[] buffer=new byte[65536];while(remaining>0){int due=2000-(int)deadline.ElapsedMilliseconds;if(due<=0||stop)throw new Exception("HTTP body absolute deadline");stream.WriteTimeout=due;int n=source.Read(buffer,0,(int)Math.Min(buffer.Length,remaining));if(n<=0)throw new Exception("HTTP EOF");stream.Write(buffer,0,n);remaining-=n;Bytes+=n;if(Bytes>16777216)throw new Exception("HTTP total payload cap");}}
 }}}}catch{if(!stop)Failed=true;}finally{Drained=true;}}
 public void Close(){stop=true;listener.Stop();if(!worker.Join(5000)){Failed=true;throw new Exception("HTTP original worker not drained");}if(!Drained)throw new Exception("HTTP no ACK");}
 public void Dispose(){Close();}
}
