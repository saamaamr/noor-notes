use adw::prelude::*;

use super::{AdapterCapabilities, EditorAdapter};
use crate::rich_buffer::RichBuffer;

#[derive(Clone)]
pub struct RichEditorAdapter {
    buffer: gtk::TextBuffer,
}

impl RichEditorAdapter {
    pub fn new(buffer: &gtk::TextBuffer) -> Self {
        Self {
            buffer: buffer.clone(),
        }
    }

    pub fn buffer(&self) -> &gtk::TextBuffer {
        &self.buffer
    }
}

impl EditorAdapter for RichEditorAdapter {
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            undo: true,
            redo: true,
            find: true,
            replace: true,
            formatting: true,
            line_numbers: false,
            syntax_highlighting: false,
        }
    }

    fn text(&self) -> String {
        self.buffer
            .text(&self.buffer.start_iter(), &self.buffer.end_iter(), true)
            .to_string()
    }

    fn replace_text(&mut self, text: String, cursor: usize) {
        self.buffer.begin_user_action();
        self.buffer.set_text(&text);
        self.buffer
            .place_cursor(&self.buffer.iter_at_offset(cursor as i32));
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
        RichBuffer::can_undo(&self.buffer)
    }
    fn can_redo(&self) -> bool {
        RichBuffer::can_redo(&self.buffer)
    }
    fn undo(&mut self) {
        RichBuffer::undo(&self.buffer);
    }
    fn redo(&mut self) {
        RichBuffer::redo(&self.buffer);
    }

    fn select_range(&mut self, start: usize, end: usize) {
        self.buffer.select_range(
            &self.buffer.iter_at_offset(start as i32),
            &self.buffer.iter_at_offset(end as i32),
        );
    }
}
