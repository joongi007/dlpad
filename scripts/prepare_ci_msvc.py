"""Create a CI-private compiler bin; never modify the installed MSVC toolchain."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import stat

MAX_FILES = 4096
MAX_FILE = 512 * 1024 * 1024
MAX_TOTAL = 2 * 1024 * 1024 * 1024
REQUIRED = {"cl.exe", "link.exe", "lib.exe"}


def ordinary(path):
    if path.is_symlink() or getattr(path.lstat(), "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0x400):
        raise ValueError("compiler input/output must not be a link or junction")


def prepare(source, destination):
    source, destination = Path(source).absolute(), Path(destination).absolute()
    for root in (source, destination.parent):
        for part in (root, *root.parents):
            ordinary(part)
    if not source.is_dir() or not destination.parent.is_dir():
        raise ValueError("existing compiler source and output parent required")
    if source == destination or source in destination.parents or destination in source.parents:
        raise ValueError("compiler copy must be independent")
    files = []
    total = 0
    for root, dirs, names in os.walk(source, followlinks=False):
        for name in sorted(dirs + names):
            path = Path(root) / name
            ordinary(path)
            if path.is_dir():
                continue
            if not path.is_file():
                raise ValueError("ordinary compiler files required")
            if path.name.casefold() == "vctip.exe":
                continue
            size = path.stat().st_size
            total += size
            if size > MAX_FILE or total > MAX_TOTAL or len(files) >= MAX_FILES:
                raise ValueError("compiler copy exceeds bounds")
            files.append((path, path.relative_to(source), size))
    if not REQUIRED <= {str(relative).casefold() for _, relative, _ in files}:
        raise ValueError("compiler cl/link/lib missing")
    destination.mkdir()  # Create-only: no cleanup or overwrite of existing tools.
    records = []
    for path, relative, size in files:
        output = destination / relative
        output.parent.mkdir(parents=True, exist_ok=True)
        digest = hashlib.sha256()
        copied = 0
        with path.open("rb") as reader, output.open("xb") as writer:
            while block := reader.read(65536):
                copied += len(block)
                if copied > size:
                    raise ValueError("compiler source grew during copy")
                digest.update(block)
                writer.write(block)
        if copied != size:
            raise ValueError("compiler source size changed")
        with output.open("rb") as reader:
            actual = hashlib.sha256()
            while block := reader.read(65536):
                actual.update(block)
            if actual.hexdigest() != digest.hexdigest():
                raise ValueError("compiler copy digest mismatch")
        records.append(dict(path=relative.as_posix(), bytes=size, sha256=digest.hexdigest()))
    return dict(schema=1, source=str(source), destination=str(destination),
                excluded="vctip.exe", bytes=total, files=records)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--destination", required=True, type=Path)
    parser.add_argument("--manifest", required=True, type=Path)
    args = parser.parse_args()
    record = prepare(args.source, args.destination)
    with args.manifest.open("x", encoding="utf-8") as stream:
        json.dump(record, stream, indent=2)
    print(f"CI-private MSVC: {len(record['files'])} files, {record['bytes']} bytes; installed tools unchanged")


if __name__ == "__main__":
    main()
