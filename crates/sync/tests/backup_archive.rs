use chrono::{Duration, TimeZone, Utc};
use noor_crypto::Vault;
use noor_domain::{Note, NoteState};
use noor_sync::{BackupArchive, BackupArchiveError};

fn notes() -> Vec<Note> {
    let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap();
    let mut archived = Note::new(now);
    archived.title = "Private archived title".into();
    archived.content = "secret archived body".into();
    archived.state = NoteState::Archived;
    let mut trashed = Note::new(now + Duration::minutes(1));
    trashed.title = "Private trash title".into();
    trashed.content = "secret trash body".into();
    trashed.state = NoteState::Trashed { deleted_at: now };
    vec![archived, trashed]
}

#[test]
fn archive_omits_plaintext_and_round_trips_every_note_state() {
    let (vault, _) = Vault::create(b"correct horse battery staple").unwrap();
    let created_at = Utc.with_ymd_and_hms(2026, 9, 3, 12, 30, 0).unwrap();
    let expected = notes();

    let backup = BackupArchive::create(&vault, created_at, "desktop-a", expected.clone()).unwrap();
    let bytes = serde_json::to_vec(&backup).unwrap();
    assert!(
        !bytes
            .windows(b"Private archived title".len())
            .any(|w| w == b"Private archived title")
    );
    assert!(
        !bytes
            .windows(b"secret trash body".len())
            .any(|w| w == b"secret trash body")
    );
    assert_eq!(BackupArchive::decrypt(&vault, &backup).unwrap(), expected);
    assert_eq!(
        BackupArchive::preview(&vault, &backup).unwrap().note_count,
        2
    );
    assert_eq!(
        BackupArchive::preview(&vault, &backup).unwrap().device_id,
        "desktop-a"
    );
}

#[test]
fn tampering_wrong_vault_and_metadata_changes_are_rejected() {
    let (vault, _) = Vault::create(b"correct horse battery staple").unwrap();
    let (wrong, _) = Vault::create(b"another correct passphrase").unwrap();
    let created_at = Utc.with_ymd_and_hms(2026, 9, 3, 12, 30, 0).unwrap();
    let backup = BackupArchive::create(&vault, created_at, "desktop-a", notes()).unwrap();

    assert!(BackupArchive::decrypt(&wrong, &backup).is_err());
    let mut tampered = backup.clone();
    tampered.ciphertext[0] ^= 1;
    assert!(BackupArchive::preview(&vault, &tampered).is_err());
    let mut metadata_changed = backup;
    metadata_changed.created_at += Duration::seconds(1);
    assert!(BackupArchive::preview(&vault, &metadata_changed).is_err());
}

#[test]
fn oversized_content_stops_at_the_archive_limit() {
    let (vault, _) = Vault::create(b"correct horse battery staple").unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 30, 0).unwrap();
    let mut note = Note::new(now);
    note.content = "x".repeat(128 * 1024 * 1024);

    assert!(matches!(
        BackupArchive::create(&vault, now, "desktop-a", vec![note]),
        Err(BackupArchiveError::TooLarge)
    ));
}
