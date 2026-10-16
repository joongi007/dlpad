//! Session-only Netscape cookie copies. No source mutation or credential logging.
//! Synchronous bounded I/O: callers must run imports/cleanup off the UI thread.
//! `path_for_tool` is a sensitive capability, never a normal job snapshot field.
use std::{
    fmt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg_attr(not(any(unix, windows)), allow(dead_code))]
mod format;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod reader_journal;
#[cfg(target_os = "linux")]
#[allow(dead_code)]
mod reader_records;
#[cfg(target_os = "linux")]
pub(crate) use reader_journal::{JournalAnchor, ReaderLaunch};
#[cfg(target_os = "linux")]
pub(crate) use reader_records::identity_digest as reader_identity_digest;
#[cfg(target_os = "linux")]
use std::sync::Mutex;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
#[allow(unused_imports)]
pub(crate) use windows::guardian::GuardianJournal;
#[cfg(all(windows, test))]
pub(crate) use windows::test_fixture as windows_test_fixture;
// Pure Windows recovery format; native journal/guardian integration is next.
#[cfg(any(windows, test))]
#[allow(dead_code)]
pub(crate) mod windows_records;

pub const MAX_COOKIE_BYTES: usize = 1024 * 1024;
pub const MAX_COOKIE_LINE_BYTES: usize = 16 * 1024;
pub const MAX_COOKIE_ENTRIES: usize = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CookieImportError {
    Unavailable,
    SourceUnavailable,
    InvalidFormat,
    TooLarge,
    LineTooLong,
    TooManyEntries,
    UnsafeCache,
    StorageUnavailable,
    CleanupOwnershipChanged,
    CacheFull,
    ReaderUnconfirmed,
}
impl fmt::Display for CookieImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "private session cookies unavailable on this platform",
            Self::SourceUnavailable => "cookie source unavailable",
            Self::InvalidFormat => "invalid Netscape cookie format",
            Self::TooLarge => "cookie file exceeds byte limit",
            Self::LineTooLong => "cookie line exceeds byte limit",
            Self::TooManyEntries => "cookie file exceeds entry limit",
            Self::UnsafeCache => "private cookie cache unavailable",
            Self::StorageUnavailable => "private cookie copy unavailable",
            Self::ReaderUnconfirmed => "cookie reader termination unconfirmed",
            Self::CacheFull => "private cookie cache capacity reached",
            Self::CleanupOwnershipChanged => "cookie cleanup ownership changed",
        })
    }
}
impl std::error::Error for CookieImportError {}

pub const MAX_COOKIE_SESSIONS: usize = 128;
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CookieRecoverySummary {
    pub scanned: usize,
    pub recovered: usize,
    pub deferred: usize,
    pub rejected: usize,
    pub truncated: bool,
}
/// Linux recovery requires anchored ownership and actual cleanup receipts, or
/// separately verified previous-boot authority. Windows requires actual bound receipts.
/// Other platforms remain unavailable.
pub fn recover_sessions(owned_cache: &Path) -> Result<CookieRecoverySummary, CookieImportError> {
    #[cfg(target_os = "linux")]
    {
        unix::recovery::recover(owned_cache)
    }
    #[cfg(windows)]
    {
        windows::recovery::recover(owned_cache)
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = owned_cache;
        Err(CookieImportError::Unavailable)
    }
}

struct SensitivePath(PathBuf);
impl fmt::Debug for SensitivePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// One session-owned copy. Drop performs best-effort identity-checked cleanup.
/// Keep this lease alive until every tool using its path has terminated.
pub struct SessionCookieLease {
    path: SensitivePath,
    entries: usize,
    bytes: usize,
    reader_pending: AtomicBool,
    #[cfg(unix)]
    owned: Option<unix::OwnedCopy>,
    #[cfg(windows)]
    owned: Option<windows::OwnedCopy>,
}
impl fmt::Debug for SessionCookieLease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionCookieLease(<redacted>)")
    }
}
/// A single reader reservation. Drop alone never acknowledges tool termination.
/// Only call confirm after the process backend has confirmed its terminal cleanup.
pub struct SessionCookieReader<'a> {
    pending: &'a AtomicBool,
}
impl fmt::Debug for SessionCookieReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionCookieReader(<redacted>)")
    }
}
impl SessionCookieReader<'_> {
    pub fn confirm(self) {
        self.pending.store(false, Ordering::Release);
    }
}
impl Drop for SessionCookieReader<'_> {
    fn drop(&mut self) {
        // Deliberately leave the reservation pending on unwind or missing receipt.
    }
}
#[cfg(target_os = "linux")]
struct ReaderSeed {
    nonce: [u8; 16],
    digest: [u8; 32],
    boot: [u8; 16],
}
#[cfg(target_os = "linux")]
type ReaderJournalMutex = Mutex<reader_journal::ReaderJournal>;
#[cfg(target_os = "linux")]
pub(crate) struct GatedCookieReader<'a> {
    journal: &'a ReaderJournalMutex,
    pending: &'a AtomicBool,
    capability: crate::process::reader_protocol::Capability,
    launch: Option<ReaderLaunch>,
    anchor: JournalAnchor,
    confirmed: bool,
}
#[cfg(target_os = "linux")]
impl fmt::Debug for GatedCookieReader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GatedCookieReader(<redacted>)")
    }
}
#[cfg(target_os = "linux")]
impl GatedCookieReader<'_> {
    pub(crate) fn take_launch(
        &mut self,
    ) -> Result<(ReaderLaunch, JournalAnchor), CookieImportError> {
        self.launch
            .take()
            .map(|launch| (launch, self.anchor))
            .ok_or(CookieImportError::ReaderUnconfirmed)
    }
    pub(crate) fn bind_identity(
        &mut self,
        identity: &crate::process::reader_protocol::ReaderIdentity,
    ) -> Result<(), CookieImportError> {
        if self.confirmed || self.launch.is_some() || identity.capability != self.capability {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        self.journal
            .try_lock()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?
            .bind(identity.clone())
            .map_err(|_| CookieImportError::ReaderUnconfirmed)
    }
    pub(crate) fn confirm_terminal(&mut self) -> Result<(), CookieImportError> {
        if self.confirmed || self.launch.is_some() {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        self.journal
            .try_lock()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?
            .resolve(self.capability.index)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        self.confirmed = true;
        self.pending.store(false, Ordering::Release);
        Ok(())
    }
}
#[cfg(target_os = "linux")]
impl Drop for GatedCookieReader<'_> {
    fn drop(&mut self) {
        // No implicit acknowledgment: durable pending state survives unwinding.
    }
}
impl SessionCookieLease {
    #[cfg(windows)]
    pub(crate) fn begin_windows_gated_reader(
        &self,
        index: u8,
    ) -> Result<windows::WindowsGatedCookieReader<'_>, CookieImportError> {
        self.validate_for_tool()?;
        self.owned
            .as_ref()
            .ok_or(CookieImportError::CleanupOwnershipChanged)?
            .begin_windows_reader(&self.reader_pending, index)
    }
    #[cfg(target_os = "linux")]
    pub(crate) fn begin_gated_reader(
        &self,
        index: u8,
    ) -> Result<GatedCookieReader<'_>, CookieImportError> {
        self.validate_for_tool()?;
        if index > 1 {
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        let owned = self
            .owned
            .as_ref()
            .ok_or(CookieImportError::CleanupOwnershipChanged)?;
        let (journal, seed) = owned.reader_seed()?;
        let mut locked = journal
            .try_lock()
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        self.reader_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        // Any failure after beginning durable intent conservatively retains pending.
        let launch = locked
            .begin(index, seed.nonce, seed.digest, seed.boot)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        let anchor = locked.anchor();
        let capability = launch.capability.clone();
        drop(locked);
        Ok(GatedCookieReader {
            journal,
            pending: &self.reader_pending,
            capability,
            launch: Some(launch),
            anchor,
            confirmed: false,
        })
    }
    pub fn begin_reader(&self) -> Result<SessionCookieReader<'_>, CookieImportError> {
        self.validate_for_tool()?;
        self.reader_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
        Ok(SessionCookieReader {
            pending: &self.reader_pending,
        })
    }
    pub fn entries(&self) -> usize {
        self.entries
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    /// Sensitive tool-only path, valid only while the lease has not been cleaned.
    pub fn path_for_tool(&self) -> &Path {
        &self.path.0
    }
    /// Revalidate private ownership immediately before launching a tool.
    /// Keep this lease alive until that tool has terminated. This check does not
    /// provide atomic protection against concurrent malicious same-UID changes.
    pub fn validate_for_tool(&self) -> Result<(), CookieImportError> {
        #[cfg(any(unix, windows))]
        {
            self.owned
                .as_ref()
                .ok_or(CookieImportError::CleanupOwnershipChanged)?
                .validate_for_tool()
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(CookieImportError::Unavailable)
        }
    }
    /// Leave this private copy for explicit recovery when tool termination is unconfirmed.
    /// Closes owned handles without unlinking; this lease becomes invalid for execution.
    /// This does not perform or promise startup recovery.
    pub fn defer_cleanup(&mut self) {
        #[cfg(any(unix, windows))]
        if let Some(mut owned) = self.owned.take() {
            owned.defer_cleanup();
        }
    }
    /// No recursive deletion. An identity mismatch leaves the changed object intact.
    pub fn cleanup(&mut self) -> Result<(), CookieImportError> {
        if self.reader_pending.load(Ordering::Acquire) {
            self.defer_cleanup();
            return Err(CookieImportError::ReaderUnconfirmed);
        }
        #[cfg(any(unix, windows))]
        {
            if let Some(owned) = self.owned.as_mut() {
                owned.cleanup()?;
            }
            self.owned = None;
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(CookieImportError::Unavailable)
        }
    }
}
impl Drop for SessionCookieLease {
    fn drop(&mut self) {
        if self.reader_pending.load(Ordering::Acquire) {
            self.defer_cleanup();
        }
        let _ = self.cleanup();
    }
}

/// The caller supplies an absolute app-owned cache directory, not user media/data.
/// Unix requires private ownership/mode and rejects symlink path components.
/// Windows uses protected current-user ACLs and pinned native file identities.
/// Other platforms fail closed. Startup recovery remains Linux-only.
pub fn import_session(
    source: &Path,
    owned_cache: &Path,
) -> Result<SessionCookieLease, CookieImportError> {
    #[cfg(unix)]
    {
        unix::import(source, owned_cache)
    }
    #[cfg(windows)]
    {
        let (owned, path, entries, bytes) = windows::import(source, owned_cache)?;
        Ok(SessionCookieLease {
            path: SensitivePath(path),
            entries,
            bytes,
            reader_pending: AtomicBool::new(false),
            owned: Some(owned),
        })
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (source, owned_cache);
        Err(CookieImportError::Unavailable)
    }
}

/// Create a new Windows app-owned private cache, or validate an existing cache.
/// Existing directories and their ACLs are never rewritten.
#[cfg(windows)]
pub fn prepare_session_cache(path: &Path) -> Result<(), CookieImportError> {
    windows::prepare_cache(path)
}

#[cfg(unix)]
mod unix {
    use super::format::{SensitiveBlock, SensitiveBytes, validate};
    use super::*;
    use std::{
        ffi::{CStr, CString},
        fs::{File, Metadata},
        io::{Read, Write},
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::{ffi::OsStrExt, fs::MetadataExt},
        },
        path::Component,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    const FILE_NAME: &CStr = c"cookies.txt";

    #[derive(Clone, Copy, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Identity {
        device: u64,
        inode: u64,
    }
    impl Identity {
        fn of(meta: &Metadata) -> Self {
            Self {
                device: meta.dev(),
                inode: meta.ino(),
            }
        }
    }
    struct Entry {
        identity: Identity,
        uid: u32,
        mode: u32,
        links: u64,
    }
    fn wide<T: TryInto<u64>>(value: T) -> Result<u64, CookieImportError> {
        value
            .try_into()
            .map_err(|_| CookieImportError::CleanupOwnershipChanged)
    }
    fn mode<T: TryInto<u32>>(value: T) -> Result<u32, CookieImportError> {
        value
            .try_into()
            .map_err(|_| CookieImportError::CleanupOwnershipChanged)
    }
    fn entry(parent: &File, name: &CStr) -> Result<Option<Entry>, CookieImportError> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: valid borrowed fd/name and a writable stat buffer.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result < 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err(CookieImportError::CleanupOwnershipChanged);
        }
        // SAFETY: fstatat initialized the complete stat on success.
        let stat = unsafe { stat.assume_init() };
        Ok(Some(Entry {
            identity: Identity {
                device: wide(stat.st_dev)?,
                inode: wide(stat.st_ino)?,
            },
            uid: stat.st_uid,
            mode: mode(stat.st_mode)?,
            links: wide(stat.st_nlink)?,
        }))
    }
    fn uid() -> u32 {
        // SAFETY: geteuid takes no pointers and cannot fail.
        unsafe { libc::geteuid() }
    }
    fn descriptor(
        result: libc::c_int,
        error: CookieImportError,
    ) -> Result<File, CookieImportError> {
        if result < 0 {
            return Err(error);
        }
        // SAFETY: each successful open returns a new fd owned by this File.
        Ok(unsafe { File::from_raw_fd(result) })
    }
    fn directory(path: &Path, error: CookieImportError) -> Result<File, CookieImportError> {
        if !path.is_absolute() {
            return Err(error);
        }
        // SAFETY: fixed NUL-terminated root path; returned descriptor is adopted once.
        let mut current = descriptor(
            unsafe {
                libc::open(
                    c"/".as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            },
            error,
        )?;
        for component in path.components() {
            match component {
                Component::RootDir => (),
                Component::Normal(name) => {
                    let name = CString::new(name.as_bytes()).map_err(|_| error)?;
                    // SAFETY: valid parent fd and NUL-terminated component, no symlink following.
                    current = descriptor(
                        unsafe {
                            libc::openat(
                                current.as_raw_fd(),
                                name.as_ptr(),
                                libc::O_RDONLY
                                    | libc::O_DIRECTORY
                                    | libc::O_NOFOLLOW
                                    | libc::O_CLOEXEC,
                            )
                        },
                        error,
                    )?;
                }
                _ => return Err(error),
            }
        }
        Ok(current)
    }
    fn private(meta: &Metadata, mode: u32, identity: Identity) -> bool {
        Identity::of(meta) == identity && meta.uid() == uid() && meta.mode() & 0o7777 == mode
    }
    fn source_bytes(source: &Path) -> Result<SensitiveBytes, CookieImportError> {
        let parent = source
            .parent()
            .ok_or(CookieImportError::SourceUnavailable)?;
        let name = source
            .file_name()
            .ok_or(CookieImportError::SourceUnavailable)?;
        let name =
            CString::new(name.as_bytes()).map_err(|_| CookieImportError::SourceUnavailable)?;
        let parent = directory(parent, CookieImportError::SourceUnavailable)?;
        let initial = entry(&parent, &name)
            .map_err(|_| CookieImportError::SourceUnavailable)?
            .ok_or(CookieImportError::SourceUnavailable)?;
        if initial.mode & mode(libc::S_IFMT)? != mode(libc::S_IFREG)? {
            return Err(CookieImportError::SourceUnavailable);
        }
        // SAFETY: anchored component. O_NONBLOCK prevents FIFO/device waits before type rejection.
        let mut file = descriptor(
            unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
                )
            },
            CookieImportError::SourceUnavailable,
        )?;
        let before = file
            .metadata()
            .map_err(|_| CookieImportError::SourceUnavailable)?;
        if !before.is_file() || Identity::of(&before) != initial.identity {
            return Err(CookieImportError::SourceUnavailable);
        }
        if before.len() > MAX_COOKIE_BYTES as u64 {
            return Err(CookieImportError::TooLarge);
        }
        let mut data = SensitiveBytes(Vec::with_capacity(MAX_COOKIE_BYTES + 1));
        let mut block = SensitiveBlock([0; 4096]);
        loop {
            let remaining = (MAX_COOKIE_BYTES + 1 - data.0.len()).min(block.0.len());
            let count = file
                .read(&mut block.0[..remaining])
                .map_err(|_| CookieImportError::SourceUnavailable)?;
            if count == 0 {
                break;
            }
            if data.0.len() + count > MAX_COOKIE_BYTES {
                return Err(CookieImportError::TooLarge);
            }
            data.0.extend_from_slice(&block.0[..count]);
        }
        let after = file
            .metadata()
            .map_err(|_| CookieImportError::SourceUnavailable)?;
        if Identity::of(&before) != Identity::of(&after)
            || before.len() != data.0.len() as u64
            || before.len() != after.len()
            || (
                before.mtime(),
                before.mtime_nsec(),
                before.ctime(),
                before.ctime_nsec(),
            ) != (
                after.mtime(),
                after.mtime_nsec(),
                after.ctime(),
                after.ctime_nsec(),
            )
        {
            return Err(CookieImportError::SourceUnavailable);
        }
        Ok(data)
    }
    pub(super) struct OwnedCopy {
        cache_path: SensitivePath,
        parent: File,
        parent_identity: Identity,
        owner: File,
        owner_identity: Identity,
        owner_name: CString,
        file: Option<(File, Identity)>,
        removed: bool,
        #[cfg(target_os = "linux")]
        records: Option<recovery::Records>,
        #[cfg(target_os = "linux")]
        journal: Option<Mutex<super::reader_journal::ReaderJournal>>,
    }
    impl OwnedCopy {
        #[cfg(target_os = "linux")]
        pub(super) fn reader_seed(
            &self,
        ) -> Result<(&ReaderJournalMutex, ReaderSeed), CookieImportError> {
            let journal = self
                .journal
                .as_ref()
                .ok_or(CookieImportError::Unavailable)?;
            let seed = self
                .records
                .as_ref()
                .ok_or(CookieImportError::Unavailable)?
                .reader_seed()?;
            Ok((journal, seed))
        }
        pub(super) fn defer_cleanup(&mut self) {
            // Suppress Drop's unlink path; normal File drops still close every FD.
            self.removed = true;
            self.file = None;
        }
        pub(super) fn validate_for_tool(&self) -> Result<(), CookieImportError> {
            if self.removed {
                return Err(CookieImportError::CleanupOwnershipChanged);
            }
            self.owner_valid()?;
            self.file_valid()?;
            #[cfg(target_os = "linux")]
            if let Some(records) = &self.records {
                records.validate(self)?;
            }
            Ok(())
        }
        fn owner_valid(&self) -> Result<(), CookieImportError> {
            let current_parent = directory(
                &self.cache_path.0,
                CookieImportError::CleanupOwnershipChanged,
            )?;
            let meta = current_parent
                .metadata()
                .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
            if !meta.is_dir() || !private(&meta, 0o700, self.parent_identity) {
                return Err(CookieImportError::CleanupOwnershipChanged);
            }
            let meta = self
                .owner
                .metadata()
                .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
            let Some(current) = entry(&self.parent, &self.owner_name)? else {
                return Err(CookieImportError::CleanupOwnershipChanged);
            };
            if !meta.is_dir()
                || !private(&meta, 0o700, self.owner_identity)
                || current.identity != self.owner_identity
                || current.uid != uid()
                || current.mode & mode(libc::S_IFMT)? != mode(libc::S_IFDIR)?
                || current.mode & 0o7777 != 0o700
            {
                return Err(CookieImportError::CleanupOwnershipChanged);
            }
            Ok(())
        }
        fn file_valid(&self) -> Result<(), CookieImportError> {
            let current =
                entry(&self.owner, FILE_NAME)?.ok_or(CookieImportError::CleanupOwnershipChanged)?;
            let Some((file, expected)) = &self.file else {
                return Err(CookieImportError::CleanupOwnershipChanged);
            };
            let meta = file
                .metadata()
                .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
            if !meta.is_file()
                || !private(&meta, 0o600, *expected)
                || current.identity != *expected
                || current.uid != uid()
                || current.mode & mode(libc::S_IFMT)? != mode(libc::S_IFREG)?
                || current.mode & 0o7777 != 0o600
                || current.links != 1
            {
                return Err(CookieImportError::CleanupOwnershipChanged);
            }
            Ok(())
        }
        pub(super) fn cleanup(&mut self) -> Result<(), CookieImportError> {
            self.cleanup_inner(
                #[cfg(target_os = "linux")]
                None,
            )
        }
        fn cleanup_inner(
            &mut self,
            #[cfg(target_os = "linux")] previous_boot: Option<
                &crate::cookies::reader_journal::PreviousBootAuthority,
            >,
        ) -> Result<(), CookieImportError> {
            if self.removed {
                return Ok(());
            }
            self.owner_valid()?;
            #[cfg(target_os = "linux")]
            if let Some(records) = &self.records {
                records.validate(self)?;
            }
            #[cfg(target_os = "linux")]
            if let Some(journal) = &self.journal {
                let journal = journal
                    .try_lock()
                    .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
                if let Some(authority) = previous_boot {
                    journal
                        .validate_previous_boot(authority)
                        .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
                } else if !journal
                    .cleanup_eligible()
                    .map_err(|_| CookieImportError::CleanupOwnershipChanged)?
                {
                    return Err(CookieImportError::ReaderUnconfirmed);
                }
            }
            #[cfg(target_os = "linux")]
            if let Some(authority) = previous_boot {
                authority
                    .revalidate()
                    .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
            }
            if entry(&self.owner, FILE_NAME)?.is_some() {
                self.file_valid()?;
                self.owner_valid()?;
                // SAFETY: private anchored directory and verified regular owned file entry.
                if unsafe { libc::unlinkat(self.owner.as_raw_fd(), FILE_NAME.as_ptr(), 0) } < 0 {
                    return Err(CookieImportError::StorageUnavailable);
                }
            }
            self.file = None;
            #[cfg(target_os = "linux")]
            if let Some(journal) = self.journal.take() {
                let journal = journal
                    .into_inner()
                    .map_err(|_| CookieImportError::ReaderUnconfirmed)?;
                match previous_boot {
                    Some(authority) => journal.remove_owned_files_previous_boot(authority),
                    None => journal.remove_owned_files(),
                }
                .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
            }
            #[cfg(target_os = "linux")]
            if let Some(records) = &self.records {
                if let Some(authority) = previous_boot {
                    authority
                        .revalidate()
                        .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
                }
                records.remove(self)?;
            }
            #[cfg(target_os = "linux")]
            {
                self.records = None;
            }
            self.owner_valid()?;
            #[cfg(target_os = "linux")]
            if let Some(authority) = previous_boot {
                authority
                    .revalidate()
                    .map_err(|_| CookieImportError::CleanupOwnershipChanged)?;
            }
            // SAFETY: verified exact owner entry; rmdir refuses any foreign/extra contents.
            if unsafe {
                libc::unlinkat(
                    self.parent.as_raw_fd(),
                    self.owner_name.as_ptr(),
                    libc::AT_REMOVEDIR,
                )
            } < 0
            {
                return Err(CookieImportError::CleanupOwnershipChanged);
            }
            self.removed = true;
            Ok(())
        }
    }
    impl Drop for OwnedCopy {
        fn drop(&mut self) {
            let _ = self.cleanup();
        }
    }

    pub(super) fn import(
        source: &Path,
        cache: &Path,
    ) -> Result<SessionCookieLease, CookieImportError> {
        let data = source_bytes(source)?;
        let entries = validate(&data.0)?;
        let parent = directory(cache, CookieImportError::UnsafeCache)?;
        let meta = parent
            .metadata()
            .map_err(|_| CookieImportError::UnsafeCache)?;
        let parent_identity = Identity::of(&meta);
        if !meta.is_dir() || !private(&meta, 0o700, parent_identity) {
            return Err(CookieImportError::UnsafeCache);
        }
        #[cfg(target_os = "linux")]
        {
            if recovery::names(&parent, MAX_COOKIE_SESSIONS)?.len() >= MAX_COOKIE_SESSIONS {
                return Err(CookieImportError::CacheFull);
            }
        }
        let mut created = None;
        for _ in 0..16 {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| CookieImportError::StorageUnavailable)?
                .as_nanos();
            let name = CString::new(format!(
                "cookie-session-{}-{timestamp:x}-{:x}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed)
            ))
            .map_err(|_| CookieImportError::StorageUnavailable)?;
            // SAFETY: private parent fd/name; mkdir creates a fresh owner, never reuses one.
            if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } == 0 {
                created = Some(name);
                break;
            }
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST) {
                return Err(CookieImportError::StorageUnavailable);
            }
        }
        let owner_name = created.ok_or(CookieImportError::StorageUnavailable)?;
        // SAFETY: new private owner entry opened without symlink following.
        let owner = descriptor(
            unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    owner_name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            },
            CookieImportError::StorageUnavailable,
        )?;
        let meta = owner
            .metadata()
            .map_err(|_| CookieImportError::StorageUnavailable)?;
        let owner_identity = Identity::of(&meta);
        let mut owned = OwnedCopy {
            cache_path: SensitivePath(cache.to_owned()),
            parent,
            parent_identity,
            owner,
            owner_identity,
            owner_name,
            file: None,
            removed: false,
            #[cfg(target_os = "linux")]
            records: None,
            #[cfg(target_os = "linux")]
            journal: None,
        };
        if !meta.is_dir() || !private(&meta, 0o700, owner_identity) {
            return Err(CookieImportError::UnsafeCache);
        }
        // SAFETY: create-new in verified private owner; no final symlink/hardlink reuse.
        let file = descriptor(
            unsafe {
                libc::openat(
                    owned.owner.as_raw_fd(),
                    FILE_NAME.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    0o600 as libc::c_uint,
                )
            },
            CookieImportError::StorageUnavailable,
        )?;
        let meta = file
            .metadata()
            .map_err(|_| CookieImportError::StorageUnavailable)?;
        let file_identity = Identity::of(&meta);
        owned.file = Some((file, file_identity));
        if !meta.is_file() || !private(&meta, 0o600, file_identity) || meta.nlink() != 1 {
            return Err(CookieImportError::UnsafeCache);
        }
        let output = &mut owned
            .file
            .as_mut()
            .ok_or(CookieImportError::StorageUnavailable)?
            .0;
        output
            .write_all(&data.0)
            .map_err(|_| CookieImportError::StorageUnavailable)?;
        output
            .flush()
            .map_err(|_| CookieImportError::StorageUnavailable)?;
        owned.owner_valid()?;
        owned.file_valid()?;
        #[cfg(target_os = "linux")]
        {
            match super::reader_journal::ReaderJournal::create(
                owned
                    .owner
                    .try_clone()
                    .map_err(|_| CookieImportError::StorageUnavailable)?,
            ) {
                Ok(journal) => owned.journal = Some(Mutex::new(journal)),
                Err(_) => {
                    owned.defer_cleanup();
                    return Err(CookieImportError::StorageUnavailable);
                }
            }
            // Partial record creation must never trigger name-based cleanup.
            let records = match recovery::Records::create(&owned) {
                Ok(records) => records,
                Err(error) => {
                    owned.defer_cleanup();
                    return Err(error);
                }
            };
            owned.records = Some(records);
            owned.validate_for_tool()?;
        }
        let path = cache
            .join(std::ffi::OsStr::from_bytes(owned.owner_name.as_bytes()))
            .join("cookies.txt");
        Ok(SessionCookieLease {
            path: SensitivePath(path),
            entries,
            bytes: data.0.len(),
            reader_pending: AtomicBool::new(false),
            owned: Some(owned),
        })
    }

    #[cfg(target_os = "linux")]
    pub(super) mod recovery {
        use super::*;
        use serde::{Deserialize, Serialize};
        use sha2::{Digest, Sha256};
        const RECORD: &CStr = c"recovery.json";
        const MARKER: &CStr = c"owner.marker";
        const RECORD_LIMIT: usize = 4096;
        const MARKER_LIMIT: usize = 512;
        #[derive(Serialize, Deserialize, Eq, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct JournalBinding {
            directory_dev: u64,
            directory_ino: u64,
            record_dev: u64,
            record_ino: u64,
        }
        impl JournalBinding {
            fn from_anchor(value: crate::cookies::reader_journal::JournalAnchor) -> Self {
                Self {
                    directory_dev: value.directory_dev,
                    directory_ino: value.directory_ino,
                    record_dev: value.record_dev,
                    record_ino: value.record_ino,
                }
            }
            fn anchor(&self) -> crate::cookies::reader_journal::JournalAnchor {
                crate::cookies::reader_journal::JournalAnchor {
                    directory_dev: self.directory_dev,
                    directory_ino: self.directory_ino,
                    record_dev: self.record_dev,
                    record_ino: self.record_ino,
                }
            }
        }
        #[derive(Serialize, Deserialize, Eq, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Record {
            version: u8,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            journal: Option<JournalBinding>,
            boot: String,
            nonce: String,
            cache: Identity,
            owner: Identity,
            cookie: Identity,
            record: Identity,
            marker: Identity,
        }
        #[derive(Serialize, Deserialize, Eq, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Marker {
            nonce: String,
            record: Identity,
            digest: String,
        }
        pub(super) struct Records {
            record_file: File,
            marker_file: File,
            record: Record,
            marker: Marker,
        }
        fn changed() -> CookieImportError {
            CookieImportError::CleanupOwnershipChanged
        }
        fn uuid(value: &str) -> bool {
            value.len() == 36
                && value.bytes().enumerate().all(|(i, b)| {
                    if [8, 13, 18, 23].contains(&i) {
                        b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                })
                && value.bytes().any(|b| b != b'0' && b != b'-')
        }
        fn boot() -> Result<String, CookieImportError> {
            let mut file = File::open("/proc/sys/kernel/random/boot_id")
                .map_err(|_| CookieImportError::Unavailable)?;
            let mut bytes = Vec::with_capacity(38);
            Read::by_ref(&mut file)
                .take(38)
                .read_to_end(&mut bytes)
                .map_err(|_| CookieImportError::Unavailable)?;
            if bytes.len() != 37 || bytes[36] != b'\n' {
                return Err(CookieImportError::Unavailable);
            }
            let text =
                std::str::from_utf8(&bytes[..36]).map_err(|_| CookieImportError::Unavailable)?;
            if !uuid(text) {
                return Err(CookieImportError::Unavailable);
            }
            Ok(text.to_ascii_lowercase())
        }
        fn nonce() -> Result<String, CookieImportError> {
            let mut bytes = [0u8; 32];
            let mut used = 0;
            while used < bytes.len() {
                // SAFETY: exclusive bounded output buffer, Linux getrandom without flags.
                let n = unsafe {
                    libc::getrandom(bytes[used..].as_mut_ptr().cast(), bytes.len() - used, 0)
                };
                if n < 0 {
                    if std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                        continue;
                    }
                    return Err(CookieImportError::StorageUnavailable);
                }
                if n == 0 {
                    return Err(CookieImportError::StorageUnavailable);
                }
                used += usize::try_from(n).map_err(|_| CookieImportError::StorageUnavailable)?;
            }
            Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
        }
        pub(super) fn names(
            parent: &File,
            limit: usize,
        ) -> Result<Vec<CString>, CookieImportError> {
            // Linux procfs opens the held directory inode, not a user-controlled path.
            let entries = std::fs::read_dir(format!("/proc/self/fd/{}", parent.as_raw_fd()))
                .map_err(|_| changed())?;
            let mut result = Vec::with_capacity(limit.min(MAX_COOKIE_SESSIONS) + 1);
            for item in entries.take(limit + 1) {
                let item = item.map_err(|_| changed())?;
                result.push(CString::new(item.file_name().as_bytes()).map_err(|_| changed())?);
            }
            Ok(result)
        }
        fn open_file(owner: &File, name: &CStr, create: bool) -> Result<File, CookieImportError> {
            let flags = if create {
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL
            } else {
                libc::O_RDONLY
            };
            // SAFETY: anchored constant basename; no symlink/device blocking; new descriptor adopted.
            descriptor(
                unsafe {
                    libc::openat(
                        owner.as_raw_fd(),
                        name.as_ptr(),
                        flags | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
                        0o600 as libc::c_uint,
                    )
                },
                changed(),
            )
        }
        fn file_identity(file: &File) -> Result<Identity, CookieImportError> {
            let meta = file.metadata().map_err(|_| changed())?;
            let id = Identity::of(&meta);
            if !meta.is_file() || !private(&meta, 0o600, id) || meta.nlink() != 1 {
                return Err(changed());
            }
            Ok(id)
        }
        fn bound(
            owner: &File,
            name: &CStr,
            file: &File,
            id: Identity,
        ) -> Result<(), CookieImportError> {
            let item = entry(owner, name)?.ok_or_else(changed)?;
            if file_identity(file)? != id
                || item.identity != id
                || item.uid != uid()
                || item.mode & 0o7777 != 0o600
                || item.mode & mode(libc::S_IFMT)? != mode(libc::S_IFREG)?
                || item.links != 1
            {
                return Err(changed());
            }
            Ok(())
        }
        fn read<T: serde::de::DeserializeOwned>(
            file: &File,
            limit: usize,
        ) -> Result<T, CookieImportError> {
            use std::os::unix::fs::FileExt;
            let meta = file.metadata().map_err(|_| changed())?;
            if meta.len() > limit as u64 {
                return Err(changed());
            }
            let mut bytes = vec![0; limit + 1];
            let mut used = 0;
            loop {
                let n = file
                    .read_at(&mut bytes[used..], used as u64)
                    .map_err(|_| changed())?;
                if n == 0 {
                    break;
                }
                used += n;
                if used > limit {
                    return Err(changed());
                }
            }
            serde_json::from_slice(&bytes[..used]).map_err(|_| changed())
        }
        fn write<T: Serialize>(
            file: &mut File,
            value: &T,
            limit: usize,
        ) -> Result<(), CookieImportError> {
            let bytes = serde_json::to_vec(value).map_err(|_| changed())?;
            if bytes.len() > limit {
                return Err(changed());
            }
            file.write_all(&bytes)
                .and_then(|_| file.sync_all())
                .map_err(|_| changed())
        }
        fn digest(record: &Record) -> Result<String, CookieImportError> {
            let bytes = serde_json::to_vec(record).map_err(|_| changed())?;
            Ok(format!("{:x}", Sha256::digest(bytes)))
        }
        impl Records {
            fn previous_boot_authority(
                &self,
                current_boot: &str,
            ) -> Result<crate::cookies::reader_journal::PreviousBootAuthority, CookieImportError>
            {
                fn decode<const N: usize>(value: &str) -> Result<[u8; N], CookieImportError> {
                    if value.len() != N * 2 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Err(changed());
                    }
                    let mut result = [0; N];
                    for (index, byte) in result.iter_mut().enumerate() {
                        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
                            .map_err(|_| changed())?;
                    }
                    Ok(result)
                }
                if !uuid(&self.record.boot)
                    || !uuid(current_boot)
                    || self.record.boot.eq_ignore_ascii_case(current_boot)
                {
                    return Err(changed());
                }
                let original: String = self.record.boot.chars().filter(|c| *c != '-').collect();
                let original = decode(&original)?;
                let digest = decode(&self.marker.digest)?;
                #[cfg(not(test))]
                let authority =
                    crate::cookies::reader_journal::PreviousBootAuthority::new(original, digest);
                #[cfg(test)]
                let authority = {
                    let current: String = current_boot.chars().filter(|c| *c != '-').collect();
                    crate::cookies::reader_journal::PreviousBootAuthority::for_test_boot(
                        original,
                        decode(&current)?,
                        digest,
                    )
                };
                authority.map_err(|_| changed())
            }
            pub(super) fn reader_seed(&self) -> Result<ReaderSeed, CookieImportError> {
                if boot()? != self.record.boot {
                    return Err(changed());
                }
                fn decode<const N: usize>(value: &str) -> Result<[u8; N], CookieImportError> {
                    if value.len() != N * 2 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
                        return Err(changed());
                    }
                    let mut bytes = [0; N];
                    for (index, byte) in bytes.iter_mut().enumerate() {
                        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
                            .map_err(|_| changed())?;
                    }
                    if bytes.iter().all(|b| *b == 0) {
                        return Err(changed());
                    }
                    Ok(bytes)
                }
                let nonce = nonce()?;
                let boot: String = self.record.boot.chars().filter(|c| *c != '-').collect();
                Ok(ReaderSeed {
                    nonce: decode(&nonce[..32])?,
                    digest: decode(&self.marker.digest)?,
                    boot: decode(&boot)?,
                })
            }
            pub(super) fn create(owned: &OwnedCopy) -> Result<Self, CookieImportError> {
                let current_boot = boot()?;
                let token = nonce()?;
                let mut record_file = open_file(&owned.owner, RECORD, true)?;
                let mut marker_file = open_file(&owned.owner, MARKER, true)?;
                let record_id = file_identity(&record_file)?;
                let marker_id = file_identity(&marker_file)?;
                let cookie = owned.file.as_ref().ok_or_else(changed)?.1;
                let record = Record {
                    version: if owned.journal.is_some() { 2 } else { 1 },
                    journal: owned
                        .journal
                        .as_ref()
                        .map(|value| {
                            value
                                .try_lock()
                                .map(|journal| JournalBinding::from_anchor(journal.anchor()))
                                .map_err(|_| changed())
                        })
                        .transpose()?,
                    boot: current_boot,
                    nonce: token.clone(),
                    cache: owned.parent_identity,
                    owner: owned.owner_identity,
                    cookie,
                    record: record_id,
                    marker: marker_id,
                };
                let marker = Marker {
                    nonce: token,
                    record: record_id,
                    digest: digest(&record)?,
                };
                owned
                    .file
                    .as_ref()
                    .ok_or_else(changed)?
                    .0
                    .sync_all()
                    .map_err(|_| changed())?;
                write(&mut record_file, &record, RECORD_LIMIT)?;
                write(&mut marker_file, &marker, MARKER_LIMIT)?;
                owned
                    .owner
                    .sync_all()
                    .and_then(|_| owned.parent.sync_all())
                    .map_err(|_| changed())?;
                Ok(Self {
                    record_file,
                    marker_file,
                    record,
                    marker,
                })
            }
            pub(super) fn validate(&self, owned: &OwnedCopy) -> Result<(), CookieImportError> {
                let mut allowed = vec![FILE_NAME, RECORD, MARKER];
                match (&self.record.journal, &owned.journal) {
                    (Some(expected), Some(journal)) => {
                        let journal = journal.try_lock().map_err(|_| changed())?;
                        if expected.anchor() != journal.anchor() {
                            return Err(changed());
                        }
                        allowed.extend(journal.owned_names().map_err(|_| changed())?);
                    }
                    (None, None) => {}
                    _ => return Err(changed()),
                }
                let names = names(&owned.owner, allowed.len())?;
                if names.len() != allowed.len()
                    || names.iter().any(|name| !allowed.contains(&name.as_c_str()))
                {
                    return Err(changed());
                }
                self.validate_binding(owned)
            }
            fn validate_binding(&self, owned: &OwnedCopy) -> Result<(), CookieImportError> {
                bound(&owned.owner, RECORD, &self.record_file, self.record.record)?;
                bound(&owned.owner, MARKER, &self.marker_file, self.record.marker)?;
                if read::<Record>(&self.record_file, RECORD_LIMIT)? != self.record
                    || read::<Marker>(&self.marker_file, MARKER_LIMIT)? != self.marker
                    || self.record.version != if self.record.journal.is_some() { 2 } else { 1 }
                    || !uuid(&self.record.boot)
                    || self.record.nonce.len() != 64
                    || !self.record.nonce.bytes().all(|b| b.is_ascii_hexdigit())
                    || self.record.nonce != self.marker.nonce
                    || self.marker.record != self.record.record
                    || self.marker.digest != digest(&self.record)?
                    || self.record.cache != owned.parent_identity
                    || self.record.owner != owned.owner_identity
                    || Some(self.record.cookie) != owned.file.as_ref().map(|f| f.1)
                {
                    return Err(changed());
                }
                Ok(())
            }
            pub(super) fn remove(&self, owned: &OwnedCopy) -> Result<(), CookieImportError> {
                for (name, file, id) in [
                    (RECORD, &self.record_file, self.record.record),
                    (MARKER, &self.marker_file, self.record.marker),
                ] {
                    owned.owner_valid()?;
                    bound(&owned.owner, name, file, id)?;
                    // SAFETY: exact anchored validated owned record/marker; never recursive.
                    if unsafe { libc::unlinkat(owned.owner.as_raw_fd(), name.as_ptr(), 0) } < 0 {
                        return Err(changed());
                    }
                }
                owned.owner.sync_all().map_err(|_| changed())
            }
        }
        pub(in crate::cookies) fn recover(
            cache: &Path,
        ) -> Result<CookieRecoverySummary, CookieImportError> {
            recover_with_boot(cache, &boot()?)
        }
        fn recover_with_boot(
            cache: &Path,
            current_boot: &str,
        ) -> Result<CookieRecoverySummary, CookieImportError> {
            if !uuid(current_boot) {
                return Err(CookieImportError::Unavailable);
            }
            let parent = directory(cache, CookieImportError::UnsafeCache)?;
            let meta = parent
                .metadata()
                .map_err(|_| CookieImportError::UnsafeCache)?;
            let parent_id = Identity::of(&meta);
            if !meta.is_dir() || !private(&meta, 0o700, parent_id) {
                return Err(CookieImportError::UnsafeCache);
            }
            let entries = names(&parent, MAX_COOKIE_SESSIONS)?;
            let mut summary = CookieRecoverySummary {
                truncated: entries.len() > MAX_COOKIE_SESSIONS,
                ..Default::default()
            };
            for name in entries.into_iter().take(MAX_COOKIE_SESSIONS) {
                summary.scanned += 1;
                match recover_one(cache, &parent, parent_id, name, current_boot) {
                    Ok(true) => summary.recovered += 1,
                    Ok(false) => summary.deferred += 1,
                    Err(_) => summary.rejected += 1,
                }
            }
            Ok(summary)
        }
        fn recover_one(
            cache: &Path,
            parent: &File,
            parent_id: Identity,
            name: CString,
            boot: &str,
        ) -> Result<bool, CookieImportError> {
            // SAFETY: bounded enumerated single component; anchored nofollow directory only.
            let owner = descriptor(
                unsafe {
                    libc::openat(
                        parent.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                    )
                },
                changed(),
            )?;
            let meta = owner.metadata().map_err(|_| changed())?;
            let owner_id = Identity::of(&meta);
            if !meta.is_dir() || !private(&meta, 0o700, owner_id) {
                return Err(changed());
            }
            let record_file = open_file(&owner, RECORD, false)?;
            let marker_file = open_file(&owner, MARKER, false)?;
            let record: Record = read(&record_file, RECORD_LIMIT)?;
            let marker: Marker = read(&marker_file, MARKER_LIMIT)?;
            let file = open_file(&owner, FILE_NAME, false)?;
            let cookie_id = file_identity(&file)?;
            let records = Records {
                record_file,
                marker_file,
                record,
                marker,
            };
            let mut owned = OwnedCopy {
                cache_path: SensitivePath(cache.to_owned()),
                parent: parent.try_clone().map_err(|_| changed())?,
                parent_identity: parent_id,
                owner,
                owner_identity: owner_id,
                owner_name: name,
                file: Some((file, cookie_id)),
                removed: true,
                records: Some(records),
                journal: None,
            };
            owned.owner_valid()?;
            owned.file_valid()?;
            owned
                .records
                .as_ref()
                .ok_or_else(changed)?
                .validate_binding(&owned)?;
            owned.journal = match owned
                .records
                .as_ref()
                .ok_or_else(changed)?
                .record
                .journal
                .as_ref()
            {
                Some(binding) => match crate::cookies::reader_journal::ReaderJournal::reopen(
                    owned.owner.try_clone().map_err(|_| changed())?,
                    binding.anchor(),
                ) {
                    Ok(journal) => Some(Mutex::new(journal)),
                    Err(crate::cookies::reader_journal::JournalError::Busy) => return Ok(false),
                    Err(_) => return Err(changed()),
                },
                None => None,
            };
            owned
                .records
                .as_ref()
                .ok_or_else(changed)?
                .validate(&owned)?;
            let same_boot = owned
                .records
                .as_ref()
                .ok_or_else(changed)?
                .record
                .boot
                .eq_ignore_ascii_case(boot);
            let previous_boot = if same_boot {
                // Empty journals from earlier ungated product builds are not
                // same-boot reader termination evidence. At least one original
                // bound identity and all independent receipts must be proven.
                let Some(journal) = &owned.journal else {
                    return Ok(false);
                };
                let recovered = journal
                    .try_lock()
                    .map_err(|_| changed())?
                    .recover_completed_readers()
                    .map_err(|_| changed())?;
                if !recovered {
                    return Ok(false);
                }
                None
            } else {
                Some(
                    owned
                        .records
                        .as_ref()
                        .ok_or_else(changed)?
                        .previous_boot_authority(boot)?,
                )
            };
            owned.removed = false;
            match owned.cleanup_inner(previous_boot.as_ref()) {
                Ok(()) => Ok(true),
                Err(e) => {
                    owned.defer_cleanup();
                    Err(e)
                }
            }
        }

        #[cfg(test)]
        mod recovery_tests {
            use super::*;
            use std::os::unix::fs::PermissionsExt;
            fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
                let temp = tempfile::tempdir().unwrap();
                let source = temp.path().join("synthetic-source");
                std::fs::write(&source,b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-secret\n").unwrap();
                let cache = temp.path().join("private-cache");
                std::fs::create_dir(&cache).unwrap();
                std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o700)).unwrap();
                (temp, source, cache)
            }
            fn next_boot() -> String {
                let mut actual = boot().unwrap().into_bytes();
                actual[0] = if actual[0] == b'a' { b'b' } else { b'a' };
                String::from_utf8(actual).unwrap()
            }
            #[test]
            fn prior_boot_recovers_original_intent_and_bound_both_slots() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                for index in 0..2 {
                    for bound in [false, true] {
                        let (_temp, source, cache) = fixture();
                        let source_bytes = std::fs::read(&source).unwrap();
                        let mut lease = import(&source, &cache).unwrap();
                        let copy = lease.path_for_tool().to_owned();
                        if index == 1 {
                            use crate::process::reader_protocol::{
                                CleanupReceipt, encode_cleanup_receipt,
                            };
                            let mut metadata = lease.begin_gated_reader(0).unwrap();
                            let (mut launch, _) = metadata.take_launch().unwrap();
                            let mut identity = crate::process::reader_protocol::ReaderIdentity {
                                capability: launch.capability.clone(),
                                identity_digest: [0; 32],
                                unit: [4; 16],
                                invocation: [5; 16],
                                cgroup_mount: 6,
                                cgroup_dev: 7,
                                cgroup_ino: 8,
                                path: "/user.slice/synthetic-completed-metadata.service".into(),
                            };
                            identity.identity_digest =
                                crate::cookies::reader_identity_digest(&identity).unwrap();
                            metadata.bind_identity(&identity).unwrap();
                            launch
                                .writer
                                .write_all(
                                    &encode_cleanup_receipt(&CleanupReceipt::from_identity(
                                        &identity,
                                    ))
                                    .unwrap(),
                                )
                                .unwrap();
                            launch.writer.sync_all().unwrap();
                            metadata.confirm_terminal().unwrap();
                        }
                        {
                            let mut reader = lease.begin_gated_reader(index).unwrap();
                            let (launch, _) = reader.take_launch().unwrap();
                            if bound {
                                let mut identity =
                                    crate::process::reader_protocol::ReaderIdentity {
                                        capability: launch.capability.clone(),
                                        identity_digest: [0; 32],
                                        unit: [4; 16],
                                        invocation: [5; 16],
                                        cgroup_mount: 6,
                                        cgroup_dev: 7,
                                        cgroup_ino: 8,
                                        path: "/user.slice/synthetic-previous-boot.service".into(),
                                    };
                                identity.identity_digest =
                                    crate::cookies::reader_identity_digest(&identity).unwrap();
                                reader.bind_identity(&identity).unwrap();
                            }
                        }
                        lease.defer_cleanup();
                        drop(lease);
                        let same = recover_with_boot(&cache, &boot().unwrap()).unwrap();
                        assert_eq!((same.recovered, same.deferred, same.rejected), (0, 1, 0));
                        assert!(copy.exists());
                        let previous = recover_with_boot(&cache, &next_boot()).unwrap();
                        assert_eq!(
                            (previous.recovered, previous.deferred, previous.rejected),
                            (1, 0, 0)
                        );
                        assert!(!copy.exists());
                        assert_eq!(std::fs::read(&source).unwrap(), source_bytes);
                    }
                }
            }
            #[test]
            fn active_journal_lock_defers_recovery_without_mutating_copy() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                let (_temp, source, cache) = fixture();
                let mut lease = import(&source, &cache).unwrap();
                let copy = lease.path_for_tool().to_owned();
                let before = std::fs::read(&copy).unwrap();
                let summary = recover_with_boot(&cache, &boot().unwrap()).unwrap();
                assert_eq!(
                    (summary.recovered, summary.deferred, summary.rejected),
                    (0, 1, 0)
                );
                assert_eq!(std::fs::read(&copy).unwrap(), before);
                assert!(lease.validate_for_tool().is_ok());
                lease.cleanup().unwrap();
                assert!(!copy.exists());
                assert!(source.exists());
            }
            #[test]
            fn busy_journal_does_not_hide_hardlinked_recovery_record() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                let (_temp, source, cache) = fixture();
                let mut lease = import(&source, &cache).unwrap();
                let copy = lease.path_for_tool().to_owned();
                let before = std::fs::read(&copy).unwrap();
                let source_before = std::fs::read(&source).unwrap();
                let record = copy.parent().unwrap().join("recovery.json");
                let link = source.with_file_name("foreign-record-link");
                std::fs::hard_link(&record, &link).unwrap();
                let record_before = std::fs::read(&link).unwrap();
                // The live lease holds the journal flock throughout this recovery.
                let summary = recover_with_boot(&cache, &next_boot()).unwrap();
                assert_eq!(
                    (
                        summary.scanned,
                        summary.recovered,
                        summary.deferred,
                        summary.rejected
                    ),
                    (1, 0, 0, 1)
                );
                assert_eq!(std::fs::read(&copy).unwrap(), before);
                assert_eq!(std::fs::read(&source).unwrap(), source_before);
                assert_eq!(std::fs::read(&link).unwrap(), record_before);
                lease.defer_cleanup();
            }
            #[test]
            fn journal_anchor_and_unused_receipt_changes_preserve_cookie() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                for change in ["replace-journal", "unused-receipt"] {
                    let (_temp, source, cache) = fixture();
                    let source_bytes = std::fs::read(&source).unwrap();
                    let mut lease = import(&source, &cache).unwrap();
                    let copy = lease.path_for_tool().to_owned();
                    let owner = copy.parent().unwrap();
                    let records = lease.owned.as_ref().unwrap().records.as_ref().unwrap();
                    assert_eq!(records.record.version, 2);
                    assert!(records.record.journal.is_some());
                    if change == "replace-journal" {
                        let journal = owner.join("recordstate.json");
                        std::fs::rename(&journal, source.with_file_name("original-journal"))
                            .unwrap();
                        std::fs::write(&journal, b"foreign sentinel").unwrap();
                        std::fs::set_permissions(&journal, std::fs::Permissions::from_mode(0o600))
                            .unwrap();
                    } else {
                        std::fs::write(owner.join("reader1.receipt"), b"foreign sentinel").unwrap();
                    }
                    assert!(lease.validate_for_tool().is_err());
                    assert!(lease.cleanup().is_err());
                    assert!(copy.exists());
                    lease.defer_cleanup();
                    assert_eq!(
                        recover_with_boot(&cache, &next_boot()).unwrap().recovered,
                        0
                    );
                    assert!(copy.exists());
                    assert_eq!(std::fs::read(&source).unwrap(), source_bytes);
                }
            }
            #[test]
            fn legacy_version_one_without_journal_still_recovers_after_boot() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                let (_temp, source, cache) = fixture();
                let mut lease = import(&source, &cache).unwrap();
                let owner = lease.path_for_tool().parent().unwrap().to_owned();
                let owned = lease.owned.as_mut().unwrap();
                owned
                    .journal
                    .take()
                    .unwrap()
                    .into_inner()
                    .unwrap()
                    .remove_owned_files()
                    .unwrap();
                let records = owned.records.as_mut().unwrap();
                records.record.version = 1;
                records.record.journal = None;
                records.marker.digest = digest(&records.record).unwrap();
                records.record_file.set_len(0).unwrap();
                std::io::Seek::rewind(&mut records.record_file).unwrap();
                write(&mut records.record_file, &records.record, RECORD_LIMIT).unwrap();
                records.marker_file.set_len(0).unwrap();
                std::io::Seek::rewind(&mut records.marker_file).unwrap();
                write(&mut records.marker_file, &records.marker, MARKER_LIMIT).unwrap();
                owned.owner.sync_all().unwrap();
                assert!(lease.validate_for_tool().is_ok());
                lease.defer_cleanup();
                assert_eq!(
                    recover_with_boot(&cache, &next_boot()).unwrap().recovered,
                    1
                );
                assert!(!owner.exists());
                assert!(source.exists());
            }
            #[test]
            fn previous_boot_recovery_removes_only_complete_owned_record() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                let (_temp, source, cache) = fixture();
                let source_bytes = std::fs::read(&source).unwrap();
                let mut lease = import(&source, &cache).unwrap();
                let copy = lease.path_for_tool().to_owned();
                let owner = copy.parent().unwrap().to_owned();
                lease.defer_cleanup();
                let summary = recover_with_boot(&cache, &next_boot()).unwrap();
                assert_eq!(
                    (
                        summary.scanned,
                        summary.recovered,
                        summary.deferred,
                        summary.rejected
                    ),
                    (1, 1, 0, 0)
                );
                assert!(!owner.exists());
                assert_eq!(std::fs::read(&source).unwrap(), source_bytes);
                assert_eq!(recover_with_boot(&cache, &next_boot()).unwrap().scanned, 0);
            }
            #[test]
            fn changed_token_boot_identity_and_partial_records_are_preserved() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                for change in [
                    "nonce", "boot", "cookie", "marker", "partial", "extra", "oversize", "symlink",
                    "mode", "hardlink", "replace",
                ] {
                    let (_temp, source, cache) = fixture();
                    let mut lease = import(&source, &cache).unwrap();
                    let copy = lease.path_for_tool().to_owned();
                    let owner = copy.parent().unwrap();
                    let record_path = owner.join("recovery.json");
                    lease.defer_cleanup();
                    if change == "partial" {
                        std::fs::write(&record_path, b"{").unwrap();
                    } else if change == "extra" {
                        std::fs::write(owner.join("foreign-sentinel"), b"foreign").unwrap();
                    } else if change == "oversize" {
                        std::fs::write(&record_path, vec![b' '; 4097]).unwrap();
                    } else if change == "symlink" {
                        std::fs::remove_file(&record_path).unwrap();
                        std::os::unix::fs::symlink(&source, &record_path).unwrap();
                    } else if change == "mode" {
                        std::fs::set_permissions(
                            &record_path,
                            std::fs::Permissions::from_mode(0o644),
                        )
                        .unwrap();
                    } else if change == "hardlink" {
                        std::fs::hard_link(&record_path, source.with_file_name("owned-link"))
                            .unwrap();
                    } else if change == "replace" {
                        let bytes = std::fs::read(&record_path).unwrap();
                        std::fs::rename(&record_path, source.with_file_name("saved-record"))
                            .unwrap();
                        std::fs::write(&record_path, bytes).unwrap();
                        std::fs::set_permissions(
                            &record_path,
                            std::fs::Permissions::from_mode(0o600),
                        )
                        .unwrap();
                    } else {
                        let mut value: serde_json::Value =
                            serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
                        match change {
                            "nonce" => value["nonce"] = "0".repeat(64).into(),
                            "boot" => value["boot"] = "not-a-boot".into(),
                            "cookie" => value["cookie"]["inode"] = 0.into(),
                            _ => value["marker"]["inode"] = 0.into(),
                        };
                        std::fs::write(&record_path, serde_json::to_vec(&value).unwrap()).unwrap();
                    }
                    let summary = recover_with_boot(&cache, &next_boot()).unwrap();
                    assert_eq!((summary.recovered, summary.rejected), (0, 1), "{change}");
                    assert!(copy.exists(), "{change}");
                    assert!(source.exists());
                }
            }
            #[test]
            fn actual_cookie_owner_and_cache_replacement_is_preserved() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                for target in ["cookie", "owner", "cache"] {
                    let (_temp, source, cache) = fixture();
                    let mut lease = import(&source, &cache).unwrap();
                    let copy = lease.path_for_tool().to_owned();
                    let owner = copy.parent().unwrap().to_owned();
                    lease.defer_cleanup();
                    if target == "cookie" {
                        std::fs::rename(&copy, source.with_file_name("saved-copy")).unwrap();
                        std::fs::write(&copy, b"foreign sentinel").unwrap();
                        std::fs::set_permissions(&copy, std::fs::Permissions::from_mode(0o600))
                            .unwrap();
                    } else {
                        let changed = if target == "owner" { &owner } else { &cache };
                        let saved = source.with_file_name("saved-directory");
                        std::fs::rename(changed, &saved).unwrap();
                        std::fs::create_dir(changed).unwrap();
                        std::fs::set_permissions(changed, std::fs::Permissions::from_mode(0o700))
                            .unwrap();
                        if target == "owner" {
                            for name in ["cookies.txt", "recovery.json", "owner.marker"] {
                                std::fs::rename(saved.join(name), owner.join(name)).unwrap();
                            }
                        } else {
                            std::fs::rename(saved.join(owner.file_name().unwrap()), &owner)
                                .unwrap();
                        }
                    }
                    let summary = recover_with_boot(&cache, &next_boot()).unwrap();
                    assert_eq!((summary.recovered, summary.rejected), (0, 1), "{target}");
                    assert!(copy.exists());
                    assert!(source.exists());
                }
            }
            #[test]
            fn unknown_crash_owner_and_invalid_current_boot_are_preserved() {
                if crate::process::reader_protocol::run_isolated_lock_test() {
                    return;
                }
                let (_temp, source, cache) = fixture();
                let owner = cache.join("arbitrary-foreign-name");
                std::fs::create_dir(&owner).unwrap();
                std::fs::set_permissions(&owner, std::fs::Permissions::from_mode(0o700)).unwrap();
                let sentinel = owner.join("cookies.txt");
                std::fs::write(&sentinel, b"foreign").unwrap();
                assert_eq!(
                    recover_with_boot(&cache, "bad-boot"),
                    Err(CookieImportError::Unavailable)
                );
                let summary = recover_with_boot(&cache, &next_boot()).unwrap();
                assert_eq!(summary.rejected, 1);
                assert_eq!(std::fs::read(&sentinel).unwrap(), b"foreign");
                assert!(source.exists());
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn secret_path_debug_is_redacted() {
            let path = SensitivePath(PathBuf::from("/synthetic-sensitive-source"));
            assert_eq!(format!("{path:?}"), "<redacted>");
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod gated_reader_tests {
    use super::*;
    use crate::process::reader_protocol::{CleanupReceipt, ReaderIdentity, encode_cleanup_receipt};
    use std::{io::Write, os::unix::fs::PermissionsExt};
    fn fixture() -> (tempfile::TempDir, SessionCookieLease) {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        std::fs::write(&source, b"# Netscape HTTP Cookie File\n.fixture.invalid\tTRUE\t/\tTRUE\t0\tS\tsynthetic-secret\n").unwrap();
        let cache = temp.path().join("cache");
        std::fs::create_dir(&cache).unwrap();
        std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o700)).unwrap();
        let lease = import_session(&source, &cache).unwrap();
        (temp, lease)
    }
    fn identity(launch: &ReaderLaunch) -> ReaderIdentity {
        let mut identity = ReaderIdentity {
            capability: launch.capability.clone(),
            identity_digest: [1; 32],
            unit: [2; 16],
            invocation: [3; 16],
            cgroup_mount: 4,
            cgroup_dev: 5,
            cgroup_ino: 6,
            path: "/synthetic.slice".into(),
        };
        identity.identity_digest = reader_identity_digest(&identity).unwrap();
        identity
    }
    #[test]
    fn two_slots_resolve_actual_receipts_before_clearing_pending() {
        let (_temp, mut lease) = fixture();
        for slot in [0, 1] {
            let mut guard = lease.begin_gated_reader(slot).unwrap();
            assert!(lease.reader_pending.load(Ordering::Acquire));
            let (mut launch, _) = guard.take_launch().unwrap();
            assert!(guard.take_launch().is_err());
            let identity = identity(&launch);
            guard.bind_identity(&identity).unwrap();
            assert!(guard.confirm_terminal().is_err());
            assert!(lease.reader_pending.load(Ordering::Acquire));
            launch
                .writer
                .write_all(
                    &encode_cleanup_receipt(&CleanupReceipt::from_identity(&identity)).unwrap(),
                )
                .unwrap();
            launch.writer.sync_all().unwrap();
            guard.confirm_terminal().unwrap();
            assert!(!lease.reader_pending.load(Ordering::Acquire));
            assert!(guard.confirm_terminal().is_err());
        }
        lease.cleanup().unwrap();
    }
    #[test]
    fn wrong_capability_and_guard_drop_preserve_cookie_and_intent() {
        let (_temp, mut lease) = fixture();
        let path = lease.path_for_tool().to_owned();
        let mut guard = lease.begin_gated_reader(0).unwrap();
        let (launch, _) = guard.take_launch().unwrap();
        let mut wrong = identity(&launch);
        wrong.capability.nonce = [9; 16];
        assert!(guard.bind_identity(&wrong).is_err());
        drop(launch);
        drop(guard);
        assert!(lease.begin_gated_reader(0).is_err());
        assert_eq!(lease.cleanup(), Err(CookieImportError::ReaderUnconfirmed));
        assert!(path.is_file());
        assert!(path.parent().unwrap().join("recordstate.json").is_file());
    }
    #[test]
    fn busy_and_poisoned_journal_never_wait_or_delete_cookie() {
        let (_temp, lease) = fixture();
        let path = lease.path_for_tool().to_owned();
        let journal = lease.owned.as_ref().unwrap().reader_seed().unwrap().0;
        let locked = journal.try_lock().unwrap();
        assert!(lease.begin_gated_reader(0).is_err());
        assert!(!lease.reader_pending.load(Ordering::Acquire));
        drop(locked);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _locked = journal.try_lock().unwrap();
            panic!("synthetic mutex poison");
        }));
        assert!(lease.begin_gated_reader(0).is_err());
        drop(lease);
        assert!(path.is_file());
    }
}
