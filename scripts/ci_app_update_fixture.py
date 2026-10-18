"""Independent Windows updater fixture capture; the fixed seven whole-build operations stay unchanged.
Only fixed fixture lint/build/case argv run below an original gated Job. No GUI or release claim.
"""
import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import threading
import time
# Embedded Windows Python's isolated _pth omits the script directory, including
# for our gated worker. Resolve only this checkout's sibling helper modules.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import ci_capture_operation as capture
import ci_whole_receipt as receipt

BIN = 'app-update-fixture'
FEATURE = 'app-update-fixture'
CASES = ('success', 'start-fail', 'runtime-crash', 'bad-hash', 'cancel', 'busy', 'stale')
BUILD = {
    'fixture-clippy': ['cargo', 'clippy', '-p', 'dlpad-engine', '--bin', BIN, '--features', FEATURE, '--locked', '--offline', '--', '-D', 'warnings'],
    'fixture-build': ['cargo', 'build', '-p', 'dlpad-engine', '--bin', BIN, '--features', FEATURE, '--locked', '--offline'],
}
MAX_LINE = 8192
RETAINED = []

def binary_path(root):
    target = Path(os.environ.get('CARGO_TARGET_DIR', 'target'))
    if not target.is_absolute():
        receipt.reject('..' not in target.parts, 'target traversal')
        target = root / target
    receipt.no_reparse(target)
    configured = os.environ.get('CARGO_BUILD_TARGET')
    receipt.reject(configured in (None, 'x86_64-pc-windows-msvc'), 'native MSVC fixture target only')
    if configured:
        target /= configured
    return target / 'debug' / (BIN + '.exe')

def argv_for(root, operation, owned_parent=None):
    if operation in BUILD:
        return BUILD[operation].copy()
    receipt.reject(operation.startswith('case-') and operation[5:] in CASES, 'fixed fixture operation')
    receipt.reject(owned_parent is not None and owned_parent.is_absolute(), 'explicit absolute owned parent required')
    return [str(binary_path(root)), '--case', operation[5:], '--owned-parent', str(owned_parent)]

def worker(arguments):
    receipt.reject(os.name == 'nt' and len(arguments) == 3, 'Windows gated worker arguments')
    root = Path(arguments[0]); operation = arguments[1]
    parent = Path(arguments[2]) if arguments[2] else None
    argv = argv_for(root, operation, parent)
    # One byte cannot fill this fresh stdin pipe. EOF on parent loss starts no child.
    receipt.reject(sys.stdin.buffer.read(1) == b'G', 'original parent gate')
    return subprocess.run(argv, cwd=root, stdin=subprocess.DEVNULL, shell=False).returncode

def case_result(file, case):
    found = []
    with file.open('rb') as source:
        while raw := source.readline(MAX_LINE + 1):
            receipt.reject(len(raw) <= MAX_LINE, 'fixture stdout line bound')
            if raw.startswith(b'CASE_PASS '):
                value = json.loads(raw[len(b'CASE_PASS '):], object_pairs_hook=receipt.unique)
                found.append(value)
                receipt.reject(len(found) <= 1, 'duplicate fixture result')
    receipt.reject(len(found) == 1, 'exactly one fixture result required')
    value = found[0]
    receipt.reject(isinstance(value, dict) and set(value) == {'case', 'generation', 'highest', 'failed', 'active', 'requests', 'workers', 'preserved'}, 'fixture result fields')
    for name, maximum in [('generation', 2**32-1), ('highest', 2**64-1), ('requests', 64), ('workers', 64)]:
        receipt.reject(type(value[name]) is int and 0 <= value[name] <= maximum, 'fixture counter domain')
    receipt.reject(value['failed'] is None or (type(value['failed']) is int and 0 <= value['failed'] <= 2**64-1), 'fixture failure sequence')
    receipt.reject(value['active'] is None or (isinstance(value['active'], str) and len(value['active']) <= 32), 'fixture active version')
    receipt.reject(value['case'] == case and value['preserved'] is True, 'fixture case/preservation')
    expected = {
        'success': (1, 2, None, '0.1.2', 3, 1),
        'stale': (1, 2, None, '0.1.2', 3, 1),
        'start-fail': (1, 2, 2, None, 5, 2),
        'runtime-crash': (3, 2, 2, None, 5, 2),
        'bad-hash': (0, 0, None, None, 3, 1),
        'cancel': (0, 0, None, None, 3, 1),
        'busy': (0, 0, None, None, 3, 1),
    }
    receipt.reject(tuple(value[k] for k in ('generation','highest','failed','active','requests','workers')) == expected[case], 'fixture expected transition')
    return value

def run_operation(root, folder, operation, seconds, owned_parent=None):
    argv = argv_for(root, operation, owned_parent)
    out_path = folder / (operation + '.stdout'); err_path = folder / (operation + '.stderr')
    result_path = folder / (operation + '.json')
    receipt.reject(not result_path.exists(), 'fresh operation receipt')
    stdout = out_path.open('xb'); stderr = err_path.open('xb')
    process = None; job = None; threads = []; streams = [{}, {}]; failure = None
    facts = dict(schema=1, kind='windows-app-update-fixture-operation', name=operation, argv=argv,
                 native_run_performed=False, original_worker_job_assigned=False, natural_job_empty=False,
                 job_closed=False, forced_cleanup=False, reader_drained=False,
                 remote_execution_authenticated=False, native_gui_acceptance_verified=False, release_ready=False)
    try:
        args = [sys.executable, str(Path(__file__).absolute()), '--worker', str(root), operation, str(owned_parent) if owned_parent is not None else '']
        process = subprocess.Popen(args, cwd=root, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, shell=False)
        kernel, job = capture.job_for(process); facts['original_worker_job_assigned'] = True
        for source, target, result in zip((process.stdout, process.stderr), (stdout, stderr), streams):
            thread = threading.Thread(target=capture.pump, args=(source, target, result), daemon=True)
            thread.start(); threads.append(thread)
        deadline = time.monotonic() + seconds
        process.stdin.write(b'G'); process.stdin.close(); facts['native_run_performed'] = True
        while process.poll() is None:
            receipt.reject(not any('error' in result for result in streams), 'bounded raw reader failure')
            receipt.reject(time.monotonic() < deadline, 'fixture operation timeout')
            time.sleep(.02)
        facts['worker_exit_code'] = process.returncode
        terminal = capture.wait_natural_empty(lambda: capture.job_active(kernel, job), min(deadline, time.monotonic()+5))
        facts.update(natural_job_empty=True, final_active=terminal['active'], accounting_queries=terminal['queries'])
    except BaseException as error:
        failure = error
        facts.update(failure_kind=type(error).__name__, failure_message=str(error)[:1024])
        if job:
            try: facts['remaining_job_processes'] = capture.job_processes(kernel, job)
            except BaseException as diagnostic: facts['job_diagnostics_error'] = type(diagnostic).__name__
    finally:
        if job:
            facts['forced_cleanup'] = not facts['natural_job_empty']
            facts['job_closed'] = bool(kernel.CloseHandle(job))
            if not facts['job_closed']: failure = failure or OSError('original Job close failed')
        if process and process.poll() is None:
            facts['forced_cleanup'] = True
            process.kill()
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: failure = failure or TimeoutError('original worker terminal unconfirmed')
        if process and process.stdin and not process.stdin.closed:
            process.stdin.close()
        for thread in threads: thread.join(timeout=5)
        facts['reader_drained'] = bool(threads) and all(not t.is_alive() for t in threads) and all(s.get('drained') is True for s in streams)
        if not facts['reader_drained']: failure = failure or ValueError('fixture raw readers terminal unconfirmed')
    if facts['reader_drained']:
        stdout.close(); stderr.close(); facts['streams'] = streams
    else:
        RETAINED.extend([stdout, stderr, process, *threads]); facts['streams'] = {'terminal_unconfirmed': True}
    facts['passed'] = failure is None and facts.get('worker_exit_code') == 0 and facts['natural_job_empty'] and facts['job_closed'] and facts['reader_drained'] and not facts['forced_cleanup']
    if facts['passed'] and operation.startswith('case-'):
        try: facts['case_result'] = case_result(out_path, operation[5:])
        except (ValueError, OSError, UnicodeError, RecursionError, TypeError) as error:
            failure = error; facts['passed'] = False
            facts.update(failure_kind=type(error).__name__, failure_message=str(error)[:1024])
    if facts['reader_drained']:
        try: facts['logs'] = {'stdout': receipt.pin(out_path, capture.LOG_LIMIT), 'stderr': receipt.pin(err_path, capture.LOG_LIMIT)}
        except (ValueError, OSError) as error: failure = error; facts['passed'] = False
    if failure is not None:
        facts.setdefault('failure_kind', type(failure).__name__)
        facts.setdefault('failure_message', str(failure)[:1024])
    receipt.publish(result_path, facts)
    capture.report(operation, facts, failure, out_path, err_path)
    return facts

def build(root, folder, seconds):
    receipt.reject(not folder.exists() and folder.parent.is_dir(), 'fresh independent capture directory')
    folder.mkdir()
    deadline = time.monotonic() + seconds
    records = []
    for operation in BUILD:
        remaining = math.ceil(deadline - time.monotonic()); receipt.reject(remaining > 0, 'fixture build phase deadline')
        facts = run_operation(root, folder, operation, min(remaining, 1800)); records.append(facts)
        if not facts['passed']: return False
    binary = binary_path(root)
    pin = receipt.pin(binary, 128*1024*1024)
    receipt.publish(folder/'build.json', dict(schema=1,kind='windows-app-update-fixture-build',binary={'path':str(binary),**pin},operations=list(BUILD),operation_bindings={name:receipt.pin(folder/(name+'.json'),65536) for name in BUILD},release_ready=False,native_gui_acceptance_verified=False))
    return True

def verify_build_operation(folder, name, binding):
    file=folder/(name+'.json')
    receipt.reject(receipt.pin(file,65536)==binding,'build lifecycle binding changed')
    value=receipt.read_json(file,65536)
    receipt.reject(value.get('schema')==1 and value.get('kind')=='windows-app-update-fixture-operation' and value.get('name')==name and value.get('argv')==BUILD[name], 'fixed build operation')
    for key in ['passed','native_run_performed','original_worker_job_assigned','natural_job_empty','job_closed','reader_drained']:
        receipt.reject(value.get(key) is True,'successful original build lifecycle')
    for key in ['forced_cleanup','remote_execution_authenticated','native_gui_acceptance_verified','release_ready']:
        receipt.reject(value.get(key) is False,'build readiness/cleanup claim')
    receipt.reject(type(value.get('worker_exit_code')) is int and value['worker_exit_code']==0 and type(value.get('final_active')) is int and value['final_active']==0,'build original terminal')
    receipt.reject(type(value.get('accounting_queries')) is int and value['accounting_queries']>0,'build accounting evidence')
    streams=value.get('streams');logs=value.get('logs')
    receipt.reject(isinstance(streams,list) and len(streams)==2 and isinstance(logs,dict) and set(logs)=={'stdout','stderr'},'build raw streams')
    for label,stream in zip(['stdout','stderr'],streams):
        current=receipt.pin(folder/(name+'.'+label),capture.LOG_LIMIT)
        receipt.reject(isinstance(stream,dict) and stream.get('drained') is True and type(stream.get('bytes')) is int and stream['bytes']==current['bytes'] and logs[label]==current,'build original raw bytes')
    receipt.reject(receipt.pin(file,65536)==binding,'build lifecycle changed while reading')

def run(root, folder, parent, seconds):
    receipt.reject(folder.is_dir() and parent.is_absolute() and parent.is_dir(), 'existing explicit owned parent')
    receipt.no_reparse(parent)
    receipt.reject(not (folder/'whole.json').exists(), 'fresh final fixture receipt')
    evidence = receipt.read_json(folder/'build.json', 65536)
    binary = binary_path(root); expected = evidence.get('binary'); bindings=evidence.get('operation_bindings')
    receipt.reject(evidence.get('schema')==1 and evidence.get('kind')=='windows-app-update-fixture-build' and evidence.get('operations')==list(BUILD) and evidence.get('release_ready') is False and evidence.get('native_gui_acceptance_verified') is False and isinstance(expected,dict) and set(expected)=={'path','sha256','bytes'} and expected.get('path')==str(binary) and isinstance(bindings,dict) and set(bindings)==set(BUILD), 'fixture build binding')
    for name in BUILD:verify_build_operation(folder,name,bindings[name])
    pin = {k:expected[k] for k in ('sha256','bytes')}
    receipt.reject(receipt.pin(binary,128*1024*1024)==pin, 'fixture executable changed before run')
    deadline=time.monotonic()+seconds; records=[]
    for case in CASES:
        remaining=math.ceil(deadline-time.monotonic());receipt.reject(remaining>0,'fixture run phase deadline')
        facts=run_operation(root,folder,'case-'+case,min(remaining,180),parent);records.append(facts)
        receipt.reject(receipt.pin(binary,128*1024*1024)==pin,'fixture executable changed during run')
        if not facts['passed']:return False
    receipt.publish(folder/'whole.json',dict(schema=1,kind='windows-app-update-fixture-seven-cases',cases=list(CASES),binary=expected,build=receipt.pin(folder/'build.json',65536),operations={v['name']:receipt.pin(folder/(v['name']+'.json'),65536) for v in records},passed=True,remote_execution_authenticated=False,native_gui_acceptance_verified=False,release_ready=False))
    return True

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,required=True);parser.add_argument('--directory',default='ci-update-captures')
    parser.add_argument('--phase',choices=['build','run'],required=True);parser.add_argument('--owned-parent',type=Path)
    parser.add_argument('--seconds',type=int,default=900);args=parser.parse_args()
    receipt.reject(os.name=='nt' and 1<=args.seconds<=1800,'bounded Windows fixture runner')
    receipt.reject('..' not in args.root.parts,'root traversal');root=args.root.absolute();receipt.no_reparse(root)
    receipt.reject(root.is_dir(),'existing checkout');folder=receipt.path(root,args.directory)
    if args.phase=='run':receipt.reject(args.owned_parent is not None,'explicit --owned-parent required')
    return 0 if (build(root,folder,args.seconds) if args.phase=='build' else run(root,folder,args.owned_parent,args.seconds)) else 1

if __name__=='__main__':
    sys.exit(worker(sys.argv[2:]) if sys.argv[1:2]==['--worker'] else main())
