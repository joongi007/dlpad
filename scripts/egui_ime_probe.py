#!/usr/bin/env python3
"""Alternative egui IME/AT-SPI evidence confined to owned Xvfb/private buses."""
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
from ime_probe import key_sequence, window_inventory, focus_owned_window, clipboard_text, set_ime_mode, stop_owned, xim_styles
from managed_tools_probe import file_sha256, point
from x11_diagnostics import click_at


def call(env, dest, path, method, *args, address=None):
    command=['gdbus','call']+(['--address',address] if address else ['--session'])+['--dest',dest,'--object-path',path,'--method',method,'--',*args]
    child=subprocess.run(command,env=env,capture_output=True,timeout=5)
    return {'exit':child.returncode,'stdout':child.stdout.decode('utf-8','replace')[:16384],'stderr':child.stderr.decode('utf-8','replace')[:2048]}


def accessibility(env):
    result={'screen_reader_verified':False,'tree_verified':False}
    reply=call(env,'org.a11y.Bus','/org/a11y/bus','org.a11y.Bus.GetAddress')
    result['bus_address_reply']=reply
    match=re.search(r"'([^']+)'",reply['stdout'])
    if reply['exit'] or not match:return result
    address=match[1]
    result['enable_reply']=call(env,'org.a11y.Bus','/org/a11y/bus','org.freedesktop.DBus.Properties.Set','org.a11y.Status','IsEnabled','<true>')
    result['adapter_reader_flag_reply']=call(env,'org.a11y.Bus','/org/a11y/bus','org.freedesktop.DBus.Properties.Set','org.a11y.Status','ScreenReaderEnabled','<true>')
    # This enables the adapter; it does not represent an actual screen reader.
    time.sleep(1)
    names=call(env,'org.freedesktop.DBus','/org/freedesktop/DBus','org.freedesktop.DBus.ListNames',address=address)
    result['names']=names
    children=call(env,'org.a11y.atspi.Registry','/org/a11y/atspi/accessible/root','org.a11y.atspi.Accessible.GetChildren',address=address)
    result['registry_children']=children
    pairs=re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",children['stdout'])
    nodes=[];queue=list(pairs);seen=set()
    while queue and len(nodes)<48:
        dest,path=queue.pop(0)
        if (dest,path) in seen:continue
        seen.add((dest,path))
        node={'destination':dest,'path':path}
        for label,method,args in [('properties','org.freedesktop.DBus.Properties.GetAll',['org.a11y.atspi.Accessible']),('role','org.a11y.atspi.Accessible.GetRole',[]),('text','org.a11y.atspi.Text.GetText',['0','-1']),('children','org.a11y.atspi.Accessible.GetChildren',[])]:
            node[label]=call(env,dest,path,method,*args,address=address)
        nodes.append(node)
        queue.extend(re.findall(r"\('([^']+)', (?:objectpath )?'([^']+)'\)",node['children']['stdout']))
    result['nodes']=nodes
    result['tree_verified']=any(n['text']['exit']==0 and "Name': <'한글 입력 검증'>" in n['properties']['stdout'] for n in nodes)
    return result


def binary_hash(path):
    with path.open("rb") as file:return file_sha256(file)


def run(args):
    repo=Path(__file__).resolve().parents[1];root=repo/'.local-tools/ime/root'
    result={'fixture_commit_fallback':args.commit_fallback,'fixture_ime_events':args.ime_events,'binary_sha256':binary_hash(args.binary),'binary_bytes':args.binary.stat().st_size,'display':'owned Xvfb only','desktop_fallback_used':False,'actual_media_used':False,'screen_reader_verified':False,'preedit_verified':False,'ime_verified':False}
    app=ime=bus=None
    previous=os.environ.get('XAUTHORITY')
    try:
        with tempfile.TemporaryDirectory(prefix='egui-ime-') as temporary:
            task=Path(temporary)
            for name in ['config','data','cache','runtime']:(task/name).mkdir(mode=0o700)
            authority=task/'runtime/owned.Xauthority';authority.write_bytes(b'');os.environ['XAUTHORITY']=str(authority)
            fonts=task/'data/fonts';fonts.mkdir();(fonts/'Korean.ttf').symlink_to(args.font)
            fc=task/'config/fonts.conf';fc.write_text('<fontconfig><include ignore_missing="yes">/etc/fonts/fonts.conf</include><dir>'+str(fonts)+'</dir><cachedir>'+str(task/'cache/fontconfig')+'</cachedir></fontconfig>')
            config=task/'config/fcitx5';(config/'conf').mkdir(parents=True)
            (config/'conf/xim.conf').write_text('UseOnTheSpot=True\n')
            (config/'profile').write_text('[Groups/0]\nName=Default\nDefault Layout=us\nDefaultIM=hangul\n\n[Groups/0/Items/0]\nName=keyboard-us\nLayout=\n\n[Groups/0/Items/1]\nName=hangul\nLayout=\n\n[GroupOrder]\n0=Default\n')
            with probe_display(args.output/'isolated.json',False) as (display,_):
                env=dict(os.environ);env.pop('WAYLAND_DISPLAY',None)
                env.update(DISPLAY=display,XAUTHORITY=str(authority),FONTCONFIG_FILE=str(fc),XMODIFIERS='@im=fcitx',GTK_IM_MODULE='xim',QT_IM_MODULE='fcitx',XDG_CONFIG_HOME=str(task/'config'),XDG_DATA_HOME=str(task/'data'),XDG_CACHE_HOME=str(task/'cache'),XDG_RUNTIME_DIR=str(task/'runtime'),FCITX_CONFIG_HOME=str(config),FCITX_DATA_DIRS=str(root/'usr/share/fcitx5'),FCITX_ADDON_DIRS=str(root/'usr/lib/x86_64-linux-gnu/fcitx5'),LD_LIBRARY_PATH=str(root/'usr/lib/x86_64-linux-gnu')+':'+str(repo/'.local-tools/xvfb/usr/lib/x86_64-linux-gnu'),DLPAD_TEST_FONT=str(args.font),LANG='C.UTF-8',LC_ALL='C.UTF-8')
                with (args.output/'bus.log').open('xb') as blog,(args.output/'ime.log').open('xb') as ilog,(args.output/'app.log').open('xb') as alog:
                    bus=subprocess.Popen(['dbus-daemon','--session','--nofork','--print-address=1','--address=unix:path='+str(task/'runtime/bus')],env=env,stdout=subprocess.PIPE,stderr=blog,start_new_session=True)
                    if not select.select([bus.stdout],[],[],10)[0]:raise RuntimeError('owned bus readiness timeout')
                    env['DBUS_SESSION_BUS_ADDRESS']=bus.stdout.readline(4096).decode().strip()
                    if not env['DBUS_SESSION_BUS_ADDRESS'].startswith('unix:'):raise RuntimeError('owned bus address invalid')
                    mapper=repo/'.local-tools/xvfb/usr/bin/proot'
                    ime=subprocess.Popen([str(mapper),'-b',str(root/'usr/share/libhangul')+':/usr/share/libhangul',str(root/'usr/bin/fcitx5'),'-D','--disable','all','--enable','keyboard,hangul,xcb,xim,dbus,classicui'],env=env,stdin=subprocess.DEVNULL,stdout=ilog,stderr=subprocess.STDOUT,start_new_session=True)
                    time.sleep(3)
                    if ime.poll() is not None:raise RuntimeError('owned IME exited')
                    app=subprocess.Popen([str(args.binary),'--fixture','--settings']+(['--ime-events'] if args.ime_events else [])+(['--ime-commit-fallback'] if args.commit_fallback else []),env=env,stdin=subprocess.DEVNULL,stdout=alog,stderr=subprocess.STDOUT,start_new_session=True)
                    time.sleep(5)
                    result['initial_windows']=window_inventory(display)
                    if app.poll() is not None:raise RuntimeError('owned alternative GUI exited')
                    origin=screenshot(args.output/'initial.png','dlpad',display)
                    if args.inspect:result['inspection_only']=True
                    else:
                        focus_owned_window(display);click_at(display,origin,*args.input_point);time.sleep(.5)
                        key_sequence(display,['a','b','c']);screenshot(args.output/'ascii-focus.png','dlpad',display)
                        key_sequence(display,['Control_L+a','Control_L+c']);result['ascii_focus_text']=clipboard_text(display)
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
                        key_sequence(display,['Control_L+a','Control_L+c']);result['committed_text']=clipboard_text(display)
                        result['commit_verified']=result['committed_text']=='한글 '
                        if not result['commit_verified']:result['commit_failure']='Hangul composition commit mismatch'
                        key_sequence(display,['End','g','k','BackSpace','BackSpace','Control_L+a','Control_L+c'])
                        result['cancel_text']=clipboard_text(display);result['cancel_verified']=result['cancel_text']=='한글 '
                        key_sequence(display,['End']);set_ime_mode(display,env,root,False);key_sequence(display,['a','b','c','Control_L+a','Control_L+c'])
                        result['latin_text']=clipboard_text(display);result['latin_transition_verified']=result['latin_text']=='한글 abc'
                        screenshot(args.output/'latin.png','dlpad',display)
                        result['accessibility']=accessibility(env)
                        key_sequence(display,['Control_L+a','x','y','Control_L+a','Control_L+c'])
                        result['selection_replace_text']=clipboard_text(display)
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
    (args.output/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k not in ['initial_windows','preedit_windows','accessibility','preedit_accessibility']},ensure_ascii=False,indent=2))
    return 0 if not result.get('error') and (args.inspect or (result.get('commit_verified') and result.get('cancel_verified') and result.get('latin_transition_verified') and result.get('preedit_verified'))) else 1


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('binary',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--font',type=Path,default=os.environ.get('DLPAD_TEST_FONT'));parser.add_argument('--input-point',type=point,default=(300,268));parser.add_argument('--ime-events',action='store_true');parser.add_argument('--commit-fallback',action='store_true');parser.add_argument('--inspect',action='store_true');parser.add_argument('--validate-only',action='store_true')
    args=parser.parse_args()
    if args.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':parser.error('Linux isolated X11 only')
    args.binary=args.binary.resolve(strict=True);args.output=args.output.resolve()
    if not os.access(args.binary,os.X_OK):parser.error('binary must be executable')
    if args.font is None or not args.font.is_file():parser.error('explicit existing Korean fixture font required')
    args.font=args.font.resolve();args.output.mkdir(parents=True,exist_ok=False)
    return run(args)

if __name__=='__main__':raise SystemExit(main())
