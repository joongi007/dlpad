#!/usr/bin/env python3
"""Current CPU native Hangul/quiet-reader regression; owned Xvfb/private buses only."""
import argparse, ast, json, os, re, select, signal, subprocess, tempfile, time
from pathlib import Path
from types import SimpleNamespace
from accessible_ime_probe import run as ime_run
from egui_ime_probe import accessibility, binary_hash, call
from ime_probe import focus_owned_window, key_sequence, stop_owned
from native_probe import probe_display, screenshot
from x11_diagnostics import click_at
from background_ui_probe import process_identity


def quiet_reader(a,output):
 r={'passed':False,'screen_reader_verified':False,'actual_audio_used':False,'user_bus_used':False,'user_desktop_used':False,'user_clipboard_used':False,'binary_sha256':binary_hash(a.binary)};app=reader=bus=None
 with tempfile.TemporaryDirectory(prefix='cpu-native-reader-owned-') as tmp:
  task=Path(tmp)
  for name in ['config','data','cache','runtime','app-data','media']:(task/name).mkdir(mode=0o700)
  authority=task/'runtime/owned.Xauthority';authority.write_bytes(b'');previous=os.environ.get('XAUTHORITY');os.environ['XAUTHORITY']=str(authority)
  try:
   with probe_display(output/'isolated',False) as (display,kind):
    if kind!='Xvfb isolated':raise RuntimeError('owned private Xvfb required')
    env=dict(os.environ)
    for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE','DLPAD_YTDLP','DLPAD_TEST_REPORT_STRESS']:env.pop(key,None)
    env.update(DISPLAY=display,XAUTHORITY=str(authority),XDG_CONFIG_HOME=str(task/'config'),XDG_DATA_HOME=str(task/'data'),XDG_CACHE_HOME=str(task/'cache'),XDG_RUNTIME_DIR=str(task/'runtime'),DLPAD_TEST_DATA=str(task/'app-data'),DLPAD_LIBRARY_ROOT=str(task/'media'),DLPAD_TEST_FONT=str(a.font),GSETTINGS_BACKEND='memory',LANG='C.UTF-8',LC_ALL='C.UTF-8')
    with (output/'bus.log').open('xb') as blog,(output/'app.log').open('xb') as alog,(output/'reader-stderr.log').open('xb') as rlog:
     bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(task/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True);r['bus_identity']=process_identity(bus.pid)
     if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('private bus readiness timeout')
     env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();r['registry_before_app']=accessibility(env);address=re.search(r"'([^']+)'",r['registry_before_app']['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
     reader_root=a.repo/'.local-tools/reader/root';renv=dict(env,PATH=str(reader_root/'usr/bin')+os.pathsep+env.get('PATH',''),PYTHONPATH=str(reader_root/'usr/lib/python3/dist-packages'),GI_TYPELIB_PATH=str(reader_root/'usr/lib/x86_64-linux-gnu/girepository-1.0'))
     runner="import resource,sys; resource.setrlimit(resource.RLIMIT_FSIZE,(2097152,2097152)); from orca import debug,orca; debug.debugLevel=debug.LEVEL_ALL; debug.eventDebugLevel=debug.LEVEL_ALL; debug.debugFile=open(sys.argv[2],'w',buffering=1); manager=orca.getSettingsManager(); manager.activate(sys.argv[1],dict(enableSpeech=False,enableBraille=False,enableSound=False,enableMouseReview=False)); sys.exit(orca.main())"
     reader=subprocess.Popen(['python3','-c',runner,str(task/'reader-prefs'),str(output/'reader.log')],env=renv,stdout=rlog,stderr=subprocess.STDOUT,start_new_session=True);r['reader_identity']=process_identity(reader.pid);time.sleep(2)
     if reader.poll() is not None:raise RuntimeError('owned quiet Orca unavailable; inspect reader-stderr.log')
     app=subprocess.Popen([str(a.binary),'--fixture','--settings'],env=env,stdin=subprocess.DEVNULL,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True);r['app_identity']=process_identity(app.pid);time.sleep(3)
     tree=accessibility(env)
     settings=next(n for n in tree['nodes'] if "Name': <'설정'>" in n['properties']['stdout'] and 'uint32 43,' in n['role']['stdout']);reply=call(env,settings['destination'],settings['path'],'org.a11y.atspi.Action.DoAction','0',address=address);assert reply['exit']==0 and 'true' in reply['stdout'];time.sleep(.25)
     tree=accessibility(env);node=next(n for n in tree['nodes'] if "Name': <'한글 입력 검증'>" in n['properties']['stdout'] and 'uint32 79,' in n['role']['stdout'])
     bounds=call(env,node['destination'],node['path'],'org.a11y.atspi.Component.GetExtents','0',address=address);x,y,w,h=ast.literal_eval(bounds['stdout'])[0]
     if not (w>0 and h>0 and 0<=x+w//2<960 and 65<=y+h//2<580):raise RuntimeError('named IME entry outside owned content viewport')
     r['input_extents']=[x,y,w,h];geometry=screenshot(output/'initial.png','dlpad',display);focus_owned_window(display);click_at(display,geometry,x+w//2,y+h//2)
     key_sequence(display,list('readerfixture'));time.sleep(.3);r['after_input']=accessibility(env)
     target=next(n for n in r['after_input']['nodes'] if "Name': <'한글 입력 검증'>" in n['properties']['stdout'] and 'uint32 79,' in n['role']['stdout']);r['entry_text']=ast.literal_eval(target['text']['stdout'])[0];assert r['entry_text']=='readerfixture'
     r['entry_state']=call(env,target['destination'],target['path'],'org.a11y.atspi.Accessible.GetState',address=address);match=re.search(r'(?:uint32 )?(\d+)',r['entry_state']['stdout']);r['entry_focused']=bool(int(match.group(1))&(1<<12));assert r['entry_focused'];screenshot(output/'input.png','dlpad',display)
     key_sequence(display,['Tab']);time.sleep(.3);r['after_tab']=accessibility(env);focused=[]
     for n in r['after_tab']['nodes']:
      reply=call(env,n['destination'],n['path'],'org.a11y.atspi.Accessible.GetState',address=address);m=re.search(r'(?:uint32 )?(\d+)',reply['stdout'])
      if m and int(m.group(1))&(1<<12):focused.append({'properties':n['properties'],'role':n['role'],'state':reply})
     r['tab_focused_nodes']=focused;assert focused;screenshot(output/'tab.png','dlpad',display)
     stop_owned(app);stop_owned(reader);stop_owned(bus)
  except Exception as error:r['error']=str(error) or type(error).__name__
  finally:
   for child in [app,reader,bus]:stop_owned(child)
   if previous is None:os.environ.pop('XAUTHORITY',None)
   else:os.environ['XAUTHORITY']=previous
   r['owned_children_reaped']=all(c is None or c.poll() is not None for c in [app,reader,bus]);r['reader_exit_code']=reader.poll() if reader else None
 log=output/'reader.log'
 if log.is_file():
  r['reader_log_bytes']=log.stat().st_size
  with log.open() as stream:text=stream.read(2097152)
  r['reader_registry_initialized']='ATSPI registry initialized' in text;r['reader_text_changed_event_processed']='PROCESS OBJECT EVENT object:text-changed:insert' in text;r['reader_keyboard_focus_processed']=bool(re.search(r'Changing locusOfFocus from .* to \[push button \| 다운로드\]',text));r['reader_event_processing_verified']=r['reader_registry_initialized'] and r.get('entry_text')=='readerfixture' and r['reader_text_changed_event_processed'] and r['reader_keyboard_focus_processed']
 r['after_binary_sha256']=binary_hash(a.binary);r['hash_stable']=r['binary_sha256']==r['after_binary_sha256'];r['passed']=not r.get('error') and r.get('reader_event_processing_verified',False) and r['owned_children_reaped'] and r['reader_exit_code']==0 and r['hash_stable'];(output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));return r


def run(a):
 def deadline(signum,frame):raise TimeoutError('owned native recovery deadline')
 signal.signal(signal.SIGALRM,deadline);signal.alarm(180)
 r={'binary_sha256':binary_hash(a.binary),'harness_sha256':binary_hash(Path(__file__)),'passed':False,'user_desktop_used':False,'user_clipboard_used':False,'actual_audio_used':False,'full_ime_verified':False,'full_screen_reader_verified':False};assert r['binary_sha256']==a.expected_sha
 if a.case in ['ime','both']:
  output=a.output/'ime';output.mkdir();deps=[a.repo/'.local-tools/ime/root/usr/bin/fcitx5',a.repo/'.local-tools/ime/root/usr/bin/fcitx5-remote',a.repo/'.local-tools/xvfb/usr/bin/proot'];r['ime_tools_available']=all(p.is_file() for p in deps)
  if r['ime_tools_available']:
   args=SimpleNamespace(binary=a.binary,output=output,font=a.font,input_point=None,inspect=False,ime_events=False,commit_fallback=False);code=ime_run(args);raw=json.loads((output/'result.json').read_text());r['ime_raw_exit']=code;r['ime_scoped_passed']=not raw.get('error') and all(raw.get(k,False) for k in ['commit_verified','cancel_verified','latin_transition_verified','selection_replace_verified','owned_children_reaped','hash_stable']) and raw.get('ascii_focus_text')=='abc';r['ime_preedit_visual_review_pending']=True
  else:r['ime_unverified_reason']='project-owned Fcitx/proot tools unavailable'
 if a.case in ['reader','both']:
  output=a.output/'reader';output.mkdir();r['reader_tool_available']=(a.repo/'.local-tools/reader/root/usr/lib/python3/dist-packages/orca/orca.py').is_file()
  if r['reader_tool_available']:result=quiet_reader(a,output);r['reader_scoped_passed']=result['passed']
  else:r['reader_unverified_reason']='project-owned quiet Orca modules unavailable'
 signal.alarm(0);r['after_binary_sha256']=binary_hash(a.binary);r['hash_stable']=r['binary_sha256']==r['after_binary_sha256'];r['passed']=r['hash_stable'] and (a.case=='reader' or r.get('ime_scoped_passed',False)) and (a.case=='ime' or r.get('reader_scoped_passed',False));(a.output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));print(json.dumps(r,ensure_ascii=False));return 0 if r['passed'] else 1


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--expected-sha',required=True);p.add_argument('--case',choices=['ime','reader','both'],default='both');p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True);a.repo=Path(__file__).resolve().parents[1];a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);return run(a)
if __name__=='__main__':raise SystemExit(main())
