#!/usr/bin/env python3
"""Default Iced two-input capability collection; screenshots require review."""
import argparse, json, os, re, select, subprocess, tempfile, threading, time
from pathlib import Path
from accessible_download_probe import file_hash, bounded_log, snapshot_database
from native_probe import probe_display, screenshot
from background_ui_probe import OwnedDisplay, reap, process_identity
from ime_probe import focus_owned_window, key_sequence
from subtitle_ui_probe import job
from x11_diagnostics import click_at

EXPECTED='94e62f13b940c96cea379af8507458859917fd5a9156d6f070c5ca26c0559856'

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--expected-sha256',default=EXPECTED)
    parser.add_argument('--font',type=Path,default=Path('/mnt/c/Windows/Fonts/malgun.ttf'))
    a=parser.parse_args()
    if re.fullmatch(r"[0-9a-f]{64}",a.expected_sha256) is None:parser.error("expected SHA256 must be 64 lowercase hex digits")
    a.binary=a.binary.resolve(strict=True);a.output.mkdir(parents=True,exist_ok=False)
    r={'measured_collection_passed':False,'keyboard_navigation_verified':False,'input_state_review':'REVIEW_REQUIRED','binary_sha256':file_hash(a.binary),'desktop_fallback_used':False,'observations':[]}
    if r['binary_sha256']!=a.expected_sha256:raise RuntimeError('unexpected immutable default binary')
    app=bus=owner=context=None;threads=[]
    with tempfile.TemporaryDirectory(prefix='default-tab-owned-') as tmp:
        root=Path(tmp)
        for name in ['data','media','config','cache','runtime','xdg-data']:(root/name).mkdir(mode=0o700)
        tool=root/'never-executed-tool';tool.write_text('#!/usr/bin/env python3\nfrom pathlib import Path\nPath(__file__).with_name("EXECUTED").write_text("unexpected")\n');tool.chmod(0o700)
        env=dict(os.environ)
        for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS']:env.pop(key,None)
        env.update(DLPAD_TEST_DATA=str(root/'data'),DLPAD_LIBRARY_ROOT=str(root/'media'),DLPAD_TEST_DOCUMENTS=str(root/'media'),DLPAD_YTDLP=str(tool),DLPAD_TEST_FONT=str(a.font),XDG_CONFIG_HOME=str(root/'config'),XDG_DATA_HOME=str(root/'xdg-data'),XDG_CACHE_HOME=str(root/'cache'),XDG_RUNTIME_DIR=str(root/'runtime'),MESA_SHADER_CACHE_DIR=str(root/'cache/shader'),WINIT_X11_SCALE_FACTOR='1')
        try:
            context=probe_display(a.output/'isolated',False);display,kind=context.__enter__()
            if kind!='Xvfb isolated':raise RuntimeError('private Xvfb required')
            env['DISPLAY']=display
            bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(root/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
            thread=threading.Thread(target=bounded_log,args=(bus.stderr,a.output/'bus.log'),daemon=True);thread.start();threads.append(thread)
            if not select.select([bus.stdout],[],[],5)[0]:raise RuntimeError('private bus readiness timeout')
            env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip()
            app=subprocess.Popen([str(a.binary),'--fixture','--fixture-events','--downloads'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
            thread=threading.Thread(target=bounded_log,args=(app.stdout,a.output/'app.log'),daemon=True);thread.start();threads.append(thread)
            r['app_identity']=process_identity(app.pid);r['bus_identity']=process_identity(bus.pid)
            time.sleep(2);owner=OwnedDisplay(display);geometry=screenshot(a.output/'initial.png','dlpad',display)
            if geometry['width']!=960 or geometry['height']!=640:raise RuntimeError('unexpected default window geometry')
            r['geometry']=geometry;r['initial_url_click']=[200,160];focus_owned_window(display);click_at(display,geometry,200,160);time.sleep(.2)
            for stage,keys in [('url-x',['x']),('after-tab',['Tab']),('after-y',['y']),('after-backspace',['BackSpace']),('after-shift-tab',['Shift_L+Tab'])]:
                if app.poll() is not None:raise RuntimeError('owned GUI exited before observation')
                key_sequence(display,keys);time.sleep(.3)
                screenshot(a.output/(stage+'.png'),'dlpad',display)
                r['observations'].append({'stage':stage,'keys':keys,'image':stage+'.png','input_readback':'REVIEW_REQUIRED'})
            if job(root/'data') is not None or (root/'EXECUTED').exists():raise RuntimeError('input probe unexpectedly created job or executed tool')
            r.update(job_created=False,tool_executed=False)
            owner.delete_window(owner.window());app.wait(timeout=10)
            if app.returncode!=0:raise RuntimeError('default GUI close failed')
            r['close_exit']=app.returncode;r['measured_collection_passed']=True
        except Exception as error:
            r['error']=str(error)
            if owner:
                try:screenshot(a.output/'failure.png','dlpad',env['DISPLAY'])
                except Exception as nested:r['capture_error']=str(nested)
        finally:
            if app:reap(app)
            if owner:owner.close()
            if bus:reap(bus)
            for thread in threads:thread.join(3)
            if context:context.__exit__(None,None,None)
            r['owned_cleanup']=all(p is None or p.poll() is not None for p in [app,bus]) and all(not t.is_alive() for t in threads)
            snapshot_database(root/'data',a.output/'jobs.sqlite')
            trace=root/'data/fixture-events.jsonl'
            if trace.exists():
                raw=trace.read_bytes()
                if len(raw)>16384:raise RuntimeError('fixture trace exceeds bound')
                (a.output/'fixture-events.jsonl').write_bytes(raw)
            r['fixture_trace_present']=trace.exists()
    r['binary_sha256_after']=file_hash(a.binary)
    r['measured_collection_passed']=r['measured_collection_passed'] and r['owned_cleanup'] and r['binary_sha256_after']==a.expected_sha256
    (a.output/'result.json').write_text(json.dumps(r,ensure_ascii=False,indent=2))
    return 0 if r['measured_collection_passed'] else 1

if __name__=='__main__':raise SystemExit(main())
