#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AdapterCapabilities {
    pub undo: bool,
    pub redo: bool,
    pub find: bool,
    pub replace: bool,
    pub formatting: bool,
    pub line_numbers: bool,
    pub syntax_highlighting: bool,
}

impl AdapterCapabilities {
    pub const fn all() -> Self {
        Self {
            undo: true,
            redo: true,
            find: true,
            replace: true,
            formatting: true,
            line_numbers: true,
            syntax_highlighting: true,
        }
    }
}

pub trait EditorAdapter {
    fn capabilities(&self) -> AdapterCapabilities;
    fn text(&self) -> String;
    fn replace_text(&mut self, text: String, cursor: usize);
    fn cursor_offset(&self) -> usize;
    fn selection(&self) -> Option<(usize, usize)>;
    fn can_undo(&self) -> bool;
    fn can_redo(&self) -> bool;
    fn undo(&mut self);
    fn redo(&mut self);
    fn select_range(&mut self, start: usize, end: usize);
}
