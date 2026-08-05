use std::fmt;

use rand::{RngCore, rngs::OsRng};
use zeroize::Zeroizing;

use crate::StorageError;

pub struct DatabaseKey(Zeroizing<[u8; 32]>);

impl DatabaseKey {
    pub fn generate() -> Self {
        let mut bytes = [0_u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(Zeroizing::new(bytes))
    }

    pub fn try_from_slice(bytes: &[u8]) -> Result<Self, StorageError> {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| StorageError::InvalidDatabaseKey)?;
        Ok(Self(Zeroizing::new(bytes)))
    }

    pub(crate) fn hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut output = String::with_capacity(64);
        for byte in self.0.iter() {
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
        }
        output
    }
}

impl fmt::Debug for DatabaseKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DatabaseKey([REDACTED])")
    }
}
