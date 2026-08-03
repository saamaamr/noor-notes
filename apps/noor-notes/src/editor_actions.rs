use adw::prelude::*;

use crate::modern_toolbar::ModernToolbar;
use crate::rich_buffer::RichBuffer;

pub fn connect(toolbar: &ModernToolbar, buffer: &gtk::TextBuffer, editor: &gtk::TextView) {
    for (button, action) in [
        (&toolbar.bold, RichBuffer::bold as fn(&gtk::TextBuffer)),
        (&toolbar.italic, RichBuffer::italic),
        (&toolbar.underline, RichBuffer::underline),
        (&toolbar.strikethrough, RichBuffer::strikethrough),
    ] {
        let buffer = buffer.clone();
        let editor = editor.clone();
        button.connect_toggled(move |_| {
            action(&buffer);
            editor.grab_focus();
        });
    }
    for (button, prefix) in [(&toolbar.bullets, "• "), (&toolbar.numbered, "1. ")] {
        let buffer = buffer.clone();
        let editor = editor.clone();
        button.connect_toggled(move |button| {
            if button.is_active() {
                RichBuffer::insert_list_prefix(&buffer, prefix);
            }
            editor.grab_focus();
        });
    }
    for button in &toolbar.emoji_buttons {
        let buffer = buffer.clone();
        let editor = editor.clone();
        button.connect_clicked(move |button| {
            if let Some(emoji) = button.label() {
                RichBuffer::insert_emoji(&buffer, &emoji);
            }
            editor.grab_focus();
        });
    }
}
