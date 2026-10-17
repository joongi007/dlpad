import unittest

from release_version import validate


class ReleaseVersionTests(unittest.TestCase):
    def test_exact_tag_allows_unpublished_draft(self):
        self.assertEqual(validate('refs/tags/v0.1.0', '0.1.0', False), ('0.1.0', True))

    def test_main_candidate_never_allows_draft(self):
        self.assertEqual(validate('refs/heads/main', '0.1.0', True), ('0.1.0', False))

    def test_candidate_tag_and_other_refs_rejected(self):
        for ref in ['refs/tags/v0.1.0', 'refs/heads/feature', 'refs/pull/1/merge']:
            with self.subTest(ref=ref), self.assertRaises(ValueError):
                validate(ref, '0.1.0', True)

    def test_wrong_version_and_unrequested_main_rejected(self):
        for ref in ['refs/tags/v0.1.1', 'refs/heads/main', 'refs/tags/0.1.0']:
            with self.subTest(ref=ref), self.assertRaises(ValueError):
                validate(ref, '0.1.0', False)

    def test_noncanonical_versions_rejected(self):
        for version in ['01.1.0', '0.1.0-preview', '0.1', '0.1.0\n', '1000000000.1.0']:
            with self.subTest(version=version), self.assertRaises(ValueError):
                validate('refs/tags/v'+version, version, False)


if __name__ == '__main__':
    unittest.main()
