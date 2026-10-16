//! Canonical Windows managed guardian control frames. No capability is established by decoding.
const MAGIC: &[u8; 8] = b"DLPMG001";
pub(super) const FRAME_LEN: usize = 128;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(super) enum Kind {
    Prepared = 1,
    Ack = 2,
    Begin = 3,
    Cancel = 4,
    TimedOut = 5,
    Terminal = 6,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct InvalidFrame;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Frame {
    kind: Kind,
    slot: u8,
    nonce: [u8; 16],
    digest: [u8; 32],
    guardian_created: u64,
    tool_created: u64,
    code: u32,
}

impl std::fmt::Debug for Frame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WindowsManagedGuardianFrameRedacted")
    }
}

impl Frame {
    pub(super) fn new(
        kind: Kind,
        slot: u8,
        nonce: [u8; 16],
        digest: [u8; 32],
        creation: (u64, u64),
        code: u32,
    ) -> Result<Self, InvalidFrame> {
        let frame = Self {
            kind,
            slot,
            nonce,
            digest,
            guardian_created: creation.0,
            tool_created: creation.1,
            code,
        };
        frame.validate()?;
        Ok(frame)
    }

    fn validate(&self) -> Result<(), InvalidFrame> {
        if self.slot > 7 || self.nonce == [0; 16] {
            return Err(InvalidFrame);
        }
        let no_creation = self.guardian_created == 0 && self.tool_created == 0;
        let canonical = match self.kind {
            Kind::Prepared => {
                self.digest == [0; 32]
                    && self.guardian_created != 0
                    && self.tool_created != 0
                    && self.code == 0
            }
            Kind::Ack | Kind::Begin => self.digest != [0; 32] && no_creation && self.code == 0,
            Kind::Cancel | Kind::TimedOut => {
                self.digest == [0; 32] && no_creation && self.code == 0
            }
            Kind::Terminal => self.digest != [0; 32] && no_creation,
        };
        if canonical { Ok(()) } else { Err(InvalidFrame) }
    }

    pub(super) fn encode(&self) -> Result<[u8; FRAME_LEN], InvalidFrame> {
        self.validate()?;
        let mut bytes = [0; FRAME_LEN];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8] = 1;
        bytes[9] = self.kind as u8;
        bytes[10] = self.slot;
        bytes[16..32].copy_from_slice(&self.nonce);
        bytes[32..64].copy_from_slice(&self.digest);
        bytes[64..72].copy_from_slice(&self.guardian_created.to_le_bytes());
        bytes[72..80].copy_from_slice(&self.tool_created.to_le_bytes());
        bytes[80..84].copy_from_slice(&self.code.to_le_bytes());
        Ok(bytes)
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, InvalidFrame> {
        if bytes.len() != FRAME_LEN
            || &bytes[..8] != MAGIC
            || bytes[8] != 1
            || bytes[11..16]
                .iter()
                .chain(bytes[84..].iter())
                .any(|byte| *byte != 0)
        {
            return Err(InvalidFrame);
        }
        let kind = match bytes[9] {
            1 => Kind::Prepared,
            2 => Kind::Ack,
            3 => Kind::Begin,
            4 => Kind::Cancel,
            5 => Kind::TimedOut,
            6 => Kind::Terminal,
            _ => return Err(InvalidFrame),
        };
        Self::new(
            kind,
            bytes[10],
            bytes[16..32].try_into().map_err(|_| InvalidFrame)?,
            bytes[32..64].try_into().map_err(|_| InvalidFrame)?,
            (
                u64::from_le_bytes(bytes[64..72].try_into().map_err(|_| InvalidFrame)?),
                u64::from_le_bytes(bytes[72..80].try_into().map_err(|_| InvalidFrame)?),
            ),
            u32::from_le_bytes(bytes[80..84].try_into().map_err(|_| InvalidFrame)?),
        )
    }

    pub(super) fn kind(&self) -> Kind {
        self.kind
    }
    pub(super) fn slot(&self) -> u8 {
        self.slot
    }
    pub(super) fn nonce(&self) -> [u8; 16] {
        self.nonce
    }
    pub(super) fn digest(&self) -> [u8; 32] {
        self.digest
    }
    pub(super) fn guardian_created(&self) -> u64 {
        self.guardian_created
    }
    pub(super) fn tool_created(&self) -> u64 {
        self.tool_created
    }
    pub(super) fn code(&self) -> u32 {
        self.code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(kind: Kind) -> Frame {
        let digest = if matches!(kind, Kind::Ack | Kind::Begin | Kind::Terminal) {
            [7; 32]
        } else {
            [0; 32]
        };
        let creation = if kind == Kind::Prepared {
            (0x0102030405060708, 19)
        } else {
            (0, 0)
        };
        Frame::new(
            kind,
            1,
            [9; 16],
            digest,
            creation,
            if kind == Kind::Terminal { u32::MAX } else { 0 },
        )
        .unwrap()
    }

    #[test]
    fn all_eight_slots_and_cookie_domain_separation() {
        for slot in 0..8 {
            let value = Frame::new(Kind::Begin, slot, [1; 16], [2; 32], (0, 0), 0).unwrap();
            assert_eq!(
                Frame::decode(&value.encode().unwrap()).unwrap().slot(),
                slot
            );
            let mut cookie = value.encode().unwrap();
            cookie[..8].copy_from_slice(b"DLPWG001");
            assert!(Frame::decode(&cookie).is_err());
        }
    }

    #[test]
    fn all_roles_roundtrip_with_native_little_endian_and_redacted_debug() {
        for kind in [
            Kind::Prepared,
            Kind::Ack,
            Kind::Begin,
            Kind::Cancel,
            Kind::TimedOut,
            Kind::Terminal,
        ] {
            let value = frame(kind);
            let bytes = value.encode().unwrap();
            assert_eq!(Frame::decode(&bytes).unwrap(), value);
            assert_eq!(value.kind(), kind);
            assert_eq!(value.slot(), 1);
            assert_eq!(value.nonce(), [9; 16]);
            assert_eq!(value.digest(), bytes[32..64]);
            assert_eq!(
                value.guardian_created(),
                u64::from_le_bytes(bytes[64..72].try_into().unwrap())
            );
            assert_eq!(
                value.tool_created(),
                u64::from_le_bytes(bytes[72..80].try_into().unwrap())
            );
            assert_eq!(
                value.code(),
                u32::from_le_bytes(bytes[80..84].try_into().unwrap())
            );
            assert_eq!(format!("{value:?}"), "WindowsManagedGuardianFrameRedacted");
        }
        assert_eq!(
            &frame(Kind::Prepared).encode().unwrap()[64..72],
            &[8, 7, 6, 5, 4, 3, 2, 1]
        );
    }

    #[test]
    fn exact_length_header_slot_nonce_and_every_reserved_byte_are_required() {
        let original = frame(Kind::Begin).encode().unwrap();
        for length in 0..FRAME_LEN {
            assert!(Frame::decode(&original[..length]).is_err());
        }
        let mut extra = original.to_vec();
        extra.push(0);
        assert!(Frame::decode(&extra).is_err());
        for index in (0..9).chain(11..16).chain(84..128) {
            let mut bytes = original;
            bytes[index] ^= 1;
            assert!(Frame::decode(&bytes).is_err());
        }
        for bad in [0, 7, 255] {
            let mut bytes = original;
            bytes[9] = bad;
            assert!(Frame::decode(&bytes).is_err());
        }
        let mut bytes = original;
        bytes[10] = 8;
        assert!(Frame::decode(&bytes).is_err());
        let mut bytes = original;
        bytes[16..32].fill(0);
        assert!(Frame::decode(&bytes).is_err());
    }

    #[test]
    fn cross_role_payloads_and_noncanonical_fields_fail_encode_and_decode() {
        for kind in [
            Kind::Prepared,
            Kind::Ack,
            Kind::Begin,
            Kind::Cancel,
            Kind::TimedOut,
            Kind::Terminal,
        ] {
            let valid = frame(kind);
            let mut bad = valid;
            bad.digest = if valid.digest == [0; 32] {
                [1; 32]
            } else {
                [0; 32]
            };
            assert!(bad.encode().is_err());
            let mut raw = valid.encode().unwrap();
            raw[32..64].copy_from_slice(&bad.digest);
            assert!(Frame::decode(&raw).is_err());
            let mut bad = valid;
            bad.guardian_created = if kind == Kind::Prepared { 0 } else { 1 };
            assert!(bad.encode().is_err());
            let mut raw = valid.encode().unwrap();
            raw[64..72].copy_from_slice(&bad.guardian_created.to_le_bytes());
            assert!(Frame::decode(&raw).is_err());
            let mut bad = valid;
            bad.tool_created = if kind == Kind::Prepared { 0 } else { 1 };
            assert!(bad.encode().is_err());
            let mut raw = valid.encode().unwrap();
            raw[72..80].copy_from_slice(&bad.tool_created.to_le_bytes());
            assert!(Frame::decode(&raw).is_err());
            if kind != Kind::Terminal {
                let mut bad = valid;
                bad.code = 1;
                assert!(bad.encode().is_err());
                let mut raw = valid.encode().unwrap();
                raw[80..84].copy_from_slice(&1u32.to_le_bytes());
                assert!(Frame::decode(&raw).is_err());
            }
            assert!(
                Frame::new(
                    kind,
                    8,
                    [1; 16],
                    valid.digest,
                    (valid.guardian_created, valid.tool_created),
                    valid.code
                )
                .is_err()
            );
            assert!(
                Frame::new(
                    kind,
                    0,
                    [0; 16],
                    valid.digest,
                    (valid.guardian_created, valid.tool_created),
                    valid.code
                )
                .is_err()
            );
        }
    }

    #[test]
    fn decode_does_not_claim_replay_or_session_authority() {
        let bytes = frame(Kind::Begin).encode().unwrap();
        assert_eq!(Frame::decode(&bytes), Frame::decode(&bytes));
        let mut other = bytes;
        other[16] = 8;
        assert_ne!(
            Frame::decode(&other).unwrap().nonce(),
            Frame::decode(&bytes).unwrap().nonce()
        );
        // Matching expected nonce/digest and legal sequencing belong to the helper state machine.
    }
}
