// Standalone bounded capability probe. Never opens the user's clipboard.
using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Security.Principal;
using System.Security.AccessControl;
class DlpadNamedClipboardProbe {
    const uint StationRights=0x000f037f, DesktopRights=0x000f01ff;
    const string Payload="dlpad synthetic clipboard probe";
    [StructLayout(LayoutKind.Sequential)] struct SA {public int length; public IntPtr descriptor; public int inherit;}
    [StructLayout(LayoutKind.Sequential,CharSet=CharSet.Unicode)] struct WC {
        public uint size,style; public WndProc procedure; public int extraClass,extraWindow;
        public IntPtr instance,icon,cursor,background; public string menu,name; public IntPtr smallIcon;
    }
    [StructLayout(LayoutKind.Sequential)] struct Flags { public int inherit,reserved; public uint flags; }
    delegate IntPtr WndProc(IntPtr h,uint message,IntPtr w,IntPtr l);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateWindowStationW(string name,uint flags,uint rights,ref SA attributes);
    [DllImport("user32.dll",SetLastError=true)] static extern bool CloseWindowStation(IntPtr handle);
    [DllImport("user32.dll")] static extern IntPtr GetProcessWindowStation();
    [DllImport("user32.dll",SetLastError=true)] static extern bool SetProcessWindowStation(IntPtr handle);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateDesktopW(string name,IntPtr device,IntPtr mode,uint flags,uint rights,ref SA attributes);
    [DllImport("user32.dll",SetLastError=true)] static extern bool CloseDesktop(IntPtr handle);
    [DllImport("user32.dll",SetLastError=true)] static extern bool SetThreadDesktop(IntPtr handle);
    [DllImport("user32.dll")] static extern IntPtr GetThreadDesktop(uint thread);
    [DllImport("kernel32.dll")] static extern uint GetCurrentThreadId();
    [DllImport("user32.dll",SetLastError=true)] static extern IntPtr OpenInputDesktop(uint flags,bool inherit,uint rights);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool GetUserObjectInformationW(IntPtr handle,int index,IntPtr data,uint size,out uint needed);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern ushort RegisterClassExW(ref WC cls);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool UnregisterClassW(string name,IntPtr instance);
    [DllImport("kernel32.dll",CharSet=CharSet.Unicode)] static extern IntPtr GetModuleHandleW(string name);
    [DllImport("user32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern IntPtr CreateWindowExW(uint ex,string cls,string title,uint style,int x,int y,int width,int height,IntPtr parent,IntPtr menu,IntPtr instance,IntPtr param);
    [DllImport("user32.dll",SetLastError=true)] static extern bool DestroyWindow(IntPtr window);
    [DllImport("user32.dll")] static extern bool IsWindow(IntPtr window);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr DefWindowProcW(IntPtr h,uint m,IntPtr w,IntPtr l);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h,uint m,IntPtr w,IntPtr l);
    [DllImport("user32.dll",SetLastError=true)] static extern bool OpenClipboard(IntPtr h);
    [DllImport("user32.dll",SetLastError=true)] static extern bool CloseClipboard();
    [DllImport("user32.dll",SetLastError=true)] static extern bool EmptyClipboard();
    [DllImport("user32.dll",SetLastError=true)] static extern IntPtr SetClipboardData(uint format,IntPtr memory);
    [DllImport("user32.dll",SetLastError=true)] static extern IntPtr GetClipboardData(uint format);
    [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr GlobalAlloc(uint flags,UIntPtr size);
    [DllImport("kernel32.dll",SetLastError=true)] static extern IntPtr GlobalLock(IntPtr memory);
    [DllImport("kernel32.dll")] static extern bool GlobalUnlock(IntPtr memory);
    [DllImport("kernel32.dll")] static extern UIntPtr GlobalSize(IntPtr memory);
    [DllImport("kernel32.dll")] static extern IntPtr GlobalFree(IntPtr memory);
    static readonly Dictionary<string,string> Fields=new Dictionary<string,string>();
    static string Quote(string value) {return "\""+value.Replace("\\","\\\\").Replace("\"","\\\"").Replace("\r","\\r").Replace("\n","\\n")+"\"";}
    static void B(string key,bool value){lock(Fields){Fields[key]=value?"true":"false";}}
    static void S(string key,string value){lock(Fields){Fields[key]=Quote(value);}}
    static void Error(string operation,int code){lock(Fields){if(!Fields.ContainsKey("failure_operation")){Fields["failure_operation"]=Quote(operation);Fields["os_error"]=code.ToString(System.Globalization.CultureInfo.InvariantCulture);}}}
    static void Cleanup(string key,bool result){int code=Marshal.GetLastWin32Error();B(key,result);if(!result){lock(Fields){Fields[key+"_os_error"]=code.ToString(System.Globalization.CultureInfo.InvariantCulture);}}}
    class Failed:Exception {public readonly string Operation;public readonly int Code; public Failed(string op,int code){Operation=op;Code=code;}}
    static void Require(bool success,string operation){if(!success)throw new Failed(operation,Marshal.GetLastWin32Error());}
    static void Assert(bool success,string operation){if(!success)throw new Failed(operation,0);}
    static string Name(IntPtr handle){uint needed;GetUserObjectInformationW(handle,2,IntPtr.Zero,0,out needed);Assert(needed>=2&&needed<=4096,"object_name_bound");IntPtr memory=Marshal.AllocHGlobal((int)needed);try{Require(GetUserObjectInformationW(handle,2,memory,needed,out needed),"object_name");return Marshal.PtrToStringUni(memory);}finally{Marshal.FreeHGlobal(memory);}}
    static string InputName(){IntPtr desktop=OpenInputDesktop(0,false,1);if(desktop==IntPtr.Zero)throw new Failed("input_desktop_read",Marshal.GetLastWin32Error());try{return Name(desktop);}finally{Require(CloseDesktop(desktop),"input_desktop_read_close");}}
    static SA Private(uint mask,out IntPtr memory){string sid=WindowsIdentity.GetCurrent().User.Value;RawSecurityDescriptor descriptor=new RawSecurityDescriptor("O:"+sid+"D:P(A;;0x"+mask.ToString("x")+";;;"+sid+")");byte[] data=new byte[descriptor.BinaryLength];descriptor.GetBinaryForm(data,0);memory=Marshal.AllocHGlobal(data.Length);Marshal.Copy(data,0,memory,data.Length);SA result=new SA();result.length=Marshal.SizeOf(typeof(SA));result.descriptor=memory;return result;}
    static bool Nonvisible(IntPtr station){uint needed;int size=Marshal.SizeOf(typeof(Flags));IntPtr data=Marshal.AllocHGlobal(size);try{Require(GetUserObjectInformationW(station,1,data,(uint)size,out needed),"station_flags");return (((Flags)Marshal.PtrToStructure(data,typeof(Flags))).flags&1)==0;}finally{Marshal.FreeHGlobal(data);}}
    static bool Owned(IntPtr station,IntPtr desktop,string stationName){return Nonvisible(station)&&GetProcessWindowStation()==station&&GetThreadDesktop(GetCurrentThreadId())==desktop&&Name(station)==stationName&&Name(desktop)=="dlpad-probe"&&!stationName.Equals("WinSta0",StringComparison.OrdinalIgnoreCase);}
    static WndProc Procedure=delegate(IntPtr h,uint m,IntPtr w,IntPtr l){if(m==0x0010){Cleanup("hidden_window_destroy_call",DestroyWindow(h));return IntPtr.Zero;}return DefWindowProcW(h,m,w,l);};
    static void Worker(IntPtr station,IntPtr desktop,IntPtr originalStation,string stationName){
        IntPtr oldDesktop=GetThreadDesktop(GetCurrentThreadId()),window=IntPtr.Zero,memory=IntPtr.Zero;bool opened=false,registered=false;string cls="dlpad-hidden-"+Guid.NewGuid().ToString("N");IntPtr instance=GetModuleHandleW(null);
        try {
            Require(SetThreadDesktop(desktop),"set_owned_thread_desktop");Assert(Owned(station,desktop,stationName),"owned_identity_before_window");
            WC wc=new WC();wc.size=(uint)Marshal.SizeOf(typeof(WC));wc.procedure=Procedure;wc.name=cls;wc.instance=instance;
            Require(RegisterClassExW(ref wc)!=0,"register_owned_class");registered=true;
            window=CreateWindowExW(0,cls,"dlpad synthetic hidden owner",0,0,0,1,1,IntPtr.Zero,IntPtr.Zero,instance,IntPtr.Zero);Require(window!=IntPtr.Zero,"create_hidden_owner");
            Assert(Owned(station,desktop,stationName),"owned_identity_before_clipboard");B("owned_identity_verified",true);
            Require(OpenClipboard(window),"open_owned_clipboard");opened=true;Require(EmptyClipboard(),"empty_owned_initial");
            Assert(Payload.Length<=64,"payload_bound");byte[] value=Encoding.Unicode.GetBytes(Payload+"\0");memory=GlobalAlloc(2,(UIntPtr)(uint)value.Length);Require(memory!=IntPtr.Zero,"allocate_payload");IntPtr ptr=GlobalLock(memory);Require(ptr!=IntPtr.Zero,"lock_payload");Marshal.Copy(value,0,ptr,value.Length);GlobalUnlock(memory);
            Require(SetClipboardData(13,memory)!=IntPtr.Zero,"set_owned_unicode");memory=IntPtr.Zero;
            Require(CloseClipboard(),"close_owned_write");opened=false;
            Assert(Owned(station,desktop,stationName),"owned_identity_before_read");Require(OpenClipboard(window),"open_owned_read");opened=true;
            IntPtr stored=GetClipboardData(13);Require(stored!=IntPtr.Zero,"get_owned_unicode");ulong size=GlobalSize(stored).ToUInt64();Assert(size>=2&&size<=130&&(size%2)==0,"payload_read_bound");ptr=GlobalLock(stored);Require(ptr!=IntPtr.Zero,"lock_owned_read");string actual;try{actual=Marshal.PtrToStringUni(ptr,(int)size/2).TrimEnd('\0');}finally{GlobalUnlock(stored);}Assert(actual==Payload,"unicode_compare");B("synthetic_roundtrip",true);
            Assert(Owned(station,desktop,stationName),"owned_identity_before_clear");Require(EmptyClipboard(),"clear_owned_payload");B("owned_clipboard_cleared",true);Require(CloseClipboard(),"close_owned_read");opened=false;
        }catch(Failed error){Error(error.Operation,error.Code);}catch{Error("managed_worker_failure",0);}finally{
            // Clear only while exact original private station/desktop remains current.
            if(opened){try{if(Owned(station,desktop,stationName)){Cleanup("cleanup_owned_clear",EmptyClipboard());}else{B("cleanup_owned_clear",false);}}catch{B("cleanup_owned_clear",false);}Cleanup("cleanup_clipboard_closed",CloseClipboard());}
            if(memory!=IntPtr.Zero)GlobalFree(memory);
            if(window!=IntPtr.Zero){SendMessageW(window,0x0010,IntPtr.Zero,IntPtr.Zero);B("hidden_window_destroyed",!IsWindow(window));}else B("hidden_window_destroyed",true);
            Cleanup("owned_class_unregistered",!registered||UnregisterClassW(cls,instance));
            bool restored=SetProcessWindowStation(originalStation);Cleanup("worker_process_station_restored",restored);Cleanup("worker_thread_desktop_restored",restored&&SetThreadDesktop(oldDesktop));
        }
    }
    static int Main(string[] args){
        if(args.Length!=1)return 2;string output;try{output=Path.GetFullPath(args[0]);if(!Directory.Exists(output)||Directory.GetFileSystemEntries(output).Length!=0||(File.GetAttributes(output)&FileAttributes.ReparsePoint)!=0)return 2;}catch{return 2;}
        IntPtr original=GetProcessWindowStation(),station=IntPtr.Zero,desktop=IntPtr.Zero,sdStation=IntPtr.Zero,sdDesktop=IntPtr.Zero;bool workerDone=true;string before=null;string name="dlpad-clip-"+Guid.NewGuid().ToString("N");
        B("user_clipboard_accessed",false);B("visible_desktop_used",false);S("owned_station_name",name);
        try{before=InputName();S("input_desktop_before",before);SA sa=Private(StationRights,out sdStation);station=CreateWindowStationW(name,1,StationRights,ref sa);Require(station!=IntPtr.Zero,"create_only_named_station");Assert(Name(station)==name&&Nonvisible(station),"new_station_exact_nonvisible_name");Require(SetProcessWindowStation(station),"set_owned_process_station");SA desk=Private(DesktopRights,out sdDesktop);desktop=CreateDesktopW("dlpad-probe",IntPtr.Zero,IntPtr.Zero,0,DesktopRights,ref desk);Require(desktop!=IntPtr.Zero,"create_owned_desktop");Assert(Name(desktop)=="dlpad-probe","new_desktop_exact_name");
            Thread thread=new Thread(delegate(){Worker(station,desktop,original,name);});thread.IsBackground=true;thread.SetApartmentState(ApartmentState.STA);workerDone=false;thread.Start();workerDone=thread.Join(10000);Assert(workerDone,"owned_worker_deadline");
        }catch(Failed error){Error(error.Operation,error.Code);}catch{Error("managed_main_failure",0);}finally{
            if(workerDone){Cleanup("process_station_restored",SetProcessWindowStation(original));if(desktop!=IntPtr.Zero)Cleanup("owned_desktop_closed",CloseDesktop(desktop));else B("owned_desktop_closed",true);if(station!=IntPtr.Zero)Cleanup("owned_station_closed",CloseWindowStation(station));else B("owned_station_closed",true);try{string after=InputName();S("input_desktop_after",after);B("input_desktop_unchanged",before!=null&&before==after);}catch(Failed error){Error(error.Operation,error.Code);}catch{Error("input_desktop_after_failure",0);}}
            else{B("process_station_restored",false);B("owned_desktop_closed",false);B("owned_station_closed",false);B("owned_worker_requires_outer_job_cleanup",true);}
            if(sdStation!=IntPtr.Zero)Marshal.FreeHGlobal(sdStation);if(sdDesktop!=IntPtr.Zero)Marshal.FreeHGlobal(sdDesktop);
        }
        bool passed;byte[] json;lock(Fields){passed=!Fields.ContainsKey("failure_operation")&&Fields.ContainsKey("synthetic_roundtrip")&&Fields["synthetic_roundtrip"]=="true"&&Fields["process_station_restored"]=="true"&&Fields["owned_desktop_closed"]=="true"&&Fields["owned_station_closed"]=="true"&&Fields["input_desktop_unchanged"]=="true"&&Fields["worker_thread_desktop_restored"]=="true"&&Fields["hidden_window_destroyed"]=="true"&&Fields["owned_class_unregistered"]=="true"&&Fields["owned_clipboard_cleared"]=="true";
        B("passed",passed);List<string> values=new List<string>();foreach(KeyValuePair<string,string> field in Fields)values.Add(Quote(field.Key)+":"+field.Value);json=Encoding.UTF8.GetBytes("{"+String.Join(",",values.ToArray())+"}");}if(json.Length>65536)return 2;
        try{using(FileStream file=new FileStream(Path.Combine(output,"result.json"),FileMode.CreateNew,FileAccess.Write,FileShare.Read)){file.Write(json,0,json.Length);file.Flush();}}catch{return 2;}return passed?0:1;
    }
}
