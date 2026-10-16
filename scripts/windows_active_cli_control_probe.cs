// Own a native CLI tree on a private, never-visible desktop. No shell or UI input.
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
class LocalInstallControlProbe {
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
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateProcessW(string app,StringBuilder command,IntPtr processAttributes,IntPtr threadAttributes,bool inherit,uint flags,IntPtr environment,string directory,ref StartupEx startup,out ProcessInformation process);
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
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool FreeConsole();
    [StructLayout(LayoutKind.Sequential)] struct ConsoleOptions { public int mode; public int useShowWindow; public ushort showWindow; }
    [UnmanagedFunctionPointer(CallingConvention.Winapi)] delegate int AllocHidden(ref ConsoleOptions options,out int result);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] static extern IntPtr GetModuleHandleW(string name);
    [DllImport("kernel32.dll",CharSet=CharSet.Ansi,ExactSpelling=true)] static extern IntPtr GetProcAddress(IntPtr module,string name);
    static void AllocateHidden() { IntPtr fn=GetProcAddress(GetModuleHandleW("kernel32.dll"),"AllocConsoleWithOptions");if(fn==IntPtr.Zero)throw new Exception("hidden_console_api_unavailable");var alloc=(AllocHidden)Marshal.GetDelegateForFunctionPointer(fn,typeof(AllocHidden));ConsoleOptions options=new ConsoleOptions();options.mode=2;options.useShowWindow=1;options.showWindow=0;int result;if(alloc(ref options,out result)!=0||result!=1)throw new Exception("hidden_console_allocation_failed"); }
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool SetStdHandle(int kind,IntPtr value);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool GenerateConsoleCtrlEvent(uint kind,uint group);
    [DllImport("kernel32.dll",SetLastError=true)] static extern uint SuspendThread(IntPtr thread);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] static extern IntPtr GetThreadDesktop(uint thread);
    [StructLayout(LayoutKind.Sequential)] struct Time {public uint low,high; public ulong Value {get{return ((ulong)high<<32)|low;}}}
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetProcessTimes(IntPtr process,out Time creation,out Time exit,out Time kernel,out Time user);
    [StructLayout(LayoutKind.Sequential)] struct Security {public uint size;public IntPtr descriptor;public int inherit;}
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool CreatePipe(out IntPtr read,out IntPtr write,ref Security attrs,uint size);
    static string Name(IntPtr h) {var s=new StringBuilder(256);uint needed;Check(GetUserObjectInformationW(h,2,s,512,out needed));return s.ToString();}
    static string Hash(string path,long cap) {using(var f=new System.IO.FileStream(path,System.IO.FileMode.Open,System.IO.FileAccess.Read,System.IO.FileShare.Read))using(var sha=System.Security.Cryptography.SHA256.Create()){if(f.Length>cap)throw new Exception("size");byte[] b=new byte[65536];long total=0;int n;while((n=f.Read(b,0,b.Length))!=0){total+=n;if(total>cap)throw new Exception("size");sha.TransformBlock(b,0,n,null,0);}sha.TransformFinalBlock(new byte[0],0,0);return BitConverter.ToString(sha.Hash).Replace("-","").ToLowerInvariant();}}
    static void Fresh(string p) {if(!System.IO.Path.IsPathRooted(p)||System.IO.Directory.Exists(p)||System.IO.File.Exists(p)||p.Contains(".."))throw new Exception("fresh");for(string a=System.IO.Path.GetDirectoryName(p);a!=null;a=System.IO.Path.GetDirectoryName(a)){if((System.IO.File.GetAttributes(a)&System.IO.FileAttributes.ReparsePoint)!=0)throw new Exception("reparse");}System.IO.Directory.CreateDirectory(p);}
    static void Put(string p,byte[] bytes){using(var f=new System.IO.FileStream(p,System.IO.FileMode.CreateNew,System.IO.FileAccess.Write,System.IO.FileShare.Read)){f.Write(bytes,0,bytes.Length);f.Flush(true);}}
    static bool PrefixMatches(string source,string copy){using(var a=System.IO.File.OpenRead(source))using(var b=new System.IO.FileStream(copy,System.IO.FileMode.Open,System.IO.FileAccess.Read,System.IO.FileShare.ReadWrite)){if(b.Length<65536||b.Length>a.Length)return false;byte[] x=new byte[65536],y=new byte[65536];long left=b.Length;while(left>0){int n=(int)Math.Min(left,x.Length);if(a.Read(x,0,n)!=n||b.Read(y,0,n)!=n)return false;for(int i=0;i<n;i++)if(x[i]!=y[i])return false;left-=n;}return true;}}
    [StructLayout(LayoutKind.Sequential)] struct FileInfo {public uint attrs;public Time created,access,written;public uint volume,high,low,links,indexHigh,indexLow;}
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetFileInformationByHandle(IntPtr h,out FileInfo info);
    static string RootId(IntPtr h){FileInfo f;Check(GetFileInformationByHandle(h,out f));if((f.attrs&0x410)!=0x10)throw new Exception("root_type");return f.volume.ToString("x8")+f.indexHigh.ToString("x8")+f.indexLow.ToString("x8");}
    [StructLayout(LayoutKind.Sequential)] struct StartupEx { public Startup startup; public IntPtr attributes; }
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr list,int count,uint flags,ref UIntPtr bytes);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr list,uint flags,UIntPtr attribute,IntPtr value,UIntPtr bytes,IntPtr oldValue,IntPtr returned);
    [DllImport("kernel32.dll")] static extern void DeleteProcThreadAttributeList(IntPtr list);
    [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool DuplicateHandle(IntPtr sourceProcess,IntPtr source,IntPtr targetProcess,out IntPtr target,uint access,bool inherit,uint options);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateEventW(ref Security attributes,bool manual,bool initial,string name);
    [DllImport("kernel32.dll",SetLastError=true)] static extern bool SetEvent(IntPtr handle);
    static System.Collections.Generic.Dictionary<string,string> PairPins(string reg,string receipt){
        var pins=new System.Collections.Generic.Dictionary<string,string>();string parent=System.IO.Path.GetDirectoryName(reg);
        foreach(string dir in new[]{"registered-normal","registered-other"}){string p=System.IO.Path.Combine(parent,dir);foreach(string leaf in new[]{"dlpad.exe","notice.txt","install-ready.json","install-registration.json","install-registration-seal.json"}){string f=System.IO.Path.Combine(p,leaf);pins.Add(f,Hash(f,134217728));}}
        foreach(string dir in new[]{"receipt-normal","receipt-other"}){string f=System.IO.Path.Combine(parent,dir,"registration-receipt.json");pins.Add(f,Hash(f,65536));}
        foreach(string leaf in new[]{"notice.txt","foreign-sentinel"}){string f=System.IO.Path.Combine(parent,leaf);pins.Add(f,Hash(f,65536));}
        return pins;
    }
    static int Main(string[] args) {
        // cliExe, exact SHA256, fresh project evidence child, cancel|timeout.
        var savedOut=Console.Out;System.IO.StreamWriter logWriter=null;IntPtr duplicateLog=IntPtr.Zero,attributeList=IntPtr.Zero,handleArray=IntPtr.Zero;bool attributesInitialized=false;string originalDesktop=null;IntPtr stdout=GetStdHandle(-11),stderr=GetStdHandle(-12),stdin=GetStdHandle(-10);
        IntPtr ready=IntPtr.Zero,release=IntPtr.Zero,environment=IntPtr.Zero;bool readyObserved=false,eventsClosed=false;System.Collections.Generic.Dictionary<string,string> preserved=null;IntPtr rootPin=IntPtr.Zero;string originalRootId=null;bool rootPreserved=false;IntPtr job=IntPtr.Zero,read=IntPtr.Zero,write=IntPtr.Zero,nul=IntPtr.Zero;ProcessInformation child=new ProcessInformation();
        System.IO.FileStream cliPin=null;bool jobAssigned=false;uint finalJobActive=UInt32.MaxValue;bool allocated=false,forced=false,natural=false,inputSame=false,consoleClosed=false;string input=null,error="none",payload="";uint code=UInt32.MaxValue;ulong start=0;string cliHash=null;bool activeAbsent=false,preservationVerified=false;System.Threading.Tasks.Task<string> reader=null;
        try {
            if(args.Length!=6||!(args[3]=="cancel"||args[3]=="timeout")||args[1].Length!=64)throw new Exception("arguments");
            string desktop=Name(GetThreadDesktop(GetCurrentThreadId()));originalDesktop=desktop;if(!desktop.StartsWith("dlpad-cli-",StringComparison.Ordinal))throw new Exception("private_desktop_required");
            input=InputName();
            if(GetProcAddress(GetModuleHandleW("kernel32.dll"),"AllocConsoleWithOptions")==IntPtr.Zero)throw new Exception("hidden_console_api_unavailable");
            Check(DuplicateHandle(GetCurrentProcess(),stdout,GetCurrentProcess(),out duplicateLog,0,false,2));
            var logHandle=new Microsoft.Win32.SafeHandles.SafeFileHandle(duplicateLog,true);duplicateLog=IntPtr.Zero;
            logWriter=new System.IO.StreamWriter(new System.IO.FileStream(logHandle,System.IO.FileAccess.Write),new UTF8Encoding(false));logWriter.AutoFlush=true;
            cliPin=new System.IO.FileStream(args[0],System.IO.FileMode.Open,System.IO.FileAccess.Read,System.IO.FileShare.Read);cliHash=Hash(args[0],134217728);if(cliHash!=args[1])throw new Exception("cli_pin");
            Fresh(args[2]);if(!System.IO.Path.IsPathRooted(args[4])||!System.IO.Path.IsPathRooted(args[5])||args[4].Contains("..")||args[5].Contains(".."))throw new Exception("pair_paths");
            string pairParent=System.IO.Path.GetDirectoryName(args[4]);if(System.IO.Path.GetDirectoryName(args[5])!=pairParent||System.IO.Path.GetFileName(args[4])!="registered-normal"||System.IO.Path.GetFileName(args[5])!="receipt-normal"||!System.IO.Path.GetFileName(pairParent).StartsWith("dlpad-app-active-selection-owned-",StringComparison.Ordinal))throw new Exception("owned_pair_namespace");
            foreach(string raw in new[]{args[4],args[5]})for(string ancestor=raw;ancestor!=null;ancestor=System.IO.Path.GetDirectoryName(ancestor))if((System.IO.File.GetAttributes(ancestor)&System.IO.FileAttributes.ReparsePoint)!=0)throw new Exception("pair_reparse");
            preserved=PairPins(args[4],args[5]);
            var json=new System.Web.Script.Serialization.JavaScriptSerializer();json.MaxJsonLength=16384;
            // Allocate only after confirming the original private desktop. No interactive console fallback.
            FreeConsole();AllocateHidden();allocated=true;
            job=CreateJobObjectW(IntPtr.Zero,null);Check(job!=IntPtr.Zero);Limits lim=new Limits();lim.basic.flags=0x2000;Check(SetInformationJobObject(job,9,ref lim,(uint)Marshal.SizeOf(typeof(Limits))));
            Security sec=new Security();sec.size=(uint)Marshal.SizeOf(typeof(Security));sec.inherit=1;Check(CreatePipe(out read,out write,ref sec,0));Check(SetHandleInformation(read,1,0));
            nul=CreateFileW("NUL",0x80000000,3,IntPtr.Zero,3,0,IntPtr.Zero);Check(nul!=new IntPtr(-1));Check(SetHandleInformation(nul,1,1));
            string target=System.IO.Path.Combine(args[2],"active");string command=Quote(args[0])+" select-active --target "+Quote(args[4])+" --receipt-target "+Quote(args[5])+" --active-target "+Quote(target)+" --timeout-seconds "+(args[3]=="cancel"?"60":"1");
            ready=CreateEventW(ref sec,true,false,null);release=CreateEventW(ref sec,true,false,null);Check(ready!=IntPtr.Zero&&release!=IntPtr.Zero&&ready!=release);
            if(write==nul||write==IntPtr.Zero||nul==IntPtr.Zero)throw new Exception("handle_allowlist");
            UIntPtr attributeBytes=UIntPtr.Zero;
            if(InitializeProcThreadAttributeList(IntPtr.Zero,1,0,ref attributeBytes)||Marshal.GetLastWin32Error()!=122||attributeBytes.ToUInt64()==0||attributeBytes.ToUInt64()>65536)throw new Exception("attribute_size");
            attributeList=Marshal.AllocHGlobal(checked((int)attributeBytes.ToUInt64()));Check(InitializeProcThreadAttributeList(attributeList,1,0,ref attributeBytes));attributesInitialized=true;
            handleArray=Marshal.AllocHGlobal(IntPtr.Size*4);IntPtr[] inherited=new[]{write,nul,ready,release};for(int i=0;i<4;i++){for(int j=0;j<i;j++)if(inherited[i]==inherited[j])throw new Exception("duplicate_inherited");Marshal.WriteIntPtr(handleArray,IntPtr.Size*i,inherited[i]);}
            Check(UpdateProcThreadAttribute(attributeList,0,new UIntPtr(0x20002),handleArray,new UIntPtr((uint)(IntPtr.Size*4)),IntPtr.Zero,IntPtr.Zero));
            StartupEx si=new StartupEx();si.startup.cb=(uint)Marshal.SizeOf(typeof(StartupEx));si.startup.desktop=desktop;si.startup.flags=0x100;si.startup.input=nul;si.startup.output=write;si.startup.error=write;si.attributes=attributeList;
            var env=new System.Collections.Generic.SortedDictionary<string,string>(StringComparer.OrdinalIgnoreCase);foreach(System.Collections.DictionaryEntry item in Environment.GetEnvironmentVariables())env[(string)item.Key]=(string)item.Value;env["DLPAD_ACTIVE_CONTROL_READY_HANDLE"]=ready.ToInt64().ToString(System.Globalization.CultureInfo.InvariantCulture);env["DLPAD_ACTIVE_CONTROL_RELEASE_HANDLE"]=release.ToInt64().ToString(System.Globalization.CultureInfo.InvariantCulture);var block=new StringBuilder();foreach(var kv in env){block.Append(kv.Key).Append('=').Append(kv.Value).Append('\0');}block.Append('\0');environment=Marshal.StringToHGlobalUni(block.ToString());
            Check(CreateProcessW(args[0],new StringBuilder(command),IntPtr.Zero,IntPtr.Zero,true,0x80604,environment,args[2],ref si,out child));Check(AssignProcessToJobObject(job,child.process));jobAssigned=true;Time ct,et,kt,ut;Check(GetProcessTimes(child.process,out ct,out et,out kt,out ut));start=ct.Value;if(start==0||child.pid==0)throw new Exception("identity");
            Check(CloseHandle(write));write=IntPtr.Zero;
            IntPtr ownedRead=read;read=IntPtr.Zero;reader=System.Threading.Tasks.Task.Factory.StartNew(delegate {using(var f=new System.IO.FileStream(new Microsoft.Win32.SafeHandles.SafeFileHandle(ownedRead,true),System.IO.FileAccess.Read))using(var ms=new System.IO.MemoryStream()){byte[] b=new byte[1024];int n;while((n=f.Read(b,0,b.Length))!=0){if(ms.Length+n>16384)throw new Exception("stdout_bound");ms.Write(b,0,n);}return Encoding.UTF8.GetString(ms.ToArray());}});
            if(ResumeThread(child.thread)!=1)throw new Exception("resume");
            if(WaitForSingleObject(ready,20000)!=0)throw new Exception("ready_unobserved");readyObserved=true;
            Check(GetProcessTimes(child.process,out ct,out et,out kt,out ut));if(ct.Value!=start||WaitForSingleObject(child.process,0)!=258)throw new Exception("ready_process_identity");
            if(args[3]=="cancel"){Check(GenerateConsoleCtrlEvent(1,child.pid));}
            // Neither case releases: queued CTRL_BREAK must be observed by the cooperative CANCEL check; timeout uses own deadline.
            if(WaitForSingleObject(child.process,15000)!=0)throw new Exception("child_deadline");Check(GetExitCodeProcess(child.process,out code));uint active;natural=WaitJobEmpty(job,2000,out active);if(!reader.Wait(2000))throw new Exception("stdout_deadline");payload=reader.Result;if(Encoding.UTF8.GetByteCount(payload)>4096)throw new Exception("cli_receipt_bound");
            var row=json.Deserialize<System.Collections.Generic.Dictionary<string,object>>(payload.Trim());string kind=args[3]=="cancel"?"Interrupted":"TimedOut";
            activeAbsent=!System.IO.Directory.Exists(target)&&!System.IO.File.Exists(target);
            if((string)row["phase"]!="active-selection"||code!=1||!natural||(string)row["status"]!="FAIL"||(string)row["kind"]!=kind||(string)row["operation"]!="select-active"||(bool)row["mutation_may_have_occurred"]||Convert.ToInt32(row["new_active_file_caps"])!=0||(bool)row["active_root_capability_acquired"]||System.IO.Directory.Exists(target)||System.IO.File.Exists(target))throw new Exception("receipt");
            foreach(var pin in preserved)if(Hash(pin.Key,134217728)!=pin.Value)throw new Exception("pair_preservation");
            if(Hash(args[0],134217728)!=cliHash)throw new Exception("cli_preservation");preservationVerified=true;
        }catch(Exception e){error=e is System.ComponentModel.Win32Exception?"win32_"+((System.ComponentModel.Win32Exception)e).NativeErrorCode:(e.Message.Length<64&&!e.Message.Contains("\\")&&!e.Message.Contains("/")?e.Message:"collector_failure");}
        finally{if(environment!=IntPtr.Zero)Marshal.FreeHGlobal(environment);bool readyClosed=ready==IntPtr.Zero||CloseHandle(ready),releaseClosed=release==IntPtr.Zero||CloseHandle(release);eventsClosed=readyClosed&&releaseClosed;if(attributesInitialized)DeleteProcThreadAttributeList(attributeList);if(attributeList!=IntPtr.Zero)Marshal.FreeHGlobal(attributeList);if(handleArray!=IntPtr.Zero)Marshal.FreeHGlobal(handleArray);if(duplicateLog!=IntPtr.Zero)CloseHandle(duplicateLog);if(child.process!=IntPtr.Zero&&WaitForSingleObject(child.process,0)!=0){forced=true;if(jobAssigned)TerminateJobObject(job,125);else TerminateProcess(child.process,125);WaitForSingleObject(child.process,3000);}if(cliPin!=null)cliPin.Dispose();if(rootPin!=IntPtr.Zero&&rootPin!=new IntPtr(-1))CloseHandle(rootPin);if(child.thread!=IntPtr.Zero)CloseHandle(child.thread);if(child.process!=IntPtr.Zero)CloseHandle(child.process);if(write!=IntPtr.Zero)CloseHandle(write);if(read!=IntPtr.Zero)CloseHandle(read);if(nul!=IntPtr.Zero)CloseHandle(nul);if(job!=IntPtr.Zero){WaitJobEmpty(job,3000,out finalJobActive);CloseHandle(job);}SetStdHandle(-11,stdout);SetStdHandle(-12,stderr);SetStdHandle(-10,stdin);if(allocated)consoleClosed=FreeConsole();try{inputSame=input!=null&&InputName()==input;}catch{inputSame=false;}}
        bool pass=error=="none"&&!forced&&natural&&finalJobActive==0&&inputSame&&consoleClosed&&eventsClosed&&readyObserved&&activeAbsent&&preservationVerified;var serializer=new System.Web.Script.Serialization.JavaScriptSerializer();string output=serializer.Serialize(new{schema=1,status=pass?"PASS":"FAIL",case_name=args.Length==6?args[3]:"invalid",original_private_desktop=originalDesktop,owned_child_pid=child.pid,owned_child_creation_filetime=start,explicit_inherited_handle_count=4,fixture_event_count=2,original_events_closed=eventsClosed,ready_ack_observed=readyObserved,ready_ack_scope="began/parser/handler-ready/prepair/precreation",release_event_signalled=false,active_target_absent=activeAbsent,error=error,natural_exit_code=code,natural_job_zero=natural,nested_job_final_active=finalJobActive,forced_cleanup=forced,input_desktop_unchanged=inputSame,private_console_closed=consoleClosed,windowless_console_requested=true,visible_console_fallback=false,original_pair_file_count=preserved==null?0:preserved.Count,original12_source_notice_sentinel_preserved=preservationVerified,physical_stall_verified=false,cli_receipt=payload,actual_ime=false,installer_complete=false});(logWriter!=null?(System.IO.TextWriter)logWriter:savedOut).WriteLine(output.Length<=16384?output:"{\"status\":\"FAIL\",\"error\":\"receipt_bound\"}");if(logWriter!=null)logWriter.Dispose();return pass?0:1;
    }
}
