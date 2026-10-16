//! Explicit readonly consistency inspection; no repair, creation or historical authority.
use super::{ActiveSelectionInspection, inspect_active_selection, windows_caps::ReadRoot};
use std::{
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
fn namespace(root: &ReadRoot, c: &AtomicBool, d: Instant) -> io::Result<()> {
    check(c, d)?;
    root.verify()?;
    let mut seen = [false; 3];
    for entry in std::fs::read_dir(root.path())? {
        check(c, d)?;
        let entry = entry?;
        let kind = entry.file_type()?;
        let index = match entry.file_name().to_str() {
            Some("registered") => 0,
            Some("receipt") => 1,
            Some("active") => 2,
            _ => return Err(io::ErrorKind::InvalidData.into()),
        };
        if seen[index] || !kind.is_dir() || kind.is_symlink() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        seen[index] = true;
    }
    if seen != [true; 3] {
        return Err(io::ErrorKind::InvalidData.into());
    }
    root.verify()?;
    check(c, d)
}
pub struct UserBundleInspection {
    outer: ReadRoot,
    selection: ActiveSelectionInspection,
}
impl std::fmt::Debug for UserBundleInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UserBundleInspection(<redacted>)")
    }
}
impl UserBundleInspection {
    /// Build bounded immutable expectations while all original read pins remain live.
    pub(super) fn explicit_removal_metadata(
        &self,
        c: &AtomicBool,
        d: Instant,
    ) -> io::Result<super::bundle_removal::BundleRemovalMetadata> {
        use super::bundle_removal::{BundleRemovalMetadata, ExpectedFile};
        use sha2::{Digest, Sha256};
        self.verify_current(c, d)?;
        let pair = self.selection.removal_pair();
        let record = pair.registered().record();
        let seal = pair.registered().seal();
        let receipt = pair.receipt().record();
        let active = self.selection.record();
        let mut files: Vec<ExpectedFile> = record
            .prepared
            .entries
            .iter()
            .map(|entry| ExpectedFile {
                root: 1,
                name: entry.name.clone(),
                identity: entry.identity.clone(),
                bytes: entry.bytes,
                sha256: entry.sha256.clone(),
            })
            .collect();
        let encoded = [
            (
                1,
                "install-ready.json",
                &record.ready_identity,
                serde_json::to_vec(&record.prepared)
                    .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?,
            ),
            (
                1,
                "install-registration.json",
                &record.record_identity,
                super::registration_schema::serialize_record(record)?,
            ),
            (
                1,
                "install-registration-seal.json",
                &record.seal_identity,
                super::registration_schema::serialize_seal(seal)?,
            ),
            (
                2,
                "registration-receipt.json",
                &receipt.receipt_identity,
                super::receipt_schema::serialize_receipt(receipt)?,
            ),
            (
                3,
                "active-selection.json",
                &active.marker_identity,
                active.canonical_bytes()?,
            ),
        ];
        for (root, name, identity, bytes) in encoded {
            check(c, d)?;
            if bytes.is_empty() || bytes.len() > 65536 {
                return Err(io::ErrorKind::InvalidData.into());
            }
            files.push(ExpectedFile {
                root,
                name: name.into(),
                identity: identity.clone(),
                bytes: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&bytes)),
            });
        }
        self.verify_current(c, d)?;
        Ok(BundleRemovalMetadata {
            root_identities: vec![
                self.outer.identity().clone(),
                record.root_identity.clone(),
                receipt.receipt_root_identity.clone(),
                active.active_root_identity.clone(),
            ],
            files,
        })
    }
    /// Move original roots only after metadata validation; release deny-DELETE child shares.
    pub(super) fn into_explicit_removal_roots(self) -> [ReadRoot; 4] {
        let Self { outer, selection } = self;
        let (active, _, pair) = selection.into_bundle_removal_inputs();
        let (registered, receipt) = pair.into_parts();
        let (registered, _, _) = registered.into_removal_inputs();
        let (receipt, _) = receipt.into_bundle_removal_inputs();
        [outer, registered, receipt, active]
    }
    pub fn path(&self) -> &Path {
        self.outer.path()
    }
    pub fn selection(&self) -> &ActiveSelectionInspection {
        &self.selection
    }
    pub fn verify_current(&self, c: &AtomicBool, d: Instant) -> io::Result<()> {
        namespace(&self.outer, c, d)?;
        self.selection.verify_current(c, d)?;
        namespace(&self.outer, c, d)
    }
}
pub fn inspect_user_bundle(
    path: &Path,
    c: &AtomicBool,
    d: Instant,
) -> io::Result<UserBundleInspection> {
    check(c, d)?;
    let outer = ReadRoot::open_existing(path)?;
    namespace(&outer, c, d)?;
    let selection = inspect_active_selection(
        &outer.path().join("active"),
        &outer.path().join("registered"),
        &outer.path().join("receipt"),
        c,
        d,
    )?;
    let value = UserBundleInspection { outer, selection };
    value.verify_current(c, d)?;
    Ok(value)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancelled_and_expired_missing_bundle_are_not_created() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("absent");
        assert_eq!(
            inspect_user_bundle(
                &path,
                &AtomicBool::new(true),
                Instant::now() + std::time::Duration::from_secs(1)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::Interrupted
        );
        assert_eq!(
            inspect_user_bundle(&path, &AtomicBool::new(false), Instant::now())
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert!(!path.exists());
    }
}
