#!/usr/bin/env python3
"""Managed runtime tools on owned Xvfb/private bus; synthetic process only, no network."""
import argparse, ast, json, os, re, select, sqlite3, subprocess, tempfile, threading, time
from pathlib import Path
from accessible_download_probe import file_hash, bounded_log
import hashlib
from managed_retained_probe import TARGET
from managed_tools_probe import active
from native_probe import probe_display, screenshot
from egui_ime_probe import accessibility, call
from background_ui_probe import reap, process_identity, OwnedDisplay
from ime_probe import focus_owned_window
from x11_diagnostics import click_at


PENDING='자동 설치 준비 중: 공급 검증과 라이선스 자료를 확인하고 있습니다. 기존 관리 버전은 검증 후 선택할 수 있습니다.'
VERSION_FAILED='실행 파일 버전이 고정 manifest와 다릅니다. 활성화하지 않았습니다.'
FAILED='도구 작업 실패·취소. 이전 활성 버전은 유지합니다. 경로·네트워크·공급 파일 검증 상태를 확인하세요.'
def seed_runtime(data,wrong):
    managed=data/'managed-tools';managed.mkdir(mode=0o700)
    (managed/'.dlpad-owner').write_bytes(b'dlpad-managed-tools-v1\n')
    installed={}
    for tool in ('deno','ffmpeg'):
        for version in ('1.0.0','2.0.0'):
            directory=managed/tool/version/TARGET;directory.mkdir(parents=True)
            members=[('deno','Executable')] if tool=='deno' else [('ffmpeg','Executable'),('ffprobe','Auxiliary')]
            components=[]
            for name,role in members:
                reported='0.0.0' if wrong and version=='2.0.0' else version
                output=(f'deno {reported} (stable, release, {TARGET})\nv8 15.0.245.2-rusty\ntypescript 6.0.3' if tool=='deno' else f'{name} version {reported} Copyright (c) 2000-2026 the FFmpeg developers')
                argv='--version' if tool=='deno' else '-version'
                contents=('#!/usr/bin/python3\nimport sys\nassert sys.argv[1:]=='+repr([argv])+'\nprint('+repr(output)+')\n').encode()
                path=directory/name;path.write_bytes(contents);path.chmod(0o700)
                components.append(dict(name=name,bytes=len(contents),sha256=hashlib.sha256(contents).hexdigest(),role=role))
                installed[(tool,version,name)]=(path,contents)
            notice=b'synthetic notice';(directory/'LICENSE').write_bytes(notice)
            components.append(dict(name='LICENSE',bytes=len(notice),sha256=hashlib.sha256(notice).hexdigest(),role='Notice'))
            package=dict(version=1,tool=tool,tool_version=version,target=TARGET,source_url='https://example.invalid/synthetic.zip',license='synthetic only',ownership='Managed',archive_bytes=123,archive_sha256='a'*64,components=components)
            manifest=dict(tool=tool,version=version,target=TARGET,source_url=package['source_url'],license=package['license'],ownership='Managed',sha256=components[0]['sha256'],executable=components[0]['name'])
            (directory/'manifest.json').write_text(json.dumps(manifest));(directory/'archive-package.json').write_text(json.dumps(package))
    with sqlite3.connect(managed/'activation.sqlite') as db:
        db.execute('CREATE TABLE active(tool TEXT PRIMARY KEY,version TEXT NOT NULL,target TEXT NOT NULL)')
        db.executemany('INSERT INTO active VALUES(?,?,?)',[(tool,'1.0.0',TARGET) for tool in ('deno','ffmpeg')])
    return managed,installed

def active_tool(data,tool):
    with sqlite3.connect((data/'managed-tools/activation.sqlite').as_uri()+'?mode=ro',uri=True,timeout=.1) as db:
        return db.execute('SELECT version,target FROM active WHERE tool=?',(tool,)).fetchone()

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
    with tempfile.TemporaryDirectory(prefix='cpu-runtime-tools-owned-') as tmp:
        root=Path(tmp)
        for name in ('data','media','config','cache','runtime','xdg-data'):(root/name).mkdir(mode=0o700)
        installed={}
        if a.case!='empty':managed,installed=seed_runtime(root/'data',a.case=='wrong-version')
        external=root/'external-tool';external_bytes=b'synthetic external immutable';external.write_bytes(external_bytes)
        env=dict(os.environ)
        for key in ('WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE','DLPAD_YTDLP','DLPAD_DENO','DLPAD_FFMPEG'):env.pop(key,None)
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
            from cpu_options_probe import wheel
            owner=OwnedDisplay(display)
            def lookup_runtime(name,role=None,require_enabled=False):
                pairs=lambda value:re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",value)
                reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
                queue=pairs(reply['stdout']);seen=set()
                while queue and len(seen)<256:
                    dest,path=queue.pop(0)
                    if (dest,path) in seen:continue
                    seen.add((dest,path))
                    props=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
                    if "Name': <'"+name+"'>" in props['stdout']:
                        kind=call(env,dest,path,'org.a11y.atspi.Accessible.GetRole',address=address)
                        actions=call(env,dest,path,'org.a11y.atspi.Action.GetNActions',address=address)
                        match=re.search(r'(?:int32 |uint32 )?(\d+)',actions['stdout'])
                        actual_role=re.search(r'uint32 (\d+)',kind['stdout'])
                        native_button=kind['exit']==0 and actual_role and int(actual_role.group(1))==43
                        matches=role is None or (role=='79' and re.search(r'uint32 79[,)]',kind['stdout'])) or (role=='action' and (native_button or (actions['exit']==0 and match and int(match.group(1))>0)))
                        bounds=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address)
                        if matches and bounds['exit']==0:
                            x,y,w,h=ast.literal_eval(bounds['stdout'])[0]
                            state=call(env,dest,path,'org.a11y.atspi.Accessible.GetState',address=address)
                            state_match=re.search(r'(?:uint32 )?(\d+)',state['stdout'])
                            enabled=state['exit']==0 and state_match and int(state_match.group(1))&(1<<8)
                            if w>0 and h>0 and 0<=x+w//2<960 and 0<=y+h//2<640 and (not require_enabled or enabled):
                                return (x+w//2,y+h//2),dict(destination=dest,path=path,role=kind,actions=actions,bounds=[x,y,w,h],state=state)
                    children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address)
                    queue.extend(pairs(children['stdout'])[:256-len(seen)])
                raise RuntimeError('owned runtime widget unavailable: '+name)
            def widget(name,role=None):
                for turn in range(16):
                    try:return lookup_runtime(name,role)
                    except RuntimeError:wheel(owner,3 if turn<8 else -3)
                raise RuntimeError('bounded widget lookup failed: '+name)
            def click(name):
                position,node=widget(name,'action')
                def fresh_enabled():
                    nonlocal position,node
                    try:position,node=lookup_runtime(name,'action',require_enabled=True)
                    except RuntimeError:
                        r['last_action_wait']=dict(name=name,reason='no enabled matching action')
                        return False
                    state=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                    if state['exit']:raise RuntimeError('runtime state query failed: '+name)
                    bits=int(re.search(r'(?:uint32 )?(\d+)',state['stdout']).group(1))
                    r['last_action_wait']=dict(node, name=name, state=state)
                    return bool(bits&(1<<8))
                try:wait(fresh_enabled,app)
                except RuntimeError as error:raise RuntimeError('runtime action did not become enabled: '+name) from error
                r.setdefault('action_observations',[]).append(dict(name=name,**node))
                click_at(display,geometry,*position);time.sleep(.25)
            def texts():
                return [ast.literal_eval(n['text']['stdout'])[0] for n in accessibility(env).get('nodes',[])[:256] if n.get('text',{}).get('exit')==0]
            def install_state():
                _,node=widget('고정 버전 설치','action')
                state=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                assert state['exit']==0
                bits=int(re.search(r'(?:uint32 )?(\d+)',state['stdout']).group(1))
                return bool(bits&(1<<8)),state
            click('도구');r['tool_observations']=[]
            if a.case=='empty':
                for label in ('yt-dlp','Deno','FFmpeg·ffprobe','yt-dlp'):
                    click(label);click('설치 목록 갱신')
                    wait(lambda:enabled_node(env,address,'설치 목록 갱신'),app)
                    wait(lambda:'앱이 관리하는 설치가 없습니다' in texts(),app)
                    enabled,state=install_state();current=texts()
                    runtime=label!='yt-dlp'
                    assert enabled!=runtime
                    assert (PENDING in current)==runtime
                    assert FAILED not in current and VERSION_FAILED not in current
                    r['tool_observations'].append(dict(tool=label,install_state=state,pending=runtime))
                    screenshot(a.output/('empty-'+str(len(r['tool_observations']))+'.png'),'dlpad',display)
            else:
                for tool,label,field in [('deno','Deno','Deno 실행 파일 (선택 사항)'),('ffmpeg','FFmpeg·ffprobe','기존 FFmpeg 실행 파일 (선택)')]:
                    click(label);click('설치 목록 갱신');click('이 버전 활성화')
                    if a.case=='wrong-version':
                        wait(lambda:VERSION_FAILED in texts(),app)
                        assert active_tool(root/'data',tool)==('1.0.0',TARGET)
                        r['tool_observations'].append(dict(tool=tool,wrong_version_rejected=True))
                    else:
                        wait(lambda:active_tool(root/'data',tool)==('2.0.0',TARGET),app)
                        click('다운로드에 이 관리 도구 사용');click('다운로드')
                        _,entry=widget(field,'79')
                        reply=call(env,entry['destination'],entry['path'],'org.a11y.atspi.Text.GetText','0','-1',address=address)
                        assert reply['exit']==0
                        actual=ast.literal_eval(reply['stdout'])[0]
                        expected=str(installed[(tool,'2.0.0',tool)][0]);assert actual==expected
                        r['tool_observations'].append(dict(tool=tool,field=field,path=actual))
                        screenshot(a.output/(tool+'-used.png'),'dlpad',display);click('도구')
                for path,contents in installed.values():assert path.read_bytes()==contents
            r['external_bytes_preserved']=external.read_bytes()==external_bytes
            assert r['external_bytes_preserved'];r['passed']=True
            if app.poll() is None:screenshot(a.output/'final.png','dlpad',display)
        except Exception as error:
            r['error']=str(error)
            if context:
                try:
                    raw=accessibility(env)
                    payload=json.dumps(raw,ensure_ascii=False,indent=2)
                    if len(payload.encode())<=1024*1024:(a.output/'failure-atspi.json').write_text(payload+'\n')
                    else:r['failure_atspi_over_limit']=True
                except Exception as capture_error:r['failure_atspi_capture_error']=type(capture_error).__name__
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
    payload=json.dumps(r,ensure_ascii=False,indent=2)+'\n'
    if len(payload.encode())>65536:raise RuntimeError('bounded result size exceeded')
    (a.output/'result.json').write_text(payload);return 0 if r['passed'] else 1


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--case',choices=('empty','select-use','wrong-version'),default='empty');p.add_argument('--validate-only',action='store_true');a=p.parse_args()
    if not re.fullmatch('[0-9a-f]{64}',a.expected_sha):p.error('expected SHA256 required')
    if a.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux only')
    a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
    return run(a)

if __name__=='__main__':raise SystemExit(main())
