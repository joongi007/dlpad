import hashlib
import json
from pathlib import Path
import stat
import tempfile
import unittest
import zipfile
import generate_install_payload as generator

def sha(data):
    return hashlib.sha256(data).hexdigest()

class PayloadTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / 'candidate.zip'
        self.output = self.root / 'out'
        self.license = self.root / 'LICENSE'
        self.license.write_bytes(b'project license\r\n')
        self.members = {'dlpad-cpu.exe': b'cpu bytes', 'dlpad-install.exe': b'installer bytes',
                        'licenses/pkg/LICENSE': b'original\x00notice\r\n',
                        'build-evidence/rust-toolchain.toml': b'toolchain',
                        'build-evidence/whole-proof-sha256.txt': b'a' * 64 + b'\n',
                        'build-evidence/graph-proof-sha256.txt': b'b' * 64 + b'\n'}
        bins = {n: {'bytes': len(v), 'sha256': sha(v)} for n, v in self.members.items() if n.endswith('.exe')}
        self.manifest = {'schema': 1, 'target': generator.TARGET, 'development_only': True,
                         'development_candidate': True, 'release_ready': False,
                         'legal_coverage_complete': False, 'entrypoint': 'dlpad-cpu.exe',
                         'binaries': bins, 'whole_proof_sha256': 'a' * 64,
                         'graph_proof_sha256': 'b' * 64,
                         'toolchain_file_sha256': sha(b'toolchain'),
                         'native_installer': {'entry': 'dlpad-install.exe', 'development_only': True,
                                              'installer_complete': False, **bins['dlpad-install.exe']}}
        self.inventory = {'complete': False, 'missing': [], 'records': [{'notices': [
            {'entry': 'licenses/pkg/LICENSE', 'sha256': sha(self.members['licenses/pkg/LICENSE'])}]}]}

    def build(self, extras=(), symlink=False):
        self.members['candidate-manifest.json'] = json.dumps(self.manifest).encode()
        self.members['license-inventory.json'] = json.dumps(self.inventory).encode()
        with zipfile.ZipFile(self.source, 'w') as z:
            for name, raw in self.members.items():
                z.writestr(name, raw)
            for name, raw in extras:
                if symlink:
                    info = zipfile.ZipInfo(name)
                    info.create_system = 3
                    info.external_attr = (stat.S_IFLNK | 0o777) << 16
                    z.writestr(info, raw)
                else:
                    z.writestr(name, raw)
        self.original = self.source.read_bytes()

    def run_generator(self, pin=None):
        return generator.generate(self.source, self.output, pin or sha(self.original), self.license, sha(self.license.read_bytes()))

    def rejection(self, pattern):
        with self.assertRaisesRegex((ValueError, zipfile.BadZipFile), pattern):
            self.run_generator()
        self.assertFalse(self.output.exists())
        self.assertEqual(self.source.read_bytes(), self.original)

    def test_success_mapping_manifest_raw_notice_and_source_preserved(self):
        self.build()
        bundle = self.run_generator()
        self.assertEqual(bundle['version'], '0.1.0')
        self.assertEqual(len(bundle['entries']), 5)
        self.assertEqual((self.output / 'payload/dlpad.exe').read_bytes(), b'cpu bytes')
        self.assertEqual((self.output / 'dlpad-install.exe').read_bytes(), b'installer bytes')
        notice = (self.output / 'payload/THIRD-PARTY-NOTICES.txt').read_bytes()
        self.assertIn(self.members['licenses/pkg/LICENSE'], notice)
        self.assertIn(sha(self.members['licenses/pkg/LICENSE']).encode(), notice)
        self.assertEqual(self.source.read_bytes(), self.original)
        binding = json.loads((self.output / 'payload/binding-record.txt').read_bytes())
        self.assertIs(binding['complete_legal_coverage'], False)
        for entry in bundle['entries']:
            raw = (self.output / 'payload' / entry['name']).read_bytes()
            self.assertEqual({'bytes': len(raw), 'sha256': sha(raw)}, {k: entry[k] for k in ['bytes', 'sha256']})

    def test_existing_output_preserved(self):
        self.build()
        self.output.mkdir()
        original = self.output / 'original.txt'
        original.write_bytes(b'keep')
        with self.assertRaisesRegex(ValueError, 'fresh output'):
            self.run_generator()
        self.assertEqual(original.read_bytes(), b'keep')

    def test_traversal_duplicate_case_and_symlink_rejected(self):
        for extras, symlink, pattern in [([('../escape', b'x')], False, 'unsafe'),
                                         ([('DLPAD-CPU.EXE', b'x')], False, 'duplicate'),
                                         ([('link', b'outside')], True, 'non-regular')]:
            with self.subTest(extras=extras):
                self.build(extras, symlink)
                self.rejection(pattern)

    def test_candidate_pin_failure_preserves_inputs(self):
        self.build()
        with self.assertRaisesRegex(ValueError, 'ZIP pin'):
            self.run_generator('0' * 64)
        self.assertFalse(self.output.exists())
        self.assertEqual(self.source.read_bytes(), self.original)

    def test_binary_and_notice_pins_rejected(self):
        self.members['dlpad-cpu.exe'] = b'changed binary'
        self.build()
        self.rejection('executable bytes/SHA')
        self.members['dlpad-cpu.exe'] = b'cpu bytes'
        self.members['licenses/pkg/LICENSE'] = b'changed notice'
        self.build()
        self.rejection('notice pin')

    def test_readiness_true_rejected(self):
        self.manifest['release_ready'] = True
        self.build()
        self.rejection('readiness')

    def test_proof_and_unbound_notice_rejected(self):
        self.members['build-evidence/whole-proof-sha256.txt'] = b'c' * 64
        self.build()
        self.rejection('proof member')
        self.members['build-evidence/whole-proof-sha256.txt'] = b'a' * 64
        self.build([('licenses/unbound/LICENSE', b'x')])
        self.rejection('exact inventory')

    def test_crc_failure_rejected_preserving_corrupt_source(self):
        self.build()
        raw = self.original.replace(b'cpu bytes', b'bad bytes', 1)
        self.source.write_bytes(raw)
        self.original = raw
        self.rejection('CRC')

    def test_symlink_output_parent_rejected(self):
        self.build()
        link = self.root / 'link'
        link.symlink_to(self.root, target_is_directory=True)
        self.output = link / 'out'
        self.rejection('symlink output')

    def test_declared_size_bound_rejected(self):
        self.build()
        with zipfile.ZipFile(self.source) as z:
            info = z.getinfo('dlpad-cpu.exe')
            info.file_size = generator.MAX_EXE + 1
            # Exercise pre-read declaration gate without allocating a huge fixture.
            with self.assertRaisesRegex(ValueError, 'member byte bound'):
                generator.validate_zip(z)

if __name__ == '__main__':
    unittest.main()
