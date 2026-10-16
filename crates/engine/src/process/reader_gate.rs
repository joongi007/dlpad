//! Bounded gate exchange only. Callers own native identity and durable binding verification.
use super::reader_protocol::{
    self as protocol, Begin, Decoder, Domain, MAX_PACKET, ReaderIdentity,
};
use std::{
    fmt,
    io::{self, Read, Write},
    os::unix::net::UnixStream,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
const STEP: Duration = Duration::from_millis(20);
const HEADER: usize = 8;
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum GateError {
    Cancelled,
    Deadline,
    Ended,
    InvalidFrame,
    BindFailed,
    Io,
}
impl fmt::Debug for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieReaderGateError")
    }
}
impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cookie reader gate failed")
    }
}
impl std::error::Error for GateError {}
struct Budget<'a> {
    deadline: Instant,
    cancel: Option<&'a AtomicBool>,
}
impl Budget<'_> {
    fn remaining(&self) -> Result<Duration, GateError> {
        if self
            .cancel
            .is_some_and(|cancel| cancel.load(Ordering::Acquire))
        {
            return Err(GateError::Cancelled);
        }
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(GateError::Deadline);
        }
        Ok(remaining.min(STEP))
    }
}
fn exchange<T>(
    stream: &mut UnixStream,
    budget: &Budget<'_>,
    run: impl FnOnce(&mut UnixStream, &Budget<'_>) -> Result<T, GateError>,
) -> Result<T, GateError> {
    let read = stream.read_timeout().map_err(|_| GateError::Io)?;
    let write = stream.write_timeout().map_err(|_| GateError::Io)?;
    let result = run(stream, budget);
    let restored = stream
        .set_read_timeout(read)
        .and_then(|_| stream.set_write_timeout(write));
    match result {
        Err(error) => Err(error),
        Ok(value) => restored.map(|_| value).map_err(|_| GateError::Io),
    }
}
fn retry(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
    )
}
fn write_frame(
    stream: &mut UnixStream,
    bytes: &[u8],
    budget: &Budget<'_>,
) -> Result<(), GateError> {
    if bytes.len() > MAX_PACKET {
        return Err(GateError::InvalidFrame);
    }
    let mut offset = 0;
    while offset < bytes.len() {
        stream
            .set_write_timeout(Some(budget.remaining()?))
            .map_err(|_| GateError::Io)?;
        match stream.write(&bytes[offset..]) {
            Ok(0) => return Err(GateError::Ended),
            Ok(count) => offset += count,
            Err(error) if retry(&error) => {}
            Err(_) => return Err(GateError::Io),
        }
    }
    budget.remaining()?;
    Ok(())
}
fn read_frame(
    stream: &mut UnixStream,
    domain: Domain,
    budget: &Budget<'_>,
) -> Result<Vec<u8>, GateError> {
    let mut decoder = Decoder::new(domain);
    let mut bytes = [0u8; MAX_PACKET];
    let mut used = 0;
    let mut target = HEADER;
    loop {
        stream
            .set_read_timeout(Some(budget.remaining()?))
            .map_err(|_| GateError::Io)?;
        match stream.read(&mut bytes[used..target]) {
            Ok(0) => return Err(GateError::Ended),
            Ok(count) => {
                let start = used;
                used += count;
                let complete = decoder
                    .push(&bytes[start..used])
                    .map_err(|_| GateError::InvalidFrame)?;
                if let Some(frame) = complete {
                    budget.remaining()?;
                    return Ok(frame);
                }
                if used == HEADER {
                    // Decoder has already validated this bounded version/domain header.
                    target = HEADER + u16::from_le_bytes([bytes[6], bytes[7]]) as usize;
                    if target <= HEADER || target > MAX_PACKET {
                        return Err(GateError::InvalidFrame);
                    }
                }
            }
            Err(error) if retry(&error) => {}
            Err(_) => return Err(GateError::Io),
        }
    }
}
/// Parent bootstrap writer; sends one bounded Capability frame before service startup.
pub(crate) fn send_capability_only(
    capability: &protocol::Capability,
    stream: &mut UnixStream,
    deadline: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<(), GateError> {
    let packet = protocol::encode_capability(capability).map_err(|_| GateError::InvalidFrame)?;
    let budget = Budget { deadline, cancel };
    exchange(stream, &budget, |stream, budget| {
        write_frame(stream, &packet, budget)
    })
}
/// Launcher bootstrap reader. Never consumes a following Begin frame.
pub(crate) fn receive_capability_only(
    stream: &mut UnixStream,
    deadline: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<protocol::Capability, GateError> {
    let budget = Budget { deadline, cancel };
    exchange(stream, &budget, |stream, budget| {
        let packet = read_frame(stream, Domain::Capability, budget)?;
        let capability =
            protocol::decode_capability(&packet).map_err(|_| GateError::InvalidFrame)?;
        budget.remaining()?;
        Ok(capability)
    })
}

/// Launcher-only identity writer. It never reads Begin from the service guardian's endpoint.
pub(crate) fn send_identity_only(
    identity: &ReaderIdentity,
    stream: &mut UnixStream,
    deadline: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<(), GateError> {
    let packet = protocol::encode_identity(identity).map_err(|_| GateError::InvalidFrame)?;
    let budget = Budget { deadline, cancel };
    exchange(stream, &budget, |stream, budget| {
        write_frame(stream, &packet, budget)
    })
}
/// Service-guardian-only Begin reader. Success is required before any tool spawn.
pub(crate) fn wait_begin_only(
    index: u8,
    nonce: [u8; 16],
    stream: &mut UnixStream,
    deadline: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<(), GateError> {
    protocol::encode_begin(&Begin { index, nonce }).map_err(|_| GateError::InvalidFrame)?;
    let budget = Budget { deadline, cancel };
    exchange(stream, &budget, |stream, budget| {
        let packet = read_frame(stream, Domain::Begin, budget)?;
        let begin = protocol::decode_begin(&packet).map_err(|_| GateError::InvalidFrame)?;
        if begin.index != index || begin.nonce != nonce {
            return Err(GateError::InvalidFrame);
        }
        budget.remaining()?;
        Ok(())
    })
}
/// Convenience wrapper for a single-role fixture; production launcher and guardian use split APIs.
pub(crate) fn send_identity_then_wait_begin(
    identity: &ReaderIdentity,
    stream: &mut UnixStream,
    deadline: Instant,
    cancel: Option<&AtomicBool>,
) -> Result<(), GateError> {
    send_identity_only(identity, stream, deadline, cancel)?;
    wait_begin_only(
        identity.capability.index,
        identity.capability.nonce,
        stream,
        deadline,
        cancel,
    )
}
/// Binding must complete durably before Begin is sent. Its private error is never exposed.
pub(crate) fn receive_identity_then_begin<E>(
    stream: &mut UnixStream,
    deadline: Instant,
    cancel: Option<&AtomicBool>,
    bind: impl FnOnce(&ReaderIdentity) -> Result<(), E>,
) -> Result<ReaderIdentity, GateError> {
    let budget = Budget { deadline, cancel };
    exchange(stream, &budget, |stream, budget| {
        let packet = read_frame(stream, Domain::Identity, budget)?;
        let identity = protocol::decode_identity(&packet).map_err(|_| GateError::InvalidFrame)?;
        budget.remaining()?;
        bind(&identity).map_err(|_| GateError::BindFailed)?;
        budget.remaining()?;
        let begin = Begin {
            index: identity.capability.index,
            nonce: identity.capability.nonce,
        };
        let packet = protocol::encode_begin(&begin).map_err(|_| GateError::InvalidFrame)?;
        write_frame(stream, &packet, budget)?;
        Ok(identity)
    })
}

#[cfg(test)]
mod tests {
    use super::super::reader_protocol::Capability;
    use super::*;
    use std::{
        sync::{Arc, atomic::AtomicU8},
        thread,
    };
    fn identity() -> ReaderIdentity {
        ReaderIdentity {
            capability: Capability {
                index: 0,
                nonce: [1; 16],
                digest: [2; 32],
                boot: [3; 16],
                receipt_dev: 4,
                receipt_ino: 5,
            },
            identity_digest: [6; 32],
            unit: [7; 16],
            invocation: [8; 16],
            cgroup_mount: 9,
            cgroup_dev: 10,
            cgroup_ino: 11,
            path: "/user.slice/owned.service".into(),
        }
    }
    fn no_begin(peer: &mut UnixStream) {
        peer.set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        let mut byte = [0];
        match peer.read(&mut byte) {
            Err(error) => assert!(retry(&error)),
            Ok(0) => {}
            Ok(_) => panic!("Begin was sent without durable binding"),
        }
    }
    #[test]
    fn capability_bootstrap_roundtrip_restores_timeouts_and_leaves_begin_unconsumed() {
        let (mut app, mut launcher) = UnixStream::pair().unwrap();
        let capability = identity().capability;
        launcher
            .set_read_timeout(Some(Duration::from_millis(73)))
            .unwrap();
        launcher
            .set_write_timeout(Some(Duration::from_millis(83)))
            .unwrap();
        let prior = (
            launcher.read_timeout().unwrap(),
            launcher.write_timeout().unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(1);
        send_capability_only(&capability, &mut app, deadline, None).unwrap();
        app.write_all(
            &protocol::encode_begin(&Begin {
                index: capability.index,
                nonce: capability.nonce,
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            receive_capability_only(&mut launcher, deadline, None).unwrap(),
            capability
        );
        assert_eq!(
            (
                launcher.read_timeout().unwrap(),
                launcher.write_timeout().unwrap()
            ),
            prior
        );
        wait_begin_only(
            capability.index,
            capability.nonce,
            &mut launcher,
            deadline,
            None,
        )
        .unwrap();
        assert_eq!(
            (
                launcher.read_timeout().unwrap(),
                launcher.write_timeout().unwrap()
            ),
            prior
        );
    }
    #[test]
    fn capability_wrong_domain_partial_eof_cancel_and_deadline_fail_closed() {
        let (mut app, mut launcher) = UnixStream::pair().unwrap();
        app.write_all(
            &protocol::encode_begin(&Begin {
                index: 0,
                nonce: [1; 16],
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            receive_capability_only(&mut launcher, Instant::now() + Duration::from_secs(1), None),
            Err(GateError::InvalidFrame)
        );
        let (mut app, mut launcher) = UnixStream::pair().unwrap();
        app.write_all(b"DLPG").unwrap();
        drop(app);
        assert_eq!(
            receive_capability_only(&mut launcher, Instant::now() + Duration::from_secs(1), None),
            Err(GateError::Ended)
        );
        let (mut app, mut launcher) = UnixStream::pair().unwrap();
        let cancel = AtomicBool::new(true);
        assert_eq!(
            send_capability_only(
                &identity().capability,
                &mut app,
                Instant::now() + Duration::from_secs(1),
                Some(&cancel)
            ),
            Err(GateError::Cancelled)
        );
        assert_eq!(
            receive_capability_only(
                &mut launcher,
                Instant::now() + Duration::from_secs(1),
                Some(&cancel)
            ),
            Err(GateError::Cancelled)
        );
        assert_eq!(
            send_capability_only(&identity().capability, &mut app, Instant::now(), None),
            Err(GateError::Deadline)
        );
        let start = Instant::now();
        assert_eq!(
            receive_capability_only(&mut launcher, start + Duration::from_millis(40), None),
            Err(GateError::Deadline)
        );
        assert!(start.elapsed() < Duration::from_millis(500));
        assert_eq!(launcher.read_timeout().unwrap(), None);
        assert_eq!(launcher.write_timeout().unwrap(), None);
    }
    #[test]
    fn split_launcher_writer_and_guardian_reader_preserve_actual_order() {
        let (mut app, mut launcher) = UnixStream::pair().unwrap();
        // Test-only duplicate models FD0 transferred to the guardian; neither helper clones it.
        let mut guardian = launcher.try_clone().unwrap();
        let value = identity();
        let expected = value.clone();
        let phase = Arc::new(AtomicU8::new(0));
        let observed = phase.clone();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            send_identity_only(&expected, &mut launcher, deadline, None).unwrap();
            wait_begin_only(
                expected.capability.index,
                expected.capability.nonce,
                &mut guardian,
                deadline,
                None,
            )
            .unwrap();
            assert_eq!(observed.load(Ordering::Acquire), 1);
            observed.store(2, Ordering::Release);
        });
        receive_identity_then_begin(
            &mut app,
            Instant::now() + Duration::from_secs(2),
            None,
            |_| {
                phase.store(1, Ordering::Release);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
        worker.join().unwrap();
        assert_eq!(phase.load(Ordering::Acquire), 2);
    }
    #[test]
    fn split_begin_partial_deadline_mismatch_and_eof_never_grant_spawn() {
        for bytes in [
            b"DLP".to_vec(),
            protocol::encode_begin(&Begin {
                index: 0,
                nonce: [9; 16],
            })
            .unwrap(),
        ] {
            let (mut app, mut guardian) = UnixStream::pair().unwrap();
            app.write_all(&bytes).unwrap();
            let start = Instant::now();
            let result = wait_begin_only(
                0,
                [1; 16],
                &mut guardian,
                start + Duration::from_millis(40),
                None,
            );
            assert_eq!(
                result,
                Err(if bytes.len() == 3 {
                    GateError::Deadline
                } else {
                    GateError::InvalidFrame
                })
            );
            assert!(start.elapsed() < Duration::from_millis(500));
        }
        let (app, mut guardian) = UnixStream::pair().unwrap();
        drop(app);
        assert_eq!(
            wait_begin_only(
                0,
                [1; 16],
                &mut guardian,
                Instant::now() + Duration::from_secs(1),
                None
            ),
            Err(GateError::Ended)
        );
        let (_app, mut guardian) = UnixStream::pair().unwrap();
        assert_eq!(
            wait_begin_only(
                0,
                [0; 16],
                &mut guardian,
                Instant::now() + Duration::from_secs(1),
                None
            ),
            Err(GateError::InvalidFrame)
        );
    }
    #[test]
    fn split_identity_deadline_sends_nothing_and_fragmented_begin_is_valid() {
        let (mut app, mut launcher) = UnixStream::pair().unwrap();
        assert_eq!(
            send_identity_only(&identity(), &mut launcher, Instant::now(), None),
            Err(GateError::Deadline)
        );
        no_begin(&mut app);
        let (mut app, mut guardian) = UnixStream::pair().unwrap();
        let frame = protocol::encode_begin(&Begin {
            index: 0,
            nonce: [1; 16],
        })
        .unwrap();
        let sender = thread::spawn(move || {
            for chunk in frame.chunks(3) {
                app.write_all(chunk).unwrap();
                thread::sleep(Duration::from_millis(1));
            }
        });
        wait_begin_only(
            0,
            [1; 16],
            &mut guardian,
            Instant::now() + Duration::from_secs(1),
            None,
        )
        .unwrap();
        sender.join().unwrap();
    }
    #[test]
    fn real_exchange_binds_before_begin_and_restores_timeouts() {
        let (mut app, mut peer) = UnixStream::pair().unwrap();
        let phase = Arc::new(AtomicU8::new(0));
        let observed = phase.clone();
        let value = identity();
        let sent = value.clone();
        let worker = thread::spawn(move || {
            send_identity_then_wait_begin(
                &sent,
                &mut peer,
                Instant::now() + Duration::from_secs(2),
                None,
            )
            .unwrap();
            assert_eq!(observed.load(Ordering::Acquire), 1);
        });
        app.set_read_timeout(Some(Duration::from_millis(71)))
            .unwrap();
        app.set_write_timeout(Some(Duration::from_millis(81)))
            .unwrap();
        let prior = (app.read_timeout().unwrap(), app.write_timeout().unwrap());
        let received = receive_identity_then_begin(
            &mut app,
            Instant::now() + Duration::from_secs(2),
            None,
            |_| {
                phase.store(1, Ordering::Release);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
        assert_eq!(received, value);
        worker.join().unwrap();
        assert_eq!(
            (app.read_timeout().unwrap(), app.write_timeout().unwrap()),
            prior
        );
    }
    #[test]
    fn bind_error_and_expired_bind_send_no_begin() {
        for expires in [false, true] {
            let (mut app, mut peer) = UnixStream::pair().unwrap();
            peer.write_all(&protocol::encode_identity(&identity()).unwrap())
                .unwrap();
            let deadline = Instant::now() + Duration::from_millis(30);
            let result = receive_identity_then_begin(&mut app, deadline, None, |_| {
                if expires {
                    thread::sleep(Duration::from_millis(50));
                    Ok(())
                } else {
                    Err("synthetic secret bind error")
                }
            });
            assert_eq!(
                result,
                Err(if expires {
                    GateError::Deadline
                } else {
                    GateError::BindFailed
                })
            );
            no_begin(&mut peer);
        }
    }
    #[test]
    fn fragmented_identity_and_mismatched_begin_are_bounded() {
        let (mut app, mut peer) = UnixStream::pair().unwrap();
        let packet = protocol::encode_identity(&identity()).unwrap();
        let sender = thread::spawn(move || {
            for part in packet.chunks(7) {
                peer.write_all(part).unwrap();
                thread::sleep(Duration::from_millis(1));
            }
            read_frame(
                &mut peer,
                Domain::Begin,
                &Budget {
                    deadline: Instant::now() + Duration::from_secs(2),
                    cancel: None,
                },
            )
            .unwrap()
        });
        receive_identity_then_begin(
            &mut app,
            Instant::now() + Duration::from_secs(2),
            None,
            |_| Ok::<_, ()>(()),
        )
        .unwrap();
        protocol::decode_begin(&sender.join().unwrap()).unwrap();
        let (mut app, mut peer) = UnixStream::pair().unwrap();
        let sent = identity();
        let worker = thread::spawn(move || {
            send_identity_then_wait_begin(
                &sent,
                &mut peer,
                Instant::now() + Duration::from_secs(2),
                None,
            )
        });
        read_frame(
            &mut app,
            Domain::Identity,
            &Budget {
                deadline: Instant::now() + Duration::from_secs(2),
                cancel: None,
            },
        )
        .unwrap();
        app.write_all(
            &protocol::encode_begin(&Begin {
                index: 1,
                nonce: [1; 16],
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(worker.join().unwrap(), Err(GateError::InvalidFrame));
    }
    #[test]
    fn partial_timeout_cancel_eof_and_oversize_send_no_begin() {
        let (mut app, mut peer) = UnixStream::pair().unwrap();
        peer.write_all(b"DLP").unwrap();
        let started = Instant::now();
        assert_eq!(
            receive_identity_then_begin(
                &mut app,
                started + Duration::from_millis(40),
                None,
                |_| Ok::<_, ()>(())
            ),
            Err(GateError::Deadline)
        );
        assert!(started.elapsed() < Duration::from_millis(500));
        no_begin(&mut peer);
        let cancel = AtomicBool::new(true);
        assert_eq!(
            receive_identity_then_begin(
                &mut app,
                Instant::now() + Duration::from_secs(1),
                Some(&cancel),
                |_| Ok::<_, ()>(())
            ),
            Err(GateError::Cancelled)
        );
        no_begin(&mut peer);
        let (mut app, peer) = UnixStream::pair().unwrap();
        drop(peer);
        assert_eq!(
            receive_identity_then_begin(
                &mut app,
                Instant::now() + Duration::from_secs(1),
                None,
                |_| Ok::<_, ()>(())
            ),
            Err(GateError::Ended)
        );
        let (mut app, mut peer) = UnixStream::pair().unwrap();
        peer.write_all(b"DLPG\x01I\xff\xff").unwrap();
        assert_eq!(
            receive_identity_then_begin(
                &mut app,
                Instant::now() + Duration::from_secs(1),
                None,
                |_| Ok::<_, ()>(())
            ),
            Err(GateError::InvalidFrame)
        );
        no_begin(&mut peer);
    }
}
