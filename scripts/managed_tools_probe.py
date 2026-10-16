#!/usr/bin/env python3
"""Owned Linux/Xvfb managed-tool GUI probe. Never builds or uses visible desktop."""
import argparse
import ctypes as C
import ctypes.util
import hashlib
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import tempfile
import time

from native_probe import probe_display, screenshot
from x11_diagnostics import Selection, click_at

VERSION = '2026.08.19'
DIGEST = '58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a'
SIZE = 40_446_224


def file_sha256(file):
    digest = hashlib.sha256()
    while chunk := file.read(64 * 1024):
        digest.update(chunk)
    return digest.hexdigest()


def point(value):
    try:
        x, y = map(int, value.split(','))
        if not (0 <= x < 960 and 0 <= y < 640):
            raise ValueError()
        return x, y
    except ValueError as error:
        raise argparse.ArgumentTypeError('expected X,Y within 960x640 fixture') from error


def parser():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('binary', type=Path)
    p.add_argument('output', type=Path, help='new JSON report; sibling screenshots/logs')
    p.add_argument('--mode', choices=['install', 'cancel', 'failure'], default='install')
    p.add_argument('--timeout', type=float, default=145)
    p.add_argument('--install-point', type=point, default=(216, 272))
    p.add_argument('--cancel-point', type=point, default=(350, 272))
    p.add_argument('--use-point', type=point, default=(150, 444))
    p.add_argument('--tool-point', type=point, default=(250, 350))
    p.add_argument('--font', type=Path, default=os.environ.get('DLPAD_TEST_FONT'), help='explicit fixture font (defaults to DLPAD_TEST_FONT)')
    p.add_argument('--validate-only', action='store_true')
    return p


def active(data):
    db = data / 'managed-tools' / 'activation.sqlite'
    if not db.is_file():
        return None
    with sqlite3.connect(db.resolve().as_uri() + '?mode=ro', uri=True, timeout=0.1) as connection:
        try:
            return connection.execute("SELECT version,target FROM active WHERE tool='yt-dlp'").fetchone()
        except sqlite3.OperationalError as error:
            if 'no such table' in str(error):
                return None
            raise


def wait_until(predicate, child, timeout):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if child.poll() is not None:
            raise RuntimeError(f'owned GUI exited early: {child.returncode}')
        value = predicate()
        if value:
            return value
        time.sleep(0.05)
    raise RuntimeError('GUI observation deadline exceeded')


def copy_input(display_name, origin, position):
    """Ctrl+A/C followed by bounded UTF8 clipboard request on owned Xvfb only."""
    click_at(display_name, origin, *position)
    x = C.CDLL(ctypes.util.find_library('X11'))
    t = C.CDLL(ctypes.util.find_library('Xtst'))
    x.XOpenDisplay.argtypes = [C.c_char_p]; x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.argtypes = [C.c_void_p]; x.XDefaultRootWindow.restype = C.c_ulong
    x.XCreateSimpleWindow.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_int, C.c_uint, C.c_uint, C.c_uint, C.c_ulong, C.c_ulong]; x.XCreateSimpleWindow.restype = C.c_ulong
    x.XDestroyWindow.argtypes = [C.c_void_p, C.c_ulong]
    x.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)), C.POINTER(C.c_uint)]
    x.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_char_p)]
    x.XSetInputFocus.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_ulong]
    x.XInternAtom.argtypes = [C.c_void_p, C.c_char_p, C.c_int]; x.XInternAtom.restype = C.c_ulong
    x.XKeysymToKeycode.argtypes = [C.c_void_p, C.c_ulong]; x.XKeysymToKeycode.restype = C.c_ubyte
    x.XConvertSelection.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_ulong, C.c_ulong, C.c_ulong]
    x.XFlush.argtypes = [C.c_void_p]; x.XPending.argtypes = [C.c_void_p]
    x.XNextEvent.argtypes = [C.c_void_p, C.c_void_p]; x.XFree.argtypes = [C.c_void_p]
    x.XGetWindowProperty.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_long, C.c_long, C.c_int, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_int), C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ubyte))]
    t.XTestFakeKeyEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]
    display = x.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError('owned Xvfb clipboard unavailable')
    window = None
    try:
        root, parent, children, count = C.c_ulong(), C.c_ulong(), C.POINTER(C.c_ulong)(), C.c_uint()
        x.XQueryTree(display, x.XDefaultRootWindow(display), C.byref(root), C.byref(parent), C.byref(children), C.byref(count))
        owned_window = None
        try:
            for index in range(count.value):
                name = C.c_char_p()
                x.XFetchName(display, children[index], C.byref(name))
                try:
                    if name.value and b'dlpad' in name.value:
                        owned_window = children[index]; break
                finally:
                    if name: x.XFree(name)
        finally:
            if children: x.XFree(children)
        if owned_window is None:
            raise RuntimeError('owned Xvfb window unavailable for keyboard focus')
        x.XSetInputFocus(display, owned_window, 2, 0)
        x.XFlush(display); time.sleep(0.1)
        control = x.XKeysymToKeycode(display, 0xffe3)
        t.XTestFakeKeyEvent(display, control, 1, 0)
        for key in (ord('a'), ord('c')):
            code = x.XKeysymToKeycode(display, key)
            t.XTestFakeKeyEvent(display, code, 1, 0); t.XTestFakeKeyEvent(display, code, 0, 0)
            x.XFlush(display); time.sleep(0.1)
        t.XTestFakeKeyEvent(display, control, 0, 0); x.XFlush(display)
        time.sleep(0.3)
        window = x.XCreateSimpleWindow(display, x.XDefaultRootWindow(display), 0, 0, 1, 1, 0, 0, 0)
        atom = x.XInternAtom(display, b'DLPAD_MANAGED_PROBE', 0)
        target = x.XInternAtom(display, b'UTF8_STRING', 0)
        x.XConvertSelection(display, x.XInternAtom(display, b'CLIPBOARD', 0), target, atom, window, 0)
        x.XFlush(display)
        deadline = time.monotonic() + 3
        event = (C.c_long * 24)()
        while time.monotonic() < deadline:
            if not x.XPending(display):
                time.sleep(0.01); continue
            x.XNextEvent(display, C.byref(event))
            notice = C.cast(C.byref(event), C.POINTER(Selection)).contents
            if notice.type != 31:
                continue
            if not notice.property:
                raise RuntimeError('tool input clipboard refused')
            kind, count, remaining = C.c_ulong(), C.c_ulong(), C.c_ulong()
            bits, data = C.c_int(), C.POINTER(C.c_ubyte)()
            status = x.XGetWindowProperty(display, window, atom, 0, 4096, 0, 0, C.byref(kind), C.byref(bits), C.byref(count), C.byref(remaining), C.byref(data))
            try:
                if status or kind.value != target or bits.value != 8 or remaining.value or count.value > 16384:
                    raise RuntimeError('invalid or oversized tool clipboard')
                return C.string_at(data, count.value).decode('utf-8')
            finally:
                if data: x.XFree(data)
        raise RuntimeError('tool clipboard timed out')
    finally:
        if window is not None: x.XDestroyWindow(display, window)
        x.XCloseDisplay(display)


def run(args):
    started = time.monotonic()
    with args.binary.open('rb') as binary:
        binary_sha256 = file_sha256(binary)
    result = {'mode': args.mode, 'display': 'Xvfb isolated', 'binary': str(args.binary),
              'binary_sha256': binary_sha256,
              'actual_media_used': False, 'native_gui_verified': False}
    with tempfile.TemporaryDirectory(prefix='dlpad-managed-probe-', dir=args.output.parent) as temporary:
        root = Path(temporary)
        data = root / 'app-data'; data.mkdir()
        media = root / 'synthetic-media'; media.mkdir()
        managed = data / 'managed-tools'
        foreign = None
        if args.mode == 'failure':
            managed.mkdir()
            foreign = managed / 'foreign-original'
            foreign.write_bytes(b'owned synthetic foreign fixture')
        with probe_display(args.output, False) as (display, _):
            env = dict(os.environ)
            for key in ['WAYLAND_DISPLAY', 'DLPAD_YTDLP', 'DLPAD_TEST_DOCUMENTS']:
                env.pop(key, None)
            env.update(DLPAD_TEST_FONT=str(args.font), DISPLAY=display, DLPAD_TEST_DATA=str(data), DLPAD_LIBRARY_ROOT=str(media),
                       MESA_SHADER_CACHE_DIR=str(root / 'shader-cache'))
            with args.output.with_suffix('.log').open('xb') as logs:
                child = subprocess.Popen([str(args.binary), '--fixture', '--tools'], env=env,
                                         stdout=logs, stderr=subprocess.STDOUT, start_new_session=True)
                try:
                    time.sleep(2)
                    geometry = screenshot(args.output.with_name(args.output.stem+'-before.png'), 'dlpad', display)
                    click_at(display, geometry, *args.install_point)
                    if args.mode == 'install':
                        selected = wait_until(lambda: active(data), child, args.timeout)
                        if selected != (VERSION, 'x86_64-unknown-linux-gnu'):
                            raise RuntimeError(f'unexpected active pin: {selected}')
                        executable = managed / 'yt-dlp' / VERSION / selected[1] / 'yt-dlp'
                        with executable.open('rb') as published:
                            digest = file_sha256(published)
                        if executable.stat().st_size != SIZE or digest != DIGEST:
                            raise RuntimeError('published executable does not match official pin')
                        record = json.loads((executable.parent / 'manifest.json').read_text())
                        if record['sha256'] != DIGEST or record['source_url'] != f'https://github.com/yt-dlp/yt-dlp/releases/download/{VERSION}/yt-dlp_linux':
                            raise RuntimeError('unexpected manifest supply')
                        time.sleep(0.4)
                        screenshot(args.output.with_name(args.output.stem+'-installed.png'), 'dlpad', display)
                        click_at(display, geometry, *args.use_point)
                        time.sleep(1)
                        click_at(display, geometry, 60, 76)
                        time.sleep(0.3)
                        copied = copy_input(display, geometry, args.tool_point)
                        if copied != str(executable):
                            raise RuntimeError(f'download tool input mismatch: {copied!r}')
                        result.update(active_pin_verified=True, executable_sha256=digest,
                                      use_active_download_input_verified=True)
                    elif args.mode == 'cancel':
                        stages = lambda: list(managed.glob('.staging-*')) if managed.exists() else []
                        wait_until(stages, child, min(args.timeout, 15))
                        click_at(display, geometry, *args.cancel_point)
                        wait_until(lambda: not stages(), child, args.timeout)
                        if active(data) is not None:
                            raise RuntimeError('cancelled request activated a tool')
                        result.update(staging_observed_before_cancel=True, no_activation_after_cancel=True,
                                      owned_staging_removed=True)
                    else:
                        time.sleep(1)
                        if foreign.read_bytes() != b'owned synthetic foreign fixture' or active(data) is not None:
                            raise RuntimeError('ownership failure changed fixture or activation')
                        result.update(foreign_fixture_preserved=True, no_activation_after_failure=True,
                                      error_text_requires_screenshot_review=True)
                    result['screenshot'] = screenshot(args.output.with_suffix('.png'), 'dlpad', display)
                    result['native_gui_verified'] = args.mode != 'failure'
                except Exception as error:
                    result['error'] = str(error)
                    try: screenshot(args.output.with_name(args.output.stem+'-error.png'), 'dlpad', display)
                    except Exception: pass
                finally:
                    if child.poll() is None:
                        os.killpg(child.pid, signal.SIGTERM)
                        try: child.wait(timeout=3)
                        except subprocess.TimeoutExpired:
                            os.killpg(child.pid, signal.SIGKILL); child.wait()
                    result['owned_gui_reaped'] = child.poll() is not None
        result['temporary_fixture_removed'] = True
    result['elapsed_seconds'] = round(time.monotonic()-started, 3)
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
    return int('error' in result)


def main():
    p = parser(); args = p.parse_args()
    if not 1 <= args.timeout <= 180:
        p.error('--timeout must be between 1 and 180 seconds')
    if args.output.suffix != '.json':
        p.error('output must end in .json')
    if args.validate_only:
        print(json.dumps({'arguments_valid': True, 'processes_started': False})); return 0
    args.binary = args.binary.resolve()
    if not args.binary.is_file() or not os.access(args.binary, os.X_OK):
        p.error('binary must be an executable file')
    if args.font is None or not args.font.is_file():
        p.error('--font or DLPAD_TEST_FONT must name a fixture font file')
    args.font = args.font.resolve()
    if os.uname().sysname != 'Linux':
        p.error('this probe validates Linux X11 only')
    args.output = args.output.resolve()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if any(args.output.parent.glob(args.output.stem+'.*')) or any(args.output.parent.glob(args.output.stem+'-*.png')):
        p.error('report prefix already exists; refusing to overwrite evidence')
    try:
        return run(args)
    except (Exception, KeyboardInterrupt) as error:
        result = {'mode': args.mode, 'display': 'Xvfb isolated', 'native_gui_verified': False,
                  'error': str(error) or type(error).__name__, 'desktop_fallback_used': False}
        args.output.write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps(result, indent=2))
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
