use chrono::Utc;
use noor_domain::{Note, NoteState};
use noor_notes::library_view::{NoteCounts, content_preview};

#[test]
fn preview_is_single_line_trimmed_and_unicode_safe() {
    assert_eq!(
        content_preview("  First line\nsecond line  ", 40),
        "First line second line"
    );
    assert_eq!(content_preview("বাংলা লেখা দীর্ঘ", 6), "বাংলা…");
    assert_eq!(content_preview("\n\t", 20), "Empty note");
}

#[test]
fn counts_partition_note_states() {
    let now = Utc::now();
    let active = Note::new(now);
    let mut archived = Note::new(now);
    archived.state = NoteState::Archived;
    let mut trashed = Note::new(now);
    trashed.state = NoteState::Trashed { deleted_at: now };
    assert_eq!(
        NoteCounts::from_notes(&[active, archived, trashed]),
        NoteCounts {
            active: 1,
            archived: 1,
            trashed: 1,
        }
    );
}
