use dlpad_engine::tools::{
    ArchiveComponent, ArchiveComponentRole, TarComponentMapping, TarExtractionRequest, TarLimits,
    TarXzSourceSpec, ToolError, extract_verified_tar_xz,
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
const VALID: &[u8] = &[
    253, 55, 122, 88, 90, 0, 0, 4, 230, 214, 180, 70, 2, 0, 33, 1, 22, 0, 0, 0, 116, 47, 229, 163,
    224, 7, 255, 0, 62, 93, 0, 57, 27, 236, 232, 47, 37, 78, 155, 192, 90, 38, 131, 209, 31, 67,
    231, 82, 39, 237, 224, 103, 128, 2, 144, 210, 59, 249, 111, 12, 195, 29, 20, 65, 207, 19, 57,
    209, 136, 92, 105, 20, 114, 2, 176, 202, 9, 249, 153, 240, 145, 2, 67, 106, 19, 180, 64, 16, 0,
    103, 9, 100, 0, 0, 0, 0, 49, 122, 82, 147, 212, 17, 103, 149, 0, 1, 90, 128, 16, 0, 0, 0, 65,
    24, 191, 143, 177, 196, 103, 251, 2, 0, 0, 0, 0, 4, 89, 90,
];
const BIG_DICTIONARY: &[u8] = &[
    253, 55, 122, 88, 90, 0, 0, 4, 230, 214, 180, 70, 2, 0, 33, 1, 30, 0, 0, 0, 155, 7, 81, 102,
    224, 7, 255, 0, 62, 93, 0, 57, 27, 236, 232, 47, 37, 78, 155, 192, 90, 38, 131, 209, 31, 67,
    231, 82, 39, 237, 224, 103, 128, 2, 144, 210, 59, 249, 111, 12, 195, 29, 20, 65, 207, 19, 57,
    209, 136, 92, 105, 20, 114, 2, 176, 202, 9, 249, 153, 240, 145, 2, 67, 106, 19, 180, 64, 16, 0,
    103, 9, 100, 0, 0, 0, 0, 49, 122, 82, 147, 212, 17, 103, 149, 0, 1, 90, 128, 16, 0, 0, 0, 65,
    24, 191, 143, 177, 196, 103, 251, 2, 0, 0, 0, 0, 4, 89, 90,
];
struct Fixture {
    root: PathBuf,
    source: File,
    output: File,
    original: Vec<u8>,
    _guard: tempfile::TempDir,
}
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let guard = tempfile::Builder::new()
            .prefix("dlpad-tar-xz-")
            .tempdir()
            .unwrap();
        let root = guard.path().to_path_buf();
        fs::write(root.join("source.xz"), bytes).unwrap();
        fs::write(root.join("sentinel"), b"foreign preserved").unwrap();
        let source = File::open(root.join("source.xz")).unwrap();
        let output = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(root.join("output"))
            .unwrap();
        Self {
            root,
            source,
            output,
            original: bytes.to_vec(),
            _guard: guard,
        }
    }
    fn run(
        &mut self,
        wrong_hash: bool,
        wrong_size: bool,
        cancel: bool,
        expired: bool,
    ) -> Result<u64, ToolError> {
        let sha = if wrong_hash {
            "0".repeat(64)
        } else {
            format!("{:x}", Sha256::digest(&self.original))
        };
        let mappings = [TarComponentMapping {
            member: "root/bin/tool".into(),
            component: ArchiveComponent {
                name: "tool".into(),
                bytes: 3,
                sha256: format!("{:x}", Sha256::digest(b"abc")),
                role: ArchiveComponentRole::Executable,
            },
        }];
        let limits = TarLimits {
            max_entries: 8,
            max_total_bytes: 1024,
            max_selected_bytes: 512,
            max_stream_bytes: 8192,
        };
        extract_verified_tar_xz(
            &self.source,
            TarXzSourceSpec {
                bytes: self.original.len() as u64 + u64::from(wrong_size),
                sha256: &sha,
            },
            TarExtractionRequest {
                root: "root",
                mappings: &mappings,
                selected: "tool",
                limits: &limits,
            },
            &mut self.output,
            &AtomicBool::new(cancel),
            if expired {
                Instant::now()
            } else {
                Instant::now() + Duration::from_secs(3)
            },
        )
    }
    fn preserved(&self) {
        assert_eq!(
            fs::read(self.root.join("source.xz")).unwrap(),
            self.original
        );
        assert_eq!(
            fs::read(self.root.join("sentinel")).unwrap(),
            b"foreign preserved"
        );
    }
}
#[test]
fn valid_source_and_component() {
    let mut f = Fixture::new(VALID);
    assert_eq!(f.run(false, false, false, false).unwrap(), 3);
    f.output.seek(SeekFrom::Start(0)).unwrap();
    let mut bytes = Vec::new();
    f.output.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"abc");
    f.preserved();
}
#[test]
fn source_hash_size_cancel_and_deadline() {
    for flags in [
        (true, false, false, false),
        (false, true, false, false),
        (false, false, true, false),
        (false, false, false, true),
    ] {
        let mut f = Fixture::new(VALID);
        let result = f.run(flags.0, flags.1, flags.2, flags.3);
        assert!(result.is_err());
        if flags.2 {
            assert!(matches!(result, Err(ToolError::Cancelled)));
        }
        if flags.3 {
            assert!(matches!(result, Err(ToolError::TimedOut)));
        }
        assert_eq!(f.output.metadata().unwrap().len(), 0);
        f.preserved();
    }
}
#[test]
fn compressed_corruption_truncation_tail_concat_and_dictionary() {
    let mut corrupt = VALID.to_vec();
    corrupt[32] ^= 1;
    let mut tail = VALID.to_vec();
    tail.extend(b"nonzero trailing garbage");
    let mut concat = VALID.to_vec();
    concat.extend(VALID);
    for bytes in [
        VALID[..VALID.len() - 1].to_vec(),
        VALID[..20].to_vec(),
        corrupt,
        tail,
        concat,
    ] {
        let mut f = Fixture::new(&bytes);
        assert!(
            f.run(false, false, false, false).is_err(),
            "malformed compressed fixture accepted"
        );
        f.preserved();
    }
}
#[test]
fn nonempty_output_is_preserved() {
    let mut f = Fixture::new(VALID);
    f.output.write_all(b"existing output").unwrap();
    f.output.seek(SeekFrom::Start(0)).unwrap();
    assert!(f.run(false, false, false, false).is_err());
    assert_eq!(fs::read(f.root.join("output")).unwrap(), b"existing output");
    f.preserved();
}

#[test]
fn oversized_dictionary_hits_memory_gate() {
    let mut f = Fixture::new(BIG_DICTIONARY);
    let error = f.run(false, false, false, false).unwrap_err();
    match error {
        ToolError::Io(error) => assert_eq!(error.kind(), std::io::ErrorKind::OutOfMemory),
        other => panic!("dictionary did not hit memory gate: {other:?}"),
    }
    f.preserved();
}
