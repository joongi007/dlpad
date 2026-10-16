import copy
import json
import unittest
from validate_update_candidate import validate_candidate

class CandidateTests(unittest.TestCase):
    def setUp(self):
        self.value = dict(schema=1, version="0.1.1", channel="preview", target="x86_64-pc-windows-msvc", package_url="https://releases.example.test/dlpad.zip", package_bytes=1, package_sha256="a" * 64, release_ready=False)
    def check(self, value):
        return validate_candidate(json.dumps(value).encode(), installed_version="0.1.0", channel="preview", allowed_hosts={"releases.example.test"})
    def test_candidate_has_no_readiness(self):
        self.assertIs(self.check(self.value)["release_ready"], False)
    def test_policy_rejections(self):
        cases = [("schema", True), ("version", "0.1.0"), ("version", "0.0.9"), ("version", "01.1.1"), ("target", "x86_64-pc-windows-gnu"), ("channel", "stable"), ("package_bytes", True), ("package_bytes", 0), ("package_bytes", 268435457), ("package_sha256", "A"*64), ("package_url", "http://releases.example.test/dlpad.zip"), ("package_url", "https://evil.test/dlpad.zip"), ("package_url", "https://user@releases.example.test/dlpad.zip"), ("package_url", "https://releases.example.test:444/dlpad.zip"), ("package_url", "https://releases.example.test/dlpad.zip?token=secret"), ("release_ready", True)]
        for key, replacement in cases:
            value = copy.deepcopy(self.value)
            value[key] = replacement
            with self.subTest(key=key, replacement=replacement), self.assertRaises(ValueError):
                self.check(value)
    def test_parse_bounds_duplicates_unknown(self):
        kwargs = dict(installed_version="0.1.0", channel="preview", allowed_hosts={"releases.example.test"})
        for raw in [b"x"*65537, b"{", b'{"schema":1,"schema":1}', b"[]"]:
            with self.subTest(raw=raw[:20]), self.assertRaises(ValueError):
                validate_candidate(raw, **kwargs)
        self.value["signature"] = "untrusted"
        with self.assertRaises(ValueError):
            self.check(self.value)

if __name__ == "__main__":
    unittest.main()
