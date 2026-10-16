#!/usr/bin/env python3
"""Ten-second idle sample of a frozen native candidate on an owned private display/bus."""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import select
import signal
import subprocess
import tempfile
import threading
import time
from native_probe import probe_display, sample, screenshot


def digest(path):
    value = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(65536), b''):
            value.update(chunk)
    return value.hexdigest()


def drain(stream, path):
    with path.open('xb') as log:
        remaining = 65536
        for chunk in iter(lambda: stream.read(4096), b''):
            log.write(chunk[:remaining])
            remaining = max(0, remaining - len(chunk))
    stream.close()


def stop(child):
    if child.poll() is None:
        os.killpg(child.pid, signal.SIGTERM)
        try:
            child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            os.killpg(child.pid, signal.SIGKILL)
            child.wait(timeout=3)
    return child.returncode


def memory_breakdown(pid):
    totals = {}
    label = "unknown"
    consumed = 0
    with Path(f"/proc/{pid}/smaps").open() as stream:
        for line in stream:
            consumed += len(line)
            if consumed > 8 * 1024 * 1024:
                raise RuntimeError("owned smaps exceeds observation budget")
            if re.match(r"^[0-9a-f]+-[0-9a-f]+ ", line):
                fields = line.split(maxsplit=5)
                label = fields[5].strip() if len(fields) == 6 else "anonymous"
            elif line.startswith(("Rss:", "Pss:", "Private_Clean:", "Private_Dirty:")):
                key, value, *_ = line.split()
                record = totals.setdefault(label, {"Rss": 0, "Pss": 0, "Private_Clean": 0, "Private_Dirty": 0})
                record[key.rstrip(":")] += int(value)
    return sorted(({"mapping": key, **value} for key, value in totals.items()),
                  key=lambda value: value["Rss"], reverse=True)[:20]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--font', type=Path, required=True)
    parser.add_argument('--software', action='store_true', help='owned child Mesa software comparison')
    parser.add_argument('--software-driver', choices=['llvmpipe', 'softpipe'], default='llvmpipe')
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    result = {'binary_sha256': digest(binary), 'binary_bytes': binary.stat().st_size,
              'display': 'owned Xvfb', 'bus': 'private D-Bus', 'desktop_fallback_used': False,
              'actual_media_used': False, 'list_10000_verified': False,
              'ime_verified': False, 'screen_reader_verified': False}
    with tempfile.TemporaryDirectory(prefix='dlpad-accessible-perf-') as directory:
        temporary = Path(directory)
        env = dict(os.environ)
        for key in ['WAYLAND_DISPLAY', 'DBUS_SESSION_BUS_ADDRESS', 'AT_SPI_BUS_ADDRESS']:
            env.pop(key, None)
        for key, suffix in [('XDG_CONFIG_HOME', 'config'), ('XDG_CACHE_HOME', 'cache'),
                            ('XDG_DATA_HOME', 'xdg-data'), ('XDG_RUNTIME_DIR', 'runtime'),
                            ('DLPAD_TEST_DATA', 'app-data'), ('DLPAD_LIBRARY_ROOT', 'media'),
                            ('MESA_SHADER_CACHE_DIR', 'shader-cache')]:
            location = temporary / suffix
            location.mkdir(mode=0o700)
            env[key] = str(location)
        env['DLPAD_TEST_FONT'] = str(args.font.resolve(strict=True))
        env['GSETTINGS_BACKEND'] = 'memory'
        if args.software:
            env['LIBGL_ALWAYS_SOFTWARE'] = 'true'
            env['GALLIUM_DRIVER'] = args.software_driver
        result['software_comparison'] = args.software
        result['requested_software_driver'] = args.software_driver if args.software else None
        bus = subprocess.Popen(['dbus-daemon', '--session', '--nofork', '--print-address=1'],
                               env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               start_new_session=True)
        bus_log = threading.Thread(target=drain, args=(bus.stderr, args.output/'bus.log'))
        bus_log.start()
        try:
            if not select.select([bus.stdout], [], [], 5)[0]:
                raise RuntimeError('private bus startup timeout')
            address = bus.stdout.readline(4096).decode().strip()
            if not address.startswith('unix:'):
                raise RuntimeError('private bus address invalid')
            env['DBUS_SESSION_BUS_ADDRESS'] = address
            with probe_display(args.output/'idle.json') as (display, _):
                env['DISPLAY'] = display
                child = subprocess.Popen([str(binary)], env=env, stdout=subprocess.PIPE,
                                         stderr=subprocess.STDOUT, start_new_session=True)
                log = threading.Thread(target=drain, args=(child.stdout, args.output/'app.log'))
                log.start()
                try:
                    time.sleep(2)
                    ticks, rss = sample(child.pid)
                    stat = Path(f'/proc/{child.pid}/stat').read_text().rsplit(')',1)[1].split()
                    result['process_identity'] = [child.pid, int(stat[19])]
                    started = time.monotonic()
                    samples = [rss]
                    for _ in range(10):
                        time.sleep(1)
                        if child.poll() is not None:
                            raise RuntimeError('owned native candidate exited')
                        current, rss = sample(child.pid)
                        samples.append(rss)
                    elapsed = time.monotonic()-started
                    result.update(sample_seconds=elapsed, rss_samples_kib=samples,
                                  rss_max_kib=max(samples),
                                  idle_cpu_percent=(current-ticks)/os.sysconf('SC_CLK_TCK')/elapsed*100,
                                  proposed_80mib_budget_pass=max(samples)<=80*1024)
                    result['memory_mappings_top20'] = memory_breakdown(child.pid)
                    result['geometry'] = screenshot(args.output/'idle.png', 'dlpad', display)
                    if digest(binary) != result['binary_sha256']:
                        raise RuntimeError('binary changed during measurement')
                except Exception as error:
                    result['error'] = str(error)
                finally:
                    result['owned_gui_exit_code'] = stop(child)
                    log.join(timeout=3)
                    result['owned_gui_reaped'] = child.poll() is not None and not log.is_alive()
        finally:
            stop(bus)
            bus.stdout.close()
            bus_log.join(timeout=3)
            result['owned_bus_reaped'] = bus.poll() is not None and not bus_log.is_alive()
    result['temporary_fixture_removed'] = not temporary.exists()
    (args.output/'result.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))
    return int('error' in result)


if __name__ == '__main__':
    raise SystemExit(main())
