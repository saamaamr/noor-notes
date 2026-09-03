use argon2::Argon2;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use noor_domain::{NoteId, Revision};
use rand::{RngCore, rngs::OsRng};
use zeroize::Zeroizing;

use crate::envelope::{
    ENVELOPE_VERSION, EncryptedNote, RecoveryWrappedVault, WrappedVault, note_aad,
};
use crate::{CryptoError, RecoveryKey};

const VAULT_WRAP_AAD: &[u8] = b"noor-notes-vault-wrap-v1";
const RECOVERY_WRAP_AAD: &[u8] = b"noor-notes-recovery-wrap-v1";
const BACKUP_AAD_DOMAIN: &[u8] = b"noor-notes-backup-v1";

pub struct Vault {
    key: Zeroizing<[u8; 32]>,
}

impl Vault {
    pub fn create(passphrase: &[u8]) -> Result<(Self, WrappedVault), CryptoError> {
        let mut key = [0_u8; 32];
        OsRng.fill_bytes(&mut key);
        let vault = Self {
            key: Zeroizing::new(key),
        };
        let wrapped = vault.wrap_for_passphrase(passphrase)?;
        Ok((vault, wrapped))
    }

    pub fn unlock(passphrase: &[u8], wrapped: &WrappedVault) -> Result<Self, CryptoError> {
        ensure_version(wrapped.version)?;
        let wrapping_key = derive_key(passphrase, &wrapped.salt)?;
        let key = decrypt_key(
            wrapping_key.as_slice(),
            &wrapped.nonce,
            &wrapped.ciphertext,
            VAULT_WRAP_AAD,
        )?;
        Ok(Self {
            key: Zeroizing::new(key),
        })
    }

    pub fn encrypt_note(
        &self,
        note_id: NoteId,
        revision: Revision,
        plaintext: &[u8],
    ) -> Result<EncryptedNote, CryptoError> {
        let nonce = random_nonce();
        let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_slice()));
        let aad = note_aad(note_id, revision);
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)?;
        Ok(EncryptedNote {
            version: ENVELOPE_VERSION,
            note_id,
            revision,
            nonce,
            ciphertext,
        })
    }

    pub fn decrypt_note(&self, envelope: &EncryptedNote) -> Result<Vec<u8>, CryptoError> {
        ensure_version(envelope.version)?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_slice()));
        let aad = note_aad(envelope.note_id, envelope.revision);
        cipher
            .decrypt(
                XNonce::from_slice(&envelope.nonce),
                Payload {
                    msg: &envelope.ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)
    }

    pub fn encrypt_backup(
        &self,
        plaintext: &[u8],
        metadata: &[u8],
    ) -> Result<([u8; 24], Vec<u8>), CryptoError> {
        let nonce = random_nonce();
        let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_slice()));
        let aad = backup_aad(metadata);
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)?;
        Ok((nonce, ciphertext))
    }

    pub fn decrypt_backup(
        &self,
        nonce: &[u8; 24],
        ciphertext: &[u8],
        metadata: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(self.key.as_slice()));
        let aad = backup_aad(metadata);
        cipher
            .decrypt(
                XNonce::from_slice(nonce),
                Payload {
                    msg: ciphertext,
                    aad: &aad,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)
    }

    pub fn wrap_for_recovery(
        &self,
        recovery: &RecoveryKey,
    ) -> Result<RecoveryWrappedVault, CryptoError> {
        let nonce = random_nonce();
        let ciphertext = encrypt_key(
            recovery.0.as_slice(),
            &nonce,
            self.key.as_slice(),
            RECOVERY_WRAP_AAD,
        )?;
        Ok(RecoveryWrappedVault {
            version: ENVELOPE_VERSION,
            nonce,
            ciphertext,
        })
    }

    pub fn unlock_with_recovery(
        recovery: &RecoveryKey,
        wrapped: &RecoveryWrappedVault,
    ) -> Result<Self, CryptoError> {
        ensure_version(wrapped.version)?;
        let key = decrypt_key(
            recovery.0.as_slice(),
            &wrapped.nonce,
            &wrapped.ciphertext,
            RECOVERY_WRAP_AAD,
        )?;
        Ok(Self {
            key: Zeroizing::new(key),
        })
    }

    fn wrap_for_passphrase(&self, passphrase: &[u8]) -> Result<WrappedVault, CryptoError> {
        let mut salt = [0_u8; 16];
        OsRng.fill_bytes(&mut salt);
        let nonce = random_nonce();
        let wrapping_key = derive_key(passphrase, &salt)?;
        let ciphertext = encrypt_key(
            wrapping_key.as_slice(),
            &nonce,
            self.key.as_slice(),
            VAULT_WRAP_AAD,
        )?;
        Ok(WrappedVault {
            version: ENVELOPE_VERSION,
            salt,
            nonce,
            ciphertext,
        })
    }
}

fn backup_aad(metadata: &[u8]) -> Vec<u8> {
    let mut aad = Vec::with_capacity(BACKUP_AAD_DOMAIN.len() + metadata.len());
    aad.extend_from_slice(BACKUP_AAD_DOMAIN);
    aad.extend_from_slice(metadata);
    aad
}

fn derive_key(passphrase: &[u8], salt: &[u8; 16]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let mut key = Zeroizing::new([0_u8; 32]);
    Argon2::default()
        .hash_password_into(passphrase, salt, key.as_mut())
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    Ok(key)
}

fn encrypt_key(
    wrapping_key: &[u8],
    nonce: &[u8; 24],
    vault_key: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    XChaCha20Poly1305::new(Key::from_slice(wrapping_key))
        .encrypt(
            XNonce::from_slice(nonce),
            Payload {
                msg: vault_key,
                aad,
            },
        )
        .map_err(|_| CryptoError::AuthenticationFailed)
}

fn decrypt_key(
    wrapping_key: &[u8],
    nonce: &[u8; 24],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<[u8; 32], CryptoError> {
    let plaintext = Zeroizing::new(
        XChaCha20Poly1305::new(Key::from_slice(wrapping_key))
            .decrypt(
                XNonce::from_slice(nonce),
                Payload {
                    msg: ciphertext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::AuthenticationFailed)?,
    );
    plaintext
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::AuthenticationFailed)
}

fn random_nonce() -> [u8; 24] {
    let mut nonce = [0_u8; 24];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

fn ensure_version(version: u8) -> Result<(), CryptoError> {
    if version == ENVELOPE_VERSION {
        Ok(())
    } else {
        Err(CryptoError::UnsupportedVersion(version))
    }
}
