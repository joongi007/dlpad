"""Synthetic CI evidence is format validation only, never actual Windows acceptance."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import collect_ci_notices as notice
import package_ci_preview as package
from test_collect_ci_notices import Fixture


class PackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.fixture = Fixture(self.temp.name)
        self.fixture.collect()
        self.inputs = self.fixture.artifact()
        self.output = Path(self.temp.name)/'output'
        self.args = dict(root=self.fixture.root, inputs=self.inputs, notices=self.fixture.output, commit='a'*40, run_id='123', version='0.1.0', output=self.output)

    def reject(self, pattern):
        with self.assertRaisesRegex((ValueError, OSError), pattern):
            package.package(**self.args)
        self.assertFalse(self.output.exists())

    def test_same_run_packages_preserve_bytes_and_have_fixed_names(self):
        result = package.package(**self.args)
        self.assertEqual(set(result), {'dlpad-0.1.0-windows-x86_64-installer.zip', 'dlpad-0.1.0-windows-x86_64-portable.zip'})
        installer = self.output/'dlpad-0.1.0-windows-x86_64-installer.zip'
        portable = self.output/'dlpad-0.1.0-windows-x86_64-portable.zip'
        original = (self.inputs/'target/release/dlpad-cpu.exe').read_bytes()
        with zipfile.ZipFile(installer) as archive:
            self.assertEqual(len(archive.infolist()), 7)
            self.assertEqual(archive.read('payload/dlpad.exe'), original)
            manifest = json.loads(archive.read('payload/manifest.json'))
            self.assertTrue(manifest['development_only'])
            self.assertEqual(manifest['version'], '0.1.0')
            self.assertEqual(len(manifest['entries']), 5)
            self.assertIn(b'original dependency license\r\n', archive.read('payload/THIRD-PARTY-NOTICES.txt'))
        with zipfile.ZipFile(portable) as archive:
            self.assertEqual(archive.read('dlpad.exe'), original)
            self.assertNotIn('dlpad-install.exe', archive.namelist())
        binding = notice.obj(self.output/'release-binding.json')
        self.assertFalse(binding['release_ready'])
        self.assertFalse(binding['signed_distribution'])
        self.assertFalse(binding['complete_legal_coverage'])
        self.assertEqual(len((self.output/'SHA256SUMS').read_text().splitlines()), 2)
        self.args['output'] = self.output.parent/'second-output'
        second = package.package(**self.args)
        self.assertEqual(result, second)

    def test_other_sha_or_run_cannot_package(self):
        for key, value in [('commit', 'b'*40), ('run_id', '124')]:
            previous = self.args[key]
            self.args[key] = value
            self.reject('same repository/SHA/run')
            self.args[key] = previous

    def test_version_mismatch_prevents_packaging(self):
        for value in ('0.2.0', '0.1.0-preview', '01.0.0'):
            self.args['version'] = value
            self.reject('version')

    def test_binary_tamper_rejected(self):
        path = self.inputs/'target/release/dlpad-cpu.exe'
        path.write_bytes(b'x'*path.stat().st_size)
        self.reject('original file pin')

    def test_raw_log_and_lifecycle_tamper_rejected(self):
        for suffix in ('stdout', 'lifecycle.json'):
            path = self.inputs/'ci-captures'/('test.'+suffix)
            original = path.read_bytes()
            path.write_bytes(original+b' ')
            self.reject('binding mismatch|raw stream bytes')
            path.write_bytes(original)

    def test_forced_cleanup_cannot_be_accepted_even_with_updated_file_pins(self):
        life_path = self.inputs/'ci-captures/test.lifecycle.json'
        life = notice.obj(life_path)
        life['forced_cleanup'] = True
        life_path.write_bytes(notice.encoded(life))
        proof_path = self.inputs/'ci-captures/whole.json'
        proof = notice.obj(proof_path)
        proof['capture_lifecycle_bindings']['test']['lifecycle'].update(notice.pin(life_path.read_bytes()))
        proof_path.write_bytes(notice.encoded(proof))
        with self.assertRaisesRegex(ValueError, 'forced cleanup'):
            package.package(**self.args)
        self.assertFalse(self.output.exists())

    def test_missing_capture_rejected(self):
        (self.inputs/'ci-captures/cpu-release.json').unlink()
        self.reject('No such file')

    def test_source_tamper_rejected(self):
        (self.fixture.root/'LICENSE').write_bytes(b'different project license')
        self.reject('original file pin')

    def test_notice_tamper_rejected(self):
        (self.fixture.output/'packages/dep-1.2.3/LICENSE').write_bytes(b'different dependency license')
        self.reject('notice bytes mismatch')

    def test_notice_package_omission_rejected(self):
        file = self.fixture.output/'inventory.json'
        data = notice.obj(file)
        data['records'] = []
        file.write_bytes(notice.encoded(data))
        self.reject('package coverage mismatch')

    def test_runtime_dependency_blocks_unusable_zip(self):
        with patch.object(package, 'pe', return_value={'import_dlls': ['VCRUNTIME140.dll']}):
            self.reject('unbundled runtime dependency')
        with patch.object(package, 'pe', return_value={'import_dlls': ['KERNEL32.dll', 'api-ms-win-crt-runtime-l1-1-0.dll'], 'dll_runtime_presence_verified': False}):
            self.assertFalse(package.runtime(self.inputs/'target/release/dlpad-cpu.exe')['dll_runtime_presence_verified'])

    def test_existing_output_preserved(self):
        self.output.mkdir()
        sentinel = self.output/'sentinel'
        sentinel.write_bytes(b'keep')
        with self.assertRaisesRegex(ValueError, 'fresh output'):
            package.package(**self.args)
        self.assertEqual(sentinel.read_bytes(), b'keep')

    def test_member_traversal_rejected(self):
        path = self.output.parent/'unsafe.zip'
        with self.assertRaises(ValueError):
            package.write_zip(path, {'../outside': b'data'})
        self.assertFalse((self.output.parent/'outside').exists())
        self.assertFalse(path.exists())
        with self.assertRaisesRegex(ValueError, 'case-colliding'):
            package.write_zip(path, {'A': b'a', 'a': b'b'})
        self.assertFalse(path.exists())


if __name__ == '__main__':
    unittest.main()
