using System;
public static class HostWrapperTests {
 const string Rust=@"D:\project\dlpad\.local-tools\windows\rust\bin\rustc.exe";
 static void Yes(bool b){if(!b)throw new Exception("policy regression");}
 static void Bad(string[] args){try{string mode;MsvcHostRustcWrapper.Transform(args,out mode);}catch{ return; }throw new Exception("accepted malformed wrapper argv");}
 public static int Main(){try{string mode;var a=MsvcHostRustcWrapper.Transform(new[]{Rust,"--crate-name","quote"},out mode);Yes(mode=="gnu-host-self-contained"&&a[a.Length-1]=="link-self-contained=yes");a=MsvcHostRustcWrapper.Transform(new[]{Rust,"--target","x86_64-pc-windows-msvc","--crate-name","synthetic"},out mode);Yes(mode=="msvc-target-unchanged"&&Array.IndexOf(a,"link-self-contained=yes")==-1);a=MsvcHostRustcWrapper.Transform(new[]{Rust,"--target=x86_64-pc-windows-gnu","--crate-name","synthetic"},out mode);Yes(a[a.Length-1]=="link-self-contained=yes");a=MsvcHostRustcWrapper.Transform(new[]{Rust,"-vV"},out mode);Yes(mode=="query-unchanged"&&a.Length==1);Bad(new[]{Rust,"--target"});Bad(new[]{Rust,"--target=unknown"});Bad(new[]{Rust,"--target","x86_64-pc-windows-msvc","--target=x86_64-pc-windows-gnu"});Bad(new[]{@"C:\Windows\rustc.exe","--version"});Console.WriteLine("wrapper-policy-8 PASS");return 0;}catch{Console.Error.WriteLine("wrapper-policy FAIL");return 1;}}
}
