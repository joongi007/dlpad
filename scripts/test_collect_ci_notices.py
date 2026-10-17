"""Owned synthetic fixtures exercise dynamic notice collection and original byte rejection."""
import io
import json
from pathlib import Path
import shutil
import tarfile
import tempfile
import unittest

import ci_notice_graph as graph
import ci_whole_orchestrate as orchestrate
import ci_whole_receipt as receipt
import collect_ci_notices as collector
from test_windows_companion_package import pe_bytes


class Fixture:
    def __init__(self, base):
        self.base = Path(base)
        self.root = self.base/'checkout'
        self.root.mkdir()
        self.home = self.base/'cargo'
        self.source = self.home/'registry/src/index/dep-1.2.3'
        self.source.mkdir(parents=True)
        (self.source/'Cargo.toml').write_bytes(b'[package]\nname="dep"\nversion="1.2.3"\nlicense="MIT"\n')
        (self.source/'LICENSE').write_bytes(b'original dependency license\r\n')
        self.archive = self.home/'registry/cache/index/dep-1.2.3.crate'
        self.archive.parent.mkdir(parents=True)
        with tarfile.open(self.archive, 'w:gz') as archive:
            for name in ('Cargo.toml', 'LICENSE'):
                data = (self.source/name).read_bytes()
                info = tarfile.TarInfo('dep-1.2.3/'+name)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
        checksum = collector.pin(self.archive.read_bytes())['sha256']
        (self.source/'.cargo-checksum.json').write_bytes(collector.encoded({'package': checksum, 'files': {name: collector.pin((self.source/name).read_bytes())['sha256'] for name in ('Cargo.toml', 'LICENSE')}}))
        (self.root/'Cargo.toml').write_bytes(b'[workspace.package]\nversion="0.1.0"\n')
        (self.root/'LICENSE').write_bytes(b'project license\n')
        (self.root/'rust-toolchain.toml').write_bytes(b'pinned toolchain')
        self.lock = ('version=4\n[[package]]\nname="dep"\nversion="1.2.3"\nsource="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="'+checksum+'"\n').encode()
        (self.root/'Cargo.lock').write_bytes(self.lock)
        self.upstream = self.root/'docs/licenses/upstream'
        self.upstream.mkdir(parents=True)
        (self.upstream/'manifest.json').write_bytes(collector.encoded({'schema': 1, 'coverage_complete': False, 'sources': {}, 'packages': {}}))
        self.meta = {'version': 1, 'packages': [{'name': 'dep', 'version': '1.2.3', 'id': 'registry+https://github.com/rust-lang/crates.io-index#dep@1.2.3', 'source': 'registry+https://github.com/rust-lang/crates.io-index', 'manifest_path': str(self.source/'Cargo.toml'), 'license': 'MIT', 'license_file': None}]}
        self.tree = b'dep v1.2.3\n'
        self.refresh_graph()
        self.identity = {'commit': 'a'*40, 'run_id': '123', 'run_attempt': '1', 'repository': 'joongi007/dlpad'}
        self.tracked = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'LICENSE', 'docs/licenses/upstream/manifest.json']
        self.raw_tracked = b''.join(name.encode()+b'\0' for name in self.tracked)

        def git(root, argv):
            if argv == orchestrate.GIT_ARGS[0]:
                return self.raw_tracked
            if argv == orchestrate.GIT_ARGS[1]:
                return (self.identity['commit']+'\n').encode()
            return b''
        orchestrate.before(self.root, 'ci-captures', self.identity, git)
        binaries = self.root/'target/release'
        binaries.mkdir(parents=True)
        for name in receipt.BINS:
            (binaries/name).write_bytes(pe_bytes(2 if name in ('dlpad-cpu.exe', 'dlpad-install.exe') else 3))
        captures = self.root/'ci-captures'
        for name, argv in receipt.REQUIRED.items():
            for kind in ('stdout', 'stderr'):
                (captures/(name+'.'+kind)).write_bytes((name+kind).encode())
            receipt.publish(captures/(name+'.json'), {'name': name, 'argv': argv, 'exit_code': 0, 'stdout': 'ci-captures/'+name+'.stdout', 'stderr': 'ci-captures/'+name+'.stderr'})
            life = dict(schema=1, kind='windows-ci-operation-lifecycle', name=name, native_run_performed=True, original_worker_job_assigned=True, natural_job_empty=True, job_closed=True, forced_cleanup=False, reader_drained=True, remote_execution_authenticated=False, native_gui_acceptance_verified=False, legal_coverage_complete=False, release_ready=False, worker_exit_code=0, final_active=0, accounting_queries=1, passed=True, streams=[dict(bytes=len(name+kind), drained=True) for kind in ('stdout', 'stderr')])
            receipt.publish(captures/(name+'.lifecycle.json'), life)
        self.proof = orchestrate.after(self.root, 'ci-captures', self.identity, git)
        self.output = self.base/'notices'

    def refresh_graph(self):
        metadata = collector.encoded(self.meta)
        (self.root/'ci-metadata.stdout.json').write_bytes(metadata)
        for name in ('ci-app-tree.stdout.txt', 'ci-engine-tree.stdout.txt'):
            (self.root/name).write_bytes(self.tree)
        value = graph.classify(self.tree, self.tree, metadata, self.lock)
        (self.root/'ci-graph-format.json').write_bytes(collector.encoded(value))

    def collect(self):
        return collector.collect(self.root, self.root/'ci-graph-format.json', self.root/'ci-metadata.stdout.json', self.root/'Cargo.lock', self.home, self.root/'ci-captures/whole.json', self.output)

    def artifact(self):
        result = self.base/'inputs'
        result.mkdir()
        for name in ('ci-captures', 'target', 'ci-graph-format.json', 'ci-app-tree.stdout.txt', 'ci-engine-tree.stdout.txt', 'ci-metadata.stdout.json', 'Cargo.lock', 'rust-toolchain.toml'):
            source = self.root/name
            if source.is_dir():
                shutil.copytree(source, result/name)
            else:
                shutil.copyfile(source, result/name)
        return result


class NoticeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.fixture = Fixture(self.temp.name)

    def reject(self, pattern):
        with self.assertRaisesRegex((ValueError, OSError), pattern):
            self.fixture.collect()
        self.assertFalse(self.fixture.output.exists())

    def test_exact_dynamic_graph_original_notice_and_false_readiness(self):
        result = self.fixture.collect()
        self.assertEqual(len(result['records']), 1)
        self.assertFalse(result['release_ready'])
        self.assertFalse(result['legal_coverage_complete'])
        self.assertEqual((self.fixture.output/'packages/dep-1.2.3/LICENSE').read_bytes(), b'original dependency license\r\n')
        self.assertTrue(result['records'][0]['original_source_binding']['cargo_checksum_map_present_validated'])

    def test_source_notice_tamper_rejected_without_output(self):
        (self.fixture.source/'LICENSE').write_bytes(b'changed')
        self.reject('mismatch')

    def test_original_archive_tamper_rejected(self):
        with self.fixture.archive.open('ab') as stream:
            stream.write(b'changed')
        self.reject('archive lock SHA')

    def test_missing_original_notice_rejected(self):
        (self.fixture.source/'LICENSE').unlink()
        self.reject('notice missing')

    def test_graph_cannot_remove_package(self):
        path = self.fixture.root/'ci-graph-format.json'
        value = json.loads(path.read_bytes())
        value['resolved_packages'] = []
        path.write_bytes(collector.encoded(value))
        self.reject('graph inputs mismatch')

    def test_checkout_license_manifest_tamper_rejected(self):
        (self.fixture.upstream/'manifest.json').write_bytes(b'{}')
        self.reject('tracked source pin')

    def test_license_file_escape_rejected(self):
        self.fixture.meta['packages'][0]['license_file'] = str(self.fixture.root/'LICENSE')
        self.fixture.refresh_graph()
        self.reject('license_file escapes')

    def test_existing_output_preserved(self):
        self.fixture.output.mkdir()
        sentinel = self.fixture.output/'sentinel'
        sentinel.write_bytes(b'keep')
        with self.assertRaisesRegex(ValueError, 'fresh output'):
            self.fixture.collect()
        self.assertEqual(sentinel.read_bytes(), b'keep')


if __name__ == '__main__':
    unittest.main()
