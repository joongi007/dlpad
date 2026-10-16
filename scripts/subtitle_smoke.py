"""Opt-in installed yt-dlp subtitle smoke using owned loopback VTT fixtures only."""
import argparse
import functools
import http.server
import json
import os
import pathlib
import signal
import subprocess
import tempfile
import threading

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("yt_dlp", type=pathlib.Path, help="Existing yt-dlp executable")
options = parser.parse_args()
TOOL = str(options.yt_dlp.resolve(strict=True))
if not pathlib.Path(TOOL).is_file():
    parser.error("yt-dlp must be an existing file")


def owned_run(args):
    process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               start_new_session=(os.name == "posix"))
    try:
        stdout, stderr = process.communicate(timeout=20)
    except BaseException:
        if process.poll() is None:
            if os.name == "posix":
                os.killpg(process.pid, signal.SIGKILL)
            else:
                process.kill()
        process.wait()
        raise
    if process.returncode != 0:
        raise RuntimeError(stderr.decode(errors="replace"))
    if len(stdout) + len(stderr) > 64 * 1024:
        raise RuntimeError("Synthetic command output exceeded fixture limit")
    return stdout

with tempfile.TemporaryDirectory(prefix='dlpad-synthetic-subtitles-') as temporary:
    root = pathlib.Path(temporary)
    fixtures = root / 'server'
    fixtures.mkdir()
    manual = b'WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nSynthetic manual Korean subtitle\n'
    automatic = b'WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nSynthetic automatic English subtitle\n'
    (fixtures / 'manual.vtt').write_bytes(manual)
    (fixtures / 'automatic.vtt').write_bytes(automatic)
    class Handler(http.server.SimpleHTTPRequestHandler):
        def log_message(self, *args): pass
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Handler, directory=str(fixtures)))
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        origin = f'http://127.0.0.1:{server.server_port}'
        info = {'id': 'synthetic-subtitle', 'title': 'Synthetic subtitle', 'extractor': 'generic',
                'webpage_url': origin + '/synthetic.mp4', 'url': origin + '/synthetic.mp4',
                'ext': 'mp4', 'subtitles': {'ko': [{'url': origin + '/manual.vtt', 'ext': 'vtt'}]},
                'automatic_captions': {'en': [{'url': origin + '/automatic.vtt', 'ext': 'vtt'}]}}
        metadata = root / 'synthetic.info.json'
        metadata.write_text(json.dumps(info), encoding='utf-8')
        outcomes = []
        for allow_auto in (False, True):
            output = root / ('auto' if allow_auto else 'manual')
            output.mkdir()
            args = [TOOL, '--ignore-config', '--no-plugin-dirs', '--newline', '--no-playlist',
                    '--retries', '3', '--paths', str(output), '--format', 'bv*+ba/b',
                    '--write-subs', '--sub-langs', 'ko,en', '--sub-format', 'vtt/best',
                    '--print', 'after_video:DLPAD_SUBTITLES:{"id":%(id)j,"requested_subtitles":{"ko":%(requested_subtitles.ko.{filepath,ext}|null)j,"en":%(requested_subtitles.en.{filepath,ext}|null)j}}', '--skip-download', '--load-info-json', str(metadata), '--output', 'synthetic.%(ext)s']
            if allow_auto: args.append('--write-auto-subs')
            stdout = owned_run(args)
            markers = [line.removeprefix('DLPAD_SUBTITLES:') for line in stdout.decode().splitlines() if line.startswith('DLPAD_SUBTITLES:')]
            assert len(markers) == 1, stdout
            evidence = json.loads(markers[0])
            assert evidence['id'] == info['id']
            for language, subtitle in evidence['requested_subtitles'].items():
                if subtitle:
                    assert set(subtitle) == {'filepath', 'ext'}, evidence
                    assert pathlib.Path(subtitle['filepath']).is_file()
            assert bool(evidence['requested_subtitles']['en']) == allow_auto
            assert (output / 'synthetic.ko.vtt').read_bytes() == manual
            english = output / 'synthetic.en.vtt'
            assert english.exists() == allow_auto
            if allow_auto: assert english.read_bytes() == automatic
            outcomes.append({'auto_generated': allow_auto, 'files': sorted(p.name for p in output.iterdir())})
        print(json.dumps({'tool': owned_run([TOOL, '--version']).decode().strip(), 'outcomes': outcomes}))
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
