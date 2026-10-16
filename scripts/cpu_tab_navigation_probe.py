#!/usr/bin/env python3
"""Actual Tab/ShiftTab navigation on owned Xvfb; no GrabFocus substitution."""
import argparse, ast, json, os, re, select, subprocess, tempfile, threading, time
from pathlib import Path
from accessible_download_probe import file_hash, bounded_log, snapshot_database
from cpu_options_probe import lookup
from native_probe import probe_display, screenshot
from egui_ime_probe import accessibility, call
from background_ui_probe import OwnedDisplay, reap, process_identity
from ime_probe import focus_owned_window, key_sequence
from subtitle_ui_probe import job
from x11_diagnostics import click_at
EXPECTED='ca8e969f028fdd50f5c7f9ae19c3c71928d7740daa5c93189ec100ee9f5fc77e'
HELP={'FFmpeg 선택 도움말','세션 쿠키 선택 도움말','품질 선택 도움말','구간 입력 도움말','자르기 방식 도움말'}

def focused(env,address,deadline):
    queue=[('org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root')];seen=set();found=[]
    while queue and len(seen)<256:
        if time.monotonic()>deadline:raise RuntimeError('navigation total deadline exceeded')
        dest,path=queue.pop(0)
        if (dest,path) in seen:continue
        seen.add((dest,path))
        state=call(env,dest,path,'org.a11y.atspi.Accessible.GetState',address=address)
        words=re.search(r'\[(?:uint32 )?(\d+), (?:uint32 )?(\d+)\]',state['stdout'])
        if state['exit'] or not words or any(int(v)>0xffffffff for v in words.groups()):raise RuntimeError('native state unavailable')
        bits=int(words.group(1))|(int(words.group(2))<<32)
        if bits&(1<<12):
            props=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
            role=call(env,dest,path,'org.a11y.atspi.Accessible.GetRole',address=address)
            bounds=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address)
            if any(v['exit'] for v in [props,role,bounds]):raise RuntimeError('focused node metadata unavailable')
            match=re.search(r"Name': <'([^']*)'>",props['stdout'])
            if not match:raise RuntimeError('focused node name unavailable')
            found.append({'name':match.group(1),'role':role,'state':state,'state_bits':bits,'bounds':list(ast.literal_eval(bounds['stdout'])[0]),'destination':dest,'path':path})
        children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address)
        if children['exit']:raise RuntimeError('native children unavailable')
        queue.extend(re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",children['stdout']))
    if queue:raise RuntimeError('native tree exceeds 256 nodes')
    if len(found)!=1:raise RuntimeError('expected one actual focused node; observed '+str(len(found)))
    return found[0]

def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--expected-sha256',default=EXPECTED);p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));a=p.parse_args()
    a.binary=a.binary.resolve(strict=True);a.output.mkdir(parents=True,exist_ok=False)
    r={'passed':False,'tab_navigation_verified':False,'visual_focus_review':'REVIEW_REQUIRED','observations':[],'binary_sha256':file_hash(a.binary),'desktop_fallback_used':False};app=bus=owner=context=None;threads=[]
    if r['binary_sha256']!=a.expected_sha256:raise RuntimeError('immutable binary mismatch')
    with tempfile.TemporaryDirectory(prefix='cpu-tab-owned-') as tmp:
        root=Path(tmp)
        for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)
        tool=root/'never-executed-tool';tool.write_text('#!/usr/bin/env python3\nfrom pathlib import Path\nPath(__file__).with_name("EXECUTED").write_text("unexpected")\n');tool.chmod(0o700)
        env=dict(os.environ)
        for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(key,None)
        env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
        try:
            context=probe_display(a.output/'isolated',False);display,kind=context.__enter__()
            if kind!='Xvfb isolated':raise RuntimeError('owned Xvfb required')
            env['DISPLAY']=display;bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
            t=threading.Thread(target=bounded_log,args=(bus.stderr,a.output/'bus.log'),daemon=True);t.start();threads.append(t)
            if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus readiness timeout')
            env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();tree=accessibility(env);address=re.search(r"'([^']+)'",tree['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
            app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
            t=threading.Thread(target=bounded_log,args=(app.stdout,a.output/'app.log'),daemon=True);t.start();threads.append(t);r['app_identity']=process_identity(app.pid);r['bus_identity']=process_identity(bus.pid)
            time.sleep(2);owner=OwnedDisplay(display);geometry=screenshot(a.output/'initial.png','dlpad',display);focus_owned_window(display)
            point,_=lookup(env,address,'영상 URL','79');click_at(display,geometry,*point);time.sleep(.2);deadline=time.monotonic()+180
            def observe(stage):
                node=focused(env,address,deadline);node['stage']=stage;r['observations'].append(node)
                x,y,w,h=node['bounds'];node['center_visible']=w>0 and h>0 and 0<=x+w//2<960 and 65<=y+h//2<630
                screenshot(a.output/(stage+'.png'),'dlpad',display)
                if not node['center_visible'] or not node['state_bits']&(1<<30):raise RuntimeError('focused node outside visible owned content')
                return node['name']
            if observe('initial-url')!='영상 URL':raise RuntimeError('initial URL click did not focus URL')
            reached=set();count=0
            for count in range(1,65):
                key_sequence(display,['Tab']);time.sleep(.12);name=observe('tab-'+str(count));reached.add(name)
                if HELP<=reached:break
            if not HELP<=reached:raise RuntimeError('five help headers not reached within 64 Tab keys')
            r['forward_tab_count']=count;r['help_reached']=sorted(HELP&reached)
            for index in range(1,count+1):
                key_sequence(display,['Shift_L+Tab']);time.sleep(.12);last=observe('shift-tab-'+str(index))
            if last!='영상 URL':raise RuntimeError('ShiftTab did not return to actual URL focus')
            if job(root/'data') is not None or (root/'EXECUTED').exists():raise RuntimeError('navigation unexpectedly created job or executed tool')
            r.update(tab_navigation_verified=True,job_created=False,tool_executed=False)
            owner.delete_window(owner.window());app.wait(timeout=10)
            if app.returncode!=0:raise RuntimeError('owned GUI close failed')
            r['passed']=True
        except Exception as error:
            r['error']=str(error)
            if owner:
                try:screenshot(a.output/'failure.png','dlpad',env['DISPLAY'])
                except Exception as nested:r['capture_error']=str(nested)
        finally:
            if app:reap(app)
            if owner:owner.close()
            if bus:reap(bus)
            for t in threads:t.join(3)
            if context:context.__exit__(None,None,None)
            r['owned_cleanup']=all(v is None or v.poll() is not None for v in [app,bus]) and all(not t.is_alive() for t in threads);snapshot_database(root/'data',a.output/'jobs.sqlite')
    r['binary_sha256_after']=file_hash(a.binary);r['passed']=r['passed'] and r['owned_cleanup'] and r['binary_sha256_after']==r['binary_sha256'];(a.output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));return 0 if r['passed'] else 1
if __name__=='__main__':raise SystemExit(main())
