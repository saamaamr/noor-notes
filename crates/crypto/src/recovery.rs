use data_encoding::BASE32_NOPAD;
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

use crate::CryptoError;
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

    pub fn decode(encoded: &str) -> Result<Self, CryptoError> {
        let compact = encoded
            .trim()
            .chars()
            .filter(|character| *character != '-')
            .map(|character| character.to_ascii_uppercase())
            .collect::<String>();
        if compact.len() != 58
            || !compact
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || (b'2'..=b'7').contains(&byte))
        {
            return Err(CryptoError::InvalidRecoveryKey);
        }
        let decoded = Zeroizing::new(
            BASE32_NOPAD
                .decode(compact.as_bytes())
                .map_err(|_| CryptoError::InvalidRecoveryKey)?,
        );
        let payload: [u8; 36] = decoded
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidRecoveryKey)?;
        let key: [u8; 32] = payload[..32].try_into().expect("slice length is checked");
        let expected = Sha256::digest(key);
        if payload[32..] != expected[..4] {
            return Err(CryptoError::InvalidRecoveryKey);
        }
        Ok(Self(Zeroizing::new(key)))
    }
}
