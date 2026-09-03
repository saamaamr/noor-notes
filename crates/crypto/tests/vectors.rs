use noor_crypto::{CryptoError, RecoveryKey, Vault};
use noor_domain::{NoteId, Revision};

#[test]
fn vault_round_trip_hides_plaintext_and_reopens_with_passphrase() {
    let (vault, wrapped) = Vault::create(b"correct horse battery staple").unwrap();
    let note_id = NoteId::new();
    let plaintext = "private note: বাংলা اختبار".as_bytes();

    let envelope = vault
        .encrypt_note(note_id, Revision::from_value(7), plaintext)
        .unwrap();

    assert!(
        !envelope
            .ciphertext
            .windows(plaintext.len())
            .any(|w| w == plaintext)
    );
    let reopened = Vault::unlock(b"correct horse battery staple", &wrapped).unwrap();
    assert_eq!(reopened.decrypt_note(&envelope).unwrap(), plaintext);
}

#[test]
fn wrong_passphrase_and_tampering_fail_authentication() {
    let (vault, wrapped) = Vault::create(b"right passphrase").unwrap();
    assert!(matches!(
        Vault::unlock(b"wrong passphrase", &wrapped),
        Err(CryptoError::AuthenticationFailed)
    ));
    let mut envelope = vault
        .encrypt_note(NoteId::new(), Revision::from_value(1), b"do not alter")
        .unwrap();
    envelope.ciphertext[0] ^= 1;
    assert!(matches!(
        vault.decrypt_note(&envelope),
        Err(CryptoError::AuthenticationFailed)
    ));
}

#[test]
fn recovery_key_is_grouped_and_can_unlock_the_same_vault() {
    let (vault, _) = Vault::create(b"account passphrase").unwrap();
    let recovery = RecoveryKey::generate();
    let encoded = recovery.encode();
    assert!(encoded.contains('-'));
    let wrapped = vault.wrap_for_recovery(&recovery).unwrap();

    let recovered = Vault::unlock_with_recovery(&recovery, &wrapped).unwrap();
    let envelope = vault
        .encrypt_note(NoteId::new(), Revision::from_value(2), b"recoverable")
        .unwrap();
    assert_eq!(recovered.decrypt_note(&envelope).unwrap(), b"recoverable");
}
#[test]
fn recovery_key_text_round_trips_and_rejects_checksum_tampering() {
    let encoded = RecoveryKey::generate().encode();
    let decoded = RecoveryKey::decode(&encoded).unwrap();
    assert_eq!(decoded.encode(), encoded);

    let mut tampered = encoded.into_bytes();
    let index = tampered.iter().position(|byte| *byte != b'-').unwrap();
    tampered[index] = if tampered[index] == b'A' { b'B' } else { b'A' };
    let tampered = String::from_utf8(tampered).unwrap();
    assert!(matches!(
        RecoveryKey::decode(&tampered),
        Err(CryptoError::InvalidRecoveryKey)
    ));
}
