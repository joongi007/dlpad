"""Local Windows current-user DPAPI bridge. Never use with a logging subprocess wrapper.

protect reads a bounded private PEM from stdin and creates an encrypted file.
unprotect writes plaintext ONLY to a trusted local parent's pipe, never a terminal.
This does not send credentials or keys to GitHub or enable hosted signing.
"""
import argparse
import ctypes
from ctypes import wintypes
import os
from pathlib import Path
import stat
import sys

LIMIT = 16384
ENTROPY = b'dlpad.local-update-signing.v1'


def transform(data, protect):
    if os.name != 'nt' or not 0 < len(data) <= 65536:
        raise ValueError('bounded Windows input required')

    class Blob(ctypes.Structure):
        _fields_ = [('size', wintypes.DWORD), ('data', ctypes.POINTER(ctypes.c_ubyte))]

    def blob(raw):
        buf = (ctypes.c_ubyte * len(raw)).from_buffer_copy(raw)
        return Blob(len(raw), buf), buf

    source, source_buffer = blob(data)
    entropy, entropy_buffer = blob(ENTROPY)
    output = Blob()
    crypt = ctypes.WinDLL('crypt32', use_last_error=True)
    free = ctypes.WinDLL('kernel32', use_last_error=True).LocalFree
    free.argtypes = [ctypes.c_void_p]
    free.restype = ctypes.c_void_p
    function = crypt.CryptProtectData if protect else crypt.CryptUnprotectData
    function.argtypes = [ctypes.POINTER(Blob), ctypes.c_void_p, ctypes.POINTER(Blob),
                         ctypes.c_void_p, ctypes.c_void_p, wintypes.DWORD, ctypes.POINTER(Blob)]
    function.restype = wintypes.BOOL
    try:
        if not function(ctypes.byref(source), None, ctypes.byref(entropy), None, None,
                        1, ctypes.byref(output)):  # UI_FORBIDDEN, current user (not LOCAL_MACHINE)
            raise ValueError('local key protection failed')
        try:
            if not 0 < output.size <= (65536 if protect else LIMIT):
                raise ValueError('local key output bound')
            return ctypes.string_at(output.data, output.size)
        finally:
            free(output.data)
    finally:
        ctypes.memset(source_buffer, 0, len(data))
        ctypes.memset(entropy_buffer, 0, len(ENTROPY))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['protect', 'unprotect'])
    parser.add_argument('path', type=Path)
    args = parser.parse_args()
    if os.name != 'nt':
        raise ValueError('Windows required')
    # Refuse symlink/reparse ancestors; never adopt/overwrite a pre-existing output.
    for path in [args.path, *args.path.absolute().parents]:
        try:
            info = path.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
            raise ValueError('reparse key path rejected')
    if args.operation == 'protect':
        data = sys.stdin.buffer.read(LIMIT + 1)
        if not 0 < len(data) <= LIMIT:
            raise ValueError('private key input bound')
        protected = transform(data, True)
        if transform(protected, False) != data:
            raise ValueError('local key roundtrip failed')
        with args.path.open('xb') as stream:
            stream.write(protected)
            stream.flush()
            os.fsync(stream.fileno())
        print('local protected key created; no key material printed')
    else:
        if sys.stdout.isatty():
            raise ValueError('refusing plaintext key output to terminal')
        with args.path.open('rb') as stream:
            data = stream.read(65537)
        plaintext = transform(data, False)
        sys.stdout.buffer.write(plaintext)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError):
        raise SystemExit('local signing key operation failed; details omitted') from None
