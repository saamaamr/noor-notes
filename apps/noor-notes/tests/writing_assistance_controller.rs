use std::sync::Arc;

use adw::prelude::*;
use noor_domain::EditorMode;
use noor_notes::writing_assistance::{
    AssistanceStatus, GrammarService, IssuePopover, ResolvedWritingAssistance,
    WritingAssistanceController,
};

#[test]
fn controller_renders_replaces_suppresses_and_presents_issues() {
    gtk::init().unwrap();
    let buffer = sourceview5::Buffer::new(None);
    buffer.set_text("This is an test.");
    let controller = WritingAssistanceController::new(
        &buffer,
        Arc::new(GrammarService::default()),
        "en",
        EditorMode::PlainText,
    );
    controller.set_preferences(ResolvedWritingAssistance {
        spelling: true,
        grammar: true,
        offline_prediction: true,
        cloud: false,
    });

    controller.check_now();

    assert!(!controller.visible_issues().is_empty());
    assert_eq!(controller.status(), AssistanceStatus::Offline);
    let index = controller
        .visible_issues()
        .iter()
        .position(|issue| issue.replacements.iter().any(|value| value == "a"))
        .unwrap();
    let replacement = controller.visible_issues()[index]
        .replacements
        .iter()
        .position(|value| value == "a")
        .unwrap();
    controller.apply_replacement(index, replacement);
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        "This is a test."
    );
    buffer.undo();
    assert_eq!(
        buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        "This is an test."
    );

    controller.set_suppressed(true);
    controller.check_now();
    assert!(controller.visible_issues().is_empty());
    assert_eq!(controller.status(), AssistanceStatus::Idle);

    let popover = IssuePopover::new("Grammar", "Use the correct article", &["a".into()]);
    assert!(popover.text().contains("Grammar"));
    assert!(popover.text().contains("Use the correct article"));
    assert!(popover.text().contains("a"));
    assert!(popover.text().contains("Ignore once"));
    assert!(popover.widget.is_focusable());
}
