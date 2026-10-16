#!/usr/bin/env python3
"""CPU typed options through owned AT-SPI bounds, synthetic argv and SQLite only."""
import argparse,ast,json,os,re,select,subprocess,tempfile,threading,time
from pathlib import Path
from accessible_download_probe import TOOL,file_hash,wait,state,bounded_log,snapshot_database,pin_worker
from native_probe import probe_display,screenshot
from subtitle_ui_probe import key_input,job
from background_ui_probe import OwnedDisplay,reap,process_identity
from egui_ime_probe import accessibility,call
from cpu_library_probe import wheel
from ime_probe import focus_owned_window,key_sequence
from x11_diagnostics import click_at
EXPECTED='2459ba126c6bcc13470b470c8a384d9d8ecef779893fd832411db0a78932f559'
CASES=['inspect','help','help-keyboard','best','height','audio','fast','accurate','subtitles','advanced','invalid-height','invalid-advanced','conflict','active-resume']
FIXTURE=TOOL.replace("cfg=json.loads((root/'control.json').read_text())", "cfg=json.loads((root/'control.json').read_text())\nlog=root/'argv.json';history=json.loads(log.read_text()) if log.exists() else [];assert len(history)<3;history.append(args);log.write_text(json.dumps(history))")
FIXTURE=FIXTURE.replace("p.write_bytes(b'synthetic candidate artifact')", "p.write_bytes(b'synthetic candidate artifact')\nif '--write-subs' in args:\n requested={}\n for language in args[args.index('--sub-langs')+1].split(','):\n  sidecar=p.with_suffix('.'+language+'.vtt');sidecar.write_text('WEBVTT synthetic');requested[language]={'filepath':str(sidecar),'ext':'vtt'}\n print('DLPAD_SUBTITLES:'+json.dumps({'id':'synthetic','requested_subtitles':requested}),flush=True)")

def lookup(env,address,name,role=None):
 pairs=lambda value:re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",value)
 reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
 queue=pairs(reply['stdout']);seen=set();candidate=None;footer_top=None
 while queue and len(seen)<256:
  dest,path=queue.pop(0)
  if (dest,path) in seen:continue
  seen.add((dest,path));properties=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
  if "Name': <'다운로드 시작'>" in properties['stdout']:
   extent=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address)
   if extent['exit']==0:
    fx,fy,fw,fh=ast.literal_eval(extent['stdout'])[0]
    if fw>0 and fh>0:footer_top=fy
  if "Name': <'"+name+"'>" in properties['stdout']:
   kind=call(env,dest,path,'org.a11y.atspi.Accessible.GetRole',address=address)
   matches=(role is None or (role=='action' and re.search(r'uint32 (43|7|62),',kind['stdout']) is not None) or (role!='action' and role in kind['stdout']))
   if matches:
    bounds=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address)
    if bounds['exit']==0:
     x,y,w,h=ast.literal_eval(bounds['stdout'])[0]
     actions=name in ['다운로드 시작','재개','작업 취소','일시 중단','이전 작업 다시 확인']
     if w>0 and h>0 and 0<=x+w//2<960 and 65<=y+h//2<630:candidate=((x+w//2,y+h//2),{'destination':dest,'path':path,'role':kind,'bounds':[x,y,w,h]},actions)
  children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address);queue.extend(pairs(children['stdout']))
 if candidate is not None:
  point,node,actions=candidate
  if actions or (footer_top is not None and node['bounds'][1]>=65 and node['bounds'][1]+node['bounds'][3]<footer_top-4):return point,node
 raise RuntimeError('owned widget unavailable or behind native footer: '+name+':'+str(role))

def run_case(a,case):
 out=a.output/case;out.mkdir();r={'case':case,'binary_sha256':file_hash(a.binary),'fixture_sha256':__import__('hashlib').sha256(FIXTURE.encode()).hexdigest(),'passed':False,'field_observations':[]};app=bus=owner=context=None;threads=[];workers=[]
 if r['binary_sha256']!=a.expected_sha256:raise RuntimeError('unexpected frozen executable')
 with tempfile.TemporaryDirectory(prefix='cpu-options-owned-') as tmp:
  root=Path(tmp)
  for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)
  tool=root/'synthetic-tool';tool.write_text(FIXTURE);tool.chmod(0o700)
  def control(seconds): (root/'control.json').write_text(json.dumps({'mode':'success','seconds':seconds,'token':'owned-options'}))
  control(120 if case=='active-resume' else 1.2)
  env=dict(os.environ)
  for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(key,None)
  env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
  try:
   context=probe_display(out/'isolated',False);display,kind=context.__enter__()
   if kind!='Xvfb isolated':raise RuntimeError('owned Xvfb required')
   env['DISPLAY']=display;bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
   t=threading.Thread(target=bounded_log,args=(bus.stderr,out/'bus.log'),daemon=True);t.start();threads.append(t)
   if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus timeout')
   env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();tree=accessibility(env);address=re.search(r"'([^']+)'",tree['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
   app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True);t=threading.Thread(target=bounded_log,args=(app.stdout,out/'app.log'),daemon=True);t.start();threads.append(t);r['app_identity']=process_identity(app.pid);r['bus_identity']=process_identity(bus.pid)
   time.sleep(2);owner=OwnedDisplay(display);g=screenshot(out/'initial.png','dlpad',display);focus_owned_window(display)
   def widget(name,role=None):
    for turn in range(16):
     try:
      p,n=lookup(env,address,name,role);r['field_observations'].append({'name':name,**n});return p,n
     except RuntimeError:wheel(owner,3 if turn<8 else -3)
    raise RuntimeError('owned widget not reached by bounded scroll: '+name)
   def click(name):p,_=widget(name,'action');click_at(display,g,*p);time.sleep(.2)
   def field(name,value):p,_=widget(name,'79');key_input(display,g,p,value);time.sleep(.2)
   if case!='inspect':field('영상 URL','https://fixture.invalid/watch')
   if case in ['height','invalid-height','active-resume']:
    click('영상 높이 제한');field('영상 높이 제한','143' if case=='invalid-height' else '720')
   if case=='audio':click('오디오 우선')
   if case in ['fast','accurate','conflict']:
    field('시작 (시작과 끝을 모두 비우면 전체 영상)','1.001');field('끝, HH:MM:SS.mmm','10.123')
    if case=='accurate':click('정확하게 (재인코딩)')
   if case in ['subtitles','conflict']:
    click('한국어와 영어');click('자동 생성 자막 허용')
   if case in ['advanced','invalid-advanced']:
    click('고급 옵션 펼치기');field('속도 제한','1048576');field('재시도','21' if case=='invalid-advanced' else '0');field('조각 동시성','16')
   screenshot(out/'configured.png','dlpad',display)
   if case in ['help','help-keyboard']:
    r['help_observations']=[]
    r['keyboard_verified']=False
    r['keyboard_scope']='GrabFocus then Return/space on five help headers; no Tab or screen-reader claim'
    catalog=(Path(__file__).resolve().parent.parent/'crates/app/src/catalog.rs').read_text()
    pairs=[('FFmpeg 선택 도움말','FFMPEG_CONFLICT_VERSION'),('세션 쿠키 선택 도움말','COOKIE_CONFLICT_VERSION'),('품질 선택 도움말','QUALITY_CONFLICT_VERSION'),('구간 입력 도움말','RANGE_CONFLICT_VERSION'),('자르기 방식 도움말','CUT_CONFLICT_VERSION')]
    for title,key in pairs:
     if case=='help-keyboard':
      _,node=widget(title,'action')
      query=lambda method,*args:call(env,node['destination'],node['path'],method,*args,address=address)
      def bits(reply):
       assert reply['exit']==0,'native state unavailable'
       words=re.search(r'\[(?:uint32 )?(\d+), (?:uint32 )?(\d+)\]',reply['stdout'])
       assert words and all(int(v)<=0xffffffff for v in words.groups()),'invalid native state array'
       return int(words.group(1))|(int(words.group(2))<<32)
      initial=query('org.a11y.atspi.Accessible.GetState');assert bits(initial)&(1<<9) and not bits(initial)&(1<<10)
      match=re.search(r'pub const '+key+r': &str =\s*("(?:[^"\\]|\\.)*");',catalog);assert match is not None;tail=json.loads(match.group(1))
      def tail_present():
       queue=[('org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root')];seen=set()
       while queue and len(seen)<256:
        dest,path=queue.pop(0)
        if (dest,path) in seen:continue
        seen.add((dest,path))
        props=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
        assert props['exit']==0,'tail absence properties unavailable'
        if "Name': <'"+tail+"'>" in props['stdout']:return True
        children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address)
        assert children['exit']==0,'tail absence children unavailable'
        queue.extend(re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",children['stdout']))
       if queue:raise RuntimeError('tail tree bound exceeded')
       return False
      observation={'title':title,'initial_state':initial,'keys':[]};r['help_observations'].append(observation)
      for native_key in ['Return','space']:
       widget(title,'action')
       focus=query('org.a11y.atspi.Component.GrabFocus');record={'key':native_key,'grab_focus':focus};observation['keys'].append(record)
       assert focus['exit']==0 and re.search(r'\(true,?\)',focus['stdout']),'GrabFocus failed or returned false'
       def focused():
        reply=query('org.a11y.atspi.Accessible.GetState');record['focused_state']=reply
        return bool(bits(reply)&(1<<12))
       wait(focused,child=app,timeout=3);assert not bits(record['focused_state'])&(1<<10)
       key_sequence(display,[native_key])
       def opened():
        reply=query('org.a11y.atspi.Accessible.GetState');record['opened_state']=reply
        return bool(bits(reply)&(1<<9) and bits(reply)&(1<<10) and bits(reply)&(1<<12))
       wait(opened,child=app,timeout=3)
       _,tail_node=widget(tail);record['tail_bounds']=tail_node['bounds'];record['tail_label']=tail
       screenshot(out/('help-keyboard-'+str(len(r['help_observations']))+'-'+native_key+'-open.png'),'dlpad',display)
       assert bits(query('org.a11y.atspi.Accessible.GetState'))&(1<<12),'focus lost while reaching tail'
       key_sequence(display,[native_key])
       def closed():
        reply=query('org.a11y.atspi.Accessible.GetState');record['closed_state']=reply
        return not bits(reply)&(1<<10) and bool(bits(reply)&(1<<12)) and not tail_present()
       wait(closed,child=app,timeout=3)
       screenshot(out/('help-keyboard-'+str(len(r['help_observations']))+'-'+native_key+'-closed.png'),'dlpad',display)
      continue
     click(title);_,node=widget(title,'action');reply=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address);bits=int(re.search(r'(?:uint32 )?(\d+)',reply['stdout']).group(1));assert bits&(1<<9) and bits&(1<<10)
     match=re.search(r'pub const '+key+r': &str =\s*("(?:[^"\\]|\\.)*");',catalog);assert match is not None;tail=json.loads(match.group(1));_,tail_node=widget(tail);assert tail_node['bounds'][3]>0
     screenshot(out/('help-'+str(len(r['help_observations']))+'.png'),'dlpad',display);r['help_observations'].append({'title':title,'state':reply,'tail_label':tail,'tail_bounds':tail_node['bounds'],'tree':accessibility(env)});click(title)
    assert job(root/'data') is None and not (root/'worker.json').exists() and not (root/'argv.json').exists()
    if case=='help-keyboard':r.update(keyboard_verified=True,job_created=False,worker_started=False,tool_executed=False)
   elif case=='inspect':r['tree']=accessibility(env)
   elif case.startswith('invalid-') or case=='conflict':
    click('다운로드 시작');time.sleep(1);r['rejected_job']=job(root/'data');r['worker_started']=(root/'worker.json').exists();assert r['rejected_job'] is None and not r['worker_started'];screenshot(out/'rejected.png','dlpad',display)
   else:
    click('다운로드 시작');wait(lambda:(root/'worker.json').is_file(),app);info,fd=pin_worker(root,'owned-options');workers.append(fd);r['worker_identity']=info
    if case=='active-resume':
     _,node=widget('영상 URL','79');r['active_url_state']=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address);bits=int(re.search(r'(?:uint32 )?(\d+)',r['active_url_state']['stdout']).group(1));assert not bits&(1<<8) and not bits&(1<<24);r['active_controls_disabled']=True;screenshot(out/'active.png','dlpad',display);before=job(root/'data');click('일시 중단');r['paused']=wait(lambda:state(root/'data','Paused'),app);time.sleep(.6);screenshot(out/'paused.png','dlpad',display);click('오디오 우선');control(1.2);click('재개');final=wait(lambda:state(root/'data','Downloaded'),app);assert final['job']['id']==before['job']['id'] and len(final['job']['attempts'])==2;assert final['options']['quality']=={'Height':720};r['resumed']=final
    else:r['downloaded']=wait(lambda:state(root/'data','Downloaded'),app)
    r['argv']=json.loads((root/'argv.json').read_text());(out/'argv.json').write_text(json.dumps(r['argv'],ensure_ascii=False,indent=2));options=(r.get('downloaded') or r['resumed'])['options'];r['typed_options']=options
    args=r['argv'][-1];selector=args[args.index('--format')+1]
    assert selector==('bestaudio/best' if case=='audio' else 'bv*[height<=720]+ba/b[height<=720]' if case in ['height','active-resume'] else 'bv*+ba/b')
    if case in ['fast','accurate']:assert args[args.index('--download-sections')+1]=='*1.001-10.123' and ('--force-keyframes-at-cuts' in args)==(case=='accurate')
    if case=='subtitles':assert args[args.index('--sub-langs')+1]=='ko,en' and '--write-auto-subs' in args
    if case=='advanced':assert args.count('--retries')==1 and args[args.index('--retries')+1]=='0' and args[args.index('--limit-rate')+1]=='1048576' and args[args.index('--concurrent-fragments')+1]=='16'
    if case=='active-resume':assert len(r['argv'])==2 and r['argv'][0][r['argv'][0].index('--format')+1]==selector
    time.sleep(.6);screenshot(out/'completed.png','dlpad',display)
   owner.delete_window(owner.window());wait(lambda:app.poll() is not None,timeout=10);assert app.returncode==0;r['close_exit']=app.returncode;r['passed']=True
  except Exception as error:
   r['error']=str(error) or type(error).__name__
   if owner:
    try:screenshot(out/'failure.png','dlpad',env['DISPLAY']);r['failure_tree']=accessibility(env)
    except Exception as nested:r['capture_error']=str(nested)
  finally:
   if app:reap(app)
   for fd in workers:os.close(fd)
   if owner:owner.close()
   if bus:reap(bus)
   for t in threads:t.join(3)
   if context:context.__exit__(None,None,None)
   r['owned_cleanup']=all(p is None or p.poll() is not None for p in [app,bus]) and all(not t.is_alive() for t in threads);snapshot_database(root/'data',out/'jobs.sqlite')
 r['binary_sha256_after']=file_hash(a.binary);r['passed']=r['passed'] and r['owned_cleanup'] and r['binary_sha256_after']==r['binary_sha256'];(out/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));print(case,r['passed'],r.get('error'),flush=True);return r

def main():
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));p.add_argument('--case',choices=CASES+['all'],default='inspect');p.add_argument('--expected-sha256',default=EXPECTED);a=p.parse_args();
 if re.fullmatch(r'[0-9a-f]{64}',a.expected_sha256) is None:p.error('expected SHA256 must be exactly 64 lowercase hex digits')
 a.binary=a.binary.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);results=[run_case(a,c) for c in CASES[1:] if a.case=='all' or c==a.case] if a.case!='inspect' else [run_case(a,'inspect')];return 0 if all(r['passed'] for r in results) else 1
if __name__=='__main__':raise SystemExit(main())
