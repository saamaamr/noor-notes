use adw::prelude::*;
use noor_domain::Alignment;

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
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        toolbar.font_size.connect_selected_notify(move |dropdown| {
            let sizes = [12, 14, 16, 18, 24];
            if let Some(size) = sizes.get(dropdown.selected() as usize) {
                RichBuffer::font_size(&buffer, *size);
            }
            editor.grab_focus();
        });
    }
    for (button, color) in toolbar
        .foreground_buttons
        .iter()
        .zip(["charcoal", "blue", "green", "red"])
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        button.connect_clicked(move |_| {
            RichBuffer::foreground(&buffer, color);
            editor.grab_focus();
        });
    }
    for (button, color) in toolbar
        .highlight_buttons
        .iter()
        .zip(["charcoal", "blue", "green", "red"])
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        button.connect_clicked(move |_| {
            RichBuffer::highlight(&buffer, color);
            editor.grab_focus();
        });
    }
    for (button, alignment) in toolbar.alignment_buttons.iter().zip([
        Alignment::Start,
        Alignment::Center,
        Alignment::End,
        Alignment::Justify,
    ]) {
        let editor = editor.clone();
        button.connect_toggled(move |button| {
            if button.is_active() {
                RichBuffer::align(&editor.buffer(), alignment);
            }
            editor.grab_focus();
        });
    }

    let shortcuts = gtk::EventControllerKey::new();
    let shortcut_buffer = buffer.clone();
    shortcuts.connect_key_pressed(move |_, key, _, state| {
        if !state.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            return gtk::glib::Propagation::Proceed;
        }
        let handled = if key == gtk::gdk::Key::b {
            RichBuffer::bold(&shortcut_buffer);
            true
        } else if key == gtk::gdk::Key::i {
            RichBuffer::italic(&shortcut_buffer);
            true
        } else if key == gtk::gdk::Key::u {
            RichBuffer::underline(&shortcut_buffer);
            true
        } else {
            false
        };
        if handled {
            gtk::glib::Propagation::Stop
        } else {
            gtk::glib::Propagation::Proceed
        }
    });
    editor.add_controller(shortcuts);

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
