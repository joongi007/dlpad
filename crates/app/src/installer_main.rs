//! Native unsigned development bootstrap. No automatic install, repair, launch or deletion.
#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
#[allow(dead_code)]
mod catalog;
#[cfg(windows)]
mod folder_picker;
#[cfg(windows)]
mod installer_catalog;
#[cfg(windows)]
mod installer_defaults;
#[cfg(windows)]
mod installer_file_picker;
#[cfg(any(windows, test))]
mod installer_journal_choice;
#[cfg(windows)]
mod installer_launch;
#[cfg(windows)]
mod installer_os;
#[cfg(windows)]
mod installer_tools;
#[cfg(windows)]
mod installer_uninstall;
#[cfg(windows)]
mod installer_uninstall_catalog;
#[cfg(windows)]
#[allow(dead_code)]
mod managed;
#[cfg(windows)]
mod managed_candidates;
#[cfg(windows)]
mod ui_tokens;
use std::{ffi::OsString, path::PathBuf};
#[cfg_attr(not(windows), allow(dead_code))]
#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct InitialPaths([PathBuf; 4], Option<PathBuf>, bool);
fn parse(args: &[OsString]) -> Result<InitialPaths, ()> {
    if args.is_empty() {
        return Ok(InitialPaths::default());
    }
    if !matches!(args.len(), 2 | 8 | 10) {
        return Err(());
    }
    let mut paths: [Option<PathBuf>; 4] = Default::default();
    let mut tool_data = None;
    for pair in args.chunks_exact(2) {
        let index = match pair[0].to_str() {
            Some("--manifest") => 0,
            Some("--source-dir") => 1,
            Some("--bundle-root") => 2,
            Some("--journal-root") => 3,
            Some("--tool-data") => 4,
            _ => return Err(()),
        };
        if index == 4 {
            if tool_data.is_some() {
                return Err(());
            }
            tool_data = Some(local_path(&pair[1])?);
            continue;
        }
        if paths[index].is_some() {
            return Err(());
        }
        paths[index] = Some(local_path(&pair[1])?);
    }
    let filled = paths.iter().filter(|p| p.is_some()).count();
    if filled != 0 && filled != 4 {
        return Err(());
    }
    Ok(InitialPaths(
        paths.map(|p| p.unwrap_or_default()),
        tool_data,
        false,
    ))
}
/// Source discovery is input convenience, not publisher trust or installation authority.
/// Explicit argv always retains the existing developer contract.
fn initial_for_executable(
    args: &[OsString],
    executable: &std::path::Path,
) -> Result<InitialPaths, ()> {
    let mut initial = parse(args)?;
    if !args.is_empty() {
        return Ok(initial);
    }
    let parent = executable.parent().ok_or(())?;
    let source = local_path(parent.join("payload").as_os_str())?;
    let manifest = source.join("manifest.json");
    match std::fs::symlink_metadata(&manifest) {
        Ok(metadata) if metadata.file_type().is_file() => {
            initial.0[0] = local_path(manifest.as_os_str())?;
            initial.0[1] = source;
            initial.2 = true;
            Ok(initial)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(initial),
        _ => Err(()),
    }
}
/// Derivation is a path hint. The worker verifies existing parent/namespace and fresh roots.
fn packaged_journal(target: &std::path::Path) -> Result<PathBuf, ()> {
    local_path(target.as_os_str())?;
    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or(())?;
    let leaf = target.file_name().and_then(|v| v.to_str()).ok_or(())?;
    if leaf.is_empty()
        || leaf.len() > 192
        || leaf.ends_with(['.', ' '])
        || leaf.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
        })
    {
        return Err(());
    }
    let device = leaf.split('.').next().unwrap_or("").to_ascii_uppercase();
    if matches!(device.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (device.len() == 4
            && (device.starts_with("COM") || device.starts_with("LPT"))
            && matches!(device.as_bytes()[3], b'1'..=b'9'))
    {
        return Err(());
    }
    local_path(parent.join(format!(".{leaf}-install-journal")).as_os_str())
}
fn local_path(value: &std::ffi::OsStr) -> Result<PathBuf, ()> {
    let path = PathBuf::from(value);
    if value.as_encoded_bytes().len() > 131064
        || value.to_string_lossy().chars().any(char::is_control)
        || !path.is_absolute()
        || path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(());
    }
    #[cfg(windows)]
    {
        use std::{
            os::windows::ffi::OsStrExt,
            path::{Component, Prefix},
        };
        let units: Vec<_> = value.encode_wide().collect();
        if String::from_utf16(&units).is_err()
            || units.len() > 32767
            || units.contains(&0)
            || !matches!(path.components().next(),Some(Component::Prefix(v)) if matches!(v.kind(),Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
        {
            return Err(());
        }
    }
    Ok(path)
}
#[cfg_attr(not(windows), allow(dead_code))]
fn apply_recommended_target(
    initial: &mut InitialPaths,
    recommend: impl FnOnce() -> Option<PathBuf>,
) {
    if !initial.2 || !initial.0[2].as_os_str().is_empty() || !initial.0[3].as_os_str().is_empty() {
        return;
    }
    if let Some(target) = recommend()
        && let Ok(journal) = packaged_journal(&target)
    {
        initial.0[2] = target;
        initial.0[3] = journal;
    }
}
#[cfg(any(windows, test))]
mod admission {
    use std::{
        io,
        path::{Component, Path, PathBuf},
        sync::atomic::{AtomicBool, Ordering},
        time::Instant,
    };
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum Phase {
        Unknown,
        Pending,
        Vacant,
        Existing,
        Unavailable,
    }
    impl Phase {
        pub(super) fn permits_install(self) -> bool {
            self == Self::Vacant
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(super) struct Snapshot {
        pub revision: u64,
        pub target: PathBuf,
        pub journal: PathBuf,
    }
    pub(super) fn current(
        snapshot: &Snapshot,
        revision: u64,
        roots: Option<(PathBuf, PathBuf)>,
    ) -> bool {
        snapshot.revision == revision
            && roots.is_some_and(|(target, journal)| {
                snapshot.target == target && snapshot.journal == journal
            })
    }
    fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        if cancel.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if Instant::now() >= deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    pub(super) fn exists(path: &Path, cancel: &AtomicBool, deadline: Instant) -> io::Result<bool> {
        if !path.is_absolute() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let parts: Vec<_> = path.components().collect();
        if parts.len() > 256 || parts.iter().any(|p| matches!(p, Component::ParentDir)) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let mut current = PathBuf::new();
        for (index, part) in parts.iter().enumerate() {
            checkpoint(cancel, deadline)?;
            current.push(part.as_os_str());
            if matches!(part, Component::Prefix(_)) {
                continue;
            }
            match std::fs::symlink_metadata(&current) {
                Ok(meta) => {
                    if meta.file_type().is_symlink() {
                        return Err(io::ErrorKind::InvalidInput.into());
                    }
                    #[cfg(windows)]
                    {
                        use std::os::windows::fs::MetadataExt;
                        if meta.file_attributes() & 0x400 != 0 {
                            return Err(io::ErrorKind::InvalidInput.into());
                        }
                    }
                    if index + 1 != parts.len() && !meta.is_dir() {
                        return Err(io::ErrorKind::InvalidInput.into());
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound && index + 1 == parts.len() => {
                    return Ok(false);
                }
                Err(e) => return Err(e),
            }
        }
        checkpoint(cancel, deadline)?;
        Ok(true)
    }
    /// Presence is presentation admission, never ownership/upgrade/launch authority.
    /// Retain only the unique journal's complete readonly inspection result.
    /// This is presentation readiness; explicit launch still performs fresh inspection.
    pub(super) fn choose_verified<T>(
        target: &Path,
        base: &Path,
        exists: impl FnMut(&Path) -> io::Result<bool>,
        mut inspect: impl FnMut(&Path) -> io::Result<Option<T>>,
    ) -> io::Result<(PathBuf, Option<T>)> {
        let mut verified = None;
        let journal = crate::installer_journal_choice::choose(target, base, exists, |path| {
            match inspect(path)? {
                Some(report) => {
                    verified = Some((path.to_path_buf(), report));
                    Ok(true)
                }
                None => Ok(false),
            }
        })?;
        let report = verified.and_then(|(path, report)| (path == journal).then_some(report));
        Ok((journal, report))
    }
    pub(super) fn inspect(snapshot: &Snapshot, cancel: &AtomicBool, deadline: Instant) -> Phase {
        let result = (|| {
            if exists(&snapshot.target, cancel, deadline)? {
                checkpoint(cancel, deadline)?;
                return Ok(true);
            }
            let journal = exists(&snapshot.journal, cancel, deadline)?;
            checkpoint(cancel, deadline)?;
            Ok::<_, io::Error>(journal)
        })();
        match result {
            Ok(true) => Phase::Existing,
            Ok(false) => Phase::Vacant,
            Err(_) => Phase::Unavailable,
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        // Create-only owned fixtures; retain tiny failure evidence, never remove
        // an arbitrary directory recursively and add no test dependency.
        struct FixtureRoot(PathBuf);
        impl FixtureRoot {
            fn path(&self) -> &Path {
                &self.0
            }
        }
        fn owned_fixture() -> FixtureRoot {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "dlpad-install-presence-owned-{}-{stamp}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).unwrap();
            FixtureRoot(root)
        }
        use std::time::Duration;
        fn snapshot(root: &Path) -> Snapshot {
            Snapshot {
                revision: 1,
                target: root.join("app"),
                journal: root.join("journal"),
            }
        }
        fn inspect_now(s: &Snapshot) -> Phase {
            inspect(
                s,
                &AtomicBool::new(false),
                Instant::now() + Duration::from_secs(2),
            )
        }
        #[test]
        fn existing_cold_check_retains_only_unique_report_without_second_inspection() {
            let root = owned_fixture();
            let s = snapshot(root.path());
            std::fs::create_dir(&s.target).unwrap();
            std::fs::write(&s.journal, b"current-journal").unwrap();
            let mut inspections = 0;
            let (journal, report) = choose_verified(
                &s.target,
                &s.journal,
                |path| {
                    exists(
                        path,
                        &AtomicBool::new(false),
                        Instant::now() + Duration::from_secs(1),
                    )
                },
                |path| {
                    inspections += 1;
                    let original = std::fs::read(path)?;
                    Ok((original == b"current-journal").then_some(original))
                },
            )
            .unwrap();
            assert_eq!(journal, s.journal);
            assert_eq!(report.as_deref(), Some(b"current-journal".as_slice()));
            assert_eq!(
                inspections, 1,
                "the completed inspection is retained, not repeated"
            );
            assert_eq!(std::fs::read(&s.journal).unwrap(), b"current-journal");
            assert!(!current(
                &s,
                s.revision + 1,
                Some((s.target.clone(), s.journal.clone()))
            ));
        }
        #[test]
        fn existing_readiness_rejects_ambiguous_cancelled_and_failed_inspection() {
            let root = owned_fixture();
            let s = snapshot(root.path());
            std::fs::create_dir(&s.target).unwrap();
            let other = root.path().join("journal-1");
            for path in [&s.journal, &other] {
                std::fs::write(path, b"preserve").unwrap();
            }
            let check = |path: &Path| {
                exists(
                    path,
                    &AtomicBool::new(false),
                    Instant::now() + Duration::from_secs(1),
                )
            };
            assert_eq!(
                choose_verified(&s.target, &s.journal, check, |_| Ok(Some(1)))
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(
                choose_verified::<()>(&s.target, &s.journal, check, |_| Err(
                    io::ErrorKind::Interrupted.into()
                ))
                .unwrap_err()
                .kind(),
                io::ErrorKind::Interrupted
            );
            assert_eq!(
                choose_verified::<()>(&s.target, &s.journal, check, |_| Ok(None))
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
            for path in [&s.journal, &other] {
                assert_eq!(std::fs::read(path).unwrap(), b"preserve");
            }
        }
        #[test]
        fn vacant_location_never_inherits_a_verification_result() {
            let root = owned_fixture();
            let s = snapshot(root.path());
            let (journal, report) = choose_verified::<()>(
                &s.target,
                &s.journal,
                |path| {
                    exists(
                        path,
                        &AtomicBool::new(false),
                        Instant::now() + Duration::from_secs(1),
                    )
                },
                |_| panic!("a fresh location has no existing bundle to verify"),
            )
            .unwrap();
            assert_eq!(journal, s.journal);
            assert!(report.is_none());
            assert!(inspect_now(&s).permits_install());
        }
        #[test]
        fn file_directory_and_journal_presence_cannot_offer_install() {
            let root = owned_fixture();
            let s = snapshot(root.path());
            assert!(inspect_now(&s).permits_install());
            std::fs::write(&s.target, b"external preserve").unwrap();
            assert_eq!(inspect_now(&s), Phase::Existing);
            assert!(!inspect_now(&s).permits_install());
            std::fs::remove_file(&s.target).unwrap();
            std::fs::create_dir(&s.target).unwrap();
            assert_eq!(inspect_now(&s), Phase::Existing);
            std::fs::remove_dir(&s.target).unwrap();
            std::fs::create_dir(&s.journal).unwrap();
            std::fs::write(s.journal.join("invalid-receipt"), b"preserve").unwrap();
            assert_eq!(inspect_now(&s), Phase::Existing);
            assert_eq!(
                std::fs::read(s.journal.join("invalid-receipt")).unwrap(),
                b"preserve"
            );
        }
        #[test]
        fn bad_parent_cancel_deadline_and_unknown_states_keep_install_disabled() {
            let root = owned_fixture();
            let mut s = snapshot(root.path());
            s.target = root.path().join("missing/app");
            assert_eq!(inspect_now(&s), Phase::Unavailable);
            s.target = root.path().join("app");
            assert_eq!(
                inspect(
                    &s,
                    &AtomicBool::new(true),
                    Instant::now() + Duration::from_secs(1)
                ),
                Phase::Unavailable
            );
            assert_eq!(
                inspect(&s, &AtomicBool::new(false), Instant::now()),
                Phase::Unavailable
            );
            for phase in [
                Phase::Unknown,
                Phase::Pending,
                Phase::Existing,
                Phase::Unavailable,
            ] {
                assert!(!phase.permits_install());
            }
        }
        #[test]
        fn edited_source_generation_and_replaced_roots_discard_stale_results() {
            let root = owned_fixture();
            let s = snapshot(root.path());
            assert!(current(&s, 1, Some((s.target.clone(), s.journal.clone()))));
            assert!(!current(&s, 2, Some((s.target.clone(), s.journal.clone()))));
            assert!(!current(
                &s,
                1,
                Some((root.path().join("new-app"), s.journal.clone()))
            ));
            assert!(!current(&s, 1, None));
        }
        #[cfg(unix)]
        #[test]
        fn symlink_parent_is_not_a_new_install_authority() {
            let root = owned_fixture();
            let foreign = owned_fixture();
            std::os::unix::fs::symlink(foreign.path(), root.path().join("link")).unwrap();
            let mut s = snapshot(root.path());
            s.target = root.path().join("link/app");
            assert_eq!(inspect_now(&s), Phase::Unavailable);
            assert!(!foreign.path().join("app").exists());
        }
    }
}
#[cfg(windows)]
mod native {
    use super::*;
    use dlpad_engine::app_install::{
        LocalInspectionRequest, LocalJournalReport, LocalJournalRequest, LocalRemovalReport,
        LocalRemovalRequest, inspect_local_existing_bundle, prepare_local_journal_bundle,
    };
    use folder_picker::{FolderPicker, FolderPickerCloser};
    use installer_catalog as text;
    use installer_file_picker::{ManifestPicker, PickerCloser};
    use ui_tokens as t;
    #[derive(Clone)]
    enum InstallerPickerCloser {
        Manifest(PickerCloser),
        Folder(FolderPickerCloser),
    }
    impl InstallerPickerCloser {
        fn cancel(&self) -> Result<(), ()> {
            match self {
                Self::Manifest(closer) => closer.cancel().map_err(|_| ()),
                Self::Folder(closer) => closer.cancel().map_err(|_| ()),
            }
        }
    }
    use std::cell::RefCell;
    use std::{
        io,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver, TryRecvError},
        },
        thread::{self, JoinHandle},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::*,
        Graphics::{
            Dwm::{DWMWA_USE_IMMERSIVE_DARK_MODE, DwmSetWindowAttribute},
            Gdi::*,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{
                BST_CHECKED, CDDS_PREPAINT, CDIS_DISABLED, CDIS_FOCUS, CDIS_HOT, CDIS_SELECTED,
                CDRF_DODEFAULT, CDRF_SKIPDEFAULT, DRAWITEMSTRUCT, EM_SETLIMITTEXT, EM_SETMARGINS,
                EM_SETRECTNP, MEASUREITEMSTRUCT, NM_CUSTOMDRAW, NMCUSTOMDRAW, NMHDR, ODS_DISABLED,
                ODS_FOCUS, ODS_SELECTED, ODT_COMBOBOX, SetScrollInfo, SetWindowTheme,
                WM_MOUSELEAVE,
            },
            Input::KeyboardAndMouse::{
                EnableWindow, GetFocus, IsWindowEnabled, TME_LEAVE, TRACKMOUSEEVENT,
                TrackMouseEvent,
            },
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::*,
        },
    };
    // Win32 static-control styles (winuser.h); windows-sys 0.61 does not expose these.
    const SS_NOPREFIX: u32 = 0x0000_0080;
    const SS_CENTERIMAGE: u32 = 0x0000_0200;
    const CLASS: &str = "DlpadDevelopmentInstaller";
    const TIMER: usize = 1;
    const ADMISSION_TIMER: usize = 2;
    const DEFAULT_COMMAND_KEY: [u16; 16] = [
        68, 108, 112, 97, 100, 68, 101, 102, 97, 117, 108, 116, 66, 116, 110, 0,
    ];
    const PICKER_CLOSE: u32 = WM_APP + 17;
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Operation {
        Prepare,
        Verify,
        Remove,
        Register,
    }
    enum Report {
        Journal(
            LocalJournalReport,
            Option<String>,
            Option<Result<(), io::ErrorKind>>,
        ),
        ShellRegistration(Result<(), io::ErrorKind>, bool),
        ToolPathsRejected,
        Removal(LocalRemovalReport),
    }
    struct Worker {
        cancel: Arc<AtomicBool>,
        handle: JoinHandle<()>,
        rx: Receiver<Report>,
        operation: Operation,
        revision: u64,
    }
    struct AdmissionResult {
        snapshot: admission::Snapshot,
        phase: admission::Phase,
        resolved_journal: Option<PathBuf>,
        verified_report: Option<LocalJournalReport>,
        shell_registered: bool,
    }
    struct AdmissionWorker {
        cancel: Arc<AtomicBool>,
        handle: JoinHandle<()>,
        rx: Receiver<AdmissionResult>,
        announce: bool,
    }
    const FOCUS_CONTROL: u32 = WM_APP + 18;
    const EDIT_SUBCLASS: usize = 0xd1;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetDpiForWindow(hwnd: HWND) -> u32;
        fn SetProcessDpiAwarenessContext(value: isize) -> i32;
    }
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegGetValueW(
            key: HANDLE,
            subkey: *const u16,
            value: *const u16,
            flags: u32,
            kind: *mut u32,
            data: *mut std::ffi::c_void,
            bytes: *mut u32,
        ) -> i32;
    }
    #[derive(Clone, Copy, Default, PartialEq, Eq)]
    enum Appearance {
        #[default]
        System,
        Light,
        Dark,
    }
    #[repr(C)]
    struct ContrastSetting {
        size: u32,
        flags: u32,
        scheme: *mut u16,
    }
    fn system_dark() -> bool {
        let subkey = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let value = wide("AppsUseLightTheme");
        let mut preference = 1u32;
        let mut bytes = 4u32;
        // Fixed current-user DWORD read only; never writes app data or OS preferences.
        unsafe {
            RegGetValueW(
                (-2147483647isize) as HANDLE,
                subkey.as_ptr(),
                value.as_ptr(),
                0x10,
                std::ptr::null_mut(),
                (&mut preference as *mut u32).cast(),
                &mut bytes,
            ) == 0
                && bytes == 4
                && preference == 0
        }
    }
    fn high_contrast() -> bool {
        let mut setting = ContrastSetting {
            size: std::mem::size_of::<ContrastSetting>() as u32,
            flags: 0,
            scheme: std::ptr::null_mut(),
        };
        unsafe {
            SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                setting.size,
                (&mut setting as *mut ContrastSetting).cast(),
                0,
            ) != 0
                && setting.flags & 1 != 0
        }
    }
    fn color(value: [u8; 3]) -> COLORREF {
        u32::from(value[0]) | (u32::from(value[1]) << 8) | (u32::from(value[2]) << 16)
    }
    fn mix(a: COLORREF, b: COLORREF, percent: u32) -> COLORREF {
        let mut result = 0;
        for shift in [0, 8, 16] {
            let a = (a >> shift) & 255;
            let b = (b >> shift) & 255;
            result |= ((a * (100 - percent) + b * percent) / 100) << shift;
        }
        result
    }
    fn px(value: f32, dpi: u32) -> i32 {
        (value * dpi.clamp(96, 1536) as f32 / 96.0).round() as i32
    }
    #[derive(Clone, Copy)]
    struct Palette {
        background: COLORREF,
        surface: COLORREF,
        secondary: COLORREF,
        foreground: COLORREF,
        muted: COLORREF,
        border: COLORREF,
        focus: COLORREF,
        dark: bool,
        contrast: bool,
    }
    impl Palette {
        fn new(appearance: Appearance) -> Self {
            let contrast = high_contrast();
            let dark = appearance == Appearance::Dark
                || (appearance == Appearance::System && system_dark());
            if contrast {
                return Self {
                    background: unsafe { GetSysColor(COLOR_WINDOW) },
                    surface: unsafe { GetSysColor(COLOR_WINDOW) },
                    secondary: unsafe { GetSysColor(COLOR_BTNFACE) },
                    foreground: unsafe { GetSysColor(COLOR_WINDOWTEXT) },
                    muted: unsafe { GetSysColor(COLOR_GRAYTEXT) },
                    border: unsafe { GetSysColor(COLOR_WINDOWTEXT) },
                    focus: unsafe { GetSysColor(COLOR_HIGHLIGHT) },
                    dark,
                    contrast,
                };
            }
            let (background, surface, secondary, foreground, muted) = if dark {
                (
                    t::DARK_BACKGROUND,
                    t::DARK_SURFACE,
                    t::DARK_SECONDARY,
                    t::DARK_FOREGROUND,
                    t::DARK_MUTED,
                )
            } else {
                (
                    t::LIGHT_BACKGROUND,
                    t::LIGHT_SURFACE,
                    t::LIGHT_SECONDARY,
                    t::LIGHT_FOREGROUND,
                    t::LIGHT_MUTED,
                )
            };
            let background = color(background);
            let foreground = color(foreground);
            Self {
                background,
                surface: color(surface),
                secondary: color(secondary),
                foreground,
                muted: color(muted),
                border: mix(background, foreground, 18),
                focus: color(if dark { t::ACCENT } else { t::PRIMARY }),
                dark,
                contrast,
            }
        }
    }
    // Fonts remain live until all child HWNDs have been destroyed. Borrowed stock fallback is never deleted.
    struct MessageFont {
        handle: HFONT,
        owned: bool,
    }
    impl MessageFont {
        fn new(size: f32, dpi: u32) -> Self {
            for face in ["Segoe UI", "Malgun Gothic"] {
                let mut definition = LOGFONTW {
                    lfHeight: -px(size, dpi),
                    lfWeight: FW_NORMAL as i32,
                    lfCharSet: DEFAULT_CHARSET,
                    lfQuality: CLEARTYPE_QUALITY,
                    ..Default::default()
                };
                for (slot, unit) in definition.lfFaceName.iter_mut().zip(face.encode_utf16()) {
                    *slot = unit;
                }
                let handle = unsafe { CreateFontIndirectW(&definition) };
                if !handle.is_null() {
                    let dc = unsafe { GetDC(std::ptr::null_mut()) };
                    let mut actual = [0u16; 32];
                    let correct = if dc.is_null() {
                        true
                    } else {
                        let previous = unsafe { SelectObject(dc, handle) };
                        let count =
                            unsafe { GetTextFaceW(dc, actual.len() as i32, actual.as_mut_ptr()) };
                        unsafe {
                            SelectObject(dc, previous);
                            ReleaseDC(std::ptr::null_mut(), dc);
                        }
                        count > 0
                            && String::from_utf16_lossy(
                                &actual
                                    [..actual.iter().position(|u| *u == 0).unwrap_or(actual.len())],
                            )
                            .eq_ignore_ascii_case(face)
                    };
                    if correct {
                        return Self {
                            handle,
                            owned: true,
                        };
                    }
                    unsafe {
                        DeleteObject(handle);
                    }
                }
            }
            Self {
                handle: unsafe { GetStockObject(DEFAULT_GUI_FONT) },
                owned: false,
            }
        }
    }
    impl Drop for MessageFont {
        fn drop(&mut self) {
            if self.owned {
                unsafe {
                    DeleteObject(self.handle);
                }
            }
        }
    }
    struct Fonts {
        body: MessageFont,
        small: MessageFont,
        heading: MessageFont,
        title: MessageFont,
    }
    impl Fonts {
        fn new(dpi: u32) -> Self {
            Self {
                body: MessageFont::new(t::BODY_FONT_SIZE, dpi),
                small: MessageFont::new(t::SMALL_FONT_SIZE, dpi),
                heading: MessageFont::new(t::HEADING_FONT_SIZE, dpi),
                title: MessageFont::new(t::TITLE_FONT_SIZE, dpi),
            }
        }
    }
    struct Brushes {
        background: HBRUSH,
        surface: HBRUSH,
        secondary: HBRUSH,
    }
    impl Brushes {
        fn new(palette: Palette) -> Self {
            unsafe {
                Self {
                    background: CreateSolidBrush(palette.background),
                    surface: CreateSolidBrush(palette.surface),
                    secondary: CreateSolidBrush(palette.secondary),
                }
            }
        }
    }
    impl Drop for Brushes {
        fn drop(&mut self) {
            unsafe {
                for brush in [self.background, self.surface, self.secondary] {
                    if !brush.is_null() {
                        DeleteObject(brush);
                    }
                }
            }
        }
    }
    struct Presentation {
        os_dpi: u32,
        dpi: u32,
        appearance: Appearance,
        palette: Palette,
        brushes: Brushes,
        fonts: Fonts,
        scroll: i32,
        content_height: i32,
    }
    impl Presentation {
        fn new(dpi: u32) -> Self {
            let palette = Palette::new(Appearance::System);
            Self {
                os_dpi: dpi,
                dpi,
                appearance: Appearance::System,
                palette,
                brushes: Brushes::new(palette),
                fonts: Fonts::new(dpi),
                scroll: 0,
                content_height: 0,
            }
        }
    }
    fn rounded(dc: HDC, rect: RECT, fill: COLORREF, border: COLORREF, line: i32, radius: i32) {
        unsafe {
            let brush = CreateSolidBrush(fill);
            let pen = CreatePen(PS_SOLID, line.max(1), border);
            if brush.is_null() || pen.is_null() {
                if !brush.is_null() {
                    DeleteObject(brush);
                }
                if !pen.is_null() {
                    DeleteObject(pen);
                }
                return;
            }
            let previous_brush = SelectObject(dc, brush);
            let previous_pen = SelectObject(dc, pen);
            RoundRect(
                dc,
                rect.left,
                rect.top,
                rect.right,
                rect.bottom,
                radius * 2,
                radius * 2,
            );
            SelectObject(dc, previous_pen);
            SelectObject(dc, previous_brush);
            DeleteObject(pen);
            DeleteObject(brush);
        }
    }
    fn command_focus_color(id: usize, neutral_focus: COLORREF) -> COLORREF {
        if matches!(id, 201 | 204 | 212) {
            color(t::LIGHT_BACKGROUND)
        } else {
            neutral_focus
        }
    }
    fn inside_focus(dc: HDC, mut rect: RECT, stroke: COLORREF, dpi: u32) {
        let inset = px(1.0, dpi).max(1);
        rect.left += inset;
        rect.top += inset;
        rect.right -= inset;
        rect.bottom -= inset;
        unsafe {
            let pen = CreatePen(PS_SOLID, px(1.5, dpi).max(1), stroke);
            if pen.is_null() {
                return;
            }
            let old_pen = SelectObject(dc, pen);
            let old_brush = SelectObject(dc, GetStockObject(NULL_BRUSH));
            RoundRect(
                dc,
                rect.left,
                rect.top,
                rect.right,
                rect.bottom,
                px(f32::from(t::RADIUS) * 2.0, dpi),
                px(f32::from(t::RADIUS) * 2.0, dpi),
            );
            SelectObject(dc, old_pen);
            SelectObject(dc, old_brush);
            DeleteObject(pen);
        }
    }
    fn paint_command(draw: &NMCUSTOMDRAW, view: &Presentation) -> LRESULT {
        if draw.dwDrawStage != CDDS_PREPAINT || view.palette.contrast {
            return CDRF_DODEFAULT as LRESULT;
        }
        let id = draw.hdr.idFrom;
        if !matches!(id, 201..=209 | 212) {
            return CDRF_DODEFAULT as LRESULT;
        }
        let p = view.palette;
        let dpi = view.dpi;
        let enabled = draw.uItemState & CDIS_DISABLED == 0
            && unsafe { IsWindowEnabled(draw.hdr.hwndFrom) } != 0;
        let pressed = draw.uItemState & CDIS_SELECTED != 0;
        let hot = draw.uItemState & CDIS_HOT != 0;
        let focused = enabled
            && (draw.uItemState & CDIS_FOCUS != 0 || unsafe { GetFocus() } == draw.hdr.hwndFrom);
        let dc = draw.hdc;
        let saved = unsafe { SaveDC(dc) };
        if saved == 0 {
            return CDRF_DODEFAULT as LRESULT;
        }
        unsafe {
            FillRect(dc, &draw.rc, view.brushes.background);
            SelectObject(dc, view.fonts.body.handle);
            SetBkMode(dc, TRANSPARENT as i32);
        }
        let mut label = [0u16; 256];
        let count =
            unsafe { GetWindowTextW(draw.hdr.hwndFrom, label.as_mut_ptr(), label.len() as i32) };
        if (206..=208).contains(&id) {
            let side = px(14.0, dpi);
            let top = draw.rc.top + ((draw.rc.bottom - draw.rc.top - side) / 2);
            let mark = RECT {
                left: draw.rc.left + px(2.0, dpi),
                top,
                right: draw.rc.left + px(2.0, dpi) + side,
                bottom: top + side,
            };
            let checked = unsafe { SendMessageW(draw.hdr.hwndFrom, BM_GETCHECK, 0, 0) }
                == BST_CHECKED as isize;
            let selected = color(t::PRIMARY);
            rounded(
                dc,
                mark,
                if checked && enabled {
                    if pressed {
                        color(t::PRIMARY_PRESSED)
                    } else if hot {
                        color(t::PRIMARY_HOVER)
                    } else {
                        selected
                    }
                } else if pressed && enabled {
                    p.border
                } else if checked || (hot && enabled) {
                    p.secondary
                } else {
                    p.surface
                },
                if checked && enabled {
                    selected
                } else {
                    p.border
                },
                px(1.0, dpi),
                px(3.0, dpi),
            );
            if checked {
                unsafe {
                    let pen = CreatePen(
                        PS_SOLID,
                        px(1.5, dpi).max(1),
                        if enabled {
                            color(t::LIGHT_BACKGROUND)
                        } else {
                            p.muted
                        },
                    );
                    if !pen.is_null() {
                        let old = SelectObject(dc, pen);
                        MoveToEx(
                            dc,
                            mark.left + side / 4,
                            mark.top + side / 2,
                            std::ptr::null_mut(),
                        );
                        LineTo(dc, mark.left + side * 9 / 20, mark.top + side * 3 / 4);
                        LineTo(dc, mark.left + side * 4 / 5, mark.top + side / 4);
                        SelectObject(dc, old);
                        DeleteObject(pen);
                    }
                }
            }
            let mut text_rect = draw.rc;
            text_rect.left = mark.right + px(t::LABEL_GAP, dpi);
            unsafe {
                SetTextColor(dc, if enabled { p.foreground } else { p.muted });
                DrawTextW(
                    dc,
                    label.as_ptr(),
                    count,
                    &mut text_rect,
                    DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
                );
            }
        } else {
            let primary = matches!(id, 201 | 212);
            let danger = id == 204;
            let ghost = id == 202;
            let neutral = if pressed {
                if p.dark { 72 } else { 218 }
            } else if hot {
                if p.dark { 60 } else { 232 }
            } else {
                0
            };
            let fill = if !enabled {
                p.secondary
            } else if primary {
                color(if pressed {
                    t::PRIMARY_PRESSED
                } else if hot {
                    t::PRIMARY_HOVER
                } else {
                    t::PRIMARY
                })
            } else if danger {
                color(if pressed {
                    [136, 27, 32]
                } else if hot {
                    [156, 33, 38]
                } else {
                    t::DANGER
                })
            } else if hot || pressed {
                color([neutral, neutral, neutral])
            } else if ghost {
                p.background
            } else {
                p.secondary
            };
            rounded(
                dc,
                draw.rc,
                fill,
                if ((primary || danger) && enabled) || ghost {
                    fill
                } else {
                    p.border
                },
                px(1.0, dpi),
                px(f32::from(t::RADIUS), dpi),
            );
            unsafe {
                SetTextColor(
                    dc,
                    if !enabled {
                        p.muted
                    } else if primary || danger {
                        color(t::LIGHT_BACKGROUND)
                    } else {
                        p.foreground
                    },
                );
                let mut rect = draw.rc;
                DrawTextW(
                    dc,
                    label.as_ptr(),
                    count,
                    &mut rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX,
                );
            }
        }
        if focused {
            inside_focus(dc, draw.rc, command_focus_color(id, p.focus), dpi);
        }
        unsafe {
            RestoreDC(dc, saved);
        }
        CDRF_SKIPDEFAULT as LRESULT
    }
    struct State {
        // Stable run-owned graphics cell is independent of the policy RefCell during nested paint.
        presentation: *const RefCell<Presentation>,
        heading: HWND,
        tools_heading: HWND,
        theme_label: HWND,
        theme: HWND,
        scale: HWND,
        scale_label: HWND,
        appearance_help: HWND,
        edits: [HWND; 4],
        labels: [HWND; 4],
        intro: HWND,
        status: HWND,
        buttons: [HWND; 4],
        launch_button: HWND,
        launch: Option<crate::installer_launch::Supervisor>,
        launch_failure: Option<crate::installer_launch::Completion>,
        picker_button: HWND,
        target_picker_button: HWND,
        tool_checks: [HWND; 3],
        tools_hint: HWND,
        tools_result: HWND,
        tool_data: Option<PathBuf>,
        packaged_source: Option<[PathBuf; 2]>,
        resolved_packaged_journal: Option<(PathBuf, PathBuf)>,
        picker_active: bool,
        picker_closer: Option<InstallerPickerCloser>,
        worker: Option<Worker>,
        admission_worker: Option<AdmissionWorker>,
        admission_pending: Option<(admission::Snapshot, Instant, bool)>,
        admission_revision: u64,
        admission_phase: admission::Phase,
        closing: bool,
        verified: bool,
        remove_eligible: bool,
        shell_registered: bool,
        confirmation: Option<LocalRemovalRequest>,
    }
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    fn sync_stage_copy(state: &State, failed: bool) {
        let (heading, intro) = if let Some(worker) = &state.worker
            && !failed
            && !state.closing
        {
            match worker.operation {
                Operation::Verify => (text::VERIFYING_HEADING, text::VERIFYING_INTRO),
                Operation::Prepare => (text::INSTALLING_HEADING, text::INSTALLING),
                Operation::Remove => (text::REMOVING_HEADING, text::REMOVING),
                Operation::Register => (text::SHELL_REGISTER_HEADING, text::SHELL_REGISTERING),
            }
        } else if admission_busy(state) && !failed && !state.closing {
            (text::CHECKING_HEADING, text::CHECKING_INTRO)
        } else if !state.verified
            && !failed
            && state.admission_phase == admission::Phase::Existing
            && state.worker.is_none()
        {
            (text::EXISTING_HEADING, text::EXISTING_INTRO)
        } else {
            text::stage_copy(state.verified, state.packaged_source.is_some(), failed)
        };
        let heading = wide(heading);
        let intro = wide(intro);
        unsafe {
            SetWindowTextW(state.heading, heading.as_ptr());
            SetWindowTextW(state.intro, intro.as_ptr());
            // Existing layout remeasures the real STATIC text at current scale.
            PostMessageW(GetParent(state.heading), LAYOUT_VIEW, 0, 0);
        }
    }
    fn status(state: &State, s: &str) {
        sync_stage_copy(state, text::failure_stage_message(s));
        let w = wide(s);
        unsafe {
            SetWindowTextW(state.status, w.as_ptr());
            PostMessageW(GetParent(state.status), LAYOUT_VIEW, 0, 0);
        }
    }
    fn controls(state: &State, active: bool) {
        let active = active || state.picker_active || launch_busy(state);
        let confirming = state.confirmation.is_some();
        unsafe {
            EnableWindow(
                state.launch_button,
                (state.verified
                    && !active
                    && !confirming
                    && !state.closing
                    && !admission_busy(state)) as i32,
            );
            for check in state.tool_checks {
                EnableWindow(check, (!active && !confirming && !state.closing) as i32);
            }
            for edit in state.edits {
                EnableWindow(edit, (!active && !confirming && !state.closing) as i32);
            }
            EnableWindow(
                state.buttons[0],
                (((!state.verified && state.admission_phase.permits_install())
                    || (state.verified
                        && state.packaged_source.is_some()
                        && !state.shell_registered))
                    && !active
                    && !confirming
                    && !state.closing) as i32,
            );
            let install_label = wide(if state.verified && state.packaged_source.is_some() {
                if state.shell_registered {
                    text::SHELL_REGISTERED_BUTTON
                } else {
                    text::SHELL_REGISTER_BUTTON
                }
            } else {
                text::INSTALL
            });
            SetWindowTextW(state.buttons[0], install_label.as_ptr());
            EnableWindow(
                state.buttons[1],
                ((active || confirming || state.admission_worker.is_some())
                    && !state.closing
                    && !state.picker_active) as i32,
            );
            EnableWindow(
                state.buttons[2],
                (!active && !confirming && !state.closing && !admission_busy(state)) as i32,
            );
            EnableWindow(
                state.buttons[3],
                (!active
                    && !state.closing
                    && (confirming
                        || state.remove_eligible
                        || (state.verified && state.shell_registered))) as i32,
            );
            EnableWindow(
                state.picker_button,
                (!active && !confirming && !state.closing && state.packaged_source.is_none())
                    as i32,
            );
            EnableWindow(
                state.target_picker_button,
                (!active && !confirming && !state.closing && state.packaged_source.is_some())
                    as i32,
            );
            let label = wide(if confirming {
                text::CONFIRM_REMOVE
            } else if state.shell_registered {
                text::SHELL_REMOVE_GUIDE_BUTTON
            } else {
                text::REMOVE
            });
            SetWindowTextW(state.buttons[3], label.as_ptr());
            let cancel = wide(if confirming {
                text::BACK
            } else if let Some(worker) = &state.worker {
                match worker.operation {
                    Operation::Prepare => text::CANCEL_INSTALL,
                    Operation::Verify | Operation::Register => text::CANCEL,
                    Operation::Remove => text::CANCEL_REMOVE,
                }
            } else if launch_busy(state) {
                text::CANCEL_LAUNCH
            } else {
                text::CANCEL
            });
            SetWindowTextW(state.buttons[1], cancel.as_ptr());
            let default = if active || confirming || state.closing || admission_busy(state) {
                0
            } else if state.verified {
                212
            } else if state.admission_phase == admission::Phase::Existing {
                203
            } else if state.admission_phase.permits_install() {
                201
            } else {
                0
            };
            SetPropW(
                GetParent(state.status),
                DEFAULT_COMMAND_KEY.as_ptr(),
                default as HANDLE,
            );
        }
    }
    fn admission_busy(state: &State) -> bool {
        state.admission_worker.is_some() || state.admission_pending.is_some()
    }
    fn queue_admission(hwnd: HWND, state: &mut State, announce: bool) {
        state.admission_revision = state.admission_revision.wrapping_add(1);
        if let Some(worker) = &state.admission_worker {
            worker.cancel.store(true, Ordering::Release);
        }
        state.admission_pending = selected_roots(state).ok().map(|(target, journal)| {
            (
                admission::Snapshot {
                    revision: state.admission_revision,
                    target,
                    journal,
                },
                Instant::now() + Duration::from_millis(250),
                announce,
            )
        });
        state.admission_phase = if state.admission_pending.is_some() {
            admission::Phase::Pending
        } else {
            admission::Phase::Unavailable
        };
        if unsafe { SetTimer(hwnd, ADMISSION_TIMER, 75, None) } == 0 {
            state.admission_pending = None;
            state.admission_phase = admission::Phase::Unavailable;
        }
        if announce && state.worker.is_none() && !launch_busy(state) && !state.closing {
            status(
                state,
                if state.admission_pending.is_some() {
                    text::CHECKING_LOCATION
                } else {
                    text::LOCATION_UNAVAILABLE
                },
            );
        }
        controls(state, state.worker.is_some());
    }
    fn poll_admission(hwnd: HWND, state: &mut State) {
        if state
            .admission_worker
            .as_ref()
            .is_some_and(|w| w.handle.is_finished())
        {
            let worker = state.admission_worker.take().unwrap();
            let result = worker.rx.try_recv();
            let joined = worker.handle.join().is_ok();
            if let Ok(AdmissionResult {
                snapshot,
                phase,
                resolved_journal,
                verified_report,
                shell_registered,
            }) = result
                && joined
                && !worker.cancel.load(Ordering::Acquire)
                && admission::current(
                    &snapshot,
                    state.admission_revision,
                    selected_roots(state).ok(),
                )
                && state.admission_pending.is_none()
                && !state.closing
            {
                if let Some(journal) = resolved_journal {
                    state.resolved_packaged_journal =
                        Some((snapshot.target.clone(), journal.clone()));
                    // Updating a native EDIT emits EN_CHANGE, including on identical text.
                    // Avoid a self-sustaining admission loop for the derived read-only field.
                    if read_edit(state.edits[3]).ok().as_ref() != Some(&journal) {
                        let value = wide(&journal.to_string_lossy());
                        unsafe {
                            SetWindowTextW(state.edits[3], value.as_ptr());
                        }
                    }
                }
                state.admission_phase = phase;
                // Presence alone never reaches ready: retain the exact journal's successful
                // cold inspection, after the original revision/input/worker checks above.
                if phase == admission::Phase::Existing
                    && verified_report
                        .is_some_and(|r| r.kind.is_none() && r.current_consistency_verified)
                {
                    state.verified = true;
                    state.shell_registered = shell_registered;
                    state.remove_eligible = !shell_registered;
                }
                if worker.announce && state.worker.is_none() && !launch_busy(state) {
                    status(
                        state,
                        match phase {
                            admission::Phase::Vacant => {
                                if state.packaged_source.is_some() {
                                    text::PACKAGED_IDLE
                                } else {
                                    text::IDLE
                                }
                            }
                            admission::Phase::Existing if state.verified => text::VERIFIED,
                            admission::Phase::Existing => text::EXISTING_LOCATION,
                            _ => text::LOCATION_UNAVAILABLE,
                        },
                    );
                }
            } else if !joined && state.admission_pending.is_none() {
                state.admission_phase = admission::Phase::Unavailable;
                status(state, text::LOCATION_UNAVAILABLE);
            } else if worker.cancel.load(Ordering::Acquire)
                && state.admission_pending.is_none()
                && !state.closing
                && state.admission_phase == admission::Phase::Unknown
            {
                // The cancelled discovery returned, but its partial result is never applied.
                status(state, text::LOCATION_CANCELLED);
            }
        }
        if state.closing
            && state.admission_worker.is_none()
            && state.worker.is_none()
            && state.launch.is_none()
        {
            unsafe {
                KillTimer(hwnd, ADMISSION_TIMER);
                DestroyWindow(hwnd);
            }
            return;
        }
        if state.admission_worker.is_none()
            && state.worker.is_none()
            && !launch_busy(state)
            && !state.picker_active
            && !state.closing
            && state
                .admission_pending
                .as_ref()
                .is_some_and(|(_, after, _)| Instant::now() >= *after)
        {
            let (snapshot, _, announce) = state.admission_pending.take().unwrap();
            let cancel = Arc::new(AtomicBool::new(false));
            let flag = cancel.clone();
            let packaged = state.packaged_source.is_some();
            let (tx, rx) = mpsc::sync_channel(1);
            match thread::Builder::new()
                .name("dlpad-install-location".into())
                .spawn(move || {
                    let deadline = Instant::now() + Duration::from_secs(10);
                    let resolved = if packaged {
                        packaged_journal(&snapshot.target)
                            .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))
                            .and_then(|base| {
                                admission::choose_verified(
                                    &snapshot.target,
                                    &base,
                                    |path| admission::exists(path, &flag, deadline),
                                    |journal| {
                                        let report = inspect_local_existing_bundle(
                                            &LocalInspectionRequest {
                                                bundle_root: snapshot.target.clone(),
                                                journal_root: journal.to_path_buf(),
                                            },
                                            &flag,
                                            deadline,
                                        );
                                        match report.kind {
                                            Some(
                                                io::ErrorKind::Interrupted
                                                | io::ErrorKind::TimedOut
                                                | io::ErrorKind::PermissionDenied,
                                            ) => Err(report.kind.unwrap().into()),
                                            _ if report.kind.is_none()
                                                && report.current_consistency_verified =>
                                            {
                                                Ok(Some(report))
                                            }
                                            _ => Ok(None),
                                        }
                                    },
                                )
                            })
                            .map(|(journal, report)| (Some(journal), report))
                    } else {
                        Ok((None, None))
                    };
                    let (phase, journal, verified_report) = match resolved {
                        Ok((journal, report)) => {
                            let actual = admission::Snapshot {
                                journal: journal
                                    .clone()
                                    .unwrap_or_else(|| snapshot.journal.clone()),
                                ..snapshot.clone()
                            };
                            (
                                admission::inspect(&actual, &flag, deadline),
                                journal,
                                report,
                            )
                        }
                        Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                            (admission::Phase::Existing, None, None)
                        }
                        Err(_) => (admission::Phase::Unavailable, None, None),
                    };
                    let shell_registered = verified_report.as_ref().is_some_and(|report| {
                        report.kind.is_none() && report.current_consistency_verified
                    }) && journal.as_ref().is_some_and(|journal| {
                        crate::installer_os::is_registered(
                            &LocalInspectionRequest {
                                bundle_root: snapshot.target.clone(),
                                journal_root: journal.clone(),
                            },
                            &flag,
                            deadline,
                        )
                    });
                    let _ = tx.try_send(AdmissionResult {
                        snapshot,
                        phase,
                        resolved_journal: journal,
                        verified_report,
                        shell_registered,
                    });
                }) {
                Ok(handle) => {
                    state.admission_worker = Some(AdmissionWorker {
                        cancel,
                        handle,
                        rx,
                        announce,
                    })
                }
                Err(_) => {
                    state.admission_phase = admission::Phase::Unavailable;
                    status(state, text::LOCATION_UNAVAILABLE);
                }
            }
        }
        controls(state, state.worker.is_some());
        if !admission_busy(state) {
            unsafe {
                KillTimer(hwnd, ADMISSION_TIMER);
            }
        }
    }
    fn launch_busy(state: &State) -> bool {
        state.launch.is_some()
            || state
                .launch_failure
                .as_ref()
                .is_some_and(|outcome| !outcome.terminal_confirmed())
    }
    fn invalidate(state: &mut State) {
        state.verified = false;
        state.remove_eligible = false;
        state.shell_registered = false;
        state.confirmation = None;
        sync_stage_copy(state, false);
    }
    fn confirm_remove(hwnd: HWND, state: &mut State) {
        if state.worker.is_some() || launch_busy(state) || state.closing || state.picker_active {
            return;
        }
        if state.verified && state.shell_registered {
            status(state, text::SHELL_REMOVE_GUIDE);
            return;
        }
        if state.confirmation.is_some() {
            start(hwnd, state, Operation::Remove);
            return;
        }
        if !state.remove_eligible {
            return;
        }
        match request_roots(state) {
            Ok(snapshot) => {
                state.confirmation = Some(snapshot);
                status(state, text::REMOVE_CONFIRM);
                controls(state, false);
            }
            Err(()) => {
                invalidate(state);
                status(state, text::INVALID_ROOTS);
                controls(state, false);
            }
        }
    }
    fn edit_snapshot(edits: [HWND; 4]) -> Result<Vec<OsString>, ()> {
        use std::os::windows::ffi::OsStringExt;
        edits
            .into_iter()
            .map(|edit| {
                let len = unsafe { GetWindowTextLengthW(edit) };
                if !(0..=32767).contains(&len) {
                    return Err(());
                }
                let mut buf = vec![0u16; len as usize + 1];
                if unsafe { GetWindowTextW(edit, buf.as_mut_ptr(), buf.len() as i32) } != len {
                    return Err(());
                }
                Ok(OsString::from_wide(&buf[..len as usize]))
            })
            .collect()
    }
    fn choose_manifest(hwnd: HWND, cell: &RefCell<State>) {
        let (edits, snapshot) = {
            let Ok(mut state) = cell.try_borrow_mut() else {
                return;
            };
            if state.packaged_source.is_some()
                || state.worker.is_some()
                || launch_busy(&state)
                || state.closing
                || state.confirmation.is_some()
                || state.picker_active
            {
                return;
            }
            let Ok(snapshot) = edit_snapshot(state.edits) else {
                status(&state, text::PICKER_FAILED);
                return;
            };
            state.picker_active = true;
            controls(&state, false);
            (state.edits, snapshot)
        };
        // No State borrow spans COM setup or the nested native modal message loop.
        let initial = local_path(&snapshot[0]).ok();
        let result = match ManifestPicker::new(
            initial.as_deref(),
            text::PICKER_TITLE,
            text::PICKER_FILTER,
        ) {
            Ok(picker) => {
                let closer = picker.closer(); // COM AddRef occurs without State borrow.
                {
                    let mut state = cell.borrow_mut();
                    state.picker_closer = Some(InstallerPickerCloser::Manifest(closer));
                }
                let closing = cell.borrow().closing;
                let result = if closing { Ok(None) } else { picker.show(hwnd) };
                let closer = {
                    let mut state = cell.borrow_mut();
                    state.picker_closer.take()
                };
                drop(closer); // COM Release occurs without State borrow.
                drop(picker); // All dialog COM/apartment ownership ends before owner destruction.
                result.map_err(|_| ())
            }
            Err(_) => Err(()),
        };
        // All modal/COM calls and their nested proc borrows have returned. Do not silently strand picker_active.
        let mut state = cell.borrow_mut();
        state.picker_active = false;
        if state.closing {
            drop(state);
            unsafe {
                DestroyWindow(hwnd);
            };
            return;
        }
        let original_owner = unsafe { IsWindow(hwnd) } != 0
            && std::ptr::eq(
                unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const RefCell<State>,
                cell,
            );
        if !original_owner
            || state.worker.is_some()
            || launch_busy(&state)
            || state.confirmation.is_some()
            || state.edits != edits
            || edit_snapshot(edits).ok().as_ref() != Some(&snapshot)
        {
            status(&state, text::PICKER_CHANGED);
            controls(&state, false);
            return;
        }
        match result {
            Ok(None) => {} // Cancel preserves all input, status and verification eligibility.
            Ok(Some(path)) => {
                use std::os::windows::ffi::OsStrExt;
                if local_path(path.as_os_str()).is_err() {
                    status(&state, text::PICKER_FAILED);
                } else {
                    let units: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
                    if unsafe { SetWindowTextW(edits[0], units.as_ptr()) } == 0 {
                        status(&state, text::PICKER_FAILED);
                    } else {
                        invalidate(&mut state);
                        queue_admission(hwnd, &mut state, true);
                    }
                }
            }
            Err(()) => status(&state, text::PICKER_FAILED),
        }
        controls(&state, false);
    }
    fn choose_target_parent(hwnd: HWND, cell: &RefCell<State>) {
        let (edits, snapshot) = {
            let Ok(mut state) = cell.try_borrow_mut() else {
                return;
            };
            if state.packaged_source.is_none()
                || state.worker.is_some()
                || launch_busy(&state)
                || state.closing
                || state.confirmation.is_some()
                || state.picker_active
            {
                return;
            }
            let Ok(snapshot) = edit_snapshot(state.edits) else {
                status(&state, text::PICKER_FAILED);
                return;
            };
            state.picker_active = true;
            controls(&state, false);
            (state.edits, snapshot)
        };
        // No State borrow spans COM setup or the nested native modal message loop.
        let initial = local_path(&snapshot[2])
            .ok()
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf));
        let result = match FolderPicker::new(initial.as_deref(), text::TARGET_PICKER_TITLE) {
            Ok(picker) => {
                let closer = picker.closer(); // COM AddRef occurs without State borrow.
                {
                    let mut state = cell.borrow_mut();
                    state.picker_closer = Some(InstallerPickerCloser::Folder(closer));
                }
                let closing = cell.borrow().closing;
                let result = if closing { Ok(None) } else { picker.show(hwnd) };
                let closer = {
                    let mut state = cell.borrow_mut();
                    state.picker_closer.take()
                };
                drop(closer); // COM Release occurs without State borrow.
                drop(picker); // All dialog COM/apartment ownership ends before owner destruction.
                result.map_err(|_| ())
            }
            Err(_) => Err(()),
        };
        // All modal/COM calls and their nested proc borrows have returned. Do not silently strand picker_active.
        let mut state = cell.borrow_mut();
        state.picker_active = false;
        if state.closing {
            drop(state);
            unsafe {
                DestroyWindow(hwnd);
            };
            return;
        }
        let original_owner = unsafe { IsWindow(hwnd) } != 0
            && std::ptr::eq(
                unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const RefCell<State>,
                cell,
            );
        if !original_owner
            || state.worker.is_some()
            || launch_busy(&state)
            || state.confirmation.is_some()
            || state.edits != edits
            || edit_snapshot(edits).ok().as_ref() != Some(&snapshot)
        {
            status(&state, text::PICKER_CHANGED);
            controls(&state, false);
            return;
        }
        match result {
            Ok(None) => {} // Cancel preserves all input, status and verification eligibility.
            Ok(Some(parent)) => {
                let path = parent.join("dlpad-preview");
                use std::os::windows::ffi::OsStrExt;
                if packaged_journal(&path).is_err() {
                    status(&state, text::PICKER_FAILED);
                } else {
                    let units: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
                    if unsafe { SetWindowTextW(edits[2], units.as_ptr()) } == 0 {
                        status(&state, text::PICKER_FAILED);
                    } else {
                        invalidate(&mut state);
                        queue_admission(hwnd, &mut state, true);
                    }
                }
            }
            Err(()) => status(&state, text::PICKER_FAILED),
        }
        controls(&state, false);
    }
    fn read_edit(edit: HWND) -> Result<PathBuf, ()> {
        let len = unsafe { GetWindowTextLengthW(edit) };
        if !(1..=32767).contains(&len) {
            return Err(());
        }
        let mut buf = vec![0u16; len as usize + 1];
        let got = unsafe { GetWindowTextW(edit, buf.as_mut_ptr(), buf.len() as i32) };
        if got != len {
            return Err(());
        }
        use std::os::windows::ffi::OsStringExt;
        local_path(&OsString::from_wide(&buf[..got as usize]))
    }
    fn selected_roots(state: &State) -> Result<(PathBuf, PathBuf), ()> {
        let target = read_edit(state.edits[2])?;
        let journal = if state.packaged_source.is_some() {
            match &state.resolved_packaged_journal {
                Some((bound_target, journal)) if bound_target == &target => journal.clone(),
                _ => packaged_journal(&target)?,
            }
        } else {
            read_edit(state.edits[3])?
        };
        Ok((target, journal))
    }
    fn request_prepare(state: &State) -> Result<LocalJournalRequest, ()> {
        let (bundle_root, journal_root) = selected_roots(state)?;
        Ok(LocalJournalRequest {
            manifest: match &state.packaged_source {
                Some(paths) => paths[0].clone(),
                None => read_edit(state.edits[0])?,
            },
            source_dir: match &state.packaged_source {
                Some(paths) => paths[1].clone(),
                None => read_edit(state.edits[1])?,
            },
            bundle_root,
            journal_root,
        })
    }
    fn request_roots(state: &State) -> Result<LocalRemovalRequest, ()> {
        let (bundle_root, journal_root) = selected_roots(state)?;
        Ok(LocalRemovalRequest {
            bundle_root,
            journal_root,
        })
    }
    enum Request {
        Prepare(
            LocalJournalRequest,
            [bool; 3],
            Option<std::path::PathBuf>,
            bool,
        ),
        Register(LocalInspectionRequest),
        Verify(LocalInspectionRequest),
        Remove(LocalRemovalRequest),
    }
    fn optional_tools_after_app(
        app: &LocalJournalReport,
        data: Option<std::path::PathBuf>,
        install: impl FnOnce(std::path::PathBuf) -> String,
    ) -> Option<String> {
        if app.kind.is_none() && app.current_consistency_verified {
            data.map(install)
        } else {
            None
        }
    }
    fn recheck(hwnd: HWND, state: &mut State) {
        if state.packaged_source.is_none() {
            start(hwnd, state, Operation::Verify);
            return;
        }
        if admission_busy(state)
            || state.worker.is_some()
            || launch_busy(state)
            || state.picker_active
            || state.closing
            || state.confirmation.is_some()
        {
            return;
        }
        // A cancelled initial check may never have resolved the current journal.
        // Rediscover the unique matching generation instead of trusting the base hint.
        // This also restores installation admission for a vacant target.
        invalidate(state);
        queue_admission(hwnd, state, true);
        if let Some((_, after, _)) = state.admission_pending.as_mut() {
            *after = Instant::now();
        }
        poll_admission(hwnd, state);
    }
    fn start(hwnd: HWND, state: &mut State, operation: Operation) {
        if (operation == Operation::Prepare
            && (state.verified || !state.admission_phase.permits_install()))
            || (operation == Operation::Register
                && (!state.verified || state.packaged_source.is_none() || state.shell_registered))
            || admission_busy(state)
            || state.worker.is_some()
            || launch_busy(state)
            || state.picker_active
            || state.closing
            || (operation != Operation::Remove && state.confirmation.is_some())
        {
            return;
        }
        let request = match match operation {
            Operation::Prepare => request_prepare(state).and_then(|request| {
                let selected = state.tool_checks.map(|check| unsafe {
                    SendMessageW(check, BM_GETCHECK, 0, 0) == BST_CHECKED as isize
                });
                let data = if selected.iter().any(|selected| *selected) {
                    Some(state.tool_data.clone().ok_or(())?)
                } else {
                    None
                };
                Ok(Request::Prepare(
                    request,
                    selected,
                    data,
                    state.packaged_source.is_some(),
                ))
            }),
            Operation::Verify => request_roots(state).map(|r| {
                Request::Verify(LocalInspectionRequest {
                    bundle_root: r.bundle_root,
                    journal_root: r.journal_root,
                })
            }),
            Operation::Remove => state.confirmation.take().ok_or(()).map(Request::Remove),
            Operation::Register => request_roots(state).map(|r| {
                Request::Register(LocalInspectionRequest {
                    bundle_root: r.bundle_root,
                    journal_root: r.journal_root,
                })
            }),
        } {
            Ok(r) => r,
            Err(()) => {
                invalidate(state);
                controls(state, false);
                status(
                    state,
                    if state.packaged_source.is_some() {
                        text::PACKAGED_INVALID
                    } else if operation == Operation::Prepare {
                        text::INVALID
                    } else {
                        text::INVALID_ROOTS
                    },
                );
                return;
            }
        };
        // SetTimer before launching: failure cannot leave a worker without completion observation.
        if unsafe { SetTimer(hwnd, TIMER, 75, None) } == 0 {
            invalidate(state);
            controls(state, false);
            status(state, text::WORKER_FAILED);
            return;
        }
        let empty = wide("");
        unsafe {
            SetWindowTextW(state.tools_result, empty.as_ptr());
            ShowWindow(state.tools_result, SW_HIDE);
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        let (tx, rx) = mpsc::sync_channel(1);
        let deadline = Instant::now() + Duration::from_secs(300);
        let handle = match thread::Builder::new()
            .name("dlpad-install-worker".into())
            .spawn(move || {
                let report = match request {
                    Request::Prepare(r, selected, data, register_shell) => {
                        if data.as_ref().is_some_and(|data| {
                            !crate::installer_tools::tool_root_is_disjoint(
                                data,
                                &[&r.bundle_root, &r.journal_root, &r.source_dir],
                            )
                        }) {
                            Report::ToolPathsRejected
                        } else {
                            let app = prepare_local_journal_bundle(&r, &flag, deadline);
                            let summary = optional_tools_after_app(&app, data, |data| {
                                crate::installer_tools::install_selected(
                                    &data, &selected, &flag, deadline,
                                )
                            });
                            let shell = if register_shell
                                && app.kind.is_none()
                                && app.current_consistency_verified
                            {
                                Some(crate::installer_os::register(
                                    &r.bundle_root,
                                    &r.journal_root,
                                    &flag,
                                    deadline,
                                ))
                            } else {
                                None
                            };
                            Report::Journal(app, summary, shell)
                        }
                    }
                    Request::Verify(r) => {
                        let app = inspect_local_existing_bundle(&r, &flag, deadline);
                        let shell = (app.kind.is_none()
                            && app.current_consistency_verified
                            && crate::installer_os::is_registered(&r, &flag, deadline))
                        .then_some(Ok(()));
                        Report::Journal(app, None, shell)
                    }
                    Request::Register(r) => {
                        let result = crate::installer_os::register(
                            &r.bundle_root,
                            &r.journal_root,
                            &flag,
                            deadline,
                        );
                        let verified = result.is_ok() || {
                            let app = inspect_local_existing_bundle(&r, &flag, deadline);
                            app.kind.is_none() && app.current_consistency_verified
                        };
                        Report::ShellRegistration(result, verified)
                    }
                    Request::Remove(r) => Report::Removal(crate::installer_os::remove(
                        &r.bundle_root,
                        &r.journal_root,
                        &flag,
                        deadline,
                    )),
                };
                // One final value fits the empty one-slot queue; no intermediate samples.
                let _ = tx.try_send(report);
            }) {
            Ok(h) => h,
            Err(_) => {
                unsafe {
                    KillTimer(hwnd, TIMER);
                }
                invalidate(state);
                controls(state, false);
                status(state, text::WORKER_FAILED);
                return;
            }
        };
        state.worker = Some(Worker {
            cancel,
            handle,
            rx,
            operation,
            revision: state.admission_revision,
        });
        invalidate(state);
        status(
            state,
            match operation {
                Operation::Verify => text::VERIFYING,
                Operation::Register => text::SHELL_REGISTERING,
                Operation::Prepare => text::INSTALLING,
                Operation::Remove => text::REMOVING,
            },
        );
        controls(state, true);
    }
    fn stop(state: &mut State, closing: bool) {
        state.closing |= closing;
        let pending_cancelled = state.admission_pending.take().is_some();
        if pending_cancelled && state.admission_worker.is_none() {
            state.admission_phase = admission::Phase::Unknown;
            if !closing {
                status(state, text::LOCATION_CANCELLED);
            }
            controls(state, state.worker.is_some());
        }
        if let Some(worker) = &state.admission_worker {
            worker.cancel.store(true, Ordering::Release);
            state.admission_phase = admission::Phase::Unknown;
            status(
                state,
                if closing {
                    text::CLOSE_WAIT
                } else {
                    text::CANCELLING
                },
            );
        }
        if let Some(launch) = &state.launch {
            launch.cancel();
            status(
                state,
                if closing {
                    text::CLOSE_WAIT
                } else {
                    text::CANCELLING
                },
            );
            controls(state, true);
            unsafe {
                EnableWindow(state.buttons[1], 0);
            }
            return;
        }
        if launch_busy(state) {
            status(state, text::RUN_CLEANUP_UNKNOWN);
            controls(state, true);
            unsafe {
                EnableWindow(state.buttons[1], 0);
            }
            return;
        }
        if state.confirmation.take().is_some() && !closing {
            status(state, text::VERIFIED);
            controls(state, false);
            return;
        }
        if let Some(w) = &state.worker {
            w.cancel.store(true, Ordering::Release);
            status(
                state,
                if w.operation == Operation::Remove {
                    if closing {
                        text::REMOVE_CLOSE_WAIT
                    } else {
                        text::REMOVE_CANCELLING
                    }
                } else if closing {
                    text::CLOSE_WAIT
                } else {
                    text::CANCELLING
                },
            );
            controls(state, true);
            unsafe {
                EnableWindow(state.buttons[1], 0);
            }
        }
    }
    fn registration_completed(
        state: &mut State,
        result: Result<(), io::ErrorKind>,
        verified: bool,
    ) {
        state.verified = verified;
        state.shell_registered = result.is_ok();
        state.remove_eligible = false;
        if verified {
            state.admission_phase = admission::Phase::Existing;
        }
        status(
            state,
            if result.is_ok() {
                text::SHELL_REGISTERED
            } else {
                text::SHELL_REGISTER_FAILED
            },
        );
    }
    fn removal_message(r: &LocalRemovalReport) -> &'static str {
        if r.kind == Some(io::ErrorKind::Unsupported) && !r.mutation_may_have_occurred {
            return text::SHELL_REMOVE_GUIDE;
        }
        if r.kind.is_none()
            && r.outer_namespace_absence_confirmed
            && r.accepted_root_dispositions == 4
        {
            text::REMOVED
        } else if r.mutation_may_have_occurred
            || r.accepted_file_dispositions > 0
            || r.accepted_root_dispositions > 0
        {
            text::REMOVAL_PARTIAL
        } else {
            text::REMOVAL_REFUSED
        }
    }

    fn start_launch(hwnd: HWND, state: &mut State) {
        if !state.verified
            || admission_busy(state)
            || state.worker.is_some()
            || launch_busy(state)
            || state.picker_active
            || state.closing
            || state.confirmation.is_some()
        {
            return;
        }
        let roots = match request_roots(state) {
            Ok(roots) => roots,
            Err(()) => {
                invalidate(state);
                status(state, text::INVALID_ROOTS);
                controls(state, false);
                return;
            }
        };
        if unsafe { SetTimer(hwnd, TIMER, 75, None) } == 0 {
            status(state, text::WORKER_FAILED);
            return;
        }
        let request = crate::installer_launch::Request {
            bundle_root: roots.bundle_root,
            journal_root: roots.journal_root,
        };
        match crate::installer_launch::Supervisor::start(request, Duration::from_secs(300)) {
            Ok(launch) => {
                // launch_busy already rejects an unknown retained owner; only an
                // accepted explicit retry clears a previous confirmed failure.
                crate::installer_launch::clear_confirmed_failure_for_retry(
                    &mut state.launch_failure,
                );
                state.launch = Some(launch);
                status(state, text::RUN_STARTING);
                controls(state, true);
            }
            Err(error) => {
                unsafe {
                    KillTimer(hwnd, TIMER);
                }
                crate::installer_launch::retain_completion(
                    &mut state.launch_failure,
                    crate::installer_launch::Completion::InspectionFailed(error.kind()),
                );
                status(state, text::RUN_FAILED);
                controls(state, false);
            }
        }
    }
    fn poll_launch(hwnd: HWND, state: &mut State) {
        let Some(launch) = &mut state.launch else {
            return;
        };
        let event = launch.poll();
        match event {
            crate::installer_launch::Event::Pending => {}
            crate::installer_launch::Event::Started(started) => {
                if started.process_id == 0 || started.creation_filetime == 0 {
                    if let Some(launch) = &state.launch {
                        launch.cancel();
                    }
                    status(state, text::RUN_FAILED);
                    return;
                }
                if !state.closing
                    && state
                        .launch
                        .as_ref()
                        .is_some_and(|launch| !launch.is_cancelled())
                {
                    status(state, text::RUNNING);
                    state.closing = true;
                    // Main's post-HWND tail retains this supervisor and Job until app exit.
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                }
            }
            crate::installer_launch::Event::Complete(outcome) => {
                state.launch.take();
                unsafe {
                    KillTimer(hwnd, TIMER);
                }
                if outcome.kind().is_none() && outcome.terminal_confirmed() {
                    state.closing = true;
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    return;
                }
                let message = if !outcome.terminal_confirmed() {
                    text::RUN_CLEANUP_UNKNOWN.to_owned()
                } else if outcome.was_cancelled() {
                    text::CANCELLED.to_owned()
                } else if outcome.kind() == Some(io::ErrorKind::TimedOut) {
                    text::TIMED_OUT.to_owned()
                } else if let crate::installer_launch::Completion::Exited(exit) = &outcome {
                    text::run_exit_failure(exit.code)
                } else if outcome.may_have_executed() {
                    text::RUN_FAILED_AFTER_START.to_owned()
                } else {
                    text::RUN_FAILED.to_owned()
                };
                crate::installer_launch::retain_completion(&mut state.launch_failure, outcome);
                if state.closing {
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                    return;
                }
                invalidate(state);
                status(state, &message);
                controls(state, false);
            }
        }
    }
    fn poll(hwnd: HWND, state: &mut State) {
        if state.launch.is_some() {
            poll_launch(hwnd, state);
            return;
        }
        if !state
            .worker
            .as_ref()
            .is_some_and(|w| w.handle.is_finished())
        {
            return;
        }
        let Some(w) = state.worker.take() else {
            return;
        };
        let report = w.rx.try_recv();
        let joined = w.handle.join().is_ok();
        unsafe {
            KillTimer(hwnd, TIMER);
        }
        if state.closing {
            unsafe {
                DestroyWindow(hwnd);
            }
            return;
        }
        if w.revision != state.admission_revision {
            invalidate(state);
            status(state, text::INPUT_CHANGED);
            controls(state, false);
            return;
        }
        match report {
            Ok(Report::Journal(r, tool_summary, shell)) if joined => {
                let summary = wide(tool_summary.as_deref().unwrap_or(""));
                unsafe {
                    SetWindowTextW(state.tools_result, summary.as_ptr());
                    ShowWindow(
                        state.tools_result,
                        if tool_summary
                            .as_deref()
                            .is_some_and(|text| !text.trim().is_empty())
                        {
                            SW_SHOW
                        } else {
                            SW_HIDE
                        },
                    );
                }
                state.verified = r.kind.is_none() && r.current_consistency_verified;
                state.remove_eligible = state.verified && w.operation == Operation::Verify;
                if state.verified || r.kind == Some(io::ErrorKind::AlreadyExists) {
                    state.admission_phase = admission::Phase::Existing;
                } else if w.operation == Operation::Prepare {
                    queue_admission(hwnd, state, false);
                }
                let message = match r.kind {
                    Some(io::ErrorKind::Interrupted) => text::CANCELLED,
                    Some(io::ErrorKind::TimedOut) => text::TIMED_OUT,
                    Some(io::ErrorKind::AlreadyExists) => text::EXISTING,
                    Some(io::ErrorKind::NotFound) => text::MISSING,
                    Some(io::ErrorKind::InvalidData) => text::SOURCE_CHANGED,
                    Some(io::ErrorKind::InvalidInput) => {
                        if w.operation == Operation::Prepare {
                            text::INVALID
                        } else {
                            text::INVALID_ROOTS
                        }
                    }
                    Some(io::ErrorKind::PermissionDenied) => text::NO_ACCESS,
                    Some(_) => text::FAILED,
                    None if state.verified => {
                        if w.operation == Operation::Verify {
                            text::VERIFIED
                        } else {
                            text::INSTALLED
                        }
                    }
                    None => text::FAILED,
                };
                if let Some(result) = shell {
                    state.shell_registered = result.is_ok();
                    if state.shell_registered {
                        state.remove_eligible = false;
                    }
                    status(
                        state,
                        if result.is_ok() && w.operation == Operation::Verify {
                            text::VERIFIED
                        } else if result.is_ok() {
                            text::SHELL_REGISTERED
                        } else {
                            text::SHELL_REGISTER_FAILED
                        },
                    );
                } else {
                    status(state, message);
                }
            }
            Ok(Report::ShellRegistration(result, verified)) if joined => {
                registration_completed(state, result, verified);
            }
            Ok(Report::ToolPathsRejected) if joined => {
                invalidate(state);
                status(state, text::TOOL_PATHS_REJECTED);
            }
            Ok(Report::Removal(r)) if joined => {
                invalidate(state);
                let message = removal_message(&r);
                status(state, message);
                queue_admission(hwnd, state, false);
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) | Ok(_) => {
                invalidate(state);
                status(
                    state,
                    if w.operation == Operation::Remove {
                        text::REMOVAL_WORKER_FAILED
                    } else {
                        text::WORKER_FAILED
                    },
                );
                queue_admission(hwnd, state, false);
            }
        }
        controls(state, false);
    }
    #[derive(Clone, Copy, Default, Debug)]
    struct Placement {
        id: usize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    }
    fn flow_row(
        available: i32,
        y: &mut i32,
        widths: &[i32],
        ids: &[usize],
        height: i32,
    ) -> Vec<Placement> {
        let gap = t::CONTROL_GAP as i32;
        let mut x = 0;
        let mut result = Vec::with_capacity(ids.len());
        for (&desired, &id) in widths.iter().zip(ids) {
            let width = desired.min(available).max(1);
            if x > 0 && x + width > available {
                x = 0;
                *y += height + gap;
            }
            result.push(Placement {
                id,
                x,
                y: *y,
                width,
                height,
            });
            x += width + gap;
        }
        if !result.is_empty() {
            *y += height;
        }
        result
    }
    struct FormMeasurements {
        intro_height: i32,
        appearance_height: i32,
        hint_height: i32,
        status_height: i32,
        result_height: i32,
        command_widths: [i32; 5],
        check_widths: [i32; 3],
        picker_width: i32,
    }
    fn form_layout(
        available: i32,
        packaged: bool,
        measured: FormMeasurements,
    ) -> (Vec<Placement>, i32) {
        let FormMeasurements {
            intro_height,
            appearance_height,
            hint_height,
            status_height,
            result_height,
            command_widths,
            check_widths,
            picker_width,
        } = measured;
        let margin = t::PAGE_MARGIN as i32;
        let width = (available - margin * 2).max(120);
        let control = t::CONTROL_HEIGHT as i32;
        let gap = t::CONTROL_GAP as i32;
        let section = t::SECTION_GAP as i32;
        let label_gap = t::LABEL_GAP as i32;
        let mut positions = Vec::with_capacity(29);
        let mut y = margin;
        let theme_width = 152;
        let label_width = 76;
        let scale_width = 92;
        positions.push(Placement {
            id: 304,
            x: 0,
            y,
            width: width.min(140),
            height: control,
        });
        if width >= 140 + label_width * 2 + theme_width + scale_width + gap * 4 {
            let theme_x = width - (label_width * 2 + theme_width + scale_width + gap * 3);
            positions.push(Placement {
                id: 306,
                x: theme_x,
                y,
                width: label_width,
                height: control,
            });
            positions.push(Placement {
                id: 210,
                x: theme_x + label_width + gap,
                y,
                width: theme_width,
                height: control,
            });
            positions.push(Placement {
                id: 307,
                x: width - scale_width - label_width - gap,
                y,
                width: label_width,
                height: control,
            });
            positions.push(Placement {
                id: 211,
                x: width - scale_width,
                y,
                width: scale_width,
                height: control,
            });
            y += control;
        } else {
            y += control + gap;
            if width >= label_width * 2 + theme_width + scale_width + gap * 3 {
                positions.push(Placement {
                    id: 306,
                    x: 0,
                    y,
                    width: label_width,
                    height: control,
                });
                positions.push(Placement {
                    id: 210,
                    x: label_width + gap,
                    y,
                    width: theme_width,
                    height: control,
                });
                positions.push(Placement {
                    id: 307,
                    x: width - scale_width - label_width - gap,
                    y,
                    width: label_width,
                    height: control,
                });
                positions.push(Placement {
                    id: 211,
                    x: width - scale_width,
                    y,
                    width: scale_width,
                    height: control,
                });
                y += control;
            } else {
                for (label, id, desired) in [(306, 210, theme_width), (307, 211, scale_width)] {
                    if width >= label_width + gap + desired {
                        positions.push(Placement {
                            id: label,
                            x: 0,
                            y,
                            width: label_width,
                            height: control,
                        });
                        positions.push(Placement {
                            id,
                            x: label_width + gap,
                            y,
                            width: desired,
                            height: control,
                        });
                        y += control + gap;
                    } else {
                        positions.push(Placement {
                            id: label,
                            x: 0,
                            y,
                            width,
                            height: 20,
                        });
                        y += 20 + label_gap;
                        positions.push(Placement {
                            id,
                            x: 0,
                            y,
                            width,
                            height: control,
                        });
                        y += control + gap;
                    }
                }
                y -= gap;
            }
        }
        y += gap;
        positions.push(Placement {
            id: 308,
            x: 0,
            y,
            width,
            height: appearance_height,
        });
        y += appearance_height + gap;
        positions.push(Placement {
            id: 300,
            x: 0,
            y,
            width,
            height: intro_height,
        });
        y += intro_height + section;
        for i in if packaged { 2..4 } else { 0..4 } {
            positions.push(Placement {
                id: 110 + i,
                x: 0,
                y,
                width,
                height: 20,
            });
            y += 20 + label_gap;
            let picker = if packaged && i == 2 {
                Some(209)
            } else if !packaged && i == 0 {
                Some(205)
            } else {
                None
            };
            let input_height = if packaged && i == 3 { 48 } else { control };
            if let Some(id) = picker {
                if width >= 160 + gap + picker_width {
                    positions.push(Placement {
                        id: 101 + i,
                        x: 0,
                        y,
                        width: width - picker_width - gap,
                        height: input_height,
                    });
                    positions.push(Placement {
                        id,
                        x: width - picker_width,
                        y,
                        width: picker_width,
                        height: control,
                    });
                    y += input_height;
                } else {
                    positions.push(Placement {
                        id: 101 + i,
                        x: 0,
                        y,
                        width,
                        height: input_height,
                    });
                    y += input_height + gap;
                    positions.push(Placement {
                        id,
                        x: 0,
                        y,
                        width: picker_width.min(width),
                        height: control,
                    });
                    y += control;
                }
            } else {
                positions.push(Placement {
                    id: 101 + i,
                    x: 0,
                    y,
                    width,
                    height: input_height,
                });
                y += input_height;
            }
            y += 16;
        }
        y += section - 16;
        positions.push(Placement {
            id: 305,
            x: 0,
            y,
            width,
            height: 24,
        });
        y += 24 + label_gap;
        positions.extend(flow_row(
            width,
            &mut y,
            &check_widths,
            &[206, 207, 208],
            control,
        ));
        y += label_gap;
        positions.push(Placement {
            id: 302,
            x: 0,
            y,
            width,
            height: hint_height,
        });
        y += hint_height + section;
        positions.extend(flow_row(
            width,
            &mut y,
            &command_widths,
            &[201, 202, 203, 204, 212],
            control,
        ));
        y += 16;
        positions.push(Placement {
            id: 301,
            x: 0,
            y,
            width,
            height: status_height,
        });
        y += status_height;
        if result_height > 0 {
            y += 16;
            positions.push(Placement {
                id: 303,
                x: 0,
                y,
                width,
                height: result_height,
            });
            y += result_height;
        }
        for place in &mut positions {
            place.x += margin;
        }
        (positions, y + margin)
    }
    fn visual(state: &State) -> &RefCell<Presentation> {
        // The Box is retained by run() until DestroyWindow has destroyed every child HWND.
        unsafe { &*state.presentation }
    }
    fn presentation(hwnd: HWND) -> Option<&'static RefCell<Presentation>> {
        let pointer =
            unsafe { GetPropW(hwnd, PRESENTATION_KEY.as_ptr()) } as *const RefCell<Presentation>;
        if pointer.is_null() {
            None
        } else {
            Some(unsafe { &*pointer })
        }
    }
    const PRESENTATION_KEY: [u16; 13] =
        [100, 108, 112, 97, 100, 46, 118, 105, 115, 117, 97, 108, 0];
    fn control(state: &State, id: usize) -> HWND {
        match id {
            101..=104 => state.edits[id - 101],
            110..=113 => state.labels[id - 110],
            201..=204 => state.buttons[id - 201],
            205 => state.picker_button,
            212 => state.launch_button,
            206..=208 => state.tool_checks[id - 206],
            209 => state.target_picker_button,
            210 => state.theme,
            211 => state.scale,
            300 => state.intro,
            301 => state.status,
            302 => state.tools_hint,
            303 => state.tools_result,
            304 => state.heading,
            305 => state.tools_heading,
            306 => state.theme_label,
            307 => state.scale_label,
            308 => state.appearance_help,
            _ => std::ptr::null_mut(),
        }
    }
    fn control_text(hwnd: HWND) -> Vec<u16> {
        let length = unsafe { GetWindowTextLengthW(hwnd) }.clamp(0, 32767);
        let mut value = vec![0u16; length as usize + 1];
        let read =
            unsafe { GetWindowTextW(hwnd, value.as_mut_ptr(), value.len() as i32) }.max(0) as usize;
        value.truncate(read);
        value
    }
    fn text_height(hwnd: HWND, font: HFONT, width: i32, dpi: u32, maximum: i32) -> i32 {
        let value = control_text(hwnd);
        let dc = unsafe { GetDC(hwnd) };
        if dc.is_null() {
            return 40;
        }
        let previous = unsafe { SelectObject(dc, font) };
        let mut rect = RECT {
            right: px(width as f32, dpi).max(1),
            ..Default::default()
        };
        unsafe {
            DrawTextW(
                dc,
                value.as_ptr(),
                value.len() as i32,
                &mut rect,
                DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX,
            );
            SelectObject(dc, previous);
            ReleaseDC(hwnd, dc);
        }
        ((rect.bottom - rect.top) * 96 / dpi as i32 + 4).clamp(20, maximum)
    }
    fn text_width(hwnd: HWND, font: HFONT, dpi: u32) -> i32 {
        let value = control_text(hwnd);
        let dc = unsafe { GetDC(hwnd) };
        if dc.is_null() {
            return 96;
        }
        let previous = unsafe { SelectObject(dc, font) };
        let mut size = SIZE::default();
        unsafe {
            GetTextExtentPoint32W(dc, value.as_ptr(), value.len() as i32, &mut size);
            SelectObject(dc, previous);
            ReleaseDC(hwnd, dc);
        }
        size.cx * 96 / dpi as i32
    }
    fn layout(hwnd: HWND, state: &State) {
        let mut rect = RECT::default();
        if unsafe { GetClientRect(hwnd, &mut rect) } == 0 {
            return;
        }
        let Ok(view) = visual(state).try_borrow() else {
            return;
        };
        let dpi = view.dpi;
        let available = (rect.right - rect.left) * 96 / dpi as i32;
        let width = (available - 2 * t::PAGE_MARGIN as i32).max(120);
        let intro = text_height(state.intro, view.fonts.body.handle, width, dpi, 64);
        let appearance = text_height(
            state.appearance_help,
            view.fonts.small.handle,
            width,
            dpi,
            88,
        );
        let hint = text_height(state.tools_hint, view.fonts.small.handle, width, dpi, 88);
        let status = text_height(state.status, view.fonts.body.handle, width, dpi, 96);
        let result = if unsafe { IsWindowVisible(state.tools_result) } != 0 {
            text_height(state.tools_result, view.fonts.body.handle, width, dpi, 240)
        } else {
            0
        };
        let commands = [
            state.buttons[0],
            state.buttons[1],
            state.buttons[2],
            state.buttons[3],
            state.launch_button,
        ]
        .map(|h| {
            (text_width(h, view.fonts.body.handle, dpi) + 2 * t::CONTROL_PADDING_X as i32)
                .max(t::CONTROL_MIN_WIDTH as i32)
        });
        let checks = state
            .tool_checks
            .map(|h| text_width(h, view.fonts.body.handle, dpi) + 28);
        let picker = if state.packaged_source.is_some() {
            state.target_picker_button
        } else {
            state.picker_button
        };
        let picker_width = (text_width(picker, view.fonts.body.handle, dpi)
            + 2 * t::CONTROL_PADDING_X as i32)
            .max(t::CONTROL_MIN_WIDTH as i32);
        let (positions, height) = form_layout(
            available,
            state.packaged_source.is_some(),
            FormMeasurements {
                intro_height: intro,
                appearance_height: appearance,
                hint_height: hint,
                status_height: status,
                result_height: result,
                command_widths: commands,
                check_widths: checks,
                picker_width,
            },
        );
        let content = px(height as f32, dpi);
        let scroll = view.scroll.clamp(0, (content - rect.bottom).max(0));
        drop(view);
        if let Ok(mut view) = visual(state).try_borrow_mut() {
            view.content_height = content;
            view.scroll = scroll;
        }
        unsafe {
            let scroll_info = SCROLLINFO {
                cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                nMin: 0,
                nMax: content.saturating_sub(1),
                nPage: rect.bottom.max(0) as u32,
                nPos: scroll,
                ..Default::default()
            };
            SetScrollInfo(hwnd, SB_VERT, &scroll_info, 1);
            for p in positions {
                let child = control(state, p.id);
                if child.is_null() {
                    continue;
                }
                let w = px(p.width as f32, dpi);
                let h = px(p.height as f32, dpi);
                MoveWindow(
                    child,
                    px(p.x as f32, dpi),
                    px(p.y as f32, dpi) - scroll,
                    w,
                    h,
                    1,
                );
                if (101..=104).contains(&p.id) {
                    let radius = px(f32::from(t::RADIUS) * 2.0, dpi);
                    let region = CreateRoundRectRgn(0, 0, w + 1, h + 1, radius, radius);
                    if !region.is_null() && SetWindowRgn(child, region, 1) == 0 {
                        DeleteObject(region);
                    }
                    let border = px(1.0, dpi);
                    let margin = (px(t::CONTROL_PADDING_X, dpi) - border).clamp(0, 65535) as usize;
                    SendMessageW(
                        child,
                        EM_SETMARGINS,
                        (EC_LEFTMARGIN | EC_RIGHTMARGIN) as usize,
                        (margin | (margin << 16)) as isize,
                    );
                    if GetWindowLongPtrW(child, GWL_STYLE) as u32 & ES_MULTILINE as u32 != 0 {
                        let padding = (px(t::LABEL_GAP, dpi) - border).max(0);
                        let mut actual_client = RECT::default();
                        GetClientRect(child, &mut actual_client);
                        let edit_rect = RECT {
                            left: margin as i32,
                            top: padding,
                            right: actual_client.right - margin as i32,
                            bottom: actual_client.bottom - padding,
                        };
                        SendMessageW(child, EM_SETRECTNP, 0, (&edit_rect as *const RECT) as isize);
                    }
                }
            }
            InvalidateRect(hwnd, std::ptr::null(), 1);
        }
    }
    fn apply_fonts(state: &State) {
        let Ok(view) = visual(state).try_borrow() else {
            return;
        };
        unsafe {
            for id in [
                101, 102, 103, 104, 110, 111, 112, 113, 201, 202, 203, 204, 205, 206, 207, 208,
                209, 210, 211, 212, 300, 301, 302, 303, 304, 305, 306, 307, 308,
            ] {
                let handle = control(state, id);
                let font = match id {
                    304 => view.fonts.title.handle,
                    305 => view.fonts.heading.handle,
                    302 | 306 | 307 | 308 => view.fonts.small.handle,
                    _ => view.fonts.body.handle,
                };
                SendMessageW(handle, WM_SETFONT, font as usize, 1);
                if (101..=104).contains(&id) {
                    SetWindowPos(
                        handle,
                        std::ptr::null_mut(),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );
                }
            }
            for combo in [state.theme, state.scale] {
                fit_combo_height(combo, view.dpi);
            }
        }
    }
    fn reveal_control(hwnd: HWND, state: &State, id: usize) {
        let child = control(state, id);
        let mut rect = RECT::default();
        let mut viewport = RECT::default();
        if child.is_null()
            || unsafe { GetWindowRect(child, &mut rect) } == 0
            || unsafe { GetClientRect(hwnd, &mut viewport) } == 0
        {
            return;
        }
        let mut origin = POINT {
            x: rect.left,
            y: rect.top,
        };
        unsafe {
            ScreenToClient(hwnd, &mut origin);
        }
        let delta = if origin.y < 0 {
            origin.y
        } else if origin.y + rect.bottom - rect.top > viewport.bottom {
            origin.y + rect.bottom - rect.top - viewport.bottom
        } else {
            0
        };
        if delta != 0 {
            if let Ok(mut view) = visual(state).try_borrow_mut() {
                view.scroll = (view.scroll + delta).max(0);
            }
            layout(hwnd, state);
        }
    }
    fn line_height(hwnd: HWND) -> i32 {
        let dc = unsafe { GetDC(hwnd) };
        if dc.is_null() {
            return 16;
        }
        let font = unsafe { SendMessageW(hwnd, WM_GETFONT, 0, 0) } as HFONT;
        let previous = unsafe { SelectObject(dc, font) };
        let mut metric = TEXTMETRICW::default();
        let height = if unsafe { GetTextMetricsW(dc, &mut metric) } != 0 {
            metric.tmHeight + 2
        } else {
            16
        };
        unsafe {
            SelectObject(dc, previous);
            ReleaseDC(hwnd, dc);
        }
        height.max(1)
    }
    fn combo_text(id: i32, index: isize) -> &'static str {
        let choices = match id {
            210 => text::THEME_CHOICES,
            211 => text::SCALE_CHOICES,
            _ => return "",
        };
        usize::try_from(index)
            .ok()
            .and_then(|i| choices.get(i))
            .copied()
            .unwrap_or("")
    }
    fn paint_combo_text(
        dc: HDC,
        mut rect: RECT,
        label: &str,
        foreground: COLORREF,
        view: &Presentation,
    ) {
        let label = wide(label);
        unsafe {
            SelectObject(dc, view.fonts.body.handle);
            SetBkMode(dc, TRANSPARENT as i32);
            SetTextColor(dc, foreground);
            rect.left += px(t::CONTROL_PADDING_X, view.dpi);
            rect.right -= px(t::CONTROL_PADDING_X, view.dpi);
            DrawTextW(
                dc,
                label.as_ptr(),
                (label.len() - 1) as i32,
                &mut rect,
                DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS,
            );
        }
    }
    fn paint_combo_closed(hwnd: HWND, dc: HDC, view: &Presentation, hot: bool) {
        let saved = unsafe { SaveDC(dc) };
        if saved == 0 {
            return;
        }
        let mut rect = RECT::default();
        unsafe {
            GetClientRect(hwnd, &mut rect);
        }
        let p = view.palette;
        let enabled = unsafe { IsWindowEnabled(hwnd) } != 0;
        let focus = enabled && unsafe { GetFocus() } == hwnd;
        let open = unsafe { SendMessageW(hwnd, CB_GETDROPPEDSTATE, 0, 0) } != 0;
        let fill = if !enabled || hot || open {
            p.secondary
        } else {
            p.surface
        };
        let border = if focus {
            p.focus
        } else if hot && enabled {
            mix(p.border, p.foreground, 28)
        } else {
            p.border
        };
        unsafe {
            FillRect(dc, &rect, view.brushes.background);
        }
        rounded(
            dc,
            rect,
            fill,
            border,
            px(if focus { 1.5 } else { 1.0 }, view.dpi),
            px(f32::from(t::RADIUS), view.dpi),
        );
        let foreground = if enabled { p.foreground } else { p.muted };
        let index = unsafe { SendMessageW(hwnd, CB_GETCURSEL, 0, 0) };
        let label = combo_text(unsafe { GetDlgCtrlID(hwnd) }, index);
        let arrow_width = px(24.0, view.dpi);
        let mut label_rect = rect;
        label_rect.right -= arrow_width;
        paint_combo_text(dc, label_rect, label, foreground, view);
        // Native ComboBox retains its own arrow hit region and keyboard behavior; this only paints.
        let center_x = rect.right - arrow_width / 2;
        let center_y = (rect.bottom + rect.top) / 2;
        let half = px(3.0, view.dpi);
        unsafe {
            let pen = CreatePen(PS_SOLID, px(1.0, view.dpi).max(1), foreground);
            if !pen.is_null() {
                let previous = SelectObject(dc, pen);
                MoveToEx(
                    dc,
                    center_x - half,
                    center_y - half / 2,
                    std::ptr::null_mut(),
                );
                LineTo(dc, center_x, center_y + half / 2);
                LineTo(dc, center_x + half, center_y - half / 2);
                SelectObject(dc, previous);
                DeleteObject(pen);
            }
            RestoreDC(dc, saved);
        }
    }
    fn paint_combo_item(item: &DRAWITEMSTRUCT, view: &Presentation) -> bool {
        if item.CtlType != ODT_COMBOBOX || !matches!(item.CtlID, 210 | 211) {
            return false;
        }
        let saved = unsafe { SaveDC(item.hDC) };
        if saved == 0 {
            return false;
        }
        let disabled = item.itemState & ODS_DISABLED != 0;
        let selected = item.itemState & ODS_SELECTED != 0;
        let p = view.palette;
        let fill = if selected { p.secondary } else { p.surface };
        unsafe {
            SetDCBrushColor(item.hDC, fill);
            FillRect(item.hDC, &item.rcItem, GetStockObject(DC_BRUSH) as HBRUSH);
        }
        paint_combo_text(
            item.hDC,
            item.rcItem,
            combo_text(item.CtlID as i32, item.itemID as isize),
            if disabled { p.muted } else { p.foreground },
            view,
        );
        if item.itemState & ODS_FOCUS != 0 {
            let mut focus = item.rcItem;
            let inset = px(1.0, view.dpi);
            focus.left += inset;
            focus.right -= inset;
            focus.top += inset;
            focus.bottom -= inset;
            rounded(
                item.hDC,
                focus,
                fill,
                p.focus,
                px(1.5, view.dpi),
                px(f32::from(t::RADIUS), view.dpi),
            );
            paint_combo_text(
                item.hDC,
                item.rcItem,
                combo_text(item.CtlID as i32, item.itemID as isize),
                if disabled { p.muted } else { p.foreground },
                view,
            );
        }
        unsafe {
            RestoreDC(item.hDC, saved);
        }
        true
    }
    fn fit_combo_height(hwnd: HWND, dpi: u32) {
        // OWNERDRAWFIXED enables selection height. Measure native chrome instead of assuming its thickness.
        // Keep selection and popup item height equal (CB_SETITEMHEIGHT documented constraint).
        let target = px(t::CONTROL_HEIGHT, dpi);
        let mut selection = (target - px(4.0, dpi)).max(line_height(hwnd));
        for _ in 0..4 {
            unsafe {
                if SendMessageW(hwnd, CB_SETITEMHEIGHT, 0, selection as isize) == CB_ERR as isize
                    || SendMessageW(hwnd, CB_SETITEMHEIGHT, usize::MAX, selection as isize)
                        == CB_ERR as isize
                {
                    break;
                }
            }
            let mut bounds = RECT::default();
            if unsafe { GetWindowRect(hwnd, &mut bounds) } == 0 {
                break;
            }
            let difference = target - (bounds.bottom - bounds.top);
            if difference == 0 {
                break;
            }
            selection = (selection + difference).clamp(line_height(hwnd), target);
        }
    }

    fn edit_border(hwnd: HWND, hovered: bool) -> bool {
        let owner = unsafe { GetParent(hwnd) };
        let Some(cell) = presentation(owner) else {
            return false;
        };
        let Ok(view) = cell.try_borrow() else {
            return false;
        };
        let dc = unsafe { GetWindowDC(hwnd) };
        if dc.is_null() {
            return false;
        }
        let saved = unsafe { SaveDC(dc) };
        if saved == 0 {
            unsafe {
                ReleaseDC(hwnd, dc);
            }
            return false;
        }
        let mut bounds = RECT::default();
        let mut client = RECT::default();
        let mut origin = POINT::default();
        unsafe {
            GetWindowRect(hwnd, &mut bounds);
            GetClientRect(hwnd, &mut client);
            ClientToScreen(hwnd, &mut origin);
        }
        let rect = RECT {
            left: 0,
            top: 0,
            right: bounds.right - bounds.left,
            bottom: bounds.bottom - bounds.top,
        };
        let enabled = unsafe { IsWindowEnabled(hwnd) } != 0;
        let readonly =
            unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32 & ES_READONLY as u32 != 0;
        let fill = if !enabled || readonly {
            view.palette.secondary
        } else {
            view.palette.surface
        };
        let focus = enabled && unsafe { GetFocus() } == hwnd;
        let border = if focus {
            view.palette.focus
        } else if hovered && enabled {
            mix(view.palette.border, view.palette.foreground, 28)
        } else {
            view.palette.border
        };
        unsafe {
            ExcludeClipRect(
                dc,
                origin.x - bounds.left,
                origin.y - bounds.top,
                origin.x - bounds.left + client.right,
                origin.y - bounds.top + client.bottom,
            );
        }
        rounded(
            dc,
            rect,
            fill,
            border,
            px(if focus { 1.5 } else { 1.0 }, view.dpi),
            px(f32::from(t::RADIUS), view.dpi),
        );
        unsafe {
            RestoreDC(dc, saved);
            ReleaseDC(hwnd, dc);
        }
        true
    }
    unsafe extern "system" fn control_proc(
        hwnd: HWND,
        message: u32,
        w: WPARAM,
        l: LPARAM,
        id: usize,
        hovered: usize,
    ) -> LRESULT {
        if message == WM_NCDESTROY {
            unsafe {
                RemoveWindowSubclass(hwnd, Some(control_proc), id);
                return DefSubclassProc(hwnd, message, w, l);
            }
        }
        let control_id = unsafe { GetDlgCtrlID(hwnd) };
        let edit = (101..=104).contains(&control_id);
        let combo = matches!(control_id, 210 | 211);
        if edit && message == WM_NCCALCSIZE && l != 0 {
            let result = unsafe { DefSubclassProc(hwnd, message, w, l) };
            let rect = if w != 0 {
                unsafe { &mut (*(l as *mut NCCALCSIZE_PARAMS)).rgrc[0] }
            } else {
                unsafe { &mut *(l as *mut RECT) }
            };
            // Sole custom border, without WS_BORDER; retain native EDIT caret/IME client.
            let border = presentation(unsafe { GetParent(hwnd) })
                .and_then(|cell| cell.try_borrow().ok().map(|view| px(1.0, view.dpi)))
                .unwrap_or(1);
            rect.left += border;
            rect.right -= border;
            rect.top += border;
            rect.bottom -= border;
            if unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32 & ES_MULTILINE as u32 == 0 {
                let line = line_height(hwnd).min((rect.bottom - rect.top).max(1));
                let inset = (rect.bottom - rect.top - line).max(0) / 2;
                rect.top += inset;
                rect.bottom = rect.top + line;
            }
            return result;
        }
        if combo && matches!(message, WM_PAINT | WM_PRINTCLIENT | WM_PRINT) {
            // First run the native procedure for validation and internal state; overlay paint only.
            let result = unsafe { DefSubclassProc(hwnd, message, w, l) };
            if let Some(cell) = presentation(unsafe { GetParent(hwnd) })
                && let Ok(view) = cell.try_borrow()
            {
                let dc = if message == WM_PAINT {
                    unsafe { GetDC(hwnd) }
                } else {
                    w as HDC
                };
                if !dc.is_null() {
                    paint_combo_closed(hwnd, dc, &view, hovered != 0);
                    if message == WM_PAINT {
                        unsafe {
                            ReleaseDC(hwnd, dc);
                        }
                    }
                }
            }
            return result;
        }
        if edit && message == WM_NCPAINT && edit_border(hwnd, hovered != 0) {
            return 0;
        }
        if message == WM_SETFOCUS || message == WM_KILLFOCUS {
            let result = unsafe { DefSubclassProc(hwnd, message, w, l) };
            unsafe {
                RedrawWindow(
                    hwnd,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    RDW_INVALIDATE | RDW_FRAME,
                );
            }
            if message == WM_SETFOCUS {
                unsafe {
                    PostMessageW(
                        GetParent(hwnd),
                        FOCUS_CONTROL,
                        GetDlgCtrlID(hwnd) as usize,
                        0,
                    );
                }
            }
            return result;
        }
        if (edit || combo) && message == WM_MOUSEMOVE && hovered == 0 {
            let mut event = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            unsafe {
                TrackMouseEvent(&mut event);
                SetWindowSubclass(hwnd, Some(control_proc), id, 1);
                RedrawWindow(
                    hwnd,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    RDW_INVALIDATE | RDW_FRAME,
                );
            }
        } else if (edit || combo) && message == WM_MOUSELEAVE {
            unsafe {
                SetWindowSubclass(hwnd, Some(control_proc), id, 0);
                RedrawWindow(
                    hwnd,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    RDW_INVALIDATE | RDW_FRAME,
                );
            }
        }
        unsafe { DefSubclassProc(hwnd, message, w, l) }
    }
    fn repaint_appearance(hwnd: HWND, state: &State) {
        let selected = unsafe { SendMessageW(state.theme, CB_GETCURSEL, 0, 0) };
        let appearance = match selected {
            0 => Appearance::System,
            1 => Appearance::Light,
            2 => Appearance::Dark,
            _ => return,
        };
        let selected = unsafe { SendMessageW(state.scale, CB_GETCURSEL, 0, 0) };
        let scale = match selected {
            0 => 100,
            1 => 150,
            2 => 200,
            _ => return,
        };
        let Ok(mut view) = visual(state).try_borrow_mut() else {
            unsafe {
                PostMessageW(hwnd, APPEARANCE_CHANGED, 0, 0);
            }
            return;
        };
        let dpi = view.os_dpi.saturating_mul(scale) / 100;
        let old_fonts = if dpi != view.dpi {
            view.dpi = dpi;
            Some(std::mem::replace(&mut view.fonts, Fonts::new(dpi)))
        } else {
            None
        };
        view.appearance = appearance;
        let palette = Palette::new(appearance);
        let brushes = Brushes::new(palette);
        view.palette = palette;
        view.brushes = brushes;
        drop(view);
        apply_fonts(state);
        drop(old_fonts);
        let name = wide("Explorer");
        let dark = (palette.dark && !palette.contrast) as i32;
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                (&dark as *const i32).cast(),
                std::mem::size_of::<i32>() as u32,
            );
            for handle in state.buttons.into_iter().chain(state.tool_checks).chain([
                state.theme,
                state.scale,
                state.picker_button,
                state.target_picker_button,
                state.launch_button,
            ]) {
                SetWindowTheme(handle, name.as_ptr(), std::ptr::null());
            }
            RedrawWindow(
                hwnd,
                std::ptr::null(),
                std::ptr::null_mut(),
                RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN,
            );
        }
        layout(hwnd, state);
    }
    const APPLY_DPI: u32 = WM_APP + 19;
    const LAYOUT_VIEW: u32 = WM_APP + 20;
    const APPEARANCE_CHANGED: u32 = WM_APP + 21;
    // Native control calls may synchronously reenter. RefCell prevents aliased State access.
    unsafe extern "system" fn proc(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        if message == WM_MEASUREITEM && l != 0 {
            let item = unsafe { &mut *(l as *mut MEASUREITEMSTRUCT) };
            if item.CtlType == ODT_COMBOBOX && matches!(item.CtlID, 210 | 211) {
                let dpi = presentation(hwnd)
                    .and_then(|cell| cell.try_borrow().ok().map(|view| view.dpi))
                    .unwrap_or(96);
                item.itemHeight = px(t::CONTROL_HEIGHT - 4.0, dpi).max(1) as u32;
                return 1;
            }
        }
        if message == WM_DRAWITEM && l != 0 {
            let item = unsafe { &*(l as *const DRAWITEMSTRUCT) };
            if item.CtlType == ODT_COMBOBOX
                && matches!(item.CtlID, 210 | 211)
                && unsafe { GetParent(item.hwndItem) } == hwnd
                && let Some(cell) = presentation(hwnd)
                && let Ok(view) = cell.try_borrow()
                && paint_combo_item(item, &view)
            {
                return 1;
            }
        }
        if matches!(
            message,
            WM_CTLCOLORSTATIC
                | WM_CTLCOLORBTN
                | WM_CTLCOLOREDIT
                | WM_CTLCOLORLISTBOX
                | WM_ERASEBKGND
                | WM_NOTIFY
        ) && let Some(cell) = presentation(hwnd)
            && let Ok(view) = cell.try_borrow()
        {
            if message == WM_ERASEBKGND {
                let mut rect = RECT::default();
                unsafe {
                    GetClientRect(hwnd, &mut rect);
                    FillRect(w as _, &rect, view.brushes.background);
                }
                return 1;
            }
            if message == WM_NOTIFY && l != 0 {
                let header = unsafe { &*(l as *const NMHDR) };
                if header.code == NM_CUSTOMDRAW
                    && matches!(header.idFrom, 201..=209 | 212)
                    && unsafe { GetParent(header.hwndFrom) } == hwnd
                {
                    return paint_command(unsafe { &*(l as *const NMCUSTOMDRAW) }, &view);
                }
            } else if message != WM_NOTIFY {
                let child = l as HWND;
                let id = unsafe { GetDlgCtrlID(child) };
                let input = (101..=104).contains(&id)
                    || message == WM_CTLCOLOREDIT
                    || message == WM_CTLCOLORLISTBOX;
                let disabled = unsafe { IsWindowEnabled(child) } == 0;
                let readonly = (101..=104).contains(&id)
                    && unsafe { GetWindowLongPtrW(child, GWL_STYLE) } as u32 & ES_READONLY as u32
                        != 0;
                let brush = if input {
                    if disabled || readonly {
                        view.brushes.secondary
                    } else {
                        view.brushes.surface
                    }
                } else {
                    view.brushes.background
                };
                unsafe {
                    SetBkColor(
                        w as _,
                        if input {
                            if disabled || readonly {
                                view.palette.secondary
                            } else {
                                view.palette.surface
                            }
                        } else {
                            view.palette.background
                        },
                    );
                    SetTextColor(
                        w as _,
                        if disabled || matches!(id, 300 | 302 | 306 | 307 | 308) {
                            view.palette.muted
                        } else {
                            view.palette.foreground
                        },
                    );
                }
                return brush as LRESULT;
            }
        }
        if message == WM_NCDESTROY {
            unsafe {
                RemovePropW(hwnd, PRESENTATION_KEY.as_ptr());
            }
            return unsafe { DefWindowProcW(hwnd, message, w, l) };
        }
        if message == WM_DPICHANGED {
            let dpi = (w & 0xffff) as u32;
            if (96..=768).contains(&dpi) {
                if l != 0 {
                    let suggested = unsafe { *(l as *const RECT) };
                    unsafe {
                        SetWindowPos(
                            hwnd,
                            std::ptr::null_mut(),
                            suggested.left,
                            suggested.top,
                            suggested.right - suggested.left,
                            suggested.bottom - suggested.top,
                            SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                }
                // No borrowed pointer is posted. Font updates run after nested resize/paint dispatch returns.
                unsafe {
                    PostMessageW(hwnd, APPLY_DPI, dpi as usize, 0);
                }
            }
            return 0;
        }
        if message == WM_SETTINGCHANGE || message == WM_THEMECHANGED {
            unsafe {
                PostMessageW(hwnd, APPEARANCE_CHANGED, 0, 0);
            }
            return unsafe { DefWindowProcW(hwnd, message, w, l) };
        }
        if message == DM_GETDEFID {
            let id = unsafe { GetPropW(hwnd, DEFAULT_COMMAND_KEY.as_ptr()) } as usize;
            return if matches!(id, 201 | 203 | 212)
                && unsafe { IsWindowEnabled(GetDlgItem(hwnd, id as i32)) } != 0
            {
                ((DC_HASDEFID << 16) as usize | id) as LRESULT
            } else {
                0
            };
        }
        if message == WM_DESTROY {
            unsafe {
                RemovePropW(hwnd, DEFAULT_COMMAND_KEY.as_ptr());
                PostQuitMessage(0);
            }
            return 0;
        }
        if !matches!(
            message,
            WM_COMMAND
                | WM_TIMER
                | WM_CLOSE
                | WM_SIZE
                | WM_GETMINMAXINFO
                | PICKER_CLOSE
                | FOCUS_CONTROL
                | APPLY_DPI
                | LAYOUT_VIEW
                | APPEARANCE_CHANGED
                | WM_VSCROLL
                | WM_MOUSEWHEEL
        ) {
            return unsafe { DefWindowProcW(hwnd, message, w, l) };
        }
        if message == WM_GETMINMAXINFO {
            let m = unsafe { &mut *(l as *mut MINMAXINFO) };
            let dpi = unsafe { GetDpiForWindow(hwnd) }.clamp(96, 768);
            m.ptMinTrackSize = POINT {
                x: px(640.0, dpi),
                y: px(480.0, dpi),
            };
            return 0;
        }
        if (message == WM_COMMAND
            && !(((w >> 16) == BN_CLICKED as usize && matches!(w & 0xffff, 201..=209 | 212))
                || ((w >> 16) == EN_CHANGE as usize && matches!(w & 0xffff, 101..=104))
                || (matches!(w & 0xffff, 210 | 211)
                    && matches!(w >> 16, x if x==CBN_SELCHANGE as usize||x==CBN_SELENDCANCEL as usize||x==CBN_CLOSEUP as usize))))
            || (message == WM_TIMER && w != TIMER && w != ADMISSION_TIMER)
        {
            return unsafe { DefWindowProcW(hwnd, message, w, l) };
        }
        let ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const RefCell<State>;
        if ptr.is_null() {
            return unsafe { DefWindowProcW(hwnd, message, w, l) };
        }
        // SAFETY: userdata points to the stable Box<RefCell<State>> retained for this message loop.
        let cell = unsafe { &*ptr };
        if message == WM_COMMAND && (w >> 16) == BN_CLICKED as usize && (w & 0xffff) == 205 {
            choose_manifest(hwnd, cell);
            return 0;
        }
        if message == WM_COMMAND && (w >> 16) == BN_CLICKED as usize && (w & 0xffff) == 209 {
            choose_target_parent(hwnd, cell);
            return 0;
        }
        if message == WM_CLOSE || message == PICKER_CLOSE {
            if message == PICKER_CLOSE
                && unsafe { InSendMessageEx(std::ptr::null()) } & ISMEX_SEND != 0
            {
                unsafe { PostMessageW(hwnd, PICKER_CLOSE, 0, 0) };
                return 0;
            }
            let close_picker = {
                let Ok(mut state) = cell.try_borrow_mut() else {
                    unsafe {
                        PostMessageW(hwnd, message, 0, 0);
                    };
                    return 0;
                };
                if state.picker_active && (message == WM_CLOSE || state.closing) {
                    state.closing = true;
                    Some(state.picker_closer.clone())
                } else {
                    None
                }
            };
            if let Some(closer) = close_picker {
                if message == WM_CLOSE {
                    // Return from cross-thread input-synchronous dispatch before calling COM.
                    // Let the modal loop dispatch a separate application message.
                    let posted = unsafe { PostMessageW(hwnd, PICKER_CLOSE, 0, 0) };
                    if posted == 0 {
                        status(&cell.borrow(), text::PICKER_CLOSE_FAILED);
                    }
                    drop(closer);
                    return 0;
                }
                // PickerCloser clone is Rc-only (no COM AddRef); cancel and clone Drop occur after State borrow ends.
                if let Some(closer) = closer {
                    if closer.cancel().is_err() {
                        status(&cell.borrow(), text::PICKER_CLOSE_FAILED);
                    }
                    drop(closer);
                }
                return 0;
            }
            if message == PICKER_CLOSE {
                return 0;
            }
        }
        let Ok(mut borrowed) = cell.try_borrow_mut() else {
            if matches!(
                message,
                WM_CLOSE
                    | WM_SIZE
                    | FOCUS_CONTROL
                    | APPLY_DPI
                    | LAYOUT_VIEW
                    | APPEARANCE_CHANGED
                    | WM_VSCROLL
                    | WM_MOUSEWHEEL
            ) || (message == WM_COMMAND
                && (w >> 16) == EN_CHANGE as usize
                && matches!(w & 0xffff, 101..=104))
                || (message == WM_COMMAND
                    && matches!(w & 0xffff, 210 | 211)
                    && matches!(
                        (w >> 16) as u32,
                        CBN_SELCHANGE | CBN_SELENDCANCEL | CBN_CLOSEUP
                    ))
            {
                // These messages carry scalar values, not borrowed pointers. Defer rather than drop Close.
                unsafe {
                    PostMessageW(hwnd, message, w, l);
                }
            }
            return 0;
        };
        let state = &mut *borrowed;
        match message {
            WM_COMMAND if matches!(w & 0xffff, 210 | 211) => {
                // ComboBox restores cancelled popup values before this queued reconciliation.
                unsafe {
                    PostMessageW(hwnd, APPEARANCE_CHANGED, 0, 0);
                }
                0
            }
            APPEARANCE_CHANGED => {
                repaint_appearance(hwnd, state);
                0
            }
            APPLY_DPI => {
                if (96..=768).contains(&(w as u32)) {
                    if let Ok(mut view) = visual(state).try_borrow_mut() {
                        view.os_dpi = w as u32;
                    }
                    repaint_appearance(hwnd, state);
                }
                0
            }
            LAYOUT_VIEW => {
                layout(hwnd, state);
                0
            }
            FOCUS_CONTROL => {
                reveal_control(hwnd, state, w);
                0
            }
            WM_VSCROLL | WM_MOUSEWHEEL => {
                let mut viewport = RECT::default();
                unsafe {
                    GetClientRect(hwnd, &mut viewport);
                }
                if let Ok(mut view) = visual(state).try_borrow_mut() {
                    let delta = if message == WM_MOUSEWHEEL {
                        -i32::from((w >> 16) as u16 as i16) * px(40.0, view.dpi) / 120
                    } else {
                        match (w & 0xffff) as i32 {
                            SB_LINEUP => -px(32.0, view.dpi),
                            SB_LINEDOWN => px(32.0, view.dpi),
                            SB_PAGEUP => -viewport.bottom,
                            SB_PAGEDOWN => viewport.bottom,
                            SB_TOP => -view.scroll,
                            SB_BOTTOM => view.content_height,
                            SB_THUMBTRACK | SB_THUMBPOSITION => {
                                let mut info = SCROLLINFO {
                                    cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                                    fMask: SIF_TRACKPOS,
                                    ..Default::default()
                                };
                                unsafe {
                                    GetScrollInfo(hwnd, SB_VERT, &mut info);
                                }
                                info.nTrackPos - view.scroll
                            }
                            _ => 0,
                        }
                    };
                    view.scroll = (view.scroll + delta)
                        .clamp(0, (view.content_height - viewport.bottom).max(0));
                }
                layout(hwnd, state);
                0
            }
            WM_COMMAND if (w >> 16) == EN_CHANGE as usize && matches!(w & 0xffff, 101..=104) => {
                // Packaged journal is readonly and derived only from the selected target.
                // Its programmatic update must not invalidate the completed cold inspection.
                if state.packaged_source.is_some() && (w & 0xffff) == 104 {
                    return 0;
                }
                invalidate(state);
                queue_admission(hwnd, state, true);
                if state.packaged_source.is_some() && (w & 0xffff) == 103 {
                    let derived = read_edit(state.edits[2])
                        .ok()
                        .and_then(|target| packaged_journal(&target).ok());
                    let display = derived
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let value = wide(&display);
                    unsafe {
                        SetWindowTextW(state.edits[3], value.as_ptr());
                    }
                }
                if state.worker.is_none() && !launch_busy(state) && !state.closing {
                    status(
                        state,
                        if state.admission_pending.is_some() {
                            text::CHECKING_LOCATION
                        } else {
                            text::LOCATION_UNAVAILABLE
                        },
                    );
                    controls(state, false);
                }
                0
            }
            WM_COMMAND if (w >> 16) == BN_CLICKED as usize => {
                if state.picker_active {
                    return 0;
                }
                match w & 0xffff {
                    201 => start(
                        hwnd,
                        state,
                        if state.verified && state.packaged_source.is_some() {
                            Operation::Register
                        } else {
                            Operation::Prepare
                        },
                    ),
                    202 => stop(state, false),
                    203 => recheck(hwnd, state),
                    212 => start_launch(hwnd, state),
                    204 => confirm_remove(hwnd, state),
                    _ => {}
                }
                0
            }
            WM_TIMER if w == ADMISSION_TIMER => {
                poll_admission(hwnd, state);
                0
            }
            WM_TIMER if w == TIMER => {
                poll(hwnd, state);
                0
            }
            WM_CLOSE => {
                if let Some(launch) = &state.launch {
                    if launch.close_policy()
                        == crate::installer_launch::ClosePolicy::CloseWindowRetainSupervisor
                    {
                        state.closing = true;
                        unsafe {
                            DestroyWindow(hwnd);
                        }
                    } else {
                        stop(state, true);
                    }
                } else if state.worker.is_some() || state.admission_worker.is_some() {
                    stop(state, true);
                } else {
                    unsafe {
                        DestroyWindow(hwnd);
                    }
                }
                0
            }
            WM_SIZE => {
                layout(hwnd, state);
                let focus = unsafe { GetFocus() };
                if !focus.is_null() && unsafe { GetParent(focus) } == hwnd {
                    reveal_control(hwnd, state, unsafe { GetDlgCtrlID(focus) } as usize);
                }
                0
            }
            _ => unsafe { DefWindowProcW(hwnd, message, w, l) },
        }
    }
    fn child(
        font: HFONT,
        hwnd: HWND,
        class: &str,
        label: &str,
        id: usize,
        style: u32,
    ) -> Result<HWND, ()> {
        let c = wide(class);
        let t = wide(label);
        let handle = unsafe {
            CreateWindowExW(
                0,
                c.as_ptr(),
                t.as_ptr(),
                WS_CHILD | WS_VISIBLE | style | if class == "STATIC" { SS_NOPREFIX } else { 0 },
                0,
                0,
                10,
                10,
                hwnd,
                id as _,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null(),
            )
        };
        if handle.is_null() {
            return Err(());
        }
        unsafe {
            SendMessageW(handle, WM_SETFONT, font as usize, 1);
            if matches!(class, "EDIT" | "BUTTON" | "COMBOBOX")
                && SetWindowSubclass(handle, Some(control_proc), EDIT_SUBCLASS, 0) == 0
            {
                return Err(());
            }
        }
        Ok(handle)
    }
    pub fn run(initial: InitialPaths) -> Result<(), ()> {
        let common_controls = windows_sys::Win32::UI::Controls::INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<windows_sys::Win32::UI::Controls::INITCOMMONCONTROLSEX>()
                as u32,
            dwICC: windows_sys::Win32::UI::Controls::ICC_STANDARD_CLASSES,
        };
        if unsafe { windows_sys::Win32::UI::Controls::InitCommonControlsEx(&common_controls) } == 0
        {
            return Err(());
        }
        // Changes awareness for this process only; OS/user display settings remain unchanged.
        unsafe {
            let _ = SetProcessDpiAwarenessContext(-4);
        }
        unsafe {
            let instance = GetModuleHandleW(std::ptr::null());
            let class = wide(CLASS);
            let wc = WNDCLASSW {
                lpfnWndProc: Some(proc),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
                hIcon: LoadIconW(instance, 101usize as _),
                hbrBackground: (COLOR_WINDOW + 1) as _,
                ..Default::default()
            };
            if RegisterClassW(&wc) == 0 {
                return Err(());
            }
            let title = wide(if initial.2 {
                text::PACKAGED_TITLE
            } else {
                text::TITLE
            });
            let hwnd = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VSCROLL | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                if initial.2 { 800 } else { 900 },
                if initial.2 { 620 } else { 820 },
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null(),
            );
            if hwnd.is_null() {
                return Err(());
            }
            let dpi = GetDpiForWindow(hwnd).clamp(96, 768);
            let presentation = Box::new(RefCell::new(Presentation::new(dpi)));
            let font = presentation.borrow().fonts.body.handle;
            let setup = (|| {
                let mut labels = [std::ptr::null_mut(); 4];
                let mut edits = [std::ptr::null_mut(); 4];
                let mut picker_button = std::ptr::null_mut();
                let intro = child(
                    font,
                    hwnd,
                    "STATIC",
                    if initial.2 {
                        text::PACKAGED_INTRO
                    } else {
                        text::INTRO
                    },
                    300,
                    0,
                )?;
                for i in 0..4 {
                    labels[i] = child(
                        font,
                        hwnd,
                        "STATIC",
                        if initial.2 && i == 3 {
                            text::PACKAGED_JOURNAL_LABEL
                        } else {
                            text::LABELS[i]
                        },
                        110 + i,
                        0,
                    )?;
                    edits[i] = child(
                        font,
                        hwnd,
                        "EDIT",
                        &initial.0[i].to_string_lossy(),
                        101 + i,
                        WS_TABSTOP
                            | if initial.2 && i == 3 {
                                (ES_MULTILINE | ES_AUTOVSCROLL) as u32
                            } else {
                                ES_AUTOHSCROLL as u32
                            },
                    )?;
                    SendMessageW(edits[i], EM_SETLIMITTEXT, 32767, 0);
                    if initial.2 && (i < 2 || i == 3) {
                        SendMessageW(
                            edits[i],
                            windows_sys::Win32::UI::Controls::EM_SETREADONLY,
                            1,
                            0,
                        );
                    }
                    if i == 0 {
                        picker_button = child(
                            font,
                            hwnd,
                            "BUTTON",
                            text::PICK_MANIFEST,
                            205,
                            WS_TABSTOP | BS_PUSHBUTTON as u32,
                        )?;
                    }
                }
                let target_picker_button = child(
                    font,
                    hwnd,
                    "BUTTON",
                    text::TARGET_PICKER,
                    209,
                    WS_TABSTOP | BS_PUSHBUTTON as u32,
                )?;
                if initial.2 {
                    for i in 0..2 {
                        ShowWindow(labels[i], SW_HIDE);
                        ShowWindow(edits[i], SW_HIDE);
                    }
                    ShowWindow(picker_button, SW_HIDE);
                } else {
                    ShowWindow(target_picker_button, SW_HIDE);
                }
                let buttons = [
                    child(
                        font,
                        hwnd,
                        "BUTTON",
                        text::INSTALL,
                        201,
                        WS_TABSTOP | BS_PUSHBUTTON as u32,
                    )?,
                    child(
                        font,
                        hwnd,
                        "BUTTON",
                        text::CANCEL,
                        202,
                        WS_TABSTOP | BS_PUSHBUTTON as u32,
                    )?,
                    child(
                        font,
                        hwnd,
                        "BUTTON",
                        text::VERIFY,
                        203,
                        WS_TABSTOP | BS_PUSHBUTTON as u32,
                    )?,
                    child(
                        font,
                        hwnd,
                        "BUTTON",
                        text::REMOVE,
                        204,
                        WS_TABSTOP | BS_PUSHBUTTON as u32,
                    )?,
                ];
                let launch_button = child(
                    font,
                    hwnd,
                    "BUTTON",
                    text::RUN_APP,
                    212,
                    WS_TABSTOP | BS_PUSHBUTTON as u32,
                )?;
                let tool_checks = [
                    child(
                        font,
                        hwnd,
                        "BUTTON",
                        text::OPTIONAL_TOOLS[0],
                        206,
                        WS_TABSTOP | BS_AUTOCHECKBOX as u32,
                    )?,
                    child(
                        font,
                        hwnd,
                        "BUTTON",
                        text::OPTIONAL_TOOLS[1],
                        207,
                        WS_TABSTOP | BS_AUTOCHECKBOX as u32,
                    )?,
                    child(
                        font,
                        hwnd,
                        "BUTTON",
                        text::OPTIONAL_TOOLS[2],
                        208,
                        WS_TABSTOP | BS_AUTOCHECKBOX as u32,
                    )?,
                ];
                let tool_data = initial
                    .1
                    .clone()
                    .or_else(|| dlpad_engine::location::default_data_dir().ok());
                let description = match &tool_data {
                    Some(data) => format!("{}\r\n{}", text::TOOLS_HINT, data.to_string_lossy()),
                    None => text::TOOLS_DATA_UNAVAILABLE.to_owned(),
                };
                let heading = child(
                    presentation.borrow().fonts.title.handle,
                    hwnd,
                    "STATIC",
                    text::PREPARATION_HEADING,
                    304,
                    0,
                )?;
                let tools_heading = child(
                    presentation.borrow().fonts.heading.handle,
                    hwnd,
                    "STATIC",
                    text::OPTIONAL_TOOLS_HEADING,
                    305,
                    0,
                )?;
                let theme_label =
                    child(font, hwnd, "STATIC", text::THEME_LABEL, 306, SS_CENTERIMAGE)?;
                let theme = child(
                    font,
                    hwnd,
                    "COMBOBOX",
                    "",
                    210,
                    WS_TABSTOP
                        | CBS_DROPDOWNLIST as u32
                        | CBS_OWNERDRAWFIXED as u32
                        | CBS_HASSTRINGS as u32
                        | WS_VSCROLL,
                )?;
                let scale_label =
                    child(font, hwnd, "STATIC", text::SCALE_LABEL, 307, SS_CENTERIMAGE)?;
                let scale = child(
                    font,
                    hwnd,
                    "COMBOBOX",
                    "",
                    211,
                    WS_TABSTOP
                        | CBS_DROPDOWNLIST as u32
                        | CBS_OWNERDRAWFIXED as u32
                        | CBS_HASSTRINGS as u32
                        | WS_VSCROLL,
                )?;
                for (combo, values) in [(theme, text::THEME_CHOICES), (scale, text::SCALE_CHOICES)]
                {
                    for value in values {
                        let value = wide(value);
                        if SendMessageW(combo, CB_ADDSTRING, 0, value.as_ptr() as isize) < 0 {
                            return Err(());
                        }
                    }
                    SendMessageW(combo, CB_SETCURSEL, 0, 0);
                }
                let state = State {
                    presentation: &*presentation,
                    heading,
                    tools_heading,
                    theme_label,
                    theme,
                    scale,
                    scale_label,
                    appearance_help: child(font, hwnd, "STATIC", text::APPEARANCE_HELP, 308, 0)?,
                    packaged_source: initial
                        .2
                        .then(|| [initial.0[0].clone(), initial.0[1].clone()]),
                    resolved_packaged_journal: None,
                    tool_data,
                    tool_checks,
                    tools_hint: child(font, hwnd, "STATIC", &description, 302, 0)?,
                    tools_result: child(font, hwnd, "STATIC", "", 303, 0)?,
                    intro,
                    labels,
                    edits,
                    buttons,
                    launch_button,
                    launch: None,
                    launch_failure: None,
                    status: child(
                        font,
                        hwnd,
                        "STATIC",
                        if initial.2 {
                            text::PACKAGED_IDLE
                        } else {
                            text::IDLE
                        },
                        301,
                        0,
                    )?,
                    picker_button,
                    target_picker_button,
                    picker_active: false,
                    picker_closer: None,
                    worker: None,
                    admission_worker: None,
                    admission_pending: None,
                    admission_revision: 0,
                    admission_phase: admission::Phase::Unknown,
                    closing: false,
                    verified: false,
                    remove_eligible: false,
                    shell_registered: false,
                    confirmation: None,
                };
                ShowWindow(state.tools_result, SW_HIDE);
                Ok::<_, ()>(Box::new(RefCell::new(state)))
            })();
            let mut state = match setup {
                Ok(s) => s,
                Err(()) => {
                    DestroyWindow(hwnd);
                    return Err(());
                }
            };
            SetWindowLongPtrW(
                hwnd,
                GWLP_USERDATA,
                (&*state as *const RefCell<State>) as isize,
            );
            if SetPropW(
                hwnd,
                PRESENTATION_KEY.as_ptr(),
                (&*presentation as *const RefCell<Presentation>) as HANDLE,
            ) == 0
            {
                DestroyWindow(hwnd);
                return Err(());
            }
            {
                let initial = state.borrow();
                apply_fonts(&initial);
                repaint_appearance(hwnd, &initial);
                controls(&initial, false);
            }
            {
                let mut initial_state = state.borrow_mut();
                queue_admission(hwnd, &mut initial_state, true);
                // Initial defaults are complete input; only interactive edits need debounce.
                if let Some((_, after, _)) = initial_state.admission_pending.as_mut() {
                    *after = Instant::now();
                }
                poll_admission(hwnd, &mut initial_state);
            }
            if dpi != 96 {
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
                    0,
                    0,
                    px(if initial.2 { 800.0 } else { 900.0 }, dpi),
                    px(if initial.2 { 620.0 } else { 820.0 }, dpi),
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);
            let mut msg = MSG::default();
            loop {
                let got = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if got <= 0 {
                    // No dispatch occurs after the loop has ended. Remove userdata before exclusive access.
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    let state = state.get_mut();
                    if let Some(worker) = state.admission_worker.take() {
                        worker.cancel.store(true, Ordering::Release);
                        let _ = worker.handle.join();
                    }
                    if let Some(worker) = &state.worker {
                        worker.cancel.store(true, Ordering::Release);
                        while !state.worker.as_ref().unwrap().handle.is_finished() {
                            thread::sleep(Duration::from_millis(75));
                        }
                        if let Some(w) = state.worker.take() {
                            let _ = w.handle.join();
                        }
                    }
                    if IsWindow(hwnd) != 0 {
                        DestroyWindow(hwnd);
                    }
                    let launch_ok = if let Some(launch) = state.launch.take() {
                        let outcome = launch.finish(got == 0);
                        crate::installer_launch::retain_completion(
                            &mut state.launch_failure,
                            outcome,
                        )
                    } else {
                        crate::installer_launch::final_receipt_success(&state.launch_failure)
                    };
                    return if got == 0 && launch_ok {
                        Ok(())
                    } else {
                        Err(())
                    };
                }
                if IsDialogMessageW(hwnd, &msg) == 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
    #[cfg(test)]
    mod command_paint_memory_tests {
        use super::*;
        #[link(name = "user32")]
        unsafe extern "system" {
            #[link_name = "GetThreadDesktop"]
            fn test_thread_desktop(thread: u32) -> HANDLE;
            #[link_name = "GetUserObjectInformationW"]
            fn test_desktop_info(
                handle: HANDLE,
                index: i32,
                value: *mut std::ffi::c_void,
                bytes: u32,
                needed: *mut u32,
            ) -> i32;
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            #[link_name = "GetCurrentThreadId"]
            fn test_thread_id() -> u32;
            #[link_name = "GetCurrentProcessId"]
            fn test_process_id() -> u32;
        }
        fn require_private_host() {
            let mut name = [0u16; 256];
            let mut needed = 0;
            unsafe {
                let desktop = test_thread_desktop(test_thread_id());
                assert!(!desktop.is_null());
                assert_ne!(
                    test_desktop_info(desktop, 2, name.as_mut_ptr().cast(), 512, &mut needed),
                    0
                );
            }
            assert!((2..=512).contains(&needed));
            let length = name.iter().position(|v| *v == 0).unwrap();
            let name = String::from_utf16(&name[..length]).unwrap();
            assert!(
                name.starts_with("dlpad-cli-")
                    && name.len() == 42
                    && name[10..].bytes().all(|v| v.is_ascii_hexdigit()),
                "actual GDI regression requires owned windows-cli-host private desktop; no Default fallback"
            );
        }
        struct OwnedWindow {
            parent: HWND,
            button: HWND,
        }
        impl OwnedWindow {
            fn new(id: usize, width: i32, height: i32) -> Self {
                require_private_host();
                let mut window = Self {
                    parent: std::ptr::null_mut(),
                    button: std::ptr::null_mut(),
                };
                let parent_class = wide("STATIC");
                let button_class = wide("BUTTON");
                let empty = wide("");
                unsafe {
                    let instance = GetModuleHandleW(std::ptr::null());
                    window.parent = CreateWindowExW(
                        0,
                        parent_class.as_ptr(),
                        empty.as_ptr(),
                        WS_POPUP,
                        0,
                        0,
                        width,
                        height,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        instance,
                        std::ptr::null(),
                    );
                    assert!(!window.parent.is_null());
                    window.button = CreateWindowExW(
                        0,
                        button_class.as_ptr(),
                        empty.as_ptr(),
                        WS_CHILD | WS_TABSTOP | BS_PUSHBUTTON as u32,
                        0,
                        0,
                        width,
                        height,
                        window.parent,
                        id as HMENU,
                        instance,
                        std::ptr::null(),
                    );
                    assert!(!window.button.is_null());
                    assert_eq!(IsWindowVisible(window.parent), 0);
                    let mut pid = 0;
                    assert_eq!(
                        GetWindowThreadProcessId(window.button, &mut pid),
                        test_thread_id()
                    );
                    assert_eq!(pid, test_process_id());
                    assert_eq!(GetParent(window.button), window.parent);
                    assert_eq!(GetDlgCtrlID(window.button), id as i32);
                }
                window
            }
        }
        impl Drop for OwnedWindow {
            fn drop(&mut self) {
                unsafe {
                    if !self.button.is_null() {
                        DestroyWindow(self.button);
                    }
                    if !self.parent.is_null() {
                        DestroyWindow(self.parent);
                    }
                }
            }
        }
        #[test]
        #[ignore = "requires owned windows-cli-host private desktop; run explicitly with --ignored"]
        fn cancelled_initial_check_retries_discovery_before_verifying_a_vacant_target() {
            let window = OwnedWindow::new(202, 160, 32);
            let root = std::env::temp_dir().join(format!(
                "dlpad-cancel-retry-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&root).unwrap();
            let target = root.join("app");
            let journal = root.join("journal");
            std::fs::create_dir(&journal).unwrap();
            let sentinel = journal.join("preserve");
            std::fs::write(&sentinel, b"previous generation must survive retry").unwrap();
            let view = RefCell::new(Presentation::new(96));
            let make = |class: &str, label: &str, id| {
                child(std::ptr::null_mut(), window.parent, class, label, id, 0).unwrap()
            };
            let status_control = make("STATIC", "", 301);
            let edits = std::array::from_fn(|index| {
                let value = match index {
                    2 => &target,
                    3 => &journal,
                    _ => &root,
                };
                make("EDIT", &value.to_string_lossy(), 101 + index)
            });
            let buttons = std::array::from_fn(|index| make("BUTTON", "", 201 + index));
            let mut state = State {
                presentation: &view,
                heading: status_control,
                tools_heading: status_control,
                theme_label: status_control,
                theme: status_control,
                scale: status_control,
                scale_label: status_control,
                appearance_help: status_control,
                edits,
                labels: [status_control; 4],
                intro: status_control,
                status: status_control,
                buttons,
                launch_button: make("BUTTON", "", 212),
                launch: None,
                launch_failure: None,
                picker_button: make("BUTTON", "", 205),
                target_picker_button: make("BUTTON", "", 209),
                tool_checks: [status_control; 3],
                tools_hint: status_control,
                tools_result: status_control,
                tool_data: None,
                packaged_source: Some([root.join("manifest"), root.clone()]),
                resolved_packaged_journal: None,
                picker_active: false,
                picker_closer: None,
                worker: None,
                admission_worker: None,
                admission_pending: None,
                admission_revision: 0,
                admission_phase: admission::Phase::Unknown,
                closing: false,
                verified: false,
                remove_eligible: false,
                shell_registered: false,
                confirmation: None,
            };
            // Registration completion must restore real HWND actions after start invalidates them.
            registration_completed(&mut state, Ok(()), true);
            controls(&state, false);
            assert_ne!(unsafe { IsWindowEnabled(state.launch_button) }, 0);
            assert_eq!(unsafe { IsWindowEnabled(state.buttons[0]) }, 0);
            confirm_remove(window.parent, &mut state);
            assert!(state.worker.is_none() && state.confirmation.is_none());
            registration_completed(&mut state, Err(io::ErrorKind::PermissionDenied), true);
            controls(&state, false);
            assert_ne!(unsafe { IsWindowEnabled(state.buttons[0]) }, 0);
            registration_completed(&mut state, Err(io::ErrorKind::InvalidData), false);
            controls(&state, false);
            assert_eq!(unsafe { IsWindowEnabled(state.launch_button) }, 0);
            invalidate(&mut state);
            state.admission_phase = admission::Phase::Unknown;
            // Use the real packaged base, with an existing historical generation.
            let base = packaged_journal(&target).unwrap();
            std::fs::create_dir(&base).unwrap();
            let keep = base.join("preserve");
            std::fs::write(&keep, b"immutable old journal").unwrap();
            queue_admission(window.parent, &mut state, true);
            state.admission_pending.as_mut().unwrap().1 = Instant::now();
            poll_admission(window.parent, &mut state);
            assert!(state.admission_worker.is_some());
            stop(&mut state, false);
            let cancelled_revision = state.admission_revision;
            recheck(window.parent, &mut state);
            assert_eq!(
                state.admission_revision, cancelled_revision,
                "a retry cannot overlap the cancelled worker before its completion ACK"
            );
            let drain = |state: &mut State| {
                let until = Instant::now() + Duration::from_secs(15);
                while (admission_busy(state) || state.worker.is_some()) && Instant::now() < until {
                    poll_admission(window.parent, state);
                    poll(window.parent, state);
                    thread::sleep(Duration::from_millis(5));
                }
                assert!(!admission_busy(state) && state.worker.is_none());
            };
            drain(&mut state);
            recheck(window.parent, &mut state);
            drain(&mut state);
            assert_eq!(std::fs::read(&keep).unwrap(), b"immutable old journal");
            assert_eq!(
                std::fs::read(&sentinel).unwrap(),
                b"previous generation must survive retry"
            );
            assert!(!target.exists());
            assert!(!state.verified);
            assert_eq!(
                state.admission_phase,
                admission::Phase::Vacant,
                "retry must rediscover the available target, not inspect a nonexistent bundle"
            );
            assert_ne!(unsafe { IsWindowEnabled(state.buttons[0]) }, 0);
            recheck(window.parent, &mut state);
            drain(&mut state);
            assert_eq!(state.admission_phase, admission::Phase::Vacant);
            assert!(!state.verified && !target.exists());
            assert_eq!(std::fs::read(&keep).unwrap(), b"immutable old journal");
            assert_eq!(
                state.resolved_packaged_journal.as_ref().unwrap().1,
                base.with_file_name(format!("{}-1", base.file_name().unwrap().to_string_lossy()))
            );
            unsafe {
                KillTimer(window.parent, ADMISSION_TIMER);
                KillTimer(window.parent, TIMER);
            }
            std::fs::remove_file(keep).unwrap();
            std::fs::remove_dir(base).unwrap();
            std::fs::remove_file(sentinel).unwrap();
            std::fs::remove_dir(journal).unwrap();
            std::fs::remove_dir(root).unwrap();
        }
        struct OwnedDib {
            dc: HDC,
            bitmap: HBITMAP,
            previous: HGDIOBJ,
            width: i32,
            height: i32,
        }
        impl OwnedDib {
            fn new(width: i32, height: i32) -> Self {
                assert!(width > 0 && height > 0 && width <= 512 && height <= 128);
                unsafe {
                    let mut surface = Self {
                        dc: CreateCompatibleDC(std::ptr::null_mut()),
                        bitmap: std::ptr::null_mut(),
                        previous: std::ptr::null_mut(),
                        width,
                        height,
                    };
                    assert!(!surface.dc.is_null());
                    let mut info: BITMAPINFO = std::mem::zeroed();
                    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
                    info.bmiHeader.biWidth = width;
                    info.bmiHeader.biHeight = -height;
                    info.bmiHeader.biPlanes = 1;
                    info.bmiHeader.biBitCount = 32;
                    info.bmiHeader.biCompression = BI_RGB;
                    let mut bits = std::ptr::null_mut();
                    surface.bitmap = CreateDIBSection(
                        surface.dc,
                        &info,
                        DIB_RGB_COLORS,
                        &mut bits,
                        std::ptr::null_mut(),
                        0,
                    );
                    assert!(!surface.bitmap.is_null() && !bits.is_null());
                    surface.previous = SelectObject(surface.dc, surface.bitmap);
                    assert!(!surface.previous.is_null() && surface.previous as isize != -1);
                    surface
                }
            }
            fn clear(&self, value: COLORREF) {
                let rect = RECT {
                    left: 0,
                    top: 0,
                    right: self.width,
                    bottom: self.height,
                };
                unsafe {
                    SetDCBrushColor(self.dc, value);
                    assert_ne!(
                        FillRect(self.dc, &rect, GetStockObject(DC_BRUSH) as HBRUSH),
                        0
                    );
                }
            }
            fn pixel(&self, x: i32, y: i32) -> COLORREF {
                assert!((0..self.width).contains(&x) && (0..self.height).contains(&y));
                let value = unsafe { GetPixel(self.dc, x, y) };
                assert_ne!(value, u32::MAX);
                value
            }
        }
        impl Drop for OwnedDib {
            fn drop(&mut self) {
                unsafe {
                    if !self.dc.is_null() {
                        if !self.previous.is_null() && self.previous as isize != -1 {
                            SelectObject(self.dc, self.previous);
                        }
                        // Destroy our DC even if restore failed, then release only our bitmap.
                        DeleteDC(self.dc);
                    }
                    if !self.bitmap.is_null() {
                        DeleteObject(self.bitmap);
                    }
                }
            }
        }
        fn luminance(value: COLORREF) -> f64 {
            let component = |shift: u32| {
                let v = f64::from(((value >> shift) & 255u32) as u8) / 255.0;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            };
            component(0) * 0.2126 + component(8) * 0.7152 + component(16) * 0.0722
        }
        fn contrast(a: COLORREF, b: COLORREF) -> f64 {
            let a = luminance(a);
            let b = luminance(b);
            (a.max(b) + 0.05) / (a.min(b) + 0.05)
        }
        fn draw(
            surface: &OwnedDib,
            window: &OwnedWindow,
            view: &Presentation,
            id: usize,
            flags: u32,
        ) -> LRESULT {
            let mut item: NMCUSTOMDRAW = unsafe { std::mem::zeroed() };
            item.hdr.hwndFrom = window.button;
            item.hdr.idFrom = id;
            item.hdr.code = NM_CUSTOMDRAW;
            item.dwDrawStage = CDDS_PREPAINT;
            item.hdc = surface.dc;
            assert_ne!(unsafe { GetClientRect(window.button, &mut item.rc) }, 0);
            assert_eq!(
                (item.rc.right, item.rc.bottom),
                (surface.width, surface.height)
            );
            item.uItemState = flags;
            paint_command(&item, view)
        }
        #[test]
        #[ignore = "requires owned windows-cli-host private desktop; run explicitly with --ignored"]
        fn actual_command_painter_focused_filled_roles_have_visible_inside_ring() {
            require_private_host();
            for dpi in [96, 192] {
                for appearance in [Appearance::Light, Appearance::Dark] {
                    let mut view = Presentation::new(dpi);
                    view.palette = Palette::new(appearance);
                    assert!(
                        !view.palette.contrast,
                        "normal-role fixture requires actual HC disabled; do not change OS settings"
                    );
                    view.brushes = Brushes::new(view.palette);
                    let width = px(160.0, dpi);
                    let height = px(t::CONTROL_HEIGHT, dpi);
                    for id in [201, 204, 212] {
                        let window = OwnedWindow::new(id, width, height);
                        let surface = OwnedDib::new(width, height);
                        for state in [0, CDIS_HOT, CDIS_SELECTED] {
                            surface.clear(color([1, 2, 3]));
                            assert_eq!(
                                draw(&surface, &window, &view, id, CDIS_FOCUS | state),
                                CDRF_SKIPDEFAULT as LRESULT
                            );
                            let focus = surface.pixel(width / 2, px(1.0, dpi));
                            let fill = surface.pixel(width / 2, height / 2);
                            assert!(
                                contrast(focus, fill) >= 3.0,
                                "actual painter inside ring/fill RGB contrast: role={id}, dpi={dpi}, focusedflags={state}, ring={focus:#x}, fill={fill:#x}"
                            );
                        }
                    }
                }
            }
        }
        #[test]
        #[ignore = "requires owned windows-cli-host private desktop; run explicitly with --ignored"]
        fn actual_neutral_command_focus_keeps_a_distinct_palette_ring() {
            require_private_host();
            for dpi in [96, 192] {
                for appearance in [Appearance::Light, Appearance::Dark] {
                    let mut view = Presentation::new(dpi);
                    view.palette = Palette::new(appearance);
                    assert!(!view.palette.contrast);
                    view.brushes = Brushes::new(view.palette);
                    let width = px(160.0, dpi);
                    let height = px(t::CONTROL_HEIGHT, dpi);
                    let window = OwnedWindow::new(203, width, height);
                    let surface = OwnedDib::new(width, height);
                    for state in [0, CDIS_HOT, CDIS_SELECTED] {
                        assert_eq!(
                            draw(&surface, &window, &view, 203, CDIS_FOCUS | state),
                            CDRF_SKIPDEFAULT as LRESULT
                        );
                        let ring = surface.pixel(width / 2, px(1.0, dpi));
                        let fill = surface.pixel(width / 2, height / 2);
                        assert!(
                            contrast(ring, fill) >= 3.0,
                            "actual neutral painter ring/fill must remain distinct"
                        );
                    }
                }
            }
        }
        #[test]
        #[ignore = "requires owned windows-cli-host private desktop; run explicitly with --ignored"]
        fn actual_native_enabled_gate_suppresses_disabled_focus_and_hc_uses_default() {
            require_private_host();
            let mut view = Presentation::new(96);
            view.palette = Palette::new(Appearance::Light);
            assert!(!view.palette.contrast);
            view.brushes = Brushes::new(view.palette);
            let window = OwnedWindow::new(212, 160, 32);
            let surface = OwnedDib::new(160, 32);
            unsafe {
                EnableWindow(window.button, 0);
            }
            assert_eq!(
                draw(&surface, &window, &view, 212, CDIS_FOCUS),
                CDRF_SKIPDEFAULT as LRESULT
            );
            assert_eq!(
                surface.pixel(80, 1),
                surface.pixel(80, 16),
                "actual disabled HWND must not draw focus ring despite synthetic focus flag"
            );
            unsafe {
                EnableWindow(window.button, 1);
            }
            surface.clear(color([1, 2, 3]));
            view.palette.contrast = true; // Local branch fixture only; OS HC is untouched.
            assert_eq!(
                draw(&surface, &window, &view, 212, CDIS_FOCUS),
                CDRF_DODEFAULT as LRESULT
            );
            assert_eq!(
                surface.pixel(80, 1),
                color([1, 2, 3]),
                "HC native-fallback branch must leave memory bitmap untouched"
            );
        }
    }
    #[cfg(test)]
    mod optional_tool_tests {
        use super::*;
        fn app() -> LocalJournalReport {
            LocalJournalReport {
                kind: None,
                mutation_may_have_occurred: true,
                bundle_capabilities_present: true,
                journal_capabilities_present: true,
                current_consistency_verified: true,
                entries: Some(2),
            }
        }
        #[test]
        fn no_tools_or_failed_app_never_enters_supply() {
            let mut report = app();
            assert!(optional_tools_after_app(&report, None, |_| panic!("none selected")).is_none());
            report.current_consistency_verified = false;
            assert!(
                optional_tools_after_app(&report, Some("owned".into()), |_| panic!(
                    "unverified app"
                ))
                .is_none()
            );
            report.current_consistency_verified = true;
            for kind in [
                io::ErrorKind::Interrupted,
                io::ErrorKind::TimedOut,
                io::ErrorKind::InvalidData,
            ] {
                report.kind = Some(kind);
                assert!(
                    optional_tools_after_app(&report, Some("owned".into()), |_| panic!(
                        "failed app"
                    ))
                    .is_none()
                );
            }
        }
        #[test]
        fn tool_failure_keeps_verified_app_report() {
            let report = app();
            let summary = optional_tools_after_app(&report, Some("owned".into()), |_| {
                "tool failed".to_owned()
            });
            assert_eq!(summary.as_deref(), Some("tool failed"));
            assert!(report.kind.is_none() && report.current_consistency_verified);
            assert!(report.bundle_capabilities_present && report.journal_capabilities_present);
        }
    }
    #[cfg(test)]
    mod removal_report_tests {
        use super::*;
        fn report() -> LocalRemovalReport {
            LocalRemovalReport {
                kind: None,
                mutation_may_have_occurred: false,
                accepted_file_dispositions: 0,
                accepted_root_dispositions: 0,
                outer_namespace_absence_confirmed: false,
            }
        }
        #[test]
        fn absence_required_even_after_all_dispositions() {
            let mut r = report();
            r.accepted_file_dispositions = 6;
            r.accepted_root_dispositions = 4;
            r.mutation_may_have_occurred = true;
            assert_eq!(removal_message(&r), text::REMOVAL_PARTIAL);
            r.outer_namespace_absence_confirmed = true;
            assert_eq!(removal_message(&r), text::REMOVED);
            r.kind = Some(io::ErrorKind::TimedOut);
            assert_eq!(removal_message(&r), text::REMOVAL_PARTIAL);
        }
        #[test]
        fn interruption_does_not_hide_partial_disposition() {
            let mut r = report();
            r.kind = Some(io::ErrorKind::Interrupted);
            assert_eq!(removal_message(&r), text::REMOVAL_REFUSED);
            r.accepted_file_dispositions = 1;
            assert_eq!(removal_message(&r), text::REMOVAL_PARTIAL);
            r.accepted_file_dispositions = 0;
            r.mutation_may_have_occurred = true;
            assert_eq!(removal_message(&r), text::REMOVAL_PARTIAL);
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).take(33).collect();
    #[cfg(windows)]
    if let Some(code) = installer_uninstall::maybe_run(&args) {
        std::process::exit(code);
    }
    let resolved = if args.is_empty() {
        std::env::current_exe()
            .map_err(|_| ())
            .and_then(|executable| initial_for_executable(&args, &executable))
    } else {
        parse(&args)
    };
    #[allow(unused_mut)]
    let mut initial = match resolved {
        Ok(v) => v,
        Err(()) => {
            std::process::exit(2);
        }
    };
    #[cfg(windows)]
    apply_recommended_target(&mut initial, installer_defaults::recommended_target);
    #[cfg(windows)]
    if native::run(initial).is_err() {
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        let _ = initial;
        std::process::exit(1);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn startup_hint_never_overwrites_explicit_or_partial_input() {
        for (packaged, target, journal) in [
            (false, "", ""),
            (true, "explicit", ""),
            (true, "", "explicit-journal"),
        ] {
            let mut initial = InitialPaths(
                [
                    PathBuf::from("manifest"),
                    PathBuf::from("payload"),
                    PathBuf::from(target),
                    PathBuf::from(journal),
                ],
                Some(PathBuf::from("tool-data")),
                packaged,
            );
            let before = initial.clone();
            apply_recommended_target(&mut initial, || {
                panic!("explicit input must not even query hints")
            });
            assert_eq!(initial, before);
        }
    }
    #[test]
    fn startup_hint_is_advisory_and_derives_journal_without_io() {
        let mut initial = InitialPaths(
            [
                PathBuf::from("manifest"),
                PathBuf::from("payload"),
                PathBuf::new(),
                PathBuf::new(),
            ],
            Some(PathBuf::from("tool-data")),
            true,
        );
        let before = initial.clone();
        apply_recommended_target(&mut initial, || None);
        assert_eq!(initial, before);
        apply_recommended_target(&mut initial, || Some(PathBuf::from("relative")));
        assert_eq!(initial, before);
        let root = if cfg!(windows) {
            PathBuf::from(r"D:\synthetic-owned")
        } else {
            PathBuf::from("/synthetic-owned")
        };
        let target = root.join("dlpad-hint");
        apply_recommended_target(&mut initial, || Some(target.clone()));
        assert_eq!(initial.0[2], target);
        assert_eq!(initial.0[3], root.join(".dlpad-hint-install-journal"));
        assert_eq!(&initial.0[..2], &before.0[..2]);
        assert_eq!(initial.1, before.1);
        assert!(initial.2);
    }

    #[test]
    fn packaged_journal_is_sibling_and_rejects_invalid_target_leaf() {
        let root = if cfg!(windows) {
            PathBuf::from(r"D:\owned")
        } else {
            PathBuf::from("/owned")
        };
        assert_eq!(
            packaged_journal(&root.join("dlpad-preview")).unwrap(),
            root.join(".dlpad-preview-install-journal")
        );
        for name in ["NUL", "con.txt", "COM1", "bad.", "bad ", "bad:name"] {
            assert!(packaged_journal(&root.join(name)).is_err(), "{name}");
        }
        assert!(packaged_journal(std::path::Path::new("relative")).is_err());
        assert!(
            packaged_journal(if cfg!(windows) {
                std::path::Path::new(r"D:\\")
            } else {
                std::path::Path::new("/")
            })
            .is_err()
        );
    }
    #[test]
    fn packaged_discovery_is_create_free_and_does_not_supply_targets() {
        let root = std::env::temp_dir().join(format!(
            "dlpad-package-input-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let executable = root.join("dlpad-install.exe");
        let absent = initial_for_executable(&[], &executable).unwrap();
        assert!(!absent.2);
        assert!(!root.join("payload").exists());
        std::fs::create_dir(root.join("payload")).unwrap();
        let manifest = root.join("payload/manifest.json");
        // Discovery does not accept content as trusted; worker validation remains required.
        std::fs::write(&manifest, b"malformed unsigned input").unwrap();
        let packaged = initial_for_executable(&[], &executable).unwrap();
        assert!(packaged.2);
        assert_eq!(packaged.0[0], manifest);
        assert!(
            packaged.0[2..]
                .iter()
                .all(|path| path.as_os_str().is_empty())
        );
        assert!(packaged.1.is_none());
        let explicit = [OsString::from("--tool-data"), root.clone().into_os_string()];
        assert!(!initial_for_executable(&explicit, &executable).unwrap().2);
        let invalid = [OsString::from("--manifest"), root.clone().into_os_string()];
        assert!(initial_for_executable(&invalid, &executable).is_err());
        std::fs::remove_file(&manifest).unwrap();
        std::fs::create_dir(&manifest).unwrap();
        assert!(initial_for_executable(&[], &executable).is_err());
        std::fs::remove_dir(&manifest).unwrap();
        #[cfg(unix)]
        {
            let other = root.join("foreign.json");
            std::fs::write(&other, b"foreign").unwrap();
            std::os::unix::fs::symlink(&other, &manifest).unwrap();
            assert!(initial_for_executable(&[], &executable).is_err());
            std::fs::remove_file(&manifest).unwrap();
            std::fs::remove_file(&other).unwrap();
        }
        std::fs::remove_dir(root.join("payload")).unwrap();
        std::fs::remove_dir(&root).unwrap();
    }
    #[test]
    fn explicit_tool_data_profile_is_bounded_and_never_selects_tools() {
        let p = if cfg!(windows) {
            r"D:\owned-profile"
        } else {
            "/owned-profile"
        };
        let args = [OsString::from("--tool-data"), OsString::from(p)];
        let parsed = parse(&args).unwrap();
        assert!(parsed.0.iter().all(|p| p.as_os_str().is_empty()));
        assert_eq!(parsed.1, Some(PathBuf::from(p)));
        let mut bad = args.clone();
        bad[1] = "relative".into();
        assert!(parse(&bad).is_err());
        let duplicate = [
            args[0].clone(),
            args[1].clone(),
            args[0].clone(),
            args[1].clone(),
        ];
        assert!(parse(&duplicate).is_err());
        let partial = [OsString::from("--manifest"), OsString::from(p)];
        assert!(parse(&partial).is_err());
        let full = [
            "--manifest",
            p,
            "--source-dir",
            p,
            "--bundle-root",
            p,
            "--journal-root",
            p,
            "--tool-data",
            p,
        ]
        .map(OsString::from);
        assert!(parse(&full).unwrap().1.is_some());
    }
    #[test]
    fn optional_prefill_is_strict_and_does_not_auto_install() {
        assert!(parse(&[]).is_ok());
        let p = if cfg!(windows) { r"D:\owned" } else { "/owned" };
        let good = [
            "--manifest",
            p,
            "--source-dir",
            p,
            "--bundle-root",
            p,
            "--journal-root",
            p,
        ]
        .map(OsString::from);
        assert!(parse(&good).is_ok());
        for index in [0, 2, 4, 6] {
            let mut v = good.clone();
            v[index] = "--unknown".into();
            assert!(parse(&v).is_err());
        }
        let mut v = good.clone();
        v[2] = "--manifest".into();
        assert!(parse(&v).is_err());
        let mut v = good.clone();
        v[1] = "relative".into();
        assert!(parse(&v).is_err());
        assert!(parse(&good[..6]).is_err());
    }
}
