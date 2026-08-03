use chrono::{DateTime, Utc};
use noor_domain::{Note, NoteState, Revision};

#[derive(Clone, Debug, PartialEq)]
pub enum MergeOutcome {
    Apply(Note),
    ConflictCopy(Note),
    Ignore,
}

pub fn merge_remote_revision(
    local: Option<&Note>,
    remote: Note,
    remote_device: &str,
    now: DateTime<Utc>,
) -> MergeOutcome {
    let Some(local) = local else {
        return MergeOutcome::Apply(remote);
    };
    if remote.revision > local.revision {
        return MergeOutcome::Apply(remote);
    }
    if remote.revision < local.revision || remote == *local {
        return MergeOutcome::Ignore;
    }
    if matches!(remote.state, NoteState::Trashed { .. }) {
        return MergeOutcome::Apply(remote);
    }

    let mut conflict = Note::new(now);
    conflict.content = format!(
        "Conflict copy — {} — {}\n\n{}",
        remote_device,
        now.format("%Y-%m-%d %H:%M:%S UTC"),
        remote.content
    );
    conflict.style = remote.style;
    conflict.geometry = remote.geometry;
    conflict.revision = Revision::default();
    MergeOutcome::ConflictCopy(conflict)
}
