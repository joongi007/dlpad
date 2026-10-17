//! Explicit create-only package export, not installation or automatic activation.
use super::{Error, VerifiedCandidate};
use crate::{created_relative_file::PublishedRelativeFile, file_identity::NativeFileIdentity};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::Instant,
};

/// The parent must be an existing caller-selected directory and its canonical native path.
/// Nonce is an ASCII alphanumeric/hyphen label (1..=64), not a path or an adoption hint.
pub struct StageRequest<'a> {
    pub parent: &'a File,
    pub parent_path: &'a Path,
    pub nonce: &'a str,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Creation {
    NotAttempted,
    AttemptedUnconfirmed,
    OwnedCreated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Publication {
    NotAttempted,
    Unconfirmed,
    PublishedUnverified,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cleanup {
    NotNeeded,
    OriginalRemoved,
    OriginalRetained,
    Unconfirmed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageFailureKind {
    Validation(Error),
    Io(io::ErrorKind),
}

/// Observed failure facts only. Paths and native identity never restore cleanup authority.
#[derive(Debug)]
pub struct StageFailure {
    pub kind: StageFailureKind,
    pub creation: Creation,
    pub publication: Publication,
    pub cleanup: Cleanup,
    pub stage_path: Option<PathBuf>,
    pub target_path: Option<PathBuf>,
    pub created_identity: Option<NativeFileIdentity>,
}
impl std::fmt::Display for StageFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "app update package export failed ({:?}); publication {:?}",
            self.kind, self.publication
        )
    }
}
impl std::error::Error for StageFailure {}

/// Retains the original published read-only reader and its parent identity pin.
/// No constructor from path, clone, deserialize, deletion or executable launch API exists.
#[derive(Debug)]
pub struct StagedPackage {
    published: PublishedRelativeFile,
    parent_path: PathBuf,
    version: String,
    sequence: u64,
    bytes: u64,
    sha256: [u8; 32],
}
impl StagedPackage {
    pub fn path(&self) -> &Path {
        self.published.path()
    }
    pub fn identity(&self) -> &NativeFileIdentity {
        self.published.identity()
    }
    pub fn reader_mut(&mut self) -> &mut File {
        self.published.reader_mut()
    }
    pub fn verify_current(&self) -> io::Result<()> {
        self.published.verify_current(&self.parent_path)
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
    pub fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }
}

/// All I/O belongs on a worker. Blocking reads require transport-owned timeouts.
/// A post-publication cancellation is a retained published file, never path cleanup.
pub fn stage_verified_package(
    candidate: &VerifiedCandidate,
    source: &mut impl Read,
    request: StageRequest<'_>,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<StagedPackage, StageFailure> {
    #[cfg(windows)]
    {
        windows::stage(candidate, source, request, cancel, deadline)
    }
    #[cfg(not(windows))]
    {
        let _ = (candidate, source, request, cancel, deadline);
        Err(StageFailure {
            kind: StageFailureKind::Io(io::ErrorKind::Unsupported),
            creation: Creation::NotAttempted,
            publication: Publication::NotAttempted,
            cleanup: Cleanup::NotNeeded,
            stage_path: None,
            target_path: None,
            created_identity: None,
        })
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use crate::created_relative_file::{CleanupOutcome, CreatedRelativeFile, PublishError};
    use std::{
        io::{Seek, SeekFrom, Write},
        sync::atomic::Ordering,
    };
    fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> Result<(), StageFailureKind> {
        if cancel.load(Ordering::Acquire) {
            return Err(StageFailureKind::Validation(Error::Cancelled));
        }
        if Instant::now() >= deadline {
            return Err(StageFailureKind::Validation(Error::TimedOut));
        }
        Ok(())
    }
    fn io(error: io::Error) -> StageFailureKind {
        StageFailureKind::Io(error.kind())
    }
    fn cleanup(created: CreatedRelativeFile) -> Cleanup {
        match created.cancel_cleanup() {
            CleanupOutcome::RemovedOriginal => Cleanup::OriginalRemoved,
            CleanupOutcome::RetainedOriginal => Cleanup::OriginalRetained,
            CleanupOutcome::Unconfirmed => Cleanup::Unconfirmed,
        }
    }
    pub(super) fn stage(
        candidate: &VerifiedCandidate,
        source: &mut impl Read,
        request: StageRequest<'_>,
        cancel: &AtomicBool,
        deadline: Instant,
    ) -> Result<StagedPackage, StageFailure> {
        let mut facts = StageFailure {
            kind: StageFailureKind::Io(io::ErrorKind::Other),
            creation: Creation::NotAttempted,
            publication: Publication::NotAttempted,
            cleanup: Cleanup::NotNeeded,
            stage_path: None,
            target_path: None,
            created_identity: None,
        };
        let prepared = (|| -> Result<CreatedRelativeFile, StageFailureKind> {
            checkpoint(cancel, deadline)?;
            if request.nonce.is_empty()
                || request.nonce.len() > 64
                || !request
                    .nonce
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                || crate::file_identity::final_path(request.parent).map_err(io)?
                    != request.parent_path
            {
                return Err(StageFailureKind::Io(io::ErrorKind::InvalidInput));
            }
            let leaf = format!(".dlpad-update-{}.partial", request.nonce);
            facts.stage_path = Some(request.parent_path.join(&leaf));
            facts.target_path = Some(request.parent_path.join(candidate.asset_name()));
            // Factory can create then fail post-create validation; never claim no mutation.
            facts.creation = Creation::AttemptedUnconfirmed;
            facts.cleanup = Cleanup::Unconfirmed;
            let created = CreatedRelativeFile::create(request.parent, &leaf).map_err(io)?;
            facts.creation = Creation::OwnedCreated;
            facts.cleanup = Cleanup::OriginalRetained;
            facts.created_identity = Some(created.identity().clone());
            Ok(created)
        })();
        let mut created = match prepared {
            Ok(c) => c,
            Err(kind) => {
                facts.kind = kind;
                return Err(facts);
            }
        };
        let written = (|| -> Result<(), StageFailureKind> {
            created.verify_current(request.parent_path).map_err(io)?;
            let mut buffer = [0u8; 64 * 1024];
            let mut total = 0u64;
            loop {
                checkpoint(cancel, deadline)?;
                let remaining = candidate.asset_bytes() - total;
                let count_limit = usize::try_from(remaining.saturating_add(1))
                    .unwrap_or(buffer.len())
                    .min(buffer.len());
                let count = source.read(&mut buffer[..count_limit]).map_err(io)?;
                checkpoint(cancel, deadline)?;
                if count == 0 {
                    break;
                }
                total = total
                    .checked_add(count as u64)
                    .ok_or(StageFailureKind::Validation(Error::PackageSize))?;
                if total > candidate.asset_bytes() {
                    return Err(StageFailureKind::Validation(Error::PackageSize));
                }
                created.file_mut().write_all(&buffer[..count]).map_err(io)?;
                checkpoint(cancel, deadline)?;
            }
            if total != candidate.asset_bytes() {
                return Err(StageFailureKind::Validation(Error::PackageSize));
            }
            created.file_mut().flush().map_err(io)?;
            checkpoint(cancel, deadline)?;
            created.file_mut().sync_all().map_err(io)?;
            checkpoint(cancel, deadline)?;
            created.verify_current(request.parent_path).map_err(io)?;
            created.file_mut().seek(SeekFrom::Start(0)).map_err(io)?;
            candidate
                .verify_package(created.file_mut(), cancel, deadline)
                .map_err(StageFailureKind::Validation)?;
            created.verify_current(request.parent_path).map_err(io)?;
            checkpoint(cancel, deadline)
        })();
        if let Err(kind) = written {
            facts.kind = kind;
            facts.cleanup = cleanup(created);
            return Err(facts);
        }
        let mut published = match created.publish_no_replace(candidate.asset_name()) {
            Ok(p) => p,
            Err(PublishError::BeforePublish { created, error }) => {
                facts.kind = io(error);
                facts.cleanup = cleanup(created);
                return Err(facts);
            }
            Err(PublishError::Unconfirmed {
                source_path,
                target_path,
                identity,
                error,
            }) => {
                facts.kind = io(error);
                facts.publication = Publication::Unconfirmed;
                facts.cleanup = Cleanup::Unconfirmed;
                facts.stage_path = Some(source_path);
                facts.target_path = Some(target_path);
                facts.created_identity = Some(identity);
                return Err(facts);
            }
        };
        let verified = (|| -> Result<(), StageFailureKind> {
            checkpoint(cancel, deadline)?;
            published.verify_current(request.parent_path).map_err(io)?;
            published
                .reader_mut()
                .seek(SeekFrom::Start(0))
                .map_err(io)?;
            candidate
                .verify_package(published.reader_mut(), cancel, deadline)
                .map_err(StageFailureKind::Validation)?;
            published.verify_current(request.parent_path).map_err(io)?;
            checkpoint(cancel, deadline)?;
            published
                .reader_mut()
                .seek(SeekFrom::Start(0))
                .map_err(io)?;
            Ok(())
        })();
        if let Err(kind) = verified {
            facts.kind = kind;
            facts.publication = Publication::PublishedUnverified;
            facts.cleanup = Cleanup::OriginalRetained;
            return Err(facts);
        }
        Ok(StagedPackage {
            published,
            parent_path: request.parent_path.to_owned(),
            version: candidate.version().to_owned(),
            sequence: candidate.sequence(),
            bytes: candidate.asset_bytes(),
            sha256: *candidate.asset_sha256(),
        })
    }
}

#[cfg(test)]
mod tests;
