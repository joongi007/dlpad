#!/usr/bin/env python3
"""CPU plan preview on owned Xvfb/private bus/temp volume. No external network or clipboard."""
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
    with tempfile.TemporaryDirectory(prefix='cpu-plan-owned-') as tmp:
        root=Path(tmp)
        for name in ('data','media','config','cache','runtime','xdg-data'):(root/name).mkdir(mode=0o700)
        media=root/'media'/"literal space $quoted ' path 한글";media.mkdir()
        tool=root/'never-executed-tool';tool.write_text('#!/bin/sh\ntouch '+str(root/'EXECUTED')+'\n');tool.chmod(0o700)
        expected_free=os.statvfs(root/'media');expected_free=expected_free.f_bavail*expected_free.f_frsize
        r['statvfs_available_bytes_before']=expected_free
        env=dict(os.environ)
        for key in ('WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE','DLPAD_YTDLP'):env.pop(key,None)
        env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(media),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
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
            r['bus_identity']=process_identity(bus.pid);r['app_identity']=process_identity(app.pid);time.sleep(2);focus_owned_window(display);geometry=screenshot(a.output/'initial.png','dlpad',display)
            def click(name):
                position,node=wait(lambda:enabled_node(env,address,name),app)
                click_at(display,geometry,*position);time.sleep(.2);return node
            from subtitle_ui_probe import key_input
            from cpu_options_probe import lookup, wheel
            owner=OwnedDisplay(display)
            def widget(name,role=None):
                for turn in range(16):
                    try:return lookup(env,address,name,role)
                    except RuntimeError:wheel(owner,3 if turn<8 else -3)
                raise RuntimeError('bounded scroll failed: '+name)
            def click(name):
                position,node=widget(name,'action')
                if name.startswith('실행 계획 미리보기'):click_at(display,geometry,*position)
                else:
                    reply=call(env,node['destination'],node['path'],'org.a11y.atspi.Action.DoAction','0',address=address);assert reply['exit']==0
                time.sleep(1)
            def field(name,value):
                position,_=widget(name,'79');key_input(display,geometry,position,value);time.sleep(.2)
            def texts():
                return [ast.literal_eval(n['text']['stdout'])[0] for n in accessibility(env).get('nodes',[]) if n.get('text',{}).get('exit')==0]
            def any_node(name):
                pairs=lambda value:re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",value)
                reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
                queue=pairs(reply['stdout']);seen=set()
                while queue and len(seen)<256:
                    dest,path=queue.pop(0)
                    if (dest,path) in seen:continue
                    seen.add((dest,path))
                    props=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
                    if "Name': <'"+name+"'>" in props['stdout']:
                        text=call(env,dest,path,'org.a11y.atspi.Text.GetText','0','-1',address=address)
                        if text['exit']==0:return {'destination':dest,'path':path,'text':text}
                    children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address);queue.extend(pairs(children['stdout']))
                return None
            def plan():
                node=any_node('실행 계획 내용')
                return json.loads(ast.literal_eval(node['text']['stdout'])[0]) if node else None
            def reach_plan():
                for _ in range(8):
                    value=plan()
                    if value:return value
                    wheel(owner,3)
                raise RuntimeError('bounded preview text not found')
            field('영상 URL','https://fixture.invalid/watch?token=fixture-secret#private')
            click('영상 높이 제한');field('영상 높이 제한','720')
            field('시작 (비우면 전체 영상)','1.001');field('끝, HH:MM:SS.mmm','10.123')
            click('고급 옵션 펼치기');field('속도 제한','1048576');field('재시도','2');field('조각 동시성','4');click('고급 옵션 접기')
            click('실행 계획 미리보기 (옵션 버전은 검증되지 않음)')
            screenshot(a.output/'opened-plan.png','dlpad',display)
            initial=reach_plan();r['initial_argv']=initial
            assert initial[initial.index('--format')+1]=='bv*[height<=720]+ba/b[height<=720]'
            assert initial[initial.index('--download-sections')+1]=='*1.001-10.123'
            assert initial[initial.index('--paths')+1]==str(media)
            assert not any('fixture-secret' in x or 'fixture.invalid' in x or '#private' in x for x in initial)
            assert initial[-2]=='--' and initial[-1]=='[원본 URL 생략]';r['url_redacted']=True
            for key,value in [('--limit-rate','1048576'),('--retries','2'),('--concurrent-fragments','4')]:assert initial[initial.index(key)+1]==value
            r['advanced_preserved']=True
            r['page_boundary_states']={}
            for name in ['이전 페이지','다음 페이지']:
                _,node=widget(name,'action');reply=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                bits=int(re.search(r'(?:uint32 )?(\d+)',reply['stdout']).group(1));assert not bits&(1<<8) and not bits&(1<<24)
                r['page_boundary_states'][name]=reply
            field('영상 높이 제한','1080')
            reach_plan()
            changed=wait(lambda:(v if v and '1080' in v[v.index('--format')+1] else None) if (v:=plan()) else None,app)
            r['changed_argv']=changed;r['immediate_update']=True
            field('영상 높이 제한','143')
            widget('입력 옵션을 확인한 뒤 실행 계획을 확인하세요.')
            wait(lambda:lookup(env,address,'입력 옵션을 확인한 뒤 실행 계획을 확인하세요.'),app)
            assert plan() is None;r['invalid_fixed_error']=True
            field('영상 높이 제한','720');reach_plan();wait(plan,app)
            r['journal_created']=(root/'data/jobs.sqlite').exists();r['tool_executed']=(root/'EXECUTED').exists()
            assert not r['journal_created'] and not r['tool_executed']
            screenshot(a.output/'plan.png','dlpad',display)
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
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--case',choices=('preview',),default='preview');p.add_argument('--validate-only',action='store_true');a=p.parse_args()
    if not re.fullmatch('[0-9a-f]{64}',a.expected_sha):p.error('expected SHA256 required')
    if a.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux only')
    a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
    return run(a)

if __name__=='__main__':raise SystemExit(main())
