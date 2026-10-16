#!/usr/bin/env python3
"""Actual Hangul/XIM exercise confined to owned Xvfb and private D-Bus."""
import argparse
import hashlib
import ctypes as C
import ctypes.util
import json
import locale
import os
from pathlib import Path
import signal
import struct
import zlib
import select
import subprocess
import tempfile
import time

from native_probe import probe_display, screenshot
from x11_diagnostics import Selection, click_at


def key_sequence(display_name, keys):
    x = C.CDLL(ctypes.util.find_library('X11'))
    t = C.CDLL(ctypes.util.find_library('Xtst'))
    x.XOpenDisplay.argtypes = [C.c_char_p]
    x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XStringToKeysym.argtypes = [C.c_char_p]
    x.XStringToKeysym.restype = C.c_ulong
    x.XKeysymToKeycode.argtypes = [C.c_void_p, C.c_ulong]
    x.XKeysymToKeycode.restype = C.c_ubyte
    x.XFlush.argtypes = [C.c_void_p]
    t.XTestFakeKeyEvent.argtypes = [C.c_void_p, C.c_uint, C.c_int, C.c_ulong]
    display = x.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError('Owned Xvfb input unavailable')
    try:
        for item in keys:
            names = item.split('+')
            codes = [x.XKeysymToKeycode(display, x.XStringToKeysym(name.encode())) for name in names]
            if not all(codes):
                raise RuntimeError('Synthetic keysym unavailable')
            for code in codes:
                t.XTestFakeKeyEvent(display, code, 1, 0)
            for code in reversed(codes):
                t.XTestFakeKeyEvent(display, code, 0, 0)
            x.XFlush(display)
            time.sleep(0.12)
    finally:
        x.XCloseDisplay(display)


def window_inventory(display_name):
    x = C.CDLL(ctypes.util.find_library('X11'))
    x.XOpenDisplay.argtypes = [C.c_char_p]
    x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.restype = C.c_ulong
    x.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)), C.POINTER(C.c_uint)]
    x.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_char_p)]
    x.XGetGeometry.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_int), C.POINTER(C.c_int), C.POINTER(C.c_uint), C.POINTER(C.c_uint), C.POINTER(C.c_uint), C.POINTER(C.c_uint)]
    x.XFree.argtypes = [C.c_void_p]
    display = x.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError('Owned inventory display unavailable')
    result = []
    def visit(window, depth):
        if depth > 3 or len(result) >= 64:
            return
        name = C.c_char_p()
        x.XFetchName(display, window, C.byref(name))
        label = name.value.decode('utf-8', 'replace') if name.value else None
        if name:
            x.XFree(name)
        xx, yy, width, height, border, pixel_depth = C.c_int(), C.c_int(), C.c_uint(), C.c_uint(), C.c_uint(), C.c_uint()
        root = C.c_ulong()
        x.XGetGeometry(display, window, C.byref(root), C.byref(xx), C.byref(yy), C.byref(width), C.byref(height), C.byref(border), C.byref(pixel_depth))
        result.append({'window': window, 'depth': depth, 'name': label, 'x': xx.value, 'y': yy.value, 'width': width.value, 'height': height.value})
        root_out, parent = C.c_ulong(), C.c_ulong()
        children, count = C.POINTER(C.c_ulong)(), C.c_uint()
        x.XQueryTree(display, window, C.byref(root_out), C.byref(parent), C.byref(children), C.byref(count))
        try:
            for index in range(count.value):
                visit(children[index], depth+1)
        finally:
            if children:
                x.XFree(children)
    try:
        visit(x.XDefaultRootWindow(display), 0)
        return result
    finally:
        x.XCloseDisplay(display)


def xim_styles(display_name):
    class Styles(C.Structure):
        _fields_ = [('count', C.c_ushort), ('styles', C.POINTER(C.c_ulong))]
    locale.setlocale(locale.LC_CTYPE, 'C.UTF-8')
    x = C.CDLL(ctypes.util.find_library('X11'))
    x.XOpenDisplay.argtypes = [C.c_char_p]
    x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XSetLocaleModifiers.argtypes = [C.c_char_p]
    x.XSetLocaleModifiers.restype = C.c_char_p
    x.XOpenIM.argtypes = [C.c_void_p, C.c_void_p, C.c_char_p, C.c_char_p]
    x.XOpenIM.restype = C.c_void_p
    x.XCloseIM.argtypes = [C.c_void_p]
    x.XGetIMValues.argtypes = [C.c_void_p, C.c_char_p, C.POINTER(C.POINTER(Styles)), C.c_void_p]
    x.XGetIMValues.restype = C.c_char_p
    x.XFree.argtypes = [C.c_void_p]
    display = x.XOpenDisplay(display_name.encode())
    im = None
    styles = C.POINTER(Styles)()
    if not display:
        raise RuntimeError('Owned XIM query display unavailable')
    try:
        x.XSetLocaleModifiers(b'@im=fcitx')
        im = x.XOpenIM(display, None, None, None)
        if not im or x.XGetIMValues(im, b'queryInputStyle', C.byref(styles), None):
            raise RuntimeError('Owned XIM style query failed')
        if not styles or styles.contents.count > 64:
            raise RuntimeError('Owned XIM style count invalid')
        return [styles.contents.styles[i] for i in range(styles.contents.count)]
    finally:
        if styles:
            x.XFree(styles)
        if im:
            x.XCloseIM(im)
        x.XCloseDisplay(display)


def focus_owned_window(display_name):
    x = C.CDLL(ctypes.util.find_library('X11'))
    x.XOpenDisplay.argtypes = [C.c_char_p]
    x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.restype = C.c_ulong
    x.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)), C.POINTER(C.c_uint)]
    x.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_char_p)]
    x.XFree.argtypes = [C.c_void_p]
    x.XSetInputFocus.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_ulong]
    x.XFlush.argtypes = [C.c_void_p]
    display = x.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError('Owned focus display unavailable')
    try:
        root = x.XDefaultRootWindow(display)
        root_out, parent = C.c_ulong(), C.c_ulong()
        children, count = C.POINTER(C.c_ulong)(), C.c_uint()
        x.XQueryTree(display, root, C.byref(root_out), C.byref(parent), C.byref(children), C.byref(count))
        found = None
        try:
            for index in range(count.value):
                name = C.c_char_p()
                x.XFetchName(display, children[index], C.byref(name))
                if name.value and b'dlpad' in name.value:
                    found = children[index]
                if name:
                    x.XFree(name)
        finally:
            if children:
                x.XFree(children)
        if found is None:
            raise RuntimeError('Owned app focus window unavailable')
        x.XSetInputFocus(display, found, 2, 0)
        x.XFlush(display)
        time.sleep(.2)
    finally:
        x.XCloseDisplay(display)


def clipboard_text(display_name):
    x = C.CDLL(ctypes.util.find_library('X11'))
    x.XOpenDisplay.argtypes = [C.c_char_p]
    x.XOpenDisplay.restype = C.c_void_p
    x.XCloseDisplay.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.argtypes = [C.c_void_p]
    x.XDefaultRootWindow.restype = C.c_ulong
    x.XCreateSimpleWindow.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_int, C.c_uint, C.c_uint, C.c_uint, C.c_ulong, C.c_ulong]
    x.XCreateSimpleWindow.restype = C.c_ulong
    x.XDestroyWindow.argtypes = [C.c_void_p, C.c_ulong]
    x.XInternAtom.argtypes = [C.c_void_p, C.c_char_p, C.c_int]
    x.XInternAtom.restype = C.c_ulong
    x.XConvertSelection.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_ulong, C.c_ulong, C.c_ulong]
    x.XFlush.argtypes = [C.c_void_p]
    x.XPending.argtypes = [C.c_void_p]
    x.XNextEvent.argtypes = [C.c_void_p, C.c_void_p]
    x.XGetWindowProperty.argtypes = [C.c_void_p, C.c_ulong, C.c_ulong, C.c_long, C.c_long, C.c_int, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_int), C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ubyte))]
    x.XFree.argtypes = [C.c_void_p]
    display = x.XOpenDisplay(display_name.encode())
    window = None
    if not display:
        raise RuntimeError('Owned clipboard display unavailable')
    try:
        window = x.XCreateSimpleWindow(display, x.XDefaultRootWindow(display), 0, 0, 1, 1, 0, 0, 0)
        selection = x.XInternAtom(display, b'CLIPBOARD', 0)
        target = x.XInternAtom(display, b'UTF8_STRING', 0)
        property_atom = x.XInternAtom(display, b'DLPAD_IME_PROBE', 0)
        x.XConvertSelection(display, selection, target, property_atom, window, 0)
        x.XFlush(display)
        event = (C.c_long * 24)()
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            if not x.XPending(display):
                time.sleep(0.01)
                continue
            x.XNextEvent(display, C.byref(event))
            notification = C.cast(C.byref(event), C.POINTER(Selection)).contents
            if notification.type != 31:
                continue
            if not notification.property:
                raise RuntimeError('Owned clipboard conversion refused')
            actual, count, remaining = C.c_ulong(), C.c_ulong(), C.c_ulong()
            bits, data = C.c_int(), C.POINTER(C.c_ubyte)()
            status = x.XGetWindowProperty(display, window, property_atom, 0, 1024, 0, 0,
                                         C.byref(actual), C.byref(bits), C.byref(count), C.byref(remaining), C.byref(data))
            try:
                if status or bits.value != 8 or remaining.value or count.value > 4096:
                    raise RuntimeError('Unsupported/oversized IME clipboard result')
                return C.string_at(data, count.value).decode('utf-8')
            finally:
                if data:
                    x.XFree(data)
        raise RuntimeError('Owned clipboard timed out')
    finally:
        if window is not None:
            x.XDestroyWindow(display, window)
        x.XCloseDisplay(display)


def set_ime_mode(display, env, root, active):
    remote = root / 'usr/bin/fcitx5-remote'
    def status():
        value = subprocess.run([str(remote)], env=env, capture_output=True, timeout=5, check=True).stdout.strip()
        if value not in [b'1', b'2']:
            raise RuntimeError('Owned fcitx did not report an input context state')
        return int(value)
    wanted = 2 if active else 1
    if status() != wanted:
        key_sequence(display, ['Control_L+space'])
    if status() != wanted:
        raise RuntimeError('Real IME mode switch did not reach requested state')


def preedit_contrast(path, legacy):
    """Supporting counts for a fixed crop; not semantic or visual acceptance."""
    if path.stat().st_size > 4 * 1024 * 1024:
        raise RuntimeError("Owned screenshot exceeds size bound")
    data = path.read_bytes()
    cursor = 8
    compressed = bytearray()
    width = height = None
    while cursor < len(data):
        length = struct.unpack('!I', data[cursor:cursor+4])[0]
        kind = data[cursor+4:cursor+8]
        body = data[cursor+8:cursor+8+length]
        if kind == b'IHDR':
            width, height, depth, color, *_ = struct.unpack('!2I5B', body)
            if depth != 8 or color != 2:
                raise RuntimeError('Unexpected owned screenshot pixel format')
        if kind == b'IDAT':
            compressed.extend(body)
        cursor += length + 12
    if (width, height) != (960, 640):
        raise RuntimeError('IME crop requires fixed fixture window size')
    pixels = zlib.decompressobj().decompress(compressed, 4 * 1024 * 1024)
    black = white = 0
    crop = [48, 317, 75, 343] if legacy else [48, 308, 60, 324]
    for y in range(crop[1], crop[3]):
        if pixels[y*(width*3+1)] != 0:
            raise RuntimeError('Unexpected owned PNG filter')
        for x in range(crop[0], crop[2]):
            start = y*(width*3+1) + 1 + x*3
            color = pixels[start:start+3]
            black += int(max(color) < 40)
            white += int(min(color) > 220)
    return {'selection_dark_pixels': black, 'contrasting_glyph_pixels': white,
            'fixed_crop': crop}


def stop_owned(child):
    if child is not None and child.poll() is None:
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=3)


def gtk_control(output):
    """Independent GTK3 XIM client, using the same owned bus/display/IME."""
    gtk = C.CDLL(ctypes.util.find_library('gtk-3'))
    gtk.gtk_init_check.argtypes = [C.c_void_p, C.c_void_p]
    gtk.gtk_window_new.argtypes = [C.c_int]
    gtk.gtk_window_new.restype = C.c_void_p
    gtk.gtk_window_set_title.argtypes = [C.c_void_p, C.c_char_p]
    gtk.gtk_entry_new.restype = C.c_void_p
    gtk.gtk_entry_get_text.argtypes = [C.c_void_p]
    gtk.gtk_entry_get_text.restype = C.c_char_p
    gtk.gtk_container_add.argtypes = [C.c_void_p, C.c_void_p]
    gtk.gtk_widget_set_size_request.argtypes = [C.c_void_p, C.c_int, C.c_int]
    gtk.gtk_widget_show_all.argtypes = [C.c_void_p]
    gtk.gtk_widget_grab_focus.argtypes = [C.c_void_p]
    gtk.gtk_main_iteration_do.argtypes = [C.c_int]
    if not gtk.gtk_init_check(None, None):
        raise RuntimeError('Owned GTK control could not initialize')
    window = gtk.gtk_window_new(0)
    gtk.gtk_window_set_title(window, b'dlpad GTK XIM control')
    entry = gtk.gtk_entry_new()
    gtk.gtk_widget_set_size_request(entry, 480, 100)
    gtk.gtk_container_add(window, entry)
    gtk.gtk_widget_show_all(window)
    gtk.gtk_widget_grab_focus(entry)
    deadline = time.monotonic() + 30
    next_write = 0
    while time.monotonic() < deadline:
        while gtk.gtk_events_pending():
            gtk.gtk_main_iteration_do(0)
        if time.monotonic() >= next_write:
            pending = output.with_suffix('.pending')
            pending.write_text(json.dumps({'text': gtk.gtk_entry_get_text(entry).decode('utf-8')}))
            pending.replace(output)
            next_write = time.monotonic() + .1
        time.sleep(.01)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path, help='New temporary output directory')
    parser.add_argument('--inspect', action='store_true', help='Capture owned settings layout without typing')
    parser.add_argument('--validate-only', action='store_true')
    parser.add_argument('--legacy-preedit', action='store_true', help='Keep fcitx5 default external XIM preedit style')
    args = parser.parse_args()
    if args.validate_only:
        print('arguments valid; no processes started')
        return 0
    output = args.output.resolve()
    if not output.is_relative_to(Path('/tmp')):
        parser.error('Output must be a fresh /tmp directory')
    output.mkdir(exist_ok=False)
    root = Path(__file__).resolve().parents[1] / '.local-tools/ime/root'
    binary = args.binary.resolve(strict=True)
    result = {'ime_verified': False, 'preedit_verified': False, 'cancel_verified': False,
              'display': 'owned Xvfb only', 'ime': 'fcitx5-hangul XIM', 'binary': str(binary),
              'on_the_spot': not args.legacy_preedit}
    digest = hashlib.sha256()
    with binary.open('rb') as source:
        while chunk := source.read(65536): digest.update(chunk)
    result['binary_sha256'] = digest.hexdigest()
    result['binary_bytes'] = binary.stat().st_size
    bus = ime = app = None
    previous_authority = os.environ.get("XAUTHORITY")
    try:
        with tempfile.TemporaryDirectory(prefix='owned-ime-') as temporary:
            task = Path(temporary)
            for name in ['config', 'data', 'cache', 'runtime', 'app-data', 'media']:
                (task / name).mkdir(mode=0o700)
            authority = task / 'runtime/owned.Xauthority'
            authority.write_bytes(b'')
            os.environ['XAUTHORITY'] = str(authority)
            font_dir = task / 'data/fonts'
            font_dir.mkdir()
            (font_dir / 'Malgun.ttf').symlink_to('/mnt/c/Windows/Fonts/malgun.ttf')
            font_config = task / 'config/fonts.conf'
            font_config.write_text('<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "fonts.dtd"><fontconfig><include ignore_missing="yes">/etc/fonts/fonts.conf</include><dir>' + str(font_dir) + '</dir><cachedir>' + str(task / 'cache/fontconfig') + '</cachedir></fontconfig>')
            fcitx_config = task / 'config/fcitx5'
            fcitx_config.mkdir()
            (fcitx_config / 'conf').mkdir()
            (fcitx_config / 'conf/xim.conf').write_text('UseOnTheSpot=' + ('False' if args.legacy_preedit else 'True') + '\n')
            (fcitx_config / 'profile').write_text('[Groups/0]\nName=Default\nDefault Layout=us\nDefaultIM=hangul\n\n[Groups/0/Items/0]\nName=keyboard-us\nLayout=\n\n[Groups/0/Items/1]\nName=hangul\nLayout=\n\n[GroupOrder]\n0=Default\n')
            with probe_display(output / 'isolated.json') as (display, _):
                env = dict(os.environ)
                env.pop('WAYLAND_DISPLAY', None)
                env['XAUTHORITY'] = str(authority)
                env.update(FONTCONFIG_FILE=str(font_config), DISPLAY=display, XMODIFIERS='@im=fcitx', GTK_IM_MODULE='fcitx', QT_IM_MODULE='fcitx',
                           XDG_CONFIG_HOME=str(task / 'config'), XDG_DATA_HOME=str(task / 'data'),
                           XDG_CACHE_HOME=str(task / 'cache'), XDG_RUNTIME_DIR=str(task / 'runtime'),
                           FCITX_CONFIG_HOME=str(fcitx_config), FCITX_DATA_DIRS=str(root / 'usr/share/fcitx5'),
                           FCITX_ADDON_DIRS=str(root / 'usr/lib/x86_64-linux-gnu/fcitx5'),
                           LD_LIBRARY_PATH=str(root / 'usr/lib/x86_64-linux-gnu') + ':' + str(root.parents[1] / 'xvfb/usr/lib/x86_64-linux-gnu'),
                           DLPAD_TEST_DATA=str(task / 'app-data'), DLPAD_LIBRARY_ROOT=str(task / 'media'),
                           DLPAD_TEST_FONT='/mnt/c/Windows/Fonts/malgun.ttf', LANG='C.UTF-8', LC_ALL='C.UTF-8')
                bus_log = (output / 'bus.log').open('wb')
                bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', '--print-address=1', '--address=unix:path=' + str(task / 'runtime/bus')],
                                       env=env, stdout=subprocess.PIPE, stderr=bus_log, start_new_session=True)
                if not select.select([bus.stdout], [], [], 10)[0]:
                    raise RuntimeError('Private D-Bus readiness timed out')
                env['DBUS_SESSION_BUS_ADDRESS'] = bus.stdout.readline(4096).decode().strip()
                if not env['DBUS_SESSION_BUS_ADDRESS'].startswith('unix:'):
                    raise RuntimeError('Private D-Bus did not provide address')
                with (output / 'fcitx.log').open('wb') as logs:
                    mapper = Path(__file__).resolve().parents[1] / '.local-tools/xvfb/usr/bin/proot'
                    ime = subprocess.Popen([str(mapper), '-b', str(root / 'usr/share/libhangul') + ':/usr/share/libhangul', str(root / 'usr/bin/fcitx5'), '-D', '--disable', 'all', '--enable', 'keyboard,hangul,xcb,xim,dbus,classicui', '--verbose', 'key_trace=5'],
                                           env=env, stdin=subprocess.DEVNULL, stdout=logs, stderr=subprocess.STDOUT, start_new_session=True)
                    time.sleep(3)
                    if ime.poll() is not None:
                        raise RuntimeError(f'Owned IME exited: {ime.returncode}')
                    with (output / 'app.log').open('wb') as logs_app:
                        app = subprocess.Popen([str(binary), '--fixture', '--settings'], env=env,
                                               stdin=subprocess.DEVNULL, stdout=logs_app, stderr=subprocess.STDOUT, start_new_session=True)
                        time.sleep(2)
                        if app.poll() is not None:
                            raise RuntimeError(f'Owned GUI exited: {app.returncode}')
                        config = subprocess.run(['gdbus', 'call', '--session', '--dest', 'org.fcitx.Fcitx5', '--object-path', '/controller', '--method', 'org.fcitx.Fcitx.Controller1.GetConfig', 'fcitx://config/addon/xim'], env=env, capture_output=True, timeout=5)
                        result['xim_config_observed'] = config.stdout.decode('utf-8', 'replace')[:4096]
                        result['xim_config_error'] = config.stderr.decode('utf-8', 'replace')[:512]
                        result['xim_server_styles'] = xim_styles(display)
                        origin = screenshot(output / 'initial.png', 'dlpad', display)
                        if args.inspect:
                            result['inspection_only'] = True
                        else:
                            click_at(display, origin, 400, 303)
                            focus_owned_window(display)
                            set_ime_mode(display, env, root, True)
                            key_sequence(display, list('gksrmf'))
                            screenshot(output / 'preedit.png', 'dlpad', display)
                            result['preedit_windows'] = window_inventory(display)
                            result['preedit_pixels'] = preedit_contrast(output / 'preedit.png', args.legacy_preedit)
                            result['legacy_preedit_pixel_proxy'] = result['preedit_pixels']['selection_dark_pixels'] > 64 and result['preedit_pixels']['contrasting_glyph_pixels'] > 8
                            result['preedit_requires_screenshot_review'] = True
                            key_sequence(display, ['space'])
                            screenshot(output / 'committed.png', 'dlpad', display)
                            key_sequence(display, ['Control_L+a', 'Control_L+c'])
                            result['committed_text'] = clipboard_text(display)
                            result['commit_verified'] = result['committed_text'] == '한글 '
                            result['preedit_evidence'] = 'preedit.png; compare input composition with committed-only mirror underneath'
                            if not result['commit_verified']:
                                raise RuntimeError('Real Hangul key composition did not produce expected committed text')
                            key_sequence(display, ['End', 'g', 'k'])
                            screenshot(output / 'before-escape.png', 'dlpad', display)
                            key_sequence(display, ['Escape'])
                            screenshot(output / 'cancel.png', 'dlpad', display)
                            click_at(display, origin, 400, 303)
                            focus_owned_window(display)
                            key_sequence(display, ['Control_L+a', 'Control_L+c'])
                            result['after_escape_text'] = clipboard_text(display)
                            result['escape_cancels'] = result['after_escape_text'] == '한글 '
                            key_sequence(display, ['End'])
                            set_ime_mode(display, env, root, True)
                            key_sequence(display, ['g', 'k'])
                            screenshot(output / 'before-backspace-cancel.png', 'dlpad', display)
                            key_sequence(display, ['BackSpace', 'BackSpace'])
                            screenshot(output / 'backspace-cancel.png', 'dlpad', display)
                            key_sequence(display, ['Control_L+a', 'Control_L+c'])
                            result['after_backspace_cancel_text'] = clipboard_text(display)
                            result['cancel_verified'] = result['after_backspace_cancel_text'] == result['after_escape_text']
                            key_sequence(display, ['End'])
                            set_ime_mode(display, env, root, False)
                            key_sequence(display, ['a', 'b', 'c'])
                            key_sequence(display, ['Control_L+a', 'Control_L+c'])
                            result['latin_transition_text'] = clipboard_text(display)
                            result['latin_transition_verified'] = result['latin_transition_text'] == result['after_escape_text'] + 'abc'
                            key_sequence(display, ['Control_L+a', 'x', 'y', 'Control_L+a', 'Control_L+c'])
                            result['selection_replace_text'] = clipboard_text(display)
                            result['selection_replace_verified'] = result['selection_replace_text'] == 'xy'
                            screenshot(output / 'selection.png', 'dlpad', display)
                            stop_owned(app)
                            env_gtk = dict(env)
                            env_gtk['GTK_IM_MODULE'] = 'xim'
                            control_file = task / 'gtk-control.json'
                            app = subprocess.Popen(['python3', str(Path(__file__).resolve()), '--gtk-control', str(control_file)],
                                                   env=env_gtk, stdin=subprocess.DEVNULL, stdout=logs_app, stderr=subprocess.STDOUT, start_new_session=True)
                            gtk_deadline = time.monotonic() + 10
                            while not control_file.exists():
                                if app.poll() is not None:
                                    raise RuntimeError('Owned GTK control exited before readiness')
                                if time.monotonic() >= gtk_deadline:
                                    raise RuntimeError('Owned GTK control readiness timed out')
                                time.sleep(.05)
                            origin_gtk = screenshot(output / 'gtk-initial.png', 'dlpad', display)
                            click_at(display, origin_gtk, 180, 45)
                            focus_owned_window(display)
                            set_ime_mode(display, env_gtk, root, True)
                            key_sequence(display, [*list('gksrmf'), 'space'])
                            time.sleep(.2)
                            result['gtk_committed_text'] = json.loads(control_file.read_text())['text']
                            key_sequence(display, ['g', 'k', 'Escape'])
                            time.sleep(.2)
                            result['gtk_after_escape_text'] = json.loads(control_file.read_text())['text']
                            screenshot(output / 'gtk-after-escape.png', 'dlpad', display)
                            result['escape_same_as_gtk'] = result['after_escape_text'] == result['gtk_after_escape_text']
                            result['ime_verified'] = result['commit_verified'] and result['preedit_verified'] and result['cancel_verified'] and result['latin_transition_verified']

    except Exception as error:
        result['error'] = str(error)
    finally:
        result['ime_verified'] = bool(result.get('commit_verified') and result['preedit_verified'] and result['cancel_verified'] and result.get('latin_transition_verified') and result.get('escape_same_as_gtk') and result.get('selection_replace_verified') and not result.get('error'))
        for child in [app, ime, bus]:
            stop_owned(child)
        result['owned_children_reaped'] = all(child is None or child.poll() is not None for child in [app, ime, bus])
        result['temporary_fixture_removed'] = True
        if previous_authority is None:
            os.environ.pop('XAUTHORITY', None)
        else:
            os.environ['XAUTHORITY'] = previous_authority
        (output / 'result.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
        print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if not result.get('error') and (args.inspect or result['ime_verified']) else 1


if __name__ == '__main__':
    if len(os.sys.argv) == 3 and os.sys.argv[1] == '--gtk-control':
        gtk_control(Path(os.sys.argv[2]))
    else:
        raise SystemExit(main())
