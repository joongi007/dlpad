#!/usr/bin/env python3
"""Accessible candidate IME/AT-SPI evidence confined to owned Xvfb/private buses."""
import argparse
import json
import os
from pathlib import Path
import re
import select
import subprocess
import tempfile
import time
from native_probe import probe_display, screenshot
from ime_probe import key_sequence, window_inventory, focus_owned_window, set_ime_mode, stop_owned, xim_styles
from managed_tools_probe import file_sha256, point
from x11_diagnostics import click_at


from egui_ime_probe import accessibility, binary_hash, call
import ast

def entry_text(env):
    snapshot=accessibility(env)
    for node in snapshot.get('nodes',[]):
        if '79' in node['role']['stdout'] and "Name': <'한글 입력 검증'>" in node['properties']['stdout'] and node['text']['exit']==0:
            return ast.literal_eval(node['text']['stdout'])[0]
    raise RuntimeError('candidate entry Text unavailable on owned AT-SPI bus')


def run(args):
    repo=Path(__file__).resolve().parents[1];root=repo/'.local-tools/ime/root'
    result={'fixture_commit_fallback':args.commit_fallback,'fixture_ime_events':args.ime_events,'binary_sha256':binary_hash(args.binary),'binary_bytes':args.binary.stat().st_size,'display':'owned Xvfb only','desktop_fallback_used':False,'actual_media_used':False,'screen_reader_verified':False,'clipboard_used':False,'preedit_verified':False,'ime_verified':False}
    app=ime=bus=None
    previous=os.environ.get('XAUTHORITY')
    try:
        with tempfile.TemporaryDirectory(prefix='egui-ime-') as temporary:
            task=Path(temporary)
            for name in ['config','data','cache','runtime','app-data','media']:(task/name).mkdir(mode=0o700)
            authority=task/'runtime/owned.Xauthority';authority.write_bytes(b'');os.environ['XAUTHORITY']=str(authority)
            fonts=task/'data/fonts';fonts.mkdir();(fonts/'Korean.ttf').symlink_to(args.font)
            fc=task/'config/fonts.conf';fc.write_text('<fontconfig><include ignore_missing="yes">/etc/fonts/fonts.conf</include><dir>'+str(fonts)+'</dir><cachedir>'+str(task/'cache/fontconfig')+'</cachedir></fontconfig>')
            config=task/'config/fcitx5';(config/'conf').mkdir(parents=True)
            (config/'conf/xim.conf').write_text('UseOnTheSpot=True\n')
            (config/'profile').write_text('[Groups/0]\nName=Default\nDefault Layout=us\nDefaultIM=hangul\n\n[Groups/0/Items/0]\nName=keyboard-us\nLayout=\n\n[Groups/0/Items/1]\nName=hangul\nLayout=\n\n[GroupOrder]\n0=Default\n')
            with probe_display(args.output/'isolated.json',False) as (display,_):
                env=dict(os.environ)
                for key in ['WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','AT_SPI_BUS_ADDRESS','NO_AT_BRIDGE']:env.pop(key,None)
                env.update(DISPLAY=display,XAUTHORITY=str(authority),FONTCONFIG_FILE=str(fc),XMODIFIERS='@im=fcitx',GTK_IM_MODULE='xim',QT_IM_MODULE='fcitx',XDG_CONFIG_HOME=str(task/'config'),XDG_DATA_HOME=str(task/'data'),XDG_CACHE_HOME=str(task/'cache'),XDG_RUNTIME_DIR=str(task/'runtime'),FCITX_CONFIG_HOME=str(config),FCITX_DATA_DIRS=str(root/'usr/share/fcitx5'),FCITX_ADDON_DIRS=str(root/'usr/lib/x86_64-linux-gnu/fcitx5'),LD_LIBRARY_PATH=str(root/'usr/lib/x86_64-linux-gnu')+':'+str(repo/'.local-tools/xvfb/usr/lib/x86_64-linux-gnu'),DLPAD_TEST_DATA=str(task/'app-data'),DLPAD_LIBRARY_ROOT=str(task/'media'),GSETTINGS_BACKEND='memory',DLPAD_TEST_FONT=str(args.font),LANG='C.UTF-8',LC_ALL='C.UTF-8')
                with (args.output/'bus.log').open('xb') as blog,(args.output/'ime.log').open('xb') as ilog,(args.output/'app.log').open('xb') as alog:
                    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(task/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
                    if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('owned bus readiness timeout')
                    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip()
                    if not env['DBUS_SESSION_BUS_ADDRESS'].startswith('unix:'):raise RuntimeError('owned bus address invalid')
                    registry=accessibility(env)
                    match=re.search(r"'([^']+)'",registry['bus_address_reply']['stdout'])
                    if not match:raise RuntimeError('owned AT-SPI readiness failed')
                    env['AT_SPI_BUS_ADDRESS']=match[1]
                    mapper=repo/'.local-tools/xvfb/usr/bin/proot'
                    ime=subprocess.Popen([str(mapper),'-b',str(root/'usr/share/libhangul')+':/usr/share/libhangul',str(root/'usr/bin/fcitx5'),'-D','--disable','all','--enable','keyboard,hangul,xcb,xim,dbus,classicui'],env=env,stdin=subprocess.DEVNULL,stdout=ilog,stderr=subprocess.STDOUT,start_new_session=True)
                    time.sleep(3)
                    if ime.poll() is not None:raise RuntimeError('owned IME exited')
                    app=subprocess.Popen([str(args.binary),'--fixture','--settings'],env=env,stdin=subprocess.DEVNULL,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True)
                    time.sleep(5)
                    result['initial_windows']=window_inventory(display)
                    if app.poll() is not None:raise RuntimeError('owned alternative GUI exited')
                    snapshot=accessibility(env)
                    for node in snapshot.get('nodes',[]):
                        if '(uint32 43,)' in node['role']['stdout'] and "Name': <'설정'>" in node['properties']['stdout']:
                            result['settings_action']=call(env,node['destination'],node['path'],'org.a11y.atspi.Action.DoAction','0',address=match[1]);time.sleep(.2);break
                    snapshot=accessibility(env)
                    origin=screenshot(args.output/'initial.png','dlpad',display)
                    input_point=args.input_point
                    if input_point is None:
                        for node in snapshot.get('nodes',[]):
                            if '(uint32 79,)' in node['role']['stdout'] and "Name': <'한글 입력 검증'>" in node['properties']['stdout']:
                                extents=call(env,node['destination'],node['path'],'org.a11y.atspi.Component.GetExtents','0',address=match[1])
                                if extents['exit']:raise RuntimeError('named IME entry extents unavailable')
                                x,y,w,h=ast.literal_eval(extents['stdout'])[0]
                                if w<=0 or h<=0 or x<origin['x'] or y<origin['y']:raise RuntimeError('named IME entry invalid extents')
                                input_point=(x+w//2-origin['x'],y+h//2-origin['y']);result['input_entry_extents']=extents;break
                        if input_point is None:raise RuntimeError('named role79 IME field unavailable')
                    result['actual_input_point']=input_point
                    if args.inspect:result['inspection_only']=True
                    else:
                        focus_owned_window(display);click_at(display,origin,*input_point);time.sleep(.5)
                        key_sequence(display,['a','b','c']);screenshot(args.output/'ascii-focus.png','dlpad',display)
                        key_sequence(display,['Control_L+a']);result['ascii_focus_text']=entry_text(env)
                        key_sequence(display,['BackSpace'])
                        result['accessibility']=accessibility(env)
                        set_ime_mode(display,env,root,True)
                        result['xim_styles']=xim_styles(display)
                        for key in 'gksrmf':
                            key_sequence(display,[key]);time.sleep(.25)
                        screenshot(args.output/'preedit.png','dlpad',display)
                        result['preedit_windows']=window_inventory(display)
                        result['preedit_requires_screenshot_review']=True
                        result['preedit_accessibility']=accessibility(env)
                        result['raw_preedit_event_verified']=any('Preedit' in node['text']['stdout'] and '글' in node['text']['stdout'] for node in result['preedit_accessibility'].get('nodes',[]))
                        key_sequence(display,['space']);screenshot(args.output/'committed.png','dlpad',display)
                        key_sequence(display,['Control_L+a']);result['committed_text']=entry_text(env)
                        result['commit_verified']=result['committed_text']=='한글 '
                        if not result['commit_verified']:result['commit_failure']='Hangul composition commit mismatch'
                        key_sequence(display,['End','g','k','BackSpace','BackSpace','Control_L+a','End'])
                        result['cancel_text']=entry_text(env);result['cancel_verified']=result['cancel_text']=='한글 '
                        key_sequence(display,['End']);set_ime_mode(display,env,root,False);key_sequence(display,['a','b','c','Control_L+a','End'])
                        result['latin_text']=entry_text(env);result['latin_transition_verified']=result['latin_text']=='한글 abc'
                        screenshot(args.output/'latin.png','dlpad',display)
                        result['accessibility']=accessibility(env)
                        key_sequence(display,['Control_L+a','x','y','Control_L+a','End'])
                        result['selection_replace_text']=entry_text(env)
                        result['selection_replace_verified']=result['selection_replace_text']=='xy'
                        screenshot(args.output/'selection.png','dlpad',display)
                    stop_owned(app);app=None;stop_owned(ime);ime=None;stop_owned(bus);bus=None
                    result['owned_children_reaped']=True
    except Exception as error:result['error']=str(error)
    finally:
        for child in [app,ime,bus]:stop_owned(child)
        result['owned_children_reaped']=all(child is None or child.poll() is not None for child in [app,ime,bus])
        if previous is None:os.environ.pop('XAUTHORITY',None)
        else:os.environ['XAUTHORITY']=previous
    result['temporary_fixture_removed']=True
    result['after_binary_sha256']=binary_hash(args.binary);result['hash_stable']=result['binary_sha256']==result['after_binary_sha256']
    (args.output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['initial_windows','preedit_windows','accessibility','preedit_accessibility']},ensure_ascii=False,indent=2))
    return 0 if not result.get('error') and (args.inspect or (result.get('commit_verified') and result.get('cancel_verified') and result.get('latin_transition_verified') and result.get('preedit_verified'))) else 1


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('binary',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--font',type=Path,default=os.environ.get('DLPAD_TEST_FONT'));parser.add_argument('--input-point',type=point,default=None);parser.set_defaults(ime_events=False,commit_fallback=False);parser.add_argument('--inspect',action='store_true');parser.add_argument('--validate-only',action='store_true')
    args=parser.parse_args()
    if args.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':parser.error('Linux isolated X11 only')
    args.binary=args.binary.resolve(strict=True);args.output=args.output.resolve()
    if not os.access(args.binary,os.X_OK):parser.error('binary must be executable')
    if args.font is None or not args.font.is_file():parser.error('explicit existing Korean fixture font required')
    args.font=args.font.resolve();args.output.mkdir(parents=True,exist_ok=False)
    return run(args)

if __name__=='__main__':raise SystemExit(main())
