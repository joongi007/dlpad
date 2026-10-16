#!/usr/bin/env python3
"""Root-only current MSVC UI collection; bounded raw evidence requires independent review."""
import argparse, hashlib, json, sys
from pathlib import Path
import windows_msvc_run_selected_cli_collect as h
n=h.n

def main():
 p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--kind',choices=['layout','network','tools','url','public'],required=True);p.add_argument('--ffmpeg',type=Path);p.add_argument('--fixture',type=Path);p.add_argument('--ytdlp',type=Path);p.add_argument('--deno',type=Path);p.add_argument('--public-url');a=p.parse_args()
 proof=h.plain(a.proof);out=h.plain(a.output);before=n.snapshot();whole=json.loads(h.bounded(proof));h.require_current_whole(whole,before)
 if out.exists() or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project evidence required')
 sources={'network':'windows_network_input_private_probe.cs','layout':'windows_media_workspace_layout_private_probe.cs','tools':'windows_tool_install_private_probe.cs','url':'windows_media_url_private_probe.cs','public':'windows_public_video_private_probe.cs'}
 source=n.ROOT/'scripts'/sources[a.kind];cpu=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release/dlpad-cpu.exe';host=n.TOOLS/'windows-cli-host.exe'
 if {'sha256':h.sha(cpu),'bytes':cpu.stat().st_size}!=whole['artifacts']['dlpad-cpu.exe']:raise ValueError('current CPU artifact')
 out.mkdir();inputs=out/'inputs';inputs.mkdir();h.copy(source,inputs/'observer.cs');h.copy(proof,inputs/'whole-proof.json');h.copy(cpu,inputs/'cpu.exe')
 pins={str(x):h.sha(x) for x in [source,proof,cpu,host,*inputs.iterdir()]};ops=[];result={'status':'FAIL','kind':a.kind,'source_before':before,'pins':pins,'operations':ops,'physical_pointer_verified':False,'root_review_pending':True}
 def tool_sha(x):
  if x not in [a.ffmpeg,a.ffmpeg.with_name('ffprobe.exe') if a.ffmpeg else None]:return h.sha(x)
  h.plain(x);initial=x.stat();assert 0<initial.st_size<=256*1024*1024
  digest=hashlib.sha256();total=0
  with x.open('rb') as stream:
   while block:=stream.read(65536):total+=len(block);assert total<=initial.st_size;digest.update(block)
  final=x.stat();assert total==initial.st_size and (initial.st_dev,initial.st_ino,initial.st_size,initial.st_mtime_ns)==(final.st_dev,final.st_ino,final.st_size,final.st_mtime_ns)
  return digest.hexdigest()
 _,env=n.configure()
 def run(name,args,seconds):
  op=n.run(name,[str(host),str(seconds),*args],env,out,seconds);ops.append(op);h.guard(op,h.bounded(out/(name+'.log')))
 try:
  fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319';exe=out/'observer.exe'
  run('compile',[fw+'\\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+fw+'\\WPF\\'+x+'.dll'for x in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+n.win(exe),n.win(inputs/'observer.cs')],60);pins[str(exe)]=h.sha(exe)
  args=[n.win(exe),n.win(inputs/'cpu.exe'),pins[str(inputs/'cpu.exe')],n.win(out/'native')]
  if a.kind in ['layout','network']:args+=['Light','Dark'];seconds=500;receipt='settings-result.json'
  elif a.kind=='tools':seconds=620;receipt='tool-install-result.json'
  elif a.kind=='public':
   if not all([a.ytdlp,a.ffmpeg,a.deno,a.public_url]):raise ValueError('explicit public tools and URL required')
   for x in [a.ytdlp,a.ffmpeg,a.ffmpeg.with_name('ffprobe.exe'),a.deno]:pins[str(x)]=tool_sha(x)
   args=[n.win(exe),n.win(inputs/'cpu.exe'),pins[str(inputs/'cpu.exe')],n.win(a.ytdlp),n.win(a.ffmpeg),n.win(a.deno),n.win(out/'native'),a.public_url];seconds=300;receipt='public-video-result.json'
  else:
   if not a.ffmpeg or not a.fixture:raise ValueError('URL synthetic tools and fixture required')
   for x in [a.ffmpeg,a.ffmpeg.with_name('ffprobe.exe'),a.fixture]:pins[str(x)]=tool_sha(x)
   args+=[n.win(a.ffmpeg),n.win(a.fixture)];seconds=240;receipt='media-url-result.json'
  run('native-gui',args,seconds);raw=json.loads(h.bounded(out/'native'/receipt,262144));assert raw['status']=='REVIEW_REQUIRED'
  assert n.snapshot()==before and all(tool_sha(Path(x))==digest for x,digest in pins.items())
  result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),source_unchanged=True,receipt=raw)
 finally:(out/'result.json').write_text(json.dumps(result,indent=2)+'\n')
 print(result['status'])
if __name__=='__main__':main()
