#!/usr/bin/env python3
"""Linux X11 fixture probe; owns launched process, never uses user Documents.
Measures RSS and process CPU ticks. Unicode rendering is not an IME test.
"""
import argparse
import contextlib
import ctypes as C
import ctypes.util
import json
import os
from pathlib import Path
import signal
import select
import shutil
import struct
import sqlite3
import subprocess
import time
import tempfile
import zlib


def png(path, width, height, data):
    def chunk(kind, body):
        return struct.pack('!I', len(body)) + kind + body + struct.pack('!I', zlib.crc32(kind + body))
    rows = b''.join(b'\0' + data[y * width * 3:(y + 1) * width * 3] for y in range(height))
    path.write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('!2I5B', width, height, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(rows)) + chunk(b'IEND', b''))


class XImage(C.Structure):
    _fields_ = [('width', C.c_int), ('height', C.c_int), ('xoffset', C.c_int), ('format', C.c_int), ('data', C.c_void_p), ('byte_order', C.c_int), ('bitmap_unit', C.c_int), ('bitmap_bit_order', C.c_int), ('bitmap_pad', C.c_int), ('depth', C.c_int), ('bytes_per_line', C.c_int), ('bits_per_pixel', C.c_int), ('red_mask', C.c_ulong), ('green_mask', C.c_ulong), ('blue_mask', C.c_ulong)]


def screenshot(path, title, display_name):
    lib = C.CDLL(ctypes.util.find_library('X11'))
    lib.XOpenDisplay.restype = C.c_void_p
    lib.XDefaultRootWindow.argtypes = [C.c_void_p]
    lib.XDefaultRootWindow.restype = C.c_ulong
    lib.XQueryTree.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_ulong), C.POINTER(C.POINTER(C.c_ulong)), C.POINTER(C.c_uint)]
    lib.XFetchName.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_char_p)]
    lib.XGetGeometry.argtypes = [C.c_void_p, C.c_ulong, C.POINTER(C.c_ulong), C.POINTER(C.c_int), C.POINTER(C.c_int), C.POINTER(C.c_uint), C.POINTER(C.c_uint), C.POINTER(C.c_uint), C.POINTER(C.c_uint)]
    lib.XGetImage.argtypes = [C.c_void_p, C.c_ulong, C.c_int, C.c_int, C.c_uint, C.c_uint, C.c_ulong, C.c_int]
    lib.XGetImage.restype = C.POINTER(XImage)
    lib.XFree.argtypes = [C.c_void_p]
    lib.XDestroyImage.argtypes = [C.POINTER(XImage)]
    lib.XCloseDisplay.argtypes = [C.c_void_p]
    lib.XOpenDisplay.argtypes = [C.c_char_p]
    display = lib.XOpenDisplay(display_name.encode())
    if not display:
        raise RuntimeError('X11 display unavailable')
    root = lib.XDefaultRootWindow(display)
    def find(window, depth=0):
        name = C.c_char_p()
        lib.XFetchName(display, window, C.byref(name))
        matched = name.value and title in name.value.decode('utf-8', 'replace')
        if name:
            lib.XFree(name)
        if matched:
            return window
        if depth > 5:
            return None
        root_out, parent, children, count = C.c_ulong(), C.c_ulong(), C.POINTER(C.c_ulong)(), C.c_uint()
        lib.XQueryTree(display, window, C.byref(root_out), C.byref(parent), C.byref(children), C.byref(count))
        result = None
        for i in range(count.value):
            result = find(children[i], depth + 1)
            if result:
                break
        if children:
            lib.XFree(children)
        return result
    try:
        window = find(root)
        if not window:
            raise RuntimeError('owned fixture window not found')
        x, y, w, h, border, depth = C.c_int(), C.c_int(), C.c_uint(), C.c_uint(), C.c_uint(), C.c_uint()
        root_geometry = C.c_ulong()
        lib.XGetGeometry(display, window, C.byref(root_geometry), C.byref(x), C.byref(y), C.byref(w), C.byref(h), C.byref(border), C.byref(depth))
        im = lib.XGetImage(display, window, 0, 0, w, h, C.c_ulong(-1), 2)
        if not im:
            raise RuntimeError('X11 screenshot unavailable')
        try:
            info = im.contents
            if info.bits_per_pixel != 32:
                raise RuntimeError('unsupported X11 pixel format')
            raw = C.string_at(info.data, info.bytes_per_line * info.height)
            rgb = bytearray(info.width * info.height * 3)
            endian = 'little' if info.byte_order == 0 else 'big'
            def channel(value, mask):
                shift = (mask & -mask).bit_length() - 1
                return ((value & mask) >> shift) * 255 // (mask >> shift)
            for yy in range(info.height):
                for xx in range(info.width):
                    offset = yy * info.bytes_per_line + xx * 4
                    pixel = int.from_bytes(raw[offset:offset+4], endian)
                    target = (yy * info.width + xx) * 3
                    rgb[target:target+3] = bytes(channel(pixel, mask) for mask in (info.red_mask, info.green_mask, info.blue_mask))
            png(path, info.width, info.height, rgb)
            return {'width': info.width, 'height': info.height, 'x': x.value, 'y': y.value}
        finally:
            lib.XDestroyImage(im)
    finally:
        lib.XCloseDisplay(display)


def sample(pid):
    fields = Path(f'/proc/{pid}/stat').read_text().split(') ', 1)[1].split()
    status = Path(f'/proc/{pid}/status').read_text().splitlines()
    rss = next(int(line.split()[1]) for line in status if line.startswith('VmRSS:'))
    return int(fields[11]) + int(fields[12]), rss


@contextlib.contextmanager
def probe_display(output, desktop=False, screen=(1280, 900)):
    if (not isinstance(screen, tuple) or len(screen) != 2
            or any(type(value) is not int or value < 1 for value in screen)
            or screen[0] > 4096 or screen[1] > 2160
            or screen[0] * screen[1] > 8 * 1024 * 1024):
        raise ValueError("Owned display screen exceeds pixel budget")
    if desktop:
        display = os.environ.get('DISPLAY')
        if not display:
            raise RuntimeError('Explicit desktop probe requires DISPLAY')
        yield display, 'desktop X11 (explicit)'
        return
    root = Path(__file__).resolve().parents[1] / '.local-tools' / 'xvfb'
    executable = shutil.which('Xvfb')
    if not executable and (root / 'usr/bin/Xvfb').is_file():
        executable = str(root / 'usr/bin/Xvfb')
    if not executable:
        raise RuntimeError('Xvfb required: refusing to open a test window on your desktop')
    read_fd, write_fd = os.pipe()
    env = dict(os.environ)
    libraries = root / 'usr/lib/x86_64-linux-gnu'
    if libraries.is_dir():
        env['LD_LIBRARY_PATH'] = str(libraries) + (':' + env['LD_LIBRARY_PATH'] if env.get('LD_LIBRARY_PATH') else '')
    server_command = [executable]
    # Some extracted X servers hard-code /usr/bin/xkbcomp. Map only that path
    # inside this owned test process, without installing anything globally.
    compiler = root / 'usr/bin/xkbcomp'
    mapper = root / 'usr/bin/proot'
    if not Path('/usr/bin/xkbcomp').exists() and compiler.is_file() and mapper.is_file():
        server_command = [str(mapper), '-b', str(compiler) + ':/usr/bin/xkbcomp', executable]
    server = None
    try:
        with output.with_suffix('.xvfb.log').open('wb') as logs:
            server = subprocess.Popen(server_command + ['-displayfd', str(write_fd), '-screen', '0', f'{screen[0]}x{screen[1]}x24', '-nolisten', 'tcp', '-nolisten', 'unix', '-noreset'],
                                      env=env, pass_fds=(write_fd,), stdin=subprocess.DEVNULL,
                                      stdout=logs, stderr=subprocess.STDOUT, start_new_session=True)
            os.close(write_fd)
            write_fd = None
            ready, _, _ = select.select([read_fd], [], [], 10)
            if not ready:
                raise RuntimeError('Isolated X server startup timed out')
            number = os.read(read_fd, 32).decode().strip()
            if not number.isdigit():
                raise RuntimeError('Isolated X server did not provide a display')
            yield ':' + number, 'Xvfb isolated'
    finally:
        os.close(read_fd)
        if write_fd is not None:
            os.close(write_fd)
        if server is not None:
            if server.poll() is None:
                os.killpg(server.pid, signal.SIGTERM)
                try:
                    server.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    os.killpg(server.pid, signal.SIGKILL)
                    server.wait()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--page', choices=['library', 'settings', 'diagnostics', 'downloads', 'history', 'tools'], default='library')
    parser.add_argument('--corrupt-journal', action='store_true', help='Exercise startup recovery failure with an owned corrupt fixture')
    parser.add_argument('--recover-jobs', action='store_true', help='Seed an owned temporary interrupted job journal')
    parser.add_argument('--pause-job', action='store_true', help='Pause a running synthetic resumed job on isolated display')
    parser.add_argument('--resume-job', action='store_true', help='Click synthetic resume on isolated history page')
    parser.add_argument('--scroll-end', action='store_true', help='Scroll within isolated fixture only')
    parser.add_argument('--scroll-steps', type=int, default=0, help='0..30 wheel steps within isolated fixture only')
    parser.add_argument('--advanced', action='store_true', help='Open download advanced controls')
    parser.add_argument('--seconds', type=float, default=10)
    parser.add_argument('--copy-report', action='store_true', help='Verify diagnostic clipboard on isolated display only')
    parser.add_argument('--default-library', action='store_true', help='Exercise first-start default location with injected temporary Documents')
    parser.add_argument('--live-library', action='store_true', help='Create temporary synthetic files and exercise real scan/watch adapters')
    parser.add_argument('--desktop', action='store_true', help='Explicitly allow a visible desktop test; default is isolated Xvfb')
    args = parser.parse_args()
    if args.pause_job and not args.resume_job:
        parser.error("--pause-job requires --resume-job")
    if args.resume_job and (args.desktop or args.page != "history" or not args.recover_jobs or args.corrupt_journal):
        parser.error("--resume-job requires isolated --page history --recover-jobs")
    if args.copy_report and (args.desktop or args.page != 'diagnostics'):
        parser.error('--copy-report requires isolated --page diagnostics')
    if not 0 <= args.scroll_steps <= 30 or (args.scroll_steps and args.scroll_end):
        parser.error('--scroll-steps requires 0..30 and cannot combine with --scroll-end')
    if args.scroll_steps and args.desktop:
        parser.error('--scroll-steps requires isolated display')
    if args.scroll_end and args.desktop:
        parser.error('--scroll-end requires isolated display')
    if args.corrupt_journal and not args.recover_jobs:
        parser.error('--corrupt-journal requires --recover-jobs')
    if args.recover_jobs and (args.page not in ['downloads', 'history'] or args.live_library or args.default_library):
        parser.error('--recover-jobs requires --page downloads/history without library exercises')
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if (args.live_library or args.default_library) and args.page != 'library':
        parser.error('Library exercises require --page library')
    if args.live_library and args.default_library:
        parser.error('Choose either live or default library exercise')
    with probe_display(args.output, args.desktop) as (display, display_kind):
        if args.recover_jobs:
            with tempfile.TemporaryDirectory(prefix='dlpad-probe-recovery-', dir=args.output.parent) as temporary:
                data = Path(temporary) / 'app-data'; data.mkdir()
                job = {'id': 'crashed-fixture', 'state': 'Downloading', 'revision': 2, 'attempts': [{'id': 'previous-attempt', 'number': 1}], 'artifacts': []}
                with sqlite3.connect(data / 'jobs.sqlite') as database:
                    database.execute('CREATE TABLE download_jobs(id TEXT PRIMARY KEY,revision INTEGER NOT NULL,job_json TEXT NOT NULL,snapshot TEXT NOT NULL,tool_path TEXT,tool_version TEXT)')
                    database.execute('INSERT INTO download_jobs VALUES(?,2,?,?,NULL,NULL)', ('crashed-fixture', json.dumps(job), '[]'))
                if args.resume_job:
                    root = data.parent / 'synthetic-media'; root.mkdir()
                    options = {'version': 1, 'url': 'https://example.org/synthetic', 'output_dir': str(root.resolve()), 'quality': 'Best', 'time_range': None, 'advanced': [], 'ffmpeg': None}
                    snapshot = ['--ignore-config', '--no-plugin-dirs', '--newline', '--no-playlist', '--retries', '3', '--paths', str(root.resolve()), '--format', 'bv*+ba/b', '--', options['url']]
                    with sqlite3.connect(data / 'jobs.sqlite') as database:
                        database.execute('ALTER TABLE download_jobs ADD COLUMN options_json TEXT')
                        database.execute('UPDATE download_jobs SET options_json=?,snapshot=?', (json.dumps(options), json.dumps(snapshot, separators=(',', ':'))))
                    tool = data / 'yt-dlp-synthetic'
                    tool.write_text("#!/usr/bin/python3\nimport sys,json,pathlib\na=sys.argv[1:]\nif '--version' in a: print('2026.10.01');sys.exit(0)\nif '--dump-single-json' in a: print(json.dumps({'id':'synthetic','duration':10}));sys.exit(0)\np=pathlib.Path(a[a.index('--paths')+1])/a[a.index('--output')+1].replace('%(ext)s','mp4')\np.write_bytes(b'synthetic-media')\nprint('DLPAD_ARTIFACT:'+json.dumps(str(p)))\n")
                    if args.pause_job:
                        code = tool.read_text().replace("p.write_bytes(b'synthetic-media')", "p.with_suffix('.part').write_bytes(b'paused partial')\nimport time; time.sleep(30)\np.write_bytes(b'synthetic-media')")
                        tool.write_text(code)
                    tool.chmod(0o700)
                if args.corrupt_journal:
                    (data / 'jobs.sqlite').write_bytes(b'synthetic corrupt journal')
                return run_probe(args, display, display_kind, data=data)
        if args.live_library or args.default_library:
            with tempfile.TemporaryDirectory(prefix='dlpad-probe-media-', dir=args.output.parent) as media:
                root = Path(media)
                if args.default_library:
                    with tempfile.TemporaryDirectory(prefix='dlpad-probe-data-', dir=args.output.parent) as data:
                        return run_probe(args, display, display_kind, root, Path(data))
                for index in range(20):
                    (root / f'합성-{index:03}.mp4').write_bytes(b'synthetic library fixture')
                return run_probe(args, display, display_kind, root)
        else:
            return run_probe(args, display, display_kind)


def run_probe(args, display, display_kind, media=None, data=None):
    env = dict(os.environ)
    env['DISPLAY'] = display
    if args.default_library:
        env.pop('DLPAD_LIBRARY_ROOT', None)
        env['DLPAD_TEST_DOCUMENTS'] = str(media.resolve())
    elif media is not None:
        env['DLPAD_LIBRARY_ROOT'] = str(media.resolve())
    if args.resume_job:
        env['DLPAD_YTDLP'] = str((data / 'yt-dlp-synthetic').resolve())
    env.pop('WAYLAND_DISPLAY', None)
    env['MESA_SHADER_CACHE_DIR'] = str(args.output.parent / 'sandbox' / 'shader-cache')
    env['DLPAD_TEST_DATA'] = str(data.resolve() if data is not None else args.output.parent / 'sandbox')
    result = {'binary': str(args.binary), 'bytes': args.binary.stat().st_size, 'platform': os.uname().release, 'display': display_kind, 'font': bool(env.get('DLPAD_TEST_FONT')), 'live_library': media is not None, 'ime_verified': False, 'screen_reader_verified': False}
    with args.output.with_suffix('.log').open('wb') as logs:
        child = subprocess.Popen([str(args.binary), *([] if media is not None else ['--fixture']), '--' + args.page, *(['--advanced'] if args.advanced else [])], env=env, stdout=logs, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            time.sleep(2)
            if child.poll() is not None:
                raise RuntimeError(f'GUI exited with {child.returncode}')
            if args.default_library:
                root = media / 'dlpad'
                if not root.is_dir():
                    raise RuntimeError('Default Documents/dlpad was not created')
                for index in range(20):
                    (root / f'합성-{index:03}.mp4').write_bytes(b'synthetic library fixture')
                result['default_library_created'] = True
                time.sleep(1)
            if args.resume_job:
                from x11_diagnostics import click_at
                geometry = screenshot(args.output.with_suffix('.png'), 'dlpad', display)
                click_at(display, geometry, 914, 290)
                if args.pause_job:
                    partial = data.parent / 'synthetic-media' / '.dlpad-partials' / 'crashed-fixture' / 'crashed-fixture.part'
                    deadline = time.monotonic() + 5
                    while not partial.is_file() and time.monotonic() < deadline:
                        time.sleep(0.05)
                    if not partial.is_file(): raise RuntimeError('Synthetic resume never entered downloading')
                    click_at(display, geometry, 60, 76)
                    time.sleep(0.3)
                    from x11_diagnostics import scroll_to_end
                    scroll_to_end(display, geometry)
                    screenshot(args.output.with_name(args.output.stem+'-pause-ready.png'), 'dlpad', display)
                    click_at(display, geometry, 190, 435)
            if args.copy_report:
                from x11_diagnostics import copy_report
                geometry = screenshot(args.output.with_suffix('.png'), 'dlpad', display)
                result.update(copy_report(display, geometry))
            if args.scroll_end:
                from x11_diagnostics import scroll_to_end
                geometry = screenshot(args.output.with_suffix('.png'), 'dlpad', display)
                scroll_to_end(display, geometry)
            if args.scroll_steps:
                from x11_diagnostics import scroll_to_end
                geometry = screenshot(args.output.with_suffix('.png'), 'dlpad', display)
                scroll_to_end(display, geometry, args.scroll_steps)
                result['scroll_steps'] = args.scroll_steps
            before, rss = sample(child.pid)
            start = time.monotonic()
            time.sleep(args.seconds)
            after, rss_after = sample(child.pid)
            elapsed = time.monotonic() - start
            result.update(rss_kib=rss_after, initial_rss_kib=rss, idle_cpu_percent=(after-before)/os.sysconf('SC_CLK_TCK')/elapsed*100, sample_seconds=elapsed)
            if args.default_library:
                with sqlite3.connect((data / 'theme.sqlite').resolve().as_uri() + '?mode=ro', uri=True) as database:
                    saved = database.execute("SELECT value FROM settings WHERE key='library_root'").fetchone()[0]
                if not os.path.samefile(saved, root):
                    raise RuntimeError('Default library setting was not persisted')
                with sqlite3.connect((data / 'library.sqlite').resolve().as_uri() + '?mode=ro', uri=True) as database:
                    count = database.execute('SELECT COUNT(*) FROM library_entries WHERE root=? AND present=1', (saved,)).fetchone()[0]
                result['default_library_observed_files'] = count
                if count != 20:
                    raise RuntimeError('Native watcher did not reconcile the 20 generated files')
            if args.recover_jobs:
                if args.corrupt_journal:
                    if (data / 'jobs.sqlite').read_bytes() != b'synthetic corrupt journal':
                        raise RuntimeError('Startup changed the corrupt fixture journal')
                    result['startup_corrupt_journal_preserved'] = True
                else:
                    with sqlite3.connect((data / 'jobs.sqlite').resolve().as_uri() + '?mode=ro', uri=True) as database:
                        job = json.loads(database.execute("SELECT job_json FROM download_jobs WHERE id='crashed-fixture'").fetchone()[0])
                    if args.pause_job:
                        partial = data.parent / 'synthetic-media' / '.dlpad-partials' / 'crashed-fixture' / 'crashed-fixture.part'
                        if job['state'] != 'Paused' or len(job['attempts']) != 2 or partial.read_bytes() != b'paused partial':
                            raise RuntimeError(f"Pause GUI failed: state={job['state']}, attempts={len(job['attempts'])}")
                        result['history_pause_verified'] = True
                    elif args.resume_job:
                        if job['state'] != 'Downloaded' or len(job['attempts']) != 2 or not job['artifacts'] or not Path(job['artifacts'][0]['path']).is_file():
                            raise RuntimeError(f"History resume failed: state={job['state']}, attempts={len(job['attempts'])}, artifacts={len(job['artifacts'])}")
                        result['history_resume_verified'] = True
                    elif job['state'] != 'Interrupted' or len(job['attempts']) != 1 or job['artifacts']:
                        raise RuntimeError('Startup did not reconcile the interrupted fixture without auto-restart')
                    result['startup_recovered_job'] = True
            result['screenshot'] = screenshot(args.output.with_suffix('.png'), 'dlpad', display)
        except Exception as error:
            result['error'] = str(error)
        finally:
            if child.poll() is None:
                os.killpg(child.pid, signal.SIGTERM)
                try:
                    child.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
            result['probe_child_reaped'] = child.poll() is not None
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    return 1 if result.get("error") else 0


if __name__ == '__main__':
    raise SystemExit(main())
