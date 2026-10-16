#!/usr/bin/env python3
"""Plan-only by default. Root executes short private A/V fixture generation."""
import argparse,json,sys,tempfile,time
from pathlib import Path
from compare_local_export_timestamps import ROOT,TEMP,plain,pinned,sha,write,metadata

def inputs(delay):
 return ['-f','lavfi','-i',"color=c=black:s=32x32:r=2:d=4,drawbox=x=0:y=0:w=iw:h=ih:color=white:t=fill:enable='gte(t,1)*lt(t,1.5)'",
         '-itsoffset',delay,'-f','lavfi','-i',
         r'aevalsrc=if(between(t\,1\,1.1)\,0.5*sin(2*PI*1000*t)\,0):s=48000:d=4']

def main():
 p=argparse.ArgumentParser(description=__doc__)
 for key in ['ffmpeg','ffprobe','host']:
  p.add_argument('--'+key,type=Path,required=True);p.add_argument('--'+key+'-sha256',required=True)
 p.add_argument('--output',type=Path,required=True);p.add_argument('--execute',action='store_true');a=p.parse_args()
 tools={key:pinned(getattr(a,key),getattr(a,key+'_sha256'))for key in ['ffmpeg','ffprobe','host']}
 out=plain(a.output);out.relative_to(ROOT/'docs/validation/evidence')
 if out.exists()or not out.parent.is_dir():raise ValueError('fresh evidence leaf')
 plan={'status':'PLAN_ONLY','native_executed':False,'audio_sync_proven':False,'cases':{'aligned':inputs('0.000'),'audio-delayed':inputs('0.200')},'scope':'4s32x32video2fps; audio48kHzmono; flash1.0; tone1.0 + intentional input offset','max_file_bytes':16777216}
 if not a.execute:print(json.dumps(plan,indent=2));return
 sys.path.insert(0,str(ROOT/'scripts'));import windows_msvc_run_selected_cli_collect as h
 n=h.n
 if tools['host']!=plain(n.TOOLS/'windows-cli-host.exe'):raise ValueError('owned original host required')
 out.mkdir();work=Path(tempfile.mkdtemp(prefix='dlpad-av-fixture-owned-',dir=TEMP));_,env=n.configure()
 result=dict(plan,status='REVIEW_REQUIRED',native_executed=True,fixture_root=str(work),fixtures={},tools={key:{'path':str(path),'sha256':getattr(a,key+'_sha256')}for key,path in tools.items()})
 try:
  for name,delay in [('aligned','0.000'),('audio-delayed','0.200')]:
   for key,path in tools.items():pinned(path,getattr(a,key+'_sha256'))
   target=work/(name+'.mp4');argv=['-nostdin','-hide_banner','-loglevel','error','-threads','1','-filter_threads','1','-max_alloc','33554432',*inputs(delay),'-map','0:v:0','-map','1:a:0','-c:v','mpeg4','-q:v','3','-threads','1','-c:a','aac','-b:a','128k','-t','4.000','-fs','16777216','-movflags','+faststart','-f','mp4','-n',n.win(target)]
   op=n.run(name+'-generate',[str(tools['host']),'60',n.win(tools['ffmpeg']),*argv],env,out,60);h.guard(op,h.bounded(out/(name+'-generate.log')))
   if not 0<target.stat().st_size<=16777216:raise ValueError('fixture byte bound')
   probe=n.run(name+'-probe',[str(tools['host']),'30',n.win(tools['ffprobe']),'-v','error','-show_entries','format=duration,start_time:stream=codec_type,start_time,duration,nb_frames,time_base','-of','json',n.win(target)],env,out,30);raw=h.bounded(out/(name+'-probe.log'));h.guard(probe,raw)
   result['fixtures'][name]={'path':str(target),'windows_path':n.win(target),'sha256':sha(target),'bytes':target.stat().st_size,'generation_seconds':op['elapsed_seconds'],'argv':op['argv'],'metadata':metadata(raw),'intentional_audio_offset_ms':int(float(delay)*1000)}
 except BaseException as e:result['status']='FAILED_PRESERVED';result['failure']=type(e).__name__+': '+str(e);raise
 finally:
  result['original_tools_preserved']=all(sha(path)==getattr(a,key+'_sha256')for key,path in tools.items());write(out/'result.json',result)
 if not result['original_tools_preserved']:raise ValueError('original tools changed')
 print(str(out/'result.json'))
if __name__=='__main__':main()
