use chrono::{DateTime, Utc};
use noor_domain::{Note, NoteState};

pub fn archive(note: &mut Note, now: DateTime<Utc>) {
    transition(note, NoteState::Archived, now);
}

pub fn trash(note: &mut Note, now: DateTime<Utc>) {
    transition(note, NoteState::Trashed { deleted_at: now }, now);
}

fn transition(note: &mut Note, state: NoteState, now: DateTime<Utc>) {
    note.state = state;
    note.revision = note.revision.next();
    note.updated_at = now;
}
