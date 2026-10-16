#!/usr/bin/env python3
"""Pinned public yt-dlp install/cancel via owned native CPU GUI and private Xvfb only."""
import argparse, ast, json, os, re, select, signal, sqlite3, subprocess, tempfile, threading, time
from pathlib import Path
from accessible_download_probe import file_hash, bounded_log
from managed_retained_probe import seed, TARGET
from managed_tools_probe import active
from native_probe import probe_display, screenshot
from egui_ime_probe import accessibility, call
from background_ui_probe import reap, process_identity, OwnedDisplay

VERSION='2026.08.19'
SIZE=40446224
SHA='58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a'
SOURCE='https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/yt-dlp_linux'


def wait(predicate, app, timeout=20):
 deadline=time.monotonic()+timeout
 while time.monotonic()<deadline:
  try:value=predicate()
  except sqlite3.OperationalError as error:
   if 'locked' not in str(error).lower():raise
   value=None
  if value:return value
  if app.poll() is not None:raise RuntimeError('owned GUI exited early')
  time.sleep(.01)
 raise TimeoutError('bounded owned observation timeout')


def nodes(env,address,name,role=43):
 reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
 pairs=lambda text:re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",text)
 queue=pairs(reply['stdout']);seen=set();found=[]
 while queue and len(seen)<128:
  dest,path=queue.pop(0)
  if (dest,path) in seen:continue
  seen.add((dest,path))
  properties=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
  if "Name': <'"+name+"'>" in properties['stdout']:
   observed=call(env,dest,path,'org.a11y.atspi.Accessible.GetRole',address=address)
   if f'uint32 {role},' in observed['stdout']:
    found.append((dest,path))
  children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address);queue.extend(pairs(children['stdout']))
 return found


def properties(env,address,node):
 dest,path=node
 reply=call(env,dest,path,'org.a11y.atspi.Accessible.GetState',address=address)
 match=re.search(r'(?:uint32 )?(\d+)',reply['stdout'])
 if reply['exit'] or not match:raise RuntimeError('owned accessible state unavailable')
 state=int(match.group(1));bounds=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address)
 if bounds['exit']:raise RuntimeError('owned accessible bounds unavailable')
 x,y,w,h=ast.literal_eval(bounds['stdout'])[0]
 return {'state':state,'enabled':bool(state&(1<<8)),'sensitive':bool(state&(1<<24)),'bounds':[x,y,w,h],'visible':w>0 and h>0 and 0<=x+w//2<960 and 0<=y+h//2<580}


def action(env,address,node):
 before=properties(env,address,node)
 if not before['enabled'] or not before['sensitive'] or not before['visible']:raise RuntimeError('owned action disabled/hidden')
 result=call(env,node[0],node[1],'org.a11y.atspi.Action.DoAction','0',address=address)
 if result['exit'] or 'true' not in result['stdout']:raise RuntimeError('owned native action rejected')
 return before


def run(a):
 def timeout(signum,frame):raise TimeoutError('owned official tools GUI deadline')
 signal.signal(signal.SIGALRM,timeout);signal.alarm(210)
 r={'case':a.case,'binary_sha256':file_hash(a.binary),'harness_sha256':file_hash(Path(__file__)),'passed':False,'user_desktop_used':False,'user_clipboard_used':False,'actual_media_used':False,'source_url':SOURCE,'expected_size':SIZE,'expected_sha256':SHA,'expected_version':VERSION};app=bus=context=owner=None;threads=[]
 if r['binary_sha256']!=a.expected_sha:raise RuntimeError('unexpected frozen binary')
 with tempfile.TemporaryDirectory(prefix='cpu-official-tools-owned-') as tmp:
  root=Path(tmp)
  for name in ('data','media','config','cache','runtime','xdg-data'):(root/name).mkdir(mode=0o700)
  managed,installed=seed(root/'data');external=root/'external-sentinel';external_bytes=b'owned synthetic external sentinel, never executed';external.write_bytes(external_bytes)
  env=dict(os.environ)
  for key in ('WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE','DLPAD_YTDLP','DLPAD_TEST_REPORT_STRESS','HTTP_PROXY','HTTPS_PROXY','ALL_PROXY','http_proxy','https_proxy','all_proxy'):env.pop(key,None)
  env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_TEST_FONT=str(a.font),DLPAD_YTDLP=str(external),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
  try:
   context=probe_display(a.output/'isolated',False);display,kind=context.__enter__()
   if kind!='Xvfb isolated':raise RuntimeError('private Xvfb required')
   env['DISPLAY']=display
   bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
   t=threading.Thread(target=bounded_log,args=(bus.stderr,a.output/'bus.log'),daemon=True);t.start();threads.append(t)
   if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus timeout')
   env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();tree=accessibility(env);address=re.search(r"'([^']+)'",tree['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
   app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True);t=threading.Thread(target=bounded_log,args=(app.stdout,a.output/'app.log'),daemon=True);t.start();threads.append(t)
   r['app_identity']=process_identity(app.pid);r['bus_identity']=process_identity(bus.pid);owner=OwnedDisplay(display);time.sleep(2)
   def click(name):
    candidates=wait(lambda:nodes(env,address,name),app)
    for node in candidates:
     observed=properties(env,address,node)
     if observed['enabled'] and observed['visible']:action(env,address,node);time.sleep(.15);return node
    raise RuntimeError('no enabled visible button: '+name)
   click('도구');click('설치 목록 갱신');time.sleep(.3)
   install=nodes(env,address,'고정 버전 설치')[0];cancel=nodes(env,address,'설치 중단 요청')[0]
   r['cancel_before']=properties(env,address,cancel);assert not r['cancel_before']['enabled'] and not r['cancel_before']['sensitive'];screenshot(a.output/'before-install.png','dlpad',display)
   r['install_action']=action(env,address,install);started=time.monotonic()
   official=managed/'yt-dlp'/VERSION/TARGET/'yt-dlp'
   if a.case=='cancel':
    def partial():
     for directory in managed.glob('.staging-*'):
      file=directory/'yt-dlp'
      if not file.is_symlink() and file.is_file():
       size=file.stat().st_size
       if 0<size<SIZE:return {'path':str(file),'bytes':size,'device':file.stat().st_dev,'inode':file.stat().st_ino}
     return None
    observed=wait(partial,app,120);r['partial_network_transfer']=observed;r['cancel_during']=action(env,address,cancel);r['cancel_request_seconds']=time.monotonic()-started
    wait(lambda:not properties(env,address,cancel)['enabled'],app,125);r['cancel_ack_seconds']=time.monotonic()-started
    assert not official.exists();assert active(root/'data')==('synthetic-v1',TARGET)
    assert not list(managed.glob('.staging-*'));r['cancel_no_publication']=True;r['cancel_staging_removed']=True;screenshot(a.output/'cancelled.png','dlpad',display)
   else:
    wait(lambda:official.is_file() and active(root/'data')==(VERSION,TARGET),app,130);wait(lambda:not properties(env,address,cancel)['enabled'],app,10)
    assert official.stat().st_size==SIZE and file_hash(official)==SHA
    manifest=json.loads((official.parent/'manifest.json').read_text());assert manifest['source_url']==SOURCE and manifest['sha256']==SHA and manifest['version']==VERSION
    r['installed_manifest']=manifest;r['installed_size']=official.stat().st_size;r['installed_sha256']=file_hash(official)
    version=subprocess.run([str(official),'--ignore-config','--no-plugin-dirs','--version'],env=env,capture_output=True,timeout=15);assert version.returncode==0 and version.stdout.strip().decode()==VERSION;r['version_stdout']=version.stdout.strip().decode();r['version_argv_flags']=['--ignore-config','--no-plugin-dirs','--version']
    r['active_remove_disabled']=properties(env,address,nodes(env,address,'보관 버전 제거')[0]);assert not r['active_remove_disabled']['enabled'] and not r['active_remove_disabled']['sensitive']
    screenshot(a.output/'installed.png','dlpad',display);click('다운로드에 이 관리 도구 사용');click('다운로드');node=nodes(env,address,'기존 yt-dlp 실행 파일',79)[0]
    text=call(env,node[0],node[1],'org.a11y.atspi.Text.GetText','0','-1',address=address);assert ast.literal_eval(text['stdout'])[0]==str(official);r['explicit_use_path']=str(official);screenshot(a.output/'used.png','dlpad',display)
    click('도구');click('이 버전 활성화');wait(lambda:active(root/'data')==('synthetic-v1',TARGET),app)
    click('보관 버전 제거');wait(lambda:not official.parent.exists(),app);assert active(root/'data')==('synthetic-v1',TARGET);r['inactive_official_removed']=True;screenshot(a.output/'removed.png','dlpad',display)
   for file,contents in installed.values():assert file.read_bytes()==contents
   assert external.read_bytes()==external_bytes;r['retained_versions_preserved']=True;r['external_sentinel_preserved']=True
   with sqlite3.connect(managed/'activation.sqlite') as db:assert db.execute('pragma integrity_check').fetchone()[0]=='ok';db.backup(sqlite3.connect(a.output/'activation.sqlite'))
   r['active_final']=active(root/'data');r['managed_entries']=[str(p.relative_to(managed)) for p in managed.rglob('*')][:128]
   owner.delete_window(owner.window());app.wait(timeout=10);assert app.returncode==0;r['close_exit']=app.returncode;r['passed']=True
  except Exception as error:
   r['error']=str(error) or type(error).__name__
   if context:
    try:screenshot(a.output/'failure.png','dlpad',display);r['failure_tree']=accessibility(env)
    except Exception as nested:r['capture_error']=str(nested)
  finally:
   if app:reap(app)
   if owner:owner.close()
   if bus:reap(bus)
   for t in threads:t.join(3)
   if context:context.__exit__(None,None,None)
   r['owned_cleanup']=all(p is None or p.poll() is not None for p in (app,bus)) and all(not t.is_alive() for t in threads)
 signal.alarm(0);r['binary_sha256_after']=file_hash(a.binary);r['passed']=r['passed'] and r['owned_cleanup'] and r['binary_sha256_after']==r['binary_sha256'];(a.output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));print(json.dumps({k:v for k,v in r.items() if k!='failure_tree'},ensure_ascii=False));return 0 if r['passed'] else 1


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',required=True,type=Path);p.add_argument('--output',required=True,type=Path);p.add_argument('--expected-sha',required=True);p.add_argument('--case',choices=['install','cancel'],required=True);p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);return run(a)
if __name__=='__main__':raise SystemExit(main())
