#!/usr/bin/env python3
"""Linux native subtitle controls on owned Xvfb and synthetic tools only; no build."""
import argparse
import ctypes as C
import ctypes.util
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import tempfile
import time

from native_probe import probe_display, screenshot
from x11_diagnostics import click_at, scroll_to_end
from managed_tools_probe import file_sha256

CASES = ('none', 'ko', 'en', 'both', 'conflict', 'freeze')
TOOL = '''#!/usr/bin/python3
import sys,json,pathlib,time
args=sys.argv[1:]
root=pathlib.Path(__file__).parent
if '--version' in args: print('2026.10.01');sys.exit(0)
if '--dump-single-json' in args: print(json.dumps({'id':'synthetic','duration':10}));sys.exit(0)
(root/'execution.json').write_text(json.dumps(args))
if (root/'slow').exists():
 (root/'running').write_text('owned fixture running')
 time.sleep(30)
path=pathlib.Path(args[args.index('--paths')+1])/args[args.index('--output')+1].replace('%(ext)s','mp4')
path.write_bytes(b'synthetic-media')
if '--write-subs' in args:
 requested={}
 for lang in args[args.index('--sub-langs')+1].split(','):
  sidecar=path.with_suffix('.'+lang+'.vtt');sidecar.write_text('WEBVTT synthetic '+lang)
  requested[lang]={'filepath':str(sidecar),'ext':'vtt'}
 print('DLPAD_SUBTITLES:'+json.dumps({'id':'synthetic','requested_subtitles':requested}),flush=True)
print('DLPAD_ARTIFACT:'+json.dumps(str(path)),flush=True)
'''


def key_input(display_name, geometry, point, text):
    if not text.isascii() or any(ord(char)<32 for char in text):
        raise ValueError('fixture keyboard accepts printable ASCII only')
    click_at(display_name, geometry, *point)
    x=C.CDLL(ctypes.util.find_library('X11'));t=C.CDLL(ctypes.util.find_library('Xtst'))
    x.XOpenDisplay.argtypes=[C.c_char_p];x.XOpenDisplay.restype=C.c_void_p
    x.XCloseDisplay.argtypes=[C.c_void_p];x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XDefaultRootWindow.restype=C.c_ulong
    x.XQueryTree.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(C.c_ulong),C.POINTER(C.c_ulong),C.POINTER(C.POINTER(C.c_ulong)),C.POINTER(C.c_uint)]
    x.XFetchName.argtypes=[C.c_void_p,C.c_ulong,C.POINTER(C.c_char_p)];x.XFree.argtypes=[C.c_void_p]
    x.XSetInputFocus.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_ulong]
    x.XKeysymToKeycode.argtypes=[C.c_void_p,C.c_ulong];x.XKeysymToKeycode.restype=C.c_ubyte
    x.XkbKeycodeToKeysym.argtypes=[C.c_void_p,C.c_ubyte,C.c_int,C.c_int];x.XkbKeycodeToKeysym.restype=C.c_ulong
    x.XFlush.argtypes=[C.c_void_p];t.XTestFakeKeyEvent.argtypes=[C.c_void_p,C.c_uint,C.c_int,C.c_ulong]
    display=x.XOpenDisplay(display_name.encode())
    if not display:raise RuntimeError('owned Xvfb keyboard unavailable')
    try:
        root,parent,children,count=C.c_ulong(),C.c_ulong(),C.POINTER(C.c_ulong)(),C.c_uint()
        x.XQueryTree(display,x.XDefaultRootWindow(display),C.byref(root),C.byref(parent),C.byref(children),C.byref(count))
        owned=None
        try:
            for index in range(count.value):
                name=C.c_char_p();x.XFetchName(display,children[index],C.byref(name))
                try:
                    if name.value and b'dlpad' in name.value:owned=children[index];break
                finally:
                    if name:x.XFree(name)
        finally:
            if children:x.XFree(children)
        if owned is None:raise RuntimeError('owned GUI window missing')
        x.XSetInputFocus(display,owned,2,0);x.XFlush(display);time.sleep(.1)
        def key(code,pressed):t.XTestFakeKeyEvent(display,code,int(pressed),0)
        control=x.XKeysymToKeycode(display,0xffe3);shift=x.XKeysymToKeycode(display,0xffe1)
        key(control,True);code=x.XKeysymToKeycode(display,ord('a'));key(code,True);key(code,False);key(control,False)
        x.XFlush(display);time.sleep(.1)
        for char in text:
            symbol=ord(char);code=x.XKeysymToKeycode(display,symbol)
            if not code:raise RuntimeError('fixture character missing from keyboard map')
            shifted=x.XkbKeycodeToKeysym(display,code,0,0)!=symbol
            if shifted and x.XkbKeycodeToKeysym(display,code,0,1)!=symbol:raise RuntimeError('fixture keyboard layout unsupported')
            if shifted:key(shift,True)
            key(code,True);key(code,False)
            if shifted:key(shift,False)
            x.XFlush(display);time.sleep(.005)
    finally:x.XCloseDisplay(display)


def scroll_up(display_name, geometry, count=3):
    x=C.CDLL(ctypes.util.find_library('X11'));t=C.CDLL(ctypes.util.find_library('Xtst'))
    x.XOpenDisplay.argtypes=[C.c_char_p];x.XOpenDisplay.restype=C.c_void_p
    x.XCloseDisplay.argtypes=[C.c_void_p];x.XFlush.argtypes=[C.c_void_p]
    t.XTestFakeMotionEvent.argtypes=[C.c_void_p,C.c_int,C.c_int,C.c_int,C.c_ulong]
    t.XTestFakeButtonEvent.argtypes=[C.c_void_p,C.c_uint,C.c_int,C.c_ulong]
    display=x.XOpenDisplay(display_name.encode())
    if not display:raise RuntimeError('owned Xvfb scrolling unavailable')
    try:
        t.XTestFakeMotionEvent(display,0,geometry['x']+400,geometry['y']+400,0)
        for _ in range(count):
            t.XTestFakeButtonEvent(display,4,1,0);t.XTestFakeButtonEvent(display,4,0,0)
            x.XFlush(display);time.sleep(.02)
        time.sleep(.2)
    finally:x.XCloseDisplay(display)


def job(data):
    db=data/'jobs.sqlite'
    if not db.is_file():return None
    with sqlite3.connect(db.resolve().as_uri()+'?mode=ro',uri=True,timeout=.1) as connection:
        try:row=connection.execute('SELECT job_json,snapshot,options_json FROM download_jobs ORDER BY id DESC LIMIT 1').fetchone()
        except sqlite3.OperationalError as error:
            if 'no such table' in str(error):return None
            raise
    return None if row is None else dict(job=json.loads(row[0]),snapshot=json.loads(row[1]),options=json.loads(row[2]))


def wait(predicate,child,timeout):
    end=time.monotonic()+timeout
    while time.monotonic()<end:
        if child.poll() is not None:raise RuntimeError('owned GUI exited early')
        value=predicate()
        if value:return value
        time.sleep(.05)
    raise RuntimeError('fixture observation timed out')


def load_layout(path):
    if path is None:return None
    if path.stat().st_size>8192:raise ValueError('layout exceeds 8192 bytes')
    layout=json.loads(path.read_text())
    needed=['url','none','ko','en','both','auto','start','plan','range_start','range_end','cancel']
    if set(layout)!=set(needed):raise ValueError('layout keys must be '+','.join(needed))
    for point in layout.values():
        if not isinstance(point,list) or len(point)!=2 or any(type(n)is not int for n in point) or not (0<=point[0]<960 and 110<=point[1]<590):raise ValueError('layout points must be inside fixture content')
    return layout


def probe(args,case):
    started=time.monotonic()
    result={'case':case,'auto':args.auto,'display':'Xvfb isolated','actual_media_used':False,'native_gui_verified':False}
    with args.binary.open('rb') as binary:result['binary_sha256']=file_sha256(binary)
    prefix=args.output.parent/(args.output.stem+'-'+case)
    with tempfile.TemporaryDirectory(prefix='dlpad-subtitle-',dir=args.output.parent) as temporary:
        root=Path(temporary);data=root/'app-data';data.mkdir();media=root/'synthetic-media';media.mkdir()
        tool=root/'yt-dlp-synthetic';tool.write_text(TOOL);tool.chmod(0o700)
        if case=='freeze':(root/'slow').write_text('owned delayed fixture')
        with probe_display(prefix,False) as (display,_):
            env=dict(os.environ);env.pop('WAYLAND_DISPLAY',None);env.pop('DLPAD_TEST_DOCUMENTS',None)
            env.update(DISPLAY=display,DLPAD_TEST_FONT=str(args.font),DLPAD_TEST_DATA=str(data),DLPAD_LIBRARY_ROOT=str(media),DLPAD_YTDLP=str(tool),MESA_SHADER_CACHE_DIR=str(root/'shader-cache'))
            with prefix.with_suffix('.log').open('xb') as logs:
                child=subprocess.Popen([str(args.binary),'--fixture','--downloads'],env=env,stdout=logs,stderr=subprocess.STDOUT,start_new_session=True)
                try:
                    time.sleep(2);geometry=screenshot(prefix.with_name(prefix.name+'-top.png'),'dlpad',display)
                    if args.inspect:
                        scroll_to_end(display,geometry);screenshot(prefix.with_name(prefix.name+'-controls.png'),'dlpad',display)
                        scroll_up(display,geometry);screenshot(prefix.with_name(prefix.name+'-range.png'),'dlpad',display)
                        result['inspection_only']=True
                    else:
                        layout=args.layout
                        key_input(display,geometry,layout['url'],'https://example.org/synthetic')
                        scroll_to_end(display,geometry);time.sleep(.2)
                        language='both' if case in ('conflict','freeze') else case
                        click_at(display,geometry,*layout[language]);time.sleep(.15)
                        if language!='none' and args.auto=='on':click_at(display,geometry,*layout['auto']);time.sleep(.15)
                        if case=='conflict':
                            scroll_up(display,geometry)
                            key_input(display,geometry,layout['range_start'],'0')
                            key_input(display,geometry,layout['range_end'],'1')
                            screenshot(prefix.with_name(prefix.name+'-range.png'),'dlpad',display)
                            scroll_to_end(display,geometry)
                        screenshot(prefix.with_name(prefix.name+'-selected.png'),'dlpad',display)
                        click_at(display,geometry,*layout['plan']);time.sleep(.2)
                        screenshot(prefix.with_name(prefix.name+'-plan.png'),'dlpad',display)
                        # Preview adds content below the controls. The existing offset stays fixed.
                        click_at(display,geometry,*layout['start'])
                        if case=='conflict':
                            time.sleep(.5)
                            if job(data) is not None or (root/'execution.json').exists():raise RuntimeError('subtitle/range conflict executed a job')
                            result.update(conflict_no_job=True,error_text_requires_screenshot_review=True)
                        else:
                            wait(lambda:(root/'execution.json').is_file(),child,args.timeout)
                            actual=json.loads((root/'execution.json').read_text())
                            expected=None if language=='none' else ('ko,en' if language=='both' else language)
                            if expected is None:
                                if '--write-subs' in actual or '--write-auto-subs' in actual:raise RuntimeError('none choice emitted subtitle flags')
                            elif '--write-subs' not in actual or actual[actual.index('--sub-langs')+1]!=expected or ('--write-auto-subs' in actual)!=(args.auto=='on'):raise RuntimeError('selected subtitle argv mismatch')
                            current=wait(lambda:job(data),child,args.timeout)
                            selected=current['options'].get('subtitles')
                            expected_options=None if expected is None else {'language':{'ko':'Korean','en':'English','both':'KoreanAndEnglish'}[language],'auto_generated':args.auto=='on'}
                            if selected!=expected_options:raise RuntimeError('selected subtitle options mismatch in journal')
                            snapshot=current['snapshot']
                            if expected is not None and (snapshot[snapshot.index('--sub-langs')+1]!=expected or ('--write-auto-subs' in snapshot)!=(args.auto=='on')):raise RuntimeError('subtitle snapshot differs from UI selection')
                            if case=='freeze':
                                wait(lambda:(root/'running').is_file(),child,args.timeout)
                                before=current['snapshot'];click_at(display,geometry,*layout['none']);click_at(display,geometry,*layout['auto']);time.sleep(.2)
                                after=job(data)
                                if after['snapshot']!=before or after['options']['subtitles']!=current['options']['subtitles']:raise RuntimeError('running selection changed pinned journal')
                                screenshot(prefix.with_name(prefix.name+'-running.png'),'dlpad',display)
                                click_at(display,geometry,*layout['cancel'])
                                current=wait(lambda:(j if (j:=job(data)) and j['job']['state']=='Cancelled' else None),child,args.timeout)
                                result['running_snapshot_preserved']=True
                                # A second GUI request also proves latent UI options stayed frozen;
                                # an immutable first snapshot alone cannot prove that.
                                (root/'slow').unlink();(root/'execution.json').unlink()
                                first_id=current['job']['id']
                                click_at(display,geometry,*layout['start'])
                                second=wait(lambda:(j if (j:=job(data)) and j['job']['id']!=first_id and j['job']['state']=='Downloaded' else None),child,args.timeout)
                                if second['options'].get('subtitles')!=current['options']['subtitles']:raise RuntimeError('running click changed subsequent GUI subtitle state')
                                result.update(subsequent_request_preserved_subtitles=True,subsequent_journal=second)
                            else:
                                current=wait(lambda:(j if (j:=job(data)) and j['job']['state']=='Downloaded' else None),child,args.timeout)
                            published=result.get('subsequent_journal',current)
                            if case!='freeze' or 'subsequent_journal' in result:
                                artifacts=published['job']['artifacts']
                                wanted=1+(0 if expected is None else len(expected.split(',')))
                                if len(artifacts)!=wanted:raise RuntimeError('GUI publication artifact count mismatch')
                                for artifact in artifacts:
                                    path=Path(artifact['path'])
                                    if path.parent!=media or artifact['bytes']!=path.stat().st_size:raise RuntimeError('artifact escaped owned fixture or size differs')
                                    if path.suffix=='.vtt' and not path.read_bytes().startswith(b'WEBVTT synthetic '):raise RuntimeError('synthetic subtitle bytes differ')
                                result['published_artifact_count_verified']=wanted
                            result.update(argv=actual,journal=current,native_gui_verified=True)
                        if case!='conflict':
                            scroll_to_end(display,geometry)
                        screenshot(prefix.with_name(prefix.name+'-final.png'),'dlpad',display)
                except Exception as error:
                    result['error']=str(error)
                    try:screenshot(prefix.with_name(prefix.name+'-error.png'),'dlpad',display)
                    except Exception:pass
                finally:
                    if child.poll() is None:
                        os.killpg(child.pid,signal.SIGTERM)
                        try:child.wait(timeout=3)
                        except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);child.wait()
                    result['owned_gui_reaped']=child.poll() is not None
    result['temporary_fixture_removed']=True
    result['elapsed_seconds']=round(time.monotonic()-started,3)
    prefix.with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({key:result[key] for key in ('case','native_gui_verified','elapsed_seconds','error') if key in result}))
    return int('error'in result)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary',type=Path);p.add_argument('output',type=Path)
    p.add_argument('--auto',choices=['on','off'],default='on');p.add_argument('--case',choices=[*CASES,'all'],default='both');p.add_argument('--inspect',action='store_true')
    p.add_argument('--layout',type=Path);p.add_argument('--font',type=Path,default=os.environ.get('DLPAD_TEST_FONT'))
    p.add_argument('--timeout',type=float,default=15);p.add_argument('--validate-only',action='store_true')
    args=p.parse_args()
    if not 1<=args.timeout<=60:p.error('timeout must be1..60 seconds')
    if args.output.suffix!='.json':p.error('output must end in.json')
    try:args.layout=load_layout(args.layout)
    except (ValueError,OSError) as error:p.error(str(error))
    if not args.inspect and args.layout is None:p.error('--layout required except --inspect')
    if args.inspect and args.case=='all':p.error('--inspect requires one case')
    if args.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('this harness validates Linux X11 only')
    args.binary=args.binary.resolve();args.output=args.output.resolve()
    if not args.binary.is_file() or not os.access(args.binary,os.X_OK):p.error('binary must be executable')
    if args.font is None or not args.font.is_file():p.error('explicit fixture font required')
    args.font=args.font.resolve();args.output.parent.mkdir(parents=True,exist_ok=True)
    if list(args.output.parent.glob(args.output.stem+'-*')):p.error('evidence prefix already exists')
    cases=CASES if args.case=='all' else [args.case]
    exit_code=0
    for case in cases:
        try:exit_code=max(exit_code,probe(args,case))
        except (Exception,KeyboardInterrupt) as error:
            result={'case':case,'display':'Xvfb isolated','native_gui_verified':False,'error':str(error) or type(error).__name__,'desktop_fallback_used':False}
            (args.output.parent/(args.output.stem+'-'+case+'.json')).write_text(json.dumps(result,indent=2)+'\n')
            print(json.dumps(result));return 1
    return exit_code


if __name__=='__main__':raise SystemExit(main())
