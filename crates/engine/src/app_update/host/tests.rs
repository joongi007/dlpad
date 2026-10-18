use super::*;
#[test]
fn one_worker_and_current_generation_only() {
    let mut state = Generations::default();
    let first = state.begin().unwrap();
    assert!(state.begin().is_err());
    assert!(state.restart(first).is_err());
    assert!(state.complete(first + 1, true).is_err());
    state.complete(first, true).unwrap();
    state.restart(first).unwrap();
    assert!(state.restart(first + 1).is_err());
}
#[test]
fn failed_or_superseded_candidate_cannot_restart() {
    let mut state = Generations::default();
    let first = state.begin().unwrap();
    state.complete(first, true).unwrap();
    let second = state.begin().unwrap();
    assert!(state.restart(first).is_err());
    state.complete(second, false).unwrap();
    assert!(state.restart(second).is_err());
}
#[test]
fn generation_overflow_does_not_start_worker() {
    let mut state = Generations {
        counter: u64::MAX,
        ..Default::default()
    };
    assert!(state.begin().is_err());
    assert!(state.worker.is_none());
}
#[cfg(not(windows))]
#[test]
fn unsupported_host_never_starts_a_ui_or_network() {
    assert_eq!(
        run(Path::new("/unused"), &[], "", "0.1.1")
            .unwrap_err()
            .kind(),
        io::ErrorKind::Unsupported
    );
}

#[test]
fn busy_rejection_waits_for_original_terminal_then_uses_new_generation() {
    let mut state = Generations::default();
    let active = state.begin().unwrap();
    assert!(state.begin().is_err());
    state.complete(active, false).unwrap();
    let rejection = state.begin().unwrap();
    state.complete(rejection, false).unwrap();
    assert!(rejection > active);
    assert!(state.restart(active).is_err());
    assert!(state.restart(rejection).is_err());
    assert!(state.begin().unwrap() > rejection);
}

#[cfg(windows)]
#[test]
fn staged_reader_cancel_and_timeout_keep_semantic_failure_codes() {
    use super::windows::transfer_kind;
    use crate::app_update::{StageFailureKind, control::FailureCode, download::FailureKind};
    assert_eq!(
        transfer_kind(&FailureKind::Stage(StageFailureKind::Io(
            std::io::ErrorKind::Interrupted
        ))),
        FailureCode::Cancelled
    );
    assert_eq!(
        transfer_kind(&FailureKind::Stage(StageFailureKind::Io(
            std::io::ErrorKind::TimedOut
        ))),
        FailureCode::Network
    );
    assert_eq!(
        transfer_kind(&FailureKind::Stage(StageFailureKind::Io(
            std::io::ErrorKind::PermissionDenied
        ))),
        FailureCode::Disk
    );
}
