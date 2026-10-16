"""Fault variants of host for real Win32 failures; NEVER publish as production host."""
import argparse
import json
from pathlib import Path
import sys


def variants(source):
    query = 'QueryInformationJobObject(job,1,out value,'
    close = 'CloseDesktop(desktop);'
    if source.count(query) != 1 or source.count(close) != 1:
        raise ValueError('exact current production fault seam required')
    query_variant = source.replace(query, 'QueryInformationJobObject(new IntPtr(-1),1,out value,')
    close_variant = source.replace(close, 'CloseDesktop(new IntPtr(-1));')
    marker = 'if(!desktopClosed)result=125;'
    if close_variant.count(marker) != 1:
        raise ValueError('exact close failure check required')
    close_variant = close_variant.replace(marker, 'if(!desktopClosed){int originalCloseError=Marshal.GetLastWin32Error();Console.Error.WriteLine("dlpad-test-close-native-error="+originalCloseError);result=125;}')
    return {'query-failure': query_variant, 'desktop-close-failure': close_variant}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--project', type=Path, default=Path.cwd())
    parser.add_argument('--fixture-source', type=Path, required=True)
    a = parser.parse_args()
    if any('..' in p.parts for p in [a.output, a.project, a.fixture_source]):
        parser.error('parent traversal rejected')
    project = a.project.absolute()
    if any(p.is_symlink() for p in [project, *project.parents]):
        parser.error('original project required')
    sys.path.insert(0, str(project / 'scripts'))
    import windows_current_workspace_probe as n
    out = a.output.absolute()
    if out.exists() or not out.is_relative_to(n.ROOT / 'docs/validation/evidence') or any(p.is_symlink() for p in [out, *out.parents]):
        parser.error('fresh owned project evidence')
    original_source = n.ROOT / 'scripts/windows_cli_host.cs'
    source_sha = n.sha(original_source)
    original_host_sha = n.sha(n.TOOLS / 'windows-cli-host.exe')
    if original_source.stat().st_size > 128 * 1024 or a.fixture_source.stat().st_size > 128 * 1024 or a.fixture_source.is_symlink():
        parser.error('source bound')
    source = original_source.read_text()
    sources = variants(source)
    sources['fixture'] = a.fixture_source.read_text()
    out.mkdir(mode=0o700)
    _, env = n.configure()
    host = str(n.TOOLS / 'windows-cli-host.exe')
    before = n.snapshot()
    records = {'schema': 1, 'test_fault_variants_not_product': True, 'source_before': before,
               'original_source_sha': source_sha, 'original_host_sha': original_host_sha, 'operations': {}}
    for kind, body in sources.items():
        cs = out / (kind + '.cs')
        with cs.open('x') as f:
            f.write(body)
        exe = out / (kind + '.exe')
        r = n.run('compile-' + kind, [host, '45', r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe',
            '/nologo', '/target:exe', '/out:' + n.win(exe), n.win(cs)], env, out, 45)
        records['operations']['compile-' + kind] = r
        if not r['passed']:
            raise RuntimeError('compile failed; raw failure retained')
    fixture = out / 'fixture.exe'
    for case in ['query-failure', 'desktop-close-failure', 'missing-executable']:
        variant = out / (case + '.exe') if case != 'missing-executable' else n.TOOLS / 'windows-cli-host.exe'
        target = fixture if case != 'missing-executable' else out / 'never-created-original.exe'
        # Outer ORIGINAL host owns variant, fixture and any descendant. A fault
        # variant's false cleanup receipt cannot establish actual tree cleanup.
        r = n.run(case, [host, '30', n.win(variant), '5', n.win(target), 'normal'], env, out, 30)
        log = (out / (case + '.log')).read_text(errors='replace')
        records['operations'][case] = r
        assert r['exit_code'] == 125 and not r['passed']
        assert 'dlpad-input-desktop-unchanged=True' in log
        assert 'dlpad-owned-job-terminal ' in log and 'result=125' in log
        # The SECOND terminal line is the original outer host's own observation.
        terminal = [line for line in log.splitlines() if line.startswith('dlpad-owned-job-terminal ')]
        assert len(terminal) == 2
        assert 'final-zero=True' in terminal[-1] and 'final-active=0' in terminal[-1] and 'private-desktop-closed=True' in terminal[-1]
        if case == 'query-failure':
            assert 'final-zero=False' in terminal[0] and 'error-type=Win32Exception' in log
        elif case == 'desktop-close-failure':
            assert 'private-desktop-closed=False' in terminal[0] and 'dlpad-test-close-native-error=6' in log
        else:
            assert 'error-type=Win32Exception' in log and 'os-error=2' in log
    records['source_after'] = n.snapshot()
    records['original_files_unchanged'] = source_sha == n.sha(original_source) and original_host_sha == n.sha(n.TOOLS / 'windows-cli-host.exe') and before == records['source_after']
    assert records['original_files_unchanged']
    records['failure_rejection_passed'] = True
    with (out / 'result.json').open('x') as f:
        json.dump(records, f, indent=2)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
