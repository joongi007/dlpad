"""Independent binary parsing checks; no target executable or GUI is launched."""
from pathlib import Path
import struct
import unittest
import xml.etree.ElementTree as ET

BASE = Path(__file__).resolve().parents[1] / 'crates/app/assets'
CANDIDATE = BASE


def read_res(raw):
    entries = {}
    offset = 0
    while offset < len(raw):
        size, header = struct.unpack_from('<II', raw, offset)
        kind = struct.unpack_from('<H', raw, offset + 10)[0]
        name = struct.unpack_from('<H', raw, offset + 14)[0]
        language = struct.unpack_from('<H', raw, offset + 22)[0]
        if size:
            entries[(kind, name, language)] = raw[offset + header:offset + header + size]
        offset = (offset + header + size + 3) & ~3
    return entries


def read_coff(raw):
    machine, section_count, _, symbol_pointer, symbol_count, _, _ = struct.unpack_from('<HHIIIHH', raw)
    assert machine == 0x8664 and section_count == 2 and symbol_count == 4
    sections = {}
    for index in range(section_count):
        row = struct.unpack_from('<8sIIIIIIHHI', raw, 20 + index * 40)
        sections[row[0]] = row
    dirs = sections[b'.rsrc$01']
    data = sections[b'.rsrc$02']
    relocations = {struct.unpack_from('<IIH', raw, dirs[5] + index * 10) for index in range(dirs[7])}
    symbol = struct.unpack_from('<8sIhHBB', raw, symbol_pointer + 2 * 18)
    assert symbol[0] == b'.rsrc$02' and symbol[1] == 0 and symbol[2] == 2
    entries = {}
    relocation_offsets = set()
    def walk(offset, keys):
        base = dirs[4] + offset
        named, ordinal = struct.unpack_from('<HH', raw, base + 12)
        assert named == 0
        for index in range(ordinal):
            key, target = struct.unpack_from('<II', raw, base + 16 + index * 8)
            assert key < 0x80000000
            if target & 0x80000000:
                walk(target & 0x7fffffff, keys + (key,))
            else:
                assert len(keys) == 2
                rva, size, codepage, reserved = struct.unpack_from('<IIII', raw, dirs[4] + target)
                assert codepage == 0 and reserved == 0 and (target, 2, 3) in relocations
                assert rva + size <= data[3]
                relocation_offsets.add(target)
                entries[keys + (key,)] = raw[data[4] + rva:data[4] + rva + size]
    walk(0, ())
    assert len(relocation_offsets) == dirs[7] == 6
    return entries


class Resources(unittest.TestCase):
    def test_exact_original_icons_and_group_survive_both_formats(self):
        original = read_res((BASE / 'dlpad.res').read_bytes())
        ico = (BASE / 'dlpad.ico').read_bytes()
        self.assertEqual(struct.unpack_from('<HHH', ico), (0, 1, 4))
        for index in range(4):
            size, offset = struct.unpack_from('<II', ico, 6 + 16 * index + 8)
            self.assertEqual(original[(3, index + 1, 0x409)], ico[offset:offset + size])
        for entries in (read_res((CANDIDATE / 'dlpad-installer.res').read_bytes()), read_coff((CANDIDATE / 'dlpad-installer.o').read_bytes())):
            self.assertEqual(set(entries), set(original) | {(24, 1, 0x409)})
            for key, bytes_ in original.items():
                self.assertEqual(entries[key], bytes_)
            self.assertEqual(entries[(24, 1, 0x409)], (BASE / 'dlpad-installer.manifest').read_bytes())
    def test_manifest_contains_only_visual_style_dependency(self):
        raw = (BASE / 'dlpad-installer.manifest').read_bytes()
        root = ET.fromstring(raw)
        ns = {'m': 'urn:schemas-microsoft-com:asm.v1'}
        dependencies = root.findall('m:dependency/m:dependentAssembly/m:assemblyIdentity', ns)
        self.assertEqual(len(dependencies), 1)
        self.assertEqual(dependencies[0].attrib['name'], 'Microsoft.Windows.Common-Controls')
        self.assertEqual(dependencies[0].attrib['version'], '6.0.0.0')
        for forbidden in [b'dpiAware', b'dpiAwareness', b'requestedExecutionLevel', b'autoElevate', b'uiAccess']:
            self.assertNotIn(forbidden, raw)


if __name__ == '__main__':
    unittest.main()
