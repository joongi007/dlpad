//! Operational verification root. Only the public key is shipped or committed.
use super::TrustedKey;

pub const KEY_ID: &str = "dlpad-release-20261018-v1";
pub const PUBLIC_KEY: [u8; 32] = [
    0x35, 0x97, 0xdf, 0xf2, 0xcb, 0xee, 0x2d, 0x4a, 0x2f, 0xae, 0x1b, 0xaa, 0xcd, 0xc5, 0xcd, 0xdd,
    0xa8, 0xdb, 0x62, 0x93, 0x35, 0xf0, 0x8f, 0xcf, 0xe9, 0x62, 0x51, 0xa8, 0x2a, 0x34, 0x5e, 0x2d,
];
pub fn keys() -> [TrustedKey<'static>; 1] {
    [TrustedKey {
        id: KEY_ID,
        public_key: &PUBLIC_KEY,
    }]
}

#[cfg(test)]
mod tests {
    #[test]
    fn release_signer_bootstrap_matches_the_compiled_verification_root() {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("release-public-key.json")).unwrap();
        assert_eq!(value["key_id"], super::KEY_ID);
        assert_eq!(value["algorithm"], "Ed25519");
        let hex = super::PUBLIC_KEY
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(value["public_key_hex"], hex);
        assert_ne!(super::PUBLIC_KEY, [0; 32]);
    }
}
