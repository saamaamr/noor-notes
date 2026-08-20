use adw::prelude::*;
use noor_domain::EditorMode;
use noor_notes::ui::editor_menu_bar::EditorMenuBar;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn menu_items_follow_source_state_and_preview_capabilities() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    let menu = EditorMenuBar::new(&toolbar);

    toolbar.set_editor_mode(EditorMode::PlainText);
    assert!(!menu.item("format.bold").property::<bool>("visible"));
    assert!(menu.item("insert.emoji").property::<bool>("visible"));

    toolbar.set_editor_mode(EditorMode::Code);
    assert!(!menu.item("insert.emoji").property::<bool>("visible"));

    toolbar.undo.set_sensitive(false);
    assert!(!menu.item("edit.undo").is_sensitive());

    toolbar.word_wrap.set_active(false);
    menu.item("view.word-wrap").emit_clicked();
    assert!(toolbar.word_wrap.is_active());
    assert!(menu.item_checked("view.word-wrap"));

    let preview_toolbar = EditorToolbar::new();
    let preview = EditorMenuBar::new_preview(&preview_toolbar);
    assert!(preview.contains("edit.undo"));
    assert!(preview.contains("insert.emoji"));
    assert!(preview.contains("format.bold"));
    assert!(!preview.contains("file.delete"));
    assert!(!preview.contains("tools.more"));
}
