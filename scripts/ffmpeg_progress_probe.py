#!/usr/bin/env python3
"""Owned loopback synthetic FFmpeg/yt-dlp progress forwarding experiment."""
import argparse
import functools
import http.server
import json
import os
from pathlib import Path
import selectors
import subprocess
import threading
import time

LIMIT=256*1024

def execute(name, argv, output):
    started=time.monotonic(); child=subprocess.Popen(argv,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    chunks={'stdout':bytearray(),'stderr':bytearray()};first={};selector=selectors.DefaultSelector()
    for kind,pipe in [('stdout',child.stdout),('stderr',child.stderr)]:selector.register(pipe,selectors.EVENT_READ,kind)
    try:
        while selector.get_map():
            if time.monotonic()-started>30:raise TimeoutError(name+' deadline')
            for key,_ in selector.select(.1):
                data=os.read(key.fd,4096)
                if not data:selector.unregister(key.fileobj);continue
                kind=key.data
                if b'progress=' in data or b'frame=' in data:first.setdefault(kind,time.monotonic()-started)
                if len(chunks[kind])+len(data)>LIMIT:raise RuntimeError('bounded capture exceeded')
                chunks[kind].extend(data)
        code=child.wait(timeout=3)
    finally:
        selector.close()
        if child.poll() is None:child.kill();child.wait()
        child.stdout.close();child.stderr.close()
    for kind,data in chunks.items():(output/(name+'.'+kind)).write_bytes(data)
    records={}
    for kind,data in chunks.items():
        lines=data.decode('utf8','replace').splitlines();records[kind]={'frame_lines':sum(x.startswith('frame=') and x[6:].strip().isdigit() for x in lines),'progress_continue':sum(x=='progress=continue' for x in lines),'progress_end':sum(x=='progress=end' for x in lines),'speed_lines':[x for x in lines if x.startswith('speed=')][:16],'out_time_us':[x for x in lines if x.startswith('out_time_us=')][:16],'bytes':len(data)}
    return {'name':name,'exit_code':code,'elapsed':time.monotonic()-started,'first_machine_observed':first,'streams':records,'argv':argv}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--ffmpeg',default='/usr/bin/ffmpeg');p.add_argument('--ytdlp',required=True);a=p.parse_args()
    if not a.output.is_absolute() or a.output.parent!=Path('/tmp') or a.output.exists():p.error('new direct /tmp output only')
    a.output.mkdir();root=a.output/'fixtures';root.mkdir();results=[]
    for filename,source,codec in [('video.mp4','color=c=blue:s=160x90:r=10','libx264'),('audio.m4a','sine=frequency=440:sample_rate=8000','aac')]:
        args=[a.ffmpeg,'-nostdin','-hide_banner','-loglevel','error','-f','lavfi','-i',source,'-t','6','-c:v' if filename.endswith('mp4') else '-c:a',codec,str(root/filename)]
        r=execute('generate-'+filename,args,a.output);assert r['exit_code']==0,r;results.append(r)
    results.append(execute('direct-stderr',[a.ffmpeg,'-nostdin','-hide_banner','-loglevel','error','-nostats','-progress','pipe:2','-i',str(root/'video.mp4'),'-f','null','-'],a.output))
    class Handler(http.server.SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
    server=http.server.ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=str(root)));thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
    origin=f'http://127.0.0.1:{server.server_port}'
    try:
        for name,extra in [('fast',['--download-sections','*1-4']),('accurate',['--download-sections','*1-4','--force-keyframes-at-cuts'])]:
            args=[a.ytdlp,'--ignore-config','--no-plugin-dirs','--no-playlist','--ffmpeg-location',a.ffmpeg,'--postprocessor-args','ffmpeg:-progress pipe:2 -nostats','--external-downloader-args','ffmpeg:-progress pipe:2 -nostats','--output',str(root/(name+'.%(ext)s'))]+extra+['--',origin+'/video.mp4']
            results.append(execute(name,args,a.output))
        info={'id':'merge','title':'synthetic-merge','duration':6,'extractor':'generic','webpage_url':origin+'/original','formats':[{'format_id':'v','url':origin+'/video.mp4','ext':'mp4','vcodec':'h264','acodec':'none','width':160,'height':90},{'format_id':'a','url':origin+'/audio.m4a','ext':'m4a','vcodec':'none','acodec':'aac'}]}
        metadata=root/'merge.info.json';metadata.write_text(json.dumps(info))
        args=[a.ytdlp,'--ignore-config','--no-plugin-dirs','--ffmpeg-location',a.ffmpeg,'--load-info-json',str(metadata),'-f','v+a','--postprocessor-args','ffmpeg:-progress pipe:2 -nostats','--external-downloader-args','ffmpeg:-progress pipe:2 -nostats','--output',str(root/'merged.%(ext)s')]
        results.append(execute('merge',args,a.output))
    finally:server.shutdown();server.server_close();thread.join(timeout=3)
    result={'scope':'owned loopback synthetic fixtures only; no GUI/user media','results':results};(a.output/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))

if __name__=='__main__':main()
