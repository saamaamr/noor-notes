use noor_notes::editor::{
    AdapterCapabilities, AutosaveController, EditorAdapter, EditorSession, SavePhase, SearchOptions,
};

#[derive(Default)]
struct FakeAdapter {
    text: String,
    cursor: usize,
    selection: Option<(usize, usize)>,
    undo: Vec<String>,
    redo: Vec<String>,
}

impl EditorAdapter for FakeAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::all()
    }
    fn text(&self) -> String {
        self.text.clone()
    }
    fn replace_text(&mut self, text: String, cursor: usize) {
        self.undo.push(self.text.clone());
        self.text = text;
        self.cursor = cursor;
        self.redo.clear();
    }
    fn cursor_offset(&self) -> usize {
        self.cursor
    }
    fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }
    fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    fn undo(&mut self) {
        if let Some(value) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.text, value));
            self.cursor = self.text.chars().count();
        }
    }
    fn redo(&mut self) {
        if let Some(value) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.text, value));
            self.cursor = self.text.chars().count();
        }
    }
    fn select_range(&mut self, start: usize, end: usize) {
        self.selection = Some((start, end));
        self.cursor = end;
    }
}

#[test]
fn five_step_undo_and_redo_are_lossless() {
    let mut session = EditorSession::new(FakeAdapter::default());
    for value in ["one", "two", "three", "four", "five"] {
        session.replace_text(value.into(), value.len());
    }
    for expected in ["four", "three", "two", "one", ""] {
        session.undo();
        assert_eq!(session.text(), expected);
    }
    for expected in ["one", "two", "three", "four", "five"] {
        session.redo();
        assert_eq!(session.text(), expected);
    }
}

#[test]
fn find_replace_and_unicode_statistics_share_one_session() {
    let adapter = FakeAdapter {
        text: "Rust rust\nবাংলা rust".into(),
        cursor: 11,
        ..FakeAdapter::default()
    };
    let mut session = EditorSession::new(adapter);
    session.search(
        "rust",
        SearchOptions {
            match_case: false,
            whole_word: true,
        },
    );
    assert_eq!(session.search_position(), Some((1, 3)));
    session.find_next();
    assert_eq!(session.search_position(), Some((2, 3)));
    assert!(session.replace_current("GTK"));
    assert_eq!(session.text(), "Rust GTK\nবাংলা rust");
    assert_eq!(session.replace_all("note"), 2);
    assert_eq!(session.text(), "note GTK\nবাংলা note");
    let statistics = session.statistics(125);
    assert_eq!(statistics.lines, 2);
    assert_eq!(statistics.words, 4);
    assert_eq!(statistics.characters, 19);
    assert_eq!(statistics.zoom, 125);
}

#[test]
fn save_transitions_never_hide_failure_or_dirty_state() {
    let mut autosave = AutosaveController::default();
    assert_eq!(autosave.phase(), &SavePhase::Saved);
    let first = autosave.mark_dirty();
    assert_eq!(autosave.phase(), &SavePhase::Unsaved);
    autosave.begin_save(first);
    assert_eq!(autosave.phase(), &SavePhase::Saving);
    autosave.fail(first, "disk full");
    assert_eq!(autosave.phase(), &SavePhase::Failed("disk full".into()));
    let second = autosave.mark_dirty();
    autosave.begin_save(second);
    autosave.finish(second);
    assert_eq!(autosave.phase(), &SavePhase::Saved);
    autosave.fail(first, "stale");
    assert_eq!(autosave.phase(), &SavePhase::Saved);
}

#[test]
fn unsupported_capabilities_are_reported_to_the_ui() {
    #[derive(Default)]
    struct ReadOnly(FakeAdapter);
    impl EditorAdapter for ReadOnly {
        fn capabilities(&self) -> AdapterCapabilities {
            AdapterCapabilities::default()
        }
        fn text(&self) -> String {
            self.0.text()
        }
        fn replace_text(&mut self, _: String, _: usize) {}
        fn cursor_offset(&self) -> usize {
            0
        }
        fn selection(&self) -> Option<(usize, usize)> {
            None
        }
        fn can_undo(&self) -> bool {
            false
        }
        fn can_redo(&self) -> bool {
            false
        }
        fn undo(&mut self) {}
        fn redo(&mut self) {}
        fn select_range(&mut self, _: usize, _: usize) {}
    }
    let session = EditorSession::new(ReadOnly::default());
    assert!(!session.capabilities().replace);
    assert!(!session.capabilities().formatting);
}
