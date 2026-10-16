//! Explicit two-path readonly consistency, no discovery, mutation or deletion authority.
use super::{
    ReceiptInspection, RegisteredInspection, inspect_registered, inspect_registration_receipt,
};
use std::{
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
fn bad() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "registration receipt pair mismatch",
    )
}
fn check(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "pair inspection cancelled",
        ))
    } else if Instant::now() >= deadline {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "pair inspection deadline",
        ))
    } else {
        Ok(())
    }
}
fn matching(registered: &RegisteredInspection, receipt: &ReceiptInspection) -> io::Result<()> {
    let r = receipt.record();
    if serde_json::to_vec(registered.record()).map_err(|_| bad())?
        != serde_json::to_vec(&r.registration).map_err(|_| bad())?
        || serde_json::to_vec(registered.seal()).map_err(|_| bad())?
            != serde_json::to_vec(&r.seal).map_err(|_| bad())?
    {
        return Err(bad());
    }
    Ok(())
}
pub struct RegisteredPairInspection {
    registered: RegisteredInspection,
    receipt: ReceiptInspection,
}
impl RegisteredPairInspection {
    /// Private input transfer for a separate explicit removal request.
    pub(super) fn into_parts(self) -> (RegisteredInspection, ReceiptInspection) {
        (self.registered, self.receipt)
    }
    pub fn registered(&self) -> &RegisteredInspection {
        &self.registered
    }
    pub fn receipt(&self) -> &ReceiptInspection {
        &self.receipt
    }
    pub fn verify_current(&self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
        check(cancel, deadline)?;
        matching(&self.registered, &self.receipt)?;
        self.registered.verify_current(cancel, deadline)?;
        check(cancel, deadline)?;
        self.receipt.verify_current(cancel, deadline)?;
        check(cancel, deadline)?;
        matching(&self.registered, &self.receipt)?;
        // These are sequential observations, not an atomic filesystem snapshot.
        check(cancel, deadline)
    }
}
impl std::fmt::Debug for RegisteredPairInspection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredPairInspection")
            .finish_non_exhaustive()
    }
}
pub fn inspect_registered_with_receipt(
    registration: &Path,
    receipt: &Path,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<RegisteredPairInspection> {
    check(cancel, deadline)?;
    let registered = inspect_registered(registration, cancel, deadline)?;
    check(cancel, deadline)?;
    let receipt = inspect_registration_receipt(receipt, cancel, deadline)?;
    let pair = RegisteredPairInspection {
        registered,
        receipt,
    };
    pair.verify_current(cancel, deadline)?;
    Ok(pair)
}

/// Only accepts retained original fresh capabilities; persisted paths/records cannot call this route.
pub(super) fn inspect_fresh_registered_pair(
    registration: &super::FreshRegistration,
    receipt: &super::FreshRegistrationReceipt,
    cancel: &AtomicBool,
    deadline: Instant,
) -> io::Result<RegisteredPairInspection> {
    check(cancel, deadline)?;
    registration.verify_current(cancel, deadline)?;
    receipt.verify_current(cancel, deadline)?;
    let registered =
        super::registration_reopen::inspect_fresh_registration(registration, cancel, deadline)?;
    let receipt_observer = super::receipt_reopen::inspect_fresh_receipt(receipt, cancel, deadline)?;
    let pair = RegisteredPairInspection {
        registered,
        receipt: receipt_observer,
    };
    pair.verify_current(cancel, deadline)?;
    registration.verify_current(cancel, deadline)?;
    receipt.verify_current(cancel, deadline)?;
    pair.verify_current(cancel, deadline)?;
    Ok(pair)
}
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::{fs, fs::File, time::Duration};
    fn later() -> Instant {
        Instant::now() + Duration::from_secs(30)
    }
    fn produced() -> (
        tempfile::TempDir,
        std::path::PathBuf,
        std::path::PathBuf,
        Vec<u8>,
    ) {
        let d = tempfile::tempdir().unwrap();
        let mut b = vec![0; 256];
        b[..2].copy_from_slice(b"MZ");
        b[60..64].copy_from_slice(&64u32.to_le_bytes());
        b[64..68].copy_from_slice(b"PE\0\0");
        b[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        b[84..86].copy_from_slice(&112u16.to_le_bytes());
        b[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        fs::write(d.path().join("source"), &b).unwrap();
        fs::write(d.path().join("sentinel"), b"outside").unwrap();
        let f = File::open(d.path().join("source")).unwrap();
        let m = super::super::manifest::BundleManifest {
            schema: 1,
            version: "0.1.0".into(),
            target: "x86_64-pc-windows-gnu".into(),
            development_only: true,
            entries: vec![super::super::manifest::BundleEntry {
                name: "dlpad.exe".into(),
                bytes: 256,
                sha256: format!("{:x}", Sha256::digest(&b)),
            }],
        };
        let cancel = AtomicBool::new(false);
        let reg = d.path().join("registered");
        let p =
            super::super::prepare_new(&m, &[("dlpad.exe", &f)], &reg, &cancel, later()).unwrap();
        let r = super::super::register_fresh(p, &cancel, later()).unwrap();
        let receipt = d.path().join("receipt");
        let proof =
            super::super::create_registration_receipt(&r, &receipt, &cancel, later()).unwrap();
        let raw = serde_json::to_vec(proof.record()).unwrap();
        drop(proof);
        drop(r);
        (d, reg, receipt, raw)
    }
    #[test]
    fn actual_producers_drop_then_pair_keeps_readonly_caps() {
        let (d, reg, receipt, raw) = produced();
        let pair =
            inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(false), later())
                .unwrap();
        pair.verify_current(&AtomicBool::new(false), later())
            .unwrap();
        assert_eq!(pair.registered().path(), reg);
        assert_eq!(pair.receipt().path(), receipt);
        drop(pair);
        assert_eq!(
            fs::read(receipt.join("registration-receipt.json")).unwrap(),
            raw
        );
        assert_eq!(fs::read(d.path().join("sentinel")).unwrap(), b"outside");
    }
    #[test]
    fn independently_valid_other_receipt_cannot_pair() {
        let (_a, reg, receipt, raw) = produced();
        let (_b, otherreg, otherreceipt, otherraw) = produced();
        inspect_registered(&reg, &AtomicBool::new(false), later()).unwrap();
        inspect_registration_receipt(&otherreceipt, &AtomicBool::new(false), later()).unwrap();
        assert_eq!(
            inspect_registered_with_receipt(&reg, &otherreceipt, &AtomicBool::new(false), later())
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            fs::read(receipt.join("registration-receipt.json")).unwrap(),
            raw
        );
        assert_eq!(
            fs::read(otherreceipt.join("registration-receipt.json")).unwrap(),
            otherraw
        );
        inspect_registered_with_receipt(&otherreg, &otherreceipt, &AtomicBool::new(false), later())
            .unwrap();
    }
    #[test]
    fn cancelled_expired_and_foreign_preserve_both_roots() {
        let (d, reg, receipt, raw) = produced();
        for (c, end, kind) in [
            (true, later(), io::ErrorKind::Interrupted),
            (false, Instant::now(), io::ErrorKind::TimedOut),
        ] {
            assert_eq!(
                inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(c), end)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        let pair =
            inspect_registered_with_receipt(&reg, &receipt, &AtomicBool::new(false), later())
                .unwrap();
        fs::write(receipt.join("foreign"), b"foreign").unwrap();
        assert!(
            pair.verify_current(&AtomicBool::new(false), later())
                .is_err()
        );
        assert_eq!(
            fs::read(receipt.join("registration-receipt.json")).unwrap(),
            raw
        );
        assert_eq!(fs::read(receipt.join("foreign")).unwrap(), b"foreign");
        assert_eq!(fs::read(d.path().join("sentinel")).unwrap(), b"outside");
    }
}
