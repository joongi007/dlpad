#!/usr/bin/env python3
"""Persisted theme observations on owned Xvfb/private bus; no desktop input."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import sqlite3
import struct
import subprocess
import tempfile
from types import SimpleNamespace
import zlib
from native_probe import probe_display, run_probe


def pixel(path, x, y):
    raw = path.read_bytes()
    if len(raw) > 8 * 1024 * 1024 or raw[:8] != b'\x89PNG\r\n\x1a\n':
        raise RuntimeError('unsupported fixture image')
    offset, compressed = 8, bytearray()
    while offset < len(raw):
        size = struct.unpack('!I', raw[offset:offset+4])[0]
        kind = raw[offset+4:offset+8]
        body = raw[offset+8:offset+8+size]
        if kind == b'IHDR':
            width, height, depth, color, *_ = struct.unpack('!2I5B', body)
            if depth != 8 or color != 2 or width * height > 2_000_000:
                raise RuntimeError('unsupported bounded image format')
        elif kind == b'IDAT':
            compressed.extend(body)
        offset += size + 12
    count = (width * 3 + 1) * height
    decoder = zlib.decompressobj()
    rows = decoder.decompress(compressed, count + 1)
    if len(rows) != count or not decoder.eof:
        raise RuntimeError('unexpected image size')
    stride = width * 3 + 1
    if rows[y * stride] != 0:
        raise RuntimeError('unsupported row filter')
    start = y * stride + 1 + x * 3
    return list(rows[start:start+3])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    binary = args.binary.resolve(strict=True)
    initial_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    result = {'binary_sha256': initial_hash, 'display': 'owned Xvfb',
              'desktop_fallback_used': False, 'first_frame_verified': False,
              'system_theme_change_verified': False, 'cases': []}
    with tempfile.TemporaryDirectory(prefix='dlpad-theme-') as directory:
        temporary = Path(directory)
        with (args.output / 'bus.log').open('wb') as logs:
            bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', '--print-address=1'],
                                   stdout=subprocess.PIPE, stderr=logs, start_new_session=True)
            try:
                ready, _, _ = select.select([bus.stdout], [], [], 5)
                if not ready:
                    raise RuntimeError('private bus startup timeout')
                address = bus.stdout.readline(4096).decode().strip()
                if not address.startswith('unix:'):
                    raise RuntimeError('invalid private bus')
                os.environ['DBUS_SESSION_BUS_ADDRESS'] = address
                os.environ['XDG_CONFIG_HOME'] = str(temporary / 'config')
                os.environ['XDG_CACHE_HOME'] = str(temporary / 'cache')
                runtime = temporary / 'runtime'
                runtime.mkdir(mode=0o700)
                os.environ['XDG_RUNTIME_DIR'] = str(runtime)
                cases = [('sqlite-dark', 'Dark', 'Light', True),
                         ('sqlite-light', 'Light', 'Dark', False),
                         ('legacy-dark', None, 'Dark', True),
                         ('oversized-legacy', None, 'x' * 4097, False)]
                for name, selected, legacy, expected_dark in cases:
                    data = temporary / name
                    data.mkdir(mode=0o700)
                    legacy_bytes = json.dumps(legacy).encode()
                    (data / 'theme.json').write_bytes(legacy_bytes)
                    if selected is not None:
                        with sqlite3.connect(data / 'theme.sqlite') as database:
                            database.execute('CREATE TABLE settings(key TEXT PRIMARY KEY,value TEXT NOT NULL)')
                            database.execute('INSERT INTO settings VALUES(?,?)', ('theme', json.dumps(selected)))
                    output = args.output / (name + '.json')
                    probe = SimpleNamespace(binary=binary, output=output, page='settings',
                        default_library=False, resume_job=False, advanced=False,
                        copy_report=False, scroll_end=False, seconds=0.05,
                        recover_jobs=False, corrupt_journal=False, pause_job=False)
                    with probe_display(output) as (display, kind):
                        if run_probe(probe, display, kind, data=data):
                            raise RuntimeError('native probe failed')
                    rgb = pixel(output.with_suffix('.png'), 600, 500)
                    dark = max(rgb) < 100
                    if dark != expected_dark or (data / 'theme.json').read_bytes() != legacy_bytes:
                        raise RuntimeError(f'{name}: unexpected theme/legacy mutation: {rgb}')
                    observed = json.loads(output.read_text())
                    if not observed['probe_child_reaped']:
                        raise RuntimeError('owned GUI was not reaped')
                    result['cases'].append({'name': name, 'background_rgb': rgb,
                        'expected_dark': expected_dark, 'verified': True,
                        'legacy_bytes_preserved': True, 'owned_gui_reaped': True})
            finally:
                if bus.poll() is None:
                    os.killpg(bus.pid, signal.SIGTERM)
                    try:
                        bus.wait(timeout=3)
                    except subprocess.TimeoutExpired:
                        os.killpg(bus.pid, signal.SIGKILL)
                        bus.wait(timeout=3)
                result['owned_bus_reaped'] = bus.poll() is not None
    result['temporary_fixture_removed'] = not temporary.exists()
    if hashlib.sha256(binary.read_bytes()).hexdigest() != initial_hash:
        raise RuntimeError('binary changed during observations')
    (args.output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
