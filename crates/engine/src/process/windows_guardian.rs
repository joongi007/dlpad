//! Windows cookie guardian runtime; original handles are the only ownership authority.
use super::windows::{PreparedChild, SpawnOutcome};
use super::windows_guardian_protocol::{FRAME_LEN, Frame, Kind};
use super::{ExecutionControl, ProcessSpec};
use crate::cookies::GuardianJournal;
use crate::cookies::windows_records::WindowsReaderRecord;
use std::{
    fs::File,
    io::{self, Read, Write},
    os::windows::io::AsRawHandle,
    path::Path,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
use windows_sys::Win32::{Foundation::ERROR_BROKEN_PIPE, System::Pipes::PeekNamedPipe};

pub(super) struct Request<'a> {
    pub spec: &'a ProcessSpec,
    pub cache: Option<crate::windows_owned_deno_cache::OwnedDenoCache>,
    pub intent: &'a WindowsReaderRecord,
    pub slot: u8,
    pub nonce: [u8; 16],
    pub owner_path: &'a Path,
    pub receipt: File,
    pub control: File,
    pub events: File,
    pub stdout: File,
    pub stderr: File,
}
fn rejected() -> io::Error {
    io::Error::other("Windows guardian protocol failed")
}
struct Control {
    pipe: File,
    bytes: [u8; FRAME_LEN],
    filled: usize,
}
enum Poll {
    Pending,
    Frame(Frame),
    Eof,
}
impl Control {
    fn poll(&mut self) -> io::Result<Poll> {
        let mut available = 0;
        // SAFETY: exclusively owned read pipe and valid byte-count output; no payload buffer.
        if unsafe {
            PeekNamedPipe(
                self.pipe.as_raw_handle(),
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                &mut available,
                std::ptr::null_mut(),
            )
        } == 0
        {
            if io::Error::last_os_error().raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                return if self.filled == 0 {
                    Ok(Poll::Eof)
                } else {
                    Err(rejected())
                };
            }
            return Err(rejected());
        }
        if available == 0 {
            return Ok(Poll::Pending);
        }
        let count = (available as usize).min(FRAME_LEN - self.filled);
        let got = self
            .pipe
            .read(&mut self.bytes[self.filled..self.filled + count])
            .map_err(|_| rejected())?;
        if got == 0 {
            return Err(rejected());
        }
        self.filled += got;
        if self.filled < FRAME_LEN {
            return Ok(Poll::Pending);
        }
        let frame = Frame::decode(&self.bytes).map_err(|_| rejected())?;
        self.filled = 0;
        Ok(Poll::Frame(frame))
    }
    fn wait(&mut self, deadline: Instant) -> io::Result<Frame> {
        loop {
            if Instant::now() >= deadline {
                return Err(rejected());
            }
            match self.poll()? {
                Poll::Frame(frame) => return Ok(frame),
                Poll::Eof => return Err(rejected()),
                Poll::Pending => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }
}
fn matches(frame: &Frame, kind: Kind, slot: u8, nonce: [u8; 16], digest: [u8; 32]) -> bool {
    frame.kind() == kind
        && frame.slot() == slot
        && frame.nonce() == nonce
        && frame.digest() == digest
}
fn emit(events: &mut File, frame: Frame) -> io::Result<()> {
    // Three frames total (384 bytes); launcher guarantees a private >=4096-byte pipe.
    events
        .write_all(&frame.encode().map_err(|_| rejected())?)
        .map_err(|_| rejected())
}
fn make(
    kind: Kind,
    slot: u8,
    nonce: [u8; 16],
    digest: [u8; 32],
    creation: (u64, u64),
    code: u32,
) -> io::Result<Frame> {
    Frame::new(kind, slot, nonce, digest, creation, code).map_err(|_| rejected())
}

pub(super) fn run(request: Request<'_>) -> io::Result<u32> {
    let Request {
        spec,
        mut cache,
        intent,
        slot,
        nonce,
        owner_path,
        receipt,
        control,
        mut events,
        stdout,
        stderr,
    } = request;
    let deadline = Instant::now()
        .checked_add(spec.timeout)
        .ok_or_else(rejected)?;
    match (&spec.runtime_environment, &cache) {
        (None, None) => {}
        (Some(policy), Some(owned)) if policy.cache_path() == owned.path() => {}
        _ => return Err(rejected()),
    }
    let prepared =
        PreparedChild::prepare_with_stdio(spec, &stdout, &stderr).map_err(|_| rejected())?;
    drop(stdout);
    drop(stderr);
    let mut control = Control {
        pipe: control,
        bytes: [0; FRAME_LEN],
        filled: 0,
    };
    let startup = (|| -> io::Result<_> {
        let creation = prepared.creation_times().map_err(|_| rejected())?;
        let mut expected = intent.clone();
        let bound = expected
            .bind(slot, creation.0, creation.1)
            .map_err(|_| rejected())?;
        if bound.nonce != nonce {
            return Err(rejected());
        }
        emit(
            &mut events,
            make(Kind::Prepared, slot, nonce, [0; 32], creation, 0)?,
        )?;
        let ack = control.wait(deadline)?;
        if !matches(&ack, Kind::Ack, slot, nonce, bound.identity_digest) {
            return Err(rejected());
        }
        if let Some(owned) = &mut cache {
            owned.reacquire().map_err(|_| rejected())?;
        }
        let journal = GuardianJournal::acquire(owner_path, intent, slot, nonce, creation, receipt)
            .map_err(|_| rejected())?;
        Ok((journal, bound.identity_digest))
    })();
    let (mut journal, digest) = match startup {
        Ok(value) => value,
        Err(error) => {
            prepared.cancel().map_err(|_| rejected())?;
            return Err(error);
        }
    };
    // After independent journal acquisition, even a failed Ack write/Begin can obtain a receipt
    // only following explicit original Job termination confirmation.
    let gate = (|| -> io::Result<()> {
        emit(
            &mut events,
            make(Kind::Ack, slot, nonce, digest, (0, 0), 0)?,
        )?;
        let begin = control.wait(deadline)?;
        if !matches(&begin, Kind::Begin, slot, nonce, digest) {
            return Err(rejected());
        }
        if Instant::now() >= deadline {
            return Err(rejected());
        }
        if let Some(owned) = &cache {
            owned.verify().map_err(|_| rejected())?;
        }
        // Reject already queued cancellation, EOF, replay or even a partial next frame.
        // A later-arriving control change is handled by the running loop, not a hard race claim.
        match control.poll()? {
            Poll::Pending if control.filled == 0 => {}
            _ => return Err(rejected()),
        }
        Ok(())
    })();
    if let Err(error) = gate {
        let status = prepared.cancel_status().map_err(|_| rejected())?;
        journal.complete().map_err(|_| rejected())?;
        emit(
            &mut events,
            make(
                Kind::Terminal,
                slot,
                nonce,
                digest,
                (0, 0),
                status.code().ok_or_else(rejected)? as u32,
            )?,
        )?;
        return Err(error);
    }
    let cancel = AtomicBool::new(false);
    let execution = ExecutionControl {
        cancel: &cancel,
        deadline,
    };
    let (mut child, ownership) = match prepared.resume(&execution).map_err(|_| rejected())? {
        SpawnOutcome::Ready(child, ownership) => (child, ownership),
        SpawnOutcome::Interrupted(_) => {
            // Resume's last deadline race confirms cleanup but supplies no original status.
            // Never invent a code or publish a receipt/Terminal without that contract.
            return Err(rejected());
        }
    };
    let status = loop {
        // Poll control before leader status: duplicated Ack/Begin or malformed trailing frames
        // may never bypass owned cleanup simply because the leader exited concurrently.
        let interrupted = match control.poll() {
            Ok(Poll::Pending) => Instant::now() >= deadline,
            Ok(Poll::Eof) | Err(_) => true,
            Ok(Poll::Frame(frame)) => {
                let _valid_cancel = matches(&frame, Kind::Cancel, slot, nonce, [0; 32])
                    || matches(&frame, Kind::TimedOut, slot, nonce, [0; 32]);
                true // Every unexpected or repeated frame is also fail-closed cancellation.
            }
        };
        if interrupted {
            ownership.signal(true).map_err(|_| rejected())?;
            break child.wait().map_err(|_| rejected())?;
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                ownership.signal(true).map_err(|_| rejected())?;
                child.wait().map_err(|_| rejected())?;
                ownership.confirm_terminal().map_err(|_| rejected())?;
                journal.complete().map_err(|_| rejected())?;
                return Err(rejected());
            }
        }
    };
    // Natural root exit does not establish descendant cleanup. Terminate the original Job.
    ownership.signal(true).map_err(|_| rejected())?;
    ownership.confirm_terminal().map_err(|_| rejected())?;
    journal.complete().map_err(|_| rejected())?;
    let cache_failed = cache.as_ref().is_some_and(|owned| {
        owned
            .inventory()
            .map_or(true, |inventory| inventory.budget_exceeded)
    });
    let code = if cache_failed {
        126
    } else {
        status.code().ok_or_else(rejected)? as u32
    };
    emit(
        &mut events,
        make(Kind::Terminal, slot, nonce, digest, (0, 0), code)?,
    )?;
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::windows::io::FromRawHandle;
    fn pipe() -> (File, File) {
        let mut read = std::ptr::null_mut();
        let mut write = std::ptr::null_mut();
        // SAFETY: fresh non-inheritable anonymous pipe handles are transferred once.
        unsafe {
            assert_ne!(
                windows_sys::Win32::System::Pipes::CreatePipe(
                    &mut read,
                    &mut write,
                    std::ptr::null(),
                    4096
                ),
                0
            );
            (File::from_raw_handle(read), File::from_raw_handle(write))
        }
    }
    #[test]
    fn original_control_pipe_preserves_partial_frame_and_reports_eof() {
        let (reader, mut writer) = pipe();
        let mut control = Control {
            pipe: reader,
            bytes: [0; FRAME_LEN],
            filled: 0,
        };
        let bytes = make(Kind::Begin, 0, [1; 16], [2; 32], (0, 0), 0)
            .unwrap()
            .encode()
            .unwrap();
        writer.write_all(&bytes[..64]).unwrap();
        assert!(matches!(control.poll().unwrap(), Poll::Pending));
        assert_eq!(control.filled, 64);
        writer.write_all(&bytes[64..]).unwrap();
        let Poll::Frame(frame) = control.poll().unwrap() else {
            panic!("missing frame")
        };
        assert!(matches(&frame, Kind::Begin, 0, [1; 16], [2; 32]));
        assert_eq!(control.filled, 0);
        drop(writer);
        assert!(matches!(control.poll().unwrap(), Poll::Eof));
    }
    #[test]
    fn incomplete_control_eof_and_elapsed_deadline_are_rejected() {
        let (reader, mut writer) = pipe();
        let mut control = Control {
            pipe: reader,
            bytes: [0; FRAME_LEN],
            filled: 0,
        };
        writer.write_all(&[1; 17]).unwrap();
        assert!(matches!(control.poll().unwrap(), Poll::Pending));
        drop(writer);
        assert!(control.poll().is_err());
        assert!(control.wait(Instant::now()).is_err());
    }
    include!("windows_guardian/tests.rs");
}
