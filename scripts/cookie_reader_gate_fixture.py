#!/usr/bin/env python3
"""Create bounded synthetic reader-gate fixtures; no manager, network, or signals."""
import argparse
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile

COOKIE = b'# Netscape HTTP Cookie File\n# synthetic fixture only\n.invalid\tTRUE\t/\tFALSE\t0\tsynthetic\tfixture-only\n'
SENTINEL = b'owned synthetic sentinel; preserve\n'
SLOTS = ('metadata', 'download')
MAX_FILES = 12
MAX_BYTES = 65536
TOOL = r'''import argparse,json,os,stat
from pathlib import Path
ROOT=Path(__file__).absolute().parent
COOKIE=COOKIE_LITERAL
CACHE=CACHE_LITERAL
EXPECTED_CACHE=CACHE_ID_LITERAL
p=argparse.ArgumentParser();p.add_argument('--slot',choices=['metadata','download'],required=True);p.add_argument('--cookies',type=Path,required=True);a=p.parse_args()
def reject():raise SystemExit('synthetic fixture input rejected')
def ident(info):return (info.st_dev,info.st_ino)
def private_dir(fd):
 info=os.fstat(fd)
 if not stat.S_ISDIR(info.st_mode) or info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)!=0o700:reject()
def chain(path):
 if not path.is_absolute() or '..' in path.parts or not path.is_relative_to(Path('/tmp')):reject()
 fds=[os.open('/',os.O_RDONLY|os.O_DIRECTORY|os.O_CLOEXEC)]
 try:
  for component in path.parts[1:]:
   fds.append(os.open(component,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fds[-1]))
  private_dir(fds[-1])
  return fds
 except BaseException:
  for fd in reversed(fds):os.close(fd)
  raise
fds=[];cookie_fd=None
try:
 rootfds=chain(ROOT);fds.extend(rootfds)
 cachefds=chain(Path(CACHE));fds.extend(cachefds)
 if ident(os.fstat(cachefds[-1]))!=EXPECTED_CACHE:reject()
 path=a.cookies
 if not path.is_absolute() or '..' in path.parts:reject()
 relative=path.relative_to(Path(CACHE))
 parts=relative.parts
 if len(parts)!=3 or parts[0]!='session-cookies' or not parts[1].startswith('cookie-session-') or parts[1]=='cookie-session-' or parts[2]!='cookies.txt':reject()
 parent=cachefds[-1];child_parent_ids=[]
 for component in parts[:-1]:
  parent=os.open(component,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=parent);fds.append(parent);private_dir(parent);child_parent_ids.append(ident(os.fstat(parent)))
 cookie_fd=os.open(parts[-1],os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=parent)
 before=os.fstat(cookie_fd)
 def private_file(info):
  if not stat.S_ISREG(info.st_mode) or info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)!=0o600 or info.st_nlink!=1 or info.st_size>1048576:reject()
 private_file(before)
 data=b''
 while len(data)<=1048576:
  block=os.read(cookie_fd,min(65536,1048577-len(data)))
  if not block:break
  data+=block
 if data!=COOKIE:reject()
 after=os.fstat(cookie_fd);private_file(after)
 if ident(before)!=ident(after) or before.st_size!=after.st_size:reject()
 rebound=os.open(parts[-1],os.O_RDONLY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=parent);fds.append(rebound)
 current=os.fstat(rebound);private_file(current)
 if ident(current)!=ident(before):reject()
 fresh=chain(path.parent);fds.extend(fresh)
 pinned_parent_ids=[ident(os.fstat(fd)) for fd in cachefds]+child_parent_ids
 if len(fresh)!=len(pinned_parent_ids) or [ident(os.fstat(fd)) for fd in fresh]!=pinned_parent_ids:reject()
 entries=[]
 with os.scandir(ROOT) as scan:
  for entry in scan:
   entries.append(Path(entry.path))
   if len(entries)>=12:reject()
 total=0
 for entry in entries:
  info=entry.lstat()
  if not stat.S_ISREG(info.st_mode) or info.st_nlink!=1 or info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)!=0o600:reject()
  total+=info.st_size
 if total+256>65536:reject()
 marker=a.slot+'-tool-started.json'
 fd=os.open(marker,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=rootfds[-1])
 with os.fdopen(fd,'wb') as f:
  f.write((json.dumps({'slot':a.slot,'event':'tool-started','synthetic':True})+'\n').encode());f.flush();os.fsync(f.fileno())
except (OSError,ValueError):reject()
finally:
 if cookie_fd is not None:os.close(cookie_fd)
 for fd in reversed(fds):os.close(fd)
print(json.dumps({'fixture':'cookie-reader-gate','slot':a.slot,'synthetic':True}))
'''


def namespace(root, existing=True):
    if not root.is_absolute() or '..' in root.parts or not root.is_relative_to(Path('/tmp')) or root == Path('/tmp'):
        raise ValueError('absolute dedicated /tmp namespace required')
    for part in [root, *root.parents]:
        if part.is_symlink():
            raise ValueError('symlink rejected')
    if existing:
        info = root.stat()
        if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o700:
            raise ValueError('private owned namespace required')
    return root


def bounded_state(root):
    entries = []
    with os.scandir(root) as scan:
        for entry in scan:
            entries.append(entry)
            if len(entries) > MAX_FILES:
                raise ValueError('fixture file bound exceeded')
    total = 0
    for entry in entries:
        info = entry.stat(follow_symlinks=False)
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or stat.S_IMODE(info.st_mode) != 0o600 or info.st_nlink != 1:
            raise ValueError('fixture entry rejected')
        total += info.st_size
        if total > MAX_BYTES:
            raise ValueError('fixture byte bound exceeded')
    return len(entries), total


def create_file(root, name, data):
    count, size = bounded_state(root)
    if count >= MAX_FILES or size + len(data) > MAX_BYTES:
        raise ValueError('fixture capacity exceeded')
    fd = os.open(root / name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'wb') as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def create(root, appcache):
    namespace(root, False)
    namespace(appcache)
    if appcache.is_relative_to(root) or root.is_relative_to(appcache):
        raise ValueError('separate private appcache required')
    cache_info = appcache.stat()
    cache_identity = (cache_info.st_dev, cache_info.st_ino)
    root.mkdir(mode=0o700, parents=False, exist_ok=False)
    create_file(root, 'source.cookies', COOKIE)
    create_file(root, 'sentinel', SENTINEL)
    tool = TOOL.replace('COOKIE_LITERAL', repr(COOKIE)).replace('CACHE_ID_LITERAL', repr(cache_identity)).replace('CACHE_LITERAL', repr(str(appcache)))
    create_file(root, 'tool.py', tool.encode())
    manifest = {'synthetic': True, 'slots': list(SLOTS), 'max_files': MAX_FILES, 'max_bytes': MAX_BYTES,
                'cookie_read_limit': 1048576, 'expected_appcache': str(appcache), 'expected_appcache_identity': list(cache_identity), 'cookie_relative_contract': 'session-cookies/cookie-session*/cookies.txt', 'tool_argv': [sys.executable, str(root / 'tool.py'), '--slot', 'SLOT', '--cookies', 'EXPECTED_APPCACHE/session-cookies/cookie-session-ID/cookies.txt'],
                'checkpoint_argv': [sys.executable, str(Path(__file__).absolute()), 'checkpoint', str(root), 'SLOT', 'before_B|after_B'],
                'marker_is_not_security_boundary': True,
                'actual_proof_requires': ['product B wire capture', 'product durable receipt', 'native ownership cleanup']}
    create_file(root, 'manifest.json', (json.dumps(manifest, indent=2) + '\n').encode())


def checkpoint(root, slot, phase):
    namespace(root)
    create_file(root, f'{slot}-{phase}.json', (json.dumps({'slot': slot, 'event': phase, 'source': 'driver-requested checkpoint', 'actual_wire_proof': False}) + '\n').encode())


def inspect(root):
    namespace(root)
    count, size = bounded_state(root)
    return {'synthetic': True, 'files': count, 'bytes': size,
            'slots': {slot: {phase: (root / f'{slot}-{phase}.json').is_file() for phase in ('before_B', 'after_B', 'tool-started')} for slot in SLOTS}}


def selftest():
    with tempfile.TemporaryDirectory(prefix='dlpad-gate-selftest-') as tmp:
        base = Path(tmp)
        cache = base / 'appcache'
        cache.mkdir(mode=0o700)
        owner = cache / 'session-cookies' / 'cookie-session-synthetic'
        (cache / 'session-cookies').mkdir(mode=0o700)
        owner.mkdir(mode=0o700)
        copy = owner / 'cookies.txt'
        copy.write_bytes(COOKIE)
        copy.chmod(0o600)
        root = base / 'fixture'
        create(root, cache)
        assert not inspect(root)['slots']['metadata']['tool-started']
        checkpoint(root, 'metadata', 'before_B')
        assert not inspect(root)['slots']['metadata']['tool-started']
        def tool(namespace_root, slot, path):
            return subprocess.run([sys.executable, str(namespace_root / 'tool.py'), '--slot', slot, '--cookies', str(path)], stdin=subprocess.DEVNULL, capture_output=True, timeout=3, check=False)
        for slot in SLOTS:
            if slot == 'download': checkpoint(root, slot, 'before_B')
            checkpoint(root, slot, 'after_B')
            result = tool(root, slot, copy)
            assert result.returncode == 0 and len(result.stdout) < 256 and not result.stderr
        for fn in (lambda: create(root, cache), lambda: checkpoint(root, 'metadata', 'before_B')):
            try: fn()
            except FileExistsError: pass
            else: raise AssertionError('collision accepted')
        negative = base / 'negative'
        create(negative, cache)
        other_cache = base / 'other-cache'
        other_cache.mkdir(mode=0o700)
        other_owner = other_cache / 'session-cookies' / 'cookie-session-other'
        (other_cache / 'session-cookies').mkdir(mode=0o700)
        other_owner.mkdir(mode=0o700)
        other_copy = other_owner / 'cookies.txt'
        other_copy.write_bytes(COOKIE); other_copy.chmod(0o600)
        for path in (root / 'source.cookies', other_copy, base / 'outside.cookies'):
            assert tool(negative, 'metadata', path).returncode != 0
        for data in (b'non-fixture-cookie', b'x' * 1048577):
            copy.write_bytes(data)
            assert tool(negative, 'metadata', copy).returncode != 0
        copy.write_bytes(COOKIE)
        copy.chmod(0o644)
        assert tool(negative, 'metadata', copy).returncode != 0
        copy.chmod(0o600)
        hardlink = base / 'hardlinked'
        os.link(copy, hardlink)
        assert tool(negative, 'metadata', copy).returncode != 0
        hardlink.unlink()
        copy.unlink(); copy.symlink_to(other_copy)
        assert tool(negative, 'metadata', copy).returncode != 0
        copy.unlink(); copy.write_bytes(COOKIE); copy.chmod(0o600)
        old_cache = base / 'old-cache'
        cache.rename(old_cache)
        cache.mkdir(mode=0o700)
        assert tool(negative, 'metadata', cache / 'session-cookies/cookie-session-synthetic/cookies.txt').returncode != 0
        cache.rmdir(); old_cache.rename(cache)
        assert (root / 'source.cookies').read_bytes() == COOKIE and (root / 'sentinel').read_bytes() == SENTINEL
    print(json.dumps({'selftest_passed': True, 'actual_B_gate_tested': False}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    for command in ('create', 'inspect', 'checkpoint'):
        item = sub.add_parser(command)
        item.add_argument('root', type=Path)
        if command == 'create':
            item.add_argument('--appcache', type=Path, required=True)
        if command == 'checkpoint':
            item.add_argument('slot', choices=SLOTS)
            item.add_argument('phase', choices=('before_B', 'after_B'))
    sub.add_parser('selftest')
    args = parser.parse_args()
    if args.command == 'create': create(args.root, args.appcache)
    elif args.command == 'checkpoint': checkpoint(args.root, args.slot, args.phase)
    elif args.command == 'inspect': print(json.dumps(inspect(args.root)))
    else: selftest()

if __name__ == '__main__':
    main()
