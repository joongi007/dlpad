//! Explicit manual/automatic tool preferences; worker-only bounded settings I/O.
use dlpad_engine::store::Store;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
};
const KEY: &str = "manual_tool_preferences";
const MAX_PATH: usize = 32768;
const MAX_JSON: usize = 128 * 1024;
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "mode",
    content = "path",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub(crate) enum Choice {
    #[default]
    Automatic,
    Manual(String),
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Preferences {
    pub ytdlp: Choice,
    pub ffmpeg: Choice,
    pub deno: Choice,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Invalid,
    Storage,
    Stopped,
}
impl Preferences {
    pub(crate) fn from_drafts(manual: [bool; 3], drafts: [&str; 3]) -> Result<Self, Error> {
        let make = |index: usize| {
            if manual[index] {
                Choice::Manual(drafts[index].to_owned())
            } else {
                Choice::Automatic
            }
        };
        let value = Self {
            ytdlp: make(0),
            ffmpeg: make(1),
            deno: make(2),
        };
        value.encode()?;
        Ok(value)
    }
    fn encode(&self) -> Result<String, Error> {
        for choice in [&self.ytdlp, &self.ffmpeg, &self.deno] {
            if let Choice::Manual(value) = choice {
                let path = Path::new(value);
                if value.is_empty()
                    || value.len() > MAX_PATH
                    || value.chars().any(char::is_control)
                    || !path.is_absolute()
                    || path.components().any(|c| matches!(c, Component::ParentDir))
                {
                    return Err(Error::Invalid);
                }
            }
        }
        let raw = serde_json::to_string(self).map_err(|_| Error::Invalid)?;
        if raw.len() > MAX_JSON {
            return Err(Error::Invalid);
        }
        Ok(raw)
    }
    fn decode(raw: &str) -> Result<Self, Error> {
        if raw.len() > MAX_JSON {
            return Err(Error::Invalid);
        }
        let value: Self = serde_json::from_str(raw).map_err(|_| Error::Invalid)?;
        value.encode()?;
        Ok(value)
    }
}
fn checked(path: &Path, directory: bool) -> Result<(), Error> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Error::Storage);
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        // A Windows drive/verbatim prefix alone is not an openable pathname.
        // Check the actual root and every subsequent component instead.
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(Error::Storage);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(Error::Storage);
                    }
                }
                if current == path {
                    if directory && !meta.is_dir() || !directory && !meta.is_file() {
                        return Err(Error::Storage);
                    }
                } else if !meta.is_dir() {
                    return Err(Error::Storage);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(Error::Storage),
        }
    }
    Ok(())
}
fn database(data: &Path) -> Result<PathBuf, Error> {
    checked(data, true)?;
    let path = data.join("theme.sqlite");
    checked(&path, false)?;
    for suffix in ["-wal", "-shm", "-journal"] {
        checked(&data.join(format!("theme.sqlite{suffix}")), false)?;
    }
    Ok(path)
}
fn load(data: &Path) -> Result<Preferences, Error> {
    let db = database(data)?;
    if !db.try_exists().map_err(|_| Error::Storage)? {
        return Ok(Preferences::default());
    }
    Store::read_setting_readonly(&db, KEY, MAX_JSON)
        .map_err(|_| Error::Storage)?
        .map_or(Ok(Preferences::default()), |raw| Preferences::decode(&raw))
}
/// Worker-only bounded preference read; never call from a UI render or setter.
/// An unreadable/invalid setting refuses inspection rather than silently choosing Auto.
pub(crate) fn automatic_modes_for_managed_restore(data: &Path) -> Result<[bool; 3], Error> {
    let value = load(data)?;
    Ok(std::array::from_fn(|index| {
        matches!(choices(&value)[index], Choice::Automatic)
    }))
}
fn save(data: &Path, value: &Preferences) -> Result<(), Error> {
    #[cfg(test)]
    {
        save_impl(data, value, None)
    }
    #[cfg(not(test))]
    {
        save_impl(data, value)
    }
}
#[cfg(test)]
type WriteCheckpoint = dyn Fn() + Send + Sync;
fn save_impl(
    data: &Path,
    value: &Preferences,
    #[cfg(test)] checkpoint: Option<&WriteCheckpoint>,
) -> Result<(), Error> {
    let raw = value.encode()?;
    // Preserve damaged/oversized existing setting; explicit recovery is separate.
    load(data)?;
    let db = database(data)?;
    fs::create_dir_all(data).map_err(|_| Error::Storage)?;
    let store = Store::open(&db).map_err(|_| Error::Storage)?;
    // Per-worker test-only boundary AFTER Store initialization, BEFORE upsert.
    // No global hooks, sleeps or changed database behavior in production builds.
    #[cfg(test)]
    if let Some(checkpoint) = checkpoint {
        checkpoint();
    }
    store.set_setting(KEY, &raw).map_err(|_| Error::Storage)
}
enum Command {
    Save(u64, Preferences),
    Stop,
}
pub(crate) enum Completion {
    Loaded(Result<Preferences, Error>),
    Saved {
        revision: u64,
        result: Result<(), Error>,
    },
}
pub(crate) struct Worker {
    tx: SyncSender<Command>,
    rx: Receiver<Completion>,
    stop: Arc<AtomicBool>,
    ack: Arc<AtomicBool>,
    revision: u64,
    active: Option<u64>,
    latest: Option<(u64, Preferences)>,
    loaded: bool,
}
impl Worker {
    pub(crate) fn new(data: PathBuf, wake: Arc<dyn Fn() + Send + Sync>) -> std::io::Result<Self> {
        #[cfg(test)]
        {
            Self::new_impl(data, wake, None)
        }
        #[cfg(not(test))]
        {
            Self::new_impl(data, wake)
        }
    }
    fn new_impl(
        data: PathBuf,
        wake: Arc<dyn Fn() + Send + Sync>,
        #[cfg(test)] checkpoint: Option<Arc<WriteCheckpoint>>,
    ) -> std::io::Result<Self> {
        let (tx, jobs) = mpsc::sync_channel(1);
        let (done, rx) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let ack = Arc::new(AtomicBool::new(false));
        let quit = stop.clone();
        let finished = ack.clone();
        thread::Builder::new()
            .name("dlpad-tool-preferences".into())
            .spawn(move || {
                struct Ack(Arc<AtomicBool>, Arc<dyn Fn() + Send + Sync>);
                impl Drop for Ack {
                    fn drop(&mut self) {
                        self.0.store(true, Ordering::Release);
                        let _ =
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (self.1)()));
                    }
                }
                let _ack = Ack(finished, wake.clone());
                let notify = || {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wake()));
                };
                let send = |mut value: Completion| loop {
                    if quit.load(Ordering::Acquire) {
                        return false;
                    }
                    match done.try_send(value) {
                        Ok(()) => {
                            notify();
                            return true;
                        }
                        Err(TrySendError::Disconnected(_)) => return false,
                        Err(TrySendError::Full(returned)) => {
                            value = returned;
                            thread::sleep(std::time::Duration::from_millis(20));
                        }
                    }
                };
                let initial =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| load(&data)))
                        .unwrap_or(Err(Error::Storage));
                if !send(Completion::Loaded(initial)) {
                    return;
                }
                while let Ok(command) = jobs.recv() {
                    if quit.load(Ordering::Acquire) {
                        return;
                    }
                    let Command::Save(revision, value) = command else {
                        return;
                    };
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        #[cfg(test)]
                        {
                            save_impl(&data, &value, checkpoint.as_deref())
                        }
                        #[cfg(not(test))]
                        {
                            save(&data, &value)
                        }
                    }))
                    .unwrap_or(Err(Error::Storage));
                    if !send(Completion::Saved { revision, result }) {
                        return;
                    }
                }
            })?;
        Ok(Self {
            tx,
            rx,
            stop,
            ack,
            revision: 0,
            active: None,
            latest: None,
            loaded: false,
        })
    }
    pub(crate) fn request(&mut self, value: Preferences) -> Result<u64, Error> {
        value.encode()?;
        if self.stop.load(Ordering::Acquire) || self.ack.load(Ordering::Acquire) {
            return Err(Error::Stopped);
        }
        self.revision = self.revision.checked_add(1).ok_or(Error::Invalid)?;
        self.latest = Some((self.revision, value));
        self.pump();
        Ok(self.revision)
    }
    fn pump(&mut self) {
        if !self.loaded || self.active.is_some() || self.stop.load(Ordering::Acquire) {
            return;
        }
        if let Some((revision, value)) = self.latest.take() {
            match self.tx.try_send(Command::Save(revision, value)) {
                Ok(()) => self.active = Some(revision),
                Err(TrySendError::Full(Command::Save(revision, value))) => {
                    self.latest = Some((revision, value))
                }
                Err(_) => self.stop.store(true, Ordering::Release),
            }
        }
    }
    pub(crate) fn poll(&mut self) -> Option<Completion> {
        let result = self.rx.try_recv().ok();
        if let Some(ref value) = result {
            match value {
                Completion::Loaded(_) => self.loaded = true,
                Completion::Saved { revision, .. } => {
                    if self.active == Some(*revision) {
                        self.active = None;
                    }
                }
            }
        }
        self.pump();
        result
    }
    pub(crate) fn busy(&self) -> bool {
        !self.loaded || self.active.is_some() || self.latest.is_some()
    }
    pub(crate) fn request_stop(&mut self) {
        self.latest = None;
        self.stop.store(true, Ordering::Release);
        // Drain result first: writer never remains blocked on bounded result send.
        let _ = self.rx.try_recv();
        let _ = self.tx.try_send(Command::Stop);
    }
    pub(crate) fn stopped(&self) -> bool {
        self.ack.load(Ordering::Acquire)
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.request_stop();
    }
}

/// Caller gates on startup load completion and supplies revision provenance.
/// Empty explicit environment value is still an override, not absence.
pub(crate) fn startup_value<'a>(
    current: Option<&'a str>,
    environment: Option<&'a str>,
    saved: &'a Choice,
    managed: Option<&'a str>,
) -> &'a str {
    if let Some(value) = current {
        return value;
    }
    if let Some(value) = environment {
        return value;
    }
    if let Choice::Manual(value) = saved {
        return value;
    }
    managed.unwrap_or("")
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "dlpad-tool-pref-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            if std::thread::panicking() {
                eprintln!(
                    "preserving owned failed preference fixture {}",
                    self.0.display()
                );
                return;
            }
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn worker_restore_mask_obeys_saved_manual_and_refuses_corrupted_preferences() {
        let fixture = Fixture::new();
        assert_eq!(
            automatic_modes_for_managed_restore(&fixture.0),
            Ok([true; 3])
        );
        assert!(
            !fixture.0.join("theme.sqlite").exists(),
            "readonly missing settings do not create DB"
        );
        let value = Preferences {
            ytdlp: Choice::Manual(fixture.0.join("custom.exe").to_str().unwrap().into()),
            ffmpeg: Choice::Automatic,
            deno: Choice::Manual(fixture.0.join("deno.exe").to_str().unwrap().into()),
        };
        save(&fixture.0, &value).unwrap();
        assert_eq!(
            automatic_modes_for_managed_restore(&fixture.0),
            Ok([false, true, false])
        );
        let store = Store::open(&fixture.0.join("theme.sqlite")).unwrap();
        store.set_setting(KEY, "not-json").unwrap();
        drop(store);
        assert_eq!(
            automatic_modes_for_managed_restore(&fixture.0),
            Err(Error::Invalid)
        );
        assert_eq!(
            Store::read_setting_readonly(&fixture.0.join("theme.sqlite"), KEY, MAX_JSON)
                .unwrap()
                .as_deref(),
            Some("not-json")
        );
    }
    #[cfg(windows)]
    #[test]
    fn canonical_windows_data_prefix_allows_worker_load_save_and_restart() {
        let fixture = Fixture::new();
        let root = fs::canonicalize(&fixture.0).unwrap();
        assert!(
            matches!(root.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), std::path::Prefix::VerbatimDisk(_)))
        );
        let original = choice(&root);
        let mut worker = Worker::new(root.clone(), Arc::new(|| {})).unwrap();
        assert!(
            matches!(wait(&mut worker), Completion::Loaded(Ok(value)) if value == Preferences::default())
        );
        let revision = worker.request(original.clone()).unwrap();
        assert!(
            matches!(wait(&mut worker), Completion::Saved { revision: actual, result: Ok(()) } if actual == revision)
        );
        close(&mut worker);
        let mut reopened = Worker::new(root.clone(), Arc::new(|| {})).unwrap();
        assert!(matches!(wait(&mut reopened), Completion::Loaded(Ok(value)) if value == original));
        close(&mut reopened);
        assert!(
            !root.join("한글 tool.exe").exists(),
            "saving must not create or execute an external tool"
        );
    }
    fn choice(root: &Path) -> Preferences {
        Preferences {
            ytdlp: Choice::Manual(root.join("한글 tool.exe").to_str().unwrap().into()),
            ffmpeg: Choice::Automatic,
            deno: Choice::Automatic,
        }
    }
    fn wait(worker: &mut Worker) -> Completion {
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(value) = worker.poll() {
                return value;
            }
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn close(worker: &mut Worker) {
        worker.request_stop();
        let until = Instant::now() + Duration::from_secs(5);
        while !worker.stopped() {
            worker.poll();
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(10));
        }
    }
    #[test]
    fn manual_unicode_and_auto_roundtrip_invalid_draft_preserves_original_row() {
        let root = Fixture::new();
        let original = choice(&root.0);
        save(&root.0, &original).unwrap();
        let db = database(&root.0).unwrap();
        let before = Store::read_setting_readonly(&db, KEY, MAX_JSON)
            .unwrap()
            .unwrap();
        for bad in [
            "relative.exe".to_owned(),
            root.0.join("../escape.exe").to_str().unwrap().to_owned(),
            format!("{}\n", root.0.display()),
            "x".repeat(MAX_PATH + 1),
        ] {
            let mut next = original.clone();
            next.ytdlp = Choice::Manual(bad);
            assert!(save(&root.0, &next).is_err());
            assert_eq!(
                Store::read_setting_readonly(&db, KEY, MAX_JSON)
                    .unwrap()
                    .unwrap(),
                before
            );
        }
        assert_eq!(load(&root.0).unwrap(), original);
        save(&root.0, &Preferences::default()).unwrap();
        assert_eq!(load(&root.0).unwrap(), Preferences::default());
    }
    #[test]
    fn malformed_or_oversize_setting_cannot_be_silently_repaired() {
        let root = Fixture::new();
        let db = database(&root.0).unwrap();
        for invalid in [r#"{"unknown":true}"#.to_owned(), "x".repeat(MAX_JSON + 1)] {
            Store::open(&db)
                .unwrap()
                .set_setting(KEY, &invalid)
                .unwrap();
            assert!(load(&root.0).is_err());
            assert!(save(&root.0, &Preferences::default()).is_err());
            assert_eq!(
                Store::read_setting_readonly(&db, KEY, MAX_JSON + 2)
                    .unwrap()
                    .unwrap(),
                invalid
            );
        }
    }
    #[test]
    fn asynchronous_save_actual_restart_and_bounded_stop_ack() {
        let root = Fixture::new();
        let mut worker = Worker::new(root.0.clone(), Arc::new(|| {})).unwrap();
        assert!(matches!(wait(&mut worker), Completion::Loaded(Ok(_))));
        let prefs = choice(&root.0);
        let revision = worker.request(prefs.clone()).unwrap();
        assert!(
            matches!(wait(&mut worker),Completion::Saved{revision:r,result:Ok(())}if r==revision)
        );
        close(&mut worker);
        assert_eq!(worker.request(Preferences::default()), Err(Error::Stopped));
        let mut reopened = Worker::new(root.0.clone(), Arc::new(|| {})).unwrap();
        assert!(matches!(wait(&mut reopened),Completion::Loaded(Ok(value))if value==prefs));
        close(&mut reopened);
        assert!(
            !root.0.join("한글 tool.exe").exists(),
            "remembering choice never creates/installs external file"
        );
    }
    #[test]
    fn startup_queued_saves_coalesce_and_stop_before_load_ack_is_bounded() {
        let root = Fixture::new();
        let mut worker = Worker::new(root.0.clone(), Arc::new(|| {})).unwrap();
        let first = choice(&root.0);
        worker.request(first.clone()).unwrap();
        worker.request(Preferences::default()).unwrap();
        let mut last = first;
        last.ytdlp = Choice::Manual(root.0.join("last.exe").to_str().unwrap().into());
        let revision = worker.request(last.clone()).unwrap();
        assert!(matches!(wait(&mut worker), Completion::Loaded(Ok(_))));
        assert!(
            matches!(wait(&mut worker), Completion::Saved {revision: r,result:Ok(())} if r == revision)
        );
        assert_eq!(load(&root.0).unwrap(), last);
        close(&mut worker);
        let mut second = Worker::new(root.0.clone(), Arc::new(|| {})).unwrap();
        close(&mut second); // no poll/load ACK requirement before cancellation
        assert_eq!(load(&root.0).unwrap(), last);
    }
    #[test]
    fn blocked_database_namespace_preserves_original_good_database() {
        let root = Fixture::new();
        let original = choice(&root.0);
        save(&root.0, &original).unwrap();
        let db = database(&root.0).unwrap();
        let held = root.0.join("original-good.sqlite");
        fs::rename(&db, &held).unwrap(); // fixture-owned DB, all Store connections closed
        let before = fs::read(&held).unwrap();
        fs::create_dir(&db).unwrap();
        assert_eq!(save(&root.0, &Preferences::default()), Err(Error::Storage));
        assert_eq!(fs::read(&held).unwrap(), before);
        assert_eq!(
            Preferences::decode(
                &Store::read_setting_readonly(&held, KEY, MAX_JSON)
                    .unwrap()
                    .unwrap()
            )
            .unwrap(),
            original
        );
    }

    struct StopAckGuard(Worker);
    impl std::ops::Deref for StopAckGuard {
        type Target = Worker;
        fn deref(&self) -> &Worker {
            &self.0
        }
    }
    impl std::ops::DerefMut for StopAckGuard {
        fn deref_mut(&mut self) -> &mut Worker {
            &mut self.0
        }
    }
    impl Drop for StopAckGuard {
        fn drop(&mut self) {
            self.0.request_stop();
            let until = Instant::now() + Duration::from_secs(15);
            while !self.0.stopped() && Instant::now() < until {
                self.0.poll();
                thread::sleep(Duration::from_millis(10));
            }
            if !self.0.stopped() {
                if std::thread::panicking() {
                    eprintln!("preference worker stop ACK missing; fixture preserved");
                } else {
                    panic!("preference worker stop ACK missing");
                }
            }
        }
    }
    // App has no direct rusqlite dependency. Test-only C ABI uses the already
    // linked engine's bundled SQLite, adding no dependency/schema/public API.
    // Root must confirm transitive native symbol linkage when compiling tests.
    unsafe extern "C" {
        fn sqlite3_open_v2(
            name: *const std::ffi::c_char,
            db: *mut *mut std::ffi::c_void,
            flags: i32,
            vfs: *const std::ffi::c_char,
        ) -> i32;
        fn sqlite3_exec(
            db: *mut std::ffi::c_void,
            sql: *const std::ffi::c_char,
            callback: Option<
                unsafe extern "C" fn(
                    *mut std::ffi::c_void,
                    i32,
                    *mut *mut std::ffi::c_char,
                    *mut *mut std::ffi::c_char,
                ) -> i32,
            >,
            context: *mut std::ffi::c_void,
            error: *mut *mut std::ffi::c_char,
        ) -> i32;
        fn sqlite3_close(db: *mut std::ffi::c_void) -> i32;
    }
    struct WriterLock(*mut std::ffi::c_void);
    impl WriterLock {
        fn open(path: &Path) -> Self {
            let name = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
            let mut db = std::ptr::null_mut();
            // READWRITE | FULLMUTEX | NOFOLLOW, never create another database.
            let code = unsafe {
                sqlite3_open_v2(
                    name.as_ptr(),
                    &mut db,
                    2 | 0x10000 | 0x1000000,
                    std::ptr::null(),
                )
            };
            if code != 0 {
                if !db.is_null() {
                    unsafe { sqlite3_close(db) };
                }
                panic!("fixture SQLite open {code}");
            }
            Self(db)
        }
        fn sql(&self, sql: &str) {
            let sql = std::ffi::CString::new(sql).unwrap();
            assert_eq!(
                unsafe {
                    sqlite3_exec(
                        self.0,
                        sql.as_ptr(),
                        None,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                },
                0,
                "fixture actual SQLite SQL"
            );
        }
        fn begin(&self) {
            self.sql("BEGIN IMMEDIATE");
        }
        fn rollback(&self) {
            self.sql("ROLLBACK");
        }
    }
    impl Drop for WriterLock {
        fn drop(&mut self) {
            let code = unsafe { sqlite3_close(self.0) };
            assert_eq!(code, 0, "fixture original connection closed");
        }
    }
    fn original_row(data: &Path) -> String {
        Store::read_setting_readonly(&database(data).unwrap(), KEY, MAX_JSON)
            .unwrap()
            .unwrap()
    }
    #[test]
    fn actual_sqlite_writer_lock_failure_preserves_original_row_and_recovers() {
        let root = Fixture::new();
        let original = choice(&root.0);
        save(&root.0, &original).unwrap();
        let row = original_row(&root.0);
        let db = database(&root.0).unwrap();
        let before = fs::read(&db).unwrap();
        let lock = WriterLock::open(&db);
        lock.begin();
        // BEGIN IMMEDIATE retains a genuine independent SQLite RESERVED writer
        // lock while permitting the production readonly preflight to succeed.
        assert_eq!(load(&root.0).unwrap(), original);
        let began = Instant::now();
        assert_eq!(save(&root.0, &Preferences::default()), Err(Error::Storage));
        assert!(began.elapsed() < Duration::from_secs(15));
        assert_eq!(original_row(&root.0), row);
        assert_eq!(fs::read(&db).unwrap(), before);
        lock.rollback();
        drop(lock);
        save(&root.0, &Preferences::default()).unwrap();
        assert_eq!(load(&root.0).unwrap(), Preferences::default());
    }
    #[test]
    fn stop_during_actual_active_locked_upsert_discards_queued_save_and_waits_for_ack() {
        let root = Fixture::new();
        let original = choice(&root.0);
        save(&root.0, &original).unwrap();
        let row = original_row(&root.0);
        let lock = WriterLock::open(&database(&root.0).unwrap());
        let (reached, ready) = mpsc::sync_channel(1);
        let (resume, permission) = mpsc::sync_channel(1);
        let permission = std::sync::Mutex::new(permission);
        let checkpoint: Arc<WriteCheckpoint> = Arc::new(move || {
            reached.send(()).unwrap();
            permission
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
        });
        let mut worker = StopAckGuard(
            Worker::new_impl(root.0.clone(), Arc::new(|| {}), Some(checkpoint)).unwrap(),
        );
        assert!(matches!(wait(&mut worker),Completion::Loaded(Ok(value))if value==original));
        let revision = worker.request(Preferences::default()).unwrap();
        ready.recv_timeout(Duration::from_secs(5)).unwrap(); // Store already open
        lock.begin();
        resume.send(()).unwrap(); // next actual upsert meets lock
        // Store's actual 5-second busy handler must retain this active operation.
        // An early ACK fails, catching a race where stop skipped the operation.
        let active_until = Instant::now() + Duration::from_millis(100);
        while Instant::now() < active_until {
            assert!(!worker.stopped());
            assert!(worker.poll().is_none());
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(worker.active, Some(revision));
        let mut unsent = original.clone();
        unsent.ytdlp = Choice::Manual(root.0.join("must-never-save.exe").to_str().unwrap().into());
        worker.request(unsent).unwrap();
        assert!(worker.latest.is_some());
        worker.request_stop();
        assert!(worker.latest.is_none());
        assert_eq!(worker.request(original.clone()), Err(Error::Stopped));
        let pending_until = Instant::now() + Duration::from_millis(100);
        while Instant::now() < pending_until {
            assert!(
                !worker.stopped(),
                "stop ACK before active SQLite operation ended"
            );
            assert!(
                worker.poll().is_none(),
                "no misleading Saved result after stop"
            );
            thread::sleep(Duration::from_millis(5));
        }
        // Keep lock until Store reports BUSY; cancellation does not forcibly
        // abort an active Store write and does not imply Saved.
        let deadline = Instant::now() + Duration::from_secs(15);
        while !worker.stopped() {
            assert!(worker.poll().is_none());
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(worker.poll().is_none());
        assert_eq!(original_row(&root.0), row);
        lock.rollback();
        drop(lock);
        drop(worker);
        let mut reopened = Worker::new(root.0.clone(), Arc::new(|| {})).unwrap();
        assert!(matches!(wait(&mut reopened),Completion::Loaded(Ok(value))if value==original));
        close(&mut reopened);
        assert!(!root.0.join("must-never-save.exe").exists());
    }

    #[test]
    fn explicit_session_then_environment_then_saved_then_managed_then_path() {
        let saved = Choice::Manual("saved".into());
        assert_eq!(
            startup_value(Some("typed"), Some("env"), &saved, Some("managed")),
            "typed"
        );
        assert_eq!(
            startup_value(None, Some("env"), &saved, Some("managed")),
            "env"
        );
        assert_eq!(startup_value(None, Some(""), &saved, Some("managed")), "");
        assert_eq!(startup_value(None, None, &saved, Some("managed")), "saved");
        assert_eq!(
            startup_value(None, None, &Choice::Automatic, Some("managed")),
            "managed"
        );
        assert_eq!(startup_value(None, None, &Choice::Automatic, None), "");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Loading,
    Ready,
    Dirty,
    Saving,
    Saved,
    OlderSaved,
    Invalid,
    Failed,
    LoadFailed,
    SessionOnly,
}
struct SaveTicket {
    revision: u64,
    snapshot: Preferences,
    draft_revision: u64,
    path_revisions: [u64; 3],
    mode_revisions: [u64; 3],
    reset_auto: [bool; 3],
}
/// The caller supplies current worker-verified compatible installation eligibility.
/// Path syntax is checked defensively, never treated as installation verification.
/// Opaque current-session slot ticket; constructed only from controller provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ManagedAutoTicket {
    path_revision: u64,
    mode_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedAvailability {
    Checking,
    Available(String),
    Unavailable(&'static str),
}
/// Owns persistence/provenance, never filesystem or SQLite I/O on the UI thread.
pub(crate) struct Controller {
    data: Option<PathBuf>,
    environment: [Option<String>; 3],
    managed: [Option<String>; 3],
    managed_availability: [ManagedAvailability; 3],
    worker: Option<Worker>,
    started: bool,
    no_worker_ack: bool,
    stopping: bool,
    loaded: bool,
    last_saved: Option<Preferences>,
    manual: [bool; 3],
    path_revisions: [u64; 3],
    mode_revisions: [u64; 3],
    draft_revision: u64,
    pending_auto: [bool; 3],
    pending_selected_revision: u64,
    inflight: Option<SaveTicket>,
    queued: Option<SaveTicket>,
    latest_requested: Option<u64>,
    status: Status,
}
fn choices(value: &Preferences) -> [&Choice; 3] {
    [&value.ytdlp, &value.ffmpeg, &value.deno]
}
fn set_auto(value: &mut Preferences, index: usize) {
    match index {
        0 => value.ytdlp = Choice::Automatic,
        1 => value.ffmpeg = Choice::Automatic,
        2 => value.deno = Choice::Automatic,
        _ => (),
    }
}
impl Controller {
    /// Read-only display provenance; the worker must still validate every action.
    /// Last UI observation only; never authorizes install, selection, or execution.
    pub(crate) fn recovery_availability(&self, index: usize) -> Option<&ManagedAvailability> {
        self.managed_availability.get(index)
    }
    pub(crate) fn displayed_managed_path_matches(&self, index: usize, path: &str) -> bool {
        index < 3
            && !path.is_empty()
            && !self.manual[index]
            && self.environment[index].is_none()
            && matches!(&self.managed_availability[index], ManagedAvailability::Available(known) if known == path)
    }
    pub(crate) fn new(data: PathBuf, environment: [Option<String>; 3]) -> Self {
        Self {
            data: Some(data),
            environment,
            managed: std::array::from_fn(|_| None),
            managed_availability: std::array::from_fn(|_| ManagedAvailability::Checking),
            worker: None,
            started: false,
            no_worker_ack: false,
            stopping: false,
            loaded: false,
            last_saved: None,
            manual: [false; 3],
            path_revisions: [0; 3],
            mode_revisions: [0; 3],
            draft_revision: 0,
            pending_auto: [false; 3],
            pending_selected_revision: 0,
            inflight: None,
            queued: None,
            latest_requested: None,
            status: Status::Loading,
        }
    }
    fn changed(&mut self, index: usize, mode: bool) {
        if index >= 3 || self.stopping {
            return;
        }
        let target = if mode {
            &mut self.mode_revisions[index]
        } else {
            &mut self.path_revisions[index]
        };
        *target = target.saturating_add(1);
        self.draft_revision = self.draft_revision.saturating_add(1);
        self.status = if self.loaded && self.last_saved.is_none() {
            Status::SessionOnly
        } else {
            Status::Dirty
        };
    }
    /// Call for every explicit path edit, including typed empty values.
    pub(crate) fn edited(&mut self, index: usize) {
        if index >= 3 || self.stopping {
            return;
        }
        self.manual[index] = true;
        self.pending_auto[index] = false;
        self.changed(index, true);
        self.changed(index, false);
    }
    fn accept_loaded(&mut self, result: Result<Preferences, Error>, fields: [&mut String; 3]) {
        self.loaded = true;
        match result {
            Ok(value) => {
                for (index, field) in fields.into_iter().enumerate() {
                    let saved = choices(&value)[index];
                    if self.mode_revisions[index] == 0 && !self.pending_auto[index] {
                        self.manual[index] = matches!(saved, Choice::Manual(_));
                    }
                    if self.path_revisions[index] == 0
                        && self.mode_revisions[index] == 0
                        && !self.pending_auto[index]
                    {
                        *field = startup_value(
                            None,
                            self.environment[index].as_deref(),
                            saved,
                            self.managed[index].as_deref(),
                        )
                        .to_owned();
                    }
                }
                self.last_saved = Some(value);
                self.status = if self.draft_revision == 0 {
                    Status::Ready
                } else {
                    Status::Dirty
                };
            }
            Err(_) => {
                self.pending_auto = [false; 3];
                self.last_saved = None;
                self.status = if self.draft_revision == 0 {
                    Status::LoadFailed
                } else {
                    Status::SessionOnly
                };
            }
        }
    }
    fn queue(
        &mut self,
        snapshot: Preferences,
        reset_auto: [bool; 3],
        expected_draft: u64,
    ) -> Result<(), Error> {
        if self.stopping {
            return Err(Error::Stopped);
        }
        if !self.loaded || self.last_saved.is_none() {
            return Err(Error::Storage);
        }
        if self.draft_revision == u64::MAX {
            return Err(Error::Invalid);
        }
        let worker = self.worker.as_mut().ok_or(Error::Storage)?;
        let revision = worker.request(snapshot.clone())?;
        let ticket = SaveTicket {
            revision,
            snapshot,
            draft_revision: expected_draft,
            path_revisions: self.path_revisions,
            mode_revisions: self.mode_revisions,
            reset_auto,
        };
        if worker.active == Some(revision) {
            self.inflight = Some(ticket);
        } else {
            self.queued = Some(ticket);
        }
        self.latest_requested = Some(revision);
        self.status = Status::Saving;
        Ok(())
    }
    fn promote(&mut self) {
        if let Some(active) = self.worker.as_ref().and_then(|worker| worker.active)
            && self
                .queued
                .as_ref()
                .is_some_and(|ticket| ticket.revision == active)
        {
            self.inflight = self.queued.take();
        }
    }
    fn accept_saved(&mut self, revision: u64, result: Result<(), Error>, fields: [&mut String; 3]) {
        let Some(ticket) = self.inflight.take() else {
            return;
        };
        if ticket.revision != revision {
            self.inflight = Some(ticket);
            return;
        }
        if result.is_ok() {
            let matches_draft = Preferences::from_drafts(
                self.manual,
                [fields[0].as_str(), fields[1].as_str(), fields[2].as_str()],
            )
            .is_ok_and(|value| value == ticket.snapshot);
            self.last_saved = Some(ticket.snapshot.clone());
            for (index, field) in fields.into_iter().enumerate() {
                if ticket.reset_auto[index]
                    && !self.manual[index]
                    && self.path_revisions[index] == ticket.path_revisions[index]
                    && self.mode_revisions[index] == ticket.mode_revisions[index]
                {
                    *field = self.managed[index].clone().unwrap_or_default();
                }
            }
            if self.latest_requested == Some(revision) {
                self.status = if self.draft_revision == ticket.draft_revision && matches_draft {
                    Status::Saved
                } else {
                    Status::OlderSaved
                };
            }
        } else if self.latest_requested == Some(revision) {
            self.status = Status::Failed;
        }
    }
    fn auto_snapshot(&self) -> Option<Preferences> {
        if !self.pending_auto.iter().any(|value| *value) {
            return None;
        }
        let mut value = self.last_saved.clone()?;
        for index in 0..3 {
            if self.pending_auto[index] {
                set_auto(&mut value, index);
            }
        }
        Some(value)
    }
    fn flush_auto(&mut self) {
        // Wait for prior accepted explicit saves to settle. Otherwise a managed
        // selection could revert the other two slots to an older saved snapshot.
        if self.stopping || !self.loaded || self.inflight.is_some() || self.queued.is_some() {
            return;
        }
        let Some(value) = self.auto_snapshot() else {
            return;
        };
        // Managed selection already applied its verified path immediately. Its
        // implicit preference save must not reset a later edited session field.
        match self.queue(value, [false; 3], self.pending_selected_revision) {
            Ok(()) => self.pending_auto = [false; 3],
            Err(_) => self.status = Status::SessionOnly,
        }
    }
    pub(crate) fn poll(&mut self, ctx: &egui::Context, fields: [&mut String; 3]) {
        let [first, second, third] = fields;
        if !self.started && !self.stopping {
            self.started = true;
            let context = ctx.clone();
            let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || context.request_repaint());
            match Worker::new(self.data.take().expect("unstarted preference data"), wake) {
                Ok(worker) => self.worker = Some(worker),
                Err(_) => {
                    self.no_worker_ack = true;
                    self.accept_loaded(
                        Err(Error::Storage),
                        [&mut *first, &mut *second, &mut *third],
                    );
                }
            }
        }
        // One bounded result queue and at most one current + one queued ticket.
        let event = self.worker.as_mut().and_then(Worker::poll);
        if self.stopping {
            self.inflight = None;
            self.queued = None;
            return;
        }
        if let Some(event) = event {
            match event {
                Completion::Loaded(value) => {
                    self.accept_loaded(value, [&mut *first, &mut *second, &mut *third])
                }
                Completion::Saved { revision, result } => {
                    self.accept_saved(revision, result, [&mut *first, &mut *second, &mut *third])
                }
            }
            self.promote();
        }
        if !self.stopping {
            self.flush_auto();
        }
    }
    pub(crate) fn managed_restored(&mut self, index: usize, path: &str, fields: [&mut String; 3]) {
        if index >= 3
            || self.stopping
            || path.len() > MAX_PATH
            || Choice::Manual(path.to_owned()).invalid_path()
        {
            return;
        }
        self.set_managed_availability(index, ManagedAvailability::Available(path.to_owned()));
        if self.loaded
            && self.path_revisions[index] == 0
            && self.mode_revisions[index] == 0
            && self.environment[index].is_none()
            && !self.manual[index]
            && self
                .last_saved
                .as_ref()
                .is_some_and(|value| matches!(choices(value)[index], Choice::Automatic))
        {
            let field = fields.into_iter().nth(index).expect("bounded tool slot");
            *field = path.to_owned();
        }
    }
    pub(crate) fn managed_selected(&mut self, index: usize, path: &str, fields: [&mut String; 3]) {
        if index >= 3
            || self.stopping
            || path.len() > MAX_PATH
            || Choice::Manual(path.to_owned()).invalid_path()
        {
            return;
        }
        self.set_managed_availability(index, ManagedAvailability::Available(path.to_owned()));
        *fields.into_iter().nth(index).expect("bounded tool slot") = path.to_owned();
        self.changed(index, false);
        self.changed(index, true);
        self.manual[index] = false;
        self.pending_auto[index] = true;
        self.pending_selected_revision = self.draft_revision;
        if self.loaded && self.last_saved.is_none() {
            self.pending_auto[index] = false;
            self.status = Status::SessionOnly;
        } else {
            self.flush_auto();
        }
    }
    pub(crate) fn managed_auto_ticket(&self, index: usize) -> Option<ManagedAutoTicket> {
        if index >= 3
            || self.stopping
            || !self.loaded
            || self.manual[index]
            || self.environment[index].is_some()
            || self.path_revisions[index] == u64::MAX
            || self.mode_revisions[index] == u64::MAX
            || !self
                .last_saved
                .as_ref()
                .is_some_and(|v| matches!(choices(v)[index], Choice::Automatic))
        {
            return None;
        }
        Some(ManagedAutoTicket {
            path_revision: self.path_revisions[index],
            mode_revision: self.mode_revisions[index],
        })
    }
    /// Only apply the exact dispatch provenance; a late manual edit wins.
    pub(crate) fn managed_auto_selected(
        &mut self,
        index: usize,
        path: &str,
        ticket: ManagedAutoTicket,
        fields: [&mut String; 3],
    ) {
        if index >= 3
            || self.stopping
            || path.len() > MAX_PATH
            || Choice::Manual(path.to_owned()).invalid_path()
        {
            return;
        }
        self.set_managed_availability(index, ManagedAvailability::Available(path.to_owned()));
        if self.managed_auto_ticket(index) == Some(ticket) {
            *fields.into_iter().nth(index).expect("bounded tool slot") = path.to_owned();
        }
    }
    /// Cache current worker eligibility only; do not replace a draft or persist settings.
    /// Root owns generation/CloseWait filtering before supplying an observation.
    pub(crate) fn set_managed_availability(&mut self, index: usize, value: ManagedAvailability) {
        if index >= 3 || self.stopping {
            return;
        }
        let value = match value {
            ManagedAvailability::Available(path)
                if path.len() > MAX_PATH || Choice::Manual(path.clone()).invalid_path() =>
            {
                ManagedAvailability::Unavailable(crate::catalog::TOOL_AUTO_UNAVAILABLE)
            }
            other => other,
        };
        self.managed[index] = match &value {
            ManagedAvailability::Available(path) => Some(path.clone()),
            _ => None,
        };
        self.managed_availability[index] = value;
    }
    #[cfg(test)]
    pub(crate) fn show_mode(&mut self, index: usize, ui: &mut egui::Ui, mutate: bool) {
        self.show_mode_inner(index, ui, mutate, None);
    }
    pub(crate) fn show_mode_with_field(
        &mut self,
        index: usize,
        ui: &mut egui::Ui,
        mutate: bool,
        field: &mut String,
    ) {
        self.show_mode_inner(index, ui, mutate, Some(field));
    }
    fn show_mode_inner(
        &mut self,
        index: usize,
        ui: &mut egui::Ui,
        mutate: bool,
        mut field: Option<&mut String>,
    ) {
        use crate::catalog as c;
        let Some(&(label, id)) = [
            (c::TOOL_YT_DLP, "tool-pref-mode-ytdlp"),
            (c::TOOL_FFMPEG, "tool-pref-mode-ffmpeg"),
            (c::TOOL_DENO, "tool-pref-mode-deno"),
        ]
        .get(index) else {
            return;
        };
        let availability = self.managed_availability[index].clone();
        let automatic_path = match &availability {
            ManagedAvailability::Available(path) => Some(path.as_str()),
            _ => None,
        };
        let reason = match &availability {
            ManagedAvailability::Checking => Some(c::TOOL_AUTO_CHECKING),
            ManagedAvailability::Unavailable(reason) => Some(*reason),
            ManagedAvailability::Available(_) => None,
        };
        ui.add_enabled_ui(mutate && !self.stopping, |ui| {
            let before = self.manual[index];
            let mut automatic_clicked = false;
            ui.horizontal_wrapped(|ui| {
                let label = ui
                    .allocate_ui_with_layout(
                        egui::vec2(112.0, crate::ui_tokens::CONTROL_HEIGHT),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_min_width(112.0);
                            ui.label(label)
                        },
                    )
                    .inner;
                let response = egui::ComboBox::from_id_salt(id)
                    .width(160.0)
                    .selected_text(if self.manual[index] {
                        c::TOOL_PREF_MANUAL
                    } else {
                        c::TOOL_PREF_AUTO
                    })
                    .show_ui(ui, |ui| {
                        for (manual, caption) in
                            [(false, c::TOOL_PREF_AUTO), (true, c::TOOL_PREF_MANUAL)]
                        {
                            let response = ui
                                .add_enabled_ui(manual || automatic_path.is_some(), |ui| {
                                    crate::ui_components::select(
                                        ui,
                                        &mut self.manual[index],
                                        manual,
                                        caption,
                                    )
                                })
                                .inner;
                            if response.clicked() {
                                automatic_clicked = !manual;
                                ui.close();
                            }
                        }
                    })
                    .response
                    .labelled_by(label.id);
                let response = crate::ui_components::combo_response(ui, response);
                reveal_control(&response);
            });
            if before != self.manual[index] {
                self.pending_auto[index] = false;
                self.changed(index, true);
            }
            if automatic_clicked
                && let Some(path) = automatic_path
                && let Some(field) = field.take()
                && field.as_str() != path
            {
                *field = path.to_owned();
                self.pending_auto[index] = false;
                self.changed(index, false);
            }
        });
        if let Some(reason) = reason {
            ui.weak(reason);
        }
    }
    pub(crate) fn show_save_status(
        &mut self,
        ui: &mut egui::Ui,
        mutate: bool,
        fields: [&mut String; 3],
    ) {
        use crate::catalog as c;
        ui.add_enabled_ui(mutate && !self.stopping, |ui| {
            let save = crate::ui_components::secondary(
                ui,
                c::TOOL_PREF_SAVE,
                self.loaded && self.last_saved.is_some(),
            );
            reveal_control(&save);
            if save.clicked() {
                let drafts: [&str; 3] =
                    [fields[0].as_str(), fields[1].as_str(), fields[2].as_str()];
                match Preferences::from_drafts(self.manual, drafts) {
                    Ok(value) => {
                        let mask = std::array::from_fn(|index| {
                            matches!(choices(&value)[index], Choice::Automatic)
                        });
                        match self.queue(value, mask, self.draft_revision) {
                            Ok(()) => self.pending_auto = [false; 3],
                            Err(Error::Invalid) => self.status = Status::Invalid,
                            Err(_) => self.status = Status::Failed,
                        }
                    }
                    Err(_) => self.status = Status::Invalid,
                }
            }
        });
        ui.weak(match self.status {
            Status::Loading => c::TOOL_PREF_LOADING,
            Status::Ready => c::TOOL_PREF_READY,
            Status::Dirty => c::TOOL_PREF_DIRTY,
            Status::Saving => c::TOOL_PREF_SAVING,
            Status::Saved => c::TOOL_PREF_SAVED,
            Status::OlderSaved => c::TOOL_PREF_OLDER_SAVED,
            Status::Invalid => c::TOOL_PREF_INVALID,
            Status::Failed => c::TOOL_PREF_FAILED,
            Status::LoadFailed => c::TOOL_PREF_LOAD_FAILED,
            Status::SessionOnly => c::TOOL_PREF_SESSION_ONLY,
        });
    }
    pub(crate) fn request_stop(&mut self) {
        if self.inflight.is_some()
            || self.queued.is_some()
            || self.pending_auto.iter().any(|value| *value)
        {
            self.status = Status::SessionOnly;
        }
        self.stopping = true;
        self.pending_auto = [false; 3];
        self.queued = None;
        if let Some(worker) = self.worker.as_mut() {
            worker.request_stop();
        } else {
            self.no_worker_ack = true;
        }
    }
    pub(crate) fn stopped(&self) -> bool {
        self.no_worker_ack || self.worker.as_ref().is_some_and(Worker::stopped)
    }
    pub(crate) fn busy(&self) -> bool {
        if self.stopping {
            return !self.stopped();
        }
        !self.started
            || self.pending_auto.iter().any(|value| *value)
            || self.worker.as_ref().is_some_and(Worker::busy)
    }
}
fn reveal_control(response: &egui::Response) {
    if !response.enabled() {
        return;
    }
    response.ctx.accesskit_node_builder(response.id, |node| {
        node.add_action(egui::accesskit::Action::ScrollIntoView);
    });
    if response.gained_focus()
        || response.ctx.input(|input| {
            input.has_accesskit_action_request(response.id, egui::accesskit::Action::ScrollIntoView)
        })
    {
        response.scroll_to_me(None);
    }
}
impl Choice {
    fn invalid_path(&self) -> bool {
        let Choice::Manual(path) = self else {
            return false;
        };
        Preferences {
            ytdlp: Choice::Manual(path.clone()),
            ffmpeg: Choice::Automatic,
            deno: Choice::Automatic,
        }
        .encode()
        .is_err()
    }
}

#[cfg(test)]
mod controller_tests {
    use super::*;
    #[test]
    fn actual_preference_mode_commit_and_reselection_close_popup() {
        use crate::catalog as c;
        use egui::accesskit::{Action, ActionRequest, Role};
        for (caption, manual) in [(c::TOOL_PREF_AUTO, false), (c::TOOL_PREF_MANUAL, true)] {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.enable_accesskit();
            let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
            controller.loaded = true;
            controller.last_saved = Some(Preferences::default());
            controller.set_managed_availability(
                0,
                ManagedAvailability::Available(path("popup-managed.exe")),
            );
            let mut time = 0.0;
            let mut frame = |controller: &mut Controller, events| {
                time += 0.05;
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(480.0, 320.0),
                        )),
                        focused: true,
                        time: Some(time),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .show(ctx, |ui| controller.show_mode(0, ui, true));
                    },
                )
            };
            let initial = frame(&mut controller, vec![]);
            let combo = initial
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::ComboBox)
                .unwrap()
                .0;
            let action = |target| {
                egui::Event::AccessKitActionRequest(ActionRequest {
                    target,
                    action: Action::Click,
                    data: None,
                })
            };
            frame(&mut controller, vec![action(combo)]);
            let open = frame(&mut controller, vec![]);
            assert!(egui::Popup::is_any_open(&ctx));
            let choice = open
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button
                        && node.label() == Some(caption)
                        && node.supports_action(Action::Click)
                })
                .unwrap()
                .0;
            frame(&mut controller, vec![action(choice)]);
            frame(&mut controller, vec![]);
            assert_eq!(controller.manual[0], manual);
            assert!(
                !egui::Popup::is_any_open(&ctx),
                "current-value reselection and mode change both close the actual menu"
            );
            assert!(
                controller.worker.is_none(),
                "UI mode choice does not start a persistence worker"
            );
        }
    }
    fn path(leaf: &str) -> String {
        std::env::temp_dir().join(leaf).to_str().unwrap().to_owned()
    }
    fn saved() -> Preferences {
        Preferences {
            ytdlp: Choice::Manual(path("saved-ytdlp.exe")),
            ffmpeg: Choice::Manual(path("saved-ffmpeg.exe")),
            deno: Choice::Automatic,
        }
    }
    #[test]
    fn late_load_respects_empty_environment_typed_empty_and_mode_revision() {
        let mut controller =
            Controller::new(std::env::temp_dir(), [Some(String::new()), None, None]);
        let mut first = String::new();
        let mut second = String::new();
        let mut third = path("session-deno.exe");
        controller.edited(1); // explicit typed empty must remain empty
        controller.manual[2] = true;
        controller.changed(2, true);
        controller.managed_restored(
            0,
            &path("managed-ytdlp.exe"),
            [&mut first, &mut second, &mut third],
        );
        controller.accept_loaded(Ok(saved()), [&mut first, &mut second, &mut third]);
        assert_eq!(first, "");
        assert_eq!(second, "");
        assert_eq!(third, path("session-deno.exe"));
        assert!(controller.manual[2]);
        assert!(
            controller.worker.is_none(),
            "load policy test never spawns I/O"
        );
    }
    #[test]
    fn editing_auto_path_selects_manual_and_empty_requires_explicit_auto() {
        let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
        let chosen = path("typed-ytdlp.exe");
        controller.edited(0);
        assert!(controller.manual[0]);
        assert_eq!(controller.mode_revisions[0], 1);
        assert_eq!(controller.path_revisions[0], 1);
        let snapshot = Preferences::from_drafts(controller.manual, [&chosen, "", ""]).unwrap();
        assert_eq!(snapshot.ytdlp, Choice::Manual(chosen));
        controller.edited(0); // deleting the entire path remains explicit Manual intent
        assert!(matches!(
            Preferences::from_drafts(controller.manual, ["", "", ""]),
            Err(Error::Invalid)
        ));
        assert_eq!(controller.mode_revisions[0], 2);
        controller.manual[0] = false; // same transition performed by the explicit Auto combo
        controller.changed(0, true);
        let snapshot = Preferences::from_drafts(controller.manual, ["", "", ""]).unwrap();
        assert_eq!(snapshot.ytdlp, Choice::Automatic);
    }
    #[test]
    fn startup_saved_manual_beats_cached_managed_and_late_restore() {
        let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
        let mut first = String::new();
        let mut second = String::new();
        let mut third = String::new();
        controller.managed_restored(
            0,
            &path("managed.exe"),
            [&mut first, &mut second, &mut third],
        );
        assert_eq!(first, "");
        controller.accept_loaded(Ok(saved()), [&mut first, &mut second, &mut third]);
        assert_eq!(first, path("saved-ytdlp.exe"));
        controller.managed_restored(
            0,
            &path("managed-new.exe"),
            [&mut first, &mut second, &mut third],
        );
        assert_eq!(first, path("saved-ytdlp.exe"));
        controller.managed_restored(
            2,
            &path("managed-deno.exe"),
            [&mut first, &mut second, &mut third],
        );
        assert_eq!(third, path("managed-deno.exe"));
        controller.edited(2);
        third.clear();
        controller.managed_restored(
            2,
            &path("later-deno.exe"),
            [&mut first, &mut second, &mut third],
        );
        assert_eq!(third, "");
    }
    #[test]
    fn selected_before_load_merges_auto_without_persisting_other_unsaved_drafts() {
        let mut controller =
            Controller::new(std::env::temp_dir(), [Some(path("env.exe")), None, None]);
        let mut first = path("env.exe");
        let mut second = path("unsaved-ffmpeg.exe");
        let mut third = String::new();
        controller.edited(1);
        controller.managed_selected(
            0,
            &path("selected.exe"),
            [&mut first, &mut second, &mut third],
        );
        assert_eq!(first, path("selected.exe"));
        assert!(!controller.manual[0]);
        assert!(controller.pending_auto[0]);
        controller.accept_loaded(Ok(saved()), [&mut first, &mut second, &mut third]);
        let merged = controller.auto_snapshot().unwrap();
        assert_eq!(merged.ytdlp, Choice::Automatic);
        assert_eq!(merged.ffmpeg, saved().ffmpeg);
        assert_eq!(second, path("unsaved-ffmpeg.exe"));
        assert!(controller.worker.is_none());
    }
    #[test]
    fn selected_load_failure_preserves_session_choice_and_never_queues_repair() {
        let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
        let mut first = String::new();
        let mut second = String::new();
        let mut third = String::new();
        controller.managed_selected(
            0,
            &path("selected.exe"),
            [&mut first, &mut second, &mut third],
        );
        controller.accept_loaded(Err(Error::Storage), [&mut first, &mut second, &mut third]);
        assert_eq!(first, path("selected.exe"));
        assert_eq!(controller.status, Status::SessionOnly);
        assert!(controller.auto_snapshot().is_none());
        assert!(controller.last_saved.is_none());
    }
    fn ticket(controller: &Controller, value: Preferences) -> SaveTicket {
        SaveTicket {
            revision: 1,
            snapshot: value,
            draft_revision: controller.draft_revision,
            path_revisions: controller.path_revisions,
            mode_revisions: controller.mode_revisions,
            reset_auto: [true; 3],
        }
    }
    #[test]
    fn late_save_preserves_new_path_and_reports_old_snapshot_not_saved_draft() {
        let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
        controller.loaded = true;
        controller.last_saved = Some(Preferences::default());
        controller.latest_requested = Some(1);
        controller.inflight = Some(ticket(&controller, Preferences::default()));
        let mut first = path("typed-new.exe");
        let mut second = path("old-session.exe");
        let mut third = String::new();
        controller.edited(0);
        controller.managed[1] = Some(path("verified-managed.exe"));
        controller.accept_saved(1, Ok(()), [&mut first, &mut second, &mut third]);
        assert_eq!(first, path("typed-new.exe"));
        assert_eq!(second, path("verified-managed.exe"));
        assert_eq!(controller.status, Status::OlderSaved);
        assert_eq!(controller.last_saved, Some(Preferences::default()));
    }
    #[test]
    fn matching_auto_save_resets_environment_for_current_explicit_intent() {
        let mut controller =
            Controller::new(std::env::temp_dir(), [Some(path("env.exe")), None, None]);
        controller.loaded = true;
        controller.last_saved = Some(saved());
        controller.latest_requested = Some(1);
        controller.inflight = Some(ticket(&controller, Preferences::default()));
        let mut first = path("env.exe");
        let mut second = path("typed.exe");
        let mut third = String::new();
        controller.accept_saved(1, Ok(()), [&mut first, &mut second, &mut third]);
        assert_eq!(first, "");
        assert_eq!(second, "");
        assert_eq!(controller.status, Status::Saved);
        assert!(
            controller.environment[0].is_some(),
            "new process still sees explicit environment presence"
        );
    }
    #[test]
    fn managed_only_snapshot_does_not_claim_other_manual_draft_saved() {
        let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
        let original = saved();
        controller.loaded = true;
        controller.manual = [false, true, false];
        controller.last_saved = Some(original.clone());
        controller.latest_requested = Some(1);
        let mut selected = original;
        selected.ytdlp = Choice::Automatic;
        controller.inflight = Some(ticket(&controller, selected));
        let mut first = path("managed.exe");
        let mut second = path("other-unsaved.exe");
        let mut third = String::new();
        controller.accept_saved(1, Ok(()), [&mut first, &mut second, &mut third]);
        assert_eq!(controller.status, Status::OlderSaved);
    }
    #[test]
    fn actual_preference_save_focus_reveals_button_in_bounded_parent_viewport() {
        for action in [
            egui::accesskit::Action::Focus,
            egui::accesskit::Action::ScrollIntoView,
        ] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
            controller.loaded = true;
            controller.last_saved = Some(Preferences::default());
            let mut paths = [String::new(), String::new(), String::new()];
            let mut frame = |events, time| {
                let mut geometry = None;
                let output = ctx.run(
                    egui::RawInput {
                        events,
                        time: Some(time),
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(480.0, 320.0),
                        )),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("preference-focus-parent")
                                .max_height(160.0)
                                .show(ui, |ui| {
                                    ui.add_space(256.0);
                                    let [a, b, c] = &mut paths;
                                    for index in 0..3 {
                                        controller.show_mode(index, ui, true);
                                    }
                                    controller.show_save_status(ui, true, [a, b, c]);
                                    geometry = Some(ui.clip_rect());
                                    ui.add_space(256.0);
                                });
                        });
                    },
                );
                let tree = output.platform_output.accesskit_update.unwrap();
                let (id, node) = tree
                    .nodes
                    .into_iter()
                    .find(|(_, node)| node.label() == Some(crate::catalog::TOOL_PREF_SAVE))
                    .unwrap();
                (id, node.bounds().unwrap(), geometry.unwrap())
            };
            let (id, before, clip) = frame(vec![], 0.0);
            assert!(
                before.y0 > f64::from(clip.bottom()),
                "fixture button starts below viewport"
            );
            frame(
                vec![egui::Event::AccessKitActionRequest(
                    egui::accesskit::ActionRequest {
                        action,
                        target: id,
                        data: None,
                    },
                )],
                1.0,
            );
            let mut actual = before;
            let mut actual_clip = clip;
            for tick in 1..=18 {
                let (_, bounds, clip) = frame(vec![], 1.0 + f64::from(tick) / 30.0);
                actual = bounds;
                actual_clip = clip;
            }
            assert!(
                actual.y0 >= f64::from(actual_clip.top())
                    && actual.y1 <= f64::from(actual_clip.bottom()),
                "focused Save must be fully visible in parent clip: {actual:?} / {actual_clip:?}"
            );
            assert!(paths.iter().all(String::is_empty));
            assert_eq!(controller.last_saved, Some(Preferences::default()));
        }
    }
    #[test]
    fn actual_preference_rows_share_columns_and_fit_narrow_parent() {
        for width in [320.0, 480.0] {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
            controller.loaded = true;
            controller.last_saved = Some(Preferences::default());
            let mut paths = [String::new(), String::new(), String::new()];
            let mut parent = egui::Rect::NOTHING;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 480.0),
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        parent = ui.max_rect();
                        let [a, b, c] = &mut paths;
                        for index in 0..3 {
                            controller.show_mode(index, ui, true);
                        }
                        controller.show_save_status(ui, true, [a, b, c]);
                    });
                },
            );
            let tree = output
                .platform_output
                .accesskit_update
                .expect("actual preferences tree");
            let mut rows = Vec::new();
            for text in [
                crate::catalog::TOOL_YT_DLP,
                crate::catalog::TOOL_FFMPEG,
                crate::catalog::TOOL_DENO,
            ] {
                let (label_id, label) = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == egui::accesskit::Role::Label
                            && (node.label() == Some(text) || node.value() == Some(text))
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "actual tool label missing: {:?}",
                            tree.nodes
                                .iter()
                                .map(|(id, n)| (
                                    id,
                                    n.role(),
                                    n.label(),
                                    n.value(),
                                    n.labelled_by()
                                ))
                                .collect::<Vec<_>>()
                        )
                    });
                let control = tree
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == egui::accesskit::Role::ComboBox
                            && node.labelled_by().contains(label_id)
                    })
                    .unwrap_or_else(|| {
                        panic!(
                            "actual labelled preference combo missing: {:?}",
                            tree.nodes
                                .iter()
                                .map(|(id, n)| (
                                    id,
                                    n.role(),
                                    n.label(),
                                    n.value(),
                                    n.labelled_by()
                                ))
                                .collect::<Vec<_>>()
                        )
                    })
                    .1
                    .bounds()
                    .unwrap();
                let label = label.bounds().unwrap();
                assert!(
                    label.x1 <= control.x0,
                    "label overlaps combo at parent width {width}"
                );
                assert!(
                    ((label.y0 + label.y1) - (control.y0 + control.y1)).abs() <= 2.0,
                    "label/combo vertical centers differ at width {width}: {label:?} {control:?}"
                );
                assert!(
                    control.x1 <= f64::from(parent.right()) + 0.5,
                    "combo exceeds narrow parent: {control:?} {parent:?}"
                );
                rows.push(control);
            }
            for row in &rows[1..] {
                assert!(
                    (row.x0 - rows[0].x0).abs() <= 0.5,
                    "actual ComboBox left edges differ: {rows:?}"
                );
                assert!(
                    ((row.x1 - row.x0) - (rows[0].x1 - rows[0].x0)).abs() <= 0.5,
                    "actual ComboBox widths differ: {rows:?}"
                );
            }
            assert!(
                rows[0].y1 <= rows[1].y0 && rows[1].y1 <= rows[2].y0,
                "actual preference rows overlap: {rows:?}"
            );
        }
    }
    #[test]
    fn stop_before_first_poll_never_spawns_and_ack_is_immediate() {
        let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
        assert!(controller.busy());
        controller.request_stop();
        assert!(controller.stopped());
        assert!(!controller.busy());
        assert!(controller.worker.is_none());
    }
}

#[cfg(test)]
mod automatic_eligibility_ui_regressions {
    use super::*;

    // Current public UI, no worker or manager fixture: unknown installed eligibility
    // must not advertise a usable Automatic action. Root executes ORIGINAL RED.
    #[test]
    fn actual_unknown_managed_eligibility_disables_automatic_preserves_manual_intent() {
        use crate::catalog as c;
        use egui::accesskit::{Action, ActionRequest, Role};
        for restored_preferences in [false, true] {
            let ctx = egui::Context::default();
            crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
            ctx.enable_accesskit();
            let mut controller =
                Controller::new(std::env::temp_dir(), [Some(String::new()), None, None]);
            controller.loaded = restored_preferences;
            if restored_preferences {
                controller.last_saved = Some(Preferences::default());
            }
            controller.edited(0);
            let before_revisions = (
                controller.path_revisions,
                controller.mode_revisions,
                controller.draft_revision,
            );
            let mut time = 0.0;
            let mut frame = |controller: &mut Controller, events| {
                time += 0.05;
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(480.0, 320.0),
                        )),
                        focused: true,
                        time: Some(time),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .show(ctx, |ui| controller.show_mode(0, ui, true));
                    },
                )
            };
            let initial = frame(&mut controller, vec![]);
            let combo = initial
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, n)| n.role() == Role::ComboBox)
                .unwrap()
                .0;
            let click = |target| {
                egui::Event::AccessKitActionRequest(ActionRequest {
                    target,
                    action: Action::Click,
                    data: None,
                })
            };
            frame(&mut controller, vec![click(combo)]);
            let opened = frame(&mut controller, vec![]);
            assert!(egui::Popup::is_any_open(&ctx), "actual mode popup opened");
            let nodes = &opened
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes;
            let (automatic_id, automatic) = nodes
                .iter()
                .find(|(_, n)| n.role() == Role::Button && n.label() == Some(c::TOOL_PREF_AUTO))
                .expect("actual Automatic option provider");
            let (_, manual) = nodes
                .iter()
                .find(|(_, n)| n.role() == Role::Button && n.label() == Some(c::TOOL_PREF_MANUAL))
                .expect("actual Manual option provider");
            assert!(manual.supports_action(Action::Click) && !manual.is_disabled());
            assert!(
                automatic.is_disabled(),
                "unknown managed eligibility must disable Automatic; restored_preferences={restored_preferences}"
            );
            frame(&mut controller, vec![click(*automatic_id)]);
            frame(&mut controller, vec![]);
            assert!(
                controller.manual[0],
                "disabled Auto cannot replace explicit typed-empty Manual intent"
            );
            assert_eq!(
                (
                    controller.path_revisions,
                    controller.mode_revisions,
                    controller.draft_revision
                ),
                before_revisions
            );
            assert_eq!(controller.environment[0].as_deref(), Some(""));
            assert!(!controller.pending_auto.iter().any(|v| *v));
            assert!(
                controller.worker.is_none()
                    && controller.inflight.is_none()
                    && controller.queued.is_none()
            );
        }
    }
}

#[cfg(test)]
mod managed_availability_regressions {
    use super::*;
    use egui::accesskit::{Action, ActionRequest, Role};

    fn path(leaf: &str) -> String {
        std::env::temp_dir().join(leaf).to_str().unwrap().to_owned()
    }
    fn click(target: egui::accesskit::NodeId) -> egui::Event {
        egui::Event::AccessKitActionRequest(ActionRequest {
            target,
            action: Action::Click,
            data: None,
        })
    }
    fn frame(
        ctx: &egui::Context,
        c: &mut Controller,
        field: &mut String,
        events: Vec<egui::Event>,
        time: &mut f64,
        mutate: bool,
    ) -> egui::FullOutput {
        *time += 0.05;
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600.0, 400.0),
                )),
                focused: true,
                time: Some(*time),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    c.show_mode_with_field(0, ui, mutate, field);
                    crate::ui_components::text_input_with_id(
                        ui,
                        field,
                        MAX_PATH,
                        480.0,
                        egui::Id::new("managed-availability-real-field"),
                    );
                });
            },
        )
    }
    fn find(out: &egui::FullOutput, role: Role, label: Option<&str>) -> egui::accesskit::NodeId {
        out.platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, n)| n.role() == role && label.is_none_or(|v| n.label() == Some(v)))
            .expect("actual rendered provider")
            .0
    }
    fn painted_text(shape: &egui::Shape, expected: &str) -> bool {
        match shape {
            egui::Shape::Text(t) => t.galley.text() == expected,
            egui::Shape::Vec(v) => v.iter().any(|s| painted_text(s, expected)),
            _ => false,
        }
    }
    #[test]
    fn actual_availability_disables_invalid_then_enables_and_applies_explicit_auto_path() {
        use crate::catalog as c;
        let ctx = egui::Context::default();
        crate::accessible::configure_context(&ctx, crate::accessible::font_bytes());
        ctx.enable_accesskit();
        let mut controller = Controller::new(std::env::temp_dir(), [None, None, None]);
        controller.loaded = true;
        controller.last_saved = Some(Preferences::default());
        let mut field = path("explicit-custom.exe");
        controller.edited(0);
        let mut time = 0.0;
        let first = frame(&ctx, &mut controller, &mut field, vec![], &mut time, true);
        let combo = find(&first, Role::ComboBox, None);
        assert!(
            first
                .shapes
                .iter()
                .any(|s| painted_text(&s.shape, c::TOOL_AUTO_CHECKING))
        );
        // Invalid worker result cannot become selectable merely because it is Available.
        controller
            .set_managed_availability(0, ManagedAvailability::Available("relative.exe".into()));
        let invalid = frame(&ctx, &mut controller, &mut field, vec![], &mut time, true);
        assert_eq!(find(&invalid, Role::ComboBox, None), combo);
        assert!(
            invalid
                .shapes
                .iter()
                .any(|s| painted_text(&s.shape, c::TOOL_AUTO_UNAVAILABLE))
        );
        assert_eq!(field, path("explicit-custom.exe"));
        frame(
            &ctx,
            &mut controller,
            &mut field,
            vec![click(combo)],
            &mut time,
            true,
        );
        let open = frame(&ctx, &mut controller, &mut field, vec![], &mut time, true);
        let auto = find(&open, Role::Button, Some(c::TOOL_PREF_AUTO));
        let nodes = &open
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let n = &nodes.iter().find(|(id, _)| *id == auto).unwrap().1;
        assert!(n.is_disabled());
        frame(
            &ctx,
            &mut controller,
            &mut field,
            vec![click(auto)],
            &mut time,
            true,
        );
        assert!(controller.manual[0]);
        assert_eq!(field, path("explicit-custom.exe"));
        // Close the actual popup before replacing its worker eligibility snapshot.
        egui::Popup::close_all(&ctx);
        let verified = path("worker-verified.exe");
        controller.set_managed_availability(0, ManagedAvailability::Available(verified.clone()));
        let available = frame(&ctx, &mut controller, &mut field, vec![], &mut time, true);
        assert_eq!(find(&available, Role::ComboBox, None), combo);
        assert_eq!(
            field,
            path("explicit-custom.exe"),
            "worker cache alone cannot overwrite draft"
        );
        frame(
            &ctx,
            &mut controller,
            &mut field,
            vec![click(combo)],
            &mut time,
            true,
        );
        let opened = frame(&ctx, &mut controller, &mut field, vec![], &mut time, true);
        let auto = find(&opened, Role::Button, Some(c::TOOL_PREF_AUTO));
        let n = &opened
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(id, _)| *id == auto)
            .unwrap()
            .1;
        assert!(!n.is_disabled() && n.supports_action(Action::Click));
        frame(
            &ctx,
            &mut controller,
            &mut field,
            vec![click(auto)],
            &mut time,
            true,
        );
        let committed = frame(&ctx, &mut controller, &mut field, vec![], &mut time, true);
        assert!(!egui::Popup::is_any_open(&ctx));
        assert!(!controller.manual[0]);
        assert_eq!(field, verified);
        assert_eq!(controller.mode_revisions[0], 2);
        assert_eq!(controller.path_revisions[0], 2);
        assert!(
            committed
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, n)| n.role() == Role::TextInput && n.value() == Some(field.as_str()))
        );
        assert_eq!(controller.status, Status::Dirty);
        assert!(
            controller.worker.is_none()
                && controller.inflight.is_none()
                && controller.queued.is_none()
        );
        assert!(!controller.pending_auto.iter().any(|v| *v));
        controller.request_stop();
        let stopped = frame(&ctx, &mut controller, &mut field, vec![], &mut time, false);
        let nodes = &stopped
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        assert!(
            nodes
                .iter()
                .find(|(id, _)| *id == combo)
                .unwrap()
                .1
                .is_disabled()
        );
        frame(
            &ctx,
            &mut controller,
            &mut field,
            vec![click(combo)],
            &mut time,
            false,
        );
        assert!(!egui::Popup::is_any_open(&ctx));
        assert_eq!(field, verified);
        assert!(controller.worker.is_none() && controller.stopped());
    }
    #[test]
    fn availability_cache_late_load_edit_and_close_wait_preserve_explicit_provenance() {
        let mut c = Controller::new(std::env::temp_dir(), [Some(String::new()), None, None]);
        let mut first = String::new();
        let mut second = path("custom-ffmpeg.exe");
        let mut third = String::new();
        c.edited(1);
        c.set_managed_availability(
            0,
            ManagedAvailability::Available(path("verified-ytdlp.exe")),
        );
        c.set_managed_availability(
            1,
            ManagedAvailability::Available(path("verified-ffmpeg.exe")),
        );
        c.accept_loaded(
            Ok(Preferences::default()),
            [&mut first, &mut second, &mut third],
        );
        assert_eq!(first, "");
        assert_eq!(second, path("custom-ffmpeg.exe"));
        assert!(c.manual[1]);
        c.set_managed_availability(
            1,
            ManagedAvailability::Unavailable(crate::catalog::TOOL_AUTO_UNAVAILABLE),
        );
        assert_eq!(second, path("custom-ffmpeg.exe"));
        assert!(c.managed[1].is_none());
        let modes = c.mode_revisions;
        let paths = c.path_revisions;
        let draft = c.draft_revision;
        c.request_stop();
        c.set_managed_availability(1, ManagedAvailability::Available(path("too-late.exe")));
        c.managed_restored(
            1,
            &path("too-late.exe"),
            [&mut first, &mut second, &mut third],
        );
        assert_eq!(second, path("custom-ffmpeg.exe"));
        assert_eq!(c.mode_revisions, modes);
        assert_eq!(c.path_revisions, paths);
        assert_eq!(c.draft_revision, draft);
        assert!(c.managed[1].is_none() && c.worker.is_none() && c.stopped());
    }
}

#[cfg(test)]
mod managed_auto_ticket_regressions {
    use super::*;
    fn path(name: &str) -> String {
        if cfg!(windows) {
            format!("C:\\owned\\{name}.exe")
        } else {
            format!("/owned/{name}")
        }
    }
    #[test]
    fn exact_slot_ticket_updates_auto_after_earlier_explicit_mode_revisions() {
        let mut c = Controller::new(PathBuf::from("owned-unused"), [None, None, None]);
        let mut fields = [String::new(), String::new(), String::new()];
        let [a, b, d] = &mut fields;
        c.accept_loaded(Ok(Preferences::default()), [a, b, d]);
        c.path_revisions[0] = 2;
        c.mode_revisions[0] = 3;
        let ticket = c
            .managed_auto_ticket(0)
            .expect("saved Auto may have earlier session revisions");
        let expected = path("new-version");
        let [a, b, d] = &mut fields;
        c.managed_auto_selected(0, &expected, ticket, [a, b, d]);
        assert_eq!(fields[0], expected);
        assert_eq!(c.path_revisions[0], 2);
        assert_eq!(c.mode_revisions[0], 3);
        assert_eq!(
            c.draft_revision, 0,
            "background path must not create a save"
        );
    }
    #[test]
    fn late_manual_edit_empty_environment_close_and_saturation_refuse_auto_ticket() {
        let mut c = Controller::new(PathBuf::from("owned-unused"), [None, None, None]);
        let mut fields = [String::new(), String::new(), String::new()];
        let [a, b, d] = &mut fields;
        c.accept_loaded(Ok(Preferences::default()), [a, b, d]);
        let ticket = c.managed_auto_ticket(0).unwrap();
        c.edited(0);
        fields[0] = path("explicit-custom");
        let original = fields[0].clone();
        let [a, b, d] = &mut fields;
        c.managed_auto_selected(0, &path("late"), ticket, [a, b, d]);
        assert_eq!(fields[0], original);
        assert!(c.managed_auto_ticket(0).is_none());
        c.manual[0] = false;
        c.environment[0] = Some(String::new());
        assert!(c.managed_auto_ticket(0).is_none());
        c.environment[0] = None;
        c.path_revisions[0] = u64::MAX;
        assert!(c.managed_auto_ticket(0).is_none());
        c.path_revisions[0] = 1;
        c.stopping = true;
        assert!(c.managed_auto_ticket(0).is_none());
        let old_cache = c.managed[0].clone();
        let [a, b, d] = &mut fields;
        c.managed_auto_selected(0, &path("closed"), ticket, [a, b, d]);
        assert_eq!(fields[0], original);
        assert_eq!(c.managed[0], old_cache);
    }
}
