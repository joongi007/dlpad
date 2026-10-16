using System;using System.IO;using System.Text;using System.Runtime.InteropServices;using System.Collections.Generic;using System.Web.Script.Serialization;
public static class RemovalCliSeed{
static void Require(bool ok,string text){if(!ok)throw new Exception(text);}
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
public static int Main(string[]args){try{Require(args.Length==1,"owned TEMP parent");S10Plain(args[0]);string root=Path.Combine(args[0],"dlpad-bundle-remove-cli-owned-"+Guid.NewGuid().ToString("N"));S10PrivateNewDirectory(root);S10PrivateNewDirectory(Path.Combine(root,"source"));Console.WriteLine(new JavaScriptSerializer().Serialize(new{schema=1,status="OWNED_PRIVATE_SEEDED",owned_root=root,current_sid_protected_acl_verified=true}));return 0;}catch{Console.WriteLine("{\"status\":\"FAIL\"}");return 1;}}}
