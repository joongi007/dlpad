#!/usr/bin/env python3
"""Bounded S10 actual CPU GUI on owned Xvfb/private AT-SPI bus only."""
import ctypes as C,ctypes.util
from accessible_download_probe import Selection
import argparse,ast,hashlib,json,os,re,select,subprocess,tempfile,threading,time
from pathlib import Path
from accessible_download_probe import TOOL,wait,state,bounded_log,snapshot_database,file_hash,clipboard_preview
from native_probe import probe_display,screenshot
from background_ui_probe import OwnedDisplay,reap,process_identity
from subtitle_ui_probe import key_input
from x11_diagnostics import click_at
from cpu_library_probe import button,wheel
from egui_ime_probe import accessibility,call
from ime_probe import focus_owned_window,key_sequence

EXPECTED='8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986'
def run(a):
 r={'binary_sha256':file_hash(a.binary),'passed':False,'user_desktop_used':False,'user_clipboard_used':False};app=bus=owner=display_context=None;threads=[]
 if r['binary_sha256']!=EXPECTED:raise RuntimeError('unexpected frozen CPU snapshot')
 with tempfile.TemporaryDirectory(prefix='cpu-diag-owned-') as tmp:
  root=Path(tmp)
  for n in ['data','media','config','cache','runtime','xdg-data']:(root/n).mkdir(mode=0o700)
  tool=root/'synthetic-tool';fixture=TOOL.replace("if cfg['mode']=='failure':","if cfg['mode']=='failure':\n print('Synthetic bounded failure A',file=sys.stderr,flush=True)\n print('Synthetic bounded failure B',file=sys.stderr,flush=True)\n print('Synthetic bounded failure C',file=sys.stderr,flush=True)");r['fixture_sha256']=hashlib.sha256(fixture.encode()).hexdigest();tool.write_text(fixture);tool.chmod(0o700);(root/'control.json').write_text(json.dumps({'mode':'failure','seconds':1,'token':'diagnostic-owned'}))
  mpd=root/'source.mpd';mpd.write_text('<MPD><Period><AdaptationSet><Representation><BaseURL>https://private.invalid/?token=mpd-secret</BaseURL><SegmentBase indexRange="0-100"><Initialization range="0-20"/></SegmentBase></Representation></AdaptationSet></Period></MPD>')
  dtd=root/'dtd.mpd';dtd.write_text('<!DOCTYPE MPD [<!ENTITY secret "fixture">]><MPD/>');oversize=root/'oversize.mpd';oversize.write_bytes(b' '*(1048576+1))
  originals={p.name:file_hash(p) for p in [mpd,dtd,oversize]};r['source_hashes']=originals
  env=dict(os.environ)
  for n in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(n,None)
  env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),GSETTINGS_BACKEND='memory',WINIT_X11_SCALE_FACTOR='1')
  try:
   display_context=probe_display(a.output/'isolated',False)
   display,kind=display_context.__enter__()
   if True:
    if kind!='Xvfb isolated':raise RuntimeError('owned Xvfb required')
    env['DISPLAY']=display
    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    t=threading.Thread(target=bounded_log,args=(bus.stderr,a.output/'bus.log'),daemon=True);t.start();threads.append(t)
    if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus readiness timeout')
    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip();tree=accessibility(env);address=re.search(r"'([^']+)'",tree['bus_address_reply']['stdout']).group(1);env['AT_SPI_BUS_ADDRESS']=address
    app=subprocess.Popen([str(a.binary),'--fixture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True);t=threading.Thread(target=bounded_log,args=(app.stdout,a.output/'app.log'),daemon=True);t.start();threads.append(t);r['app_identity']=process_identity(app.pid);r['bus_identity']=process_identity(bus.pid)
    owner=OwnedDisplay(display);time.sleep(2);g=screenshot(a.output/'initial.png','dlpad',display);focus_owned_window(display)
    def point(name,role=None):
     for turn in range(12):
      try:return widget(env,address,name,role)
      except RuntimeError:wheel(owner,3 if turn<6 else -3)
     raise RuntimeError('widget outside owned scroll: '+name)
    def click(name):click_at(display,g,*point(name));time.sleep(.25)
    def field(name,text):key_input(display,g,point(name,'79'),text);time.sleep(.2)
    def shot(name):screenshot(a.output/(name+'.png'),'dlpad',display)
    def copy(name):
     click('검토한 자료 복사');text=clipboard_preview(display) if name.endswith('.md') else raw_clipboard(display)
     if len(text.encode())>2097152:raise RuntimeError('report cap exceeded')
     (a.output/name).write_text(text);return text
    field('영상 URL','https://fixture.invalid/watch');click('다운로드 시작');r['failed']=wait(lambda:state(root/'data','Failed'),app);time.sleep(.7);shot('failed');r['tree_failed']=accessibility(env)
    # Reports are a collapsing header. The actual label is discovered from AT-SPI.
    click('실패 원인 보기');shot('expanded');r['tree_expanded']=accessibility(env)
    if a.inspect:r['inspection_only']=True
    else:
     copy('initial.md');click('JSON');raw=copy('initial.json');initial=json.loads(raw);assert len(initial['logs'])>=3
     field('선택한 로그 수정','Authorization: Bearer new-secret https://private.invalid/?token=log-secret');click('정제 후 반영');raw=copy('edited.json');assert 'new-secret' not in raw and 'log-secret' not in raw;assert len(json.loads(raw)['logs'])==len(initial['logs'])-1
     field('추가할 로컬 MPD 경로 (최대 1 MiB)',str(mpd));click('구조만 첨부');time.sleep(.7);attached=json.loads(copy('attached.json'));assert len(attached['mpd_attachments'])==1;assert 'mpd-secret' not in json.dumps(attached);shot('attached');click('MPD 0');click('선택한 MPD 구조 제외');assert not json.loads(copy('omit-one-mpd.json'))['mpd_attachments'];click('구조만 첨부');time.sleep(.5);assert len(json.loads(copy('reattached.json'))['mpd_attachments'])==1
     for source in [dtd,oversize]:
      field('추가할 로컬 MPD 경로 (최대 1 MiB)',str(source));click('구조만 첨부');time.sleep(.5);shot(source.stem+'-failed');bad=json.loads(copy(source.stem+'-report.json'));assert len(bad['mpd_attachments'])==1 and bad['attachment_omissions']
     field('진단 자료 저장 경로',str(root/'export.json'));click('새 파일로 저장');wait(lambda:(root/'export.json').is_file(),app);export=(root/'export.json').read_bytes();(a.output/'export.json').write_bytes(export);click('새 파일로 저장');time.sleep(.5);assert (root/'export.json').read_bytes()==export;r['collision_preserved']=True
     before_omit=json.loads(copy('before-omit.json'));click('선택한 로그 제외');assert len(json.loads(copy('omit-log.json'))['logs'])==len(before_omit['logs'])-1;click('로그 제외');assert not json.loads(copy('omit-all-logs.json'))['logs'];click('MPD 구조 모두 제외');assert not json.loads(copy('omit-all-mpd.json'))['mpd_attachments'];shot('omitted')
     r['sources_preserved']=all(file_hash(root/name)==digest for name,digest in originals.items());assert r['sources_preserved']
    owner.delete_window(owner.window());wait(lambda:app.poll() is not None,timeout=10);r['closewait_exit']=app.returncode;assert app.returncode==0;r['passed']=True
  except Exception as e:
   r['error']=str(e) or type(e).__name__
   if owner:
    try:screenshot(a.output/'failure.png','dlpad',env['DISPLAY']);r['tree_failure']=accessibility(env)
    except Exception as nested:r['capture_error']=str(nested)
  finally:
   if app:reap(app)
   if owner:owner.close()
   if bus:reap(bus)
   for t in threads:t.join(3)
   if display_context:display_context.__exit__(None,None,None)
   r['owned_channels_reaped']=all(not t.is_alive() for t in threads) and all(p is None or p.poll() is not None for p in [app,bus]);snapshot_database(root/'data',a.output/'jobs.sqlite')
 r['binary_sha256_after']=file_hash(a.binary);r['passed']=r['passed'] and r['owned_channels_reaped'] and r['binary_sha256_after']==r['binary_sha256'];(a.output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2));print(json.dumps({k:v for k,v in r.items() if not k.startswith('tree_')},ensure_ascii=False));return 0 if r['passed'] else 1

def widget(env,address,name,role=None):
 reply=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
 pairs=lambda text:re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",text)
 queue=pairs(reply['stdout']);seen=set()
 while queue and len(seen)<128:
  dest,path=queue.pop(0)
  if (dest,path) in seen:continue
  seen.add((dest,path))
  properties=call(env,dest,path,'org.freedesktop.DBus.Properties.GetAll','org.a11y.atspi.Accessible',address=address)
  if "Name': <'"+name+"'>" in properties['stdout']:
   observed=call(env,dest,path,'org.a11y.atspi.Accessible.GetRole',address=address)
   if role is None or role in observed['stdout']:
    bounds=call(env,dest,path,'org.a11y.atspi.Component.GetExtents','0',address=address)
    if bounds['exit']==0:
     x,y,w,h=ast.literal_eval(bounds['stdout'])[0]
     top_level=name in ['영상 URL','다운로드 시작','실패 원인 보기']
     visible=w>0 and h>0 and 0<=x+w//2<960 and (0<=y+h//2<640 if top_level else 310<=y+h//2<590)
     if visible:
      return (x+w//2,y+h//2)
  children=call(env,dest,path,'org.a11y.atspi.Accessible.GetChildren',address=address)
  queue.extend(pairs(children['stdout']))
 raise RuntimeError('owned accessible widget unavailable: '+name)

def raw_clipboard(display_name):
    """Read only the new owned Xvfb CLIPBOARD, with a 256 KiB cap."""
    x=C.CDLL(ctypes.util.find_library('X11'))
    x.XOpenDisplay.argtypes=[C.c_char_p];x.XOpenDisplay.restype=C.c_void_p
    x.XCloseDisplay.argtypes=[C.c_void_p];x.XDefaultRootWindow.argtypes=[C.c_void_p];x.XDefaultRootWindow.restype=C.c_ulong
    x.XCreateSimpleWindow.argtypes=[C.c_void_p,C.c_ulong,C.c_int,C.c_int,C.c_uint,C.c_uint,C.c_uint,C.c_ulong,C.c_ulong];x.XCreateSimpleWindow.restype=C.c_ulong
    x.XDestroyWindow.argtypes=[C.c_void_p,C.c_ulong]
    x.XInternAtom.argtypes=[C.c_void_p,C.c_char_p,C.c_int];x.XInternAtom.restype=C.c_ulong
    x.XConvertSelection.argtypes=[C.c_void_p,C.c_ulong,C.c_ulong,C.c_ulong,C.c_ulong,C.c_ulong]
    x.XPending.argtypes=[C.c_void_p];x.XNextEvent.argtypes=[C.c_void_p,C.c_void_p];x.XFlush.argtypes=[C.c_void_p]
    x.XGetWindowProperty.argtypes=[C.c_void_p,C.c_ulong,C.c_ulong,C.c_long,C.c_long,C.c_int,C.c_ulong,C.POINTER(C.c_ulong),C.POINTER(C.c_int),C.POINTER(C.c_ulong),C.POINTER(C.c_ulong),C.POINTER(C.POINTER(C.c_ubyte))]
    x.XFree.argtypes=[C.c_void_p]
    display=x.XOpenDisplay(display_name.encode())
    if not display:raise RuntimeError('owned clipboard display unavailable')
    window=0
    try:
        window=x.XCreateSimpleWindow(display,x.XDefaultRootWindow(display),0,0,1,1,0,0,0)
        atom=lambda name:x.XInternAtom(display,name,0)
        prop=atom(b'DLPAD_ACCESSIBLE_PROBE');x.XConvertSelection(display,atom(b'CLIPBOARD'),atom(b'UTF8_STRING'),prop,window,0);x.XFlush(display)
        deadline=time.monotonic()+3;event=(C.c_long*24)()
        while time.monotonic()<deadline:
            if not x.XPending(display):time.sleep(.01);continue
            x.XNextEvent(display,C.byref(event));notification=C.cast(C.byref(event),C.POINTER(Selection)).contents
            if notification.type!=31:continue
            if not notification.property:raise RuntimeError('owned clipboard conversion refused')
            actual,count,remaining=C.c_ulong(),C.c_ulong(),C.c_ulong();bits=C.c_int();data=C.POINTER(C.c_ubyte)()
            status=x.XGetWindowProperty(display,window,prop,0,65536,0,0,C.byref(actual),C.byref(bits),C.byref(count),C.byref(remaining),C.byref(data))
            try:
                if status or bits.value!=8 or remaining.value or count.value>262144:raise RuntimeError('owned clipboard result exceeded cap')
                text=C.string_at(data,count.value).decode('utf-8')
            finally:
                if data:x.XFree(data)
            if 'fixture-secret' in text or 'signature=fixture' in text:raise RuntimeError('owned diagnostic preview was unredacted or invalid')
            return text
        raise RuntimeError('owned clipboard preview timeout')
    finally:
        if window:x.XDestroyWindow(display,window)
        x.XCloseDisplay(display)


def main():
 p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'));p.add_argument('--inspect',action='store_true');a=p.parse_args();a.binary=a.binary.resolve(strict=True);a.output=a.output.resolve();a.output.mkdir(parents=True,exist_ok=False);return run(a)
if __name__=='__main__':raise SystemExit(main())
