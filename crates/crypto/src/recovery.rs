use data_encoding::BASE32_NOPAD;
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub struct RecoveryKey(pub(crate) Zeroizing<[u8; 32]>);

impl RecoveryKey {
    pub fn generate() -> Self {
        let mut bytes = [0_u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(Zeroizing::new(bytes))
    }

    pub fn encode(&self) -> String {
        let checksum = Sha256::digest(self.0.as_slice());
        let mut payload = Vec::with_capacity(36);
        payload.extend_from_slice(self.0.as_slice());
        payload.extend_from_slice(&checksum[..4]);
        let encoded = BASE32_NOPAD.encode(&payload);
        encoded
            .as_bytes()
            .chunks(5)
            .map(|chunk| std::str::from_utf8(chunk).expect("base32 is ASCII"))
            .collect::<Vec<_>>()
            .join("-")
    }
}
