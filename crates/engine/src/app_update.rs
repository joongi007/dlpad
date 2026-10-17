//! Signed app-update candidates. These values grant no install, execution or deletion authority.
//! Network transfer and installation remain separate worker-owned operations.
mod metadata;
pub use metadata::{
    Channel, Error, MAX_METADATA_BYTES, MAX_PACKAGE_BYTES, Policy, TrustedKey, VerifiedCandidate,
    VerifiedPackage, verify_metadata,
};
mod staging;
pub use staging::{
    Cleanup, Creation, Publication, StageFailure, StageFailureKind, StageRequest, StagedPackage,
    stage_verified_package,
};

pub mod discovery;
pub mod transport;

pub mod bundle;
