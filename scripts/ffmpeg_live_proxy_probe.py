#!/usr/bin/env python3
"""Linux owned FIFO progress sink through real yt-dlp postprocessors; throwaway."""
import argparse
import ctypes
import functools
import http.server
import json
import math
import os
from pathlib import Path
import selectors
import socket
import subprocess
import sys
import threading
import time
from ffmpeg_progress_probe import execute
from long_job_soak import cleanup, process_sample

LIMIT=64*1024


def pin_owned_tree(guardian_pid, identities, descendants):
    todo=[guardian_pid]
    while todo and len(descendants)<32:
        parent=todo.pop()
        try: children=Path(f'/proc/{parent}/task/{parent}/children').read_text().split()
        except FileNotFoundError:continue
        for value in children:
            pid=int(value)
            if pid in descendants:continue
            try:descriptor=os.pidfd_open(pid)
            except ProcessLookupError:continue
            identities.append(descriptor);descendants.append(pid);todo.append(pid)


def trial(name, args, guardian_binary, folder, cancel=False):
    fifo=folder/(name+'.fifo');os.mkfifo(fifo,0o600)
    descriptor=os.open(fifo,os.O_RDWR|os.O_NONBLOCK|os.O_CLOEXEC)
    control,peer=socket.socketpair();guardian=None;identities=[];descendants=[];selector=selectors.DefaultSelector();started=time.monotonic()
    result={'name':name,'scope':'Linux FIFO dedicated machine stream','records':0,'buffer_limit':LIMIT,'cancelled':False,'record_fields':[],'argv':args,'human_stdout_bytes':0,'human_stderr_bytes':0}
    line=bytearray();record={};line_count=0;record_bytes=0;latest=None;first=None;cancel_at=None
    argv=[x.replace('@PROGRESS@',str(fifo)) for x in args]
    try:
        guardian=subprocess.Popen([str(guardian_binary),'--dlpad-internal-guardian',str(peer.fileno()),'--']+argv,pass_fds=(peer.fileno(),),stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
        peer.close();identities.append(os.pidfd_open(guardian.pid));control.settimeout(5)
        if control.recv(1)!=b'R':raise RuntimeError('guardian readiness failed')
        for pipe,kind in [(guardian.stdout,'stdout'),(guardian.stderr,'stderr')]:selector.register(pipe,selectors.EVENT_READ,kind)
        selector.register(descriptor,selectors.EVENT_READ,'progress')
        while True:
            now=time.monotonic()
            if now-started>20:raise TimeoutError('owned trial timeout')
            if cancel and cancel_at is not None and now>=cancel_at and not result['cancelled']:
                pin_owned_tree(guardian.pid,identities,descendants)
                control.close()  # Actual owner-loss cleanup; no global PID scan/kill.
                result['cancelled']=True
            for key,_ in selector.select(.1):
                data=os.read(key.fd,4096)
                if key.data!='progress':
                    if not data:selector.unregister(key.fileobj)
                    else:
                        keyname='human_'+key.data+'_bytes';result[keyname]+=len(data)
                        tailkey='human_'+key.data+'_tail';result[tailkey]=(result.get(tailkey,'')+data.decode('utf8','replace'))[-1024:]
                        if result[keyname]>256*1024:raise RuntimeError('human log bound')
                    continue
                for byte in data:
                    if byte!=10:
                        if len(line)>=LIMIT:raise RuntimeError('machine line bound')
                        line.append(byte);continue
                    record_bytes+=len(line)
                    if record_bytes>LIMIT:raise RuntimeError('machine record byte bound')
                    text=line.decode('utf8');line.clear();line_count+=1
                    if line_count>64:raise RuntimeError('machine record line bound')
                    k,v=text.rstrip('\r').split('=',1)
                    if k in record:raise RuntimeError('duplicate machine key')
                    record[k]=v
                    if k!='progress':continue
                    if v not in ('continue','end'):raise RuntimeError('invalid record terminator')
                    result['records']+=1
                    if first is None:
                        first=time.monotonic()-started;pin_owned_tree(guardian.pid,identities,descendants)
                        result['owned_processes']={str(pid):process_sample(pid) for pid in [guardian.pid]+descendants}
                        if cancel:cancel_at=time.monotonic()+1
                    if not result['record_fields']:result['record_fields']=list(record)
                    speed=record.get('speed','').strip().removesuffix('x')
                    try:speed=float(speed);speed=speed if math.isfinite(speed) and speed>=0 else None
                    except ValueError:speed=None
                    output=record.get('out_time_us',record.get('out_time_ms'))
                    output=int(output) if output and output.isdigit() else None
                    latest={'speed':speed,'output_time_us':output,'ended':v=='end','observed_seconds':time.monotonic()-started}
                    record={};line_count=0;record_bytes=0
            if guardian.poll() is not None:
                # FIFO reader intentionally holds a writer to avoid startup EOF.
                # Drain pending bytes before ending; never wait for impossible EOF.
                ready=selector.select(0)
                if not ready:break
                if all(key.data=='progress' for key,_ in ready):continue
        result.update(exit_code=guardian.wait(),elapsed_seconds=time.monotonic()-started,first_record_seconds=first,latest=latest,live_before_exit=first is not None and first<time.monotonic()-started-0.5)
    finally:
        peer.close();selector.close();os.close(descriptor)
        if guardian is not None:
            cleanup(guardian,control,identities,descendants)
            guardian.stdout.close();guardian.stderr.close()
        else:control.close()
        fifo.unlink(missing_ok=True)
    result['owned_descendants_gone']=all(not Path('/proc/'+str(pid)).exists() for pid in descendants)
    result['fifo_removed']=not fifo.exists()
    if not result['owned_descendants_gone']:raise RuntimeError('owned tree remained')
    return result


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);p.add_argument('--ytdlp',required=True);p.add_argument('--ffmpeg',default='/usr/bin/ffmpeg');p.add_argument('--guardian',required=True,type=Path);a=p.parse_args()
    if sys.platform!='linux' or not a.output.is_absolute() or a.output.parent!=Path('/tmp') or a.output.exists():p.error('Linux new direct /tmp output only')
    a.output.mkdir(mode=0o700);root=a.output/'fixtures';root.mkdir(mode=0o700)
    libc=ctypes.CDLL(None,use_errno=True)
    if libc.prctl(36,1,0,0,0)!=0:raise OSError(ctypes.get_errno(),'owned subreaper setup')
    for filename,source,codec in [('video.mp4','color=c=blue:s=160x90:r=10','libx264'),('audio.m4a','sine=frequency=440:sample_rate=8000','aac')]:
        r=execute('generate-'+filename,[a.ffmpeg,'-nostdin','-hide_banner','-loglevel','error','-f','lavfi','-i',source,'-t','6','-c:v' if filename.endswith('mp4') else '-c:a',codec,str(root/filename)],a.output);assert r['exit_code']==0
    class Handler(http.server.SimpleHTTPRequestHandler):
        def log_message(self,*args):pass
    server=http.server.ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=str(root)));thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start();origin=f'http://127.0.0.1:{server.server_port}'
    outsider=subprocess.Popen([sys.executable,'-c','import time;time.sleep(60)'],stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    results=[]
    try:
        metadata=root/'merge.info.json';metadata.write_text(json.dumps({'id':'merge','title':'synthetic','duration':6,'extractor':'generic','webpage_url':origin+'/original','formats':[{'format_id':'v','url':origin+'/video.mp4','ext':'mp4','vcodec':'h264','acodec':'none'},{'format_id':'a','url':origin+'/audio.m4a','ext':'m4a','vcodec':'none','acodec':'aac'}]}))
        common=[a.ytdlp,'--ignore-config','--no-plugin-dirs','--ffmpeg-location',a.ffmpeg,'--postprocessor-args','ffmpeg:-progress @PROGRESS@ -nostats','--postprocessor-args','Merger+ffmpeg_i1:-re','--postprocessor-args','ExtractAudio+ffmpeg_i1:-re']
        for name,cancel in [('merge',False),('merge-cancel',True)]:
            args=common+['--load-info-json',str(metadata),'-f','v+a','--output',str(root/(name+'.%(ext)s'))]
            r=trial(name,args,a.guardian,a.output,cancel)
            if cancel:assert r['cancelled'] and r['exit_code']!=0 and not r['latest']['ended'],r
            else:assert r['exit_code']==0 and r['live_before_exit'] and r['latest']['ended'],r
            results.append(r)
        args=common+['--extract-audio','--audio-format','mp3','--output',str(root/'audio-only.%(ext)s'),'--',origin+'/audio.m4a']
        r=trial('audio-only',args,a.guardian,a.output);assert r['exit_code']==0 and r['live_before_exit'] and r['latest']['ended'] and 'frame' not in r['record_fields'],r;results.append(r)
        bad_args=common+['--load-info-json',str(metadata),'-f','v+a','--output',str(root/'invalid-sink.%(ext)s')]
        bad_args=[x.replace('@PROGRESS@',str(a.output/'nonexistent/progress.fifo')) for x in bad_args]
        r=trial('invalid-sink',bad_args,a.guardian,a.output);assert r['exit_code']!=0 and r['records']==0,r;results.append(r)
        assert outsider.poll() is None,'unrelated owned sentinel died'
    finally:
        server.shutdown();server.server_close();thread.join(timeout=3)
        outsider.terminate();outsider.wait(timeout=3)
    result={'scope':'Linux FIFO only, production unmodified, Windows/macOS not verified','outsider_preserved':True,'results':results};(a.output/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))

if __name__=='__main__':main()
