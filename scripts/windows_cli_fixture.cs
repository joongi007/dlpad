// Synthetic native CLI ownership test; no GUI, media, credentials, or user files.
using System;
using System.Diagnostics;
using System.Reflection;
using System.Threading;
class DlpadCliFixture {
    static Process Start(string executable,string arguments,bool output) {
        return Process.Start(new ProcessStartInfo(executable,arguments) {
            UseShellExecute=false,CreateNoWindow=true,RedirectStandardOutput=output,
            RedirectStandardError=output
        });
    }
    static int Main(string[] args) {
        string self=Assembly.GetExecutingAssembly().Location;
        if(args.Length>0 && args[0]=="--sleep") { Thread.Sleep(60000);return 0; }
        if(args.Length>0 && args[0]=="--tree") {
            using(Process child=Start(self,"--sleep",false)) {
                Console.WriteLine(child.Id);Console.Out.Flush();Thread.Sleep(60000);return 0;
            }
        }
        if(args.Length!=2 || args[0]!="--test") return 2;
        Process outsider=null,host=null,nested=null;
        try {
            outsider=Start(self,"--sleep",false);
            foreach(string mode in new string[] { "timeout", "cancel", "eof" }) {
                ProcessStartInfo info=new ProcessStartInfo(args[1],(mode=="timeout"?"1":"30")+" \""+self+"\" --tree") {
                    UseShellExecute=false,CreateNoWindow=true,RedirectStandardOutput=true,
                    RedirectStandardError=true,RedirectStandardInput=mode!="timeout"
                };
                if(mode!="timeout") info.EnvironmentVariables["DLPAD_NATIVE_CONTROL_STDIN"]="1";
                host=Process.Start(info);
                string line=host.StandardOutput.ReadLine();
                if(line==null) throw new Exception("owned fixture did not start: "+host.StandardError.ReadToEnd());
                nested=Process.GetProcessById(Int32.Parse(line));
                // Force a process handle before teardown to prevent PID reuse ambiguity.
                IntPtr pinned=nested.Handle;
                if(mode=="cancel") { host.StandardInput.Write("C");host.StandardInput.Flush(); }
                if(mode=="eof") host.StandardInput.Close();
                if(!host.WaitForExit(5000)||host.ExitCode!=(mode=="timeout"?124:130)) throw new Exception("host "+mode+" failed");
                if(!nested.WaitForExit(3000)) throw new Exception("owned descendant survived "+mode);
                if(outsider.HasExited) throw new Exception("unrelated fixture was killed");
                Console.WriteLine("PASS: native private CLI "+mode+" killed owned descendant; unrelated fixture survived");
                nested.Dispose();nested=null;host.Dispose();host=null;
            }
            return 0;
        } catch(Exception error) { Console.Error.WriteLine(error);return 1; }
        finally {
            if(host!=null) { if(!host.HasExited) host.Kill();host.WaitForExit();host.Dispose(); }
            if(nested!=null) { if(!nested.HasExited) nested.Kill();nested.WaitForExit();nested.Dispose(); }
            if(outsider!=null) { if(!outsider.HasExited) outsider.Kill();outsider.WaitForExit();outsider.Dispose(); }
        }
    }
}
