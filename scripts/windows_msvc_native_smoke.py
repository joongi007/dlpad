"""Root-only future execution. No supply/install/adoption; fixed project tool paths."""
import argparse,hashlib,json,os,pathlib,sys,struct
ROOT=pathlib.Path('/mnt/d/project/dlpad');sys.path.insert(0,str(ROOT/'scripts'))
import windows_current_workspace_probe as n
SUP=n.TOOLS/'msvc-supply-20261006'
def pin(p):
 if not p.is_absolute() or '..'in p.parts or any(x.is_symlink()for x in [p,*p.parents]):raise ValueError('plain tool input')
 size=p.stat().st_size
 if not p.is_file()or size<=0 or size>256*1024*1024:raise ValueError('input bound')
 h=hashlib.sha256();total=0
 with p.open('rb')as f:
  while b:=f.read(65536):
   total+=len(b)
   if total>size:raise ValueError('growth')
   h.update(b)
 if total!=size or p.stat().st_size!=size:raise ValueError('size changed')
 return {'bytes':size,'sha256':h.hexdigest()}
def main():
 a=argparse.ArgumentParser();a.add_argument('--output',type=pathlib.Path,required=True);a.add_argument('--execute-root-smoke',action='store_true');o=a.parse_args()
 if not o.execute_root_smoke:raise ValueError('draft requires explicit root execution')
 out=o.output
 if not out.is_absolute()or '..'in out.parts or any(p.is_symlink()for p in [out,*out.parents])or not out.is_relative_to(ROOT/'docs/validation/evidence'):raise ValueError('fresh evidence path')
 out.mkdir(exist_ok=False);before=n.snapshot();result={'status':'FAIL','release_ready':False,'msvc_full_workspace_verified':False,'sdk_supply_integrity_verified_by_this_probe':False};pins={}
 try:
  std=SUP/'sysroot';sdk=SUP/'vs16-sysroot/Windows Kits/10';crt=SUP/'vs16-sysroot/VC/Tools/MSVC/14.29.16.10';link=SUP/'clang-driver-native/clang-tools/lld-link.exe';cc=link.with_name('clang-cl.exe');rust=n.TOOLS/'rust/bin/rustc.exe'
  libs=[sdk/'Lib/10.0.22621/um/x64',sdk/'Lib/10.0.22621/ucrt/x64',crt/'lib/x64']
  for directory in libs:
   if not directory.is_dir():raise ValueError('actual SDK/CRT layout incomplete')
  files=[n.TOOLS/'windows-cli-host.exe',rust,link,cc,libs[0]/'kernel32.lib',libs[1]/'ucrt.lib',libs[1]/'libucrt.lib',libs[2]/'libcmt.lib',libs[2]/'libvcruntime.lib',crt/'include/stdio.h',sdk/'Include/10.0.22621/ucrt/stdio.h']
  # Header stdio belongs to UCRT; do not invent a duplicate CRT stdio requirement.
  files.remove(crt/'include/stdio.h');files += [crt/'include/vcruntime.h']
  stdlib=std/'lib/rustlib/x86_64-pc-windows-msvc/lib';rlibs=list(stdlib.glob('libstd-*.rlib'))
  if len(rlibs)!=1:raise ValueError('exact std rlib required')
  files+=rlibs
  files.append(pathlib.Path(__file__).absolute());pins={str(p):pin(p)for p in files};values,env=n.configure()
  for key in ['LIB','LIBPATH','INCLUDE','CL','_CL_','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER','CC_x86_64_pc_windows_gnu','AR_x86_64_pc_windows_gnu']:env.pop(key,None)
  env['WSLENV']=':'.join(s for s in env.get('WSLENV','').split(':')if s.split('/')[0]not in ['LIB','LIBPATH','INCLUDE','CL','_CL_','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER','CC_x86_64_pc_windows_gnu','AR_x86_64_pc_windows_gnu'])
  # Explicit argv, no shell/compiler discovery/global PATH update.
  (out/'collector.py').write_bytes(pathlib.Path(__file__).read_bytes());source=out/'smoke.rs';source.write_text('fn main(){assert_eq!(std::env::args().nth(1).as_deref(),Some("--smoke"));let t=std::time::Instant::now();let p=std::path::PathBuf::from("synthetic-only");assert_eq!(p.as_os_str(),"synthetic-only");assert!(t.elapsed().as_secs()<60);println!("MSVC_STD_SMOKE_OK");}\n')
  csource=out/'headers.c';csource.write_text('#include <stdio.h>\n#include <vcruntime.h>\n#include <windows.h>\nint main(void){return puts("MSVC_HEADERS_CRT_SMOKE_OK")<0;}\n')
  def run(name,tool,args,seconds):
   rec=n.run(name,[str(n.TOOLS/'windows-cli-host.exe'),str(seconds),n.win(tool),*args],env,out,seconds);raw=(out/(name+'.log')).read_bytes()
   if rec['passed']is not True or rec['source_unchanged']is not True or b'natural-zero=True'not in raw or b'final-active=0'not in raw or b'forced=False'not in raw or b'private-desktop-closed=True'not in raw or b'dlpad-input-desktop-unchanged=True'not in raw:raise ValueError('native original Job/private terminal')
   return rec,raw
  result['rustc_version']=run('rustc-version',rust,['--version','--verbose'],30)[0]
  exe=out/'smoke.exe';args=[n.win(source),'--edition=2024','--target','x86_64-pc-windows-msvc','--sysroot',n.win(std),'-C','linker='+n.win(link),'-C','panic=abort','-C','target-feature=+crt-static','-o',n.win(exe)]
  for d in libs:args+=['-L','native='+n.win(d)]
  result['rust_compile']=run('rust-compile',rust,args,90)[0];pins[str(exe)]=pin(exe)
  rec,raw=run('rust-run',exe,['--smoke'],30);assert b'MSVC_STD_SMOKE_OK'in raw;result['rust_run']=rec
  cexe=out/'headers.exe';ca=['-resource-dir',n.win(n.TOOLS/'llvm-mingw/lib/clang/23'),'--target=x86_64-pc-windows-msvc','/nologo','/MT','/c',n.win(csource),'/Fo'+n.win(out/'headers.obj')]
  for d in [crt/'include',sdk/'Include/10.0.22621/ucrt',sdk/'Include/10.0.22621/shared',sdk/'Include/10.0.22621/um']:ca+=['/imsvc',n.win(d)]
  result['c_compile']=run('c-compile',cc,ca,90)[0]
  result['c_link']=run('c-link',link,['/nologo','/machine:x64','/subsystem:console','/out:'+n.win(cexe),n.win(out/'headers.obj')]+['/libpath:'+n.win(d)for d in libs],90)[0];pins[str(cexe)]=pin(cexe)
  rec,raw=run('c-run',cexe,['--smoke'],30);assert b'MSVC_HEADERS_CRT_SMOKE_OK'in raw;result['c_run']=rec
  for image in [exe,cexe]:
   with image.open('rb')as f:header=f.read(4096)
   if header[:2]!=b'MZ':raise ValueError('PE magic')
   offset=struct.unpack_from('<I',header,60)[0]
   if offset>len(header)-26 or header[offset:offset+4]!=b'PE\0\0' or struct.unpack_from('<H',header,offset+4)[0]!=0x8664:raise ValueError('AMD64 PE header')
  if n.snapshot()!=before or any(pin(pathlib.Path(p))!=v for p,v in pins.items()):raise ValueError('original pins changed')
  result.update(status='PASS_NARROW_MSVC_STD_HEADERS_CRT_SMOKE',source_before=before,source_after=n.snapshot(),pins=pins)
 finally:(out/'result.json').write_text(json.dumps(result,indent=2))
if __name__=='__main__':main()
