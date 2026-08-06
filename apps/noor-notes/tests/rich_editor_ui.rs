use adw::prelude::*;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn compact_toolbar_exposes_only_frequent_actions_at_top_level() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    assert!(toolbar.widget.has_css_class("nn-editor-toolbar"));
    assert_eq!(toolbar.widget.selection_mode(), gtk::SelectionMode::None);
    assert_eq!(toolbar.widget.max_children_per_line(), 9);
    assert_eq!(toolbar.widget.observe_children().n_items(), 9);
    assert!(toolbar.more.is_visible());

    let (_, narrow_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 190);
    let (_, wide_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 900);
    assert!(
        narrow_height > wide_height,
        "narrow toolbars must wrap into rows"
    );

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
