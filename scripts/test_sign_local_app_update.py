"""No operational key access. Rejected inputs must never launch a key reader."""
import hashlib
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import sign_local_app_update as local


class LocalSigningTests(unittest.TestCase):
    def test_only_local_windows_drive_paths_are_translated(self):
        self.assertEqual(local.windows_path(Path('/mnt/c/한글 folder/key.dpapi')),
                         'C:\\한글 folder\\key.dpapi')
        for name in ('/tmp/key.dpapi', '/mnt/server/share/key', '/mnt/1/key'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                local.windows_path(Path(name))

    @unittest.skipUnless(os.name == 'posix', 'local signing is a Linux/WSL developer tool')
    def test_wrong_python_pin_rejects_before_decryption_and_output_creation(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-local-key-test-') as folder:
            root = Path(folder)
            executable = root / 'fixture-python.exe'
            executable.write_bytes(b'synthetic executable')
            output = root / 'public-metadata'
            with patch.object(local.subprocess, 'run', side_effect=AssertionError('must not run')):
                with self.assertRaisesRegex(ValueError, 'Python pin mismatch'):
                    local.sign(root / 'absent-key.dpapi', executable, '0' * 64,
                               output=output, key_id='fixture')
            self.assertFalse(output.exists())

    @unittest.skipUnless(os.name == 'posix', 'local signing is a Linux/WSL developer tool')
    def test_missing_protected_key_does_not_launch_reader(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-local-key-test-') as folder:
            root = Path(folder)
            executable = root / 'fixture-python.exe'
            executable.write_bytes(b'synthetic executable')
            sha = hashlib.sha256(executable.read_bytes()).hexdigest()
            with patch.object(local.subprocess, 'run', side_effect=AssertionError('must not run')):
                with self.assertRaises(FileNotFoundError):
                    local.sign(root / 'absent-key.dpapi', executable, sha,
                               output=root / 'metadata', key_id='fixture')


if __name__ == '__main__':
    unittest.main()
