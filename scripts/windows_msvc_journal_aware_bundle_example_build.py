"""Owned current-source native example producer; no GUI or runtime installation."""
import argparse, hashlib, json, os, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import windows_current_workspace_probe as n
from windows_msvc_workspace_probe import require_current_whole
from windows_selected_gui_collect import plain, bounded, sha, guard, copy, write
p=argparse.ArgumentParser();p.add_argument('--proof',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
before=n.snapshot();proof=plain(a.proof);raw=bounded(proof);whole=json.loads(raw);require_current_whole(whole,before)
for name,op in whole['operations'].items():guard(op,bounded(proof.parent/(name+'.log')))
out=plain(a.output)
if out.exists() or not out.is_relative_to(n.ROOT/'docs/validation/evidence'):raise ValueError('fresh project output required')
config_path=plain(proof.parent/'effective-configuration.json')
config_raw=bounded(config_path,65536);configuration=json.loads(config_raw)
values=dict(configuration['values']);values['CARGO_BUILD_JOBS']='2'
expected_target=n.TOOLS/'msvc-supply-20261006/target-msvc'
if values.get('CARGO_TARGET_DIR','').replace('\\','/').lower()!=n.win(expected_target).replace('\\','/').lower():raise ValueError('exact owned default target required')
_,env=n.configure()
removed=set(configuration['remove_inherited'])
for key in removed:env.pop(key,None)
env['WSLENV']=':'.join(s for s in env.get('WSLENV','').split(':') if s.split('/')[0] not in removed)
env.update(values)
forward=[s for s in env.get('WSLENV','').split(':') if s]
for key in values:
 if key not in {s.split('/')[0] for s in forward}:forward.append(key)
env['WSLENV']=':'.join(forward)
base=n.TOOLS/'msvc-supply-20261006/target-msvc/x86_64-pc-windows-msvc/release'
originals={str(base/leaf):whole['artifacts'][leaf] for leaf in ['dlpad-cpu.exe','dlpad-local-install.exe']}
originals[str(proof)]={'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
originals[str(config_path)]={'sha256':hashlib.sha256(config_raw).hexdigest(),'bytes':len(config_raw)}
for path,pin in originals.items():
 f=plain(Path(path));assert {'sha256':sha(f),'bytes':f.stat().st_size}==pin
for helper in [Path(__file__).resolve(),Path(n.__file__).resolve(),Path(__file__).resolve().parent/'windows_msvc_workspace_probe.py',Path(__file__).resolve().parent/'windows_selected_gui_collect.py']:
 originals[str(helper)]={'sha256':sha(helper),'bytes':helper.stat().st_size}
out.mkdir();(out/'inputs').mkdir();copy(proof,out/'inputs/whole-proof.json');copy(config_path,out/'inputs/effective-configuration.json')
assert sha(out/'inputs/whole-proof.json')==hashlib.sha256(raw).hexdigest()
assert sha(out/'inputs/effective-configuration.json')==hashlib.sha256(config_raw).hexdigest()
result={'status':'FAIL','source_before':before,'target':'x86_64-pc-windows-msvc','whole_proof_sha256':hashlib.sha256(raw).hexdigest(),'originals':originals}
try:
 args=['build','-p','dlpad-engine','--example','windows_journal_aware_bundle_fixture','--locked','--offline','--release','--target','x86_64-pc-windows-msvc']
 op=n.run('example-build',[str(n.TOOLS/'windows-cli-host.exe'),'1800',n.win(n.TOOLS/'rust/bin/cargo.exe'),*args],env,out,1800);result['operation']=op;guard(op,bounded(out/'example-build.log'))
 assert before==n.snapshot()
 for path,pin in originals.items():
  f=Path(path);assert {'sha256':sha(f),'bytes':f.stat().st_size}==pin
 binary=plain(base/'examples/windows_journal_aware_bundle_fixture.exe');copy(binary,out/'inputs/example.exe')
 result.update(status='REVIEW_REQUIRED',source_after=n.snapshot(),binary=str(out/'inputs/example.exe'),artifact={'sha256':sha(binary),'bytes':binary.stat().st_size},gui_verified=False,release_ready=False)
finally:write(out/'result.json',result)
