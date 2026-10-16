"""Validate a pinned Windows dependency tree; retain whole-lock notice omissions."""
import hashlib
import json
import re
from pathlib import Path

MAX_LOG = 2 * 1024 * 1024
MAX_PACKAGES = 2048
DIGEST = re.compile(r'[0-9a-f]{64}\Z')
PACKAGE = re.compile(r'([A-Za-z0-9_][A-Za-z0-9_-]{0,127}) v([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.+-]+)?)(?: \(([^\r\n\x00]{1,4096})\))?(?: (\(\*\)))?\Z')
TAIL = ['tree', '--offline', '--locked', '--target', 'x86_64-pc-windows-gnu', '-p', 'dlpad', '--features', 'cpu', '--edges', 'normal,build', '--prefix', 'none', '--format', '{p}', '--no-dedupe']


def reject(message):
    raise ValueError(message)


def parse_log(raw):
    if len(raw) > MAX_LOG:
        reject('tree log bound')
    text = raw.decode('utf-8', errors='strict')
    found = set()
    banners = []
    for line in text.splitlines():
        if not line:
            continue
        if line.startswith(('dlpad-owned-cli ', 'dlpad-owned-job-terminal ', 'dlpad-input-desktop-unchanged=')):
            banners.append(line)
            continue
        m = PACKAGE.fullmatch(line)
        if not m:
            reject('unrecognized tree line')
        found.add(m[1] + '-' + m[2])
        if len(found) > MAX_PACKAGES:
            reject('tree package bound')
    if not found:
        reject('empty tree')
    # Verify terminal authority separately from n.run's older passed calculation.
    if not any(re.fullmatch(r'dlpad-owned-cli pid=[1-9][0-9]* private-desktop=true', b) for b in banners):
        reject('missing original private process receipt')
    terminal = 'dlpad-owned-job-terminal natural-zero=True final-zero=True final-active=0 forced=False job-closed=True private-desktop-closed=True stdin-cancel=False result=0'
    if banners.count(terminal) != 1 or banners.count('dlpad-input-desktop-unchanged=True') != 1:
        reject('missing safe terminal receipt')
    if any(b != terminal and b != 'dlpad-input-desktop-unchanged=True' and not re.fullmatch(r'dlpad-owned-cli pid=[1-9][0-9]* private-desktop=true', b) for b in banners):
        reject('unknown host receipt')
    return found


def classify(receipt, raw, inventory, current_snapshot, host, cargo, locked_packages):
    argv = receipt.get('argv')
    if not isinstance(argv, list) or len(argv) != len(TAIL) + 3 or argv[0] != host or argv[2] != cargo or argv[3:] != TAIL:
        reject('unexpected native command')
    if not isinstance(argv[1], str) or not argv[1].isdigit() or not 1 <= int(argv[1]) <= 60:
        reject('native deadline bound')
    if receipt.get('passed') is not True or receipt.get('exit_code') != 0 or receipt.get('collector_failure') is not None:
        reject('native operation failed')
    before, after = receipt.get('source_before'), receipt.get('source_after')
    if not isinstance(before, dict) or not before or before != after or any(not isinstance(v, str) or not DIGEST.fullmatch(v) for v in before.values()):
        reject('source receipt mismatch')
    if not DIGEST.fullmatch(current_snapshot.get('Cargo.lock', '')) or before.get('Cargo.lock') != current_snapshot['Cargo.lock']:
        reject('current lock differs')
    packages = parse_log(raw)
    entries = inventory.get('records')
    if not isinstance(entries, list) or len(entries) > MAX_PACKAGES:
        reject('inventory records required')
    keys = set()
    for entry in entries:
        if not isinstance(entry, dict):
            reject('invalid inventory record')
        name, version = entry.get('name'), entry.get('version')
        if not isinstance(name, str) or not isinstance(version, str) or not PACKAGE.fullmatch(name + ' v' + version):
            reject('invalid inventory key')
        key = name + '-' + version
        if key in keys:
            reject('duplicate inventory key')
        keys.add(key)
    missing = inventory.get('missing')
    if not isinstance(missing, list) or len(missing) > MAX_PACKAGES or any(not isinstance(x, str) for x in missing):
        reject('missing inventory bound')
    missing_keys = {x.split(': ', 1)[0] for x in missing}
    key_pattern = re.compile(r'([A-Za-z0-9_][A-Za-z0-9_-]{0,127})-([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.+-]+)?)\Z')
    if not isinstance(locked_packages, set) or not 1 <= len(locked_packages) <= MAX_PACKAGES or any(not isinstance(k, str) or not key_pattern.fullmatch(k) for k in locked_packages):
        reject('bounded exact lock package keys required')
    if any(not key_pattern.fullmatch(k) for k in missing_keys) or not missing_keys <= locked_packages or len(missing_keys) != len(missing):
        reject('unknown or duplicate missing key')
    if keys | missing_keys != locked_packages or not packages <= locked_packages:
        reject('inventory coverage differs from exact lock')
    return {'scope': 'Windows GNU normal/build CPU package reachability; whole-lock retained separately',
            'source_current': before == current_snapshot,
            'source_difference_keys': sorted(k for k in set(before) | set(current_snapshot) if before.get(k) != current_snapshot.get(k)),
            'lock_sha256': before['Cargo.lock'], 'log_sha256': hashlib.sha256(raw).hexdigest(),
            'tree_packages': sorted(packages), 'whole_lock_packages': sorted(locked_packages),
            'missing_in_tree': sorted(missing_keys & packages), 'missing_not_in_this_tree': sorted(missing_keys - packages),
            'whole_lock_missing_unchanged': missing,
            'non_reachability_is_omission_authority': False,
            'linked_inventory_complete': False, 'embedded_inventory_complete': False,
            'license_complete': False, 'release_ready': False}
