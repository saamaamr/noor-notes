use noor_notes::edit_save_gate::EditSaveGate;

#[test]
fn rapid_editor_changes_arm_only_one_snapshot() {
    let mut gate = EditSaveGate::default();

    assert!(gate.mark_changed());
    for _ in 0..1_000 {
        assert!(!gate.mark_changed());
    }
    assert!(gate.take_snapshot());
    assert!(!gate.take_snapshot());
}

#[test]
fn close_can_take_the_latest_pending_snapshot() {
    let mut gate = EditSaveGate::default();

    assert!(gate.mark_changed());
    assert!(gate.take_snapshot());
    assert!(!gate.take_snapshot());
    assert!(gate.mark_changed());
}
