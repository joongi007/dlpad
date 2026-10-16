//! Bounded terminal confirmation after the caller has reaped its owned child.
use super::OwnershipCleanupError;
use std::{
    io::{self, Read},
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

const LIMIT: Duration = Duration::from_secs(1);
const STEP: Duration = Duration::from_millis(20);

fn unconfirmed() -> io::Error {
    io::Error::other(OwnershipCleanupError::TerminalUnconfirmed)
}

/// Accept only GC + EOF, or GC followed by LC + EOF for a service launcher.
/// Caller must reap its child first and serialize all accesses to this socket.
/// No read bytes or OS errors are exposed through the fixed typed error.
pub(super) fn confirm(control: &mut UnixStream, expects_service: bool) -> io::Result<()> {
    let previous = control.read_timeout().map_err(|_| unconfirmed())?;
    let result = read_terminal(control, expects_service);
    let restored = control
        .set_read_timeout(previous)
        .map_err(|_| unconfirmed());
    result.and(restored)
}

fn read_terminal(control: &mut UnixStream, expects_service: bool) -> io::Result<()> {
    let expected: &[u8] = if expects_service { b"GCLC" } else { b"GC" };
    let started = Instant::now();
    let mut bytes = [0_u8; 5];
    let mut used = 0;
    loop {
        let Some(remaining) = LIMIT.checked_sub(started.elapsed()) else {
            return Err(unconfirmed());
        };
        if remaining.is_zero() {
            return Err(unconfirmed());
        }
        control
            .set_read_timeout(Some(remaining.min(STEP)))
            .map_err(|_| unconfirmed())?;
        match control.read(&mut bytes[used..]) {
            Ok(0) => {
                return if &bytes[..used] == expected {
                    Ok(())
                } else {
                    Err(unconfirmed())
                };
            }
            Ok(count) => {
                used += count;
                // Also reject bytes queued after a valid terminal frame.
                if used > expected.len() || bytes[..used] != expected[..used] {
                    return Err(unconfirmed());
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                ) => {}
            Err(_) => return Err(unconfirmed()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write, thread};

    fn assert_unconfirmed(result: io::Result<()>) {
        let error = result.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(matches!(
            error
                .get_ref()
                .and_then(|value| value.downcast_ref::<OwnershipCleanupError>()),
            Some(OwnershipCleanupError::TerminalUnconfirmed)
        ));
    }
    #[test]
    fn valid_frame_followed_by_unread_command_reset_is_unconfirmed() {
        for (service, frame) in [(false, &b"GC"[..]), (true, &b"GCLC"[..])] {
            let (mut reader, mut writer) = UnixStream::pair().unwrap();
            writer.write_all(frame).unwrap();
            reader.write_all(b"K").unwrap();
            drop(writer);
            // A complete acknowledgment is insufficient without genuine EOF.
            assert_unconfirmed(confirm(&mut reader, service));
        }
    }
    #[test]
    fn exact_direct_and_service_frames_require_eof() {
        for (service, data) in [(false, &b"GC"[..]), (true, &b"GCLC"[..])] {
            let (mut reader, mut writer) = UnixStream::pair().unwrap();
            writer.write_all(data).unwrap();
            drop(writer);
            confirm(&mut reader, service).unwrap();
            assert_eq!(reader.read_timeout().unwrap(), None);
        }
    }
    #[test]
    fn split_service_frames_and_prior_timeout_are_supported() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        reader
            .set_read_timeout(Some(Duration::from_millis(73)))
            .unwrap();
        // Compare the actual kernel value: requested durations may be quantized.
        let previous = reader.read_timeout().unwrap();
        let task = thread::spawn(move || {
            for byte in b"GCLC" {
                writer.write_all(&[*byte]).unwrap();
                thread::sleep(Duration::from_millis(5));
            }
        });
        confirm(&mut reader, true).unwrap();
        task.join().unwrap();
        assert_eq!(reader.read_timeout().unwrap(), previous);
    }
    #[test]
    fn malformed_truncated_duplicate_wrong_order_and_extra_are_rejected() {
        for service in [false, true] {
            for data in [
                &b""[..],
                b"G",
                b"GU",
                b"LU",
                b"GCx",
                b"GCGC",
                b"LCGC",
                b"GCL",
                b"GCLCG",
                b"cleanup confirmed",
                b"GCLC\0",
            ] {
                let (mut reader, mut writer) = UnixStream::pair().unwrap();
                writer.write_all(data).unwrap();
                drop(writer);
                assert_unconfirmed(confirm(&mut reader, service));
                assert_eq!(reader.read_timeout().unwrap(), None);
            }
        }
    }
    #[test]
    fn service_and_direct_contracts_are_distinct() {
        for (service, data) in [(true, &b"GC"[..]), (false, &b"GCLC"[..])] {
            let (mut reader, mut writer) = UnixStream::pair().unwrap();
            writer.write_all(data).unwrap();
            drop(writer);
            assert_unconfirmed(confirm(&mut reader, service));
        }
    }
    #[test]
    fn valid_prefix_without_eof_has_absolute_bound() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        writer.write_all(b"GC").unwrap();
        let started = Instant::now();
        assert_unconfirmed(confirm(&mut reader, false));
        let elapsed = started.elapsed();
        assert!(elapsed >= Duration::from_millis(950), "{elapsed:?}");
        assert!(elapsed < Duration::from_secs(2), "{elapsed:?}");
        assert_eq!(reader.read_timeout().unwrap(), None);
        drop(writer);
    }
    #[test]
    fn extra_byte_after_split_valid_prefix_is_rejected() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        let task = thread::spawn(move || {
            writer.write_all(b"GC").unwrap();
            thread::sleep(Duration::from_millis(15));
            writer.write_all(b"!").unwrap();
        });
        assert_unconfirmed(confirm(&mut reader, false));
        task.join().unwrap();
    }
}
