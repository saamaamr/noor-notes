const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn note_chrome_is_compact_but_keeps_accessible_hit_targets() {
    assert!(CSS.contains(".nn-control-compact { min-width: 32px; min-height: 32px"));
    assert!(CSS.contains(".nn-editor-toolbar { min-height: 32px"));
    assert!(CSS.contains(
        ".nn-editor-toolbar button, .nn-editor-toolbar menubutton > button { min-width: 32px; min-height: 32px"
    ));
    assert!(CSS.contains(".nn-more-actions { padding: 6px;"));
    assert!(CSS.contains(".nn-more-actions > flowboxchild { padding: 0;"));
    assert!(CSS.contains(".nn-menu-row"));
    assert!(CSS.contains(".nn-menu-separator"));
    assert!(CSS.contains(".nn-menu-danger"));
    assert!(CSS.contains(".nn-note-card"));
    assert!(CSS.contains("border-radius: 8px"));
    assert!(CSS.contains("@nn_surface"));
    assert!(CSS.contains("@nn_text"));
    assert!(CSS.contains("@nn_border"));
    assert!(!CSS.contains(".nn-theme-graphite"));
    assert!(!CSS.contains(".nn-theme-oled"));
    assert!(CSS.contains(".nn-icon-button"));
    assert!(CSS.contains(".nn-icon-neutral"));
}
