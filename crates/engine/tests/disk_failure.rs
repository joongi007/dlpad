//! Owned writer failures through the real process/SQLite adapter.
//! RLIMIT_FSIZE yields EFBIG; the ENOSPC case is explicitly software injection.
#![cfg(target_os = "linux")]

use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    download::{DownloadError, DownloadRepository},
    process,
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    sync::{Arc, atomic::AtomicBool},
};

const TOOL: &str = r#"#!/usr/bin/python3
import errno,json,pathlib,resource,signal,sys
base=pathlib.Path(__file__).parent
args=sys.argv[1:]
if '--version' in args: print('2026.10.02');sys.exit(0)
if '--dump-single-json' in args:
 with (base/'extractions').open('a') as f: f.write(args[-1]+'\n')
 print(json.dumps({'id':'synthetic','duration':1}));sys.exit(0)
mode=(base/'mode').read_text()
root=pathlib.Path(args[args.index('--paths')+1])
final=root/args[args.index('--output')+1].replace('%(ext)s','mp4')
partial=final.with_suffix('.part')
if mode=='success':
 assert partial.read_bytes()==b'p'*65536, 'partial must survive unchanged'
 final.write_bytes(b'synthetic recovered')
 print('DLPAD_ARTIFACT:'+json.dumps(str(final)),flush=True);sys.exit(0)
partial.write_bytes(b'p'*65536)
try:
 if mode=='efbig':
  signal.signal(signal.SIGXFSZ,signal.SIG_IGN)
  resource.setrlimit(resource.RLIMIT_FSIZE,(4096,4096))
  with partial.open('ab',buffering=0) as f:f.write(b'x')
 elif mode=='injected-enospc':
  # Deterministic writer error, not actual kernel/filesystem exhaustion.
  raise OSError(errno.ENOSPC,'synthetic injected writer error')
 else: raise RuntimeError('unknown fixture mode')
except OSError as e:
 print('DLPAD_WRITER_ERROR:'+json.dumps({'errno':e.errno,'mode':mode}),file=sys.stderr,flush=True)
 sys.exit(7)
raise RuntimeError('writer was expected to fail')
"#;

fn writer_failure_then_resume(mode: &str, errno: i32) {
    let dir = tempfile::tempdir().unwrap();
    let tool = dir.path().join("synthetic-writer");
    fs::write(&tool, TOOL).unwrap();
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(dir.path().join("mode"), mode).unwrap();
    let output = dir.path().join("media 한글");
    fs::create_dir(&output).unwrap();
    let unrelated = output.join("unrelated-user-shaped-fixture.mp4");
    fs::write(&unrelated, b"preserve independent owned fixture").unwrap();
    let options = DownloadOptions {
        url: SourceUrl::parse("https://fixture.invalid/original-source").unwrap(),
        output_dir: output.clone(),
        quality: Quality::Best,
        time_range: None,
        advanced: vec![],
        ffmpeg: None,
        deno: None,
        final_name_style: None,
        subtitles: None,
    };
    let db = dir.path().join("jobs.sqlite");
    let repo = DownloadRepository::open(&db).unwrap();
    repo.create_job("writer-failure", &options).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let failure = repo
        .execute("writer-failure", &options, &tool, &cancel)
        .err()
        .expect("writer must fail");
    let DownloadError::Execution { stage, result } = failure else {
        panic!("writer failure must use process execution error");
    };
    assert_eq!(stage, "download");
    assert!(result.logs.len() <= process::MAX_LOG_BYTES);
    let evidence = result
        .logs
        .lines()
        .find_map(|line| line.strip_prefix("DLPAD_WRITER_ERROR:"))
        .expect("bounded writer errno evidence");
    let evidence: serde_json::Value = serde_json::from_str(evidence).unwrap();
    assert_eq!(evidence["errno"], errno);
    assert_eq!(evidence["mode"], mode);
    let failed = repo.job("writer-failure").unwrap();
    assert_eq!(failed.state, JobState::Failed);
    assert_eq!(failed.attempts.len(), 1);
    assert!(failed.artifacts.is_empty());
    let partial = output.join(".dlpad-partials/writer-failure/writer-failure.part");
    assert_eq!(fs::read(&partial).unwrap(), vec![b'p'; 65536]);
    assert!(!output.join("writer-failure.mp4").exists());
    drop(repo);
    fs::write(dir.path().join("mode"), "success").unwrap();
    let repo = DownloadRepository::open(&db).unwrap();
    assert_eq!(repo.job("writer-failure").unwrap().state, JobState::Failed);
    let recovered = repo.resume("writer-failure", &tool, &cancel).unwrap();
    assert_eq!(recovered.job.id, failed.id);
    assert_eq!(recovered.job.state, JobState::Downloaded);
    assert_eq!(recovered.job.attempts.len(), 2);
    assert_ne!(recovered.job.attempts[0].id, recovered.job.attempts[1].id);
    assert_eq!(fs::read(&partial).unwrap(), vec![b'p'; 65536]);
    assert_eq!(
        fs::read(&recovered.job.artifacts[0].path).unwrap(),
        b"synthetic recovered"
    );
    assert_eq!(
        fs::read(&unrelated).unwrap(),
        b"preserve independent owned fixture"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("extractions"))
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec![options.url.as_str(); 2]
    );
    drop(repo);
    let repo = DownloadRepository::open(&db).unwrap();
    assert_eq!(repo.recover().unwrap(), 0);
    assert_eq!(
        repo.job("writer-failure").unwrap().state,
        JobState::Downloaded
    );
}

#[test]
fn kernel_file_size_limit_efbig_preserves_partial_and_resumes() {
    writer_failure_then_resume("efbig", libc::EFBIG);
}

#[test]
fn injected_enospc_writer_error_preserves_partial_and_resumes() {
    writer_failure_then_resume("injected-enospc", libc::ENOSPC);
}
