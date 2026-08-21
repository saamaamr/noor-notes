use adw::prelude::*;
use noor_domain::SourceLanguage;
use sourceview5::prelude::*;

use crate::appearance::EffectiveTheme;

use super::{AdapterCapabilities, EditorAdapter, resolve_language, source_palette};

#[derive(Clone)]
pub struct SourceEditorAdapter {
    buffer: sourceview5::Buffer,
    view: sourceview5::View,
    search_settings: sourceview5::SearchSettings,
    search_context: sourceview5::SearchContext,
}

impl SourceEditorAdapter {
    pub fn new(text: &str, language: &SourceLanguage) -> Self {
        Self::new_with_theme(text, Some(language), EffectiveTheme::Light)
    }

    pub fn new_rich(text: &str, theme: EffectiveTheme) -> Self {
        let editor = Self::new_with_theme(text, None, theme);
        editor.buffer.set_highlight_syntax(false);
        editor.buffer.set_highlight_matching_brackets(false);
        editor.view.set_show_line_numbers(false);
        editor.view.set_highlight_current_line(false);
        editor.view.set_auto_indent(false);
        editor
    }

    pub fn new_with_theme(
        text: &str,
        language: Option<&SourceLanguage>,
        theme: EffectiveTheme,
    ) -> Self {
        let manager = sourceview5::LanguageManager::default();
        let buffer = sourceview5::Buffer::builder()
            .text(text)
            .enable_undo(true)
            .highlight_syntax(true)
            .highlight_matching_brackets(true)
            .build();
        crate::rich_buffer::RichBuffer::prepare(&buffer.clone().upcast::<gtk::TextBuffer>());
        if let Some(language) = language.and_then(|language| resolve_language(&manager, language)) {
            buffer.set_language(Some(&language));
        }
        source_palette::apply(&buffer, theme);
        let view = sourceview5::View::with_buffer(&buffer);
        view.add_css_class("nn-writing-canvas");
        view.add_css_class("nn-source-canvas");
        view.set_show_line_numbers(true);
        view.set_highlight_current_line(true);
        view.set_auto_indent(true);
        view.set_insert_spaces_instead_of_tabs(true);
        view.set_tab_width(4);
        view.set_wrap_mode(gtk::WrapMode::WordChar);
        let search_settings = sourceview5::SearchSettings::new();
        search_settings.set_wrap_around(true);
        let search_context = sourceview5::SearchContext::new(&buffer, Some(&search_settings));
        search_context.set_highlight(true);
        Self {
            buffer,
            view,
            search_settings,
            search_context,
        }
    }

    pub fn view(&self) -> &sourceview5::View {
        &self.view
    }

    pub fn buffer(&self) -> &sourceview5::Buffer {
        &self.buffer
    }

    pub fn configure_search(&self, query: &str, match_case: bool, whole_word: bool, regex: bool) {
        self.search_settings.set_case_sensitive(match_case);
        self.search_settings.set_at_word_boundaries(whole_word);
        self.search_settings.set_regex_enabled(regex);
        self.search_settings
            .set_search_text((!query.is_empty()).then_some(query));
    }

    pub fn search_settings(&self) -> &sourceview5::SearchSettings {
        &self.search_settings
    }

    pub fn search_context(&self) -> &sourceview5::SearchContext {
        &self.search_context
    }

    pub fn apply_theme(&self, theme: EffectiveTheme) {
        source_palette::apply(&self.buffer, theme);
    }

    pub fn set_language(&self, language: &SourceLanguage) -> bool {
        let manager = sourceview5::LanguageManager::default();
        let Some(language) = resolve_language(&manager, language) else {
            return false;
        };
        self.buffer.set_language(Some(&language));
        true
    }

    pub fn set_bookmark(&self, line: u32) -> bool {
        let Some(iter) = self.buffer.iter_at_line(line as i32) else {
            return false;
        };
        self.buffer.create_source_mark(None, "noor-bookmark", &iter);
        true
    }
}

impl EditorAdapter for SourceEditorAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::all()
    }

    fn text(&self) -> String {
        self.buffer
            .text(&self.buffer.start_iter(), &self.buffer.end_iter(), true)
            .to_string()
    }

    fn replace_text(&mut self, text: String, cursor: usize) {
        self.buffer.begin_user_action();
        let mut start = self.buffer.start_iter();
        let mut end = self.buffer.end_iter();
        self.buffer.delete(&mut start, &mut end);
        self.buffer.insert(&mut start, &text);
        self.buffer.place_cursor(
            &self
                .buffer
                .iter_at_offset(cursor.min(text.chars().count()) as i32),
        );
        self.buffer.end_user_action();
    }

    fn cursor_offset(&self) -> usize {
        self.buffer
            .iter_at_mark(&self.buffer.get_insert())
            .offset()
            .max(0) as usize
    }

    fn selection(&self) -> Option<(usize, usize)> {
        self.buffer
            .selection_bounds()
            .map(|(start, end)| (start.offset() as usize, end.offset() as usize))
    }

    fn can_undo(&self) -> bool {
        self.buffer.can_undo()
    }
    fn can_redo(&self) -> bool {
        self.buffer.can_redo()
    }
    fn undo(&mut self) {
        if self.buffer.can_undo() {
            self.buffer.undo();
        }
    }
    fn redo(&mut self) {
        if self.buffer.can_redo() {
            self.buffer.redo();
        }
    }

    fn select_range(&mut self, start: usize, end: usize) {
        self.buffer.select_range(
            &self.buffer.iter_at_offset(start as i32),
            &self.buffer.iter_at_offset(end as i32),
        );
    }
}
