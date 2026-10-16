//! Bounded immutable Windows cookie ownership binding.
//! Marker consistency is not deletion authority: actual receipt/lock/allowlist
//! and original native handle provenance remain necessary at every removal.
use super::{acl, kind};
use crate::{
    cookies::windows_records::{SessionAnchors, WindowsAnchor},
    file_identity::{self, NativeFileIdentity},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

pub(super) const MARKER_LIMIT: usize = 4096;
pub(super) const MARKER_NAME: &str = "owner.marker";
pub(super) const OWNER_ENTRIES: [&str; 5] = [
    "cookies.txt",
    "reader-state.json",
    "reader0.receipt",
    "reader1.receipt",
    MARKER_NAME,
];
#[derive(Clone, Copy)]
pub(super) struct MarkerError;
impl fmt::Debug for MarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WindowsMarkerError")
    }
}
impl fmt::Display for MarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WindowsMarkerError")
    }
}
impl std::error::Error for MarkerError {}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Marker {
    version: u8,
    session_nonce: [u8; 16],
    base_digest: [u8; 32],
    originals: SessionAnchors,
    marker_id: WindowsAnchor,
    payload_digest: [u8; 32],
}
impl fmt::Debug for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WindowsMarkerRedacted")
    }
}
fn anchor(h: &mut Sha256, a: WindowsAnchor) {
    h.update(a.volume_serial.to_le_bytes());
    h.update(a.file_id);
}
fn valid_anchor(a: WindowsAnchor) -> bool {
    a.file_id != [0; 16]
}
pub(super) fn base_digest(
    session_nonce: [u8; 16],
    cache: WindowsAnchor,
    owner: WindowsAnchor,
    copy: WindowsAnchor,
) -> Result<[u8; 32], MarkerError> {
    if session_nonce == [0; 16] || ![cache, owner, copy].into_iter().all(valid_anchor) {
        return Err(MarkerError);
    }
    let mut h = Sha256::new();
    h.update(b"dlpad/windows-cookie-base/v1\0");
    h.update(session_nonce);
    for a in [cache, owner, copy] {
        anchor(&mut h, a);
    }
    Ok(h.finalize().into())
}
impl Marker {
    /// Only call with IDs observed from import's retained actual CREATE_NEW handles.
    pub(super) fn from_created(
        session_nonce: [u8; 16],
        originals: SessionAnchors,
        marker_id: WindowsAnchor,
    ) -> Result<Self, MarkerError> {
        let mut value = Self {
            version: 1,
            session_nonce,
            base_digest: base_digest(
                session_nonce,
                originals.cache,
                originals.owner,
                originals.copy,
            )?,
            originals,
            marker_id,
            payload_digest: [0; 32],
        };
        value.payload_digest = value.digest();
        value.validate()?;
        Ok(value)
    }
    fn digest(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"dlpad/windows-cookie-marker/v1\0");
        h.update([self.version]);
        h.update(self.session_nonce);
        h.update(self.base_digest);
        for a in [
            self.originals.cache,
            self.originals.owner,
            self.originals.copy,
            self.originals.journal,
            self.originals.receipts[0],
            self.originals.receipts[1],
            self.marker_id,
        ] {
            anchor(&mut h, a);
        }
        h.finalize().into()
    }
    pub(super) fn validate(&self) -> Result<(), MarkerError> {
        let all = [
            self.originals.cache,
            self.originals.owner,
            self.originals.copy,
            self.originals.journal,
            self.originals.receipts[0],
            self.originals.receipts[1],
            self.marker_id,
        ];
        if self.version != 1
            || !all.into_iter().all(valid_anchor)
            || self.base_digest
                != base_digest(
                    self.session_nonce,
                    self.originals.cache,
                    self.originals.owner,
                    self.originals.copy,
                )?
            || self.payload_digest != self.digest()
        {
            return Err(MarkerError);
        }
        // Original objects must be distinct; each actual file must also have nlink1.
        for (i, a) in all.iter().enumerate() {
            if all[..i].contains(a) {
                return Err(MarkerError);
            }
        }
        Ok(())
    }
    pub(super) fn encode(&self) -> Result<Vec<u8>, MarkerError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| MarkerError)?;
        if bytes.len() > MARKER_LIMIT {
            return Err(MarkerError);
        }
        Ok(bytes)
    }
    pub(super) fn decode(bytes: &[u8]) -> Result<Self, MarkerError> {
        if bytes.len() > MARKER_LIMIT {
            return Err(MarkerError);
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| MarkerError)?;
        value.validate()?;
        Ok(value)
    }
    pub(super) fn journal_seed(&self) -> (SessionAnchors, [u8; 16], [u8; 32]) {
        (self.originals, self.session_nonce, self.base_digest)
    }
    /// Consistency only, never grants removal by itself. Caller must retain
    /// namespace pins and independently prove receipt completion under journal lock.
    pub(super) fn check_actual(&self, sid: &str, handles: [&File; 4]) -> Result<(), MarkerError> {
        self.validate()?;
        let expected = [
            self.originals.cache,
            self.originals.owner,
            self.originals.copy,
            self.marker_id,
        ];
        for (i, (file, original)) in handles.into_iter().zip(expected).enumerate() {
            kind(file, i < 2).map_err(|_| MarkerError)?;
            acl(file, sid).map_err(|_| MarkerError)?;
            if native_anchor(file)? != original {
                return Err(MarkerError);
            }
        }
        // Journal::reopen must independently validate original journal/receipt
        // handles from journal_seed(), hold its lock, and match actual receipts.
        // Never replace durable originals with newly observed current IDs.
        Ok(())
    }
}
fn native_anchor(file: &File) -> Result<WindowsAnchor, MarkerError> {
    match file_identity::from_file(file).map_err(|_| MarkerError)? {
        NativeFileIdentity::Windows {
            volume_serial,
            file_id,
        } if file_id != [0; 16] => Ok(WindowsAnchor {
            volume_serial,
            file_id,
        }),
        _ => Err(MarkerError),
    }
}
pub(super) fn read_marker(file: &File) -> Result<Marker, MarkerError> {
    let length = file.metadata().map_err(|_| MarkerError)?.len();
    if length > MARKER_LIMIT as u64 {
        return Err(MarkerError);
    }
    let mut source = file;
    source.seek(SeekFrom::Start(0)).map_err(|_| MarkerError)?;
    let mut bytes = Vec::with_capacity(length as usize);
    source
        .take((MARKER_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| MarkerError)?;
    if bytes.len() as u64 != length || bytes.len() > MARKER_LIMIT {
        return Err(MarkerError);
    }
    Marker::decode(&bytes)
}
// PRODUCT ORDER (root integration, intentionally not executable draft):
// 1. Hold original namespace pins, private SID ACL, cache/owner/copy handles.
// 2. sync copy; random nonce; compute base_digest from actual cache/owner/copy.
// 3. Journal::create(... nonce, base, actual_cache, actual_copy); retain lock.
// 4. CREATE_NEW marker with Descriptor::private; original marker native ID.
// 5. Marker::from_created(nonce, journal.anchors(), marker ID); bounded write,
//    sync_all, re-read exact bytes/actual identities; never reopen with new IDs.
// 6. ONLY NOW publish Initialized lease and allow begin intent.
// Any partial journal/marker failure after step3: defer, never old one-file cleanup.
// Recovery: bounded128 namespace scan -> validate strict5entries/pins/ACL ->
// marker read/check_actual -> Journal::reopen(originals,nonce,base) lock ->
// all_receipts_match() true -> repeat all identity/namespace/allowlist checks ->
// deletion via validated original DELETE handles. There is no deletion API here.

#[cfg(test)]
mod tests {
    // Append as #[cfg(test)] mod tests in recovery_marker.rs. Native Windows only.
    // Uses existing TestFixture; no new private journal-handle bridge is required.
    // Fixture::create currently uses arbitrary nonzero journal digest [2;32], so
    // these test marker/native consistency, NOT final production journal_seed binding.
    // Add TestFixture::create_with_base_digest() before testing integrated reopen.
    use super::super::{Descriptor, META_ACCESS, SHARE_ALL, current_sid, open};
    use super::*;
    use crate::cookies::windows::test_fixture::TestFixture;
    use std::{fs, io::Write, os::windows::io::AsRawHandle};
    use windows_sys::Win32::{
        Security::{
            Authorization::{SE_FILE_OBJECT, SetSecurityInfo},
            DACL_SECURITY_INFORMATION, UNPROTECTED_DACL_SECURITY_INFORMATION,
        },
        Storage::FileSystem::{FILE_GENERIC_READ, FILE_GENERIC_WRITE, WRITE_DAC},
    };

    struct NativeMarker {
        fixture: TestFixture,
        marker: Marker,
        file: File,
        sid: String,
        cache: File,
        owner: File,
        copy: File,
    }
    impl NativeMarker {
        fn create() -> Self {
            let fixture = TestFixture::create_with_base_digest();
            let sid = current_sid().unwrap();
            let path = fixture.owner_path();
            let cache = open(path.parent().unwrap(), true, META_ACCESS, SHARE_ALL, None).unwrap();
            let owner = open(path, true, META_ACCESS, SHARE_ALL, None).unwrap();
            let copy = open(
                &path.join("cookies.txt"),
                false,
                META_ACCESS,
                SHARE_ALL,
                None,
            )
            .unwrap();
            let descriptor = Descriptor::private(&sid).unwrap();
            let mut file = open(
                &path.join(MARKER_NAME),
                false,
                FILE_GENERIC_READ | FILE_GENERIC_WRITE | WRITE_DAC,
                SHARE_ALL,
                Some(&descriptor),
            )
            .unwrap();
            let marker = Marker::from_created(
                [1; 16],
                fixture.record().anchors(),
                native_anchor(&file).unwrap(),
            )
            .unwrap();
            file.write_all(&marker.encode().unwrap()).unwrap();
            file.sync_all().unwrap();
            fs::write(path.join("foreign-sentinel"), b"foreign synthetic sentinel").unwrap();
            Self {
                fixture,
                marker,
                file,
                sid,
                cache,
                owner,
                copy,
            }
        }
        fn check(&self, marker: &File) -> Result<(), MarkerError> {
            self.marker
                .check_actual(&self.sid, [&self.cache, &self.owner, &self.copy, marker])
        }
        fn sentinel(&self) {
            assert_eq!(
                fs::read(self.fixture.owner_path().join("foreign-sentinel")).unwrap(),
                b"foreign synthetic sentinel"
            );
        }
    }
    #[test]
    fn actual_native_roundtrip_and_payload_rejection() {
        let f = NativeMarker::create();
        f.check(&f.file).unwrap();
        assert_eq!(
            read_marker(&f.file).unwrap().encode().unwrap(),
            f.marker.encode().unwrap()
        );
        let mut value: serde_json::Value =
            serde_json::from_slice(&f.marker.encode().unwrap()).unwrap();
        value["unexpected"] = true.into();
        assert!(Marker::decode(&serde_json::to_vec(&value).unwrap()).is_err());
        assert!(Marker::decode(&vec![b' '; MARKER_LIMIT + 1]).is_err());
        let mut bad = f.marker.clone();
        bad.session_nonce = [9; 16];
        assert!(bad.validate().is_err());
        bad = f.marker.clone();
        bad.originals.receipts.swap(0, 1);
        assert!(bad.validate().is_err());
        bad = f.marker.clone();
        bad.marker_id = bad.originals.copy;
        bad.payload_digest = bad.digest();
        assert!(bad.validate().is_err());
        f.sentinel();
    }
    #[test]
    fn native_replacement_is_rejected_and_original_preserved() {
        let f = NativeMarker::create();
        let path = f.fixture.owner_path().join(MARKER_NAME);
        let bytes = fs::read(&path).unwrap();
        let original = f.fixture.owner_path().join("original-marker-sentinel");
        fs::rename(&path, &original).unwrap();
        let descriptor = Descriptor::private(&f.sid).unwrap();
        let mut replacement = open(
            &path,
            false,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            SHARE_ALL,
            Some(&descriptor),
        )
        .unwrap();
        replacement.write_all(&bytes).unwrap();
        replacement.sync_all().unwrap();
        assert!(f.check(&replacement).is_err());
        assert_eq!(fs::read(original).unwrap(), bytes);
        f.sentinel();
    }
    #[test]
    fn native_acl_widening_rejected_without_deletion() {
        let f = NativeMarker::create();
        let before = f.marker.encode().unwrap();
        // Owned fixture marker only; NULL unprotected DACL is deliberately unsafe.
        let code = unsafe {
            SetSecurityInfo(
                f.file.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | UNPROTECTED_DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(code, 0);
        assert!(f.check(&f.file).is_err());
        assert_eq!(
            fs::read(f.fixture.owner_path().join(MARKER_NAME)).unwrap(),
            before
        );
        f.sentinel();
    }
    #[test]
    fn actual_marker_seed_reopens_original_journal_without_receipt_authority() {
        let mut f = NativeMarker::create();
        let (originals, nonce, digest) = read_marker(&f.file).unwrap().journal_seed();
        assert!(f.fixture.reopen_seed(originals, nonce, digest));
        assert!(!f.fixture.all_receipts_match());
        assert!(f.fixture.receipt_bytes(0).is_empty());
        assert!(f.fixture.receipt_bytes(1).is_empty());
        let mut foreign = digest;
        foreign[0] ^= 1;
        assert!(!f.fixture.reopen_seed(originals, nonce, foreign));
        f.sentinel();
    }
}
