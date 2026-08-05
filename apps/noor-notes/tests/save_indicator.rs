use gtk::prelude::*;
use noor_notes::save_status::{SaveState, SaveStatusIndicator};

#[test]
fn indicator_exposes_saving_saved_and_retryable_failure() {
    gtk::init().unwrap();
    let indicator = SaveStatusIndicator::new();
    indicator.set_state(&SaveState::Saving);
    assert_eq!(indicator.label.text(), "Saving…");
    assert!(!indicator.retry.is_visible());
    indicator.set_state(&SaveState::Saved);
    assert_eq!(indicator.label.text(), "Saved");
    indicator.set_state(&SaveState::Failed("disk full".into()));
    assert_eq!(indicator.label.text(), "Save failed");
    assert!(indicator.retry.is_visible());
    assert_eq!(
        indicator.retry.tooltip_text().as_deref(),
        Some("Retry saving this note")
    );
}
