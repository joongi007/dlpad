#!/usr/bin/env python3
"""Native CPU library observations on owned Xvfb/private bus and 10,000 dummy files."""
import argparse, ast, ctypes as C, ctypes.util, json, os, re, select, sqlite3, subprocess, tempfile, time
from pathlib import Path
from native_probe import probe_display,screenshot,sample
from egui_ime_probe import accessibility,binary_hash,call
from background_ui_probe import OwnedDisplay
from ime_probe import stop_owned,focus_owned_window,key_sequence
from subtitle_ui_probe import key_input
from x11_diagnostics import click_at

def wait(predicate, app, timeout=15):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        value=predicate()
        if value:return value
        if app.poll() is not None:raise RuntimeError('owned candidate exited early')
        time.sleep(.05)
    raise RuntimeError('owned fixture observation timeout')

def inventory(data):
    p=data/'library.sqlite'
    if not p.is_file():return None
    try:
        with sqlite3.connect(p.as_uri()+'?mode=ro',uri=True,timeout=.1) as db:
            return [r[0] for r in db.execute('SELECT path FROM library_entries WHERE present=1 ORDER BY path LIMIT 10001')]
    except sqlite3.Error:return None

def button(env,address,name,role=None,snapshot=None):
    snapshot=snapshot or accessibility(env)
    for n in snapshot.get('nodes',[]):
        if f"Name': <'{name}'>" not in n['properties']['stdout']:continue
        if role and role not in n['role']['stdout']:continue
        reply=call(env,n['destination'],n['path'],'org.a11y.atspi.Component.GetExtents','0',address=address)
        if reply['exit']:continue
        x,y,w,h=ast.literal_eval(reply['stdout'])[0]
        if w>0 and h>0 and 0<=x<960 and 0<=y<640:return (x+w//2,y+h//2),n
    raise RuntimeError('owned accessible widget unavailable: '+name)

def wheel(owner,amount):
    t=C.CDLL(ctypes.util.find_library('Xtst'));t.XTestFakeMotionEvent.argtypes=[C.c_void_p,C.c_int,C.c_int,C.c_int,C.c_ulong];t.XTestFakeButtonEvent.argtypes=[C.c_void_p,C.c_uint,C.c_int,C.c_ulong]
    t.XTestFakeMotionEvent(owner.display,0,150,400,0)
    for _ in range(abs(amount)):
        t.XTestFakeButtonEvent(owner.display,5 if amount>0 else 4,1,0);t.XTestFakeButtonEvent(owner.display,5 if amount>0 else 4,0,0)
    owner.x.XFlush(owner.display);time.sleep(.25)

def labels(tree):
    return [n['text']['stdout'] for n in tree.get('nodes',[]) if n['text']['exit']==0 and 'media-' in n['text']['stdout']]

def timed_call(result,env,address,node,method,*args):
    started=time.perf_counter();reply=call(env,node['destination'],node['path'],method,*args,address=address)
    result.setdefault('calls',[]).append({'method':method,'start':started,'duration':time.perf_counter()-started,'reply':reply})
    return reply

def find_fast(env,address,name):
    reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
    queue=re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",reply['stdout']);seen=set()
    while queue and len(seen)<64:
        dest,path=queue.pop(0)
        if (dest,path) in seen:continue
        seen.add((dest,path));node={'destination':dest,'path':path}
        props=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
        if f"Name': <'{name}'>" in props['stdout']:return node
        children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address)
        queue.extend(re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",children['stdout']))
    raise RuntimeError('fast named widget not found: '+name)

def run(args):
    result={'binary_sha256':binary_hash(args.binary),'binary_bytes':args.binary.stat().st_size,'font_sha256':binary_hash(args.font),'fixture_count':10000,'user_desktop_used':False,'clipboard_used':False,'audio_used':False,'actual_media_used':False}
    app=bus=owner=lock=None
    try:
        with tempfile.TemporaryDirectory(prefix='cpu-lib-') as tmp:
            root=Path(tmp);data=root/'data';media=root/'media'
            for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)
            for index in range(10000):(media/f'media-{index:05d}.mp4').write_bytes(b'dummy')
            result['fixture_setup_complete']=True
            with probe_display(args.output/'isolated.json',False) as (display,kind):
                if kind!='Xvfb isolated':raise RuntimeError('refusing desktop fallback')
                env=dict(os.environ)
                for name in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(name,None)
                env.update(DISPLAY=display,DLPAD_TEST_DATA=str(data),DLPAD_LIBRARY_ROOT=str(media),DLPAD_TEST_FONT=str(args.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
                with (args.output/'bus.log').open('xb') as blog,(args.output/'app.log').open('xb') as alog:
                    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
                    if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('private bus timeout')
                    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();registry=accessibility(env);address=re.search(r"'([^']+)'",registry['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
                    app=subprocess.Popen([str(args.binary),'--fixture'],env=env,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True);owner=OwnedDisplay(display);time.sleep(2)
                    origin=screenshot(args.output/'downloads.png','dlpad',display);focus_owned_window(display)
                    point,_=button(env,address,'파일 목록');ticks,rss=sample(app.pid);started=time.perf_counter();click_at(display,origin,*point)
                    paths=wait(lambda:(v if v and len(v)==10000 else None) if (v:=inventory(data)) is not None else None,app)
                    result['scan_committed_seconds']=time.perf_counter()-started;result['scan_cpu_ticks_delta']=sample(app.pid)[0]-ticks;result['scan_rss_kib_before']=rss;result['scan_rss_kib_after']=sample(app.pid)[1]
                    started=time.perf_counter();tree=accessibility(env);result['full_tree_seconds']=time.perf_counter()-started
                    refresh,node=button(env,address,'목록 새로고침',snapshot=tree);_,next_node=button(env,address,'저장 폴더','79',snapshot=tree)
                    result['next_before']=timed_call(result,env,address,next_node,'org.a11y.atspi.Accessible.GetState')
                    result['action_names']=timed_call(result,env,address,node,'org.a11y.atspi.Action.GetActions')
                    lock=sqlite3.connect(data/'library.sqlite',timeout=.1);lock.execute('BEGIN EXCLUSIVE');result['lock_acquired_at']=time.perf_counter()
                    result['refresh_action']=timed_call(result,env,address,node,'org.a11y.atspi.Action.DoAction','0')
                    deadline=time.perf_counter()+1.5
                    while time.perf_counter()<deadline:
                        state=timed_call(result,env,address,next_node,'org.a11y.atspi.Accessible.GetState');match=re.search(r'uint32 (\d+)',state['stdout'])
                        if match and not int(match.group(1))&(1<<8):result['busy_verified_at']=time.perf_counter();break
                        time.sleep(.02)
                    if 'busy_verified_at' not in result:raise RuntimeError('Refresh action did not expose busy within 1.5 seconds')
                    result['close_requested_at']=time.perf_counter();owner.delete_window(owner.window());time.sleep(.1)
                    started=time.perf_counter();stop_node=find_fast(env,address,'중단 후 종료');result['prompt_lookup_seconds']=time.perf_counter()-started
                    result['stop_action']=timed_call(result,env,address,stop_node,'org.a11y.atspi.Action.DoAction','0');time.sleep(.15)
                    result['close_alive_while_owned_db_lock']=app.poll() is None;result['lock_released_at']=time.perf_counter();lock.rollback();lock.close();lock=None
                    app.wait(timeout=8);result['close_exit_code']=app.returncode;result['close_after_release_seconds']=time.perf_counter()-result['lock_released_at']
                    stop_owned(app);stop_owned(bus);owner.close();owner=None
    except Exception as error:result['error']=str(error)
    finally:
        if lock:lock.rollback();lock.close()
        stop_owned(app);stop_owned(bus)
        result['owned_direct_children_reaped']=all(p is None or p.poll() is not None for p in [app,bus])
    result['temporary_fixture_removed']=True;result['after_binary_sha256']=binary_hash(args.binary);result['hash_stable']=result['binary_sha256']==result['after_binary_sha256']
    (args.output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if not k.startswith('tree_')},ensure_ascii=False,indent=2));return int('error' in result or result.get('close_timeout',False) or not result.get('close_alive_while_owned_db_lock',False) or result.get('close_exit_code') != 0)

def main():
    p=argparse.ArgumentParser(description='Narrow timed library CloseWait probe');p.add_argument('binary',type=Path);p.add_argument('output',type=Path);p.add_argument('--font',type=Path,required=True);p.add_argument('--validate-only',action='store_true');a=p.parse_args()
    if a.validate_only:print('valid; no processes started');return 0
    a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);return run(a)
if __name__=='__main__':raise SystemExit(main())
