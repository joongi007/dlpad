"""Real OpenSSL signing tests use only throwaway fixture keys and synthetic ZIPs."""
import json
import os
from pathlib import Path
import shutil
import select
import subprocess
import tempfile
import threading
import time
import unittest
import zipfile
from unittest.mock import patch

import package_ci_preview
import sign_app_update as signer
from test_collect_ci_notices import Fixture


class SignTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='dlpad-sign-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        if os.name == 'posix':
            self.root.chmod(0o700)
        self.fixture = Fixture(self.root)
        self.fixture.collect()
        inputs = self.fixture.artifact()
        packages = self.root/'packages'
        package_ci_preview.package(self.fixture.root, inputs, self.fixture.output, 'a'*40, '123', '0.1.0', packages)
        self.package = packages/'dlpad-0.1.0-windows-x86_64-installer.zip'
        now = int(time.time())
        self.key = self.root/'fixture-ed25519.pem'
        self.args = dict(package=self.package, version='0.1.0', sequence=7, channel='preview', key_id='fixture-test-key', private_key=self.key, output=self.root/'signed', issued_at=now, expires_at=now+86400)

    def fixture_key(self, algorithm='ED25519'):
        if not shutil.which('openssl'):
            self.skipTest('OpenSSL executable unavailable; no operating keys generated')
        subprocess.run(['openssl', 'genpkey', '-algorithm', algorithm, '-out', str(self.key)], stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10, check=True, env=dict(os.environ, OPENSSL_CONF=os.devnull))
        if os.name == 'posix':
            self.key.chmod(0o600)

    def run_sign(self):
        return signer.sign(**self.args)

    def test_actual_openssl_exact_signature_and_modified_payload_rejection(self):
        self.fixture_key()
        original = self.key.read_bytes()
        public = self.run_sign()
        output = self.args['output']
        self.assertEqual({p.name for p in output.iterdir()}, {'update-windows-x86_64.json', 'update-windows-x86_64.json.sig', 'update-public-key.json'})
        payload = (output/'update-windows-x86_64.json').read_bytes()
        signature = (output/'update-windows-x86_64.json.sig').read_text()
        self.assertRegex(signature, r'\A[0-9a-f]{128}\Z')
        self.assertEqual(len(bytes.fromhex(public['public_key_hex'])), 32)
        value = json.loads(payload)
        self.assertEqual(set(value), {'schema', 'domain', 'key_id', 'sequence', 'version', 'channel', 'target', 'repository', 'asset_name', 'asset_url', 'asset_bytes', 'asset_sha256', 'issued_at_unix', 'expires_at_unix'})
        self.assertEqual(value['asset_sha256'], signer.hash_file(self.package, signer.MAX_PACKAGE)[0]['sha256'])
        self.assertEqual(value['asset_url'], 'https://github.com/joongi007/dlpad/releases/download/v0.1.0/dlpad-0.1.0-windows-x86_64-installer.zip')
        public_file, signature_file = self.root/'public.der', self.root/'signature.bin'
        public_file.write_bytes(signer.PUBLIC_PREFIX+bytes.fromhex(public['public_key_hex']))
        signature_file.write_bytes(bytes.fromhex(signature))
        argv = ['pkeyutl', '-verify', '-pubin', '-keyform', 'DER', '-inkey', str(public_file), '-rawin', '-in', str(output/'update-windows-x86_64.json'), '-sigfile', str(signature_file)]
        signer.run_openssl(argv)
        (output/'update-windows-x86_64.json').write_bytes(payload.replace(b'"sequence":7', b'"sequence":8'))
        with self.assertRaisesRegex(ValueError, 'OpenSSL operation failed'):
            signer.run_openssl(argv)
        self.assertEqual(self.key.read_bytes(), original)
        self.assertNotIn(original, payload)
        self.assertNotIn(original, (output/'update-public-key.json').read_bytes())

    def test_existing_output_never_overwritten(self):
        self.args['output'].mkdir()
        sentinel = self.args['output']/'sentinel'
        sentinel.write_bytes(b'existing user output')
        with self.assertRaisesRegex(ValueError, 'fresh output'):
            self.run_sign()
        self.assertEqual(sentinel.read_bytes(), b'existing user output')

    @unittest.skipUnless(os.name == 'posix', 'POSIX key mode policy only')
    def test_private_key_group_and_world_read_rejected(self):
        self.fixture_key()
        for mode in (0o640, 0o604, 0o644, 0o660):
            self.key.chmod(mode)
            with self.subTest(mode=mode), self.assertRaisesRegex(ValueError, 'group/world permissions'):
                self.run_sign()
        self.assertFalse(self.args['output'].exists())

    def test_private_key_oversize_rejected(self):
        self.key.write_bytes(b'x'*(signer.MAX_KEY+1))
        if os.name == 'posix':
            self.key.chmod(0o600)
        with self.assertRaisesRegex(ValueError, 'private key size'):
            self.run_sign()
        self.assertFalse(self.args['output'].exists())

    def test_private_key_symlink_rejected(self):
        self.fixture_key()
        link = self.root/'alias.pem'
        try:
            link.symlink_to(self.key)
        except OSError as error:
            if os.name == 'nt' and getattr(error, 'winerror', None) == 1314:
                self.skipTest('Windows symlink privilege unavailable; no system changes')
            raise
        self.args['private_key'] = link
        with self.assertRaisesRegex(ValueError, 'symlink/reparse'):
            self.run_sign()

    def test_oversize_package_rejected_before_hash_or_openssl(self):
        with self.package.open('r+b') as stream:
            stream.truncate(signer.MAX_PACKAGE+1)
        with patch.object(signer, 'run_openssl', side_effect=AssertionError('must not execute')):
            with self.assertRaisesRegex(ValueError, 'bounded input'):
                signer.metadata(self.package, '0.1.0', 1, 'preview', 'key', self.args['issued_at'], self.args['expires_at'])

    def test_invalid_policy_inputs_do_not_produce_metadata(self):
        parameters = {name: self.args[name] for name in ('package', 'version', 'sequence', 'channel', 'key_id', 'issued_at', 'expires_at')}
        for key, value in [('version', '01.0.0'), ('version', '0.1.0-preview'), ('sequence', 0), ('sequence', True), ('sequence', 1<<64), ('channel', 'stable'), ('key_id', '../key'), ('expires_at', self.args['issued_at']+signer.MAX_VALIDITY+1), ('issued_at', self.args['expires_at']), ('issued_at', self.args['issued_at']+100)]:
            args = dict(parameters, **{key: value})
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                signer.metadata(**args)
        self.assertFalse(self.args['output'].exists())

    def test_noncanonical_package_name_rejected(self):
        self.package = self.package.rename(self.package.parent/'different.zip')
        with self.assertRaisesRegex(ValueError, 'canonical installer filename'):
            signer.metadata(self.package, '0.1.0', 1, 'preview', 'key', self.args['issued_at'], self.args['expires_at'])

    def test_wrong_key_algorithm_rejected(self):
        self.fixture_key('X25519')
        with self.assertRaisesRegex(ValueError, 'Ed25519 private key'):
            self.run_sign()
        self.assertFalse(self.args['output'].exists())

    def test_tampered_zip_member_and_false_preview_policy_rejected(self):
        with zipfile.ZipFile(self.package) as archive:
            original = {name: archive.read(name) for name in archive.namelist()}
        def rewrite(values):
            with zipfile.ZipFile(self.package, 'w') as archive:
                for name, data in values.items():
                    archive.writestr(name, data)
        for name, data in [('payload/dlpad.exe', b'changed executable'), ('payload/binding-record.txt', json.dumps({'schema': 1, 'version': '0.1.0', 'development_only': True, 'release_ready': True}).encode())]:
            rewrite(dict(original, **{name: data}))
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'member|preview binding'):
                signer.metadata(self.package, '0.1.0', 1, 'preview', 'key', self.args['issued_at'], self.args['expires_at'])
        self.assertFalse(self.args['output'].exists())

    def test_pem_change_during_signing_rejected_and_preserved(self):
        self.fixture_key()
        original = signer.run_openssl
        changed = False
        def mutate(args, **kwargs):
            nonlocal changed
            value = original(args, **kwargs)
            if not changed:
                with self.key.open('ab') as stream:
                    stream.write(b'\n')
                changed = True
            return value
        with patch.object(signer, 'run_openssl', side_effect=mutate), self.assertRaisesRegex(ValueError, 'private key changed'):
            self.run_sign()
        self.assertFalse(self.args['output'].exists())
        self.assertTrue(self.key.read_bytes().endswith(b'\n\n'))

    def test_actual_openssl_timeout_and_output_bounds(self):
        if not shutil.which('openssl'):
            self.skipTest('OpenSSL unavailable')
        with self.assertRaisesRegex(ValueError, 'timed out'):
            signer.run_openssl(['version'], timeout=0)
        with self.assertRaisesRegex(ValueError, 'output'):
            signer.run_openssl(['version'], output_limit=1)


@unittest.skipUnless(os.name == 'posix' and hasattr(os, 'pidfd_open'), 'Linux original-pidfd process ownership regression')
class OwnedProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='dlpad-openssl-tree-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.root.chmod(0o700)
        program = self.root/'openssl'
        program.write_text('''#!/usr/bin/python3
import os, pathlib, signal, sys, time
pidfile, gate, mode = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
child = os.fork()
if child == 0:
    while True:
        signal.pause()
pidfile.write_text(str(child)+" "+str(os.getpid()))
while not gate.exists():
    time.sleep(0.005)
if mode == 'output':
    os.write(1, b'x'*65536)
if mode == 'exit':
    os.write(1, b'ok')
    os._exit(0)
while True:
    time.sleep(60)
''')
        program.chmod(0o700)

    def exercise(self, mode):
        pidfile, gate = self.root/'owned-pids', self.root/'release-gate'
        result = {}
        def run():
            try:
                result['output'] = signer.run_openssl([str(pidfile), str(gate), mode], timeout=2, output_limit=64)
            except BaseException as error:
                result['error'] = error
        worker = threading.Thread(target=run)
        with patch.dict(os.environ, {'PATH': str(self.root)+os.pathsep+os.environ.get('PATH', '')}):
            worker.start()
            try:
                deadline = time.monotonic()+1
                while not pidfile.exists() and time.monotonic() < deadline:
                    time.sleep(0.005)
                self.assertTrue(pidfile.exists(), 'owned fixture did not become ready')
                child, leader = map(int, pidfile.read_text().split())
                self.assertEqual(os.getpgid(child), leader)
                child_handle = os.pidfd_open(child)
                try:
                    gate.write_bytes(b'release')
                    worker.join(timeout=5)
                    self.assertFalse(worker.is_alive(), 'owned OpenSSL call did not finish')
                    # Kernel readiness refers to this exact original process, even
                    # if its numeric PID is subsequently reused or it is a zombie.
                    self.assertEqual(select.select([child_handle], [], [], 1)[0], [child_handle], 'original descendant is still running')
                finally:
                    os.close(child_handle)
            finally:
                worker.join(timeout=5)
        if mode == 'exit':
            self.assertEqual(result, {'output': b'ok'})
        else:
            self.assertIsInstance(result.get('error'), ValueError)
            self.assertIn('timed out' if mode == 'timeout' else 'output', str(result['error']))

    def test_timeout_terminates_original_fork_descendant(self):
        self.exercise('timeout')

    def test_stdout_overflow_terminates_original_fork_descendant(self):
        self.exercise('output')

    def test_successful_leader_cannot_leave_pipe_inheriting_child_running(self):
        self.exercise('exit')

    def test_group_cleanup_precedes_original_leader_reap(self):
        original_killpg = signer.os.killpg
        events = []
        def checked_killpg(pid, sig):
            # WNOWAIT succeeds only while this exact leader remains our child;
            # this check would fail after an unsafe poll/wait reap.
            os.waitid(os.P_PID, pid, os.WEXITED | os.WNOHANG | os.WNOWAIT)
            self.assertEqual(os.getpgid(pid), pid)
            events.append('group-cleanup')
            return original_killpg(pid, sig)
        original_wait = signer.subprocess.Popen.wait
        def checked_wait(process, *args, **kwargs):
            self.assertEqual(events, ['group-cleanup'])
            value = original_wait(process, *args, **kwargs)
            events.append('leader-reaped')
            return value
        with patch.object(signer.os, 'killpg', side_effect=checked_killpg), patch.object(signer.subprocess.Popen, 'wait', checked_wait):
            self.assertTrue(signer.run_openssl(['version']))
        self.assertEqual(events, ['group-cleanup', 'leader-reaped'])

    def test_windows_is_rejected_before_reading_key_or_starting_process(self):
        with patch.object(signer.sys, 'platform', 'win32'), patch.object(signer, 'plain', side_effect=AssertionError('must not read')), patch.object(signer.subprocess, 'Popen', side_effect=AssertionError('must not spawn')):
            with self.assertRaisesRegex(ValueError, 'Unsupported platform'):
                signer.sign(Path('/missing'), '0.1.0', 1, 'preview', 'key', Path('/missing-key'), Path('/missing-output'), 1, 2)
            with self.assertRaisesRegex(ValueError, 'Unsupported platform'):
                signer.run_openssl(['version'])
        with patch.object(signer.signal, 'getsignal', return_value=signer.signal.SIG_IGN), patch.object(signer.subprocess, 'Popen', side_effect=AssertionError('must not spawn')):
            with self.assertRaisesRegex(ValueError, 'Unsupported platform'):
                signer.run_openssl(['version'])


if __name__ == '__main__':
    unittest.main()
