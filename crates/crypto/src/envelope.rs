use noor_domain::{NoteId, Revision};
use serde::{Deserialize, Serialize};

pub const ENVELOPE_VERSION: u8 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedNote {
    pub version: u8,
    pub note_id: NoteId,
    pub revision: Revision,
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrappedVault {
    pub version: u8,
    pub salt: [u8; 16],
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryWrappedVault {
    pub version: u8,
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
}

pub(crate) fn note_aad(note_id: NoteId, revision: Revision) -> Vec<u8> {
    let mut aad = Vec::with_capacity(25);
    aad.push(ENVELOPE_VERSION);
    aad.extend_from_slice(note_id.value().as_bytes());
    aad.extend_from_slice(&revision.value().to_be_bytes());
    aad
}
