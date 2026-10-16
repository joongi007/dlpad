using System;
using System.IO;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Web.Script.Serialization;

// Wrap the existing private-desktop settings collector. No global input or fallback.
public static class PreparedLaunchObserver {
 [StructLayout(LayoutKind.Sequential)] struct FileIdentity {
  public ulong volume;
  [MarshalAs(UnmanagedType.ByValArray,SizeConst=16)] public byte[] id;
 }
 [DllImport("kernel32.dll",SetLastError=true)] static extern bool GetFileInformationByHandleEx(IntPtr file,int kind,out FileIdentity info,uint bytes);
 static string ReadIdentity(FileStream pin,ulong volume,string expected) {
  FileIdentity info;
  if(!GetFileInformationByHandleEx(pin.SafeFileHandle.DangerousGetHandle(),18,out info,(uint)Marshal.SizeOf(typeof(FileIdentity))))throw new IOException("Original file identity query failed");
  if(info.id==null||info.id.Length!=16)throw new IOException("Native ID bound");
  string hex=BitConverter.ToString(info.id).Replace("-","").ToLowerInvariant();
  if(info.volume!=volume||hex!=expected)throw new IOException("Original prepared file identity mismatch");
  return hex;
 }
 static string Hash(FileStream pin) {
  if(pin.Length<=0||pin.Length>134217728)throw new IOException("Payload size bound");
  pin.Position=0;using(var sha=SHA256.Create())return BitConverter.ToString(sha.ComputeHash(pin)).Replace("-","").ToLowerInvariant();
 }
 public static int Main(string[] args) {
  try {
   if(args.Length!=7)throw new ArgumentException("Binary SHA output seed select volume ID required");
   ulong volume;if(!UInt64.TryParse(args[5],out volume)||args[6].Length!=32)throw new ArgumentException("Native identity syntax");
   foreach(char c in args[6])if(!((c>='0'&&c<='9')||(c>='a'&&c<='f')))throw new ArgumentException("Native ID hex syntax");
   string exe=Path.GetFullPath(args[0]);string output=Path.GetFullPath(args[2]);
   using(var pin=new FileStream(exe,FileMode.Open,FileAccess.Read,FileShare.Read)) {
    string before=ReadIdentity(pin,volume,args[6]);string digest=Hash(pin);
    if(digest!=args[1])throw new IOException("Prepared payload SHA mismatch");
    int code=CookiePrivateGui.Main(new[]{exe,args[1],output,args[3],args[4]});
    if(code!=0)return code;
    string after=ReadIdentity(pin,volume,args[6]);if(Hash(pin)!=digest)throw new IOException("Prepared payload changed during launch");
    var result=new Dictionary<string,object>{{"schema",1},{"status","REVIEW_REQUIRED"},{"executable",exe},{"sha256",digest},{"volume_serial",volume},{"file_id_hex_before",before},{"file_id_hex_after",after},{"actual_launch_uses_original_pinned_prepared_file",true},{"private_settings_collector_exit_code",code},{"installer_lifecycle_verified",false},{"upgrade_verified",false},{"trusted_provenance",false}};
    byte[] bytes=Encoding.UTF8.GetBytes(new JavaScriptSerializer(){MaxJsonLength=16384}.Serialize(result));
    if(bytes.Length>16384)throw new IOException("Receipt bound");
    using(var file=new FileStream(Path.Combine(output,"prepared-launch-receipt.json"),FileMode.CreateNew,FileAccess.Write,FileShare.Read)){file.Write(bytes,0,bytes.Length);file.Flush(true);}
    Console.WriteLine("prepared-launch-original-file-verified=true");return 0;
   }
  }catch(Exception e){Console.Error.WriteLine("Prepared launch failed: "+e.GetType().Name);return 2;}
 }
}
