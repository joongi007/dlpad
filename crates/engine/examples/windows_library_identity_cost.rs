//! Fixed owned-fixture microprobe, not a scanner policy change or CPU phase profiler.
#[cfg(windows)]
mod native {
    use dlpad_engine::file_identity::{self, NativeFileIdentity};
    use std::{fs, io, time::Instant};
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentProcess, GetProcessTimes},
    };
    const COUNT: usize = 10_000;
    const DEADLINE_SECS: u64 = 120;
    const PHASES: [&str; 7] = [
        "path_metadata",
        "open_regular",
        "handle_final_path",
        "canonicalize",
        "handle_metadata",
        "native_id",
        "identity_encoding",
    ];
    fn rejected() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "fixed identity experiment rejected",
        )
    }
    fn ticks(t: FILETIME) -> u64 {
        (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
    }
    fn cpu() -> io::Result<(u64, u64, u64)> {
        let (mut c, mut e, mut k, mut u) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        // SAFETY: process-owned pseudo handle and initialized writable FILETIME outputs.
        if unsafe { GetProcessTimes(GetCurrentProcess(), &mut c, &mut e, &mut k, &mut u) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((ticks(c), ticks(k), ticks(u)))
    }
    fn elapsed(t: Instant) -> u64 {
        u64::try_from(t.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }
    fn add(times: &mut [u64; 7], i: usize, start: Instant) {
        times[i] = times[i].saturating_add(elapsed(start));
    }
    pub fn run() -> io::Result<()> {
        let mut args = std::env::args_os().skip(1);
        if args.next() != Some(std::ffi::OsString::from("--fixed-identity-cost-experiment"))
            || args.next().is_some()
        {
            return Err(rejected());
        }
        let experiment_start = Instant::now();
        let fixture = tempfile::Builder::new()
            .prefix("dlpad-identity-cost-owned-")
            .tempdir()?;
        let kept = fixture.keep();
        let root = fs::canonicalize(&kept)?;
        use std::io::Write;
        let initial = serde_json::to_vec(
            &serde_json::json!({"phase":"created", "owned_fixture_root":root, "synthetic_only":true}),
        )?;
        if initial.len() > 8192 {
            return Err(rejected());
        }
        let mut initial_stdout = io::stdout().lock();
        initial_stdout.write_all(&initial)?;
        initial_stdout.write_all(b"\n")?;
        initial_stdout.flush()?;
        drop(initial_stdout);
        let sentinel = root.join("sentinel");
        fs::write(&sentinel, b"preserve owned sentinel")?;
        let files = root.join("synthetic");
        fs::create_dir(&files)?;
        for i in 0..COUNT {
            if experiment_start.elapsed().as_secs() >= DEADLINE_SECS {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "fixture creation deadline",
                ));
            }
            use std::io::Write;
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(files.join(format!("synthetic-{i:05}.mp4")))?;
            f.write_all(b"abc")?;
        }
        let creation = cpu()?.0;
        let mut previous_cpu = cpu()?;
        let mut rounds = Vec::with_capacity(3); // Fixed three aggregate records, never per-file observations.
        for round in 0..3 {
            let before = cpu()?;
            if before.0 != creation || before.1 < previous_cpu.1 || before.2 < previous_cpu.2 {
                return Err(rejected());
            }
            let wall = Instant::now();
            let mut times = [0_u64; 7];
            let mut observed = 0;
            for entry in fs::read_dir(&files)? {
                if experiment_start.elapsed().as_secs() >= DEADLINE_SECS {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "fixed identity experiment deadline",
                    ));
                }
                let path = entry?.path();
                let t = Instant::now();
                let metadata = fs::symlink_metadata(&path)?;
                add(&mut times, 0, t);
                if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() != 3 {
                    return Err(rejected());
                }
                let t = Instant::now();
                let file = file_identity::open_regular_no_reparse(&path)?;
                add(&mut times, 1, t);
                let t = Instant::now();
                let final_path = file_identity::final_path(&file)?;
                add(&mut times, 2, t);
                let t = Instant::now();
                let canonical = fs::canonicalize(&path)?;
                add(&mut times, 3, t);
                if final_path != canonical {
                    return Err(rejected());
                }
                let t = Instant::now();
                let current = file.metadata()?;
                add(&mut times, 4, t);
                if current.len() != metadata.len()
                    || current.modified().ok() != metadata.modified().ok()
                {
                    return Err(rejected());
                }
                let t = Instant::now();
                let identity = file_identity::from_file(&file)?;
                add(&mut times, 5, t);
                let NativeFileIdentity::Windows {
                    volume_serial,
                    file_id,
                } = identity
                else {
                    return Err(rejected());
                };
                let t = Instant::now();
                let id = file_id
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let encoded = format!("windows:v1:{volume_serial}:{id}");
                add(&mut times, 6, t);
                if id.len() != 32 || encoded.len() > 80 {
                    return Err(rejected());
                }
                observed += 1;
                if observed > COUNT {
                    return Err(rejected());
                }
            }
            let after = cpu()?;
            if after.0 != creation {
                return Err(rejected());
            }
            let kernel_ticks = after.1.checked_sub(before.1).ok_or_else(rejected)?;
            let user_ticks = after.2.checked_sub(before.2).ok_or_else(rejected)?;
            let cpu_ticks = kernel_ticks.checked_add(user_ticks).ok_or_else(rejected)?;
            previous_cpu = after;
            if observed != COUNT {
                return Err(rejected());
            }
            let phase: Vec<_>=PHASES.iter().zip(times).map(|(name,ns)|serde_json::json!({"phase":name,"wall_nanoseconds":ns,"calls":COUNT})).collect();
            rounds.push(serde_json::json!({"round":round,"observed_some":observed,"whole_wall_nanoseconds":elapsed(wall),"whole_process_cpu_100ns":cpu_ticks,"kernel_100ns":kernel_ticks,"user_100ns":user_ticks,"creation_filetime":creation,"phases":phase}));
        }
        if fs::read(&sentinel)? != b"preserve owned sentinel" {
            return Err(rejected());
        }
        let output = serde_json::to_vec(
            &serde_json::json!({"scope":"Windows original identity full-chain owned microprobe; phase wall only, whole process CPU; no SQL/UI; warm repeats without OS cache purge","owned_fixture_root":root,"synthetic_only":true,"synthetic_files":COUNT,"bytes_each":3,"rounds":rounds,"sentinel_preserved":true}),
        )?;
        if output.len() > 8192 {
            return Err(rejected());
        }
        let mut stdout = io::stdout().lock();
        stdout.write_all(&output)?;
        stdout.write_all(b"\n")?;
        Ok(())
    }
}
fn main() {
    #[cfg(windows)]
    if let Err(error) = native::run() {
        eprintln!("fixed identity cost experiment failed: {:?}", error.kind());
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("fixed identity cost experiment unsupported");
        std::process::exit(2);
    }
}
