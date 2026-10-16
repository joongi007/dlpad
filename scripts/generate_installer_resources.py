"""Create installer-only RES/AMD64 COFF from original icon resource records, offline."""
import argparse
from pathlib import Path
import struct
import xml.etree.ElementTree as ET


def align(value, unit=4):
    return (value + unit - 1) & -unit


def resources(raw):
    result = []
    offset = 0
    while offset < len(raw):
        size, header = struct.unpack_from('<II', raw, offset)
        if header != 32 or offset + header + size > len(raw):
            raise ValueError('only bounded ordinal original resource records supported')
        type_tag, kind, name_tag, name = struct.unpack_from('<HHHH', raw, offset + 8)
        if type_tag != 0xffff or name_tag != 0xffff:
            raise ValueError('ordinal resource IDs required')
        language = struct.unpack_from('<H', raw, offset + 22)[0]
        if size:
            result.append((kind, name, language, raw[offset + header:offset + header + size]))
        offset = align(offset + header + size)
    if offset != len(raw):
        raise ValueError('unaligned resource file')
    return result


def manifest_record(data):
    header = struct.pack('<IIHHHHIHHII', len(data), 32, 0xffff, 24, 0xffff, 1, 0, 0x30, 0x409, 0, 0)
    return header + data + bytes(align(len(data)) - len(data))


def coff(records):
    tree = {}
    for kind, name, language, data in records:
        languages = tree.setdefault(kind, {}).setdefault(name, {})
        if language in languages:
            raise ValueError('duplicate resource identity')
        languages[language] = data
    directories = bytearray()
    payload = bytearray()
    relocations = []

    def directory(branch, depth):
        offset = len(directories)
        keys = sorted(branch)
        directories.extend(struct.pack('<IIHHHH', 0, 0, 0, 0, 0, len(keys)))
        entries = len(directories)
        directories.extend(bytes(8 * len(keys)))
        for index, key in enumerate(keys):
            if depth < 2:
                target = directory(branch[key], depth + 1) | 0x80000000
            else:
                target = len(directories)
                data = branch[key]
                data_offset = len(payload)
                payload.extend(data)
                payload.extend(bytes(align(len(payload), 8) - len(payload)))
                directories.extend(struct.pack('<IIII', data_offset, len(data), 0, 0))
                relocations.append(target)
            struct.pack_into('<II', directories, entries + index * 8, key, target)
        return offset

    directory(tree, 0)
    directory_pointer = 100
    relocation_pointer = directory_pointer + len(directories)
    data_pointer = align(relocation_pointer + len(relocations) * 10, 8)
    symbol_pointer = align(data_pointer + len(payload))
    # Two STATIC section symbols, each with its section-definition auxiliary record.
    header = struct.pack('<HHIIIHH', 0x8664, 2, 0, symbol_pointer, 4, 0, 0x100)
    section = lambda name, size, ptr, rp, count: struct.pack('<8sIIIIIIHHI', name, 0, 0, size, ptr, rp, 0, count, 0, 0x40000040)
    output = bytearray(header + section(b'.rsrc$01', len(directories), directory_pointer, relocation_pointer, len(relocations)) + section(b'.rsrc$02', len(payload), data_pointer, 0, 0))
    output.extend(directories)
    for offset in relocations:
        output.extend(struct.pack('<IIH', offset, 2, 3))  # IMAGE_REL_AMD64_ADDR32NB → .rsrc$02
    output.extend(bytes(data_pointer - len(output)))
    output.extend(payload)
    output.extend(bytes(symbol_pointer - len(output)))
    for name, number, size, count in ((b'.rsrc$01', 1, len(directories), len(relocations)), (b'.rsrc$02', 2, len(payload), 0)):
        output.extend(struct.pack('<8sIhHBB', name, 0, number, 0, 3, 1))
        output.extend(struct.pack('<IHHIHBBH', size, count, 0, 0, 0, 0, 0, 0))
    output.extend(struct.pack('<I', 4))  # Empty COFF string table.
    return bytes(output)


def generate(original, manifest, destination):
    raw = original.read_bytes()
    records = resources(raw)
    if [kind for kind, _, _, _ in records] != [3, 3, 3, 3, 14]:
        raise ValueError('exact original four-icon/group resource contract')
    xml = manifest.read_bytes()
    root = ET.fromstring(xml)
    ns = '{urn:schemas-microsoft-com:asm.v1}'
    identity = root.find(f'{ns}dependency/{ns}dependentAssembly/{ns}assemblyIdentity')
    if identity is None or identity.attrib != {'type': 'win32', 'name': 'Microsoft.Windows.Common-Controls', 'version': '6.0.0.0', 'processorArchitecture': '*', 'publicKeyToken': '6595b64144ccf1df', 'language': '*'}:
        raise ValueError('exact native visual styles dependency')
    if any(token in xml for token in (b'dpiAware', b'dpiAwareness', b'requestedExecutionLevel', b'uiAccess', b'autoElevate')):
        raise ValueError('DPI/elevation declarations outside narrow scope')
    combined = raw + manifest_record(xml)
    destination.mkdir(exist_ok=False)
    for leaf, data in (('dlpad-installer.res', combined), ('dlpad-installer.o', coff(resources(combined)))):
        with (destination / leaf).open('xb') as file:
            file.write(data)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--original-res', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    generate(args.original_res, args.manifest, args.output)
