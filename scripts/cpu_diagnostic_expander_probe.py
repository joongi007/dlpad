#!/usr/bin/env python3
"""Native diagnostic/MPD expansion state on a small real failure, owned Xvfb only."""
import argparse, ast, json, os, re, select, signal, sqlite3, subprocess, tempfile, threading, time
from pathlib import Path
from cpu_report_stress_probe import widget, scroll_outer, state
from accessible_download_probe import TOOL, wait, bounded_log, snapshot_database, file_hash
from native_probe import probe_display, screenshot
from background_ui_probe import OwnedDisplay, reap, process_identity
from subtitle_ui_probe import key_input
from egui_ime_probe import accessibility, call
from ime_probe import focus_owned_window


def run(a):
 def deadline(signum,frame):raise TimeoutError('owned expander trial deadline')
 signal.signal(signal.SIGALRM,deadline);signal.alarm(150)
 r={'binary_sha256':file_hash(a.binary),'harness_sha256':file_hash(Path(__file__)),'passed':False,'user_desktop_used':False,'user_clipboard_used':False,'actual_media_used':False,'large_stress_fixture_used':False};app=bus=owner=context=None;threads=[]
 assert r['binary_sha256']==a.expected_sha
 with tempfile.TemporaryDirectory(prefix='cpu-expander-owned-') as tmp:
  root=Path(tmp)
  for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)
  tool=root/'synthetic-tool';tool.write_text(TOOL);tool.chmod(0o700);(root/'control.json').write_text(json.dumps({'mode':'failure','seconds':1,'token':'expander-owned'}))
  mpd=root/'source.mpd';mpd.write_text('<MPD><Period><Representation><BaseURL>https://fixture.invalid/?token=synthetic-secret</BaseURL><SegmentBase indexRange="0-100"><Initialization range="0-20"/></SegmentBase></Representation></Period></MPD>');r['mpd_source_sha256_before']=file_hash(mpd)
  env=dict(os.environ)
  for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE','DLPAD_TEST_REPORT_STRESS']:env.pop(key,None)
  env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
  try:
   context=probe_display(a.output/'isolated',False);display,kind=context.__enter__()
   if kind!='Xvfb isolated':raise RuntimeError('owned Xvfb required')
   env['DISPLAY']=display
   bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True);r['bus_identity']=process_identity(bus.pid)
   t=threading.Thread(target=bounded_log,args=(bus.stderr,a.output/'bus.log'),daemon=True);t.start();threads.append(t)
   if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus timeout')
   env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();registry=accessibility(env);address=re.search(r"'([^']+)'",registry['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address;r['private_bus_address']=address
   app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True);r['app_identity']=process_identity(app.pid)
   t=threading.Thread(target=bounded_log,args=(app.stdout,a.output/'app.log'),daemon=True);t.start();threads.append(t);owner=OwnedDisplay(display);time.sleep(2);geometry=screenshot(a.output/'initial.png','dlpad',display);focus_owned_window(display)
   def locate(name,role='43'):
    for turn in range(12):
     try:return widget(env,address,name,role,action=True)
     except RuntimeError:scroll_outer(owner,3 if turn<6 else -3,inner=turn>0 and name not in ['영상 URL','다운로드 시작','실패 원인 보기'])
    raise RuntimeError('owned visible widget unavailable: '+name)
   def click(name):
    dest,path=locate(name);status=call(env,dest,path,'org.a11y.atspi.Accessible.GetState',address=address);m=re.search(r'(?:uint32 )?(\d+)',status['stdout']);assert m and int(m.group(1))&(1<<8)
    reply=call(env,dest,path,'org.a11y.atspi.Action.DoAction','0',address=address);assert reply['exit']==0 and 'true' in reply['stdout'];time.sleep(.3)
   def field(name,text):
    dest,path=locate(name,'79');ext=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address);x,y,w,h=ast.literal_eval(ext['stdout'])[0];key_input(display,geometry,(x+w//2,y+h//2),text);time.sleep(.2)
   def header_state(name):
    dest,path=locate(name);value=call(env,dest,path,'org.a11y.atspi.Accessible.GetState',address=address);match=re.search(r'(?:uint32 )?(\d+)',value['stdout']);assert match
    bits=int(match.group(1));bounds=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address);return {'state':bits,'expandable':bool(bits&(1<<9)),'expanded':bool(bits&(1<<10)),'bounds':ast.literal_eval(bounds['stdout'])[0],'role':43,'address_path':[dest,path]}
   def toggle_sequence(name,prefix):
    sequence=[]
    for index,expanded in enumerate([False,True,False,True]):
     if index:click(name)
     observed=header_state(name);sequence.append(observed);assert observed['expandable'] and observed['expanded']==expanded,(name,observed,expanded)
     screenshot(a.output/(prefix+'-'+str(index)+'.png'),'dlpad',display)
    return sequence
   field('영상 URL','https://fixture.invalid/watch');click('다운로드 시작');before=wait(lambda:state(root/'data','Failed'),app);r['original_failed']=before
   r['diagnostic_header']=toggle_sequence('실패 원인 보기','diagnostic')
   field('추가할 로컬 MPD 경로 (최대 1 MiB)',str(mpd));click('구조만 첨부');time.sleep(.5)
   r['mpd_header']=toggle_sequence('MPD 0','mpd')
   r['failed_after_diagnostics']=state(root/'data','Failed');assert r['failed_after_diagnostics']==before;r['original_job_attempt_preserved']=True
   r['mpd_source_sha256_after']=file_hash(mpd);assert r['mpd_source_sha256_after']==r['mpd_source_sha256_before'];r['mpd_source_preserved']=True
   snapshot_database(root/'data',a.output/'jobs.sqlite');r['tree_final']=accessibility(env)
   owner.delete_window(owner.window());wait(lambda:app.poll() is not None,timeout=10);r['close_exit']=app.returncode;assert app.returncode==0;r['passed']=True
  except Exception as error:
   r['error']=str(error) or type(error).__name__
   if owner:
    try:screenshot(a.output/'failure.png','dlpad',display);r['tree_failure']=accessibility(env)
    except Exception as nested:r['capture_error']=str(nested)
  finally:
   if app:reap(app)
   if owner:owner.close()
   if bus:reap(bus)
   for t in threads:t.join(3)
   if context:context.__exit__(None,None,None)
   r['owned_cleanup']=all(p is None or p.poll() is not None for p in [app,bus]) and all(not t.is_alive() for t in threads)
   if not (a.output/'jobs.sqlite').is_file():snapshot_database(root/'data',a.output/'jobs.sqlite')
 signal.alarm(0);r['binary_sha256_after']=file_hash(a.binary);r['passed']=r['passed'] and r['owned_cleanup'] and r['binary_sha256_after']==r['binary_sha256'];(a.output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));print(json.dumps({k:v for k,v in r.items() if not k.startswith('tree_')},ensure_ascii=False));return 0 if r['passed'] else 1


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--expected-sha',required=True);p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);return run(a)
if __name__=='__main__':raise SystemExit(main())
