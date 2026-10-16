#!/usr/bin/env python3
"""Close/hidden/restore probes confined to owned Xvfb, native GUI and synthetic workers."""
import argparse
import ctypes as C
import ctypes.util
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time

from native_probe import probe_display, screenshot
from managed_tools_probe import file_sha256, point
from subtitle_ui_probe import key_input, job
from x11_diagnostics import click_at, scroll_to_end

TOOL='''#!/usr/bin/python3
import sys,json,pathlib,time,os,shlex
args=sys.argv[1:]
if '--version' in args:print('2026.10.02');sys.exit(0)
if '--dump-single-json' in args:print(json.dumps({'id':'synthetic','duration':30}));sys.exit(0)
root=pathlib.Path(__file__).parent
(root/'worker.pid').write_text(str(os.getpid()))
feed=None
if '--postprocessor-args' in args:
 tokens=shlex.split(args[args.index('--postprocessor-args')+1].split(':',1)[1])
 feed=open(tokens[tokens.index('-progress')+1],'w')
 (root/'feed-opened').write_text('synthetic progress FIFO')
for counter in range(150):
 if feed:feed.write('out_time_us=1000000\\nspeed=2.5x\\nprogress=continue\\n');feed.flush()
 temporary=root/'heartbeat.tmp';temporary.write_text(str(counter));temporary.replace(root/'heartbeat')
 print('DLPAD_PROGRESS:'+json.dumps({'speed':1,'eta':30-counter/5}),flush=True)
 time.sleep(.2)
path=pathlib.Path(args[args.index('--paths')+1])/args[args.index('--output')+1].replace('%(ext)s','mp4')
path.write_bytes(b'synthetic-media')
print('DLPAD_ARTIFACT:'+json.dumps(str(path)),flush=True)
'''


class Attributes(C.Structure):
    _fields_=[('x',C.c_int),('y',C.c_int),('width',C.c_int),('height',C.c_int),('border_width',C.c_int),('depth',C.c_int),('visual',C.c_void_p),('root',C.c_ulong),('window_class',C.c_int),('bit_gravity',C.c_int),('win_gravity',C.c_int),('backing_store',C.c_int),('backing_planes',C.c_ulong),('backing_pixel',C.c_ulong),('save_under',C.c_int),('colormap',C.c_ulong),('map_installed',C.c_int),('map_state',C.c_int),('all_event_masks',C.c_long),('your_event_mask',C.c_long),('do_not_propagate_mask',C.c_long),('override_redirect',C.c_int),('screen',C.c_void_p)]
class ClientData(C.Union):
    _fields_=[('bytes',C.c_char*20),('shorts',C.c_short*10),('longs',C.c_long*5)]
class ClientMessage(C.Structure):
    _fields_=[('type',C.c_int),('serial',C.c_ulong),('send_event',C.c_int),('display',C.c_void_p),('window',C.c_ulong),('message_type',C.c_ulong),('format',C.c_int),('data',ClientData)]
class Event(C.Union):
    _fields_=[('client',ClientMessage),('padding',C.c_long*24)]


class OwnedDisplay:
    def __init__(self,name):
        self.x=C.CDLL(ctypes.util.find_library('X11'));x=self.x
        x.XOpenDisplay.argtypes=[C.c_char_p];x.XOpenDisplay.restype=C.c_void_p;x.XCloseDisplay.argtypes=[C.c_void_p]
        x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XDefaultRootWindow.restype=C.c_ulong
        x.XQueryTree.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(C.c_ulong),C.POINTER(C.c_ulong),C.POINTER(C.POINTER(C.c_ulong)),C.POINTER(C.c_uint)]
        x.XFetchName.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(C.c_char_p)];x.XFree.argtypes=[C.c_void_p]
        x.XGetWindowAttributes.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(Attributes)]
        x.XInternAtom.argtypes=[C.c_void_p,C.c_char_p,C.c_int];x.XInternAtom.restype=C.c_ulong
        x.XSendEvent.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_long,C.c_void_p];x.XFlush.argtypes=[C.c_void_p]
        self.display=x.XOpenDisplay(name.encode())
        if not self.display:raise RuntimeError('owned isolated Xvfb unavailable')
    def close(self):self.x.XCloseDisplay(self.display)
    def window(self):
        def find(window,depth=0):
            name=C.c_char_p();self.x.XFetchName(self.display,window,C.byref(name))
            try:
                if name.value and b'dlpad' in name.value:return window
            finally:
                if name:self.x.XFree(name)
            if depth>=5:return None
            root,parent,children,count=C.c_ulong(),C.c_ulong(),C.POINTER(C.c_ulong)(),C.c_uint()
            self.x.XQueryTree(self.display,window,C.byref(root),C.byref(parent),C.byref(children),C.byref(count))
            try:
                for i in range(count.value):
                    found=find(children[i],depth+1)
                    if found:return found
            finally:
                if children:self.x.XFree(children)
            return None
        result=find(self.x.XDefaultRootWindow(self.display))
        if result is None:raise RuntimeError('owned dlpad window missing')
        return result
    def map_state(self,window):
        attributes=Attributes()
        if not self.x.XGetWindowAttributes(self.display,window,C.byref(attributes)):raise RuntimeError('owned window attributes unavailable')
        return attributes.map_state
    def delete_window(self,window):
        event=Event();message=event.client;message.type=33;message.send_event=1;message.display=self.display;message.window=window
        message.message_type=self.x.XInternAtom(self.display,b'WM_PROTOCOLS',0);message.format=32
        message.data.longs[0]=self.x.XInternAtom(self.display,b'WM_DELETE_WINDOW',0);message.data.longs[1]=0
        if not self.x.XSendEvent(self.display,window,0,0,C.byref(event)):raise RuntimeError('owned close event delivery failed')
        self.x.XFlush(self.display)


def process_identity(pid):
    try:
        fields=Path(f'/proc/{pid}/stat').read_text().rsplit(') ',1)[1].split()
        if fields[0] in ('Z','X'):return None
        return (pid,int(fields[19]))
    except (OSError,ValueError,IndexError):return None


def wait(predicate,timeout=5):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        value=predicate()
        if value:return value
        time.sleep(.05)
    raise RuntimeError('owned fixture observation timed out')


def heartbeat(root):
    try:return int((root/'heartbeat').read_text())
    except (OSError,ValueError):return -1


def rss_kib(pid):
    for line in Path(f'/proc/{pid}/status').read_text().splitlines():
        if line.startswith('VmRSS:'):return int(line.split()[1])
    return None


def reap(child):
    if child.poll() is None:
        os.killpg(child.pid,signal.SIGTERM)
        try:child.wait(timeout=3)
        except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);child.wait()


def run(args):
    started=time.monotonic()
    with args.binary.open('rb') as binary:sha=file_sha256(binary)
    result={'mode':args.mode,'binary_sha256':sha,'display':'Xvfb isolated without WM',
            'native_gui_verified':False,'actual_media_used':False,'desktop_fallback_used':False}
    with tempfile.TemporaryDirectory(prefix='dlpad-background-') as temporary:
        root=Path(temporary);data=root/'app-data';data.mkdir(mode=0o700);media=root/'synthetic-media';media.mkdir()
        foreign_endpoint=data/'activation.sock'
        if args.mode=='unavailable':foreign_endpoint.write_bytes(b'owned synthetic foreign endpoint')
        tool=root/'yt-dlp-synthetic';tool.write_text(TOOL);tool.chmod(0o700)
        with probe_display(args.output,False) as (display,_):
            env=dict(os.environ);env.pop('WAYLAND_DISPLAY',None);env.pop('DLPAD_TEST_DOCUMENTS',None)
            env.update(DISPLAY=display,DLPAD_TEST_FONT=str(args.font),DLPAD_TEST_DATA=str(data),DLPAD_LIBRARY_ROOT=str(media),DLPAD_YTDLP=str(tool),MESA_SHADER_CACHE_DIR=str(root/'shader-cache'))
            with args.output.with_suffix('.log').open('xb') as logs:
                command=[str(args.binary),'--fixture','--downloads']
                child=subprocess.Popen(command,env=env,stdout=logs,stderr=subprocess.STDOUT,start_new_session=True)
                x=None;worker=None;secondary=None
                try:
                    time.sleep(2);x=OwnedDisplay(display);window=x.window()
                    app_identity=process_identity(child.pid)
                    result['rss_visible_kib']=rss_kib(child.pid)
                    result['ipc_endpoint_present_at_start']=(data/'activation.sock').is_socket() if (data/'activation.sock').exists() else False
                    geometry=screenshot(args.output.with_name(args.output.stem+'-before.png'),'dlpad',display)
                    key_input(display,geometry,args.url_point,'https://example.org/synthetic')
                    scroll_to_end(display,geometry);click_at(display,geometry,*args.start_point)
                    wait(lambda:(root/'worker.pid').is_file())
                    worker=process_identity(int((root/'worker.pid').read_text()))
                    if worker is None:raise RuntimeError('owned synthetic worker not live')
                    wait(lambda:heartbeat(root)>=1)
                    baseline=job(data)
                    if baseline is None:raise RuntimeError('running journal missing')
                    if args.mode=='processing':
                        wait(lambda:(root/'feed-opened').is_file())
                        time.sleep(.5)
                        screenshot(args.output.with_name(args.output.stem+'-processing.png'),'dlpad',display)
                        wait(lambda:job(data)['job']['state']=='Downloaded',35)
                        time.sleep(.5)
                        screenshot(args.output.with_name(args.output.stem+'-finished.png'),'dlpad',display)
                        result.update(synthetic_progress_fifo_opened=True,completed_journal=job(data),processing_text_requires_screenshot_review=True)
                    else:x.delete_window(window)
                    time.sleep(.3)
                    screenshot(args.output.with_name(args.output.stem+'-close.png'),'dlpad',display)
                    if args.mode=='processing':pass
                    elif args.mode=='inspect':result['inspection_only']=True
                    elif args.mode=='return':
                        count=heartbeat(root);click_at(display,geometry,*args.return_point)
                        wait(lambda:heartbeat(root)>count)
                        if x.map_state(window)!=2 or process_identity(child.pid)!=app_identity or process_identity(worker[0])!=worker:raise RuntimeError('Return changed owned app/worker or visibility')
                        result.update(return_keeps_visible_worker=True,return_dismissal_requires_screenshot_review=True,native_gui_verified=True)
                        screenshot(args.output.with_suffix('.png'),'dlpad',display)
                    elif args.mode=='stop-exit':
                        click_at(display,geometry,*args.stop_point)
                        wait(lambda:child.poll() is not None,args.timeout)
                        wait(lambda:process_identity(worker[0])!=worker)
                        if child.returncode!=0:raise RuntimeError(f'StopExit native exit={child.returncode}')
                        final=job(data)
                        if final is None or final['job']['state']!='Cancelled':raise RuntimeError('StopExit exited without Cancelled journal')
                        result.update(stop_exit_child_reaped=True,owned_worker_stopped=True,cancelled_journal=final,native_gui_verified=True)
                    elif args.mode=='unavailable':
                        count=heartbeat(root);click_at(display,geometry,*args.background_point);time.sleep(.5)
                        if x.map_state(window)!=2 or process_identity(child.pid)!=app_identity or process_identity(worker[0])!=worker or heartbeat(root)<=count or foreign_endpoint.read_bytes()!=b'owned synthetic foreign endpoint':raise RuntimeError('unavailable restore did not preserve window/worker/foreign endpoint')
                        result.update(unavailable_restore_keeps_visible=True,foreign_endpoint_preserved=True,error_text_requires_screenshot_review=True)
                        screenshot(args.output.with_suffix('.png'),'dlpad',display)
                    else:
                        click_at(display,geometry,*args.background_point)
                        wait(lambda:x.map_state(window)==0)
                        count=heartbeat(root);time.sleep(1)
                        if heartbeat(root)<=count or process_identity(child.pid)!=app_identity or process_identity(worker[0])!=worker:raise RuntimeError('hidden app did not preserve same live worker')
                        result['rss_hidden_kib']=rss_kib(child.pid)
                        hidden=job(data)
                        if hidden is None or hidden['job']['id']!=baseline['job']['id'] or len(hidden['job']['attempts'])!=len(baseline['job']['attempts']):raise RuntimeError('hidden app duplicated journal attempt')
                        if args.complete_hidden:
                            wait(lambda:job(data)['job']['state']=='Downloaded',35)
                            if x.map_state(window)!=0 or process_identity(child.pid)!=app_identity:raise RuntimeError('completion automatically showed or replaced hidden app')
                            wait(lambda:process_identity(worker[0])!=worker)
                            result['completed_without_automatic_show']=True
                        with args.output.with_name(args.output.stem+'-restore.log').open('xb') as restore_logs:
                            secondary=subprocess.Popen(command,env=env,stdout=restore_logs,stderr=subprocess.STDOUT,start_new_session=True)
                            secondary.wait(timeout=2)
                            if secondary.returncode!=0:raise RuntimeError(f'explicit restore launcher exit={secondary.returncode}')
                        wait(lambda:x.map_state(window)==2)
                        result['rss_restored_kib']=rss_kib(child.pid)
                        restored=job(data)
                        if process_identity(child.pid)!=app_identity or (not args.complete_hidden and process_identity(worker[0])!=worker) or restored['job']['id']!=baseline['job']['id'] or len(restored['job']['attempts'])!=len(baseline['job']['attempts']):raise RuntimeError('restore replaced owned app/worker or duplicated attempt')
                        result.update(hidden_map_state=0,restored_map_state=2,same_app_identity=list(app_identity),same_worker_identity=list(worker),hidden_heartbeat_advanced=True,restore_launcher_exit=secondary.returncode,same_job_attempts=True,native_gui_verified=True)
                        screenshot(args.output.with_suffix('.png'),'dlpad',display)
                except Exception as error:
                    result['error']=str(error)
                    try:
                        if x.map_state(window)==2:screenshot(args.output.with_name(args.output.stem+'-error.png'),'dlpad',display)
                    except Exception:pass
                finally:
                    if secondary is not None:reap(secondary)
                    reap(child)
                    if worker is not None:
                        try:wait(lambda:process_identity(worker[0])!=worker)
                        except RuntimeError:result['error']=result.get('error','owned worker did not stop after app teardown')
                        result['owned_worker_stopped_after_teardown']=process_identity(worker[0])!=worker
                    if x is not None:x.close()
                    result['owned_gui_reaped']=child.poll() is not None
    result['temporary_fixture_removed']=True;result['elapsed_seconds']=round(time.monotonic()-started,3)
    args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
    return int('error'in result)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary',type=Path);p.add_argument('output',type=Path)
    p.add_argument('--mode',choices=['inspect','return','stop-exit','background','unavailable','processing'],default='inspect')
    p.add_argument('--font',type=Path,default=os.environ.get('DLPAD_TEST_FONT'));p.add_argument('--timeout',type=float,default=10)
    for name,default in [('url',(250,160)),('start',(80,435)),('return',(180,604)),('stop',(80,604)),('background',(295,604))]:p.add_argument('--'+name+'-point',type=point,default=default)
    p.add_argument('--complete-hidden',action='store_true',help='wait for synthetic completion while hidden before explicit restore')
    p.add_argument('--validate-only',action='store_true');args=p.parse_args()
    if not 1<=args.timeout<=30:p.error('timeout must be1..30 seconds')
    if args.output.suffix!='.json':p.error('output must end in.json')
    if args.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux X11 only')
    args.binary=args.binary.resolve();args.output=args.output.resolve()
    if not args.binary.is_file() or not os.access(args.binary,os.X_OK):p.error('binary must be executable')
    if args.font is None or not args.font.is_file():p.error('explicit fixture font required')
    args.font=args.font.resolve();args.output.parent.mkdir(parents=True,exist_ok=True)
    if list(args.output.parent.glob(args.output.stem+'.*')) or list(args.output.parent.glob(args.output.stem+'-*')):p.error('evidence prefix exists')
    try:return run(args)
    except (Exception,KeyboardInterrupt) as error:
        report={'mode':args.mode,'native_gui_verified':False,'error':str(error) or type(error).__name__,'desktop_fallback_used':False}
        args.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report));return 1


if __name__=='__main__':raise SystemExit(main())
