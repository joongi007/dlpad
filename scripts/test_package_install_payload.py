import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile
import package_install_payload as pack

def sha(raw):
    return hashlib.sha256(raw).hexdigest()

class PackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.source.mkdir()
        self.payload = self.source / "payload"
        self.payload.mkdir()
        self.output = self.root / "installer.zip"
        self.cpu = b"ORIGINAL CPU bytes"
        self.installer = b"ORIGINAL installer bytes"
        (self.source / "dlpad-install.exe").write_bytes(self.installer)
        raw = {"dlpad.exe": self.cpu, "LICENSE": b"license", "THIRD-PARTY-NOTICES.txt": b"notice", "README.txt": b"development instructions"}
        self.binding = dict(schema=1, development_only=True, release_ready=False, complete_legal_coverage=False, signed_distribution=False, candidate_zip_sha256="a"*64, candidate_manifest_sha256="b"*64, mapping={"dlpad-cpu.exe":"payload/dlpad.exe", "dlpad-install.exe":"dlpad-install.exe"}, binaries={"dlpad-cpu.exe":dict(bytes=len(self.cpu),sha256=sha(self.cpu)), "dlpad-install.exe":dict(bytes=len(self.installer),sha256=sha(self.installer))}, project_license=dict(source_name="LICENSE",bytes=len(raw["LICENSE"]),sha256=sha(raw["LICENSE"])), notices={"licenses/pkg/LICENSE":dict(bytes=6,sha256=sha(b"notice"))})
        raw["binding-record.txt"] = pack.base.encoded(self.binding)
        for name,value in raw.items():
            (self.payload/name).write_bytes(value)
        self.manifest = dict(schema=1,version="0.1.0",target=pack.base.TARGET,development_only=True,entries=[dict(name=name,bytes=len(value),sha256=sha(value)) for name,value in sorted(raw.items())])
        self.publish_manifest()
        self.before = {str(p.relative_to(self.source)):p.read_bytes() for p in self.source.rglob("*") if p.is_file()}
    def publish_manifest(self):
        (self.payload/"manifest.json").write_bytes(pack.base.encoded(self.manifest))
    def run_package(self, output=None):
        return pack.package(self.source, output or self.output, sha((self.payload/"manifest.json").read_bytes()), sha((self.payload/"binding-record.txt").read_bytes()), sha(self.installer))
    def assert_sources(self):
        self.assertEqual(self.before,{str(p.relative_to(self.source)):p.read_bytes() for p in self.source.rglob("*") if p.is_file()})
    def test_deterministic_seven_regular_members_crc_receipt_and_preservation(self):
        result=self.run_package()
        other=self.root/"other.zip"
        second=self.run_package(other)
        self.assertEqual(self.output.read_bytes(),other.read_bytes())
        self.assertEqual(result,second)
        self.assertIs(result["release_ready"],False)
        self.assertEqual(len(result["members"]),7)
        self.assertEqual(result["zip_sha256"],sha(self.output.read_bytes()))
        with zipfile.ZipFile(self.output) as archive:
            self.assertIsNone(archive.testzip())
            self.assertEqual(archive.read("payload/dlpad.exe"),self.cpu)
        self.assert_sources()
    def test_inside_source_and_dotdot_output_are_rejected(self):
        for output in [self.source / "new.zip", self.root / "missing" / ".." / "new.zip"]:
            with self.subTest(output=str(output)),self.assertRaises(ValueError):
                self.run_package(output)
        self.assert_sources()
    def test_existing_outputs_and_receipt_preserved(self):
        self.output.write_bytes(b"keep")
        with self.assertRaisesRegex(ValueError,"fresh"):
            self.run_package()
        self.assertEqual(self.output.read_bytes(),b"keep")
        self.output.unlink()
        receipt=Path(str(self.output)+".receipt.json")
        receipt.write_bytes(b"keep receipt")
        with self.assertRaisesRegex(ValueError,"fresh"):
            self.run_package()
        self.assertFalse(self.output.exists())
        self.assertEqual(receipt.read_bytes(),b"keep receipt")
        self.assert_sources()
    def test_bad_entry_sha_or_installer_preflight_preserves_absent_output(self):
        (self.payload/"dlpad.exe").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError,"entry SHA"):
            self.run_package()
        self.assertFalse(self.output.exists())
        (self.payload/"dlpad.exe").write_bytes(self.cpu)
        (self.source/"dlpad-install.exe").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError,"installer SHA"):
            self.run_package()
        self.assertFalse(self.output.exists())
    def test_duplicate_case_traversal_unknown_schema_rejected(self):
        original=json.loads(json.dumps(self.manifest))
        for value in ["dlpad.exe","DLPAD.EXE","../escape"]:
            self.manifest=json.loads(json.dumps(original))
            self.manifest["entries"][0]["name"]=value
            self.publish_manifest()
            with self.subTest(name=value),self.assertRaises(ValueError):
                self.run_package()
            self.assertFalse(self.output.exists())
        self.manifest=original
        self.manifest["release_ready"]=True
        self.publish_manifest()
        with self.assertRaisesRegex(ValueError,"schema"):
            self.run_package()
    def test_unsigned_binding_cannot_claim_readiness(self):
        self.binding["release_ready"]=True
        raw=pack.base.encoded(self.binding)
        (self.payload/"binding-record.txt").write_bytes(raw)
        for entry in self.manifest["entries"]:
            if entry["name"]=="binding-record.txt":
                entry.update(bytes=len(raw),sha256=sha(raw))
        self.publish_manifest()
        with self.assertRaisesRegex(ValueError,"readiness"):
            self.run_package()
        self.assertFalse(self.output.exists())
    def test_symlink_extra_namespace_and_expanded_bound_rejected(self):
        extra=self.source/"extra"
        extra.write_text("extra")
        with self.assertRaisesRegex(ValueError,"namespace"):
            self.run_package()
        extra.unlink()
        original=self.payload/"README.txt"
        original.unlink()
        original.symlink_to(self.root/"outside")
        with self.assertRaisesRegex(ValueError,"symlink"):
            self.run_package()
        original.unlink()
        original.write_bytes(self.before["payload/README.txt"])
        with patch.object(pack.base,"MAX_ZIP",1),self.assertRaisesRegex(ValueError,"expanded"):
            self.run_package()
        self.assertFalse(self.output.exists())
        self.assert_sources()
    def test_archive_crc_failure_has_no_success_receipt_and_preserves_source(self):
        original_namespace=pack.namespace
        calls=[]
        def corrupt_after_copy(root):
            original_namespace(root)
            calls.append(root)
            if len(calls)==2:
                raw=self.output.read_bytes()
                self.output.write_bytes(raw.replace(self.cpu,b"X"*len(self.cpu),1))
        with patch.object(pack,"namespace",side_effect=corrupt_after_copy),self.assertRaises(zipfile.BadZipFile):
            self.run_package()
        self.assertFalse(Path(str(self.output)+".receipt.json").exists())
        self.assert_sources()

if __name__=="__main__":
    unittest.main()
