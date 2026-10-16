#!/usr/bin/env python3
"""CPU tools on an owned Xvfb/private bus. No network installation or desktop fallback."""
import argparse, ast, json, os, re, select, sqlite3, subprocess, tempfile, threading, time
from pathlib import Path
from accessible_download_probe import file_hash, bounded_log
from managed_retained_probe import seed, TARGET
from managed_tools_probe import active
from native_probe import probe_display, screenshot
from egui_ime_probe import accessibility, call
from background_ui_probe import reap, process_identity, OwnedDisplay
from ime_probe import focus_owned_window
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
    with tempfile.TemporaryDirectory(prefix='cpu-tools-owned-') as tmp:
        root=Path(tmp)
        for name in ('data','media','config','cache','runtime','xdg-data'):(root/name).mkdir(mode=0o700)
        managed,installed=seed(root/'data');external=root/'external-tool';external_bytes=b'synthetic external immutable';external.write_bytes(external_bytes)
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
            click('도구');click('설치 목록 갱신');wait(lambda:enabled_node(env,address,'다운로드에 이 관리 도구 사용'),app)
            screenshot(a.output/'inventory.png','dlpad',display)
            if a.case=='select-remove-use':
                click('이 버전 활성화');wait(lambda:active(root/'data')==('synthetic-v2',TARGET),app)
                screenshot(a.output/'selected.png','dlpad',display)
                click('보관 버전 제거');wait(lambda:not installed['synthetic-v1'][0].parent.exists(),app)
                assert active(root/'data')==('synthetic-v2',TARGET)
                assert installed['synthetic-v2'][0].read_bytes()==installed['synthetic-v2'][1]
                assert enabled_node(env,address,'이 버전 활성화') is None and enabled_node(env,address,'보관 버전 제거') is None
                click('다운로드에 이 관리 도구 사용')
                time.sleep(.5);r['tree_after_use']=accessibility(env)
                # Path must appear in the download field tree, not merely the tools inventory.
                click('다운로드');tree=accessibility(env);r['download_tree']=tree
                expected=str(installed['synthetic-v2'][0]);r['verified_path_visible']=any("Name': <'기존 yt-dlp 실행 파일'>" in n['properties']['stdout'] and 'uint32 79' in n['role']['stdout'] and n.get('text',{}).get('exit')==0 and ast.literal_eval(n['text']['stdout'])[0]==expected for n in tree.get('nodes',[]))
                if not r['verified_path_visible']:raise RuntimeError('verified path absent from download tree')
                r.update(inactive_selected=True,inactive_removed=True,active_preserved=True)
            elif a.case=='closewait':
                lock=sqlite3.connect(managed/'activation.sqlite',timeout=.1);lock.execute('BEGIN EXCLUSIVE')
                click('설치 목록 갱신');time.sleep(.2);owner=OwnedDisplay(display);owner.delete_window(owner.window())
                click('중단 후 종료');time.sleep(.15);r['alive_while_lock_held']=app.poll() is None
                if not r['alive_while_lock_held']:raise RuntimeError('CloseWait exited before owned worker boundary')
                lock.rollback();lock.close();lock=None;started=time.monotonic();app.wait(timeout=8);r['closewait_exit_seconds']=time.monotonic()-started
                if app.returncode:raise RuntimeError('CloseWait nonzero exit')
            r['external_bytes_preserved']=external.read_bytes()==external_bytes
            assert r['external_bytes_preserved'];r['passed']=True
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
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--case',choices=('inspect','select-remove-use','closewait'),default='inspect');p.add_argument('--validate-only',action='store_true');a=p.parse_args()
    if not re.fullmatch('[0-9a-f]{64}',a.expected_sha):p.error('expected SHA256 required')
    if a.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux only')
    a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
    return run(a)

if __name__=='__main__':raise SystemExit(main())
