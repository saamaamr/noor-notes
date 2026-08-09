use noor_notes::editor_status::{EditorStatistics, clamp_zoom, line_offset};
use noor_notes::ui::editor_status_bar::EditorStatusBar;

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
fn status_bar_presents_live_statistics_and_mode_without_fake_fields() {
    gtk::init().unwrap();
    let status = EditorStatusBar::new("Rich Text");
    status.update_statistics(EditorStatistics {
        line: 2,
        column: 4,
        lines: 3,
        words: 9,
        characters: 42,
        selection: 5,
        zoom: 125,
    });
    assert_eq!(
        status.statistics.text(),
        "Ln 2, Col 4  ·  3 lines  ·  9 words  ·  42 characters  ·  5 selected  ·  125%"
    );
    assert_eq!(status.mode.text(), "Rich Text  ·  UTF-8");
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
