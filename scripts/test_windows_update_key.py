"""Synthetic DPAPI failure-path regressions; no operational private key access."""
import io
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import windows_update_key as key


class KeyBridgeTests(unittest.TestCase):
    def test_non_windows_rejects_without_crypto_or_file_mutation(self):
        with patch.object(key.os, 'name', 'posix'):
            with self.assertRaises(ValueError):
                key.transform(b'synthetic', True)

    @unittest.skipUnless(os.name == 'nt', 'actual current-user Windows DPAPI required')
    def test_roundtrip_and_tamper_rejection(self):
        plaintext = b'synthetic fixture key, never an operational credential'
        encrypted = key.transform(plaintext, True)
        self.assertNotIn(plaintext, encrypted)
        self.assertEqual(key.transform(encrypted, False), plaintext)
        altered = bytearray(encrypted)
        altered[-1] ^= 1
        with self.assertRaises(ValueError):
            key.transform(bytes(altered), False)
        with patch.object(key, 'ENTROPY', b'wrong application binding'):
            with self.assertRaises(ValueError):
                key.transform(encrypted, False)

    @unittest.skipUnless(os.name == 'nt', 'actual current-user Windows DPAPI required')
    def test_create_only_output_preserves_existing_key(self):
        with tempfile.TemporaryDirectory(prefix='dlpad-key-fixture-') as folder:
            path = Path(folder) / 'preserve.dpapi'
            path.write_bytes(b'existing synthetic encrypted key')
            stdin = type('Input', (), {'buffer': io.BytesIO(b'synthetic new key')})()
            with patch.object(sys, 'argv', ['bridge', 'protect', str(path)]), patch.object(sys, 'stdin', stdin):
                with self.assertRaises(FileExistsError):
                    key.main()
            self.assertEqual(path.read_bytes(), b'existing synthetic encrypted key')

    def test_oversized_input_is_rejected(self):
        with self.assertRaises(ValueError):
            key.transform(b'x' * 65537, True)


if __name__ == '__main__':
    unittest.main()
