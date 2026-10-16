#!/usr/bin/env python3
"""CPU background lifetime on newly owned Xvfb/private bus, never user display."""
import argparse,ast,json,os,re,select,signal,subprocess,tempfile,threading,time
from pathlib import Path
from accessible_download_probe import TOOL,file_hash,wait,state,bounded_log,snapshot_database,pin_worker
from native_probe import probe_display,screenshot
from subtitle_ui_probe import key_input,job
from background_ui_probe import OwnedDisplay,reap,process_identity
from egui_ime_probe import accessibility,call
from cpu_options_probe import lookup
from cpu_library_probe import wheel
from ime_probe import focus_owned_window
from x11_diagnostics import click_at
CASES=['active-hidden','complete-hidden','stop-exit','unavailable']

def modal(env,address,name,expected_role=43):
 pairs=lambda text:re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",text)
 reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address);queue=pairs(reply['stdout']);seen=set()
 while queue and len(seen)<256:
  dest,path=queue.pop(0)
  if (dest,path) in seen:continue
  seen.add((dest,path));properties=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
  if "Name': <'"+name+"'>" in properties['stdout']:
   role=call(env,dest,path,'org.a11y.atspi.Accessible.GetRole',address=address)
   if 'uint32 '+str(expected_role)+',' in role['stdout']:
    bounds=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address)
    if bounds['exit']==0:
     x,y,w,h=ast.literal_eval(bounds['stdout'])[0]
     if w>0 and h>0 and 0<=x+w//2<960 and 0<=y+h//2<640:return (x+w//2,y+h//2),{'destination':dest,'path':path,'bounds':[x,y,w,h],'role':role}
  children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address);queue.extend(pairs(children['stdout']))
 raise RuntimeError('owned modal button unavailable: '+name)

def run_case(a,case):
 out=a.output/case;out.mkdir();r={'case':case,'binary_sha256':file_hash(a.binary),'passed':False,'user_display_used':False};app=bus=owner=context=secondary=None;threads=[];worker_fd=None
 if r['binary_sha256']!=a.expected_sha:raise RuntimeError('unexpected frozen executable')
 with tempfile.TemporaryDirectory(prefix='cpu-background-owned-') as tmp:
  root=Path(tmp)
  for n in ['data','media','config','cache','runtime','xdg-data']:(root/n).mkdir(mode=0o700)
  tool=root/'synthetic-tool';tool.write_text(TOOL);tool.chmod(0o700);(root/'control.json').write_text(json.dumps({'mode':'success','seconds':8 if case=='complete-hidden' else 120,'token':'owned-background'}))
  foreign=root/'data/activation.sock';foreign_identity=None
  if case=='unavailable':foreign.write_bytes(b'owned foreign-endpoint sentinel');foreign.chmod(0o600);foreign_identity=(foreign.stat().st_dev,foreign.stat().st_ino,file_hash(foreign))
  env=dict(os.environ)
  for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(key,None)
  env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
  try:
   context=probe_display(out/'isolated',False);display,kind=context.__enter__()
   if kind!='Xvfb isolated':raise RuntimeError('owned Xvfb required')
   env['DISPLAY']=display;bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
   t=threading.Thread(target=bounded_log,args=(bus.stderr,out/'bus.log'),daemon=True);t.start();threads.append(t)
   if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus readiness timeout')
   env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();tree=accessibility(env);address=re.search(r"'([^']+)'",tree['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
   app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True);t=threading.Thread(target=bounded_log,args=(app.stdout,out/'app.log'),daemon=True);t.start();threads.append(t);r['app_identity']=process_identity(app.pid);r['bus_identity']=process_identity(bus.pid)
   time.sleep(2);owner=OwnedDisplay(display);window=owner.window();g=screenshot(out/'initial.png','dlpad',display);focus_owned_window(display)
   def content(name,role):
    for turn in range(12):
     try:return lookup(env,address,name,role)[0]
     except RuntimeError:wheel(owner,3 if turn<6 else -3)
    raise RuntimeError('owned content control missing: '+name)
   def press_modal(name):p,node=modal(env,address,name);r.setdefault('modal_bounds',[]).append({'name':name,**node});click_at(display,g,*p);time.sleep(.2)
   key_input(display,g,content('영상 URL','79'),'https://fixture.invalid/watch');click_at(display,g,*content('다운로드 시작','action'));wait(lambda:(root/'worker.json').exists(),app);info,worker_fd=pin_worker(root,'owned-background');r['worker_identity']=info;r['job_before']=job(root/'data')
   _,url_node=lookup(env,address,'영상 URL','79');r['active_url_state']=call(env,url_node['destination'],url_node['path'],'org.a11y.atspi.Accessible.GetState',address=address);url_bits=int(re.search(r'(?:uint32 )?(\d+)',r['active_url_state']['stdout']).group(1));r['active_url_disabled_state_correct']=not url_bits&(1<<8) and not url_bits&(1<<24);assert r['active_url_disabled_state_correct']
   owner.delete_window(window);time.sleep(.3);screenshot(out/'close-prompt.png','dlpad',display)
   if case=='unavailable':
    _,node=modal(env,address,'백그라운드 계속');r['background_state']=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address);bits=int(re.search(r'(?:uint32 )?(\d+)',r['background_state']['stdout']).group(1));r['accessibility_disabled_state_correct']=not bits&(1<<8) and not bits&(1<<24);assert r['accessibility_disabled_state_correct']
    click_at(display,g,*modal(env,address,'백그라운드 계속')[0]);time.sleep(.3);r['disabled_click_remained_visible']=owner.map_state(window)==2;assert r['disabled_click_remained_visible'];assert process_identity(info['pid'])==(info['pid'],info['start_ticks'])
    _,r['unavailable_notice']=modal(env,address,'이 환경에서는 안전한 창 복귀를 준비하지 못했습니다.',29);screenshot(out/'unavailable.png','dlpad',display);press_modal('돌아가기');assert owner.map_state(window)==2;owner.delete_window(window);time.sleep(.2);press_modal('중단 후 종료')
   elif case=='stop-exit':press_modal('중단 후 종료')
   else:
    press_modal('백그라운드 계속');wait(lambda:owner.map_state(window)==0,app);r['window_hidden']=True;heartbeat=int((root/'heartbeat').read_text());time.sleep(1);r['heartbeat_before']=heartbeat;r['heartbeat_after']=int((root/'heartbeat').read_text());assert r['heartbeat_after']>heartbeat;r['same_worker_while_hidden']=process_identity(info['pid'])==(info['pid'],info['start_ticks']);assert r['same_worker_while_hidden']
    if case=='complete-hidden':r['downloaded_hidden']=wait(lambda:state(root/'data','Downloaded'),app,timeout=20)
    secondary=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True);r['secondary_identity']=process_identity(secondary.pid);t=threading.Thread(target=bounded_log,args=(secondary.stdout,out/'secondary.log'),daemon=True);t.start();threads.append(t);wait(lambda:secondary.poll() is not None,timeout=10);r['secondary_exit']=secondary.returncode;assert secondary.returncode==0;wait(lambda:owner.map_state(window)==2,app);r['restored_original_app']=process_identity(app.pid)==tuple(r['app_identity']);assert r['restored_original_app'];time.sleep(.6);screenshot(out/'restored.png','dlpad',display)
    current=job(root/'data');assert current['job']['id']==r['job_before']['job']['id'] and len(current['job']['attempts'])==1;r['job_after_restore']=current
    if case=='active-hidden':assert process_identity(info['pid'])==(info['pid'],info['start_ticks']);owner.delete_window(window);time.sleep(.2);press_modal('중단 후 종료')
    else:owner.delete_window(window)
   wait(lambda:app.poll() is not None,timeout=10);r['primary_exit']=app.returncode;assert app.returncode==0;wait(lambda:process_identity(info['pid'])!=(info['pid'],info['start_ticks']),timeout=5);r['worker_stopped']=True
   if foreign_identity:r['foreign_endpoint_preserved']=(foreign.stat().st_dev,foreign.stat().st_ino,file_hash(foreign))==foreign_identity;assert r['foreign_endpoint_preserved']
   r['passed']=True
  except Exception as error:
   r['error']=str(error) or type(error).__name__
   if owner and app and app.poll() is None:
    try:
     if owner.map_state(window)==2:screenshot(out/'failure.png','dlpad',env['DISPLAY'])
     else:r['failure_capture_skipped_unmapped']=True
     r['failure_tree']=accessibility(env)
    except Exception as nested:r['capture_error']=str(nested)
  finally:
   if secondary:reap(secondary)
   if app:reap(app)
   if worker_fd is not None:
    if hasattr(signal,'pidfd_send_signal') and process_identity(info['pid'])==(info['pid'],info['start_ticks']):
     signal.pidfd_send_signal(worker_fd,signal.SIGKILL)
     wait(lambda:process_identity(info['pid'])!=(info['pid'],info['start_ticks']),timeout=5)
    r['owned_worker_cleanup']=process_identity(info['pid'])!=(info['pid'],info['start_ticks'])
    os.close(worker_fd)
   if owner:owner.close()
   if bus:reap(bus)
   for t in threads:t.join(3)
   if context:context.__exit__(None,None,None)
   r['owned_cleanup']=all(p is None or p.poll() is not None for p in [app,bus,secondary]) and all(not t.is_alive() for t in threads);snapshot_database(root/'data',out/'jobs.sqlite')
   if foreign_identity:
    r['foreign_endpoint_preserved_after_cleanup']=foreign.exists() and (foreign.stat().st_dev,foreign.stat().st_ino,file_hash(foreign))==foreign_identity
    if not r['foreign_endpoint_preserved_after_cleanup']:r['passed']=False;r['foreign_cleanup_error']='existing fixture endpoint changed or missing'
 r['binary_sha256_after']=file_hash(a.binary);r['passed']=r['passed'] and r['owned_cleanup'] and r['binary_sha256_after']==r['binary_sha256'];(out/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));print(case,r['passed'],r.get('error'),flush=True);return r

def main():
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--expected-sha',required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));p.add_argument('--case',choices=CASES+['all'],default='all');a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);records=[run_case(a,c) for c in CASES if a.case=='all' or a.case==c];return 0 if all(r['passed'] for r in records) else 1
if __name__=='__main__':raise SystemExit(main())
