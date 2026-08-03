use chrono::{Duration, TimeZone, Utc};
use noor_domain::{Note, NoteState, Revision};
use noor_sync::{MergeOutcome, merge_remote_revision};

#[test]
fn concurrent_content_preserves_remote_as_named_conflict_copy() {
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    let mut local = Note::new(now);
    local.content = "local edit".into();
    local.revision = Revision::from_value(5);
    let mut remote = local.clone();
    remote.content = "remote edit".into();

    let outcome = merge_remote_revision(Some(&local), remote, "laptop", now);

    let MergeOutcome::ConflictCopy(conflict) = outcome else {
        panic!("expected a conflict copy");
    };
    assert!(
        conflict
            .content
            .starts_with("Conflict copy — laptop — 2026-08-04")
    );
    assert!(conflict.content.contains("remote edit"));
    assert_ne!(conflict.id, local.id);
}

#[test]
fn newer_tombstone_wins_over_an_offline_active_note() {
    let now = Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap();
    let mut local = Note::new(now);
    local.revision = Revision::from_value(3);
    let mut remote = local.clone();
    remote.revision = Revision::from_value(4);
    remote.state = NoteState::Trashed {
        deleted_at: now + Duration::minutes(2),
    };

    let outcome = merge_remote_revision(Some(&local), remote.clone(), "desktop", now);

    assert_eq!(outcome, MergeOutcome::Apply(remote));
}
