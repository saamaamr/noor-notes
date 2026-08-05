use adw::prelude::*;
use noor_domain::{Alignment, ListKind};

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
    for (button, kind) in [
        (&toolbar.bullets, ListKind::Bullet),
        (&toolbar.numbered, ListKind::Numbered),
    ] {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let bullets = toolbar.bullets.clone();
        let numbered = toolbar.numbered.clone();
        button.connect_clicked(move |_| {
            RichBuffer::toggle_list(&buffer, kind);
            sync_list_buttons(&buffer, &bullets, &numbered);
            editor.grab_focus();
        });
    }
    {
        let bullets = toolbar.bullets.clone();
        let numbered = toolbar.numbered.clone();
        buffer.connect_mark_set(move |buffer, _, _| sync_list_buttons(buffer, &bullets, &numbered));
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
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let entry = toolbar.custom_font_size.clone();
        toolbar.apply_font_size.connect_clicked(move |_| {
            if let Some(size) = RichBuffer::parse_font_size(&entry.text()) {
                RichBuffer::font_size(&buffer, size);
                entry.remove_css_class("error");
                editor.grab_focus();
            } else {
                entry.add_css_class("error");
            }
        });
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        toolbar.custom_font_size.connect_activate(move |entry| {
            if let Some(size) = RichBuffer::parse_font_size(&entry.text()) {
                RichBuffer::font_size(&buffer, size);
                entry.remove_css_class("error");
                editor.grab_focus();
            } else {
                entry.add_css_class("error");
            }
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
        if key == gtk::gdk::Key::Return && !state.contains(gtk::gdk::ModifierType::SHIFT_MASK) {
            return if RichBuffer::continue_list(&shortcut_buffer) {
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            };
        }
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

fn sync_list_buttons(
    buffer: &gtk::TextBuffer,
    bullets: &gtk::ToggleButton,
    numbered: &gtk::ToggleButton,
) {
    let kind = RichBuffer::list_kind_at_cursor(buffer);

    bullets.set_active(kind == Some(ListKind::Bullet));

    numbered.set_active(kind == Some(ListKind::Numbered));
}
