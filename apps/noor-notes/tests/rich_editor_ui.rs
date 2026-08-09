use adw::prelude::*;
use noor_notes::ui::editor_toolbar::EditorToolbar;

#[test]
fn compact_toolbar_exposes_only_frequent_actions_at_top_level() {
    gtk::init().unwrap();
    let toolbar = EditorToolbar::new();
    assert!(toolbar.widget.has_css_class("nn-editor-toolbar"));
    assert_eq!(toolbar.widget.orientation(), gtk::Orientation::Horizontal);
    assert!(toolbar.widget.spacing() >= 4);
    assert!(toolbar.more.is_visible());
    let mut separators = 0;
    let mut child = toolbar.widget.first_child();
    while let Some(widget) = child {
        if widget.is::<gtk::Separator>() {
            separators += 1;
        }
        child = widget.next_sibling();
    }
    assert_eq!(
        separators, 3,
        "primary actions must have three logical separators"
    );

    let (_, narrow_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 420);
    let (_, wide_height, _, _) = toolbar.widget.measure(gtk::Orientation::Vertical, 900);
    assert_eq!(
        narrow_height, wide_height,
        "the primary toolbar must never wrap into multiple rows"
    );
    assert_eq!(
        toolbar
            .formatting
            .section_labels
            .iter()
            .map(|label| label.text().to_string())
            .collect::<Vec<_>>(),
        [
            "Typography",
            "Formatting",
            "Alignment",
            "Text color",
            "Highlight"
        ]
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
    assert_eq!(toolbar.foreground_palette.preset_buttons.len(), 7);
    assert_eq!(toolbar.highlight_palette.preset_buttons.len(), 7);
    assert!(toolbar.foreground_palette.custom.can_focus());
    assert!(toolbar.highlight_palette.custom.can_focus());
    assert!(toolbar.foreground_palette.reset.tooltip_text().is_some());
    assert!(toolbar.highlight_palette.reset.tooltip_text().is_some());
    assert_eq!(
        toolbar.foreground_palette.preset_buttons[1]
            .tooltip_text()
            .as_deref(),
        Some("Blue text")
    );
    assert_eq!(
        toolbar.highlight_palette.preset_buttons[0]
            .tooltip_text()
            .as_deref(),
        Some("Yellow highlight")
    );
    assert_eq!(
        toolbar.foreground_palette.custom.tooltip_text().as_deref(),
        Some("Custom text color")
    );
    assert_eq!(
        toolbar.highlight_palette.custom.tooltip_text().as_deref(),
        Some("Custom highlight color")
    );
}
