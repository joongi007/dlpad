#!/usr/bin/env python3
"""Bounded history observations on owned Xvfb, synthetic workers and temporary journal."""
import argparse,ast,json,os,re,select,sqlite3,subprocess,tempfile,time
from pathlib import Path
from cpu_options_probe import FIXTURE,lookup
from accessible_download_probe import wait,state,file_hash,snapshot_database,pin_worker
from native_probe import probe_display,screenshot
from background_ui_probe import OwnedDisplay,reap
from egui_ime_probe import accessibility,call
from cpu_library_close_probe import find_fast
from cpu_library_probe import wheel
from subtitle_ui_probe import key_input,job
from ime_probe import focus_owned_window
from x11_diagnostics import click_at

def run(a,case):
 out=a.output/case;out.mkdir();r={'case':case,'binary_sha256':file_hash(a.binary),'passed':False,'user_desktop_used':False,'clipboard_used':False,'actual_media_used':False};app=bus=owner=context=lock=None;worker_fds=[]
 with tempfile.TemporaryDirectory(prefix='cpu-history-owned-') as tmp:
  root=Path(tmp)
  for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)
  tool=root/'synthetic-tool';tool.write_text(FIXTURE);tool.chmod(0o700)
  def control(seconds):(root/'control.json').write_text(json.dumps({'mode':'success','seconds':seconds,'token':'owned-history'}))
  control(120)
  env=dict(os.environ)
  for k in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(k,None)
  env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
  try:
   context=probe_display(out/'isolated.json',False);display,kind=context.__enter__()
   if kind!='Xvfb isolated':raise RuntimeError('no desktop fallback')
   env['DISPLAY']=display
   with (out/'bus.log').open('xb') as blog,(out/'app.log').open('xb') as alog:
    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
    if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus deadline')
    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();tree=accessibility(env);address=re.search(r"'([^']+)'",tree['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
    app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True);time.sleep(2);owner=OwnedDisplay(display);focus_owned_window(display);geometry=screenshot(out/'initial.png','dlpad',display)
    def action(name):
     node=find_fast(env,address,name);reply=call(env,node['destination'],node['path'],'org.a11y.atspi.Action.DoAction','0',address=address)
     r.setdefault('actions',[]).append({'name':name,'reply':reply,'at':time.perf_counter()})
     if reply['exit'] or '(true' not in reply['stdout']:raise RuntimeError('action rejected '+name)
     time.sleep(.15)
    def widget(name,role):
     for turn in range(16):
      try:return lookup(env,address,name,role)
      except RuntimeError:wheel(owner,3 if turn<8 else -3)
     raise RuntimeError('bounded scroll did not reach '+name)
    def physical(name):
     p,node=widget(name,'action');r.setdefault('physical_controls',[]).append({'name':name,'point':p,'node':node,'at':time.perf_counter()});click_at(display,geometry,*p);time.sleep(.2)
    def field(name,value):p,_=widget(name,'79');key_input(display,geometry,p,value);time.sleep(.2)
    if case=='missing':
     action('작업 이력');time.sleep(.3);r['empty_node']=find_fast(env,address,'저장된 작업이 없습니다');r['journal_created']=(root/'data/jobs.sqlite').exists();assert not r['journal_created'];screenshot(out/'empty.png','dlpad',display)
    else:
     field('영상 URL','https://fixture.invalid/history');physical('영상 높이 제한');field('영상 높이 제한','720');physical('다운로드 시작');r['phase']='worker-start';wait(lambda:(root/'worker.json').is_file(),app);info,fd=pin_worker(root,'owned-history');worker_fds.append(fd);r['worker_identity']=info;r['before_pause']=job(root/'data');r['phase']='pause-action';action('일시 중단');paused=wait(lambda:state(root/'data','Paused'),app);r['paused']=paused;r['phase']='paused';original=paused['job']['id'];time.sleep(.3)
     # Actual editor changes must not replace the paused job's saved typed snapshot.
     physical('오디오 우선')
     if case=='corrupt':
      with sqlite3.connect(root/'data/jobs.sqlite') as db:db.execute('UPDATE download_jobs SET options_json=? WHERE id=?',('{}',original))
     action('작업 이력');time.sleep(.3);r['history_tree']=accessibility(env);screenshot(out/'paused-history.png','dlpad',display)
     if case=='corrupt':
      node=find_fast(env,address,'재개');r['resume_action_on_corrupt']=call(env,node['destination'],node['path'],'org.a11y.atspi.Action.DoAction','0',address=address);time.sleep(.4);current=job(root/'data');r['after_corrupt_action']=current
      assert current['job']['state']=='Paused' and len(current['job']['attempts'])==1
      assert len(json.loads((root/'argv.json').read_text()))==1;r['corrupt_snapshot_no_new_attempt']=True
     else:
      control(1.2);action('재개');completed=wait(lambda:state(root/'data','Downloaded'),app);r['resumed']=completed;assert completed['job']['id']==original and len(completed['job']['attempts'])==2 and completed['options']['quality']=={'Height':720}
      argv=json.loads((root/'argv.json').read_text());r['argv']=argv;assert len(argv)==2 and all(v[v.index('--format')+1]=='bv*[height<=720]+ba/b[height<=720]' for v in argv)
      # Synthetic journal rows are owned fixtures, not claims of actual downloads.
      with sqlite3.connect(root/'data/jobs.sqlite') as db:
       row=db.execute('SELECT job_json,snapshot,options_json FROM download_jobs WHERE id=?',(original,)).fetchone()
       for index in range(260):
        identifier=f'fixture-{index:05d}';saved=json.loads(row[0]);saved['id']=identifier;saved['state']='Cancelled';saved['attempts']=[];saved['artifacts']=[]
        db.execute('INSERT INTO download_jobs(id,revision,job_json,snapshot,options_json) VALUES(?,0,?,?,?)',(identifier,json.dumps(saved),row[1],row[2]))
      action('작업 이력');action('처음부터 새로 고침');time.sleep(.3);r['page1_tree']=accessibility(env);screenshot(out/'page1.png','dlpad',display)
      action('다음 페이지');time.sleep(.3);r['page2_tree']=accessibility(env);screenshot(out/'page2.png','dlpad',display)
      action('다음 페이지');time.sleep(.3);r['page3_tree']=accessibility(env);screenshot(out/'page3.png','dlpad',display)
      def text_contains(tree,value):return any(value in n['text']['stdout'] for n in tree['nodes'] if n['text']['exit']==0)
      assert text_contains(r['page1_tree'],'fixture-00000') and text_contains(r['page2_tree'],'fixture-00128') and text_contains(r['page3_tree'],'fixture-00256');r['synthetic_page_starts_verified']=True
      node=find_fast(env,address,'재개');before=job(root/'data');r['terminal_resume_action']=call(env,node['destination'],node['path'],'org.a11y.atspi.Action.DoAction','0',address=address);time.sleep(.3);assert job(root/'data')==before;r['terminal_no_new_attempt']=True
      # Named loading label is a busy proxy; disabled button Enabled bits are unreliable.
      lock=sqlite3.connect(root/'data/jobs.sqlite',timeout=.1);lock.execute('BEGIN EXCLUSIVE');action('처음부터 새로 고침');r['busy_loading_node']=find_fast(env,address,'작업 이력을 읽는 중');r['close_requested_at']=time.perf_counter();owner.delete_window(owner.window());time.sleep(.1);action('중단 후 종료');time.sleep(.15);r['alive_while_history_db_locked']=app.poll() is None;assert r['alive_while_history_db_locked'];released=time.perf_counter();lock.rollback();lock.close();lock=None;app.wait(timeout=8);r['close_after_release_seconds']=time.perf_counter()-released
    if app.poll() is None:owner.delete_window(owner.window());app.wait(timeout=8)
    r['close_exit']=app.returncode;assert app.returncode==0;r['passed']=True
  except Exception as error:
   r['error']=str(error) or type(error).__name__
   if owner:
    try:screenshot(out/'failure.png','dlpad',env['DISPLAY'])
    except Exception as nested:r['capture_error']=str(nested)
  finally:
   if lock:lock.rollback();lock.close()
   if app:reap(app)
   for fd in worker_fds:os.close(fd)
   if owner:owner.close()
   if bus:reap(bus)
   if context:context.__exit__(None,None,None)
   snapshot_database(root/'data',out/'jobs.sqlite');r['owned_cleanup']=all(p is None or p.poll() is not None for p in [app,bus])
 r['binary_sha256_after']=file_hash(a.binary);r['hash_stable']=r['binary_sha256_after']==r['binary_sha256'];r['passed']=r['passed'] and r['owned_cleanup'] and r['hash_stable'];(out/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n');print(case,r['passed'],r.get('error'),flush=True);return r

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,required=True);p.add_argument('--ready-sha',required=True);p.add_argument('--case',choices=['missing','resume-pages-close','corrupt','all'],default='all');p.add_argument('--validate-only',action='store_true');a=p.parse_args()
 if a.validate_only:print('valid; no processes started');return 0
 a.binary=a.binary.resolve(strict=True);a.font=a.font.resolve(strict=True)
 if file_hash(a.binary)!=a.ready_sha:p.error('READY hash mismatch')
 a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);cases=['missing','resume-pages-close','corrupt'] if a.case=='all' else [a.case];results=[run(a,c) for c in cases];return int(not all(r['passed'] for r in results))
if __name__=='__main__':raise SystemExit(main())
