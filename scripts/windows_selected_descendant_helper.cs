// Fixture-only exact-self child. Both processes remain in the original inherited Job.
using System;
using System.Diagnostics;
using System.IO;
using System.Threading;
public static class OwnedDescendantHelper {
 [System.Runtime.InteropServices.DllImport("kernel32.dll",EntryPoint="GetModuleFileNameW",CharSet=System.Runtime.InteropServices.CharSet.Unicode,ExactSpelling=true,SetLastError=true)] static extern uint ModuleName(IntPtr module,System.Text.StringBuilder path,uint size);
 public static int Main(string[] args) {
  try { return Run(args); } catch(Exception e) {
   string tmp=Environment.GetEnvironmentVariable("TMP");
   if(!String.IsNullOrEmpty(tmp)&&Path.GetFileName(tmp)=="tmp"&&Path.GetFileName(Path.GetDirectoryName(tmp)).StartsWith("dlpad-selected-process-owned-",StringComparison.Ordinal)) {
    string type=e.GetType().FullName; if(type.Length>128)type=type.Substring(0,128);
    var native=e as System.ComponentModel.Win32Exception; int code=native==null?0:native.NativeErrorCode;
    string message=e.Message??""; if(message.Length>512)message=message.Substring(0,512);
    string stack=e.StackTrace??""; if(stack.Length>1024)stack=stack.Substring(0,1024);
    byte[] bytes=System.Text.Encoding.UTF8.GetBytes("{\"synthetic_only\":true,\"exception_type\":\""+type+"\",\"native_error_code\":"+code+",\"exception_message\":\""+Escape(message)+"\",\"exception_stack\":\""+Escape(stack)+"\"}");
    using(var f=new FileStream(Path.Combine(tmp,"helper-parent-error.json"),FileMode.CreateNew,FileAccess.Write,FileShare.Read)){f.Write(bytes,0,bytes.Length);f.Flush();}
   }
   return 65;
  }
 }
 static string Escape(string s) {return s.Replace("\\","\\\\").Replace("\"","\\\"").Replace("\r","\\r").Replace("\n","\\n").Replace("\t","\\t");}
 static int Run(string[] args) {
  if(args.Length==1&&args[0]=="--owned-child") {
   using(var output=Console.OpenStandardOutput()){output.WriteByte(82);output.Flush();}
   Thread.Sleep(10000); return 0;
  }
  if(args.Length!=0)return 64;
  // CLR FileIOPermission rejects Assembly.Location for the selected runtime's extended DOS path.
  // Observe this original native module; strip only the DOS extended prefix for the fixture child.
  var module=new System.Text.StringBuilder(32768); uint length=ModuleName(IntPtr.Zero,module,32768);
  if(length==0||length>=32768)return 67;
  string exact=module.ToString();
  if(exact.StartsWith(@"\\?\",StringComparison.Ordinal)) {
   if(exact.Length<7||!Char.IsLetter(exact[4])||exact[5]!=':'||exact[6]!='\\')return 68;
   exact=exact.Substring(4);
  }
  if(exact.Length<3||exact.Length>32000||!Char.IsLetter(exact[0])||exact[1]!=':'||exact[2]!='\\')return 69;
  var start=new ProcessStartInfo(exact,"--owned-child");
  start.UseShellExecute=false;start.CreateNoWindow=true;start.RedirectStandardOutput=true;start.RedirectStandardInput=true;start.RedirectStandardError=true;
  using(var child=Process.Start(start)) {
   int ready=-1;Exception error=null;
   var pump=new Thread(delegate(){try{ready=child.StandardOutput.BaseStream.ReadByte();}catch(Exception e){error=e;}});
   pump.IsBackground=true;pump.Start();
   if(!pump.Join(2000)||error!=null||ready!=82||child.HasExited){if(!child.HasExited)child.Kill();return 65;}
   // Original child Process owns its HANDLE; disposing releases it without terminating.
   return 17;
  }
 }
}
