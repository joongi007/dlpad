using System;using System.IO;using System.Threading;using System.Diagnostics;
class SyntheticWorker {
static string J(string s){return "\""+s.Replace("\\","\\\\").Replace("\"","\\\"")+"\"";}
static void Main(string[] args){if(Array.IndexOf(args,"--version")>=0){Console.WriteLine("2026.10.02");return;}if(Array.IndexOf(args,"--dump-single-json")>=0){Console.WriteLine("{\"id\":\"synthetic\",\"duration\":120}");return;}
string root=Environment.GetEnvironmentVariable("DLPAD_SYNTHETIC_ROOT");File.WriteAllText(Path.Combine(root,"worker.pid"),Process.GetCurrentProcess().Id.ToString());File.WriteAllText(Path.Combine(root,"worker.creation"),Process.GetCurrentProcess().StartTime.ToUniversalTime().ToFileTimeUtc().ToString());
for(int i=0;i<600;i++){string next=Path.Combine(root,"heartbeat.next");File.WriteAllText(next,i.ToString());string current=Path.Combine(root,"heartbeat");if(File.Exists(current))File.Replace(next,current,null);else File.Move(next,current);Console.WriteLine("DLPAD_PROGRESS:{\"speed\":1024,\"eta\":"+(120-i/5)+"}");Console.Out.Flush();Thread.Sleep(200);}
int a=Array.IndexOf(args,"--paths"),b=Array.IndexOf(args,"--output");if(a<0||b<0)Environment.Exit(3);string path=Path.Combine(args[a+1],args[b+1].Replace("%(ext)s","mp4"));File.WriteAllBytes(path,new byte[65536]);Console.WriteLine("DLPAD_ARTIFACT:"+J(path));}
}