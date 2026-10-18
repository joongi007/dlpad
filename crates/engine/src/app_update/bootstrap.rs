//! Fixed-image Windows parent/UI ownership. No path-based execution constructor.
//! Profile compatibility and update selection remain caller-owned policy.
#[cfg(any(windows, test))]
use crate::file_identity::NativeFileIdentity;
#[cfg(any(windows, test))]
use std::io;

pub const PROFILE_EPOCH: u32 = 1;
#[cfg(any(windows, test))]
const FRAME_BYTES: usize = 128;
#[cfg(any(windows, test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Hello = 1,
    Ready = 2,
    Commit = 3,
    RuntimeReady = 4,
}
#[cfg(any(windows, test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageRole {
    SelfOriginal = 1,
    Prepared = 2,
    Installed = 3,
}
#[cfg(any(windows, test))]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Frame {
    kind: Kind,
    role: ImageRole,
    epoch: u32,
    parent: u32,
    parent_creation: u64,
    child: u32,
    child_creation: u64,
    volume: u64,
    file_id: [u8; 16],
    nonce: [u8; 32],
}
#[cfg(any(windows, test))]
fn rejected() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "app bootstrap rejected")
}
#[cfg(any(windows, test))]
impl Frame {
    fn profile_matches(&self, identity: &NativeFileIdentity, epoch: u32) -> bool {
        epoch == PROFILE_EPOCH
            && self.epoch == epoch
            && matches!(identity, NativeFileIdentity::Windows {volume_serial,file_id}
                if *volume_serial==self.volume && *file_id==self.file_id)
    }
    fn encode(&self) -> [u8; FRAME_BYTES] {
        let mut bytes = [0; FRAME_BYTES];
        bytes[..8].copy_from_slice(b"DLPBOOT1");
        bytes[8] = self.kind as u8;
        bytes[9] = self.role as u8;
        bytes[16..20].copy_from_slice(&self.epoch.to_le_bytes());
        bytes[20..24].copy_from_slice(&self.parent.to_le_bytes());
        bytes[24..32].copy_from_slice(&self.parent_creation.to_le_bytes());
        bytes[32..36].copy_from_slice(&self.child.to_le_bytes());
        bytes[40..48].copy_from_slice(&self.child_creation.to_le_bytes());
        bytes[48..56].copy_from_slice(&self.volume.to_le_bytes());
        bytes[56..72].copy_from_slice(&self.file_id);
        bytes[72..104].copy_from_slice(&self.nonce);
        bytes
    }
    fn decode(bytes: &[u8; FRAME_BYTES]) -> io::Result<Self> {
        if &bytes[..8] != b"DLPBOOT1"
            || bytes[10..16]
                .iter()
                .chain(&bytes[36..40])
                .chain(&bytes[104..])
                .any(|b| *b != 0)
        {
            return Err(rejected());
        }
        let frame = Self {
            kind: match bytes[8] {
                1 => Kind::Hello,
                2 => Kind::Ready,
                3 => Kind::Commit,
                4 => Kind::RuntimeReady,
                _ => return Err(rejected()),
            },
            role: match bytes[9] {
                1 => ImageRole::SelfOriginal,
                2 => ImageRole::Prepared,
                3 => ImageRole::Installed,
                _ => return Err(rejected()),
            },
            epoch: u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
            parent: u32::from_le_bytes(bytes[20..24].try_into().unwrap()),
            parent_creation: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            child: u32::from_le_bytes(bytes[32..36].try_into().unwrap()),
            child_creation: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            volume: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            file_id: bytes[56..72].try_into().unwrap(),
            nonce: bytes[72..104].try_into().unwrap(),
        };
        if frame.epoch != PROFILE_EPOCH
            || frame.parent == 0
            || frame.child == 0
            || frame.parent == frame.child
            || frame.parent_creation == 0
            || frame.child_creation == 0
            || frame.file_id == [0; 16]
            || frame.nonce == [0; 32]
        {
            return Err(rejected());
        }
        Ok(frame)
    }
    fn expect(&self, actual: Self, kind: Kind) -> io::Result<()> {
        let mut expected = self.clone();
        expected.kind = kind;
        if actual != expected {
            return Err(rejected());
        }
        Ok(())
    }
}
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{
    InstalledImage, OwnedUi, PreparedImage, SelfImage, SpawnFailure, UiChild, UiExit, child_entry,
    spawn_installed_ui, spawn_prepared_ui, spawn_ui,
};

#[cfg(test)]
mod tests;

/// Requests carry bounded user intent, never file paths, keys or execution authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiRequest {
    Prepare {
        version: String,
        channel: super::Channel,
    },
    CancelPrepare,
    RestartPrepared {
        generation: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParentEvent {
    RestartAccepted {
        generation: u64,
    },
    Progress {
        phase: u8,
        received: u64,
        total: u64,
        generation: u64,
    },
    Prepared {
        version: String,
        generation: u64,
    },
    Failed {
        code: u16,
        generation: u64,
    },
}
#[cfg(any(windows, test))]
fn put_version(bytes: &mut [u8], version: &str) -> io::Result<()> {
    if version.is_empty()
        || version.len() > 32
        || !version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'+' | b'-'))
    {
        return Err(rejected());
    }
    bytes[..version.len()].copy_from_slice(version.as_bytes());
    Ok(())
}
#[cfg(any(windows, test))]
fn get_version(bytes: &[u8]) -> io::Result<String> {
    let length = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    if bytes[length..].iter().any(|b| *b != 0) {
        return Err(rejected());
    }
    let value = std::str::from_utf8(&bytes[..length])
        .map_err(|_| rejected())?
        .to_owned();
    put_version(&mut [0; 32], &value)?;
    Ok(value)
}
#[cfg(any(windows, test))]
fn runtime_base(magic: &[u8; 8], nonce: &[u8; 32], kind: u8) -> [u8; FRAME_BYTES] {
    let mut bytes = [0; FRAME_BYTES];
    bytes[..8].copy_from_slice(magic);
    bytes[8] = kind;
    bytes[16..48].copy_from_slice(nonce);
    bytes
}
#[cfg(any(windows, test))]
impl UiRequest {
    fn encode(&self, nonce: &[u8; 32]) -> io::Result<[u8; FRAME_BYTES]> {
        let mut bytes = runtime_base(
            b"DLPREQ01",
            nonce,
            match self {
                Self::Prepare { .. } => 1,
                Self::CancelPrepare => 2,
                Self::RestartPrepared { .. } => 3,
            },
        );
        match self {
            Self::Prepare { version, channel } => {
                bytes[9] = match channel {
                    super::Channel::Stable => 1,
                    super::Channel::Preview => 2,
                };
                put_version(&mut bytes[72..104], version)?;
            }
            Self::CancelPrepare => {}
            Self::RestartPrepared { generation } => {
                if *generation == 0 {
                    return Err(rejected());
                }
                bytes[48..56].copy_from_slice(&generation.to_le_bytes());
            }
        }
        Ok(bytes)
    }
    fn decode(bytes: &[u8; FRAME_BYTES], nonce: &[u8; 32]) -> io::Result<Self> {
        let value = match bytes[8] {
            1 => Self::Prepare {
                version: get_version(&bytes[72..104])?,
                channel: match bytes[9] {
                    1 => super::Channel::Stable,
                    2 => super::Channel::Preview,
                    _ => return Err(rejected()),
                },
            },
            2 => Self::CancelPrepare,
            3 => Self::RestartPrepared {
                generation: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
            },
            _ => return Err(rejected()),
        };
        if value.encode(nonce)? != *bytes {
            return Err(rejected());
        }
        Ok(value)
    }
}
#[cfg(any(windows, test))]
impl ParentEvent {
    fn encode(&self, nonce: &[u8; 32]) -> io::Result<[u8; FRAME_BYTES]> {
        let mut bytes = runtime_base(
            b"DLPEVT01",
            nonce,
            match self {
                Self::Progress { .. } => 1,
                Self::Prepared { .. } => 2,
                Self::Failed { .. } => 3,
                Self::RestartAccepted { .. } => 4,
            },
        );
        let generation = match self {
            Self::RestartAccepted { generation } => *generation,
            Self::Progress {
                phase,
                received,
                total,
                generation,
            } => {
                if *received > *total {
                    return Err(rejected());
                }
                bytes[9] = *phase;
                bytes[56..64].copy_from_slice(&received.to_le_bytes());
                bytes[64..72].copy_from_slice(&total.to_le_bytes());
                *generation
            }
            Self::Prepared {
                version,
                generation,
            } => {
                put_version(&mut bytes[72..104], version)?;
                *generation
            }
            Self::Failed { code, generation } => {
                if *code == 0 {
                    return Err(rejected());
                }
                bytes[10..12].copy_from_slice(&code.to_le_bytes());
                *generation
            }
        };
        // Recovery is a parent startup notice, not an operation generation. No
        // other event may bypass the nonzero operation-generation requirement.
        if generation == 0
            && !matches!(
                self,
                Self::Failed {
                    code: 13,
                    generation: 0
                }
            )
        {
            return Err(rejected());
        }
        bytes[48..56].copy_from_slice(&generation.to_le_bytes());
        Ok(bytes)
    }
    fn decode(bytes: &[u8; FRAME_BYTES], nonce: &[u8; 32]) -> io::Result<Self> {
        let generation = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
        let value = match bytes[8] {
            1 => Self::Progress {
                phase: bytes[9],
                received: u64::from_le_bytes(bytes[56..64].try_into().unwrap()),
                total: u64::from_le_bytes(bytes[64..72].try_into().unwrap()),
                generation,
            },
            2 => Self::Prepared {
                version: get_version(&bytes[72..104])?,
                generation,
            },
            3 => Self::Failed {
                code: u16::from_le_bytes(bytes[10..12].try_into().unwrap()),
                generation,
            },
            4 => Self::RestartAccepted { generation },
            _ => return Err(rejected()),
        };
        if value.encode(nonce)? != *bytes {
            return Err(rejected());
        }
        Ok(value)
    }
}
