"""Collect original CI dependency notices; no legal completeness or publisher trust grant."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re

import ci_notice_graph as graph_format
import ci_registry_source_binding as registry
import ci_whole_receipt as receipt

MAX_DOC = 16 * 1024 * 1024
TARGET = 'x86_64-pc-windows-msvc'


def require(ok, message):
    if not ok:
        raise ValueError(message)


def raw(path, limit=MAX_DOC):
    path = registry.plain(Path(path).absolute())
    require(path.is_file(), 'regular input required')
    with path.open('rb') as stream:
        data = stream.read(limit + 1)
    require(len(data) <= limit, 'input byte bound')
    return data


def pin(data):
    return {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}


def obj(path, limit=MAX_DOC):
    return json.loads(raw(path, limit), object_pairs_hook=registry.unique)


def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2) + '\n').encode()


def tracked(root, whole, path):
    path = registry.plain(path.absolute())
    require(path.is_relative_to(root), 'tracked file escapes checkout')
    data = raw(path)
    require(whole['source_before'].get(path.relative_to(root).as_posix()) == pin(data), 'tracked source pin mismatch')
    return data


def collect(root, graph, metadata, lock, cargo_home, whole, output):
    root = registry.plain(Path(root).absolute())
    output = registry.plain(Path(output).absolute())
    require(not output.exists() and output.parent.is_dir(), 'fresh output required')
    whole_raw = raw(whole)
    proof = json.loads(whole_raw, object_pairs_hook=registry.unique)
    require(proof.get('source_before') == proof.get('source_after') and proof.get('source_before'), 'whole source equality')
    require(proof.get('target') == TARGET and proof.get('release_ready') is False, 'whole target/readiness')
    graph_raw, metadata_raw, lock_raw = raw(graph), raw(metadata), raw(lock)
    require(proof.get('lock') == pin(lock_raw), 'whole lock pin')
    observed = graph_format.classify(raw(root/'ci-app-tree.stdout.txt'), raw(root/'ci-engine-tree.stdout.txt'), metadata_raw, lock_raw)
    selected = json.loads(graph_raw, object_pairs_hook=registry.unique)
    require(selected == observed, 'original graph inputs mismatch')
    packages = graph_format.index(json.loads(metadata_raw, object_pairs_hook=registry.unique)['packages'], metadata=True)
    manifest = json.loads(tracked(root, proof, root/'docs/licenses/upstream/manifest.json'), object_pairs_hook=registry.unique)
    require(manifest.get('schema') == 1 and manifest.get('coverage_complete') is False, 'upstream schema')
    notices = {}
    normalized_names = set()
    total = 0

    def add(name, data):
        nonlocal total
        registry.leaf(name)
        receipt.relative_parts(name)
        require(name.casefold() not in normalized_names, 'duplicate/case-colliding notice entry')
        normalized_names.add(name.casefold())
        require(len(notices) < 4096 and total + len(data) <= MAX_DOC, 'notice aggregate bound')
        total += len(data)
        notices[name] = data
        return {'entry': name, **pin(data)}

    upstream = {}
    for key, source in manifest['sources'].items():
        data = tracked(root, proof, receipt.path(root, source['file']))
        require(pin(data) == {'bytes': source['bytes'], 'sha256': source['sha256']}, 'upstream original notice pin')
        upstream[key] = add('upstream/' + Path(source['file']).name, data)
    records = []
    home = registry.plain(Path(cargo_home).absolute())
    for row in selected['resolved_packages']:
        name, version = row['name'], row['version']
        key = name + '-' + version
        package = packages[(name, version)]
        source = registry.plain(Path(package['manifest_path']).absolute().parent)
        require(source.is_dir(), 'package source directory missing')
        candidates = []
        for index, child in enumerate(source.iterdir()):
            require(index < 4096, 'package entry count bound')
            if child.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE')) and child.is_file():
                candidates.append(child)
        if package.get('license_file'):
            candidate = Path(package['license_file'])
            if not candidate.is_absolute():
                candidate = source/candidate
            candidate = registry.plain(candidate.absolute())
            require(candidate.is_relative_to(source) and candidate.is_file(), 'license_file escapes or missing')
            candidates.append(candidate)
        if name == 'epaint_default_fonts':
            for filename in ('Hack-Regular.txt', 'OFL.txt', 'UFL.txt', 'emoji-icon-font-mit-license.txt'):
                candidates.append(source/'fonts'/filename)
        candidates = sorted(set(candidates))
        require(len(candidates) <= 60, 'package notice count bound')
        required = {'Cargo.toml': pin(raw(source/'Cargo.toml', 1024*1024))}
        if name == 'epaint_default_fonts':
            for filename in ('Hack-Regular.ttf', 'NotoEmoji-Regular.ttf', 'Ubuntu-Light.ttf', 'emoji-icon-font.ttf'):
                required['fonts/'+filename] = pin(raw(source/'fonts'/filename, 8*1024*1024))
            required['src/lib.rs'] = pin(raw(source/'src/lib.rs', 1024*1024))
        package_notices = []
        for candidate in candidates:
            require(candidate.is_relative_to(source), 'notice source escape')
            relative = candidate.relative_to(source).as_posix()
            data = raw(candidate, 1024*1024)
            required[relative] = pin(data)
            package_notices.append(add('packages/' + key + '/' + relative, data))
        original = None
        if row['source'] is None:
            require(source.is_relative_to(root), 'path package outside checkout')
            tracked(root, proof, source/'Cargo.toml')
            for candidate in candidates:
                tracked(root, proof, candidate)
            if not package_notices:
                # Workspace inherits the repository's actual MIT license.
                declaration = raw(source/'Cargo.toml').decode().split('[package]', 1)[-1].split('\n[', 1)[0]
                require(re.search(r'(?m)^license\.workspace\s*=\s*true\s*(?:#.*)?$', declaration), 'path package notice absent')
                package_notices.append(add('packages/' + key + '/LICENSE', tracked(root, proof, root/'LICENSE')))
        else:
            require(row['source'].startswith(('registry+', 'sparse+')) and source.is_relative_to(home/'registry'/'src'), 'unsupported dependency source')
            archive = home/'registry'/'cache'/source.parent.name/(key+'.crate')
            original = registry.validate(source, archive, name, version, row['lock_checksum'], required)
        for identifier in manifest.get('packages', {}).get(key, {}).get('sources', []):
            require(identifier in upstream, 'mapped upstream notice missing')
            package_notices.append(upstream[identifier])
        require(package_notices, 'dependency notice missing: ' + key)
        records.append({'name': name, 'version': version, 'source': row['source'], 'notices': package_notices, 'original_source_binding': original})
    require(0 < len(records) <= 4096, 'selected package count')
    result = {'schema': 1, 'target': TARGET, 'identity': proof['identity'], 'whole': pin(whole_raw), 'graph': pin(graph_raw), 'lock': pin(lock_raw), 'development_only': True, 'release_ready': False, 'legal_coverage_complete': False, 'records': records, 'files': {name: pin(data) for name, data in notices.items()}}
    # Prepare all bounded inputs first, then create a new output. Partial output is preserved.
    output.mkdir()
    for name, data in notices.items():
        destination = output/name
        destination.parent.mkdir(parents=True, exist_ok=True)
        with destination.open('xb') as stream:
            stream.write(data)
    with (output/'inventory.json').open('xb') as stream:
        stream.write(encoded(result))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('root', 'graph', 'metadata', 'lock', 'whole', 'output'):
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--cargo-home', type=Path, default=Path(os.environ.get('CARGO_HOME') or Path.home()/'.cargo'))
    collect(**vars(parser.parse_args()))


if __name__ == '__main__':
    main()
