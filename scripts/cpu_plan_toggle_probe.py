#!/usr/bin/env python3
"""CPU plan header toggle on owned Xvfb/private bus/temp volume. No external network or clipboard."""
import argparse, ast, hashlib, json, os, re, select, sqlite3, subprocess, tempfile, threading, time
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


def state_bits(reply):
    if reply['exit'] != 0:
        raise RuntimeError('native state unavailable')
    words = re.search(r'\[(?:uint32 )?(\d+), (?:uint32 )?(\d+)\]', reply['stdout'])
    if not words or any(int(word) > 0xffffffff for word in words.groups()):
        raise RuntimeError('invalid native state array')
    return int(words.group(1)) | (int(words.group(2)) << 32)


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
    r={'case':a.case,'binary_sha256':file_hash(a.binary),'passed':False,'keyboard_verified':False,'actual_media_used':False,'network_install_started':False,'desktop_fallback_used':False}
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
                    try:
                        position,node=lookup(env,address,name,role)
                        if position[1]>530:raise RuntimeError('covered by fixed footer')
                        return position,node
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
                candidates=[]
                pairs=lambda value:re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",value)
                tree=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
                queue=pairs(tree['stdout']);seen=set()
                while queue and len(seen)<256:
                    dest,path=queue.pop(0)
                    if (dest,path) in seen:continue
                    seen.add((dest,path))
                    props=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
                    children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address)
                    queue.extend(pairs(children['stdout']))
                    if "Name': <'실행 계획 내용'>" not in props['stdout']:continue
                    if len(candidates)>=8:raise RuntimeError('plan candidate bound exceeded')
                    query=lambda method,*args:call(env,dest,path,method,*args,address=address)
                    item={'destination':dest,'path':path,
                          'role':query('org.a11y.atspi.Accessible.GetRole'),
                          'state':query('org.a11y.atspi.Accessible.GetState'),
                          'bounds':query('org.a11y.atspi.Component.GetExtents','0'),
                          'text':query('org.a11y.atspi.Text.GetText','0','-1')}
                    for reply in [item['role'],item['state'],item['bounds'],item['text']]:
                        if len(reply.get('stdout','').encode())>65536:raise RuntimeError('plan raw reply bound exceeded')
                    candidates.append(item)
                trace={'candidates':candidates,'status':'absent'}
                r.setdefault('plan_reads',[]).append(trace)
                if len(r['plan_reads'])>64:raise RuntimeError('plan observation bound exceeded')
                if not candidates:return None
                values=[]
                for item in candidates:
                    reply=item['text']
                    if reply['exit']!=0:item['parse_status']='text-unavailable';continue
                    try:
                        value=ast.literal_eval(reply['stdout'])[0]
                        if not isinstance(value,str):raise ValueError('native text is not string')
                        item['text_length']=len(value);item['text_sha256']=hashlib.sha256(value.encode()).hexdigest()
                        if not value.strip():item['parse_status']='pending-empty';continue
                        parsed=json.loads(value)
                        if not isinstance(parsed,list) or not parsed or not all(isinstance(arg,str) for arg in parsed):raise ValueError('invalid argv preview')
                        item['parse_status']='valid-json';values.append(parsed)
                    except (ValueError,TypeError,IndexError,SyntaxError) as error:
                        item['parse_status']='invalid';item['parse_error']=type(error).__name__
                if len(values)==1:trace['status']='valid-json';return values[0]
                trace['status']='pending-empty' if all(item['parse_status']=='pending-empty' for item in candidates) else 'invalid-or-unavailable'
                if len(values)>1:raise RuntimeError('ambiguous native plan text')
                raise RuntimeError('native plan text '+trace['status'])
            def reach_plan():
                for _ in range(8):
                    value=plan()
                    if value:return value
                    wheel(owner,3)
                raise RuntimeError('bounded preview text not found')
            field('영상 URL','https://fixture.invalid/watch?token=fixture-secret')
            header='실행 계획 미리보기 (옵션 버전은 검증되지 않음)'
            position,node=widget(header,'action')
            r['header_node']=node
            def native(method,*args):return call(env,node['destination'],node['path'],method,*args,address=address)
            r['header_properties']=native('org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible')
            r['header_role']=native('org.a11y.atspi.Accessible.GetRole')
            count_reply=native('org.freedesktop.DBus.Properties.Get','org.a11y.atspi.Action','NActions');r['action_count_reply']=count_reply
            count_match=re.search(r'(?:int32 )?(\d+)',count_reply['stdout'])
            count=int(count_match.group(1)) if count_reply['exit']==0 and count_match else 0
            if not 0<count<=8:
                r['action_scope']='unsupported action discovery; keyboard scope is separate'
                count=0
            else:
                r['action_scope']='discovered; activation results below'
            r['actions']=[]
            for i in range(count):
                r['actions'].append({'index':i,**{name:native('org.a11y.atspi.Action.'+method,str(i)) for name,method in [('name','GetName'),('description','GetDescription'),('binding','GetKeyBinding')]}})
            r['observations']=[]
            def observe(label):
                time.sleep(.6)
                observation={'label':label,'state':native('org.a11y.atspi.Accessible.GetState')}
                r['observations'].append(observation)
                bits=state_bits(observation['state'])
                observation.update(expandable=bool(bits&(1<<9)),expanded=bool(bits&(1<<10)),focused=bool(bits&(1<<12)))
                def coherent():
                    state=native('org.a11y.atspi.Accessible.GetState');current=state_bits(state)
                    observation['state']=state
                    try:value=plan()
                    except RuntimeError as error:
                        observation['plan_error']=str(error)
                        return None
                    opened=value is not None
                    if bool(current&(1<<10))!=opened:return None
                    observation['expanded_by_preview_presence']=opened
                    return (opened,)
                value=wait(coherent,app,timeout=3)
                observation['expanded']=bool(state_bits(observation['state'])&(1<<10))
                if a.require_expanded_state:
                    observation = r['observations'][-1]
                    reply = observation['state']
                    assert reply['exit'] == 0
                    bits = state_bits(reply)
                    observation['expandable'] = bool(bits & (1 << 9))
                    observation['expanded'] = bool(bits & (1 << 10))
                    assert observation['expandable']
                    assert observation['expanded'] == observation['expanded_by_preview_presence']

                screenshot(a.output/(label+'.png'),'dlpad',display)
                return r['observations'][-1]['expanded_by_preview_presence']
            assert not observe('initial')
            position,_=widget(header,'action');click_at(display,geometry,*position);assert observe('pointer-open')
            position,_=widget(header,'action');click_at(display,geometry,*position);assert not observe('pointer-close')
            r['action_results']=[]
            for i in range(count):
                reply=native('org.a11y.atspi.Action.DoAction',str(i));r['action_results'].append(reply)
                opened=observe('action-'+str(i))
                if not opened:
                    time.sleep(1);observe('action-delay-'+str(i))
                    # Move only in the owned display; no click. This yields a host paint event.
                    import ctypes as C,ctypes.util
                    t=C.CDLL(ctypes.util.find_library('Xtst'));t.XTestFakeMotionEvent.argtypes=[C.c_void_p,C.c_int,C.c_int,C.c_int,C.c_ulong]
                    t.XTestFakeMotionEvent(owner.display,0,500,100,0);owner.x.XFlush(owner.display)
                    observe('action-motion-'+str(i))
                r['action_results'][-1]['activation_verified']=reply['exit']==0 and plan() is not None
                if not r['action_results'][-1]['activation_verified']:
                    r['action_scope']='unsupported or ineffective action activation; not keyboard proof'
                if plan() is not None:
                    position,_=widget(header,'action');click_at(display,geometry,*position);assert not observe('reset-'+str(i))
            r['keyboard_observations']=[]
            for key in ['Return','space']:
                assert plan() is None, 'keyboard case must begin collapsed'
                focus=native('org.a11y.atspi.Component.GrabFocus')
                r['keyboard_observations'].append({'key':key,'grab_focus':focus})
                assert focus['exit']==0 and re.search(r'\(true,?\)',focus['stdout']), 'native GrabFocus failed or returned false'
                def focused():
                    reply=native('org.a11y.atspi.Accessible.GetState')
                    bits=state_bits(reply)
                    return reply if bits&(1<<12) else None
                focused_reply=wait(focused,app,timeout=3)
                r['keyboard_observations'][-1]['focused_state']=focused_reply
                assert state_bits(focused_reply)&(1<<9), 'header not expandable'
                assert not state_bits(focused_reply)&(1<<10), 'header not collapsed'
                key_sequence(display,[key]);assert observe('key-'+key), 'keyboard did not open preview'
                opened=native('org.a11y.atspi.Accessible.GetState')
                r['keyboard_observations'][-1]['opened_state']=opened
                assert state_bits(opened)&(1<<10), 'native expanded state missing after keyboard open'
                assert state_bits(opened)&(1<<12), 'header focus lost after keyboard open'
                key_sequence(display,[key]);assert not observe('key-close-'+key), 'keyboard did not close preview'
                closed=native('org.a11y.atspi.Accessible.GetState')
                r['keyboard_observations'][-1]['closed_state']=closed
                assert not state_bits(closed)&(1<<10), 'native expanded state remains after keyboard close'
                assert state_bits(closed)&(1<<12), 'header focus lost after keyboard close'
            r['keyboard_verified']=True
            r['journal_created']=(root/'data/jobs.sqlite').exists();r['tool_executed']=(root/'EXECUTED').exists()
            assert not r['journal_created'] and not r['tool_executed']
            owner=OwnedDisplay(display);started=time.monotonic();owner.delete_window(owner.window());time.sleep(.15)
            stop_node=enabled_node(env,address,'중단 후 종료') if app.poll() is None else None
            if stop_node:click_at(display,geometry,*stop_node[0])
            app.wait(timeout=8);r['close_seconds']=time.monotonic()-started;r['close_exit']=app.returncode
            assert app.returncode==0;r['expanded_state_verified']=a.require_expanded_state;r['passed']=True
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
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--case',choices=('preview',),default='preview');p.add_argument('--validate-only',action='store_true');p.add_argument('--require-expanded-state',action='store_true');a=p.parse_args()
    if not re.fullmatch('[0-9a-f]{64}',a.expected_sha):p.error('expected SHA256 required')
    if a.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux only')
    a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False)
    return run(a)

if __name__=='__main__':raise SystemExit(main())
