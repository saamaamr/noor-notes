const CSS: &str = include_str!("../resources/modern.css");

#[test]
fn note_chrome_is_compact_but_keeps_accessible_hit_targets() {
    assert!(CSS.contains("min-height: 40px"));
    assert!(CSS.contains("min-width: 40px"));
    assert!(CSS.contains("-gtk-icon-size: 16px"));
    assert!(CSS.contains("border-radius: 8px"));
    assert!(CSS.contains(".dark .noor-note"));
}
