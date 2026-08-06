use adw::prelude::*;
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
fn view_only_is_a_direct_action_in_the_main_more_menu() {
    assert!(
        TOOLBAR.contains("view_only.upcast_ref()") && TOOLBAR.contains("more_actions.insert"),
        "View Only must be directly visible in More note actions"
    );
    assert!(
        !TOOLBAR.contains("view_box.append(&view_only)"),
        "View Only must not be hidden behind a second ellipsis menu"
    );
}

#[test]
fn more_actions_are_height_bounded_and_can_flow_into_columns() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    assert_eq!(
        toolbar.more_actions.orientation(),
        gtk::Orientation::Vertical
    );
    assert_eq!(
        toolbar.more_actions.selection_mode(),
        gtk::SelectionMode::None
    );
    assert_eq!(toolbar.more_actions.max_children_per_line(), 6);
    assert!(toolbar.more_actions.observe_children().n_items() >= 9);
    assert!(toolbar.view_only.is_ancestor(&toolbar.more_actions));
}
