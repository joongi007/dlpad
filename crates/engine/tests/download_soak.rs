//! Synthetic adapter+SQLite longevity. No actual network, media, or GUI.
#![cfg(target_os = "linux")]
use dlpad_core::{
    jobs::JobState,
    options::{DownloadOptions, Quality, SourceUrl},
};
use dlpad_engine::{
    download::{DownloadError, DownloadRepository},
    metrics::ProgressFeed,
    process,
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const WORKER: &str = r#"#!/usr/bin/python3
import sys,json,pathlib,time,os
base=pathlib.Path(__file__).parent
cfg=json.loads((base/'control.json').read_text())
a=sys.argv[1:]
if '--version' in a: print('2026.10.01');sys.exit(0)
if '--dump-single-json' in a:
 with (base/'extractions.jsonl').open('a') as f: f.write(json.dumps({'source':a[-1],'generation':cfg['generation']})+'\n')
 print(json.dumps({'id':'synthetic','duration':cfg['seconds'],'url':'https://expired.invalid/generation/'+str(cfg['generation'])}));sys.exit(0)
root=pathlib.Path(a[a.index('--paths')+1]);root.mkdir(parents=True,exist_ok=True)
p=root/a[a.index('--output')+1].replace('%(ext)s','mp4')
partial=p.with_suffix('.part');partial.write_bytes(b'x'*65536)
start=time.monotonic();next_sample=0
while time.monotonic()-start<cfg['seconds']:
 elapsed=time.monotonic()-start
 print('DLPAD_PROGRESS:'+json.dumps({'speed':65536,'eta':max(0,cfg['seconds']-elapsed)}),flush=True)
 if elapsed>=next_sample:
  partial.write_bytes(b'x'*65536)
  st=pathlib.Path('/proc/self/stat').read_text().rsplit(')',1)[1].split()
  def observed(pid):
   st=pathlib.Path('/proc/'+str(pid)+'/stat').read_text().rsplit(')',1)[1].split()
   return {'pid':pid,'start_ticks':int(st[19]),'cpu_ticks':int(st[11])+int(st[12]),'rss_bytes':int(st[21])*os.sysconf('SC_PAGE_SIZE')}
  sample={'processes':{'owner':observed(cfg['owner_pid']),'guardian':observed(os.getppid()),'worker':observed(os.getpid())},'elapsed':elapsed,'pid':os.getpid(),'start_ticks':int(st[19]),'partial_bytes':partial.stat().st_size,'generation':cfg['generation']}
  sample_log=base/'samples.jsonl'
  if sample_log.exists() and sample_log.stat().st_size>1024*1024: raise RuntimeError('bounded sample limit')
  with sample_log.open('a') as f:f.write(json.dumps(sample)+'\n')
  tmp=base/'worker-status.tmp';tmp.write_text(json.dumps(sample));tmp.replace(base/'worker-status.json')
  next_sample+=60
 time.sleep(min(.05 if cfg['seconds']<5 else 1,cfg['seconds']))
if cfg['mode'] in ['disconnect','expired']:
 print('ERROR: '+('network disconnected' if cfg['mode']=='disconnect' else 'HTTP Error 403: URL expired'),file=sys.stderr,flush=True);sys.exit(7)
if cfg['mode']=='flood':
 for _ in range(4000): print('synthetic stderr '+('x'*100),file=sys.stderr)
p.write_bytes(b'synthetic-final-'+str(cfg['generation']).encode())
print('DLPAD_ARTIFACT:'+json.dumps(str(p)),flush=True)
"#;
struct Fixture {
    root: PathBuf,
    tool: PathBuf,
    options: DownloadOptions,
}
impl Fixture {
    fn new(root: &Path) -> Self {
        fs::create_dir_all(root).unwrap();
        let tool = root.join("synthetic-ytdlp");
        fs::write(&tool, WORKER).unwrap();
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).unwrap();
        let options = DownloadOptions {
            url: SourceUrl::parse("https://example.invalid/original-source").unwrap(),
            output_dir: root.join("media 한글"),
            quality: Quality::Best,
            time_range: None,
            advanced: vec![],
            ffmpeg: None,
            deno: None,
            final_name_style: None,
            subtitles: None,
        };
        Self {
            root: root.into(),
            tool,
            options,
        }
    }
    fn mode(&self, mode: &str, seconds: f64, generation: u32) {
        fs::write(
            self.root.join("control.json"),
            serde_json::to_vec(
                &serde_json::json!({"mode":mode,"seconds":seconds,"generation":generation,"owner_pid":std::process::id()}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    fn repo(&self) -> DownloadRepository {
        DownloadRepository::open(&self.root.join("jobs.sqlite")).unwrap()
    }
    fn partial(&self, id: &str) -> PathBuf {
        self.options
            .output_dir
            .join(".dlpad-partials")
            .join(id)
            .join(format!("{id}.part"))
    }
    fn assert_extractions(&self, count: usize) {
        let rows = fs::read_to_string(self.root.join("extractions.jsonl")).unwrap();
        assert_eq!(rows.lines().count(), count);
        for (index, line) in rows.lines().enumerate() {
            let row: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(row["source"], self.options.url.as_str());
            assert_eq!(row["generation"], index + 1);
        }
    }
}
fn short_paths(root: &Path) {
    let f = Fixture::new(root);
    let repo = f.repo();
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let feed = ProgressFeed::default();
    f.mode("pause", 30., 1);
    repo.create_job("paused", &f.options).unwrap();
    let path = f.partial("paused");
    let seen = path.clone();
    let c = cancel.clone();
    let p = pause.clone();
    let requester = std::thread::spawn(move || {
        let start = Instant::now();
        while !seen.is_file() && start.elapsed() < Duration::from_secs(5) {
            std::thread::sleep(Duration::from_millis(10));
        }
        let observed = seen.is_file();
        p.store(true, Ordering::Release);
        c.store(true, Ordering::Release);
        observed
    });
    let failed =
        repo.execute_with_control("paused", &f.options, &f.tool, &cancel, &pause, feed.clone());
    assert!(requester.join().unwrap());
    assert!(failed.is_err());
    assert_eq!(repo.job("paused").unwrap().state, JobState::Paused);
    assert_eq!(fs::metadata(&path).unwrap().len(), 65536);
    drop(repo);
    cancel.store(false, Ordering::Release);
    pause.store(false, Ordering::Release);
    f.mode("flood", 0.15, 2);
    let repo = f.repo();
    assert_eq!(repo.job("paused").unwrap().attempts.len(), 1);
    let done = repo
        .resume_with_control("paused", &f.tool, &cancel, &pause, feed.clone())
        .unwrap();
    assert_eq!(done.job.state, JobState::Downloaded);
    assert_eq!(done.job.attempts.len(), 2);
    assert!(done.execution.truncated);
    assert!(done.execution.logs.len() <= process::MAX_LOG_BYTES);
    assert!(feed.snapshot().is_some());
    assert_eq!(
        fs::read(&done.job.artifacts[0].path).unwrap(),
        b"synthetic-final-2"
    );
    assert_eq!(fs::metadata(&path).unwrap().len(), 65536);
    drop(repo);
    assert_eq!(f.repo().recover().unwrap(), 0);
    for (id, mode, generation) in [("disconnected", "disconnect", 3), ("expired", "expired", 5)] {
        f.mode(mode, 0.1, generation);
        let repo = f.repo();
        repo.create_job(id, &f.options).unwrap();
        let error = repo
            .execute(id, &f.options, &f.tool, &cancel)
            .err()
            .expect("synthetic disconnect/expiry must fail");
        assert!(matches!(
            error,
            DownloadError::Execution {
                stage: "download",
                ..
            }
        ));
        let before = repo.job(id).unwrap();
        assert_eq!(before.state, JobState::Failed);
        assert_eq!(before.attempts.len(), 1);
        assert!(before.artifacts.is_empty());
        assert_eq!(fs::metadata(f.partial(id)).unwrap().len(), 65536);
        drop(repo);
        f.mode("success", 0.1, generation + 1);
        let repo = f.repo();
        let done = repo.resume(id, &f.tool, &cancel).unwrap();
        assert_eq!(done.job.id, id);
        assert_eq!(done.job.attempts.len(), 2);
        assert_ne!(done.job.attempts[0].id, done.job.attempts[1].id);
        assert_eq!(done.job.state, JobState::Downloaded);
        assert_eq!(
            fs::read(&done.job.artifacts[0].path).unwrap(),
            format!("synthetic-final-{}", generation + 1).as_bytes()
        );
        drop(repo);
        assert_eq!(f.repo().job(id).unwrap().state, JobState::Downloaded);
    }
    f.assert_extractions(6);
}
#[test]
fn synthetic_pause_reopen_disconnect_expiry_reextract_publish() {
    let dir = tempfile::tempdir().unwrap();
    short_paths(dir.path());
}

#[test]
#[ignore = "explicit DLPAD_DOWNLOAD_SOAK_SECONDS=28800 and new /tmp DLPAD_DOWNLOAD_SOAK_OUTPUT required; engine+DB only"]
fn synthetic_eight_hour_download_repository() {
    let seconds: u64 = std::env::var("DLPAD_DOWNLOAD_SOAK_SECONDS")
        .expect("explicit seconds")
        .parse()
        .unwrap();
    assert!(
        seconds >= 28800,
        "eight-hour acceptance requires at least 28800 seconds"
    );
    let root =
        PathBuf::from(std::env::var_os("DLPAD_DOWNLOAD_SOAK_OUTPUT").expect("new owned output"));
    assert!(root.is_absolute());
    assert_eq!(root.parent(), Some(Path::new("/tmp")));
    assert!(!root.exists(), "never overwrite existing data");
    fs::create_dir(&root).unwrap();
    let start = Instant::now();
    short_paths(&root.join("preflight"));
    let f = Fixture::new(&root.join("long-job"));
    f.mode("success", seconds as f64, 1);
    let repo = f.repo();
    repo.create_job("long-download", &f.options).unwrap();
    let feed = ProgressFeed::default();
    fs::write(root.join("status.json"),serde_json::to_vec(&serde_json::json!({"state":"running","scope":"synthetic DownloadRepository+SQLite, no media/network/UI","planned_seconds":seconds,"owner_pid":std::process::id()})).unwrap()).unwrap();
    let result = repo
        .execute_with_progress(
            "long-download",
            &f.options,
            &f.tool,
            &Arc::new(AtomicBool::new(false)),
            feed.clone(),
        )
        .unwrap();
    assert_eq!(result.job.state, JobState::Downloaded);
    assert_eq!(result.job.attempts.len(), 1);
    assert!(start.elapsed() >= Duration::from_secs(seconds));
    assert!(result.execution.logs.len() <= process::MAX_LOG_BYTES);
    assert!(feed.snapshot().is_some());
    assert_eq!(
        fs::metadata(f.partial("long-download")).unwrap().len(),
        65536
    );
    drop(repo);
    let reopened = f.repo();
    assert_eq!(reopened.recover().unwrap(), 0);
    let job = reopened.job("long-download").unwrap();
    assert_eq!(job.state, JobState::Downloaded);
    assert_eq!(
        fs::read(&job.artifacts[0].path).unwrap(),
        b"synthetic-final-1"
    );
    f.assert_extractions(1);
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"state":"completed","scope":"synthetic DownloadRepository+SQLite, no media/network/UI","elapsed_seconds":start.elapsed().as_secs_f64(),"eight_hours_verified":true,"attempts":job.attempts.len(),"artifact_bytes":job.artifacts[0].bytes,"partial_bytes":65536})).unwrap()).unwrap();
}
