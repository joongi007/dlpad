// Own a native CLI tree on a private, never-visible desktop. No shell or UI input.
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
class DlpadCliHost {
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] struct Startup {
        public uint cb; public string reserved, desktop, title;
        public uint x,y,xsize,ysize,xchars,ychars,fill,flags; public ushort show, reservedSize;
        public IntPtr reservedPtr, input, output, error;
    }
    [StructLayout(LayoutKind.Sequential)] struct ProcessInformation {
        public IntPtr process, thread; public uint pid, tid;
    }
    [StructLayout(LayoutKind.Sequential)] struct BasicLimits {
        public long processTime, jobTime; public uint flags;
        public UIntPtr minWorking, maxWorking; public uint active;
        public UIntPtr affinity; public uint priority, scheduling;
    }
    [StructLayout(LayoutKind.Sequential)] struct IoCounters {
        public ulong readOps,writeOps,otherOps,readBytes,writeBytes,otherBytes;
    }
    [StructLayout(LayoutKind.Sequential)] struct Limits {
        public BasicLimits basic; public IoCounters io;
        public UIntPtr processMemory,jobMemory,peakProcess,peakJob;
    }
    [StructLayout(LayoutKind.Sequential)] struct Accounting { public long user,kernel,periodUser,periodKernel; public uint faults,total,active,terminated; }
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool QueryInformationJobObject(IntPtr job,int kind,out Accounting value,uint bytes,IntPtr returned);
    static bool WaitJobEmpty(IntPtr job,int milliseconds,out uint active) {
        var timer=System.Diagnostics.Stopwatch.StartNew();active=UInt32.MaxValue;
        do { Accounting value;Check(QueryInformationJobObject(job,1,out value,(uint)Marshal.SizeOf(typeof(Accounting)),IntPtr.Zero));active=value.active;if(active==0)return true;Thread.Sleep(25); } while(timer.ElapsedMilliseconds<milliseconds);
        return false;
    }
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern IntPtr CreateJobObjectW(IntPtr attributes,string name);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool SetInformationJobObject(IntPtr job,int kind,ref Limits limits,uint size);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool AssignProcessToJobObject(IntPtr job,IntPtr process);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool TerminateJobObject(IntPtr job,uint exit);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateProcessW(string app,StringBuilder command,IntPtr processAttributes,IntPtr threadAttributes,bool inherit,uint flags,IntPtr environment,string directory,ref Startup startup,out ProcessInformation process);
    [DllImport("kernel32.dll",SetLastError=true)] static extern uint ResumeThread(IntPtr thread);
    [DllImport("kernel32.dll",SetLastError=true)] static extern uint WaitForSingleObject(IntPtr handle,uint timeout);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr handle,out uint code);
    [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr handle);
    [DllImport("kernel32.dll")] static extern IntPtr GetStdHandle(int kind);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool TerminateProcess(IntPtr process,uint code);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateDesktopW(string name,IntPtr device,IntPtr mode,uint flags,uint access,IntPtr security);
    [DllImport("user32.dll",SetLastError=true)] static extern bool CloseDesktop(IntPtr desktop);
    [DllImport("user32.dll",SetLastError=true)] static extern IntPtr OpenInputDesktop(uint flags,bool inherit,uint access);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool GetUserObjectInformationW(IntPtr handle,int index,StringBuilder name,uint size,out uint needed);
    static string InputName() {
        IntPtr handle=OpenInputDesktop(0,false,1);Check(handle!=IntPtr.Zero);
        try { StringBuilder name=new StringBuilder(256);uint needed;Check(GetUserObjectInformationW(handle,2,name,512,out needed));return name.ToString(); }
        finally { Check(CloseDesktop(handle)); }
    }
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateFileW(string name,uint access,uint share,IntPtr security,uint creation,uint flags,IntPtr template);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool SetHandleInformation(IntPtr handle,uint mask,uint flags);
    static string Quote(string value) {
        StringBuilder result = new StringBuilder("\""); int slashes=0;
        foreach(char c in value) {
            if(c=='\\') { slashes++; continue; }
            if(c=='\"') { result.Append('\\',slashes*2+1); result.Append(c); }
            else { result.Append('\\',slashes); result.Append(c); }
            slashes=0;
        }
        result.Append('\\',slashes*2); result.Append('"'); return result.ToString();
    }
    static void Check(bool result) { if(!result) throw new Win32Exception(Marshal.GetLastWin32Error()); }
    static int Main(string[] args) {
        if(args.Length<3) { Console.Error.WriteLine("usage: windows-cli-host.exe timeout-seconds absolute-executable argv..."); return 2; }
        int seconds; if(!Int32.TryParse(args[0],out seconds)||seconds<1||seconds>7200) return 2;
        if(!System.IO.Path.IsPathRooted(args[1])) return 2;
        string tools=Environment.GetEnvironmentVariable("DLPAD_NATIVE_TOOL_PATH");
        if(!String.IsNullOrEmpty(tools)) Environment.SetEnvironmentVariable("PATH",tools+";"+Environment.GetEnvironmentVariable("PATH"));
        string inputBefore=null;int result=125;bool naturalJobZero=false,finalJobZero=false,forced=false,desktopClosed=false,inputUnchanged=false,jobClosed=false;uint finalActive=UInt32.MaxValue;
        IntPtr desktop=IntPtr.Zero,job=IntPtr.Zero,nullInput=IntPtr.Zero;
        object gate=new object();bool shuttingDown=false,cancelRequested=false,controlFailed=false;
        bool controlled=Environment.GetEnvironmentVariable("DLPAD_NATIVE_CONTROL_STDIN")=="1"; ProcessInformation child=new ProcessInformation();
        try {
            inputBefore=InputName();
            string name="dlpad-cli-"+Guid.NewGuid().ToString("N");
            desktop=CreateDesktopW(name,IntPtr.Zero,IntPtr.Zero,0,0x83,IntPtr.Zero);
            Check(desktop!=IntPtr.Zero);
            job=CreateJobObjectW(IntPtr.Zero,null); Check(job!=IntPtr.Zero);
            Limits limits=new Limits(); limits.basic.flags=0x2000;
            Check(SetInformationJobObject(job,9,ref limits,(uint)Marshal.SizeOf(typeof(Limits))));
            StringBuilder command=new StringBuilder();
            for(int i=1;i<args.Length;i++) { if(i>1) command.Append(' '); command.Append(Quote(args[i])); }
            Startup startup=new Startup(); startup.cb=(uint)Marshal.SizeOf(typeof(Startup));
            startup.desktop=name; startup.flags=0x100; startup.input=GetStdHandle(-10);startup.output=GetStdHandle(-11);startup.error=GetStdHandle(-12);
            if(controlled) {
                Check(SetHandleInformation(startup.input,1,0));
                nullInput=CreateFileW("NUL",0x80000000,3,IntPtr.Zero,3,0x80,IntPtr.Zero);
                Check(nullInput!=IntPtr.Zero && nullInput!=new IntPtr(-1));
                Check(SetHandleInformation(nullInput,1,1));startup.input=nullInput;
            }
            Check(CreateProcessW(args[1],command,IntPtr.Zero,IntPtr.Zero,true,0x08000004,IntPtr.Zero,null,ref startup,out child));
            if(!AssignProcessToJobObject(job,child.process)) {
                int error=Marshal.GetLastWin32Error(); TerminateProcess(child.process,125); throw new Win32Exception(error);
            }
            if(ResumeThread(child.thread)==0xffffffff) throw new Win32Exception(Marshal.GetLastWin32Error());
            if(controlled) {
                Thread controller=new Thread(delegate() {
                    try { Console.OpenStandardInput().ReadByte(); } catch { }
                    lock(gate) { if(!shuttingDown) {cancelRequested=true;if(!TerminateJobObject(job,130))controlFailed=true;} }
                });
                controller.IsBackground=true;controller.Start();
            }
            Console.Error.WriteLine("dlpad-owned-cli pid="+child.pid+" private-desktop=true");
            uint wait=WaitForSingleObject(child.process,(uint)(seconds*1000));
            if(wait==258) { forced=true;Check(TerminateJobObject(job,124));Check(WaitForSingleObject(child.process,5000)==0); Console.Error.WriteLine("dlpad-owned-cli timeout");result=124; }
            else {Check(wait==0);uint code;Check(GetExitCodeProcess(child.process,out code));result=unchecked((int)code);}
            bool observedEmpty=WaitJobEmpty(job,2000,out finalActive);
            lock(gate){naturalJobZero=observedEmpty&&!forced&&!cancelRequested;}
            if(!naturalJobZero){forced=true;if(result==0)result=125;Check(TerminateJobObject(job,125));}
            finalJobZero=WaitJobEmpty(job,5000,out finalActive);Check(finalJobZero);
        } catch(Exception error) { Console.Error.WriteLine("dlpad-owned-cli error-type="+error.GetType().Name+" os-error="+Marshal.GetLastWin32Error());result=125; }
        finally {
            // Prove original Job empty BEFORE closing its last original handle.
            lock(gate) {
                shuttingDown=true;
                if(cancelRequested)result=130;
                if(controlFailed)result=125;
                if(job!=IntPtr.Zero) {
                    try {
                        if(!finalJobZero){forced=true;Check(TerminateJobObject(job,125));finalJobZero=WaitJobEmpty(job,5000,out finalActive);}
                    } catch {finalJobZero=false;result=125;}
                    jobClosed=CloseHandle(job);
                    if(!jobClosed||!finalJobZero)result=125;
                }
            }
            if(nullInput!=IntPtr.Zero && nullInput!=new IntPtr(-1))if(!CloseHandle(nullInput))result=125;
            if(child.thread!=IntPtr.Zero)if(!CloseHandle(child.thread))result=125;
            if(child.process!=IntPtr.Zero)if(!CloseHandle(child.process))result=125;
            desktopClosed=desktop!=IntPtr.Zero&&CloseDesktop(desktop);if(!desktopClosed)result=125;
            try {inputUnchanged=inputBefore!=null&&inputBefore==InputName();}catch{inputUnchanged=false;}
            if(!inputUnchanged)result=125;
            Console.Error.WriteLine("dlpad-owned-job-terminal natural-zero="+naturalJobZero+" final-zero="+finalJobZero+" final-active="+finalActive+" forced="+forced+" job-closed="+jobClosed+" private-desktop-closed="+desktopClosed+" stdin-cancel="+cancelRequested+" result="+result);
            Console.Error.WriteLine("dlpad-input-desktop-unchanged="+inputUnchanged);
        }
        return result;
    }
}
