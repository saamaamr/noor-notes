use chrono::{Duration, TimeZone, Utc};
use noor_domain::{Note, NoteState};
use noor_storage::SqliteNoteRepository;

#[tokio::test]
async fn search_finds_latin_case_insensitively_and_preserves_unicode() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    let mut latin = Note::new(now);
    latin.content = "Project ALPHA".into();
    let mut bangla = Note::new(now);
    bangla.content = "বাংলা পরীক্ষা".into();
    repo.save_note(&latin).await.unwrap();
    repo.save_note(&bangla).await.unwrap();

    assert_eq!(repo.search_notes("alpha").await.unwrap()[0].id, latin.id);
    assert_eq!(repo.search_notes("বাংলা").await.unwrap()[0].id, bangla.id);
}

#[tokio::test]
async fn archive_trash_and_restore_are_persisted_as_new_revisions() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    let note = Note::new(now);
    repo.save_note(&note).await.unwrap();

    repo.archive(note.id, now + Duration::minutes(1))
        .await
        .unwrap();
    let archived = repo.get_note(note.id).await.unwrap().unwrap();
    assert_eq!(archived.state, NoteState::Archived);
    assert_eq!(archived.revision.value(), 1);

    repo.trash(note.id, now + Duration::minutes(2))
        .await
        .unwrap();
    let trashed = repo.get_note(note.id).await.unwrap().unwrap();
    assert_eq!(
        trashed.state,
        NoteState::Trashed {
            deleted_at: now + Duration::minutes(2)
        }
    );
    assert_eq!(trashed.revision.value(), 2);

    repo.restore(note.id, now + Duration::minutes(3))
        .await
        .unwrap();
    let restored = repo.get_note(note.id).await.unwrap().unwrap();
    assert_eq!(restored.state, NoteState::Active);
    assert_eq!(restored.revision.value(), 3);
}

#[tokio::test]
async fn acknowledged_change_is_removed_from_pending_results() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let note = Note::new(Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap());
    repo.save_note(&note).await.unwrap();
    let change = repo.pending_changes(1).await.unwrap().remove(0);

    repo.ack_change(change.id).await.unwrap();

    assert!(repo.pending_changes(10).await.unwrap().is_empty());
}

#[tokio::test]
async fn permanent_delete_removes_only_the_selected_note_and_its_changes() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    let deleted = Note::new(now);
    let survivor = Note::new(now);
    repo.save_note(&deleted).await.unwrap();
    repo.save_note(&survivor).await.unwrap();
    repo.delete_permanently(deleted.id).await.unwrap();
    assert!(repo.get_note(deleted.id).await.unwrap().is_none());
    assert_eq!(
        repo.get_note(survivor.id).await.unwrap().unwrap().id,
        survivor.id
    );
    assert!(
        repo.pending_changes(20)
            .await
            .unwrap()
            .iter()
            .all(|change| change.note_id != deleted.id)
    );
}
