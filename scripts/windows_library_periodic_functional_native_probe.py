#!/usr/bin/env python3
"""Explicit owned Windows periodic GUI fixture; no visible desktop fallback.

Builds the existing default-OFF fixture feature. A fixture PE is retained and
never promoted as a default release. Native event observations and independent
baseline DB/payload review are required; GUI CPU acceptance is separate.
"""
import argparse
import json
import ntpath
import os
import stat
import re
import shutil
import sqlite3
from pathlib import Path
import windows_current_workspace_probe as n


def arguments(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--case', choices=['completion', 'next', 'stop'], required=True)
    parser.add_argument('--thread-cpu-observer', action='store_true', help='fixed original-thread CPU diagnostic collector; not GUI cost acceptance')
    args = parser.parse_args(argv)
    root, output = args.project, args.output
    if (not root.is_absolute() or '..' in root.parts or not root.is_dir()
            or root.is_symlink() or not root.samefile(n.ROOT)):
        parser.error('original project required')
    if (not output.is_absolute() or '..' in output.parts
            or not output.is_relative_to(root / 'docs/validation/evidence')
            or output.exists() or any(p.is_symlink() for p in [output, *output.parents])):
        parser.error('fresh project evidence required')
    return args



def require(condition, message):
    if not condition:
        raise ValueError(message)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')


def parse_seed(log, environment):
    require(log.stat().st_size <= 128 * 1024, 'seed log byte bound')
    rows = [json.loads(line) for line in log.read_text().splitlines()
            if line.startswith('{')]
    require(len(rows) == 1, 'exact seed summary required')
    seed = rows[0]
    require(seed['phase'] == 'seed-ready' and seed['count'] == 10000,
            'synthetic seed contract')
    require(ntpath.dirname(seed['owned_root']).casefold() == environment['TEMP'].casefold(),
            'original TEMP namespace mismatch')
    root = n.ROOT / '.local-tools/windows/tmp' / ntpath.basename(seed['owned_root'])
    require(root.name.startswith('dlpad-library-scan-closewait-owned-')
            and root.is_dir() and not any(p.is_symlink() for p in [root, *root.parents]),
            'owned fixture namespace')
    require(n.sha(root / 'fixture.cap') == seed['fixture_cap_sha256'], 'seed CAP changed')
    return seed, root


def database_rows(path):
    require(path.is_file() and not path.is_symlink()
            and path.stat().st_size <= 16 * 1024 * 1024, 'database regular/byte bound')
    with sqlite3.connect(path.as_uri() + '?mode=ro', uri=True) as database:
        require(database.execute('pragma integrity_check').fetchone() == ('ok',),
                'database integrity')
        return database.execute(
            'select rowid,root,path,size,modified_ms,present,file_identity '
            'from library_entries order by path LIMIT 10001').fetchall()


def validate_native_case(native, scenario):
    if native.get('scenario') != scenario:
        raise ValueError('native scenario mismatch')
    if type(native.get('natural_exitcode')) is not int or native['natural_exitcode'] != 0:
        raise ValueError('native original exit')
    if type(native.get('exact_inherited_event_caps')) is not int or native['exact_inherited_event_caps'] != 4:
        raise ValueError('exact four capability contract')
    if scenario == 'completion':
        duration = native.get('periodic_commit_ms')
        if type(duration) is not int or not 0 <= duration <= 180000:
            raise ValueError('actual periodic commit duration')
    elif scenario == 'next':
        duration = native.get('next_response_ms')
        if type(duration) is not int or not 0 <= duration <= 5000:
            raise ValueError('actual Next duration')
    elif scenario == 'stop':
        if native.get('stop_pre_release_ack_false') is not True or native.get('stop_library_controls_disabled') is not True:
            raise ValueError('actual Stop gate proof')
    else:
        raise ValueError('unknown scenario')


def synthetic_payload_intact(path):
    # This WSL orchestrator uses one no-follow descriptor instead of repeated
    # pathname metadata RPCs. It does not establish Windows native file identity.
    if not hasattr(os, 'O_NOFOLLOW'):
        raise ValueError('no-follow fixture reader unavailable')
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | getattr(os, 'O_CLOEXEC', 0))
    except OSError:
        return False
    try:
        observed = os.fstat(descriptor)
        return (stat.S_ISREG(observed.st_mode) and observed.st_size == 3
                and os.read(descriptor, 4) == b'\x01\x02\x03')
    finally:
        os.close(descriptor)


def review_fixture(root, gui, seed, scenario, bindings):
    native = json.loads((gui / 'result.json').read_text())
    validate_native_case(native, scenario)
    require(all(native.get(key) == value for key, value in bindings.items()),
            'original native input binding mismatch')
    require(native['status'] == 'REVIEW_REQUIRED' and native['natural_exitcode'] == 0,
            'original native terminal result')
    for key in ['actual_periodic_first_insert', 'owned_job_naturally_empty',
                'owned_job_empty_after_cleanup', 'private_desktop_closed',
                'input_desktop_unchanged', 'baseline_database_copied_at_actual_insert_gate']:
        require(native.get(key) is True, 'native proof absent: ' + key)
    baseline = database_rows(gui / 'baseline-library.sqlite')
    final = database_rows(root / 'data/library.sqlite')
    require(baseline == final and len(final) == 10000, 'baseline/final rows changed')
    expected = {f'media-{index:05}.mp4' for index in range(10000)}
    require({ntpath.basename(row[2]) for row in final} == expected, 'row membership')
    require(all(row[3] == 3 and row[5] == 1
                and isinstance(row[6], str)
                and re.fullmatch(r'windows:v1:[0-9]+:[0-9a-f]{32}', row[6])
                for row in final), 'row size/presence/identity shape')
    media = root / 'media'
    for name in expected:
        payload = media / name
        require(synthetic_payload_intact(payload),
                'synthetic payload changed: ' + name)
    require(sum(1 for _ in media.iterdir()) == 10000, 'media membership count')
    require(n.sha(root / 'foreign-sentinel') == seed['sentinel_sha256']
            and n.sha(root / 'fixture.cap') == seed['fixture_cap_sha256'],
            'source CAP/sentinel changed')
    return {'passed': True, 'actual_native': native,
            'baseline_final_all_10000_rows_exact': True,
            'all_10000_payloads_and_cap_sentinel_preserved': True,
            'baseline_copy_is_atomic_snapshot': False,
            'native_identity_reopened_for_all_files': False,
            'gui_cpu_budget_verified': False,
            'original_native_input_bindings_verified': True}


def main(argv=None):
    args = arguments(argv)
    root, output = args.project, args.output
    output.mkdir()
    inputs = output / 'inputs'
    inputs.mkdir()
    before = n.snapshot()
    operations = []
    stage = 'native-configuration'

    def run(name, command, bound):
        record = n.run(name, [host, str(bound), *command], environment, output, bound)
        operations.append(record)
        require(record['passed'], name + ' failed; raw operation retained')

    def finish(passed, error=None):
        write_json(output / 'result.json', {
            'passed': passed, 'error': error, 'failure_scope': None if passed else stage,
            'operations': operations, 'product_source_unchanged': before == n.snapshot(),
            'default_cpu_artifact_promoted': False,
            'original_thread_cpu_diagnostic': args.thread_cpu_observer,
            'scope': 'default-OFF periodic functional fixture; CPU budget separate'})

    try:
        _, environment = n.configure()
        host = str(n.TOOLS / 'windows-cli-host.exe')
        cargo = n.win(n.TOOLS / 'rust/bin/cargo.exe')
        framework = r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
        stage = 'pin-observer-and-seeder'
        observer_name = ('windows_library_periodic_thread_cpu_probe.cs' if args.thread_cpu_observer else 'windows_library_periodic_functional_probe.cs')
        for name in [observer_name, 'windows_library_periodic_seed.cs']:
            shutil.copyfile(root / 'scripts' / name, inputs / name)
        pins = {path.name: n.sha(path) for path in inputs.iterdir()}
        original_input_pins = {name: n.sha(root / 'scripts' / name) for name in pins}
        write_json(output / 'input-pins.json',
                   {'production_source': before, 'input_sha256': pins})
        stage = 'fixture-feature-build'
        run('feature-build', [cargo, 'build', '-p', 'dlpad', '--bin', 'dlpad-cpu',
                             '--release', '--offline', '--locked', '--features',
                             'cpu,library-scan-closewait-fixture'], 900)
        binary = inputs / 'dlpad-cpu-fixture.exe'
        shutil.copyfile(n.TOOLS / 'target/release/dlpad-cpu.exe', binary)
        binary_sha = n.sha(binary)
        references = ['/reference:System.Drawing.dll', '/reference:System.Web.Extensions.dll']
        references += ['/reference:' + framework + '\\WPF\\' + name + '.dll'
                       for name in ['UIAutomationClient', 'UIAutomationTypes', 'WindowsBase']]
        observer, seeder = output / 'observer.exe', output / 'seed.exe'
        source = inputs / observer_name
        stage = 'observer-and-seeder-compile'
        run('observer-compile', [framework + '\\csc.exe', '/nologo', *references,
                                '/out:' + n.win(observer), n.win(source)], 45)
        run('seed-compile', [framework + '\\csc.exe', '/nologo',
                            '/reference:System.Web.Extensions.dll',
                            '/out:' + n.win(seeder),
                            n.win(inputs / 'windows_library_periodic_seed.cs')], 45)
        executable_pins = {path.name: n.sha(path) for path in [observer, seeder]}
        write_json(output / 'native-binary-pins.json',
                   {'fixture_cpu_sha256': binary_sha, 'compiled_sha256': executable_pins})
        scenario = args.case
        stage = 'synthetic-seed'
        run('seed-' + scenario, [n.win(seeder), '--fixed-periodic-seed'], 210)
        seed, owned_root = parse_seed(output / ('seed-' + scenario + '.log'), environment)
        gui = output / ('gui-' + scenario)
        stage = 'private-periodic-native-' + scenario
        run('gui-' + scenario, [n.win(observer), n.win(binary), binary_sha,
                               seed['owned_root'], seed['fixture_cap_sha256'],
                               n.win(gui), n.win(source), scenario], 290)
        stage = 'independent-database-and-payload-review'
        review = review_fixture(owned_root, gui, seed, scenario, {
            'binary_sha256': binary_sha,
            'observer_source_sha256': pins[source.name],
            'owned_root': seed['owned_root'],
            'fixture_cap_sha256': seed['fixture_cap_sha256'],
            'sentinel_sha256': seed['sentinel_sha256'],
        })
        review['scenario'] = scenario
        write_json(output / ('root-' + scenario + '-review.json'), review)
        stage = 'final-source-and-input-pins'
        require(before == n.snapshot(), 'production source changed')
        require(n.sha(binary) == binary_sha, 'fixture CPU changed')
        require(all(n.sha(root / 'scripts' / name) == digest for name, digest in original_input_pins.items()), 'original observer/seeder changed')
        require(all(n.sha(inputs / name) == digest for name, digest in pins.items()),
                'observer/seeder source changed')
        require(all(n.sha(output / name) == digest
                    for name, digest in executable_pins.items()), 'compiled input changed')
        finish(True)
        print('PERIODIC FUNCTIONAL PASS; GUI CPU separate')
    except BaseException as error:
        finish(False, type(error).__name__ + ': ' + str(error))
        raise


if __name__ == '__main__':
    main()
