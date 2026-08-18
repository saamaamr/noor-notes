use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::ui::editor_header::EditorHeader;
use noor_notes::ui::editor_toolbar::EditorToolbar;

const NOTE_WINDOW: &str = include_str!("../src/note_window.rs");
const PRESENTATION: &str = include_str!("../src/ui/editor_presentation.rs");
const TOOLBAR: &str = include_str!("../src/ui/editor_toolbar.rs");

#[test]
fn note_window_persists_and_exits_view_only_mode_from_keyboard_or_body() {
    assert!(TOOLBAR.contains("pub view_only"));
    assert!(NOTE_WINDOW.contains("editor_preferences.view_only"));
    assert!(NOTE_WINDOW.contains("presentation.set_view_only"));
    assert!(NOTE_WINDOW.contains("GestureClick"));
    assert!(NOTE_WINDOW.contains("set_button(0)"));
    assert!(NOTE_WINDOW.contains("gtk::gdk::Key::Escape"));
    assert!(PRESENTATION.contains("editor.set_editable(false)"));
    assert!(PRESENTATION.contains("widget.set_visible(false)"));
}

#[test]
fn view_only_toggle_stays_visible_in_the_header_and_more_actions_remain_bounded() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    let header = EditorHeader::new(&Note::new(Utc::now()), &toolbar, &gtk::Button::new(), false);
    assert_eq!(
        toolbar.more_actions.orientation(),
        gtk::Orientation::Vertical
    );
    assert_eq!(
        toolbar.more_actions.selection_mode(),
        gtk::SelectionMode::None
    );
    assert_eq!(toolbar.more_actions.max_children_per_line(), 6);
    assert!(toolbar.more_actions.observe_children().n_items() >= 8);
    assert!(!toolbar.view_only.is_ancestor(&toolbar.more_actions));
    assert!(toolbar.view_only.is_ancestor(&header.widget));

    toolbar.set_view_only_state(false);
    assert_eq!(toolbar.view_only.label().as_deref(), Some("View Only"));
    assert_eq!(
        toolbar.view_only.tooltip_text().as_deref(),
        Some("Read this note without editing controls")
    );
    toolbar.set_view_only_state(true);
    assert_eq!(toolbar.view_only.label().as_deref(), Some("Exit View Only"));
    assert_eq!(
        toolbar.view_only.tooltip_text().as_deref(),
        Some("Return to editing this note")
    );
}
