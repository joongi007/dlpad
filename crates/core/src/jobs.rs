//! Pure state policy; the adapter verifies final files before supplying evidence.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Preparing,
    Downloading,
    PostProcessing,
    Verifying,
    Downloaded,
    Stopping,
    Paused,
    Cancelled,
    Failed,
    Interrupted,
    RetryWaiting,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub number: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedArtifact {
    pub path: PathBuf,
    pub bytes: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub state: JobState,
    pub revision: u64,
    pub attempts: Vec<Attempt>,
    pub artifacts: Vec<VerifiedArtifact>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    InvalidTransition,
    MissingArtifact,
    InvalidArtifact,
    DuplicateAttempt,
}
impl Job {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            state: JobState::Queued,
            revision: 0,
            attempts: vec![],
            artifacts: vec![],
        }
    }
    pub fn transition(&mut self, next: JobState) -> Result<(), TransitionError> {
        use JobState::*;
        if next == Downloaded {
            return Err(TransitionError::MissingArtifact);
        }
        let allowed = matches!(
            (self.state, next),
            (Queued, Preparing | Cancelled)
                | (Preparing, Downloading)
                | (Downloading, PostProcessing | Verifying)
                | (PostProcessing, Verifying)
                | (
                    Preparing | Downloading | PostProcessing | Verifying,
                    Stopping | Failed | Interrupted
                )
                | (Stopping, Paused | Cancelled | Failed | Interrupted)
                | (Failed, RetryWaiting)
                | (RetryWaiting, Preparing | Cancelled)
                | (Paused | Failed | Interrupted, Preparing | Cancelled)
        );
        if !allowed {
            return Err(TransitionError::InvalidTransition);
        }
        self.state = next;
        self.revision += 1;
        Ok(())
    }
    pub fn begin_attempt(&mut self, id: impl Into<String>) -> Result<(), TransitionError> {
        let id = id.into();
        if id.is_empty() || self.attempts.iter().any(|a| a.id == id) {
            return Err(TransitionError::DuplicateAttempt);
        }
        self.transition(JobState::Preparing)?;
        self.attempts.push(Attempt {
            id,
            number: self.attempts.len() as u32 + 1,
        });
        Ok(())
    }
    pub fn complete(&mut self, artifact: VerifiedArtifact) -> Result<(), TransitionError> {
        self.complete_many(vec![artifact])
    }
    /// The adapter must verify every final file before supplying this bounded batch.
    /// Validation precedes all mutation, including revision and terminal state.
    pub fn complete_many(
        &mut self,
        artifacts: Vec<VerifiedArtifact>,
    ) -> Result<(), TransitionError> {
        if self.state != JobState::Verifying {
            return Err(TransitionError::InvalidTransition);
        }
        if artifacts.is_empty() {
            return Err(TransitionError::MissingArtifact);
        }
        if artifacts.len() > 3
            || !self.artifacts.is_empty()
            || artifacts.iter().enumerate().any(|(index, artifact)| {
                artifact.bytes == 0
                    || artifact.path.as_os_str().is_empty()
                    || artifacts[..index]
                        .iter()
                        .any(|previous| previous.path == artifact.path)
            })
        {
            return Err(TransitionError::InvalidArtifact);
        }
        self.artifacts = artifacts;
        self.state = JobState::Downloaded;
        self.revision += 1;
        Ok(())
    }
    pub fn recover_interrupted(&mut self) -> bool {
        if matches!(
            self.state,
            JobState::Preparing
                | JobState::Downloading
                | JobState::PostProcessing
                | JobState::Verifying
                | JobState::Stopping
        ) {
            self.state = JobState::Interrupted;
            self.revision += 1;
            true
        } else {
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_completion_only() {
        let mut j = Job::new("j");
        assert_eq!(
            j.transition(JobState::Downloaded),
            Err(TransitionError::MissingArtifact)
        );
        j.begin_attempt("a").unwrap();
        j.transition(JobState::Downloading).unwrap();
        j.transition(JobState::Verifying).unwrap();
        assert_eq!(
            j.complete(VerifiedArtifact {
                path: "x".into(),
                bytes: 0
            }),
            Err(TransitionError::InvalidArtifact)
        );
        j.complete(VerifiedArtifact {
            path: "x".into(),
            bytes: 1,
        })
        .unwrap();
        assert_eq!(j.state, JobState::Downloaded);
    }
    #[test]
    fn artifact_batch_is_bounded_and_rejected_without_partial_mutation() {
        let mut job = Job::new("batch");
        job.begin_attempt("attempt").unwrap();
        job.transition(JobState::Downloading).unwrap();
        job.transition(JobState::Verifying).unwrap();
        let artifact = |path: &str, bytes| VerifiedArtifact {
            path: path.into(),
            bytes,
        };
        let revision = job.revision;
        for invalid in [
            vec![],
            vec![artifact("video.mp4", 1), artifact("ko.vtt", 0)],
            vec![artifact("video.mp4", 1), artifact("video.mp4", 2)],
            vec![artifact("video.mp4", 1), artifact("", 2)],
            vec![
                artifact("video.mp4", 1),
                artifact("ko.vtt", 2),
                artifact("en.vtt", 3),
                artifact("extra", 4),
            ],
        ] {
            assert!(job.complete_many(invalid).is_err());
            assert_eq!(job.state, JobState::Verifying);
            assert_eq!(job.revision, revision);
            assert!(job.artifacts.is_empty());
        }
        job.complete_many(vec![
            artifact("video.mp4", 1),
            artifact("ko.vtt", 2),
            artifact("en.vtt", 3),
        ])
        .unwrap();
        assert_eq!(job.state, JobState::Downloaded);
        assert_eq!(job.revision, revision + 1);
        assert_eq!(job.artifacts.len(), 3);
        assert_eq!(
            job.complete_many(vec![artifact("other", 1)]),
            Err(TransitionError::InvalidTransition)
        );
    }
    #[test]
    fn retry_preserves_id() {
        let mut j = Job::new("stable");
        j.begin_attempt("a").unwrap();
        j.transition(JobState::Failed).unwrap();
        assert_eq!(j.begin_attempt("a"), Err(TransitionError::DuplicateAttempt));
        j.begin_attempt("b").unwrap();
        assert_eq!(j.id, "stable");
        assert_eq!(j.attempts[1].number, 2);
        assert!(j.recover_interrupted());
        assert!(!j.recover_interrupted());
    }
    #[test]
    fn stop_requires_ack() {
        let mut j = Job::new("j");
        j.begin_attempt("a").unwrap();
        assert!(j.transition(JobState::Paused).is_err());
        j.transition(JobState::Stopping).unwrap();
        j.transition(JobState::Paused).unwrap();
        assert!(!j.recover_interrupted());
    }
}
