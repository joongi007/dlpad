#!/usr/bin/env python3
"""CPU metrics on owned Xvfb/private bus/temp volume. No external network or clipboard."""
import argparse, ast, json, os, re, select, sqlite3, subprocess, tempfile, threading, time
from pathlib import Path
from accessible_download_probe import file_hash, bounded_log
from native_probe import probe_display, screenshot
from egui_ime_probe import accessibility, call
from background_ui_probe import reap, process_identity, OwnedDisplay
from ime_probe import focus_owned_window, key_sequence
from x11_diagnostics import click_at


def wait(predicate, child, timeout=15):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        value=predicate()
        if value:return value
        if child.poll() is not None:raise RuntimeError('owned GUI exited early')
        time.sleep(.05)
    raise RuntimeError('owned observation deadline exceeded')


def enabled_node(env,address,name):
    for node in accessibility(env).get('nodes',[])[:256]:
        if "Name': <'"+name+"'>" not in node['properties']['stdout']:continue
        state=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
        match=re.search(r'(?:uint32 )?(\d+)',state['stdout'])
        if not match or not int(match.group(1))&(1<<8):continue
        bounds=call(env,node['destination'],node['path'],'org.a11y.atspi.Component.GetExtents','0',address=address)
        if bounds['exit']:continue
        x,y,w,h=ast.literal_eval(bounds['stdout'])[0]
        if w>0 and h>0 and 0<=x+w//2<960 and 0<=y+h//2<640:return (x+w//2,y+h//2),node
    return None


def run(a):
    r={'case':a.case,'binary_sha256':file_hash(a.binary),'passed':False,'actual_media_used':False,'network_install_started':False,'desktop_fallback_used':False}
    if r['binary_sha256']!=a.expected_sha:raise RuntimeError('frozen binary hash mismatch')
    app=bus=None;context=None;threads=[];lock=None;owner=None
    r['input_exact_observations']=[]
    with tempfile.TemporaryDirectory(prefix='cpu-metrics-owned-') as tmp:
        root=Path(tmp)
        for name in ('data','media','config','cache','runtime','xdg-data'):(root/name).mkdir(mode=0o700)
        expected_free=os.statvfs(root/'media');expected_free=expected_free.f_bavail*expected_free.f_frsize
        r['statvfs_available_bytes_before']=expected_free
        env=dict(os.environ)
        for key in ('WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE','DLPAD_YTDLP'):env.pop(key,None)
        env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
        try:
            context=probe_display(a.output/'isolated',False);display,kind=context.__enter__()
            if kind!='Xvfb isolated':raise RuntimeError('owned Xvfb required')
            env['DISPLAY']=display
            bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
            t=threading.Thread(target=bounded_log,args=(bus.stderr,a.output/'bus.log'),daemon=True);t.start();threads.append(t)
            if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus timeout')
            env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();tree=accessibility(env);address=re.search(r"'([^']+)'",tree['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
            app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
            t=threading.Thread(target=bounded_log,args=(app.stdout,a.output/'app.log'),daemon=True);t.start();threads.append(t)
            r['app_identity']=process_identity(app.pid);time.sleep(2);focus_owned_window(display);geometry=screenshot(a.output/'initial.png','dlpad',display)
            def click(name):
                position,node=wait(lambda:enabled_node(env,address,name),app)
                click_at(display,geometry,*position);time.sleep(.2);return node
            from subtitle_ui_probe import key_input
            click('설정')
            def field(value):
                from cpu_options_probe import lookup
                position,_=wait(lambda:lookup(env,address,'인터페이스','79'),app)
                key_input(display,geometry,position,value)
                if value=='':key_sequence(display,['BackSpace'])
                time.sleep(.15)
                def exact():
                    for n in accessibility(env).get('nodes',[]):
                        if "Name': <'인터페이스'>" in n['properties']['stdout'] and 'uint32 79' in n['role']['stdout'] and n.get('text',{}).get('exit')==0 and ast.literal_eval(n['text']['stdout'])[0]==value:return True
                    return False
                wait(exact,app)
                r['input_exact_observations'].append(value)

            def labels():
                return [ast.literal_eval(n['text']['stdout'])[0] for n in accessibility(env).get('nodes',[]) if n.get('text',{}).get('exit')==0]
            field('lo');click('관측 적용')
            valid=wait(lambda: next((text for text in labels() if text.startswith('수신 ') and 'KiB/s' in text),None),app)
            r['lo_observation']=valid
            free_text=wait(lambda:next((text for text in labels() if text.startswith('대상 볼륨 여유: ') and 'GiB' in text),None),app)
            r['volume_ui']=free_text
            shown=float(re.search(r'([0-9.]+) GiB',free_text).group(1));actual=os.statvfs(root/'media');actual=actual.f_bavail*actual.f_frsize
            r['statvfs_available_bytes_after']=actual
            assert abs(shown-actual/1073741824.0)<=.1
            r['rounded_volume_matches']=True
            field('../invalid');click('관측 적용')
            wait(lambda:'64자 이하의 영문·숫자·점·밑줄·하이픈 이름을 입력하세요.' in labels(),app)
            r['invalid_preserves_lo']=any(text.startswith('수신 ') and 'KiB/s' in text for text in labels())
            assert r['invalid_preserves_lo']
            field('dlpad-fixture-missing');click('관측 적용')
            wait(lambda:'수신·송신 측정 불가' in labels(),app)
            r['missing_interface_unknown']=True
            field('');click('관측 적용');time.sleep(1.2)
            assert '수신·송신 측정 불가' in labels();r['blank_off']=True
            screenshot(a.output/'settings.png','dlpad',display)
            field('lo');click('관측 적용');wait(lambda:any(text.startswith('수신 ') and 'KiB/s' in text for text in labels()),app)
            started=time.monotonic();r['periodic_samples']=[]
            for _ in range(3):
                time.sleep(1.05);r['periodic_samples'].append({'elapsed':time.monotonic()-started,'network':next((text for text in labels() if text.startswith('수신 ') and 'KiB/s' in text),None)})
            assert all(item['network'] is not None for item in r['periodic_samples'])
            # Repeated visible samples prove ongoing observation, not exact internal syscall counts.
            owner=OwnedDisplay(display);started=time.monotonic();owner.delete_window(owner.window());time.sleep(.15)
            stop_node=enabled_node(env,address,'중단 후 종료') if app.poll() is None else None
            if stop_node:click_at(display,geometry,*stop_node[0])
            app.wait(timeout=8);r['close_seconds']=time.monotonic()-started;r['close_exit']=app.returncode
            assert app.returncode==0;r['passed']=True
            if app.poll() is None:screenshot(a.output/'final.png','dlpad',display)
        except Exception as error:
            r['error']=str(error)
            if context:
                try:screenshot(a.output/'failure.png','dlpad',display)
                except Exception:pass
        finally:
            if lock:lock.rollback();lock.close()
            if app:reap(app)
            if bus:reap(bus)
            for t in threads:t.join(3)
            if owner:owner.close()
            if context:context.__exit__(None,None,None)
            r['owned_cleanup']=all(p is None or p.poll() is not None for p in (app,bus)) and all(not t.is_alive() for t in threads)
    r['binary_sha256_after']=file_hash(a.binary);r['passed']=r['passed'] and r['owned_cleanup'] and r['binary_sha256_after']==r['binary_sha256']
    (a.output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n');return 0 if r['passed'] else 1


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--case',choices=('observations',),default='observations');p.add_argument('--validate-only',action='store_true');a=p.parse_args()
    if not re.fullmatch('[0-9a-f]{64}',a.expected_sha):p.error('expected SHA256 required')
    if a.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux only')
    a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
    return run(a)

if __name__=='__main__':raise SystemExit(main())
