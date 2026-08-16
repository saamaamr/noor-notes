use noor_domain::SourceLanguage;
use noor_notes::{
    appearance::EffectiveTheme,
    editor::{EditorAdapter, SourceEditorAdapter, available_language_ids},
};
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
    source_editor_applies_theme_without_losing_text_selection_or_undo();
    plain_text_has_no_language_while_markdown_and_code_keep_theirs();
    rich_editor_uses_source_widgets_without_source_presentation();
}

fn rich_editor_uses_source_widgets_without_source_presentation() {
    let editor = SourceEditorAdapter::new_rich("hello", EffectiveTheme::Light);

    assert!(!editor.buffer().is_highlight_syntax());
    assert!(editor.buffer().style_scheme().is_some());
    assert!(!editor.view().shows_line_numbers());
    assert!(!editor.view().is_highlight_current_line());
    let gtk_buffer: gtk::TextBuffer = editor.buffer().clone().upcast();
    assert_eq!(
        gtk_buffer.text(&gtk_buffer.start_iter(), &gtk_buffer.end_iter(), true),
        "hello"
    );
}

fn source_editor_applies_theme_without_losing_text_selection_or_undo() {
    let mut editor = SourceEditorAdapter::new_with_theme(
        "hello বাংলা",
        Some(&SourceLanguage::Markdown),
        EffectiveTheme::Light,
    );
    editor.replace_text("hello বাংলা!".into(), 12);
    editor.select_range(0, 5);

    editor.apply_theme(EffectiveTheme::Midnight);

    assert_eq!(editor.text(), "hello বাংলা!");
    assert_eq!(editor.selection(), Some((0, 5)));
    assert!(editor.can_undo());
    assert_eq!(
        editor.buffer().style_scheme().unwrap().id().as_str(),
        "noor-midnight"
    );
}

fn plain_text_has_no_language_while_markdown_and_code_keep_theirs() {
    let plain = SourceEditorAdapter::new_with_theme("let value = 1;", None, EffectiveTheme::Light);
    let markdown = SourceEditorAdapter::new_with_theme(
        "# Heading",
        Some(&SourceLanguage::Markdown),
        EffectiveTheme::Graphite,
    );
    let rust = SourceLanguage::new("rust").unwrap();
    let code =
        SourceEditorAdapter::new_with_theme("fn main() {}", Some(&rust), EffectiveTheme::Oled);

    assert!(plain.buffer().language().is_none());
    assert_eq!(
        markdown.buffer().language().unwrap().id().as_str(),
        "markdown"
    );
    assert_eq!(code.buffer().language().unwrap().id().as_str(), "rust");
    assert_eq!(
        plain.buffer().style_scheme().unwrap().id().as_str(),
        "noor-light"
    );
    assert_eq!(
        markdown.buffer().style_scheme().unwrap().id().as_str(),
        "noor-graphite"
    );
    assert_eq!(
        code.buffer().style_scheme().unwrap().id().as_str(),
        "noor-oled"
    );
}
