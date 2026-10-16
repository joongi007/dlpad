#!/usr/bin/env python3
"""CPU appearance persistence and scale observations on owned Xvfb/private bus."""
import argparse, ast, ctypes as C, ctypes.util, json, os, re, select, sqlite3, subprocess, tempfile, time
from cpu_scale_paint_probe import pixels
from native_probe import XImage
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

def find_fast(env,address,name,require_action=False):
    reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
    queue=re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",reply['stdout']);seen=set()
    while queue and len(seen)<64:
        dest,path=queue.pop(0)
        if (dest,path) in seen:continue
        seen.add((dest,path));node={'destination':dest,'path':path}
        props=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
        if f"Name': <'{name}'>" in props['stdout']:
            actions=call(env,dest,path,'org.a11y.atspi.Action.GetActions',address=address) if require_action else None
            role=call(env,dest,path,'org.a11y.atspi.Accessible.GetRole',address=address) if require_action else None
            if not require_action or (actions['exit']==0 and "'click'" in actions['stdout'] and 'uint32 29' not in role['stdout']):return node
        children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address)
        queue.extend(re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",children['stdout']))
    raise RuntimeError('fast named widget not found: '+name)

def run(args):
    result={'binary_sha256':binary_hash(args.binary),'binary_bytes':args.binary.stat().st_size,'font_sha256':binary_hash(args.font),'fixture_count':0,'user_desktop_used':False,'clipboard_used':False,'audio_used':False,'actual_media_used':False}
    app=bus=owner=lock=None
    try:
        with tempfile.TemporaryDirectory(prefix='cpu-lib-') as tmp:
            root=Path(tmp);data=root/'data';media=root/'media'
            for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)

            result['fixture_setup_complete']=True
            with probe_display(args.output/'isolated.json',False,screen=(2200,1400)) as (display,kind):
                if kind!='Xvfb isolated':raise RuntimeError('refusing desktop fallback')
                env=dict(os.environ)
                for name in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(name,None)
                env.update(DISPLAY=display,DLPAD_TEST_DATA=str(data),DLPAD_LIBRARY_ROOT=str(media),DLPAD_TEST_FONT=str(args.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
                with (args.output/'bus.log').open('xb') as blog,(args.output/'app.log').open('xb') as alog:
                    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
                    if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('private bus timeout')
                    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();registry=accessibility(env);address=re.search(r"'([^']+)'",registry['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
                    owner=OwnedDisplay(display)
                    def launch(label):
                        nonlocal app
                        started=time.perf_counter();app=subprocess.Popen([str(args.binary),'--fixture'],env=env,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True)
                        def painted():
                            try:
                                window=owner.window()
                                if owner.map_state(window)!=2:return None
                                ok,observations=pixels(owner,window,1)
                                return (window,observations) if ok else None
                            except RuntimeError:return None
                        window,observations=wait(painted,app,5)
                        record={'label':label,'native_glyph_seconds':time.perf_counter()-started,'glyph_regions':observations}
                        owner.x.XGetImage.restype=C.POINTER(XImage)
                        im=owner.x.XGetImage(owner.display,window,900,300,1,1,C.c_ulong(-1),2)
                        if not im:raise RuntimeError('native background unavailable')
                        try:record['background_bgr']=list(C.string_at(im.contents.data,4)[:3])
                        finally:owner.x.XDestroyImage(im)
                        result.setdefault('launches',[]).append(record)
                        focus_owned_window(display)
                        return window
                    def action(name):
                        node=find_fast(env,address,name,require_action=True)
                        if name in ['화면 테마','화면 배율']:
                            extent=call(env,node['destination'],node['path'],'org.a11y.atspi.Component.GetExtents','0',address=address)
                            x,y,w,h=ast.literal_eval(extent['stdout'])[0]
                            if h<32:raise RuntimeError('Theme/scale ComboBox height below32: '+name+' '+str(h))
                            click_at(display,{'x':0,'y':0},x+w//2,y+h//2)
                            result.setdefault('physical_combo_clicks',[]).append({'name':name,'extents':extent})
                            reply={'exit':0,'stdout':'(true,)'}
                        else:reply=timed_call(result,env,address,node,'org.a11y.atspi.Action.DoAction','0')
                        if reply['exit'] or '(true' not in reply['stdout']:raise RuntimeError('GUI action rejected: '+name)
                        time.sleep(.15)
                    def theme_value():
                        try:
                            with sqlite3.connect((data/'theme.sqlite').as_uri()+'?mode=ro',uri=True,timeout=.1) as db:
                                row=db.execute("SELECT value FROM settings WHERE key='theme'").fetchone()
                                return row[0] if row else None
                        except sqlite3.Error:return None
                    launch('initial')
                    for label,value in [('어둡게','Dark'),('밝게','Light'),('시스템','System')]:
                        action('설정');action('화면 테마')
                        popup_tree=accessibility(env);result.setdefault('theme_popup_trees',[]).append(popup_tree)
                        _,fixture_entry=button(env,address,'한글 입력 검증','79',snapshot=popup_tree)
                        fixture_extent=call(env,fixture_entry['destination'],fixture_entry['path'],'org.a11y.atspi.Component.GetExtents','0',address=address)
                        fixture_rect=ast.literal_eval(fixture_extent['stdout'])[0]
                        result.setdefault('ime_input100_extents',[]).append(fixture_rect)
                        if fixture_rect[3]<32:raise RuntimeError('Settings fixture input below32px')
                        screenshot(args.output/(value+'-popup.png'),'dlpad',display)
                        action(label)
                        try:find_fast(env,address,'어둡게' if label!='어둡게' else '밝게',require_action=True)
                        except RuntimeError:result.setdefault('popup_closed_after_theme_action',[]).append(True)
                        else:raise RuntimeError('Theme popup remained open after accessible selection')
                        wait(lambda:theme_value()==json.dumps(value),app,5)
                        result.setdefault('saved_themes',[]).append({'label':label,'value':theme_value()})
                        screenshot(args.output/(value+'-selected.png'),'dlpad',display)
                        owner.delete_window(owner.window());app.wait(timeout=8)
                        if app.returncode!=0:raise RuntimeError('close failed')
                        result.setdefault('close_codes',[]).append(app.returncode)
                        launch(value+'-restart')
                        frame=result['launches'][-1]
                        if value=='Light' and min(frame['background_bgr'])<180:raise RuntimeError('Light restart first painted frame was not light')
                        if value=='Dark' and max(frame['background_bgr'])>80:raise RuntimeError('Dark restart first painted frame was not dark')
                        screenshot(args.output/(value+'-first.png'),'dlpad',display)
                    action('설정');action('화면 배율');action('200%')
                    try:find_fast(env,address,'175%',require_action=True)
                    except RuntimeError:result['scale_popup_closed_after_action']=True
                    else:raise RuntimeError('Scale popup remained open after accessible selection')
                    window=owner.window();owner.x.XResizeWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_uint,C.c_uint]
                    owner.x.XResizeWindow(owner.display,window,1600,1200);owner.x.XFlush(owner.display);time.sleep(.3)
                    result['tree_scale']=accessibility(env)
                    for n in result['tree_scale']['nodes']:n['extents']=call(env,n['destination'],n['path'],'org.a11y.atspi.Component.GetExtents','0',address=address)
                    screenshot(args.output/'scale200.png','dlpad',display)
                    _,input_node=button(env,address,'한글 입력 검증','79',snapshot=result['tree_scale']);extent=call(env,input_node['destination'],input_node['path'],'org.a11y.atspi.Component.GetExtents','0',address=address)
                    x,y,w,h=ast.literal_eval(extent['stdout'])[0]
                    result['ime_input200_extents']=[x,y,w,h]
                    if h<64:raise RuntimeError('Settings fixture input below64px at200%')
                    click_at(display,{'x':0,'y':0},x+w//2,y+h//2);time.sleep(.15);key_sequence(display,['a','b','c'])
                    result['entry_text']=call(env,input_node['destination'],input_node['path'],'org.a11y.atspi.Text.GetText','0','-1',address=address)
                    result['entry_state_after_click']=call(env,input_node['destination'],input_node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                    if "'abc'" not in result['entry_text']['stdout']:raise RuntimeError('ASCII input not captured by actual entry')
                    key_sequence(display,['Shift_L+Tab']);time.sleep(.1)
                    result['tree_keyboard']=accessibility(env)
                    for n in result['tree_keyboard']['nodes']:n['state']=call(env,n['destination'],n['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                    result['keyboard_focused_nodes']=[n['properties']['stdout'] for n in result['tree_keyboard']['nodes'] if (m:=re.search(r'uint32 (\d+)',n['state']['stdout'])) and int(m.group(1))&(1<<12)]
                    if not any("화면 배율" in text for text in result['keyboard_focused_nodes']):raise RuntimeError('ShiftTab did not focus actual scale combo')
                    key_sequence(display,['Tab']);time.sleep(.1)
                    result['entry_state_after_return_tab']=call(env,input_node['destination'],input_node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                    state_match=re.search(r'uint32 (\d+)',result['entry_state_after_return_tab']['stdout'])
                    if not state_match or not int(state_match.group(1))&(1<<12):raise RuntimeError('Tab did not return focus to actual entry')
                    screenshot(args.output/'scale200-keyboard.png','dlpad',display)
                    action('화면 배율');action('100%');action('파일 목록')
                    library_tree=accessibility(env);_,library_root_node=button(env,address,'저장 폴더','79',snapshot=library_tree)
                    library_extent=call(env,library_root_node['destination'],library_root_node['path'],'org.a11y.atspi.Component.GetExtents','0',address=address)
                    library_rect=ast.literal_eval(library_extent['stdout'])[0];result['library_root100_extents']=library_rect
                    if library_rect[3]<32:raise RuntimeError('Library root input below32px')
                    screenshot(args.output/'library-root32.png','dlpad',display);action('설정')
                    result['scale_is_session_only']=True
                    result['system_os_theme_event_verified']=False
                    lock=sqlite3.connect(data/'theme.sqlite',timeout=.1);lock.execute('BEGIN EXCLUSIVE');result['save_lock_acquired_at']=time.perf_counter()
                    action('화면 테마');action('밝게');key_sequence(display,['Escape']);time.sleep(.1)
                    owner.delete_window(owner.window());time.sleep(.1)
                    stop_node=find_fast(env,address,'중단 후 종료',require_action=True)
                    result['save_stop_action']=timed_call(result,env,address,stop_node,'org.a11y.atspi.Action.DoAction','0');time.sleep(.15)
                    result['alive_while_theme_write_locked']=app.poll() is None
                    released=time.perf_counter();lock.rollback();lock.close();lock=None
                    app.wait(timeout=8);result['close_exit_code']=app.returncode;result['save_close_after_release_seconds']=time.perf_counter()-released
                    result['theme_after_save_close']=theme_value()
                    if not result['alive_while_theme_write_locked'] or result['theme_after_save_close']!=json.dumps('Light'):raise RuntimeError('in-flight theme close did not preserve completed write')
                    stop_owned(app);stop_owned(bus);owner.close();owner=None
    except Exception as error:result['error']=str(error)
    finally:
        if lock:lock.rollback();lock.close()
        stop_owned(app);stop_owned(bus)
        result['owned_direct_children_reaped']=all(p is None or p.poll() is not None for p in [app,bus])
    result['temporary_fixture_removed']=True;result['after_binary_sha256']=binary_hash(args.binary);result['hash_stable']=result['binary_sha256']==result['after_binary_sha256']
    (args.output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if not k.startswith('tree_') and k not in ['theme_popup_trees','calls']},ensure_ascii=False,indent=2));return int('error' in result or result.get('close_timeout',False) or result.get('close_exit_code') != 0)

def main():
    p=argparse.ArgumentParser(description='Owned native CPU settings probe; requires root READY hash');p.add_argument('binary',type=Path);p.add_argument('output',type=Path);p.add_argument('--font',type=Path,required=True);p.add_argument('--ready-sha',required=True);p.add_argument('--validate-only',action='store_true');a=p.parse_args()
    if a.validate_only:print('valid; no processes started');return 0
    a.binary=a.binary.resolve(strict=True)
    if binary_hash(a.binary)!=a.ready_sha:p.error('READY binary hash mismatch')
    a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);return run(a)
if __name__=='__main__':raise SystemExit(main())
