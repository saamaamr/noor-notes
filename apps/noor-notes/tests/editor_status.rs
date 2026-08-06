use noor_notes::editor_status::{EditorStatistics, clamp_zoom, line_offset};

#[test]
fn statistics_report_unicode_words_lines_and_selection() {
    let stats = EditorStatistics::calculate("Hello বাংলা\nsecond line", 8, Some((6, 11)), 125);
    assert_eq!(stats.lines, 2);
    assert_eq!(stats.words, 4);
    assert_eq!(stats.characters, 23);
    assert_eq!(stats.line, 1);
    assert_eq!(stats.column, 9);
    assert_eq!(stats.selection, 5);
    assert_eq!(stats.zoom, 125);
}

#[test]
fn zoom_and_line_navigation_are_bounded() {
    assert_eq!(clamp_zoom(10), 50);
    assert_eq!(clamp_zoom(500), 300);
    assert_eq!(clamp_zoom(100), 100);
    assert_eq!(line_offset("first\nবাংলা\nlast", 1), 0);
    assert_eq!(line_offset("first\nবাংলা\nlast", 2), 6);
    assert_eq!(line_offset("first\nবাংলা\nlast", 99), 12);
}
