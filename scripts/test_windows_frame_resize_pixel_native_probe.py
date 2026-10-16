"""Reject misleading DB preservation evidence and unsafe input paths."""
import tempfile
import unittest
from pathlib import Path
from windows_frame_resize_pixel_native_probe import plain, read, validate_rows

class ResizeEvidenceFailureTests(unittest.TestCase):
    @staticmethod
    def rows():
        return [('D:\\fixture\\media', f'D:\\fixture\\media\\media-{i:05d}.mp4', 3, 1,
                 'windows:v1:42:'+'a'*32) for i in range(10000)]

    def test_full_inventory_accepts_native_extended_paths(self):
        rows=self.rows()
        rows[0]=('\\\\?\\'+rows[0][0], '\\\\?\\'+rows[0][1], *rows[0][2:])
        validate_rows(rows, 'D:\\fixture\\media')

    def test_foreign_root_and_duplicate_rows_are_rejected(self):
        for replacement in [('D:\\other', *self.rows()[0][1:]), self.rows()[1]]:
            rows=self.rows();rows[0]=replacement
            with self.assertRaises(ValueError):validate_rows(rows, 'D:\\fixture\\media')

    def test_incorrect_inventory_identity_and_presence_are_rejected(self):
        original=self.rows()[0]
        for replacement in [(original[0], original[1].replace('00000','99999'), *original[2:]),
                            (*original[:2], 3, 0, original[4]),
                            (*original[:4], 'unknown:v1')]:
            rows=self.rows();rows[0]=replacement
            with self.assertRaises(ValueError):validate_rows(rows, 'D:\\fixture\\media')

    def test_traversal_symlink_and_oversized_inputs_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);target=root/'target';target.write_bytes(b'1234')
            link=root/'link';link.symlink_to(target)
            with self.assertRaises(ValueError):plain(link)
            with self.assertRaises(ValueError):plain(root/'..'/'other')
            with self.assertRaises(ValueError):read(target, 3)

if __name__=='__main__':unittest.main()
