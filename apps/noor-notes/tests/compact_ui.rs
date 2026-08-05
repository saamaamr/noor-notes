const CSS: &str = include_str!("../resources/modern.css");

#[test]
fn note_chrome_uses_compact_approved_dimensions() {
    assert!(CSS.contains("min-height: 28px"));
    assert!(CSS.contains("min-width: 28px"));
    assert!(CSS.contains("-gtk-icon-size: 12px"));
    assert!(CSS.contains("border-radius: 3px 3px 0 0"));
}
