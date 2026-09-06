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
    assert!(menu.contains("mode.rich"));
    assert!(menu.contains("save-as.docx"));

    toolbar.set_editor_mode(EditorMode::Code);
    assert!(!menu.item("format.bold").property::<bool>("visible"));
    let format = menu
        .menu_buttons()
        .iter()
        .find(|button| button.label().as_deref() == Some("Format"))
        .unwrap();
    assert!(
        !format.get_visible(),
        "Code mode must not expose an empty Format menu"
    );
    toolbar.set_editor_mode(EditorMode::Rich);
    assert!(
        format.get_visible(),
        "Rich formatting menu must return after mode switching"
    );

    toolbar.word_wrap.set_active(false);
    menu.item("view.word-wrap").emit_clicked();
    assert!(toolbar.word_wrap.is_active());
    assert!(menu.item_checked("view.word-wrap"));

    let preview_toolbar = EditorToolbar::new();
    let preview = EditorMenuBar::new_preview(&preview_toolbar);
    preview.set_editor_mode(EditorMode::PlainText);
    assert!(
        preview
            .item("mode.plain")
            .has_css_class("nn-selected-menu-row")
    );
    assert!(
        !preview
            .item("mode.rich")
            .has_css_class("nn-selected-menu-row")
    );
    let labels = preview
        .menu_buttons()
        .iter()
        .filter_map(|button| button.label())
        .map(|label| label.to_string())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["Save As", "Editor Mode", "Format"]);
    assert!(
        !labels
            .iter()
            .any(|label| label == "Edit" || label == "Insert")
    );
    assert!(!preview.contains("edit.undo"));
    assert!(!preview.contains("insert.emoji"));
    assert!(preview.contains("format.bold"));
    for key in [
        "mode.rich",
        "mode.markdown",
        "mode.plain",
        "mode.code",
        "save-as.docx",
        "save-as.pdf",
        "save-as.html",
        "save-as.text",
        "save-as.markdown",
    ] {
        assert!(preview.contains(key), "preview is missing {key}");
    }
    assert!(!preview.contains("file.delete"));
    assert!(!preview.contains("tools.more"));
}
