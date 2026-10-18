//! Worker-only signed download and separate-install preparation. No activation authority.
#[cfg(any(windows, test))]
use std::{
    io::{self, Read},
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Verifying,
    Downloading,
    Staging,
    Preparing,
    Prepared,
}
/// Latest observation only. Callbacks must replace a bounded mailbox, never queue samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub phase: Phase,
    pub received: u64,
    pub total: u64,
}
#[cfg(any(windows, test))]
fn checkpoint(cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    if Instant::now() >= deadline {
        return Err(io::ErrorKind::TimedOut.into());
    }
    Ok(())
}
#[cfg(any(windows, test))]
struct CountingReader<'a, R: Read> {
    source: R,
    cancel: &'a AtomicBool,
    deadline: Instant,
    received: u64,
    total: u64,
    progress: &'a mut dyn FnMut(Progress),
}
#[cfg(any(windows, test))]
impl<R: Read> Read for CountingReader<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        checkpoint(self.cancel, self.deadline)?;
        if self.received > self.total {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let remaining = self.total.saturating_sub(self.received).saturating_add(1);
        let limit = output
            .len()
            .min(65536)
            .min(usize::try_from(remaining).unwrap_or(usize::MAX));
        let count = self.source.read(&mut output[..limit]);
        checkpoint(self.cancel, self.deadline)?;
        let count = count?;
        self.received = self
            .received
            .checked_add(count as u64)
            .ok_or(io::ErrorKind::InvalidData)?;
        if self.received > self.total {
            return Err(io::ErrorKind::InvalidData.into());
        }
        if count > 0 {
            (self.progress)(Progress {
                phase: Phase::Downloading,
                received: self.received,
                total: self.total,
            });
        }
        checkpoint(self.cancel, self.deadline)?;
        Ok(count)
    }
}
#[cfg(windows)]
mod windows {
    use super::*;
    use crate::{
        app_install::{
            PreparedUpdateBundle, UpdatePreparationError, UpdatePreparationFailure,
            UpdatePreparationRequest, prepare_staged_update,
        },
        app_update::{
            self, Policy, StageFailure, StageFailureKind, StageRequest, StagedPackage, TrustedKey,
            VerifiedCandidate,
            transport::{self, Transport},
        },
    };
    use std::sync::Arc;
    pub struct DownloadRequest<'a> {
        pub version: &'a str,
        pub key_id_hint: &'a str,
        pub keys: &'a [TrustedKey<'a>],
        pub policy: Policy<'a>,
        pub stage: StageRequest<'a>,
        pub prepare: UpdatePreparationRequest<'a>,
    }
    pub struct PreparedDownload {
        candidate: VerifiedCandidate,
        package: StagedPackage,
        bundle: PreparedUpdateBundle,
    }
    impl PreparedDownload {
        pub fn candidate(&self) -> &VerifiedCandidate {
            &self.candidate
        }
        pub fn package(&self) -> &StagedPackage {
            &self.package
        }
        pub fn bundle(&self) -> &PreparedUpdateBundle {
            &self.bundle
        }
        /// Recheck original signed ZIP bytes and newly prepared capabilities, not paths alone.
        /// Current trust/time/sequence policy must still be rechecked by the activation owner.
        pub fn verify_current(&mut self, cancel: &AtomicBool, deadline: Instant) -> io::Result<()> {
            checkpoint(cancel, deadline)?;
            self.package.verify_current()?;
            app_update::bundle::inspect_staged_bundle(&mut self.package, cancel, deadline)
                .map_err(io::Error::other)?;
            self.bundle.verify_current(cancel, deadline)?;
            self.package.verify_current()?;
            checkpoint(cancel, deadline)
        }
    }
    #[derive(Debug)]
    pub enum FailureKind {
        Io(io::ErrorKind),
        Transport(transport::Error),
        Stage(StageFailureKind),
        Preparation(UpdatePreparationError),
    }
    struct FailureState {
        kind: FailureKind,
        candidate: Option<VerifiedCandidate>,
        package: Option<StagedPackage>,
        stage: Option<StageFailure>,
        preparation: Option<UpdatePreparationFailure>,
        bundle: Option<PreparedUpdateBundle>,
    }
    /// Failure retains original capabilities. Dropping it never deletes any file or installation.
    pub struct DownloadFailure(Box<FailureState>);
    impl DownloadFailure {
        pub fn kind(&self) -> &FailureKind {
            &self.0.kind
        }
        pub fn candidate(&self) -> Option<&VerifiedCandidate> {
            self.0.candidate.as_ref()
        }
        pub fn package(&self) -> Option<&StagedPackage> {
            self.0.package.as_ref()
        }
        pub fn stage_failure(&self) -> Option<&StageFailure> {
            self.0.stage.as_ref()
        }
        pub fn preparation_failure(&self) -> Option<&UpdatePreparationFailure> {
            self.0.preparation.as_ref()
        }
        pub fn bundle(&self) -> Option<&PreparedUpdateBundle> {
            self.0.bundle.as_ref()
        }
    }
    impl std::fmt::Debug for DownloadFailure {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DownloadFailure")
                .field("kind", self.kind())
                .finish_non_exhaustive()
        }
    }
    impl std::fmt::Display for DownloadFailure {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "app update download failed ({:?})", self.kind())
        }
    }
    impl std::error::Error for DownloadFailure {}
    /// Synchronous blocking worker operation. UI must own one worker and one latest mailbox.
    pub fn prepare_download(
        transport: &mut impl Transport,
        request: &DownloadRequest<'_>,
        cancel: Arc<AtomicBool>,
        deadline: Instant,
        progress: &mut impl FnMut(Progress),
    ) -> Result<PreparedDownload, DownloadFailure> {
        let mut state = Box::new(FailureState {
            kind: FailureKind::Io(io::ErrorKind::Other),
            candidate: None,
            package: None,
            stage: None,
            preparation: None,
            bundle: None,
        });
        let result = (|| -> Result<(), FailureKind> {
            checkpoint(&cancel, deadline).map_err(|e| FailureKind::Io(e.kind()))?;
            progress(Progress {
                phase: Phase::Verifying,
                received: 0,
                total: 0,
            });
            let candidate = transport::fetch_candidate(
                transport,
                request.version,
                request.key_id_hint,
                request.keys,
                &request.policy,
                &cancel,
                deadline,
            )
            .map_err(FailureKind::Transport)?;
            let total = candidate.asset_bytes();
            state.candidate = Some(candidate);
            checkpoint(&cancel, deadline).map_err(|e| FailureKind::Io(e.kind()))?;
            let source = transport::open_package(
                transport,
                state.candidate.as_ref().expect("retained candidate"),
                cancel.clone(),
                deadline,
            )
            .map_err(FailureKind::Transport)?;
            progress(Progress {
                phase: Phase::Staging,
                received: 0,
                total,
            });
            let mut reader = CountingReader {
                source,
                cancel: &cancel,
                deadline,
                received: 0,
                total,
                progress,
            };
            let package = app_update::stage_verified_package(
                state.candidate.as_ref().expect("retained candidate"),
                &mut reader,
                StageRequest {
                    parent: request.stage.parent,
                    parent_path: request.stage.parent_path,
                    nonce: request.stage.nonce,
                },
                &cancel,
                deadline,
            )
            .map_err(|error| {
                let kind = error.kind;
                state.stage = Some(error);
                FailureKind::Stage(kind)
            })?;
            state.package = Some(package);
            checkpoint(&cancel, deadline).map_err(|e| FailureKind::Io(e.kind()))?;
            progress(Progress {
                phase: Phase::Preparing,
                received: total,
                total,
            });
            state.bundle = Some(
                prepare_staged_update(
                    state.package.as_mut().expect("retained package"),
                    &request.prepare,
                    &cancel,
                    deadline,
                )
                .map_err(|error| {
                    let kind = error.kind();
                    state.preparation = Some(error);
                    FailureKind::Preparation(kind)
                })?,
            );
            state
                .bundle
                .as_ref()
                .expect("retained bundle")
                .verify_current(&cancel, deadline)
                .map_err(|e| FailureKind::Io(e.kind()))?;
            state
                .package
                .as_ref()
                .expect("retained package")
                .verify_current()
                .map_err(|e| FailureKind::Io(e.kind()))?;
            checkpoint(&cancel, deadline).map_err(|e| FailureKind::Io(e.kind()))?;
            progress(Progress {
                phase: Phase::Prepared,
                received: total,
                total,
            });
            checkpoint(&cancel, deadline).map_err(|e| FailureKind::Io(e.kind()))?;
            Ok(())
        })();
        if let Err(kind) = result {
            state.kind = kind;
            return Err(DownloadFailure(state));
        }
        Ok(PreparedDownload {
            candidate: state.candidate.take().expect("verified candidate"),
            package: state.package.take().expect("verified package"),
            bundle: state.bundle.take().expect("verified bundle"),
        })
    }
}
#[cfg(windows)]
pub use windows::{
    DownloadFailure, DownloadRequest, FailureKind, PreparedDownload, prepare_download,
};
#[cfg(test)]
mod tests;
