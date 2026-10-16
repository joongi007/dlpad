#!/usr/bin/env python3
"""Inspect accessible candidate AT-SPI tree on owned Xvfb/private buses; never audio or user bus."""
import argparse
import json
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time
from egui_ime_probe import accessibility, binary_hash, call
from ime_probe import focus_owned_window, key_sequence, stop_owned
from native_probe import probe_display, screenshot
from x11_diagnostics import click_at


def inspect(binary,name,output,font):
    result={'binary_sha256':binary_hash(binary),'binary_bytes':binary.stat().st_size,'screen_reader_verified':False,'actual_audio_used':False,'user_bus_used':False,'desktop_fallback_used':False}
    bus=app=reader=None
    try:
        with tempfile.TemporaryDirectory(prefix='reader-bus-') as temporary:
            task=Path(temporary)
            for folder in ['config','data','cache','runtime','app-data','media']:(task/folder).mkdir(mode=0o700)
            authority=task/'runtime/owned.Xauthority';authority.write_bytes(b'')
            previous=os.environ.get('XAUTHORITY');os.environ['XAUTHORITY']=str(authority)
            try:
                with probe_display(output/'isolated.json',False) as (display,_):
                    env=dict(os.environ)
                    for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','XMODIFIERS','NO_AT_BRIDGE']:env.pop(key,None)
                    env.update(DISPLAY=display,XAUTHORITY=str(authority),XDG_CONFIG_HOME=str(task/'config'),XDG_DATA_HOME=str(task/'data'),XDG_CACHE_HOME=str(task/'cache'),XDG_RUNTIME_DIR=str(task/'runtime'),DLPAD_TEST_DATA=str(task/'app-data'),DLPAD_LIBRARY_ROOT=str(task/'media'),DLPAD_TEST_FONT=str(font),GSETTINGS_BACKEND='memory',MESA_SHADER_CACHE_DIR=str(task/'cache/mesa'),LIBGL_SHADER_CACHE_DIR=str(task/'cache/gl'),LANG='C.UTF-8',LC_ALL='C.UTF-8')
                    with (output/'bus.log').open('xb') as blog,(output/'app.log').open('xb') as alog:
                        bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(task/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
                        if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('private bus readiness timeout')
                        env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip()
                        if not env['DBUS_SESSION_BUS_ADDRESS'].startswith('unix:'):raise RuntimeError('private bus address invalid')
                        # Private registry/flags exist before app starts. They are not a reader.
                        result['registry_before_app']=accessibility(env)
                        address_reply=result['registry_before_app']['bus_address_reply']
                        if address_reply['exit']:raise RuntimeError('private accessibility bus unavailable')
                        import re
                        match=re.search(r"'([^']+)'",address_reply['stdout'])
                        if not match:raise RuntimeError('private accessibility address unavailable')
                        address=match[1];env['AT_SPI_BUS_ADDRESS']=address
                        reader_root=Path(__file__).resolve().parents[1]/'.local-tools/reader/root'
                        renv=dict(env,PATH=str(reader_root/'usr/bin')+os.pathsep+env.get('PATH',''),PYTHONPATH=str(reader_root/'usr/lib/python3/dist-packages'),GI_TYPELIB_PATH=str(reader_root/'usr/lib/x86_64-linux-gnu/girepository-1.0'))
                        runner="import resource,sys; resource.setrlimit(resource.RLIMIT_FSIZE,(2097152,2097152)); from orca import debug,orca; debug.debugLevel=debug.LEVEL_ALL; debug.eventDebugLevel=debug.LEVEL_ALL; debug.debugFile=open(sys.argv[2],'w',buffering=1); manager=orca.getSettingsManager(); manager.activate(sys.argv[1],dict(enableSpeech=False,enableBraille=False,enableSound=False,enableMouseReview=False)); sys.exit(orca.main())"
                        reader_log=output/'reader.log'
                        with (output/'reader-stderr.log').open('xb') as rlog:
                            reader=subprocess.Popen(['python3','-c',runner,str(task/'reader-prefs'),str(reader_log)],env=renv,stdout=rlog,stderr=subprocess.STDOUT,start_new_session=True)
                        time.sleep(2)
                        result['reader_alive_before_app']=reader.poll() is None
                        if reader.poll() is not None:raise RuntimeError('owned Orca initialization failed; inspect reader-stderr.log')
                        app=subprocess.Popen([str(binary),'--fixture','--settings'],env=env,stdin=subprocess.DEVNULL,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True)
                        time.sleep(5)
                        if app.poll() is not None:raise RuntimeError('owned native app exited')
                        origin=screenshot(output/'initial.png','dlpad',display)
                        focus_owned_window(display);click_at(display,origin,300,102)
                        key_sequence(display,list('readerfixture'));time.sleep(.3)
                        result['after_input']=accessibility(env)
                        for node in result['after_input'].get('nodes',[]):node['state']=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                        screenshot(output/'input.png','dlpad',display)
                        key_sequence(display,['Tab']);time.sleep(.3)
                        result['after_tab']=accessibility(env)
                        for node in result['after_tab'].get('nodes',[]):node['state']=call(env,node['destination'],node['path'],'org.a11y.atspi.Accessible.GetState',address=address)
                        screenshot(output/'tab.png','dlpad',display)
                        result['registry_has_any_nodes']=bool(result['after_input'].get('nodes'))
                        result['tree_published']=any("Name': <'한글 입력 검증'>" in node['properties']['stdout'] for node in result['after_input'].get('nodes',[]))
                        result['fixture_text_accessible']=any('readerfixture' in node['text']['stdout'] for node in result['after_input'].get('nodes',[]))
                        result['label_accessible']=any("Name': <'한글 입력 검증'>" in node['properties']['stdout'] for node in result['after_input'].get('nodes',[]))
                        stop_owned(app);stop_owned(reader);stop_owned(bus)
            finally:
                if previous is None:os.environ.pop('XAUTHORITY',None)
                else:os.environ['XAUTHORITY']=previous
    except Exception as error:result['error']=str(error)
    finally:
        stop_owned(app);stop_owned(reader);stop_owned(bus)
        result['owned_children_reaped']=all(child is None or child.poll() is not None for child in [app,reader,bus])
    result['temporary_fixture_removed']=True
    result['reader_exit_code']=reader.poll() if reader else None
    log=output/'reader.log'
    if log.exists():
        result['reader_log_bytes']=log.stat().st_size
        text=log.read_text(errors='replace')[:2097152]
        result['reader_registry_initialized']='ATSPI registry initialized' in text
        result['reader_candidate_events']=[line[:500] for line in text.splitlines() if any(word in line for word in ['dlpad','readerfixture','한글 입력 검증','object:state-changed:focused'])][-64:]
        result['reader_candidate_observed']=any('dlpad' in line for line in result['reader_candidate_events'])
        result['reader_text_observed']=any('readerfixture' in line for line in result['reader_candidate_events'])
        result['reader_text_changed_event_processed']='PROCESS OBJECT EVENT object:text-changed:insert' in text
        import re
        result['reader_keyboard_focus_processed']=bool(re.search(r'Changing locusOfFocus from .* to \[push button \| 다운로드\]', text))
        result['reader_event_processing_verified']=result.get('reader_registry_initialized',False) and result.get('fixture_text_accessible',False) and result['reader_text_changed_event_processed'] and result['reader_keyboard_focus_processed']
        result['screen_reader_verified']=False  # Audio and full reader usability were not tested.

    (output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    return {key:value for key,value in result.items() if key not in ['registry_before_app','after_input','after_tab']}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('binary',type=Path);p.add_argument('output',type=Path);p.add_argument('--font',type=Path,default=os.environ.get('DLPAD_TEST_FONT'));p.add_argument('--validate-only',action='store_true');args=p.parse_args()
    if args.validate_only:print('valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux isolated X11 only')
    if args.font is None or not args.font.is_file():p.error('explicit read-only Korean fixture font required')
    args.font=args.font.resolve();args.binary=args.binary.resolve(strict=True)
    if not os.access(args.binary,os.X_OK):p.error('candidate binaries must be executable')
    args.output=args.output.resolve();args.output.mkdir(parents=True,exist_ok=False)
    report={}
    for name in ['candidate']:
        output=args.output/name;output.mkdir();report[name]=inspect(args.binary,name,output,args.font)
    (args.output/'comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps(report,ensure_ascii=False,indent=2))
    return 1 if any('error' in value for value in report.values()) else 0

if __name__=='__main__':raise SystemExit(main())
