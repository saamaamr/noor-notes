use crate::editor_status::EditorStatistics;
use crate::note_find::{FindResults, replace_all, replace_current};

use super::{AdapterCapabilities, EditorAdapter, SearchOptions};

pub struct EditorSession<A: EditorAdapter> {
    adapter: A,
    query: String,
    options: SearchOptions,
    results: FindResults,
}

impl<A: EditorAdapter> EditorSession<A> {
    pub fn new(adapter: A) -> Self {
        Self {
            adapter,
            query: String::new(),
            options: SearchOptions::default(),
            results: FindResults::default(),
        }
    }

    pub fn capabilities(&self) -> AdapterCapabilities {
        self.adapter.capabilities()
    }

    pub fn text(&self) -> String {
        self.adapter.text()
    }

    pub fn replace_text(&mut self, text: String, cursor: usize) {
        self.adapter.replace_text(text, cursor);
        self.refresh_search();
    }

    pub fn can_undo(&self) -> bool {
        self.adapter.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.adapter.can_redo()
    }

    pub fn undo(&mut self) {
        if self.capabilities().undo && self.adapter.can_undo() {
            self.adapter.undo();
            self.refresh_search();
        }
    }

    pub fn redo(&mut self) {
        if self.capabilities().redo && self.adapter.can_redo() {
            self.adapter.redo();
            self.refresh_search();
        }
    }

    pub fn search(&mut self, query: impl Into<String>, options: SearchOptions) {
        self.query = query.into();
        self.options = options;
        self.refresh_search();
        self.select_current();
    }

    pub fn search_position(&self) -> Option<(usize, usize)> {
        self.results.position()
    }

    pub fn find_next(&mut self) {
        self.results.next();
        self.select_current();
    }

    pub fn find_previous(&mut self) {
        self.results.previous();
        self.select_current();
    }

    pub fn replace_current(&mut self, replacement: &str) -> bool {
        let Some((text, cursor)) =
            replace_current(&self.adapter.text(), &self.results, replacement)
        else {
            return false;
        };
        self.adapter.replace_text(text, cursor);
        self.refresh_search();
        self.select_current();
        true
    }

    pub fn replace_all(&mut self, replacement: &str) -> usize {
        let (text, count) =
            replace_all(&self.adapter.text(), &self.query, replacement, self.options);
        if count > 0 {
            let cursor = text.chars().count();
            self.adapter.replace_text(text, cursor);
            self.refresh_search();
        }
        count
    }

    pub fn statistics(&self, zoom: u16) -> EditorStatistics {
        EditorStatistics::calculate(
            &self.adapter.text(),
            self.adapter.cursor_offset(),
            self.adapter.selection(),
            zoom,
        )
    }

    fn refresh_search(&mut self) {
        self.results
            .update_with_options(&self.adapter.text(), &self.query, self.options);
    }

    fn select_current(&mut self) {
        if let Some((start, end)) = self.results.current_range() {
            self.adapter.select_range(start, end);
        }
    }
}
