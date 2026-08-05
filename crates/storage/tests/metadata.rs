use chrono::{Duration, TimeZone, Utc};
use noor_domain::{Note, NoteColor};
use noor_storage::{NoteSort, SqliteNoteRepository};

#[tokio::test]
async fn metadata_search_sort_and_duplicate_are_persistent() {
    let dir = tempfile::tempdir().unwrap();
    let repo = SqliteNoteRepository::open(&dir.path().join("notes.db"))
        .await
        .unwrap();
    let now = Utc.with_ymd_and_hms(2026, 8, 5, 10, 0, 0).unwrap();
    let mut beta = Note::new(now);
    beta.title = "Beta".into();
    beta.content = "second".into();
    beta.color = NoteColor::Blue;
    beta.set_tags(vec!["Urgent".into()]);
    let mut alpha = Note::new(now + Duration::minutes(1));
    alpha.title = "Alpha".into();
    alpha.content = "first".into();
    repo.save_note(&beta).await.unwrap();
    repo.save_note(&alpha).await.unwrap();
    let tagged = repo
        .search_notes_sorted("urgent", NoteSort::UpdatedDesc)
        .await
        .unwrap();
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].id, beta.id);
    assert_eq!(tagged[0].color, NoteColor::Blue);
    let titled = repo
        .search_notes_sorted("", NoteSort::TitleAsc)
        .await
        .unwrap();
    assert_eq!(
        titled.iter().map(|n| n.title.as_str()).collect::<Vec<_>>(),
        vec!["Alpha", "Beta"]
    );
    let copy = repo
        .duplicate_note(beta.id, now + Duration::hours(1))
        .await
        .unwrap();
    assert_ne!(copy.id, beta.id);
    assert_eq!(
        repo.get_note(copy.id).await.unwrap().unwrap().tags,
        vec!["Urgent"]
    );
}
