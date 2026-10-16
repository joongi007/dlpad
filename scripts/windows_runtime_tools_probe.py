#!/usr/bin/env python3
"""Windows privateDesktop runtime selector capability, no real credentials/media."""
import argparse,json,shutil,sqlite3,hashlib
from pathlib import Path
import windows_current_workspace_probe as native

def main():
 p=argparse.ArgumentParser();p.add_argument('output',type=Path);p.add_argument('--case',choices=['empty','seeded','wrong-version'],default='empty');a=p.parse_args();out=a.output
 if '..' in out.parts or any(q.is_symlink() for q in [out,*out.parents]) or not out.is_absolute() or not out.is_relative_to(native.ROOT/'docs/validation/evidence') or out.exists():p.error('fresh evidence root required')
 out.mkdir(mode=0o700);root=out/'owned-fixture';root.mkdir();sentinel=root/'foreign-sentinel';sentinel.write_bytes(b'synthetic foreign preserve');sentinel_before=native.sha(sentinel);runout=out/'gui';runout.mkdir();binary=out/'dlpad-cpu.exe';shutil.copyfile(native.TOOLS/'target/release/dlpad-cpu.exe',binary);before=native.sha(binary);source=native.snapshot();values,env=native.configure();host=native.TOOLS/'windows-cli-host.exe';fw=r'C:\Windows\Microsoft.NET\Framework64\v4.0.30319'
 shutil.copyfile(native.ROOT/'scripts/windows_runtime_tools_observer.cs',out/'observer-source.cs')
 argv=[str(host),'30',fw+r'\csc.exe','/nologo','/reference:System.Drawing.dll','/reference:System.Web.Extensions.dll',*['/reference:'+fw+'\\WPF\\'+v+'.dll' for v in ['UIAutomationClient','UIAutomationTypes','WindowsBase']],'/out:'+native.win(out/'observer.exe'),native.win(out/'observer-source.cs')]
 compiled=native.run('compile',argv,env,out,30);actual=None
 if compiled['passed'] and a.case!='empty':
  managed=root/'data/managed-tools';managed.mkdir(parents=True);(managed/'.dlpad-owner').write_bytes(b'dlpad-managed-tools-v1\n');(managed/'.install-transfer-lock').write_bytes(b'')
  fixture_source=root/'runtime-fixture.cs'
  fixture_source.write_text('using System;using System.IO;class Fixture{static int Main(string[] a){string n=Path.GetFileNameWithoutExtension(Environment.GetCommandLineArgs()[0]);string v=Path.GetFileName(Path.GetDirectoryName(Environment.GetCommandLineArgs()[0]));v="1.0.0";if(n=="deno"){if(a.Length!=1||a[0]!="--version")return 2;Console.WriteLine("deno "+v+" (stable, release, x86_64-pc-windows-msvc)\\nv8 15.0.245.2-rusty\\ntypescript 6.0.3");}else{if(a.Length!=1||a[0]!="-version")return 2;Console.WriteLine(n+" version "+v+" Copyright (c) 2000-2026 the FFmpeg developers");}return 0;}}')
  # Two versions intentionally contain the same fixed 1.0.0 reporter: version 2 must fail.
  for tool in ['deno','ffmpeg']:
   for version in ['1.0.0','2.0.0']:
    d=managed/tool/version/'x86_64-pc-windows-msvc';d.mkdir(parents=True);components=[]
    names=['deno.exe'] if tool=='deno' else ['ffmpeg.exe','ffprobe.exe']
    for i,name in enumerate(names):
     compile_fixture=native.run('fixture-'+tool+'-'+version+'-'+str(i),[str(host),'30',fw+r'\csc.exe','/nologo','/out:'+native.win(d/name),native.win(fixture_source)],env,out,30)
     if not compile_fixture['passed']:raise RuntimeError('synthetic fixture compile failed')
     components.append(dict(name=name,bytes=(d/name).stat().st_size,sha256=native.sha(d/name),role='Executable' if i==0 else 'Auxiliary'))
    notice=b'synthetic notice';(d/'LICENSE').write_bytes(notice);components.append(dict(name='LICENSE',bytes=len(notice),sha256=hashlib.sha256(notice).hexdigest(),role='Notice'))
    package=dict(version=1,tool=tool,tool_version=version,target='x86_64-pc-windows-msvc',source_url='https://example.invalid/synthetic.zip',license='synthetic only',ownership='Managed',archive_bytes=123,archive_sha256='a'*64,components=components)
    manifest=dict(tool=tool,version=version,target=package['target'],source_url=package['source_url'],license=package['license'],ownership='Managed',sha256=components[0]['sha256'],executable=names[0]);(d/'manifest.json').write_text(json.dumps(manifest));(d/'archive-package.json').write_text(json.dumps(package))
  with sqlite3.connect(managed/'activation.sqlite') as db:
   db.execute('CREATE TABLE active(tool TEXT PRIMARY KEY,version TEXT NOT NULL,target TEXT NOT NULL)');db.executemany('INSERT INTO active VALUES(?,?,?)',[(t,'1.0.0','x86_64-pc-windows-msvc') for t in ['deno','ffmpeg']])
  (root/'seeded').write_text('wrong-version')
  original_files={str(q.relative_to(managed)):native.sha(q) for q in managed.rglob('*') if q.is_file() and q.name!='activation.sqlite'}
  (out/'seed-before.json').write_text(json.dumps(original_files,indent=2)+'\n')

 if compiled['passed']:actual=native.run('gui',[str(host),'120',native.win(out/'observer.exe'),native.win(binary),native.win(root),native.win(runout)],env,out,120)
 passed=compiled['passed'] and actual is not None and actual['passed'] and native.sha(binary)==before
 actual_scope_passed=actual is not None and actual['exit_code']==0 and actual['collector_failure'] is None and actual['private_desktop_reported'] and actual['input_desktop_unchanged'] and native.sha(binary)==before
 if actual_scope_passed and a.case!='empty':
  after_files={str(q.relative_to(managed)):native.sha(q) for q in managed.rglob('*') if q.is_file() and q.name!='activation.sqlite'}
  (out/'seed-after.json').write_text(json.dumps(after_files,indent=2)+'\n')
  assert original_files==after_files
  with sqlite3.connect(managed/'activation.sqlite') as db:assert db.execute('SELECT tool,version FROM active ORDER BY tool').fetchall()==[('deno','1.0.0'),('ffmpeg','1.0.0')]
 assert native.sha(sentinel)==sentinel_before
 result={'sentinel_sha256_before':sentinel_before,'sentinel_sha256_after':native.sha(sentinel),'observer_source_sha256':native.sha(out/'observer-source.cs'),'actual_gui_scope_passed':actual_scope_passed,'case':a.case,'passed':passed,'binary_sha256':before,'source_baseline':source,'compile':compiled,'actual':actual,'scope':'Actual Windows immutable GUI; wrong-version case checks rejection and prior active Use, no inactive successful activation; global source snapshot status remains separate'};(out/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passed':passed}));return 0 if passed else 2
if __name__=='__main__':raise SystemExit(main())
