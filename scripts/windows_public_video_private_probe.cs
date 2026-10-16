using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Automation;
using System.Web.Script.Serialization;
public static class PublicVideoPrivateGui {
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
 static void InputEvidence(AutomationElement e,string expected,string observed,string output,string stage,IntPtr window,PI pi,long start){var row=new Dictionary<string,object>();row["field"]=e.Current.Name;row["stage"]=stage;row["expected_length"]=expected.Length;row["actual_length"]=observed.Length;row["expected_sha256"]=Hash(expected);row["actual_sha256"]=Hash(observed);row["actual_public_uia_value"]=observed.Length<=1024?observed:"[bounded omission]";row["synthetic_only"]=false;var bounds=e.Current.BoundingRectangle;row["bounds_left"]=bounds.Left;row["bounds_top"]=bounds.Top;row["bounds_width"]=bounds.Width;row["bounds_height"]=bounds.Height;string field=expected.StartsWith("https://",StringComparison.Ordinal)?"url":"field";Save(Path.Combine(output,"input-"+field+"-"+stage+".json"),row);Capture(window,pi,start,Path.Combine(output,"input-"+field+"-"+stage+".png"));}
 static string Fill(AutomationElement e,string value,IntPtr window,PI pi,long start,string output,bool replace=false){Require(value.Length<=1024,"Input bound");Require(replace||Read(e)=="","Expected initially empty field");Owned(window,pi,start);Require(e.Current.IsKeyboardFocusable,"Owned input focusable");e.SetFocus();Pump(window,pi,start,300);Require(e.Current.HasKeyboardFocus,"Owned input focus readback");if(replace||e.Current.Name=="\uc9c4\ub2e8 \uc790\ub8cc \uc800\uc7a5 \uacbd\ub85c")EnsureContent(e,window,pi,start,output);object pattern;bool setter=e.TryGetCurrentPattern(ValuePattern.Pattern,out pattern);if(setter){try{((ValuePattern)pattern).SetValue(value);}catch(InvalidOperationException){}Pump(window,pi,start,300);string observed=Read(e);InputEvidence(e,value,observed,output,"setter",window,pi,start);if(observed==value)return "ValuePattern/readback";}
 if(e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out pattern)){((ScrollItemPattern)pattern).ScrollIntoView();Pump(window,pi,start,250);}
 Owned(window,pi,start);e=Find(AutomationElement.FromHandle(window),e.Current.Name,ControlType.Edit);e.SetFocus();Pump(window,pi,start,200);Require(e.Current.HasKeyboardFocus,"Typed input stays focused");
 string inputLabel=e.Current.Name;string partial=Read(Find(AutomationElement.FromHandle(window),inputLabel,ControlType.Edit));if(partial!=""){Require(partial.Length<=1024,"Partial input clear bound");Require(PostMessage(window,0x100,new IntPtr(0x23),new IntPtr(1)),"Owned End down");Require(PostMessage(window,0x101,new IntPtr(0x23),new IntPtr(unchecked((int)0xc0000001))),"Owned End up");Pump(window,pi,start,150);for(int i=0;i<partial.Length;i++){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(8),new IntPtr(1)),"Owned Backspace down");Require(PostMessage(window,0x101,new IntPtr(8),new IntPtr(unchecked((int)0xc0000001))),"Owned Backspace up");Pump(window,pi,start,150);}e=Find(AutomationElement.FromHandle(window),inputLabel,ControlType.Edit);string cleared=Read(e);InputEvidence(e,value,cleared,output,"clear",window,pi,start);Require(cleared=="","Bounded local backspace did not clear field; no append fallback");}
 foreach(char c in value){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(0x87),new IntPtr(1)),"Owned non-text F24 envelope down");Require(PostMessage(window,0x102,new IntPtr(c),new IntPtr(1)),"Owned WM_CHAR");Require(PostMessage(window,0x101,new IntPtr(0x87),new IntPtr(unchecked((int)0xc0000001))),"Owned non-text F24 envelope up");Pump(window,pi,start,150);}e=Find(AutomationElement.FromHandle(window),inputLabel,ControlType.Edit);string actual=Read(e);for(int i=0;i<40&&actual!=value;i++){Pump(window,pi,start,50);e=Find(AutomationElement.FromHandle(window),inputLabel,ControlType.Edit);actual=Read(e);}InputEvidence(e,value,actual,output,"readback",window,pi,start);Require(actual==value,"Owned input readback mismatch: "+e.Current.Name+" expected_length="+value.Length+" actual_length="+actual.Length);return "UIA SetFocus + owned-F24-envelope-WM_CHAR/readback (UIA setter unavailable/unapplied)";}
 static void Invoke(AutomationElement e){object pattern=null;Require(e.Current.IsEnabled&&e.TryGetCurrentPattern(InvokePattern.Pattern,out pattern),"Invoke pattern unavailable/disabled");((InvokePattern)pattern).Invoke();}
 static void Capture(IntPtr window,PI pi,long start,string path){Owned(window,pi,start);Rect r;Require(GetClientRect(window,out r)&&r.right>0&&r.bottom>0&&r.right<=4096&&r.bottom<=4096,"Screenshot dimensions");using(var bitmap=new System.Drawing.Bitmap(r.right,r.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool ok;try{ok=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(ok,"Private PrintWindow unsupported");using(var f=new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.None))bitmap.Save(f,System.Drawing.Imaging.ImageFormat.Png);Require(new FileInfo(path).Length<=16777216,"PNG byte bound");}}
 static void Save(string path,object value){string text=new JavaScriptSerializer().Serialize(value);Require(Encoding.UTF8.GetByteCount(text)<=65536,"Evidence JSON limit");File.WriteAllText(path,text,new UTF8Encoding(false));}

 static Dictionary<string,object> Patterns(AutomationElement e){object p;var row=new Dictionary<string,object>();row["name"]=e.Current.Name;row["type"]=e.Current.ControlType.ProgrammaticName;row["enabled"]=e.Current.IsEnabled;row["keyboard_focusable"]=e.Current.IsKeyboardFocusable;row["has_keyboard_focus"]=e.Current.HasKeyboardFocus;row["invoke"]=e.TryGetCurrentPattern(InvokePattern.Pattern,out p);row["scroll_item"]=e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out p);row["text"]=e.TryGetCurrentPattern(TextPattern.Pattern,out p);AutomationProperty text2=AutomationProperty.LookupById(30119);if(text2!=null){object availability=e.GetCurrentPropertyValue(text2,true);row["text_pattern2_available"]=availability is bool?(object)(bool)availability:"unsupported";}bool hasValue=e.TryGetCurrentPattern(ValuePattern.Pattern,out p);row["value"]=hasValue;if(hasValue)row["read_only"]=((ValuePattern)p).Current.IsReadOnly;return row;}
 static bool InContent(AutomationElement e,IntPtr window,PI pi,long start,out Point point){Owned(window,pi,start);point=new Point();var b=e.Current.BoundingRectangle;if(b.IsEmpty||b.Width<=0||b.Height<=0||e.Current.IsOffscreen)return false;point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);Rect r;if(!ScreenToClient(window,ref point)||!GetClientRect(window,out r)||point.x<0||point.y<0||point.x>=r.right||point.y>=r.bottom)return false;AutomationElement app=AutomationElement.FromHandle(window);foreach(AutomationElement n in Nodes(app)){
 if(n.Current.ControlType==ControlType.Button&&(n.Current.Name=="다운로드"||n.Current.Name=="설정"||n.Current.Name=="파일 목록"||n.Current.Name=="도구"||n.Current.Name=="작업 이력")){
  var tabs=n.Current.BoundingRectangle;Point edge=new Point();edge.x=(int)tabs.Left;edge.y=(int)Math.Ceiling(tabs.Bottom);
  if(!ScreenToClient(window,ref edge)||point.y<=edge.y)return false;
 }
 if(n.Current.ControlType==ControlType.Button&&n.Current.Name=="\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791"){var footer=n.Current.BoundingRectangle;Point top=new Point();top.x=(int)footer.Left;top.y=(int)footer.Top;if(!ScreenToClient(window,ref top)||point.y>=top.y)return false;}}return true;}
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
 static void BoundsClick(AutomationElement e,IntPtr window,PI pi,long start,string output){EnsureContent(e,window,pi,start,output);object pattern;if(e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out pattern)){((ScrollItemPattern)pattern).ScrollIntoView();Pump(window,pi,start,300);}var b=e.Current.BoundingRectangle;Require(!b.IsEmpty&&!e.Current.IsOffscreen&&b.Width>0&&b.Height>0,"Report control bounds unavailable");Point point=new Point();point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);Owned(window,pi,start);Rect r;Require(ScreenToClient(window,ref point)&&GetClientRect(window,out r)&&point.x>=0&&point.y>=0&&point.x<r.right&&point.y<r.bottom,"Report control outside owned client");IntPtr xy=new IntPtr((point.y<<16)|(point.x&65535));Require(PostMessage(window,0x200,IntPtr.Zero,xy),"Report owned move");Require(PostMessage(window,0x201,new IntPtr(1),xy),"Report owned down");Require(PostMessage(window,0x202,IntPtr.Zero,xy),"Report owned up");Pump(window,pi,start,300);}
 static bool HasLabel(AutomationElement app,string name){foreach(AutomationElement e in Nodes(app))if(e.Current.Name==name)return true;return false;}
 static void Privacy(string text,string root){var forbidden=new List<string>();forbidden.Add("windows-download-fixture-only");forbidden.Add(Path.Combine(root,"source.cookies"));string calls=Path.Combine(root,"calls.jsonl");Require(File.Exists(calls)&&new FileInfo(calls).Length<=16384,"Report toolcalls evidence missing/bounded");foreach(string line in File.ReadAllLines(calls)){if(line.Length==0)continue;var call=new JavaScriptSerializer().Deserialize<Dictionary<string,object>>(line);object path;if(call.TryGetValue("cookie_path",out path)&&path is string)forbidden.Add((string)path);}foreach(string value in forbidden){Require(!text.Contains(value),"Report synthetic secret/private path leaked");string escaped=new JavaScriptSerializer().Serialize(value);Require(!text.Contains(escaped.Substring(1,escaped.Length-2)),"Report escaped private value leaked");}}
 static Dictionary<string,object> Geometry(AutomationElement target,IntPtr window,PI pi,long start){Owned(window,pi,start);var b=target.Current.BoundingRectangle;Point rawPoint=new Point();rawPoint.x=(int)(b.Left+b.Width/2);rawPoint.y=(int)(b.Top+b.Height/2);Require(ScreenToClient(window,ref rawPoint),"Owned geometry coordinate conversion");Point visiblePoint;var row=new Dictionary<string,object>();row["name"]=target.Current.Name;row["screen_top"]=b.Top;row["screen_left"]=b.Left;row["width"]=b.Width;row["height"]=b.Height;row["client_x"]=rawPoint.x;row["client_y"]=rawPoint.y;row["is_offscreen"]=target.Current.IsOffscreen;row["in_owned_content"]=InContent(target,window,pi,start,out visiblePoint);row["has_keyboard_focus"]=target.Current.HasKeyboardFocus;return row;}
 static bool PreviewIntersection(AutomationElement preview,AutomationElement app,IntPtr window,PI pi,long start,string output){Owned(window,pi,start);Rect client=new Rect();Require(GetClientRect(window,out client),"Owned preview clip client");var bounds=preview.Current.BoundingRectangle;Point origin=new Point();origin.x=(int)Math.Floor(bounds.Left);origin.y=(int)Math.Floor(bounds.Top);Require(ScreenToClient(window,ref origin),"Owned preview clip transform");int bottom=client.bottom;foreach(AutomationElement n in Nodes(app)){if(n.Current.ControlType==ControlType.Button&&n.Current.Name=="\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791"){var footer=n.Current.BoundingRectangle;Point edge=new Point();edge.x=(int)footer.Left;edge.y=(int)footer.Top;Require(ScreenToClient(window,ref edge),"Owned footer clip transform");bottom=Math.Min(bottom,edge.y);}}
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
 worker=new Thread(delegate(){S10Step(output,"uia-worker","begin");IntPtr previous=GetThreadDesktop(GetCurrentThreadId());try{Require(SetThreadDesktop(desktop),"MTA private desktop binding");Owned(window,pi,start);Capture(window,pi,start,Path.Combine(output,"before-uia.png"));AutomationElement app=Activate(window,pi,start,output);Require(app!=null&&app.Current.ProcessId==pi.pid,"UIA root identity");var tree=new List<Dictionary<string,object>>();foreach(AutomationElement n in Nodes(app)){string label=n.Current.Name;Require(label.Length<=4096,"UIA name bound");object p;var row=new Dictionary<string,object>();row["name"]=label;row["native_window_handle"]=n.Current.NativeWindowHandle;row["process_id"]=n.Current.ProcessId;row["framework_id"]=n.Current.FrameworkId;row["automation_id"]=n.Current.AutomationId;row["control_type"]=n.Current.ControlType.ProgrammaticName;row["enabled"]=n.Current.IsEnabled;row["value"]=n.TryGetCurrentPattern(ValuePattern.Pattern,out p);row["text"]=n.TryGetCurrentPattern(TextPattern.Pattern,out p);row["invoke"]=n.TryGetCurrentPattern(InvokePattern.Pattern,out p);tree.Add(row);}Save(Path.Combine(output,"uia-tree.json"),tree);MediaObserve(app,window,pi,start,output,result,mode);

 Owned(window,pi,start);Require(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero),"Owned WM_CLOSE");Require(WaitForSingleObject(pi.process,5000)==0,"WM_CLOSE did not exit (no global input fallback)");uint naturalExit;Require(GetExitCodeProcess(pi.process,out naturalExit),"Read original process exit code");result["natural_exit_code"]=naturalExit;Require(naturalExit==0,"Natural app exit nonzero");result["wm_close_exited"]=true;result["closewait_inflight_verified"]=false;result["status"]="REVIEW_REQUIRED";
 }catch(Exception error){workerError=error;}finally{S10Step(output,"uia-worker","end");restored=SetThreadDesktop(previous);}});worker.IsBackground=true;worker.SetApartmentState(ApartmentState.MTA);worker.Start();Require(worker.Join(240000),"UIA worker deadline exceeded");if(workerError!=null)throw workerError;Require(restored,"MTA desktop restoration failed");result["mta_desktop_restored"]=restored;
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
 static void StableNavigation(AutomationElement app,string page,IntPtr window,PI pi,long start){var clock=System.Diagnostics.Stopwatch.StartNew();while(clock.Elapsed.TotalSeconds<10){try{SettingsAction(app,page,window,pi,start);return;}catch(ElementNotAvailableException){Pump(window,pi,start,250);}}throw new Exception("Navigation UIA refresh deadline");}
 static void MediaObserve(AutomationElement app,IntPtr window,PI pi,long start,string output,Dictionary<string,object> result,string mode){
  SettingsAction(app,"다운로드",window,pi,start);var field=Find(app,"영상 URL",ControlType.Edit);
  result["url_transport"]=Fill(field,PreviewUrl,window,pi,start,output,Read(field)!="");Require(Read(field)==PreviewUrl,"Authorized URL readback");Capture(window,pi,start,Path.Combine(output,"url-filled.png"));
  var clock=System.Diagnostics.Stopwatch.StartNew();bool ready=false;var labels=new List<string>();
  while(clock.Elapsed.TotalSeconds<90){Owned(window,pi,start);bool slider=false,play=false;labels.Clear();
   try{foreach(var n in Nodes(app)){if(n.Current.ControlType==ControlType.Slider)slider=true;if(n.Current.ControlType==ControlType.Button&&n.Current.Name=="미리보기 재생"&&n.Current.IsEnabled)play=true;if(n.Current.ControlType==ControlType.Text&&labels.Count<64&&n.Current.Name.Length<=512)labels.Add(n.Current.Name);}}catch(ElementNotAvailableException){Pump(window,pi,start,250);continue;}
   if(slider&&play){ready=true;break;}if(HasLabel(app,"영상을 확인하지 못했습니다. 주소와 미리보기 도구를 확인해 주세요."))break;Pump(window,pi,start,250);
  }
  result["prepared_controls_observed"]=ready;result["prepared_text_labels_observed_not_metadata_verified"]=labels.ToArray();result["prepare_wait_ms"]=clock.ElapsedMilliseconds;Capture(window,pi,start,Path.Combine(output,ready?"prepared.png":"prepare-failed-or-deadline.png"));
  result["real_site_extraction_verified"]=false;result["actual_decode_verified"]=false;result["public_video_preparation_acceptance"]=ready?"controls-observed-review-required":"FAILED_OR_TIMED_OUT";
  if(ready){
   Invoke(Find(app,"미리보기 재생",ControlType.Button));result["play_action_route"]="UIA Invoke";string first=Path.Combine(output,"play-00.png");Pump(window,pi,start,250);Capture(window,pi,start,first);string firstHash;using(var fp=File.OpenRead(first))firstHash=S10FileHash(fp);bool changed=false;
   for(int i=1;i<=8;i++){Pump(window,pi,start,250);string p=Path.Combine(output,"play-"+i.ToString("D2")+".png");Capture(window,pi,start,p);using(var fp=File.OpenRead(p))changed|=S10FileHash(fp)!=firstHash;}
   result["full_client_raster_changed_not_decode_proof"]=changed;
   var pauseClock=System.Diagnostics.Stopwatch.StartNew();bool paused=false;var pauseRows=new List<object>();
   while(pauseClock.Elapsed.TotalSeconds<10&&!paused){
    var buttons=new List<object>();string traversalError=null;bool attempted=false;
    try{Owned(window,pi,start);var fresh=AutomationElement.FromHandle(window);Require(fresh.Current.ProcessId==pi.pid,"Pause root identity");foreach(var n in Nodes(fresh)){try{if(n.Current.ControlType==ControlType.Button){string name=n.Current.Name;if(buttons.Count<16)buttons.Add(new{name=name.Length<=256?name:"[omitted]",enabled=n.Current.IsEnabled});if(name=="미리보기 일시 정지"&&n.Current.IsEnabled){attempted=true;Invoke(n);paused=true;break;}}}catch(ElementNotAvailableException){}}}catch(ElementNotAvailableException e){traversalError=e.GetType().Name;}
    if(pauseRows.Count<12)pauseRows.Add(new{elapsed_ms=pauseClock.ElapsedMilliseconds,traversal_error=traversalError,buttons=buttons.ToArray(),invoke_attempted=attempted,invoke_returned=paused});
    if(!paused)Pump(window,pi,start,250);
   }
   Save(Path.Combine(output,"pause-diagnostic.json"),pauseRows);Capture(window,pi,start,Path.Combine(output,paused?"pause-invoked.png":"pause-unconfirmed.png"));
   result["pause_action_observed"]=paused;
  }
  if(ready){
   var head=Find(app,"미리보기 위치",ControlType.Slider);object range;
   if(head.TryGetCurrentPattern(RangeValuePattern.Pattern,out range)){
    var values=(RangeValuePattern)range;double target=Math.Min(60000,values.Current.Maximum);
    values.SetValue(target);var seekClock=System.Diagnostics.Stopwatch.StartNew();Pump(window,pi,start,1000);
    while(seekClock.Elapsed.TotalSeconds<18){try{if(!HasLabel(app,"선택한 위치를 불러오고 있습니다"))break;}catch(ElementNotAvailableException){}Pump(window,pi,start,250);}
    result["seek_action_target_ms"]=target;Capture(window,pi,start,Path.Combine(output,"seek-60s.png"));
   }else result["seek_pattern_supported"]=false;
   PublicPreparedFields(app,window,pi,start,output,result);
  }
  foreach(string page in new[]{"설정","파일 목록","도구","작업 이력"}){StableNavigation(app,page,window,pi,start);Pump(window,pi,start,300);string slug=page=="설정"?"settings":page=="파일 목록"?"library":page=="도구"?"tools":"history";Capture(window,pi,start,Path.Combine(output,slug+".png"));var focus=new List<object>();try{foreach(var n in Nodes(app))if(n.Current.HasKeyboardFocus&&focus.Count<8)focus.Add(new{name=n.Current.Name.Length<=256?n.Current.Name:"[omitted]",type=n.Current.ControlType.ProgrammaticName});}catch(ElementNotAvailableException){focus.Clear();}Save(Path.Combine(output,slug+"-focus.json"),focus);}
  result["navigation_pages_observed"]=true;result["physical_pointer_verified"]=false;result["range_handle_drag_verified"]=false;result["inflight_cancel_verified"]=false;result["full_video_download_started"]=false;
 }
// Insert inside PublicVideoPrivateGui. Call only after prepared_controls_observed=true,
// BEFORE page navigation, with ready media paused. No DownloadStart action.
// Source labels: catalog RANGE_START/RANGE_END/QUALITY_MODE/QUALITY_AUDIO/
// OPTIONAL_MEDIA/ADVANCED_OPEN/ADVANCED_CLOSE/CANCEL. Private UIA only; not physical input.
static AutomationElement PublicRetryFind(AutomationElement app,string name,ControlType type,IntPtr window,PI pi,long start){
 for(int attempt=0;attempt<8;attempt++){
  Owned(window,pi,start);
  try{return Find(app,name,type);}catch(ElementNotAvailableException){if(attempt==7)throw;Pump(window,pi,start,100);app=AutomationElement.FromHandle(window);}
 }
 throw new Exception("Bounded stale field retry");
}
static void PublicFocusCapture(AutomationElement app,IntPtr window,PI pi,long start,string output,string slug){
 var rows=new List<object>();foreach(var node in Nodes(app)){
  if(rows.Count>=8)break;
  try{if(node.Current.HasKeyboardFocus)rows.Add(new{name=node.Current.Name.Length<=256?node.Current.Name:"[omitted]",type=node.Current.ControlType.ProgrammaticName});}
  catch(ElementNotAvailableException){/* Observation-only stale node omitted, not proof. */}
 }
 Save(Path.Combine(output,slug+"-focus.json"),rows);Capture(window,pi,start,Path.Combine(output,slug+".png"));
}
static void PublicPreparedFields(AutomationElement app,IntPtr window,PI pi,long start,string output,Dictionary<string,object> result){
 SettingsAction(app,"다운로드",window,pi,start);
 // When media_prepared=true, range edits are in media preview. For an unprepared
 // form they are behind '구간 및 자막'; this helper deliberately rejects that state.
 var begin=PublicRetryFind(app,"시작 (시작과 끝을 모두 비우면 전체 영상)",ControlType.Edit,window,pi,start);
 var end=PublicRetryFind(app,"끝, HH:MM:SS.mmm",ControlType.Edit,window,pi,start);
 string oldStart=Read(begin),oldEnd=Read(end);
 string fields=Path.Combine(output,"public-time-start"),endFields=Path.Combine(output,"public-time-end");S10PrivateNewDirectory(fields);S10PrivateNewDirectory(endFields);
 // Small valid nonzero interval for both authorized long public videos. This is
 // local form editing only, not proof that FFmpeg cut semantics were executed.
 Fill(begin,"00:00:01.000",window,pi,start,fields,true);
 // Fill can redraw/rebuild the accessible subtree: always reacquire end.
 end=PublicRetryFind(app,"끝, HH:MM:SS.mmm",ControlType.Edit,window,pi,start);
 Fill(end,"00:00:02.000",window,pi,start,endFields,true);
 begin=PublicRetryFind(app,"시작 (시작과 끝을 모두 비우면 전체 영상)",ControlType.Edit,window,pi,start);
 end=PublicRetryFind(app,"끝, HH:MM:SS.mmm",ControlType.Edit,window,pi,start);
 Require(Read(begin)=="00:00:01.000"&&Read(end)=="00:00:02.000","Time edit exact readback");
 result["time_form_readback_verified"]=true;result["cut_execution_verified"]=false;
 PublicFocusCapture(app,window,pi,start,output,"time-fields");
 // QUALITY_MODE is a visible weak Text label; current ComboBox has no labelled_by.
 // Resolve selected-text Best plus typed ComboBox/Button, not label-only matching.
 AutomationElement quality=null;
 foreach(var node in Nodes(app)){
  if(node.Current.ControlType==ControlType.ComboBox&&Read(node)=="최고 품질"){
   Require(quality==null,"Unique typed quality control");quality=node;
  }
 }
 Require(quality!=null,"Best quality control unavailable; inspect bounded tree, no fallback click");
 object pattern;
 if(quality.TryGetCurrentPattern(ExpandCollapsePattern.Pattern,out pattern))((ExpandCollapsePattern)pattern).Expand();
 else Invoke(quality);
 Pump(window,pi,start,150);
 AutomationElement choice=null;
 foreach(var node in Nodes(app))if(node.Current.Name=="오디오 우선"&&(node.Current.ControlType==ControlType.ListItem||node.Current.ControlType==ControlType.RadioButton||node.Current.ControlType==ControlType.Button)){
  Require(choice==null,"Unique typed quality choice");choice=node;
 }
 Require(choice!=null,"Audio choice missing");
 if(choice.TryGetCurrentPattern(SelectionItemPattern.Pattern,out pattern))((SelectionItemPattern)pattern).Select();else Invoke(choice);
 Pump(window,pi,start,200);
 bool audio=false;foreach(var node in Nodes(app))if(node.Current.ControlType==ControlType.ComboBox&&Read(node)=="오디오 우선")audio=true;
 Require(audio,"Selected quality visible readback");result["quality_audio_selection_observed"]=true;PublicFocusCapture(app,window,pi,start,output,"quality-selected");
 Invoke(PublicRetryFind(app,"고급 옵션 펼치기",ControlType.Button,window,pi,start));Pump(window,pi,start,150);
 PublicRetryFind(app,"고급 옵션 접기",ControlType.Button,window,pi,start);PublicFocusCapture(app,window,pi,start,output,"advanced-open");
 Invoke(PublicRetryFind(app,"고급 옵션 접기",ControlType.Button,window,pi,start));Pump(window,pi,start,150);
 PublicRetryFind(app,"고급 옵션 펼치기",ControlType.Button,window,pi,start);result["advanced_expand_collapse_observed"]=true;
 // Re-prepare metadata only to exercise the real preview cancellation control.
 StableNavigation(app,"영상 확인",window,pi,start);
 bool cancel=false;var cancelClock=System.Diagnostics.Stopwatch.StartNew();
 while(cancelClock.Elapsed.TotalSeconds<10&&!cancel){try{foreach(var node in Nodes(app))if(node.Current.Name=="작업 취소"&&node.Current.ControlType==ControlType.Button&&node.Current.IsEnabled){Invoke(node);cancel=true;break;}}catch(ElementNotAvailableException){}if(!cancel)Pump(window,pi,start,100);}
 result["preview_cancel_action_observed"]=cancel;result["cancel_terminal_ACK_verified"]=false;
 Pump(window,pi,start,500);
 PublicFocusCapture(app,window,pi,start,output,"after-conditional-preview-cancel");
 result["full_video_download_started"]=false;
 // oldStart/oldEnd retained as local form observations, not backend execution evidence.
 result["time_before_start"]=oldStart;result["time_before_end"]=oldEnd;
}

 public static int Main(string[] argv){
  var result=new Dictionary<string,object>{{"status","FAIL"},{"synthetic_extractor_only",false},{"loopback_server_used",false},{"full_video_download_started",false},{"cookies_auth_used",false},{"global_input",false},{"clipboard_used",false}};string output=null;bool owned=false;
  try{Require(argv.Length==7,"CPU SHA yt-dlp FFmpeg Deno freshOutput publicURL");
   string exe=Path.GetFullPath(argv[0]),yt=Path.GetFullPath(argv[2]),ff=Path.GetFullPath(argv[3]),deno=Path.GetFullPath(argv[4]);output=Path.GetFullPath(argv[5]);
   Require(argv[6]=="https://chzzk.naver.com/video/15441192"||argv[6]=="https://youtu.be/lxOLdQXE2yw?si=D_OjqkJT2d5WKMl7","Explicit authorized public URL only");PreviewUrl=argv[6];PinnedExe=exe;
   string probe=Path.Combine(Path.GetDirectoryName(ff),"ffprobe.exe");foreach(string p in new[]{exe,yt,ff,deno,probe,output})S10Plain(p);
   using(var cpu=S10Pin(exe,argv[1]))using(var y=new FileStream(yt,FileMode.Open,FileAccess.Read,FileShare.Read))using(var f=new FileStream(ff,FileMode.Open,FileAccess.Read,FileShare.Read))using(var d=new FileStream(deno,FileMode.Open,FileAccess.Read,FileShare.Read))using(var pr=new FileStream(probe,FileMode.Open,FileAccess.Read,FileShare.Read)){
    string ys=S10FileHash(y),fs=S10FileHash(f),ds=S10FileHash(d),ps=S10FileHash(pr);S10PrivateNewDirectory(output);owned=true;
    string root=Path.Combine(Path.GetTempPath(),"dlpad-public-video-owned-"+Guid.NewGuid().ToString("N"));S10PrivateNewDirectory(root);foreach(string n in new[]{"data","media","tmp"})S10PrivateNewDirectory(Path.Combine(root,n));
    S10WriteNew(Path.Combine(root,"data","theme.json"),Encoding.UTF8.GetBytes("\"System\""));
    Environment.SetEnvironmentVariable("DLPAD_TEST_DATA",Path.Combine(root,"data"));Environment.SetEnvironmentVariable("DLPAD_LIBRARY_ROOT",Path.Combine(root,"media"));Environment.SetEnvironmentVariable("TMP",Path.Combine(root,"tmp"));Environment.SetEnvironmentVariable("TEMP",Path.Combine(root,"tmp"));
    Environment.SetEnvironmentVariable("DLPAD_YTDLP",yt);Environment.SetEnvironmentVariable("DLPAD_FFMPEG",ff);Environment.SetEnvironmentVariable("DLPAD_DENO",deno);
    foreach(string key in new[]{"DLPAD_PREVIEW_FIXTURE_PAGE_URL","DLPAD_PREVIEW_FIXTURE_DIRECT_URL","DLPAD_GUARDIAN_BACKEND","DLPAD_TEST_GUARDIAN_EXECUTABLE"})Environment.SetEnvironmentVariable(key,null);
    string gui=Path.Combine(output,"gui");S10PrivateNewDirectory(gui);var observed=new JavaScriptSerializer(){MaxJsonLength=65536}.Deserialize<Dictionary<string,object>>(Run(exe,root,gui,"public-video",false,0,false,"single"));result["native"]=observed;
    Require(Object.Equals(observed["status"],"REVIEW_REQUIRED"),"Native observation failed");foreach(string key in new[]{"wm_close_exited","original_handles_close_safe","owned_job_naturally_empty","owned_job_empty_after_cleanup","private_desktop_closed","input_desktop_unchanged","mta_desktop_restored"})Require(Object.Equals(observed[key],true),"Original cleanup "+key);
    Require(Convert.ToInt64(observed["natural_exit_code"])==0&&Convert.ToInt64(observed["owned_job_active_before_cleanup"])==0,"Natural original exit/Job0");
    Require(S10FileHash(cpu)==argv[1]&&S10FileHash(y)==ys&&S10FileHash(f)==fs&&S10FileHash(d)==ds&&S10FileHash(pr)==ps,"Original tool bytes preserved");
    result["owned_root"]=root;result["CPU_sha256"]=argv[1];result["ytdlp_sha256"]=ys;result["ffmpeg_sha256"]=fs;result["ffprobe_sha256"]=ps;result["deno_sha256"]=ds;result["status"]="REVIEW_REQUIRED";result["public_url"]=PreviewUrl;result["tool_official_provenance_outer_required"]=true;result["raw_tool_stderr_not_collected_by_observer"]=true;
   }
  }catch(Exception error){result["error"]=error.GetType().Name+": "+error.Message;}
  if(owned)S10SaveNew(Path.Combine(output,"public-video-result.json"),result);return Object.Equals(result["status"],"REVIEW_REQUIRED")?0:1;
 }

}

static class NativeIconObservation {
 [DllImport("user32.dll",EntryPoint="SendMessageTimeoutW",SetLastError=true)] static extern IntPtr SendMessageTimeout(IntPtr hwnd,uint msg,IntPtr wp,IntPtr lp,uint flags,uint timeout,out IntPtr answer);
 [DllImport("user32.dll",EntryPoint="GetClassLongPtrW",SetLastError=true)] static extern IntPtr GetClassLongPtr(IntPtr hwnd,int index);
 [DllImport("shell32.dll",EntryPoint="ExtractIconExW",CharSet=CharSet.Unicode,SetLastError=true)] static extern uint ExtractIconEx(string path,int index,IntPtr[] large,IntPtr[] small,uint count);
 [DllImport("user32.dll",SetLastError=true)] static extern bool DestroyIcon(IntPtr icon);
 public static Dictionary<string,object> Observe(IntPtr hwnd,string originalPinnedExe,Action guard){guard();var r=new Dictionary<string,object>();foreach(int kind in new[]{0,1,2}){IntPtr icon;bool delivered=SendMessageTimeout(hwnd,0x7f,new IntPtr(kind),IntPtr.Zero,3,1000,out icon)!=IntPtr.Zero;r["wm_geticon_"+kind+"_delivered"]=delivered;r["wm_geticon_"+kind+"_handle"]=icon.ToInt64();if(!delivered)throw new InvalidOperationException("Owned WM_GETICON timed out");}r["class_big_icon"]=GetClassLongPtr(hwnd,-14).ToInt64();r["class_small_icon"]=GetClassLongPtr(hwnd,-34).ToInt64();var big=new IntPtr[1];var small=new IntPtr[1];try{uint count=ExtractIconEx(originalPinnedExe,0,big,small,1);r["exe_extract_count"]=count;r["exe_big_icon_present"]=big[0]!=IntPtr.Zero;r["exe_small_icon_present"]=small[0]!=IntPtr.Zero;if(count!=2||big[0]==IntPtr.Zero||small[0]==IntPtr.Zero)throw new InvalidOperationException("Pinned executable icon absent: count="+count+" big="+big[0].ToInt64()+" small="+small[0].ToInt64()+" error="+Marshal.GetLastWin32Error());guard();r["taskbar_visible_icon_pixels_verified"]=false;r["icon_visual_identity_verified"]=false;return r;}finally{bool ok=true;if(big[0]!=IntPtr.Zero)ok=DestroyIcon(big[0])&&ok;if(small[0]!=IntPtr.Zero&&small[0]!=big[0])ok=DestroyIcon(small[0])&&ok;if(!ok)throw new InvalidOperationException("Owned extracted icon cleanup failed");}}
}
