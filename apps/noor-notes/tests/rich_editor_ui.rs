use adw::prelude::*;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn compact_toolbar_exposes_only_frequent_actions_at_top_level() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    assert!(toolbar.widget.has_css_class("nn-editor-toolbar"));
    assert_eq!(toolbar.widget.observe_children().n_items(), 5);
    for button in [
        toolbar.undo.upcast_ref::<gtk::Widget>(),
        toolbar.redo.upcast_ref::<gtk::Widget>(),
        toolbar.find.upcast_ref::<gtk::Widget>(),
        toolbar.bold.upcast_ref::<gtk::Widget>(),
        toolbar.italic.upcast_ref::<gtk::Widget>(),
        toolbar.bullets.upcast_ref::<gtk::Widget>(),
    ] {
        assert!(button.tooltip_text().is_some());
    }
}
