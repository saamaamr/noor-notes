use adw::prelude::*;
use noor_domain::{Alignment, ListKind};

use crate::rich_buffer::RichBuffer;
use crate::rich_color::{ColorRole, presets};
use crate::ui::editor_toolbar::EditorToolbar;
use crate::ui::rich_color_palette::RichColorPalette;

pub fn connect(toolbar: &EditorToolbar, buffer: &gtk::TextBuffer, editor: &gtk::TextView) {
    {
        let undo_buffer = buffer.clone();
        toolbar
            .undo
            .connect_clicked(move |_| RichBuffer::undo(&undo_buffer));
        let redo_buffer = buffer.clone();
        toolbar
            .redo
            .connect_clicked(move |_| RichBuffer::redo(&redo_buffer));
        let undo = toolbar.undo.clone();
        buffer.connect_can_undo_notify(move |buffer| undo.set_sensitive(buffer.can_undo()));
        let redo = toolbar.redo.clone();
        buffer.connect_can_redo_notify(move |buffer| redo.set_sensitive(buffer.can_redo()));
    }
    for (button, action) in [
        (&toolbar.bold, RichBuffer::bold as fn(&gtk::TextBuffer)),
        (&toolbar.italic, RichBuffer::italic),
        (&toolbar.underline, RichBuffer::underline),
        (&toolbar.strikethrough, RichBuffer::strikethrough),
        (&toolbar.quick_underline, RichBuffer::underline),
        (&toolbar.quick_strikethrough, RichBuffer::strikethrough),
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
        (&toolbar.quick_numbered, ListKind::Numbered),
    ] {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let bullets = toolbar.bullets.clone();
        let numbered = toolbar.numbered.clone();
        let quick_numbered = toolbar.quick_numbered.clone();
        button.connect_clicked(move |_| {
            RichBuffer::toggle_list(&buffer, kind);
            sync_list_buttons(&buffer, &bullets, &numbered);
            quick_numbered.set_active(numbered.is_active());
            editor.grab_focus();
        });
    }
    {
        let bullets = toolbar.bullets.clone();
        let numbered = toolbar.numbered.clone();
        let quick_numbered = toolbar.quick_numbered.clone();
        buffer.connect_mark_set(move |buffer, _, _| {
            sync_list_buttons(buffer, &bullets, &numbered);
            quick_numbered.set_active(numbered.is_active());
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
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        toolbar
            .quick_font_size
            .connect_selected_notify(move |dropdown| {
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
    connect_color_palette(&toolbar.foreground_palette, buffer, editor);
    connect_color_palette(&toolbar.highlight_palette, buffer, editor);
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

    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        toolbar.clear_formatting.connect_clicked(move |_| {
            RichBuffer::clear_formatting(&buffer);
            editor.grab_focus();
        });
    }

    let shortcuts = gtk::EventControllerKey::new();
    let shortcut_buffer = buffer.clone();
    let find_button = toolbar.find.clone();
    let zoom_in = toolbar.zoom_in.clone();
    let zoom_out = toolbar.zoom_out.clone();
    let zoom_reset = toolbar.zoom_reset.clone();
    let go_to_line = toolbar.go_to_line.clone();
    let fullscreen = toolbar.fullscreen.clone();
    shortcuts.connect_key_pressed(move |_, key, _, state| {
        if key == gtk::gdk::Key::Escape && find_button.is_active() {
            find_button.set_active(false);
            return gtk::glib::Propagation::Stop;
        }
        if key == gtk::gdk::Key::F11 {
            fullscreen.set_active(!fullscreen.is_active());
            return gtk::glib::Propagation::Stop;
        }
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
        let handled = if key == gtk::gdk::Key::f || key == gtk::gdk::Key::h {
            find_button.set_active(true);
            true
        } else if key == gtk::gdk::Key::g {
            go_to_line.emit_clicked();
            true
        } else if key == gtk::gdk::Key::plus || key == gtk::gdk::Key::equal {
            zoom_in.emit_clicked();
            true
        } else if key == gtk::gdk::Key::minus {
            zoom_out.emit_clicked();
            true
        } else if key == gtk::gdk::Key::_0 {
            zoom_reset.emit_clicked();
            true
        } else if key == gtk::gdk::Key::z && state.contains(gtk::gdk::ModifierType::SHIFT_MASK) {
            RichBuffer::redo(&shortcut_buffer);
            true
        } else if key == gtk::gdk::Key::z {
            RichBuffer::undo(&shortcut_buffer);
            true
        } else if key == gtk::gdk::Key::y {
            RichBuffer::redo(&shortcut_buffer);
            true
        } else if key == gtk::gdk::Key::b {
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

fn connect_color_palette(
    palette: &RichColorPalette,
    buffer: &gtk::TextBuffer,
    editor: &gtk::TextView,
) {
    for (index, (button, preset)) in palette
        .preset_buttons
        .iter()
        .zip(presets(palette.role))
        .enumerate()
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let palette = palette.clone();
        let color = preset.id;
        button.connect_clicked(move |_| {
            if buffer.selection_bounds().is_none() {
                palette.clear_selection();
                editor.grab_focus();
                return;
            }
            match palette.role {
                ColorRole::Foreground => RichBuffer::foreground(&buffer, color),
                ColorRole::Highlight => RichBuffer::highlight(&buffer, color),
            }
            palette.select_preset(Some(index));
            editor.grab_focus();
        });
    }

    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let palette = palette.clone();
        palette.reset.clone().connect_clicked(move |_| {
            if buffer.selection_bounds().is_none() {
                palette.clear_selection();
                editor.grab_focus();
                return;
            }
            match palette.role {
                ColorRole::Foreground => RichBuffer::clear_foreground(&buffer),
                ColorRole::Highlight => RichBuffer::clear_highlight(&buffer),
            }
            palette.select_preset(None);
            editor.grab_focus();
        });
    }

    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let palette = palette.clone();
        palette.custom.clone().connect_rgba_notify(move |button| {
            if buffer.selection_bounds().is_none() {
                editor.grab_focus();
                return;
            }
            let rgba = button.rgba();
            let component = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            let color = format!(
                "#{:02X}{:02X}{:02X}",
                component(rgba.red()),
                component(rgba.green()),
                component(rgba.blue())
            );
            match palette.role {
                ColorRole::Foreground => RichBuffer::foreground(&buffer, &color),
                ColorRole::Highlight => RichBuffer::highlight(&buffer, &color),
            }
            palette.clear_selection();
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
