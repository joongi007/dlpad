using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Automation;
using System.Web.Script.Serialization;
public static class CookiePrivateGui {
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
 string partial=Read(e);if(partial!=""){Require(partial.Length<=1024,"Partial input clear bound");Require(PostMessage(window,0x100,new IntPtr(0x23),new IntPtr(1)),"Owned End down");Require(PostMessage(window,0x101,new IntPtr(0x23),new IntPtr(unchecked((int)0xc0000001))),"Owned End up");Pump(window,pi,start,150);for(int i=0;i<partial.Length;i++){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(8),new IntPtr(1)),"Owned Backspace down");Require(PostMessage(window,0x101,new IntPtr(8),new IntPtr(unchecked((int)0xc0000001))),"Owned Backspace up");Pump(window,pi,start,35);}string cleared=Read(e);InputEvidence(e,value,cleared,output,"clear",window,pi,start);Require(cleared=="","Bounded local backspace did not clear field; no append fallback");}
 foreach(char c in value){Owned(window,pi,start);Require(PostMessage(window,0x100,new IntPtr(0x87),new IntPtr(1)),"Owned non-text F24 envelope down");Require(PostMessage(window,0x102,new IntPtr(c),new IntPtr(1)),"Owned WM_CHAR");Require(PostMessage(window,0x101,new IntPtr(0x87),new IntPtr(unchecked((int)0xc0000001))),"Owned non-text F24 envelope up");Pump(window,pi,start,35);}string actual=Read(e);for(int i=0;i<40&&actual!=value;i++){Pump(window,pi,start,50);actual=Read(e);}InputEvidence(e,value,actual,output,"readback",window,pi,start);Require(actual==value,"Owned input readback mismatch: "+e.Current.Name+" expected_length="+value.Length+" actual_length="+actual.Length);return "owned-F24-envelope-WM_CHAR/readback (UIA setter unavailable/unapplied)";}
 static void Invoke(AutomationElement e){object pattern=null;Require(e.Current.IsEnabled&&e.TryGetCurrentPattern(InvokePattern.Pattern,out pattern),"Invoke pattern unavailable/disabled");((InvokePattern)pattern).Invoke();}
 static void Capture(IntPtr window,PI pi,long start,string path){Owned(window,pi,start);Rect r;Require(GetClientRect(window,out r)&&r.right>0&&r.bottom>0&&r.right<=4096&&r.bottom<=4096,"Screenshot dimensions");using(var bitmap=new System.Drawing.Bitmap(r.right,r.bottom))using(var graphics=System.Drawing.Graphics.FromImage(bitmap)){IntPtr dc=graphics.GetHdc();bool ok;try{ok=PrintWindow(window,dc,3);}finally{graphics.ReleaseHdc(dc);}Require(ok,"Private PrintWindow unsupported");bitmap.Save(path,System.Drawing.Imaging.ImageFormat.Png);}}
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
 static void BoundsClick(AutomationElement e,IntPtr window,PI pi,long start,string output){EnsureContent(e,window,pi,start,output);object pattern;if(e.TryGetCurrentPattern(ScrollItemPattern.Pattern,out pattern)){((ScrollItemPattern)pattern).ScrollIntoView();Pump(window,pi,start,300);}var b=e.Current.BoundingRectangle;Require(!b.IsEmpty&&!e.Current.IsOffscreen&&b.Width>0&&b.Height>0,"Report control bounds unavailable");Point point=new Point();point.x=(int)(b.Left+b.Width/2);point.y=(int)(b.Top+b.Height/2);Owned(window,pi,start);Rect r;Require(ScreenToClient(window,ref point)&&GetClientRect(window,out r)&&point.x>=0&&point.y>=0&&point.x<r.right&&point.y<r.bottom,"Report control outside owned client");IntPtr xy=new IntPtr((point.y<<16)|(point.x&65535));Require(PostMessage(window,0x200,IntPtr.Zero,xy),"Report owned move");Pump(window,pi,start,100);Require(PostMessage(window,0x201,new IntPtr(1),xy),"Report owned down");Pump(window,pi,start,100);Require(PostMessage(window,0x202,IntPtr.Zero,xy),"Report owned up");Pump(window,pi,start,300);}
 static bool HasLabel(AutomationElement app,string name){foreach(AutomationElement e in Nodes(app))if(e.Current.Name==name)return true;return false;}
 static void Privacy(string text,string root){var forbidden=new List<string>();forbidden.Add("windows-download-fixture-only");forbidden.Add(Path.Combine(root,"source.cookies"));string calls=Path.Combine(root,"calls.jsonl");Require(File.Exists(calls)&&new FileInfo(calls).Length<=16384,"Report toolcalls evidence missing/bounded");foreach(string line in File.ReadAllLines(calls)){if(line.Length==0)continue;var call=new JavaScriptSerializer().Deserialize<Dictionary<string,object>>(line);object path;if(call.TryGetValue("cookie_path",out path)&&path is string)forbidden.Add((string)path);}foreach(string value in forbidden){Require(!text.Contains(value),"Report synthetic secret/private path leaked");string escaped=new JavaScriptSerializer().Serialize(value);Require(!text.Contains(escaped.Substring(1,escaped.Length-2)),"Report escaped private value leaked");}}
 static Dictionary<string,object> Geometry(AutomationElement target,IntPtr window,PI pi,long start){Owned(window,pi,start);var b=target.Current.BoundingRectangle;Point rawPoint=new Point();rawPoint.x=(int)(b.Left+b.Width/2);rawPoint.y=(int)(b.Top+b.Height/2);Require(ScreenToClient(window,ref rawPoint),"Owned geometry coordinate conversion");Point visiblePoint;var row=new Dictionary<string,object>();row["name"]=target.Current.Name;row["screen_top"]=b.Top;row["screen_left"]=b.Left;row["width"]=b.Width;row["height"]=b.Height;row["client_x"]=rawPoint.x;row["client_y"]=rawPoint.y;row["is_offscreen"]=target.Current.IsOffscreen;row["in_owned_content"]=InContent(target,window,pi,start,out visiblePoint);row["has_keyboard_focus"]=target.Current.HasKeyboardFocus;return row;}
 static bool PreviewIntersection(AutomationElement preview,AutomationElement app,IntPtr window,PI pi,long start,string output){Owned(window,pi,start);Rect client;Require(GetClientRect(window,out client),"Owned preview clip client");var bounds=preview.Current.BoundingRectangle;Point origin=new Point();origin.x=(int)Math.Floor(bounds.Left);origin.y=(int)Math.Floor(bounds.Top);Require(ScreenToClient(window,ref origin),"Owned preview clip transform");int bottom=client.bottom;foreach(AutomationElement n in Nodes(app)){if(n.Current.ControlType==ControlType.Button&&n.Current.Name=="\ub2e4\uc6b4\ub85c\ub4dc \uc2dc\uc791"){var footer=n.Current.BoundingRectangle;Point edge=new Point();edge.x=(int)footer.Left;edge.y=(int)footer.Top;Require(ScreenToClient(window,ref edge),"Owned footer clip transform");bottom=Math.Min(bottom,edge.y);}}
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
 try{desktop=CreateDesktop(name,IntPtr.Zero,IntPtr.Zero,0,0x1ff,IntPtr.Zero);Require(desktop!=IntPtr.Zero,"Create private desktop");job=CreateJobObject(IntPtr.Zero,null);Require(job!=IntPtr.Zero,"Create owned Job");Limits limits=new Limits();limits.basic.flags=0x2000;Require(SetInformationJobObject(job,9,ref limits,(uint)Marshal.SizeOf(typeof(Limits))),"Set owned Job kill-on-close");SI si=new SI();si.cb=Marshal.SizeOf(typeof(SI));si.desktop=name;Require(CreateProcess(exe,new StringBuilder("\""+exe+"\" --fixture"),IntPtr.Zero,IntPtr.Zero,false,0x08000004,IntPtr.Zero,Path.GetDirectoryName(exe),ref si,out pi),"Create suspended app");if(!AssignProcessToJobObject(job,pi.process)){TerminateProcess(pi.process,91);throw new Exception("Owned Job binding failed before resume");}long e,k,u;Require(GetProcessTimes(pi.process,out start,out e,out k,out u),"App start identity");result["pid"]=pi.pid;result["start_filetime"]=start;Require(ResumeThread(pi.thread)!=0xffffffff,"Resume owned app");for(int i=0;i<150&&window==IntPtr.Zero;i++){int count=0;EnumDesktopWindows(desktop,delegate(IntPtr h,IntPtr l){if(++count>64)return false;uint id;GetWindowThreadProcessId(h,out id);if(id==pi.pid&&IsWindowVisible(h)){StringBuilder title=new StringBuilder(256);GetWindowText(h,title,256);if(title.ToString()=="dlpad"){Require(window==IntPtr.Zero||window==h,"Duplicate owned dlpad windows");window=h;}}return true;},IntPtr.Zero);if(WaitForSingleObject(pi.process,0)==0)break;Thread.Sleep(100);}Require(window!=IntPtr.Zero,"Owned visible HWND missing");result["hwnd"]=window.ToInt64();Owned(window,pi,start);StringBuilder windowClass=new StringBuilder(256);GetClassName(window,windowClass,256);result["window_class"]=windowClass.ToString();Rect windowRect;Require(GetClientRect(window,out windowRect),"Owned client metadata");result["client_width"]=windowRect.right;result["client_height"]=windowRect.bottom;
 worker=new Thread(delegate(){S10Step(output,"uia-worker","begin");IntPtr previous=GetThreadDesktop(GetCurrentThreadId());try{Require(SetThreadDesktop(desktop),"MTA private desktop binding");Owned(window,pi,start);Capture(window,pi,start,Path.Combine(output,"before-uia.png"));AutomationElement app=Activate(window,pi,start,output);Require(app!=null&&app.Current.ProcessId==pi.pid,"UIA root identity");var tree=new List<Dictionary<string,object>>();foreach(AutomationElement n in Nodes(app)){string label=n.Current.Name;Require(label.Length<=4096,"UIA name bound");object p;var row=new Dictionary<string,object>();row["name"]=label;row["native_window_handle"]=n.Current.NativeWindowHandle;row["process_id"]=n.Current.ProcessId;row["framework_id"]=n.Current.FrameworkId;row["automation_id"]=n.Current.AutomationId;row["control_type"]=n.Current.ControlType.ProgrammaticName;row["enabled"]=n.Current.IsEnabled;row["value"]=n.TryGetCurrentPattern(ValuePattern.Pattern,out p);row["text"]=n.TryGetCurrentPattern(TextPattern.Pattern,out p);row["invoke"]=n.TryGetCurrentPattern(InvokePattern.Pattern,out p);tree.Add(row);}Save(Path.Combine(output,"uia-tree.json"),tree);ThemeObserve(app,window,pi,start,output,result,mode);

 Owned(window,pi,start);Require(PostMessage(window,0x10,IntPtr.Zero,IntPtr.Zero),"Owned WM_CLOSE");Require(WaitForSingleObject(pi.process,5000)==0,"WM_CLOSE did not exit (no global input fallback)");uint naturalExit;Require(GetExitCodeProcess(pi.process,out naturalExit),"Read original process exit code");result["natural_exit_code"]=naturalExit;Require(naturalExit==0,"Natural app exit nonzero");result["wm_close_exited"]=true;result["closewait_inflight_verified"]=false;result["status"]="REVIEW_REQUIRED";
 }catch(Exception error){workerError=error;}finally{S10Step(output,"uia-worker","end");restored=SetThreadDesktop(previous);}});worker.IsBackground=true;worker.SetApartmentState(ApartmentState.MTA);worker.Start();Require(worker.Join(90000),"UIA worker deadline exceeded");if(workerError!=null)throw workerError;Require(restored,"MTA desktop restoration failed");result["mta_desktop_restored"]=restored;
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
 // binary SHA freshEvidenceDir seed(System|Light|Dark) select(System|Light|Dark)
 public static int Main(string[] args){
  string output=null;bool owned=false;var result=new Dictionary<string,object>{{"status","FAIL"},{"clipboard_invoked",false},{"system_setting_changed",false}};
  try{
   Require(args.Length==5,"Five explicit arguments required");
   var names=new[]{"System","Light","Dark"};var labels=new[]{"시스템","밝게","어둡게"};
   int seed=Array.IndexOf(names,args[3]),select=Array.IndexOf(names,args[4]);Require(seed>=0&&select>=0,"Invalid theme choice");
   string exe=Path.GetFullPath(args[0]);output=Path.GetFullPath(args[2]);S10Plain(exe);S10Plain(output);
   using(var pin=S10Pin(exe,args[1])){
    S10PrivateNewDirectory(output);owned=true;
    string root=Path.Combine(Path.GetFullPath(Path.GetTempPath()),"dlpad-settings-owned-"+Guid.NewGuid().ToString("N"));S10PrivateNewDirectory(root);
    string data=Path.Combine(root,"data"),media=Path.Combine(root,"media");S10PrivateNewDirectory(data);S10PrivateNewDirectory(media);
    S10WriteNew(Path.Combine(data,"theme.json"),Encoding.UTF8.GetBytes("\""+names[seed]+"\""));
    foreach(string key in new[]{"DLPAD_YTDLP","DLPAD_FFMPEG","DLPAD_DENO","DLPAD_GUARDIAN_BACKEND","DLPAD_TEST_GUARDIAN_EXECUTABLE"})Environment.SetEnvironmentVariable(key,null);
    Environment.SetEnvironmentVariable("DLPAD_TEST_DATA",data);Environment.SetEnvironmentVariable("DLPAD_LIBRARY_ROOT",media);
    string first=Path.Combine(output,"first"),restart=Path.Combine(output,"restart");S10PrivateNewDirectory(first);S10PrivateNewDirectory(restart);
    Desired=labels[select];var a=new JavaScriptSerializer(){MaxJsonLength=65536}.Deserialize<Dictionary<string,object>>(Run(exe,root,first,"settings",false,0,false,"single"));
    S10SaveNew(Path.Combine(first,"result.json"),a);
    object exited;Require(a.TryGetValue("status",out exited)&&Object.Equals(exited,"REVIEW_REQUIRED"),"First run failed");Require(a.TryGetValue("wm_close_exited",out exited)&&Object.Equals(exited,true),"First app did not acknowledge close");
    foreach(string key in new[]{"owned_job_empty_after_cleanup","owned_process_exited_after_job_close","private_desktop_closed","input_desktop_unchanged","mta_desktop_restored"})Require(a.TryGetValue(key,out exited)&&Object.Equals(exited,true),"First ownership cleanup incomplete");
    Require(File.Exists(Path.Combine(data,"theme.sqlite")),"Actual theme store missing");
    Desired=null;var b=new JavaScriptSerializer(){MaxJsonLength=65536}.Deserialize<Dictionary<string,object>>(Run(exe,root,restart,"settings",false,0,false,"single"));
    S10SaveNew(Path.Combine(restart,"result.json"),b);
    Require(b.TryGetValue("status",out exited)&&Object.Equals(exited,"REVIEW_REQUIRED"),"Restart run failed");Require(b.TryGetValue("wm_close_exited",out exited)&&Object.Equals(exited,true),"Restart did not acknowledge close");
    foreach(string key in new[]{"owned_job_empty_after_cleanup","owned_process_exited_after_job_close","private_desktop_closed","input_desktop_unchanged","mta_desktop_restored"})Require(b.TryGetValue(key,out exited)&&Object.Equals(exited,true),"Restart ownership cleanup incomplete");
    Require(S10FileHash(pin)==args[1].ToLowerInvariant(),"Binary changed");
    result["first"]=a;result["restart"]=b;result["owned_root"]=root;result["binary_sha256"]=args[1];result["seed"]=names[seed];result["selected"]=names[select];
    result["status"]="REVIEW_REQUIRED";result["sqlite_and_pixels_require_independent_review"]=true;
   }
  }catch(Exception e){result["error"]=e.GetType().Name+": "+e.Message;}
  if(owned)S10SaveNew(Path.Combine(output,"settings-result.json"),result);
  return Object.Equals(result["status"],"REVIEW_REQUIRED")?0:1;
 }
}
