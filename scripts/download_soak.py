#!/usr/bin/env python3
"""Own one prebuilt ignored engine+DB soak test; no Cargo mutation or GUI."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def atomic(path, value):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2) + '\n')
    temporary.replace(path)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('test_binary', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--seconds', type=int, default=28800)
    args = parser.parse_args()
    binary = args.test_binary.resolve(strict=True)
    output = args.output
    if args.seconds < 28800 or not output.is_absolute() or output.parent != Path('/tmp'):
        parser.error('at least 28800 seconds and new direct /tmp output required')
    runner = output.with_name(output.name + '-runner')
    if output.exists() or runner.exists():
        parser.error('output/runner must not exist; never overwrite prior data')
    runner.mkdir()
    digest = hashlib.sha256()
    with binary.open('rb') as source:
        for chunk in iter(lambda: source.read(65536), b''):
            digest.update(chunk)
    env = dict(os.environ, DLPAD_DOWNLOAD_SOAK_SECONDS=str(args.seconds),
               DLPAD_DOWNLOAD_SOAK_OUTPUT=str(output))
    status = dict(scope='synthetic DownloadRepository+SQLite; no real media/network/UI',
                  state='starting', planned_seconds=args.seconds,
                  binary=str(binary), binary_sha256=digest.hexdigest())
    start = time.monotonic()
    process = None
    with (runner / 'test.log').open('wb') as log:
        try:
            process = subprocess.Popen([str(binary), '--ignored', '--exact',
                'synthetic_eight_hour_download_repository', '--nocapture'],
                env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                start_new_session=True)
            fields = Path(f'/proc/{process.pid}/stat').read_text().rsplit(')', 1)[1].split()
            status.update(state='running', owner_pid=process.pid, owner_start_ticks=int(fields[19]))
            atomic(runner / 'status.json', status)
            while process.poll() is None:
                status['elapsed_seconds'] = time.monotonic() - start
                worker = output / 'long-job/worker-status.json'
                if worker.exists():
                    status['latest_worker'] = json.loads(worker.read_text())
                atomic(runner / 'status.json', status)
                if (runner / 'stop.request').exists():
                    status['state'] = 'stopped'
                    process.terminate()  # owned child only; guardian observes owner EOF
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
                    break
                time.sleep(5)
            code = process.wait()
            status.update(exit_code=code, elapsed_seconds=time.monotonic() - start)
            result = output / 'result.json'
            if status['state'] != 'stopped':
                status['state'] = 'completed' if code == 0 and result.exists() else 'failed'
            if result.exists():
                status['test_result'] = json.loads(result.read_text())
            atomic(runner / 'status.json', status)
            atomic(runner / 'result.json', status)
            print(json.dumps(status, indent=2), flush=True)
            return 0 if status['state'] == 'completed' else 1
        finally:
            if process is not None and process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


if __name__ == '__main__':
    sys.exit(main())
