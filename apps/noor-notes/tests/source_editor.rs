use noor_domain::SourceLanguage;
use noor_notes::editor::{EditorAdapter, SourceEditorAdapter, available_language_ids};
use sourceview5::prelude::*;

#[test]
fn source_editor_supports_languages_regex_unicode_lines_and_bookmarks() {
    gtk::init().unwrap();
    let mut editor = SourceEditorAdapter::new(
        "# বাংলা\nlet answer = 42;\nlet second = 42;",
        &SourceLanguage::Markdown,
    );
    assert!(editor.capabilities().syntax_highlighting);
    assert!(editor.view().shows_line_numbers());
    assert!(editor.view().is_highlight_current_line());
    assert_eq!(
        editor.buffer().language().unwrap().id().as_str(),
        "markdown"
    );

    editor.configure_search(r"let\s+\w+", true, false, true);
    assert!(editor.search_settings().is_regex_enabled());
    assert!(editor.search_settings().is_case_sensitive());
    assert_eq!(
        editor.search_settings().search_text().as_deref(),
        Some(r"let\s+\w+")
    );
    while gtk::glib::MainContext::default().iteration(false) {}
    assert!(editor.search_context().regex_error().is_none());
    assert_eq!(editor.search_context().occurrences_count(), 2);

    assert!(editor.set_bookmark(1));
    assert!(!editor.set_bookmark(99));
    assert!(editor.set_language(&SourceLanguage::new("rust").unwrap()));
    assert_eq!(editor.buffer().language().unwrap().id().as_str(), "rust");
    assert!(!editor.set_language(&SourceLanguage::new("not-a-language").unwrap()));

    editor.replace_text("αβγ\nবাংলা".into(), 4);
    assert_eq!(editor.text(), "αβγ\nবাংলা");
    assert_eq!(editor.cursor_offset(), 4);
    editor.undo();
    assert!(editor.text().contains("answer"));
    editor.redo();
    assert_eq!(editor.text(), "αβγ\nবাংলা");

    let manager = sourceview5::LanguageManager::default();
    let ids = available_language_ids(&manager);
    assert!(ids.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(ids.contains(&"markdown".to_string()));
    assert!(ids.contains(&"python3".to_string()));
}
