// Standalone synthetic seeder draft; no GUI/DB/shell/native execution by author.
using System;
using System.IO;
using System.Diagnostics;
using System.Text;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Runtime.InteropServices;
using System.Collections.Generic;
using System.Web.Script.Serialization;
using Microsoft.Win32.SafeHandles;
public static class PeriodicSeed {
 [StructLayout(LayoutKind.Sequential)] struct SA { public int length;public IntPtr descriptor;public int inherit; }
 [DllImport("advapi32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool ConvertStringSecurityDescriptorToSecurityDescriptor(string text,uint revision,out IntPtr descriptor,out uint size);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern bool CreateDirectory(string name,ref SA security);
 [DllImport("kernel32.dll",CharSet=CharSet.Unicode,SetLastError=true)] static extern SafeFileHandle CreateFile(string name,uint access,uint share,IntPtr security,uint creation,uint flags,IntPtr template);
 [DllImport("kernel32.dll")] static extern IntPtr LocalFree(IntPtr value);
 static void Require(bool v,string code){if(!v)throw new InvalidOperationException(code);}
 static void Plain(string path){string full=Path.GetFullPath(path);Require(Path.IsPathRooted(path)&&full.Length<32700,"absolute bounded path");for(string p=full;!String.IsNullOrEmpty(p);p=Path.GetDirectoryName(p)){if(Directory.Exists(p)||File.Exists(p))Require((File.GetAttributes(p)&FileAttributes.ReparsePoint)==0,"reparse rejected");}}
 static string Sha(byte[] value){using(SHA256 h=SHA256.Create()){return BitConverter.ToString(h.ComputeHash(value)).Replace("-","").ToLowerInvariant();}}
 static void NewFile(string path,byte[] data){Require(data.Length<=4096,"file bound");using(FileStream f=new FileStream(path,FileMode.CreateNew,FileAccess.Write,FileShare.Read)){f.Write(data,0,data.Length);f.Flush(true);}Plain(path);}
 static void NewDir(string path,IntPtr descriptor){Require(!Directory.Exists(path)&&!File.Exists(path),"fresh directory required");SA sa=new SA();sa.length=Marshal.SizeOf(typeof(SA));sa.descriptor=descriptor;sa.inherit=0;Require(CreateDirectory(path,ref sa),"create-only private directory failed");Plain(path);}
 public static int Main(string[] args){IntPtr descriptor=IntPtr.Zero;SafeFileHandle rootPin=null;string stage="start";try{
  Require(args.Length==1&&args[0]=="--fixed-periodic-seed","fixed argument required");string temp=Path.GetFullPath(Path.GetTempPath()).TrimEnd(Path.DirectorySeparatorChar);Plain(temp);
  string root=Path.Combine(temp,"dlpad-library-scan-closewait-owned-"+Guid.NewGuid().ToString("N"));Require(String.Equals(Path.GetDirectoryName(root),temp,StringComparison.OrdinalIgnoreCase),"TEMP direct child required");
  using(WindowsIdentity identity=WindowsIdentity.GetCurrent()){Require(identity.User!=null&&identity.ImpersonationLevel==TokenImpersonationLevel.None,"original process identity required");uint sdSize;Require(ConvertStringSecurityDescriptorToSecurityDescriptor("D:P(A;OICI;FA;;;"+identity.User.Value+")",1,out descriptor,out sdSize),"descriptor failed");}
  stage="root";NewDir(root,descriptor);rootPin=CreateFile(root,0x80,3,IntPtr.Zero,3,0x02200000,IntPtr.Zero);Require(!rootPin.IsInvalid,"root pin failed");
  string data=Path.Combine(root,"data"),media=Path.Combine(root,"media");NewDir(data,descriptor);NewDir(media,descriptor);
  byte[] nonce=new byte[32];using(RandomNumberGenerator rng=RandomNumberGenerator.Create()){rng.GetBytes(nonce);}bool nonzero=false;foreach(byte b in nonce)nonzero|=b!=0;Require(nonzero,"nonce failed");byte[] prefix=Encoding.ASCII.GetBytes("dlpad-library-scan-closewait-v1\n");byte[] cap=new byte[prefix.Length+nonce.Length];Buffer.BlockCopy(prefix,0,cap,0,prefix.Length);Buffer.BlockCopy(nonce,0,cap,prefix.Length,nonce.Length);
  byte[] sentinel=Encoding.ASCII.GetBytes("dlpad-library-scan-closewait-foreign-v1");NewFile(Path.Combine(root,"fixture.cap"),cap);NewFile(Path.Combine(root,"foreign-sentinel"),sentinel);
  stage="media";byte[] payload=new byte[]{1,2,3};Stopwatch timer=Stopwatch.StartNew();for(int i=0;i<10000;i++){Require(timer.Elapsed.TotalSeconds<180,"seed deadline");NewFile(Path.Combine(media,"media-"+i.ToString("D5")+".mp4"),payload);}
  Require(!File.Exists(Path.Combine(data,"library.sqlite")),"DB must not be seeded");int count=0;foreach(string file in Directory.EnumerateFiles(media)){Require(++count<=10000,"file count bound");Plain(file);Require(new FileInfo(file).Length==3,"size mismatch");}Require(count==10000,"file count mismatch");Plain(root);
  Dictionary<string,object> result=new Dictionary<string,object>();result["phase"]="seed-ready";result["schema"]=1;result["owned_root"]=root;result["data"]=data;result["media"]=media;result["fixture_cap_sha256"]=Sha(cap);result["count"]=count;result["sentinel_sha256"]=Sha(sentinel);result["synthetic_only"]=true;result["db_seeded"]=false;result["fixture_retained"]=true;
  string json=new JavaScriptSerializer().Serialize(result);Require(Encoding.UTF8.GetByteCount(json)<=16384,"JSON bound");Console.WriteLine(json);return 0;
 }catch(Exception error){Console.Error.WriteLine("seed-failed stage="+stage+" type="+error.GetType().Name+" os="+Marshal.GetLastWin32Error());return 2;}finally{if(rootPin!=null)rootPin.Dispose();if(descriptor!=IntPtr.Zero)LocalFree(descriptor);}}
}
