#!/usr/bin/env python3
"""Owned synthetic installed-version UI fixtures and official staging-only fault injection."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import sqlite3
import stat
import subprocess
import tempfile
import time

from native_probe import probe_display, screenshot
from managed_tools_probe import active, file_sha256, point, wait_until
from x11_diagnostics import click_at

TARGET='x86_64-unknown-linux-gnu'
OFFICIAL_BYTES=40_446_224
OFFICIAL_SHA256='58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a'


def seed(data):
    managed=data/'managed-tools';managed.mkdir(mode=0o700)
    (managed/'.dlpad-owner').write_bytes(b'dlpad-managed-tools-v1\n')
    installed={}
    for version in ('synthetic-v1','synthetic-v2'):
        destination=managed/'yt-dlp'/version/TARGET;destination.mkdir(parents=True)
        executable=destination/'yt-dlp'
        contents=("#!/usr/bin/python3\nprint('"+version+"')\n").encode()
        executable.write_bytes(contents);executable.chmod(0o700)
        manifest={'tool':'yt-dlp','version':version,'target':TARGET,'source_url':'https://example.org/'+version,
                  'sha256':hashlib.sha256(contents).hexdigest(),'license':'synthetic UI fixture; no distribution/version evidence',
                  'ownership':'Managed','executable':'yt-dlp'}
        (destination/'manifest.json').write_text(json.dumps(manifest))
        installed[version]=(executable,contents)
    with sqlite3.connect(managed/'activation.sqlite') as connection:
        connection.execute('CREATE TABLE active(tool TEXT PRIMARY KEY,version TEXT NOT NULL,target TEXT NOT NULL)')
        connection.execute('INSERT INTO active VALUES(?,?,?)',('yt-dlp','synthetic-v1',TARGET))
    return managed,installed


def keep_original(data,installed,external,external_bytes):
    if active(data)!=('synthetic-v1',TARGET):raise RuntimeError('previous active selection changed')
    for executable,contents in installed.values():
        if executable.read_bytes()!=contents:raise RuntimeError('previous managed fixture bytes changed')
    if external.read_bytes()!=external_bytes:raise RuntimeError('external fixture bytes changed')


def inject_padding(managed):
    """Exactly one bounded mutation of this invocation's owned ELF staging executable."""
    for stage in managed.glob('.staging-*'):
        candidate=stage/'yt-dlp'
        if stage.is_symlink() or candidate.is_symlink() or not candidate.is_file():continue
        info=candidate.stat()
        if info.st_size<1024*1024:continue
        if candidate.resolve().parent!=stage.resolve() or stage.resolve().parent!=managed.resolve():raise RuntimeError('staging escaped owned fixture')
        with candidate.open('r+b',buffering=0) as file:
            opened=os.fstat(file.fileno())
            if not stat.S_ISREG(opened.st_mode) or (opened.st_dev,opened.st_ino)!=(info.st_dev,info.st_ino):raise RuntimeError('staging identity changed before fault injection')
            header=file.read(16)
            if header[:4]!=b'\x7fELF' or len(header)!=16:raise RuntimeError('official Linux fixture is not ELF')
            previous=header[9];replacement=previous^1
            file.seek(9);file.write(bytes([replacement]));os.fsync(file.fileno())
        return {'owned_staging':str(candidate),'offset':9,'previous_byte':previous,'replacement_byte':replacement,
                'bytes_observed':info.st_size,'device':info.st_dev,'inode':info.st_ino,'same_file_identity':True,'official_source_unchanged':True}
    return None


def run(args):
    started=time.monotonic()
    with args.binary.open('rb') as binary:binary_sha=file_sha256(binary)
    result={'mode':args.mode,'binary_sha256':binary_sha,'display':'Xvfb isolated',
            'seed':'synthetic executable metadata/hash UI fixture; no --version execution evidence',
            'native_gui_verified':False,'actual_media_used':False}
    with tempfile.TemporaryDirectory(prefix='dlpad-retained-',dir=args.output.parent) as temporary:
        root=Path(temporary);data=root/'app-data';data.mkdir();media=root/'synthetic-media';media.mkdir()
        external=root/'external-original';external_bytes=b'owned synthetic external tool';external.write_bytes(external_bytes)
        managed,installed=seed(data)
        with probe_display(args.output,False) as (display,_):
            env=dict(os.environ);env.pop('WAYLAND_DISPLAY',None);env.pop('DLPAD_TEST_DOCUMENTS',None);env.pop('DLPAD_YTDLP',None)
            env.update(DISPLAY=display,DLPAD_TEST_FONT=str(args.font),DLPAD_TEST_DATA=str(data),DLPAD_LIBRARY_ROOT=str(media),MESA_SHADER_CACHE_DIR=str(root/'shader-cache'))
            with args.output.with_suffix('.log').open('xb') as logs:
                child=subprocess.Popen([str(args.binary),'--fixture','--tools'],env=env,stdout=logs,stderr=subprocess.STDOUT,start_new_session=True)
                try:
                    time.sleep(2);geometry=screenshot(args.output.with_name(args.output.stem+'-before.png'),'dlpad',display)
                    if args.mode=='inspect':result['inspection_only']=True
                    elif args.mode=='select-remove':
                        click_at(display,geometry,*args.select_point)
                        wait_until(lambda:active(data)==('synthetic-v2',TARGET),child,args.timeout)
                        time.sleep(.3)
                        screenshot(args.output.with_name(args.output.stem+'-selected.png'),'dlpad',display)
                        click_at(display,geometry,*args.remove_point)
                        old=installed['synthetic-v1'][0].parent
                        wait_until(lambda:not old.exists(),child,args.timeout)
                        if active(data)!=('synthetic-v2',TARGET) or installed['synthetic-v2'][0].read_bytes()!=installed['synthetic-v2'][1] or external.read_bytes()!=external_bytes:raise RuntimeError('inactive removal changed active/external fixture')
                        result.update(inactive_selected=True,old_inactive_removed=True,new_active_bytes_preserved=True,external_bytes_preserved=True,native_gui_verified=True)
                    else:
                        click_at(display,geometry,*args.install_point)
                        stages=lambda:list(managed.glob('.staging-*'))
                        wait_until(stages,child,min(args.timeout,15))
                        if args.mode=='cancel':
                            click_at(display,geometry,*args.cancel_point)
                            result['owned_staging_observed_before_cancel']=True
                        else:
                            result['injection']=wait_until(lambda:inject_padding(managed),child,args.timeout)
                        def completed():
                            present=stages()
                            if args.mode=='hash-failure':
                                path=Path(result['injection']['owned_staging'])
                                try:
                                    with path.open('rb') as file:
                                        info=os.fstat(file.fileno())
                                        if (info.st_dev,info.st_ino)!=(result['injection']['device'],result['injection']['inode']):raise RuntimeError('owned staging identity changed after injection')
                                        result['max_staging_bytes_observed']=max(result.get('max_staging_bytes_observed',0),info.st_size)
                                        if info.st_size==OFFICIAL_BYTES and 'mutated_executable_sha256' not in result:
                                            result['mutated_executable_sha256']=file_sha256(file)
                                            result['mutated_executable_bytes']=info.st_size
                                            if result['mutated_executable_sha256']==OFFICIAL_SHA256:raise RuntimeError('fault injection did not change official executable digest')
                                except FileNotFoundError:pass
                            return not present
                        wait_until(completed,child,args.timeout)
                        time.sleep(.3)
                        keep_original(data,installed,external,external_bytes)
                        result.update(previous_active_selection_preserved=True,previous_active_path_bytes_preserved=True,
                                      inactive_bytes_preserved=True,external_bytes_preserved=True,owned_staging_removed=True,
                                      error_text_requires_screenshot_review=True)
                        result['native_gui_verified']=args.mode=='cancel'
                    screenshot(args.output.with_suffix('.png'),'dlpad',display)
                except Exception as error:
                    result['error']=str(error)
                    try:screenshot(args.output.with_name(args.output.stem+'-error.png'),'dlpad',display)
                    except Exception:pass
                finally:
                    if child.poll() is None:
                        os.killpg(child.pid,signal.SIGTERM)
                        try:child.wait(timeout=3)
                        except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);child.wait()
                    result['owned_gui_reaped']=child.poll() is not None
    result['temporary_fixture_removed']=True;result['elapsed_seconds']=round(time.monotonic()-started,3)
    args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
    return int('error'in result)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary',type=Path);p.add_argument('output',type=Path)
    p.add_argument('--mode',choices=['inspect','select-remove','cancel','hash-failure'],default='select-remove')
    p.add_argument('--font',type=Path,default=os.environ.get('DLPAD_TEST_FONT'))
    p.add_argument('--install-point',type=point,default=(216,272));p.add_argument('--cancel-point',type=point,default=(350,272))
    p.add_argument('--select-point',type=point,default=(740,380));p.add_argument('--remove-point',type=point,default=(885,348))
    p.add_argument('--timeout',type=float,default=145);p.add_argument('--validate-only',action='store_true')
    args=p.parse_args()
    if not 1<=args.timeout<=180:p.error('timeout must be1..180 seconds')
    if args.output.suffix!='.json':p.error('output must end in.json')
    if args.validate_only:print('arguments valid; no processes started');return 0
    if os.name!='posix' or os.uname().sysname!='Linux':p.error('Linux X11 only')
    args.binary=args.binary.resolve();args.output=args.output.resolve()
    if not args.binary.is_file() or not os.access(args.binary,os.X_OK):p.error('binary must be executable')
    if args.font is None or not args.font.is_file():p.error('explicit fixture font required')
    args.font=args.font.resolve();args.output.parent.mkdir(parents=True,exist_ok=True)
    if list(args.output.parent.glob(args.output.stem+'.*')) or list(args.output.parent.glob(args.output.stem+'-*.png')):p.error('evidence prefix exists')
    try:return run(args)
    except (Exception,KeyboardInterrupt) as error:
        result={'mode':args.mode,'native_gui_verified':False,'error':str(error) or type(error).__name__,'desktop_fallback_used':False}
        args.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));return 1


if __name__=='__main__':raise SystemExit(main())
