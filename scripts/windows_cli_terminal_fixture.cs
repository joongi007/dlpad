using System;using System.Runtime.InteropServices;using System.Text;using System.Threading;
class TerminalFixture {
 [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)]struct SI{public uint cb;public string reserved,desktop,title;public uint x,y,w,h,xc,yc,fill,flags;public ushort show,reservedSize;public IntPtr reservedPtr,input,output,error;}
 [StructLayout(LayoutKind.Sequential)]struct PI{public IntPtr process,thread;public uint pid,tid;}
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)]static extern bool CreateProcessW(string exe,StringBuilder argv,IntPtr sa,IntPtr ta,bool inherit,uint flags,IntPtr env,string cwd,ref SI si,out PI pi);
 [DllImport("kernel32.dll")]static extern bool CloseHandle(IntPtr h);
 static int Main(string[] args){
  if(args.Length==2&&args[0]=="stdin-driver"){
   var info=new System.Diagnostics.ProcessStartInfo(args[1],"5 \""+System.Diagnostics.Process.GetCurrentProcess().MainModule.FileName+"\" linger");
   info.UseShellExecute=false;info.RedirectStandardInput=true;info.CreateNoWindow=true;
   using(var host=System.Diagnostics.Process.Start(info)){
    host.StandardInput.Close();if(!host.WaitForExit(15000))return 66;
    // Outer ORIGINAL native host Job owns this driver plus new host subtree.
    return host.ExitCode==130?0:67;
   }
  }
  if(args.Length!=1)return 64;
  if(args[0]=="normal")return 0;
  if(args[0]=="linger"){Thread.Sleep(30000);return 0;}
  if(args[0]=="grandchild"){
   string exe=System.Diagnostics.Process.GetCurrentProcess().MainModule.FileName;
   SI si=new SI();si.cb=(uint)Marshal.SizeOf(typeof(SI));PI child;
   if(!CreateProcessW(exe,new StringBuilder("\""+exe+"\" linger"),IntPtr.Zero,IntPtr.Zero,false,0x08000000,IntPtr.Zero,null,ref si,out child))return 65;
   Console.WriteLine("dlpad-synthetic-grandchild pid="+child.pid);CloseHandle(child.thread);CloseHandle(child.process);return 0;
  }return 64;
 }
}
