#!/usr/bin/env python3
"""Explicit XIM1026 callbacks reference confined to owned Xvfb/private Fcitx."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import subprocess
import tempfile
import time
from native_probe import probe_display
from ime_probe import key_sequence, focus_owned_window, set_ime_mode, stop_owned, window_inventory


def digest(path):
    value=hashlib.sha256()
    with path.open('rb') as file:
        while chunk:=file.read(65536):value.update(chunk)
    return value.hexdigest()


def trace(path):
    if path.stat().st_size>192*1024:raise RuntimeError('reference trace exceeded bound')
    records=[]
    for line in path.read_text().splitlines():
        if len(records)>=128:raise RuntimeError('reference event limit exceeded')
        records.append(json.loads(line))
    return records


def wait(predicate,limit=5):
    until=time.monotonic()+limit
    while time.monotonic()<until:
        if predicate():return
        time.sleep(.05)
    raise RuntimeError('owned reference readiness timeout')


def run(args):
    repo=Path(__file__).resolve().parents[1];root=repo/'.local-tools/ime/root'
    source=repo/'scripts/xim_reference_probe.c'
    result={'reference_only':True,'event_window_filter':args.event_window_filter,'requested_style':1026,'reference_sha256':digest(args.binary),'source_sha256':digest(source),'desktop_fallback_used':False,'callbacks_verified':False,'actual_media_used':False}
    app=ime=bus=None;previous=os.environ.get('XAUTHORITY');started=time.monotonic()
    try:
        with tempfile.TemporaryDirectory(prefix='xim-reference-') as temporary:
            task=Path(temporary)
            for name in ['config','data','cache','runtime']:(task/name).mkdir(mode=0o700)
            authority=task/'runtime/owned.Xauthority';authority.write_bytes(b'');os.environ['XAUTHORITY']=str(authority)
            config=task/'config/fcitx5';(config/'conf').mkdir(parents=True)
            (config/'conf/xim.conf').write_text('UseOnTheSpot=True\n')
            (config/'profile').write_text('[Groups/0]\nName=Default\nDefault Layout=us\nDefaultIM=hangul\n\n[Groups/0/Items/0]\nName=keyboard-us\nLayout=\n\n[Groups/0/Items/1]\nName=hangul\nLayout=\n\n[GroupOrder]\n0=Default\n')
            fonts=task/'data/fonts';fonts.mkdir();(fonts/'Korean.ttf').symlink_to(args.font)
            fc=task/'config/fonts.conf';fc.write_text('<fontconfig><include ignore_missing="yes">/etc/fonts/fonts.conf</include><dir>'+str(fonts)+'</dir><cachedir>'+str(task/'cache/fontconfig')+'</cachedir></fontconfig>')
            with probe_display(args.output/'isolated.json',False) as (display,_):
                result['display']='owned Xvfb only'
                env=dict(os.environ);env.pop('WAYLAND_DISPLAY',None)
                env.update(DISPLAY=display,XAUTHORITY=str(authority),FONTCONFIG_FILE=str(fc),XMODIFIERS='@im=fcitx',XDG_CONFIG_HOME=str(task/'config'),XDG_DATA_HOME=str(task/'data'),XDG_CACHE_HOME=str(task/'cache'),XDG_RUNTIME_DIR=str(task/'runtime'),FCITX_CONFIG_HOME=str(config),FCITX_DATA_DIRS=str(root/'usr/share/fcitx5'),FCITX_ADDON_DIRS=str(root/'usr/lib/x86_64-linux-gnu/fcitx5'),LD_LIBRARY_PATH=str(root/'usr/lib/x86_64-linux-gnu')+':'+str(repo/'.local-tools/xvfb/usr/lib/x86_64-linux-gnu'),LANG='C.UTF-8',LC_ALL='C.UTF-8')
                with (args.output/'bus.log').open('xb') as blog,(args.output/'ime.log').open('xb') as ilog,(args.output/'reference.jsonl').open('xb') as output,(args.output/'reference.stderr').open('xb') as errors:
                    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(task/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
                    if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('private bus readiness timeout')
                    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip()
                    if not env['DBUS_SESSION_BUS_ADDRESS'].startswith('unix:'):raise RuntimeError('invalid private bus address')
                    ime=subprocess.Popen([str(repo/'.local-tools/xvfb/usr/bin/proot'),'-b',str(root/'usr/share/libhangul')+':/usr/share/libhangul',str(root/'usr/bin/fcitx5'),'-D','--disable','all','--enable','keyboard,hangul,xcb,xim,dbus,classicui','--verbose','key_trace=5'],env=env,stdin=subprocess.DEVNULL,stdout=ilog,stderr=subprocess.STDOUT,start_new_session=True)
                    time.sleep(3)
                    if ime.poll() is not None:raise RuntimeError('owned IME exited')
                    app=subprocess.Popen([str(args.binary)]+(['--event-window-filter'] if args.event_window_filter else []),env=env,stdin=subprocess.DEVNULL,stdout=output,stderr=errors,start_new_session=True)
                    def ready():
                        if app.poll() is not None:raise RuntimeError('reference exited before style readiness')
                        return any(e['event']=='ready_style1026' for e in trace(args.output/'reference.jsonl'))
                    wait(ready)
                    focus_owned_window(display);time.sleep(.5);set_ime_mode(display,env,root,True)
                    for key in 'gksrmf':key_sequence(display,[key]);time.sleep(.25)
                    before=trace(args.output/'reference.jsonl');result['precommit_records']=before;result['preedit_window_inventory']=window_inventory(display)
                    key_sequence(display,['space']);time.sleep(.3)
                    committed=trace(args.output/'reference.jsonl');result['committed_text']=''.join(e['text'] for e in committed if e['event']=='commit')
                    key_sequence(display,['g','k','BackSpace','BackSpace']);time.sleep(.3)
                    cancelled=trace(args.output/'reference.jsonl');result['after_cancel_committed_text']=''.join(e['text'] for e in cancelled if e['event']=='commit')
                    set_ime_mode(display,env,root,False);key_sequence(display,['a','b','c']);time.sleep(.3)
                    records=trace(args.output/'reference.jsonl');result['final_committed_text']=''.join(e['text'] for e in records if e['event']=='commit')
                    result['start_observed']=any(e['event']=='start' for e in records);result['draw_observed']=any(e['event']=='draw' for e in records);result['done_observed']=any(e['event']=='done' for e in records);result['caret_observed']=any(e['event']=='caret' for e in records)
                    result['hangul_preedit_observed']=any(e['event']=='draw' and any('가'<=c<='힣' for c in e['text']) for e in before)
                    result['draw_metadata']=[e for e in records if e['event']=='draw_metadata']
                    result['callback_errors']=[e for e in records if e['event'].startswith(('invalid_','preedit_limit','commit_limit'))]
                    result['commit_verified']=result['committed_text']=='한글 ';result['cancel_verified']=result['after_cancel_committed_text']=='한글 ';result['latin_verified']=result['final_committed_text']=='한글 abc'
                    result['server_preedit_delivery_verified']=all(result[k] for k in ['draw_observed','hangul_preedit_observed','commit_verified','cancel_verified','latin_verified']) and not result['callback_errors']
                    result['all_callback_kinds_observed']=all(result[k] for k in ['start_observed','draw_observed','done_observed','caret_observed'])
                    result['callbacks_verified']=result['server_preedit_delivery_verified']
                    result['style_readback_supported']=any(e['event']=='style_readback1026' for e in records)
                    result['style_readback_error']=next((e['text'] for e in records if e['event']=='style_readback_unsupported'),None)
                    stop_owned(app);stop_owned(ime);stop_owned(bus)
    except Exception as error:result['error']=str(error)
    finally:
        for child in [app,ime,bus]:stop_owned(child)
        result['owned_children_reaped']=all(child is None or child.poll() is not None for child in [app,ime,bus])
        if previous is None:os.environ.pop('XAUTHORITY',None)
        else:os.environ['XAUTHORITY']=previous
    result['temporary_fixture_removed']=True;result['elapsed_seconds']=round(time.monotonic()-started,3)
    (args.output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['precommit_records','preedit_window_inventory']},ensure_ascii=False,indent=2))
    return 0 if result.get('server_preedit_delivery_verified') and not result.get('error') else 1


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('binary',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--font',type=Path,default=os.environ.get('DLPAD_TEST_FONT'));parser.add_argument('--event-window-filter',action='store_true');parser.add_argument('--validate-only',action='store_true');args=parser.parse_args()
    if args.validate_only:print('valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':parser.error('Linux isolated X11 only')
    args.binary=args.binary.resolve(strict=True)
    if not os.access(args.binary,os.X_OK):parser.error('reference must be executable')
    if args.font is None or not args.font.is_file():parser.error('explicit read-only font fixture required')
    args.font=args.font.resolve();args.output=args.output.resolve();args.output.mkdir(parents=True,exist_ok=False)
    return run(args)

if __name__=='__main__':raise SystemExit(main())
