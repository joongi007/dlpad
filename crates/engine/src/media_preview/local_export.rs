//! Windows-only local range export candidate. No Drop or pathname cleanup.
//! Caller must finish its managed execution scope only after confirmed process cleanup.
use super::{
    CaptureRole, Error, Prepared, PreviewSession, Source, Tools, args, capture, check, verify_local,
};
use std::os::windows::{
    ffi::OsStrExt,
    fs::{MetadataExt, OpenOptionsExt},
    io::{AsRawHandle, FromRawHandle},
};
use std::{
    fs::{File, OpenOptions},
    io,
    path::{Component, Path, PathBuf, Prefix},
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};
use windows_sys::{
    Wdk::{
        Foundation::OBJECT_ATTRIBUTES,
        Storage::FileSystem::{
            FILE_CREATE, FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE,
            FILE_SYNCHRONOUS_IO_NONALERT, NtCreateFile,
        },
    },
    Win32::{
        Foundation::{GENERIC_READ, OBJ_CASE_INSENSITIVE, RtlNtStatusToDosError, UNICODE_STRING},
        Storage::FileSystem::*,
        System::IO::IO_STATUS_BLOCK,
    },
};
const MAX_RANGE_MS: u64 = 10 * 60 * 1000;
const MAX_OUTPUT_BYTES: u64 = 256 * 1024 * 1024;
pub struct ExportRequest {
    pub prepared: Arc<Prepared>,
    pub start_ms: u64,
    pub end_ms: u64,
    pub output_parent: PathBuf,
    pub folder_name: String,
}
pub struct Exported {
    pub path: PathBuf,
    pub bytes: u64,
    pub duration_ms: u64,
    /// Original stage hardlink remains intentionally. It shares file bytes with path.
    pub retained_stage_path: PathBuf,
}
/// Path facts only, never deletion capabilities. Any failure preserves fresh output.
pub struct ExportFailure {
    pub error: Error,
    pub fresh_directory: Option<PathBuf>,
    pub stage_preserved: bool,
    pub final_created: bool,
}
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "local export capability rejected",
    )
}
fn leaf(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    !name.is_empty()
        && name.len() <= 96
        && name.is_ascii()
        && !name.ends_with(['.', ' '])
        && name != "."
        && name != ".."
        && !name
            .bytes()
            .any(|b| b.is_ascii_control() || b"/\\:<>\"|?*".contains(&b))
        && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
        && !(stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}
fn path_valid(path: &Path) -> bool {
    path.is_absolute()
        && path.components().count() <= 256
        && path.as_os_str().encode_wide().count() <= 32766
        && !path
            .components()
            .any(|p| matches!(p, Component::ParentDir | Component::CurDir))
        && matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_)|Prefix::VerbatimDisk(_)))
}
fn open_directory(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES | READ_CONTROL)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(invalid());
    }
    Ok(file)
}
fn create_relative(parent: &File, name: &str, directory: bool) -> io::Result<File> {
    if !leaf(name) {
        return Err(invalid());
    }
    let sid = crate::windows_private_security::current_sid()?;
    let descriptor = crate::windows_private_security::PrivateDescriptor::new(&sid)?;
    let mut units: Vec<_> = name.encode_utf16().collect();
    let length = u16::try_from(units.len() * 2).map_err(|_| invalid())?;
    let mut unicode = UNICODE_STRING {
        Length: length,
        MaximumLength: length,
        Buffer: units.as_mut_ptr(),
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: parent.as_raw_handle(),
        ObjectName: &mut unicode,
        Attributes: OBJ_CASE_INSENSITIVE,
        SecurityDescriptor: descriptor.attributes().lpSecurityDescriptor.cast(),
        ..Default::default()
    };
    let mut status = IO_STATUS_BLOCK::default();
    let mut handle = std::ptr::null_mut();
    let access = if directory {
        FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | READ_CONTROL | SYNCHRONIZE
    } else {
        GENERIC_READ | SYNCHRONIZE | READ_CONTROL
    };
    let options = if directory {
        FILE_DIRECTORY_FILE
    } else {
        FILE_NON_DIRECTORY_FILE
    } | FILE_SYNCHRONOUS_IO_NONALERT;
    // SAFETY: original live parent, one bounded component, owned descriptor, correctly sized outputs.
    let result = unsafe {
        NtCreateFile(
            &mut handle,
            access,
            &attributes,
            &mut status,
            std::ptr::null(),
            if directory {
                FILE_ATTRIBUTE_DIRECTORY
            } else {
                FILE_ATTRIBUTE_NORMAL
            },
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            FILE_CREATE,
            options,
            std::ptr::null(),
            0,
        )
    };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    if handle.is_null() || handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return Err(invalid());
    }
    // SAFETY: exactly one new successful noninheritable HANDLE transferred to File.
    let file = unsafe { File::from_raw_handle(handle) };
    Ok(file)
}
fn verify_path(path: &Path, file: &File, directory: bool) -> io::Result<()> {
    let current = if directory {
        open_directory(path)?
    } else {
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?
    };
    if crate::file_identity::from_file(file)? != crate::file_identity::from_file(&current)?
        || crate::file_identity::final_path(file)? != crate::file_identity::final_path(&current)?
    {
        return Err(invalid());
    }
    let metadata = current.metadata()?;
    if metadata.is_dir() != directory
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(invalid());
    }
    Ok(())
}
fn stamp(ms: u64) -> String {
    format!("{}.{:03}", ms / 1000, ms % 1000)
}
fn export_arguments(
    source: &Path,
    output: &Path,
    start_ms: u64,
    duration_ms: u64,
) -> Vec<std::ffi::OsString> {
    // Input seeking avoids decoding hours of preceding media; accurate seek discards
    // the short segment from the preceding seek point during transcoding.
    let mut argv = args(&[
        "-nostdin",
        "-hide_banner",
        "-loglevel",
        "error",
        "-threads",
        "1",
        "-filter_threads",
        "1",
        "-max_alloc",
        "33554432",
        "-protocol_whitelist",
        "file",
        "-ss",
        &stamp(start_ms),
        "-accurate_seek",
        "-i",
    ]);
    argv.push(source.as_os_str().into());
    argv.extend(args(&[
        "-t",
        &stamp(duration_ms),
        "-map",
        "0:v:0",
        "-map",
        "0:a:0?",
        "-sn",
        "-dn",
        // Preserve the requested common timeline instead of rounding a low-FPS
        // video to whole frame periods or resetting only its stream epoch.
        "-enc_time_base:v",
        "1:1000",
        "-fps_mode:v",
        "passthrough",
        "-c:v",
        "mpeg4",
        "-threads",
        "1",
        "-q:v",
        "3",
        "-c:a",
        "aac",
        "-movflags",
        "+faststart",
        "-fs",
        "268435456",
        "-f",
        "mp4",
        "-y",
    ]));
    argv.push(output.as_os_str().into());
    argv
}
/// FFmpeg writes only the original fresh stage, whose retained read handle denies
/// foreign DELETE while permitting a writer. After terminal, the validation read
/// pin denies both WRITE and DELETE. Neither stage nor root is deleted automatically.
pub fn export_local_range(
    tools: &Tools,
    request: ExportRequest,
    session: &PreviewSession<'_>,
    cancel: &Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Exported, ExportFailure> {
    let mut directory = None;
    let mut stage_preserved = false;
    let mut final_created = false;
    let result = (|| {
        check(cancel, deadline)?;
        if request.start_ms >= request.end_ms
            || request.end_ms > request.prepared.duration_ms
            || request.end_ms - request.start_ms > MAX_RANGE_MS
            || !leaf(&request.folder_name)
            || !path_valid(&request.output_parent)
        {
            return Err(Error::InvalidInput);
        }
        let source = match &request.prepared.input {
            Source::Local(path) => path,
            _ => return Err(Error::UnsupportedMedia),
        };
        let source_pin = request
            .prepared
            .local_pin
            .as_ref()
            .ok_or(Error::InvalidInput)?;
        verify_local(source, source_pin)?;
        let mut ancestors = vec![];
        let ancestor_paths: Vec<_> = request.output_parent.ancestors().collect();
        for path in ancestor_paths.into_iter().rev() {
            check(cancel, deadline)?;
            ancestors.push((
                path.to_owned(),
                open_directory(path).map_err(|_| Error::InvalidInput)?,
            ));
        }
        let parent = &ancestors.last().ok_or(Error::InvalidInput)?.1;
        check(cancel, deadline)?;
        let root =
            create_relative(parent, &request.folder_name, true).map_err(|_| Error::InvalidInput)?;
        directory = Some(request.output_parent.join(&request.folder_name));
        let sid =
            crate::windows_private_security::current_sid().map_err(|_| Error::InvalidInput)?;
        crate::windows_private_security::verify_acl(&root, &sid)
            .map_err(|_| Error::InvalidInput)?;
        let root_path = crate::file_identity::final_path(&root).map_err(|_| Error::InvalidInput)?;
        directory = Some(root_path.clone());
        for (path, cap) in &ancestors {
            verify_path(path, cap, true).map_err(|_| Error::InvalidInput)?;
        }
        verify_path(&root_path, &root, true).map_err(|_| Error::InvalidInput)?;
        check(cancel, deadline)?;
        let stage = create_relative(&root, "stage.mp4", false).map_err(|_| Error::InvalidInput)?;
        stage_preserved = true;
        crate::windows_private_security::verify_acl(&stage, &sid)
            .map_err(|_| Error::InvalidInput)?;
        let stage_path = root_path.join("stage.mp4");
        let identity = crate::file_identity::from_file(&stage).map_err(|_| Error::InvalidInput)?;
        let duration = request.end_ms - request.start_ms;
        let argv = export_arguments(source, &stage_path, request.start_ms, duration);
        let stdout = capture(
            &tools.ffmpeg,
            argv,
            cancel,
            deadline,
            Duration::from_secs(120),
            session,
            CaptureRole::DirectProcessing,
        )?;
        if !stdout.is_empty() {
            return Err(Error::InvalidMetadata);
        }
        verify_local(source, source_pin)?;
        for (path, cap) in &ancestors {
            verify_path(path, cap, true).map_err(|_| Error::InvalidInput)?;
        }
        verify_path(&stage_path, &stage, false).map_err(|_| Error::InvalidInput)?;
        // Deny new writers and DELETE after the terminal process; retain original identity.
        let read_pin = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&stage_path)
            .map_err(|_| Error::InvalidInput)?;
        if crate::file_identity::from_file(&read_pin).map_err(|_| Error::InvalidInput)? != identity
        {
            return Err(Error::InvalidInput);
        }
        let bytes = read_pin.metadata().map_err(|_| Error::InvalidInput)?.len();
        if bytes == 0 || bytes > MAX_OUTPUT_BYTES {
            return Err(Error::OutputLimit);
        }
        let mut probe = args(&[
            "-v",
            "error",
            "-protocol_whitelist",
            "file",
            "-show_entries",
            "format=duration:stream=codec_type",
            "-of",
            "json",
            "-i",
        ]);
        probe.push(stage_path.as_os_str().into());
        let raw = capture(
            &tools.ffprobe,
            probe,
            cancel,
            deadline,
            Duration::from_secs(10),
            session,
            CaptureRole::ProbeMetadata,
        )?;
        let metadata: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|_| Error::InvalidMetadata)?;
        let actual = super::milliseconds(&metadata["format"]["duration"])?;
        if actual.abs_diff(duration) > 250 {
            return Err(Error::InvalidMetadata);
        }
        let mut decode = args(&[
            "-nostdin",
            "-v",
            "error",
            "-threads",
            "1",
            "-max_alloc",
            "33554432",
            "-protocol_whitelist",
            "file",
            "-i",
        ]);
        decode.push(stage_path.as_os_str().into());
        decode.extend(args(&[
            "-map",
            "0:v:0",
            "-an",
            "-frames:v",
            "1",
            "-vf",
            "scale=320:180",
            "-pix_fmt",
            "rgb24",
            "-f",
            "rawvideo",
            "pipe:1",
        ]));
        let frame = capture(
            &tools.ffmpeg,
            decode,
            cancel,
            deadline,
            Duration::from_secs(10),
            session,
            CaptureRole::DirectProcessing,
        )?;
        if frame.len() != super::FRAME_BYTES {
            return Err(Error::InvalidMetadata);
        }
        verify_local(source, source_pin)?;
        verify_path(&stage_path, &stage, false).map_err(|_| Error::InvalidInput)?;
        if read_pin.metadata().map_err(|_| Error::InvalidInput)?.len() != bytes {
            return Err(Error::InvalidInput);
        }
        check(cancel, deadline)?;
        let final_path = root_path.join("clip.mp4");
        std::fs::hard_link(&stage_path, &final_path).map_err(|_| Error::InvalidInput)?; // create-only atomic final name.
        final_created = true;
        let final_pin = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&final_path)
            .map_err(|_| Error::OwnershipUnconfirmed)?;
        if crate::file_identity::from_file(&final_pin).map_err(|_| Error::OwnershipUnconfirmed)?
            != identity
            || final_pin
                .metadata()
                .map_err(|_| Error::OwnershipUnconfirmed)?
                .len()
                != bytes
        {
            return Err(Error::OwnershipUnconfirmed);
        }
        crate::file_identity::ensure_regular_under(&root, &final_pin)
            .map_err(|_| Error::OwnershipUnconfirmed)?;
        // Deliberately preserve stage: no pathname cleanup, DELETE cap acquisition or disposition.
        for (path, cap) in &ancestors {
            verify_path(path, cap, true).map_err(|_| Error::OwnershipUnconfirmed)?;
        }
        verify_path(&root_path, &root, true).map_err(|_| Error::OwnershipUnconfirmed)?;
        check(cancel, deadline)?;
        Ok(Exported {
            path: final_path,
            bytes,
            duration_ms: actual,
            retained_stage_path: stage_path,
        })
    })();
    result.map_err(|error| ExportFailure {
        error,
        fresh_directory: directory,
        stage_preserved,
        final_created,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reserved_and_traversal_names_fail_before_mutation() {
        for name in [
            "",
            ".",
            "..",
            "../foreign",
            "CON.mp4",
            "NUL",
            "LPT1",
            "COM9.txt",
            "clip:",
            "trailing.",
            "trailing ",
            "한글",
        ] {
            assert!(!leaf(name), "{name}");
        }
        assert!(leaf("range-001"));
    }
    #[test]
    fn original_stage_identity_and_validation_gate_reject_external_writer() {
        let temporary = tempfile::tempdir().unwrap();
        let parent = open_directory(temporary.path()).unwrap();
        let root = create_relative(&parent, "fresh-export", true).unwrap();
        let root_path = crate::file_identity::final_path(&root).unwrap();
        let stage = create_relative(&root, "stage.mp4", false).unwrap();
        let stage_path = root_path.join("stage.mp4");
        let original = crate::file_identity::from_file(&stage).unwrap();
        let writer = OpenOptions::new()
            .write(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(&stage_path)
            .unwrap();
        assert!(
            OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .open(&stage_path)
                .is_err()
        );
        assert!(std::fs::remove_file(&stage_path).is_err());
        drop(writer);
        let validation = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&stage_path)
            .unwrap();
        assert_eq!(
            crate::file_identity::from_file(&validation).unwrap(),
            original
        );
        assert!(
            OpenOptions::new()
                .write(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .open(&stage_path)
                .is_err()
        );
        let final_path = root_path.join("clip.mp4");
        std::fs::hard_link(&stage_path, &final_path).unwrap();
        assert_eq!(
            crate::file_identity::from_file(&File::open(&final_path).unwrap()).unwrap(),
            original
        );
        drop(validation);
        drop(stage);
        drop(root);
        drop(parent);
        assert!(stage_path.exists());
        assert!(final_path.exists());
    }
    #[test]
    fn fresh_creation_and_final_link_never_overwrite_existing_bytes() {
        let temporary = tempfile::tempdir().unwrap();
        let parent = open_directory(temporary.path()).unwrap();
        let root = create_relative(&parent, "fresh-export", true).unwrap();
        assert!(create_relative(&parent, "fresh-export", true).is_err());
        let root_path = crate::file_identity::final_path(&root).unwrap();
        let stage = create_relative(&root, "stage.mp4", false).unwrap();
        assert!(create_relative(&root, "stage.mp4", false).is_err());
        let final_path = root_path.join("clip.mp4");
        std::fs::write(&final_path, b"foreign-original").unwrap();
        assert!(std::fs::hard_link(root_path.join("stage.mp4"), &final_path).is_err());
        assert_eq!(std::fs::read(&final_path).unwrap(), b"foreign-original");
        assert!(std::fs::rename(&root_path, temporary.path().join("changed")).is_err());
        drop(stage);
        drop(root);
        drop(parent);
    }
    #[test]
    fn times_use_literal_millisecond_argv_values() {
        assert_eq!(stamp(12345), "12.345");
    }
    #[test]
    fn long_range_export_seeks_before_opening_input_and_keeps_duration_output_scoped() {
        let source = PathBuf::from(r"C:\original source\long.mp4");
        let output = PathBuf::from(r"C:\fresh output\stage.mp4");
        let argv = export_arguments(&source, &output, 8 * 60 * 60 * 1000 - 1234, 2500);
        let input = argv.iter().position(|arg| arg == "-i").unwrap();
        let seek = argv.iter().position(|arg| arg == "-ss").unwrap();
        let duration = argv.iter().position(|arg| arg == "-t").unwrap();
        assert!(
            seek < input,
            "long source seek must not decode all preceding media"
        );
        assert!(argv[..input].iter().any(|arg| arg == "-accurate_seek"));
        assert_eq!(argv[seek + 1], std::ffi::OsString::from("28798.766"));
        assert!(duration > input);
        assert_eq!(argv[duration + 1], std::ffi::OsString::from("2.500"));
        assert_eq!(argv[input + 1], source.as_os_str());
        assert_eq!(argv.last().unwrap(), output.as_os_str());
        let output_threads = argv[input + 2..]
            .windows(2)
            .find(|pair| pair[0] == "-threads")
            .unwrap();
        assert_eq!(output_threads[1], std::ffi::OsString::from("1"));
        assert_eq!(source, PathBuf::from(r"C:\original source\long.mp4"));
    }
    // Append inside local_export.rs existing #[cfg(test)] mod tests.
    #[cfg(windows)]
    #[test]
    #[ignore = "root explicitly supplies pinned FFmpeg/ffprobe, owned TEMP and the 2s red-to-blue DLPAD_MEDIA_FIXTURE"]
    fn actual_synthetic_range_export_verifies_blue_clip_and_preserves_source_and_existing_outputs()
    {
        use crate::media_preview::{Product, Request, execute};
        use std::sync::atomic::Ordering;
        let source = PathBuf::from(
            std::env::var_os("DLPAD_MEDIA_FIXTURE").expect("explicit synthetic source"),
        );
        let ffmpeg =
            PathBuf::from(std::env::var_os("DLPAD_MEDIA_FFMPEG").expect("explicit pinned FFmpeg"));
        assert!(source.is_absolute() && ffmpeg.is_absolute());
        use std::io::Read;
        let original_file = File::open(&source).expect("original fixture read");
        let original_size = original_file.metadata().unwrap().len();
        assert!(original_size > 0 && original_size < 1_048_576);
        let mut original = Vec::new();
        original_file
            .take(1_048_576)
            .read_to_end(&mut original)
            .expect("bounded original fixture bytes");
        assert_eq!(original.len() as u64, original_size);
        let tools = Tools {
            ffprobe: ffmpeg.parent().expect("FFmpeg parent").join("ffprobe.exe"),
            ffmpeg,
            ytdlp: PathBuf::new(),
            deno: None,
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let deadline = || Instant::now() + Duration::from_secs(30);
        let prepared = match execute(
            &tools,
            Request::Prepare(Source::Local(source.clone())),
            &cancel,
            deadline(),
        ) {
            Ok(Product::Prepared(prepared)) => prepared,
            Err(error) => panic!("synthetic source metadata: {error:?}"),
            _ => panic!("expected prepared source"),
        };
        assert!((1900..=2100).contains(&prepared.duration_ms));
        // Parent root host binds TEMP to its project-owned fixture area. Never use a user folder.
        let temporary = tempfile::Builder::new()
            .prefix("dlpad-export-real-owned-")
            .tempdir()
            .expect("owned TEMP fixture");
        let output_parent = temporary
            .path()
            .canonicalize()
            .expect("original output parent");
        let session = PreviewSession::external();
        let request = |name: &str| ExportRequest {
            prepared: prepared.clone(),
            start_ms: 1100,
            end_ms: 1800,
            output_parent: output_parent.clone(),
            folder_name: name.into(),
        };
        let exported =
            match export_local_range(&tools, request("blue-clip"), &session, &cancel, deadline()) {
                Ok(exported) => exported,
                Err(failure) => panic!(
                    "real export error {:?}; fresh={} stage={} final={}",
                    failure.error,
                    failure.fresh_directory.is_some(),
                    failure.stage_preserved,
                    failure.final_created
                ),
            };
        assert!(
            (450..=950).contains(&exported.duration_ms),
            "nominal 700ms clip uses the existing 250ms duration tolerance"
        );
        assert_eq!(exported.path.file_name().unwrap(), "clip.mp4");
        assert_eq!(
            exported.retained_stage_path.file_name().unwrap(),
            "stage.mp4"
        );
        assert_eq!(
            exported.path.parent(),
            exported.retained_stage_path.parent()
        );
        let final_file = File::open(&exported.path).expect("original final read");
        let stage_file = File::open(&exported.retained_stage_path).expect("retained stage read");
        assert_eq!(
            crate::file_identity::from_file(&final_file).expect("final NativeID"),
            crate::file_identity::from_file(&stage_file).expect("stage NativeID")
        );
        assert_eq!(final_file.metadata().unwrap().len(), exported.bytes);
        assert_eq!(stage_file.metadata().unwrap().len(), exported.bytes);
        drop(final_file);
        drop(stage_file);
        let clip = match execute(
            &tools,
            Request::Prepare(Source::Local(exported.path.clone())),
            &cancel,
            deadline(),
        ) {
            Ok(Product::Prepared(clip)) => clip,
            Err(error) => panic!("exported clip probe: {error:?}"),
            _ => panic!("expected prepared exported clip"),
        };
        assert!((450..=950).contains(&clip.duration_ms));
        let frame = match execute(
            &tools,
            Request::Seek {
                prepared: clip.clone(),
                position_ms: 100,
            },
            &cancel,
            deadline(),
        ) {
            Ok(Product::Frame(frame)) => frame,
            Err(error) => panic!("exported clip frame: {error:?}"),
            _ => panic!("expected exported RGB frame"),
        };
        assert_eq!(frame.rgb.len(), super::super::FRAME_BYTES);
        let center = (90 * 320 + 160) * 3;
        assert!(
            frame.rgb[center + 2] > 200 && frame.rgb[center] < 40,
            "1100..1800ms must contain the blue source segment"
        );
        assert_eq!(std::fs::read(&source).unwrap(), original);

        let existing = output_parent.join("foreign-original");
        std::fs::create_dir(&existing).unwrap();
        let foreign_final = existing.join("clip.mp4");
        let foreign_stage = existing.join("stage.mp4");
        std::fs::write(&foreign_final, b"foreign final remains byte-identical").unwrap();
        std::fs::write(&foreign_stage, b"foreign stage remains byte-identical").unwrap();
        let existing_cap = open_directory(&existing).unwrap();
        let before_identity = crate::file_identity::from_file(&existing_cap).unwrap();
        match export_local_range(
            &tools,
            request("foreign-original"),
            &session,
            &cancel,
            deadline(),
        ) {
            Err(failure) => {
                assert_eq!(failure.error, Error::InvalidInput);
                assert!(
                    failure.fresh_directory.is_none()
                        && !failure.stage_preserved
                        && !failure.final_created
                );
            }
            Ok(_) => panic!("existing root must not be reused"),
        }
        assert_eq!(
            crate::file_identity::from_file(&open_directory(&existing).unwrap()).unwrap(),
            before_identity
        );
        assert_eq!(
            std::fs::read(&foreign_final).unwrap(),
            b"foreign final remains byte-identical"
        );
        assert_eq!(
            std::fs::read(&foreign_stage).unwrap(),
            b"foreign stage remains byte-identical"
        );
        drop(existing_cap);

        cancel.store(true, Ordering::Release);
        match export_local_range(&tools, request("pre-cancel"), &session, &cancel, deadline()) {
            Err(failure) => {
                assert_eq!(failure.error, Error::Cancelled);
                assert!(
                    failure.fresh_directory.is_none()
                        && !failure.stage_preserved
                        && !failure.final_created
                );
            }
            Ok(_) => panic!("pre-cancel export must fail"),
        }
        assert!(!output_parent.join("pre-cancel").exists());
        cancel.store(false, Ordering::Release);
        match export_local_range(
            &tools,
            request("elapsed-deadline"),
            &session,
            &cancel,
            Instant::now() - Duration::from_millis(1),
        ) {
            Err(failure) => {
                assert_eq!(failure.error, Error::TimedOut);
                assert!(
                    failure.fresh_directory.is_none()
                        && !failure.stage_preserved
                        && !failure.final_created
                );
            }
            Ok(_) => panic!("elapsed deadline export must fail"),
        }
        assert!(!output_parent.join("elapsed-deadline").exists());
        assert_eq!(std::fs::read(&source).unwrap(), original);
        assert_eq!(
            std::fs::read(&foreign_final).unwrap(),
            b"foreign final remains byte-identical"
        );
        // Final Prepared holds a no-delete pin, so release it before own tempfile teardown.
        drop(clip);
    }
    // Append inside local_export.rs existing tests module. Existing sha2 dependency only.
    fn late8h_streamed_source_sha(path: &Path) -> (u64, String) {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let mut file = File::open(path).expect("original bounded source reader");
        let original_size = file.metadata().unwrap().len();
        assert!(original_size > 0 && original_size <= 256 * 1024 * 1024);
        let identity = crate::file_identity::from_file(&file).unwrap();
        let mut digest = Sha256::new();
        let mut buffer = vec![0; 1024 * 1024];
        let mut total = 0u64;
        loop {
            let limit = buffer.len().min((original_size - total + 1) as usize);
            let count = file
                .read(&mut buffer[..limit])
                .expect("bounded source hash read");
            if count == 0 {
                break;
            }
            total += count as u64;
            assert!(total <= original_size, "source grew while hashing");
            digest.update(&buffer[..count]);
        }
        assert_eq!(total, original_size);
        assert_eq!(file.metadata().unwrap().len(), original_size);
        assert_eq!(
            crate::file_identity::from_file(&File::open(path).unwrap()).unwrap(),
            identity
        );
        let sha = digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        (total, sha)
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "root explicitly generates the real 8h owned fixture, records generation cost and provides pinned tool/source SHA"]
    fn actual_late_eight_hour_range_export_has_blue_frame_and_preserves_streamed_source_sha() {
        use crate::media_preview::{Product, Request, execute};
        let source = PathBuf::from(
            std::env::var_os("DLPAD_MEDIA_FIXTURE").expect("explicit generated 8h source"),
        );
        let expected_sha = std::env::var("DLPAD_MEDIA_FIXTURE_SHA256")
            .expect("generator source SHA binding")
            .to_ascii_lowercase();
        assert!(
            expected_sha.len() == 64 && expected_sha.bytes().all(|byte| byte.is_ascii_hexdigit())
        );
        let canonical = source.canonicalize().expect("original fixture path");
        let project = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let owned_temp = project
            .join(".local-tools/windows/tmp")
            .canonicalize()
            .expect("original project TEMP");
        let evidence = project
            .join("docs/validation/evidence")
            .canonicalize()
            .expect("original project evidence root");
        assert!(
            canonical.starts_with(&owned_temp) || canonical.starts_with(&evidence),
            "only project-owned TEMP/evidence sources accepted"
        );
        assert!(
            canonical
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("dlpad-late8h-owned-"),
            "fresh generator-owned fixture namespace required"
        );
        let ffmpeg =
            PathBuf::from(std::env::var_os("DLPAD_MEDIA_FFMPEG").expect("explicit pinned FFmpeg"));
        assert!(source.is_absolute() && ffmpeg.is_absolute());
        let tools = Tools {
            ffprobe: ffmpeg.parent().unwrap().join("ffprobe.exe"),
            ffmpeg,
            ytdlp: PathBuf::new(),
            deno: None,
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let deadline = || Instant::now() + Duration::from_secs(120);
        let prepared = match execute(
            &tools,
            Request::Prepare(Source::Local(source.clone())),
            &cancel,
            deadline(),
        ) {
            Ok(Product::Prepared(prepared)) => prepared,
            Err(error) => panic!("8h original probe: {error:?}"),
            _ => panic!("expected original prepared video"),
        };
        assert!(prepared.duration_ms.abs_diff(28_800_000) <= 500);
        let before = late8h_streamed_source_sha(&source);
        assert_eq!(before.1, expected_sha);
        let temporary = tempfile::Builder::new()
            .prefix("dlpad-export-late8h-owned-")
            .tempdir_in(&owned_temp)
            .expect("fresh project-owned export parent")
            .keep(); // Preserve test outputs and partial facts for root evidence review.
        let output_parent = temporary.canonicalize().unwrap();
        let session = PreviewSession::external();
        let start = Instant::now();
        let exported = match export_local_range(
            &tools,
            ExportRequest {
                prepared: prepared.clone(),
                start_ms: 28_798_250,
                end_ms: 28_799_750,
                output_parent,
                folder_name: "late-blue".into(),
            },
            &session,
            &cancel,
            deadline(),
        ) {
            Ok(exported) => exported,
            Err(failure) => panic!(
                "late8h export {:?}; fresh={} stage={} final={}",
                failure.error,
                failure.fresh_directory.is_some(),
                failure.stage_preserved,
                failure.final_created
            ),
        };
        println!(
            "late8h_export_elapsed_seconds={:.6}; source_bytes={}; source_sha256={}",
            start.elapsed().as_secs_f64(),
            before.0,
            before.1
        );
        assert!(exported.duration_ms.abs_diff(1500) <= 500);
        assert!(exported.bytes > 0 && exported.bytes <= 256 * 1024 * 1024);
        let final_file = File::open(&exported.path).unwrap();
        let stage_file = File::open(&exported.retained_stage_path).unwrap();
        assert_eq!(
            crate::file_identity::from_file(&final_file).unwrap(),
            crate::file_identity::from_file(&stage_file).unwrap()
        );
        assert_eq!(final_file.metadata().unwrap().len(), exported.bytes);
        assert_eq!(stage_file.metadata().unwrap().len(), exported.bytes);
        drop(final_file);
        drop(stage_file);
        let clip = match execute(
            &tools,
            Request::Prepare(Source::Local(exported.path.clone())),
            &cancel,
            deadline(),
        ) {
            Ok(Product::Prepared(clip)) => clip,
            Err(error) => panic!("late exported clip probe: {error:?}"),
            _ => panic!("expected exported clip"),
        };
        assert!(clip.duration_ms.abs_diff(1500) <= 500);
        let frame = match execute(
            &tools,
            Request::Seek {
                prepared: clip.clone(),
                position_ms: 100,
            },
            &cancel,
            deadline(),
        ) {
            Ok(Product::Frame(frame)) => frame,
            Err(error) => panic!("late blue frame: {error:?}"),
            _ => panic!("expected actual exported frame"),
        };
        assert_eq!(frame.rgb.len(), super::super::FRAME_BYTES);
        let center = (90 * 320 + 160) * 3;
        assert!(frame.rgb[center + 2] > 200 && frame.rgb[center] < 40);
        assert_eq!(late8h_streamed_source_sha(&source), before);
        drop(clip); // Output directory remains deliberately retained; no pathname cleanup.
    }
}

// Append only; root integrates into local_export.rs after selecting timestamp policy.
// Native/Cargo execution is required before this can be accepted.
#[cfg(test)]
mod av_actual_tests {
    use super::*;
    use crate::media_preview::{Product, Request, execute};

    fn original_sha(path: &Path) -> (u64, String) {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let mut file = File::open(path).expect("original bounded source reader");
        let original_size = file.metadata().unwrap().len();
        assert!(original_size > 0 && original_size <= 256 * 1024 * 1024);
        let identity = crate::file_identity::from_file(&file).unwrap();
        let mut digest = Sha256::new();
        let mut buffer = vec![0; 1024 * 1024];
        let mut total = 0u64;
        loop {
            let limit = buffer.len().min((original_size - total + 1) as usize);
            let count = file
                .read(&mut buffer[..limit])
                .expect("bounded source hash read");
            if count == 0 {
                break;
            }
            total += count as u64;
            assert!(total <= original_size, "source grew while hashing");
            digest.update(&buffer[..count]);
        }
        assert_eq!(total, original_size);
        assert_eq!(file.metadata().unwrap().len(), original_size);
        assert_eq!(
            crate::file_identity::from_file(&File::open(path).unwrap()).unwrap(),
            identity
        );
        let sha = digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        (total, sha)
    }

    fn run_bytes(tool: &PathBuf, argv: Vec<std::ffi::OsString>) -> Vec<u8> {
        capture(
            tool,
            argv,
            &Arc::new(AtomicBool::new(false)),
            Instant::now() + Duration::from_secs(30),
            Duration::from_secs(30),
            &PreviewSession::external(),
            CaptureRole::DirectProcessing,
        )
        .unwrap()
    }
    fn flash_time(tools: &Tools, path: &Path) -> f64 {
        let mut probe = args(&[
            "-v",
            "error",
            "-protocol_whitelist",
            "file",
            "-select_streams",
            "v:0",
            "-read_intervals",
            "0%4",
            "-show_frames",
            "-show_entries",
            "frame=best_effort_timestamp_time",
            "-of",
            "json",
            "-i",
        ]);
        probe.push(path.as_os_str().into());
        let raw = run_bytes(&tools.ffprobe, probe);
        let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let frames = value["frames"].as_array().unwrap();
        assert!(!frames.is_empty() && frames.len() <= 16);
        let mut decode = args(&[
            "-nostdin",
            "-v",
            "error",
            "-threads",
            "1",
            "-filter_threads",
            "1",
            "-max_alloc",
            "33554432",
            "-protocol_whitelist",
            "file",
            "-i",
        ]);
        decode.push(path.as_os_str().into());
        decode.extend(args(&[
            "-t",
            "4",
            "-map",
            "0:v:0",
            "-an",
            "-sn",
            "-dn",
            "-vf",
            "scale=1:1",
            "-fps_mode:v",
            "passthrough",
            "-pix_fmt",
            "gray",
            "-f",
            "rawvideo",
            "pipe:1",
        ]));
        let pixels = run_bytes(&tools.ffmpeg, decode);
        assert_eq!(pixels.len(), frames.len(), "decoded frame/PTS pairing");
        let index = pixels
            .iter()
            .position(|v| *v >= 180)
            .expect("white flash retained");
        let time = frames[index]["best_effort_timestamp_time"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap();
        assert!(time.is_finite() && time >= 0.0);
        time
    }
    fn tone_time(tools: &Tools, path: &Path, window_start_ms: u64) -> f64 {
        // 0.8s * 8000Hz * mono16bit = 12800 bytes, bounded far below the existing 8MiB metadata stdout cap.
        let mut decode = args(&[
            "-nostdin",
            "-v",
            "error",
            "-threads",
            "1",
            "-max_alloc",
            "33554432",
            "-protocol_whitelist",
            "file",
            "-ss",
            &stamp(window_start_ms),
            "-i",
        ]);
        decode.push(path.as_os_str().into());
        decode.extend(args(&[
            "-t",
            "0.800",
            "-map",
            "0:a:0",
            "-vn",
            "-sn",
            "-dn",
            "-ac",
            "1",
            "-af",
            "aresample=8000:first_pts=0",
            "-ar",
            "8000",
            "-f",
            "s16le",
            "pipe:1",
        ]));
        let raw = run_bytes(&tools.ffmpeg, decode);
        assert!(!raw.is_empty() && raw.len() <= 12800 && raw.len().is_multiple_of(2));
        let samples: Vec<i16> = raw
            .chunks_exact(2)
            .map(|v| i16::from_le_bytes([v[0], v[1]]))
            .collect();
        // Eight samples (1ms) avoid selecting a zero crossing as onset.
        let onset = samples
            .windows(8)
            .position(|w| w.iter().map(|s| i32::from(*s).abs()).sum::<i32>() > 8 * 2000)
            .expect("bounded audible tone retained");
        window_start_ms as f64 / 1000.0 + onset as f64 / 8000.0
    }
    #[test]
    #[ignore = "root supplies pinned 4s owned A/V fixtures, pinned tools and explicit project-owned parent; private guarded native host only"]
    fn actual_av_export_preserves_common_flash_tone_phase_and_intentional_audio_offset() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let temp = project
            .join(".local-tools/windows/tmp")
            .canonicalize()
            .unwrap();
        let parent = PathBuf::from(
            std::env::var_os("DLPAD_AV_EXPORT_PARENT").expect("explicit owned parent"),
        );
        let parent = parent.canonicalize().unwrap();
        assert!(
            parent.starts_with(&temp)
                && parent
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("dlpad-av-export-owned-")
        );
        let ffmpeg = PathBuf::from(std::env::var_os("DLPAD_MEDIA_FFMPEG").unwrap());
        let ffprobe = ffmpeg.parent().unwrap().join("ffprobe.exe");
        assert!(
            ffmpeg
                .canonicalize()
                .unwrap()
                .starts_with(project.join(".local-tools").canonicalize().unwrap())
        );
        assert!(
            ffprobe
                .canonicalize()
                .unwrap()
                .starts_with(project.join(".local-tools").canonicalize().unwrap())
        );
        for (path, key) in [
            (&ffmpeg, "DLPAD_MEDIA_FFMPEG_SHA256"),
            (&ffprobe, "DLPAD_MEDIA_FFPROBE_SHA256"),
        ] {
            assert_eq!(original_sha(path).1, std::env::var(key).unwrap());
        }
        let tools = Tools {
            ffmpeg,
            ffprobe,
            ytdlp: PathBuf::new(),
            deno: None,
        };
        let session = PreviewSession::external();
        let cancel = Arc::new(AtomicBool::new(false));
        for (case, source_key, sha_key, offset) in [
            (
                "aligned",
                "DLPAD_AV_ALIGNED_FIXTURE",
                "DLPAD_AV_ALIGNED_SHA256",
                0.0,
            ),
            (
                "delayed",
                "DLPAD_AV_DELAYED_FIXTURE",
                "DLPAD_AV_DELAYED_SHA256",
                0.2,
            ),
        ] {
            let source = PathBuf::from(std::env::var_os(source_key).unwrap());
            let canonical = source.canonicalize().unwrap();
            assert!(
                canonical.starts_with(&temp)
                    && canonical
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("dlpad-av-fixture-owned-")
            );
            let original = original_sha(&source);
            assert_eq!(original.1, std::env::var(sha_key).unwrap());
            assert!(original.0 <= 16 * 1024 * 1024);
            let prepared = match execute(
                &tools,
                Request::Prepare(Source::Local(source.clone())),
                &cancel,
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap()
            {
                Product::Prepared(p) => p,
                _ => panic!("original prepared"),
            };
            let original_flash = flash_time(&tools, &source);
            let original_tone = tone_time(&tools, &source, 750);
            assert!((original_flash - 1.0).abs() <= 0.001);
            assert!(
                (original_tone - (1.0 + offset)).abs() <= 0.030,
                "fixture event phase measured, not assumed"
            );
            for start in [750, 1000] {
                let exported = match export_local_range(
                    &tools,
                    ExportRequest {
                        prepared: prepared.clone(),
                        start_ms: start,
                        end_ms: start + 1500,
                        output_parent: parent.clone(),
                        folder_name: format!("{case}-{start}"),
                    },
                    &session,
                    &cancel,
                    Instant::now() + Duration::from_secs(120),
                ) {
                    Ok(v) => v,
                    Err(f) => panic!(
                        "A/V export {:?}; partial preserved={}",
                        f.error, f.stage_preserved
                    ),
                };
                assert!(exported.duration_ms.abs_diff(1500) <= 250);
                let flash = flash_time(&tools, &exported.path);
                let tone = tone_time(&tools, &exported.path, 0);
                let expected_flash = original_flash - start as f64 / 1000.0;
                let expected_tone = original_tone - start as f64 / 1000.0;
                // Millisecond encoder epoch must preserve video phase, not a permissive 500ms frame bound.
                assert!(
                    (flash - expected_flash).abs() <= 0.0011,
                    "video common epoch drift"
                );
                // AAC is lossy; this candidate's 30ms onset budget requires native calibration.
                assert!((tone - expected_tone).abs() <= 0.030, "audio onset drift");
                assert!(
                    ((flash - tone) - (original_flash - original_tone)).abs() <= 0.0311,
                    "A/V phase drift"
                );
                println!(
                    "avcase={case} start={start} original_flash={original_flash:.6} original_tone={original_tone:.6} output_flash={flash:.6} output_tone={tone:.6}"
                );
                assert_eq!(original_sha(&source), original);
            }
        }
        for (path, key) in [
            (&tools.ffmpeg, "DLPAD_MEDIA_FFMPEG_SHA256"),
            (&tools.ffprobe, "DLPAD_MEDIA_FFPROBE_SHA256"),
        ] {
            assert_eq!(original_sha(path).1, std::env::var(key).unwrap());
        }
    }
}
