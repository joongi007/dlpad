#!/usr/bin/env python3
"""Accessible candidate acceptance on a newly owned Xvfb only. No build or desktop fallback."""
import argparse
import ctypes as C
import ctypes.util
import hashlib
import json
import os
from pathlib import Path
import signal
import select
import sqlite3
import subprocess
import tempfile
import threading
import time

from native_probe import probe_display, screenshot
from subtitle_ui_probe import key_input, job
from x11_diagnostics import click_at, copy_report, Selection
from background_ui_probe import OwnedDisplay, process_identity, reap

TOOL = r'''#!/usr/bin/python3
import json,os,pathlib,sys,time
root=pathlib.Path(__file__).parent
args=sys.argv[1:]
if '--version' in args:print('2026.10.02');sys.exit(0)
if '--dump-single-json' in args:
 with (root/'extractions').open('a') as f:f.write(args[-1]+'\n')
 print(json.dumps({'id':'synthetic','duration':120}));sys.exit(0)
cfg=json.loads((root/'control.json').read_text())
p=pathlib.Path(args[args.index('--paths')+1])/args[args.index('--output')+1].replace('%(ext)s','mp4')
partial=p.with_suffix('.part')
if partial.exists():assert partial.read_bytes()==b'p'*65536
else:partial.write_bytes(b'p'*65536)
st=pathlib.Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
(root/'worker.json').write_text(json.dumps({'pid':os.getpid(),'start_ticks':int(st[19]),'token':cfg['token'],'partial':str(partial)}))
for count in range(min(600,int(cfg['seconds']*5)+1)):
 (root/'heartbeat').write_text(str(count))
 print('DLPAD_PROGRESS:'+json.dumps({'speed':1024,'eta':max(0,cfg['seconds']-count/5)}),flush=True)
 time.sleep(.2)
if cfg['mode']=='failure':
 print('ERROR: https://media.invalid/watch?signature=fixture Authorization: Bearer fixture-secret',file=sys.stderr,flush=True);sys.exit(7)
p.write_bytes(b'synthetic candidate artifact')
print('DLPAD_ARTIFACT:'+json.dumps(str(p)),flush=True)
'''
DEFAULT = {'url':[250,100], 'start':[55,264], 'resume':[123,264],
           'pause':[112,300], 'cancel':[55,300], 'diagnostics':[70,295],
           'copy':[85,350], 'stop_exit':[65,146]}


def file_hash(path):
    digest=hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda:stream.read(65536),b''):digest.update(chunk)
    return digest.hexdigest()


def wait(predicate, child=None, timeout=10):
    end=time.monotonic()+timeout
    while time.monotonic()<end:
        value=predicate()
        if value:return value
        if child is not None and child.poll() is not None:raise RuntimeError('owned GUI exited before observation')
        time.sleep(.05)
    raise RuntimeError('owned fixture observation timed out')


def load_layout(path):
    if path is None:return DEFAULT.copy()
    if path.stat().st_size>4096:raise ValueError('layout exceeds 4096 bytes')
    value=json.loads(path.read_text())
    if set(value)!=set(DEFAULT):raise ValueError('layout must contain exactly '+','.join(DEFAULT))
    for point in value.values():
        if not isinstance(point,list) or len(point)!=2 or any(type(x)is not int for x in point) or not(0<=point[0]<960 and 0<=point[1]<640):raise ValueError('layout point outside owned candidate window')
    return value


def state(data, wanted):
    row=job(data)
    return row if row and row['job']['state']==wanted else None


def clipboard_preview(display_name):
    """Read only the new owned Xvfb CLIPBOARD, with a 256 KiB cap."""
    x=C.CDLL(ctypes.util.find_library('X11'))
    x.XOpenDisplay.argtypes=[C.c_char_p];x.XOpenDisplay.restype=C.c_void_p
    x.XCloseDisplay.argtypes=[C.c_void_p];x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XDefaultRootWindow.restype=C.c_ulong
    x.XCreateSimpleWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_int,C.c_uint,C.c_uint,C.c_uint,C.c_ulong,C.c_ulong];x.XCreateSimpleWindow.restype=C.c_ulong
    x.XDestroyWindow.argtypes=[C.c_void_p,C.c_ulong]
    x.XInternAtom.argtypes=[C.c_void_p,C.c_char_p,C.c_int];x.XInternAtom.restype=C.c_ulong
    x.XConvertSelection.argtypes=[C.c_void_p,C.c_ulong,C.c_ulong,C.c_ulong,C.c_ulong,C.c_ulong]
    x.XPending.argtypes=[C.c_void_p];x.XNextEvent.argtypes=[C.c_void_p,C.c_void_p];x.XFlush.argtypes=[C.c_void_p]
    x.XGetWindowProperty.argtypes=[C.c_void_p,C.c_ulong,C.c_ulong,C.c_long,C.c_long,C.c_int,C.c_ulong,C.POINTER(C.c_ulong),C.POINTER(C.c_int),C.POINTER(C.c_ulong),C.POINTER(C.c_ulong),C.POINTER(C.POINTER(C.c_ubyte))]
    x.XFree.argtypes=[C.c_void_p]
    display=x.XOpenDisplay(display_name.encode())
    if not display:raise RuntimeError('owned clipboard display unavailable')
    window=0
    try:
        window=x.XCreateSimpleWindow(display,x.XDefaultRootWindow(display),0,0,1,1,0,0,0)
        atom=lambda name:x.XInternAtom(display,name,0)
        prop=atom(b'DLPAD_ACCESSIBLE_PROBE');x.XConvertSelection(display,atom(b'CLIPBOARD'),atom(b'UTF8_STRING'),prop,window,0);x.XFlush(display)
        deadline=time.monotonic()+3;event=(C.c_long*24)()
        while time.monotonic()<deadline:
            if not x.XPending(display):time.sleep(.01);continue
            x.XNextEvent(display,C.byref(event));notification=C.cast(C.byref(event),C.POINTER(Selection)).contents
            if notification.type!=31:continue
            if not notification.property:raise RuntimeError('owned clipboard conversion refused')
            actual,count,remaining=C.c_ulong(),C.c_ulong(),C.c_ulong();bits=C.c_int();data=C.POINTER(C.c_ubyte)()
            status=x.XGetWindowProperty(display,window,prop,0,65536,0,0,C.byref(actual),C.byref(bits),C.byref(count),C.byref(remaining),C.byref(data))
            try:
                if status or bits.value!=8 or remaining.value or count.value>262144:raise RuntimeError('owned clipboard result exceeded cap')
                text=C.string_at(data,count.value).decode('utf-8')
            finally:
                if data:x.XFree(data)
            if not text.startswith('# dlpad diagnostic report') or 'media.invalid' in text or 'signature=fixture' in text or 'fixture-secret' in text:raise RuntimeError('owned diagnostic preview was unredacted or invalid')
            return text
        raise RuntimeError('owned clipboard preview timeout')
    finally:
        if window:x.XDestroyWindow(display,window)
        x.XCloseDisplay(display)


def snapshot_database(data, destination):
    source=data/'jobs.sqlite'
    if not source.is_file():return False
    deadline=time.monotonic()+3
    with sqlite3.connect(source.resolve().as_uri()+'?mode=ro',uri=True,timeout=.1) as original:
        if original.execute('PRAGMA page_count').fetchone()[0]*original.execute('PRAGMA page_size').fetchone()[0]>2097152:raise RuntimeError('owned SQLite evidence exceeds 2 MiB')
        def bounded(status,left,total):
            if time.monotonic()>deadline or total>512:raise RuntimeError('owned SQLite backup exceeded bounds')
        with sqlite3.connect(destination) as target:original.backup(target,pages=32,progress=bounded,sleep=.05)
    return True


def bounded_log(stream,path):
    with path.open('xb') as output:
        retained=0
        while chunk:=stream.read(4096):
            keep=chunk[:max(0,65536-retained)]
            output.write(keep);retained+=len(keep)
    stream.close()


def pin_worker(root, token):
    info=json.loads((root/'worker.json').read_text())
    if info['token']!=token or process_identity(info['pid'])!=(info['pid'],info['start_ticks']):raise RuntimeError('worker identity mismatch')
    command=Path(f"/proc/{info['pid']}/cmdline").read_bytes().split(b'\0')
    if os.fsencode(root/'synthetic-tool') not in command:raise RuntimeError('worker executable argv ownership mismatch')
    descriptor=os.pidfd_open(info['pid'])
    if process_identity(info['pid'])!=(info['pid'],info['start_ticks']):os.close(descriptor);raise RuntimeError('worker changed while pinning')
    return info,descriptor


def run_case(args, case, layout):
    prefix=args.output.parent/(args.output.stem+'-'+case)
    result={'case':case,'display':'owned Xvfb','actual_media':False,'desktop_fallback':False,'passed':False}
    result['binary_sha256']=file_hash(args.binary)
    result['source_sha256']={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path('crates/app/src/accessible.rs'),Path('crates/app/src/accessible_main.rs'),Path('crates/app/src/accessible_cpu_main.rs')] if p.is_file()}
    with tempfile.TemporaryDirectory(prefix='accessible-owned-') as tmp:
        root=Path(tmp);data=root/'data';data.mkdir(mode=0o700);media=root/'media';media.mkdir(mode=0o700)
        tool=root/'synthetic-tool';tool.write_text(TOOL);tool.chmod(0o700)
        token=os.urandom(16).hex();result['fixture_sha256']=hashlib.sha256(TOOL.encode()).hexdigest()
        def control(mode,seconds):
            (root/'control.json').write_text(json.dumps({'mode':mode,'seconds':min(seconds,120),'token':token}))
        control('failure' if case=='failure' else 'success',120 if case in ('pause','close') else 1.2)
        child=None;display_owner=None;workers=[];log_thread=None;bus=None;bus_log_thread=None
        with probe_display(prefix,False) as (display,kind):
            if kind!='Xvfb isolated':raise RuntimeError('refusing non-owned display')
            env=dict(os.environ)
            for key in ['WAYLAND_DISPLAY','DLPAD_TEST_DOCUMENTS','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS']:env.pop(key,None)
            for key,name in [('XDG_CONFIG_HOME','xdg-config'),('XDG_CACHE_HOME','xdg-cache'),('XDG_DATA_HOME','xdg-data'),('XDG_RUNTIME_DIR','runtime')]:
                location=root/name;location.mkdir(mode=0o700);env[key]=str(location)
            env['GSETTINGS_BACKEND']='memory'
            result.update(bus='private D-Bus session',inherited_accessibility_bus_removed=True,ui_completion_stabilization_seconds=.6)
            env.update(DISPLAY=display,DLPAD_TEST_DATA=str(data),DLPAD_LIBRARY_ROOT=str(media),DLPAD_YTDLP=str(tool),MESA_SHADER_CACHE_DIR=str(root/'shader-cache'),XDG_CACHE_HOME=str(root/'xdg-cache'))
            if args.font:env['DLPAD_TEST_FONT']=str(args.font)
            try:
                bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
                result['bus_identity']=list(process_identity(bus.pid))
                bus_log_thread=threading.Thread(target=bounded_log,args=(bus.stderr,prefix.with_name(prefix.name+'-bus.log')),daemon=True);bus_log_thread.start()
                if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('owned private bus startup timeout')
                address=bus.stdout.readline(4097).decode().strip()
                if len(address)>4096 or not address.startswith('unix:'):raise RuntimeError('owned private bus address invalid')
                env['DBUS_SESSION_BUS_ADDRESS']=address
                child=subprocess.Popen([str(args.binary)],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
                log_thread=threading.Thread(target=bounded_log,args=(child.stdout,prefix.with_suffix('.log')),daemon=True);log_thread.start()
                result['app_identity']=list(process_identity(child.pid));time.sleep(2)
                display_owner=OwnedDisplay(display);window=display_owner.window()
                geometry=screenshot(prefix.with_name(prefix.name+'-initial.png'),'dlpad',display)
                if case=='inspect':result['inspection_only']=True
                else:
                    key_input(display,geometry,layout['url'],'https://example.invalid/synthetic-source')
                    click_at(display,geometry,*layout['start'])
                    wait(lambda:(root/'worker.json').is_file(),child)
                    info,pidfd=pin_worker(root,token);workers.append((info,pidfd));result['worker_identity']=info
                    if case=='success':
                        final=wait(lambda:state(data,'Downloaded'),child)
                        if Path(final['job']['artifacts'][0]['path']).read_bytes()!=b'synthetic candidate artifact':raise RuntimeError('artifact content mismatch')
                        result['downloaded']=final
                    elif case=='failure':
                        result['failed']=wait(lambda:state(data,'Failed'),child);time.sleep(.75)
                        click_at(display,geometry,*layout['diagnostics']);time.sleep(.25)
                        copy_geometry={'x':geometry['x']+layout['copy'][0]-80,'y':geometry['y']+layout['copy'][1]-200}
                        result.update(copy_report(display,copy_geometry));preview=clipboard_preview(display);preview_path=prefix.with_name(prefix.name+'-report.md');preview_path.write_text(preview);result['report_preview_sha256']=hashlib.sha256(preview.encode()).hexdigest();result['report_preview_file']=str(preview_path)
                    elif case=='pause':
                        before=job(data);time.sleep(.5);screenshot(prefix.with_name(prefix.name+'-pause-ready.png'),'dlpad',display);click_at(display,geometry,*layout['pause'])
                        paused=wait(lambda:state(data,'Paused'),child)
                        time.sleep(.6);screenshot(prefix.with_name(prefix.name+'-paused.png'),'dlpad',display)
                        wait(lambda:process_identity(info['pid'])!=(info['pid'],info['start_ticks']),child)
                        if Path(info['partial']).read_bytes()!=b'p'*65536:raise RuntimeError('pause did not preserve partial')
                        control('success',1.2);time.sleep(.75);click_at(display,geometry,*layout['resume'])
                        final=wait(lambda:state(data,'Downloaded'),child)
                        if final['job']['id']!=before['job']['id'] or len(final['job']['attempts'])!=2 or final['job']['attempts'][0]['id']==final['job']['attempts'][1]['id']:raise RuntimeError('resume did not keep job with new attempt')
                        if Path(info['partial']).read_bytes()!=b'p'*65536:raise RuntimeError('resume changed retained partial')
                        if (root/'extractions').read_text().splitlines()!=['https://example.invalid/synthetic-source']*2:raise RuntimeError('resume did not re-extract original URL')
                        result.update(paused=paused,resumed=final)
                    elif case=='close':
                        display_owner.delete_window(window);time.sleep(.3)
                        screenshot(prefix.with_name(prefix.name+'-prompt.png'),'dlpad',display)
                        click_at(display,geometry,*layout['stop_exit'])
                        wait(lambda:child.poll() is not None,timeout=10)
                        if child.returncode!=0:raise RuntimeError('CancelWait native exit was not zero')
                        wait(lambda:process_identity(info['pid'])!=(info['pid'],info['start_ticks']))
                        result['cancelled']=wait(lambda:state(data,'Cancelled'))
                        result.update(cancelwait_exit=child.returncode,worker_stopped=True)
                    if case in ('success','pause'):
                        artifact=Path((result.get('downloaded') or result['resumed'])['job']['artifacts'][0]['path']);content=artifact.read_bytes()
                        if len(content)>65536:raise RuntimeError('synthetic artifact exceeds fixture cap')
                        artifact_evidence=prefix.with_name(prefix.name+'-artifact.bin');artifact_evidence.write_bytes(content);result['artifact_sha256']=hashlib.sha256(content).hexdigest()
                    if child.poll() is None:
                        if case in ('success','pause'):time.sleep(.6)
                        screenshot(prefix.with_name(prefix.name+'-final.png'),'dlpad',display)
                    result['binary_sha256_after']=file_hash(args.binary)
                    if result['binary_sha256_after']!=result['binary_sha256']:raise RuntimeError('candidate binary changed during GUI trial')
                    result['passed']=True
            except Exception as error:
                result['error']=str(error)
                try:result['last_journal']=job(data)
                except Exception:pass
                if child is not None and child.poll() is None:
                    try:screenshot(prefix.with_name(prefix.name+'-error.png'),'dlpad',display)
                    except Exception:pass
            finally:
                if child is not None:reap(child)
                for info,pidfd in workers:
                    identity=(info['pid'],info['start_ticks'])
                    try:wait(lambda:process_identity(info['pid'])!=identity,timeout=5)
                    except RuntimeError:
                        signal.pidfd_send_signal(pidfd,signal.SIGKILL)
                        result['cleanup_fallback_used']=True;result['passed']=False
                    result['owned_worker_stopped_after_teardown']=process_identity(info['pid'])!=identity
                    os.close(pidfd)
                if log_thread:log_thread.join(3)
                if display_owner:display_owner.close()
                if bus is not None:
                    reap(bus)
                    if bus.stdout:bus.stdout.close()
                    if bus_log_thread:bus_log_thread.join(3)
                    result['owned_bus_reaped']=bus.poll() is not None and (bus_log_thread is None or not bus_log_thread.is_alive())
                    if not result['owned_bus_reaped']:result['passed']=False;result['bus_cleanup_error']='owned bus/drainer did not finish'
                try:result['sqlite_evidence']=str(prefix.with_suffix('.sqlite')) if snapshot_database(data,prefix.with_suffix('.sqlite')) else None
                except Exception as error:result['evidence_backup_error']=str(error);result['passed']=False
    prefix.with_suffix('.json').write_text(json.dumps(result,ensure_ascii=False,indent=2))
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--case',choices=['inspect','success','failure','pause','close','all'],default='all')
    parser.add_argument('--font',type=Path);parser.add_argument('--layout',type=Path)
    args=parser.parse_args();args.binary=args.binary.resolve()
    if not args.binary.is_file():parser.error('candidate binary must already exist; probe never builds')
    args.output.parent.mkdir(parents=True,exist_ok=True)
    if list(args.output.parent.glob(args.output.stem+'*')):parser.error('output prefix must be fresh')
    layout=load_layout(args.layout)
    cases=['success','failure','pause','close'] if args.case=='all' else [args.case]
    results=[run_case(args,case,layout) for case in cases]
    args.output.write_text(json.dumps({'results':results,'passed':all(r['passed'] for r in results),'layout':layout},ensure_ascii=False,indent=2))
    return 0 if all(r['passed'] or r.get('inspection_only') for r in results) else 1

if __name__=='__main__':raise SystemExit(main())
