use adw::prelude::*;
use noor_domain::{Alignment, ListKind};

use crate::editor_commands::{EditorCommand, execute};
use crate::rich_buffer::RichBuffer;
use crate::rich_color::{ColorRole, presets};
use crate::ui::editor_toolbar::EditorToolbar;
use crate::ui::rich_color_palette::RichColorPalette;

pub fn connect(toolbar: &EditorToolbar, buffer: &gtk::TextBuffer, editor: &gtk::TextView) {
    let editable = toolbar.edit_state();
    let syncing = std::rc::Rc::new(std::cell::Cell::new(false));
    {
        let undo_buffer = buffer.clone();
        let undo_editable = editable.clone();
        toolbar.undo.connect_clicked(move |_| {
            if undo_editable.get() {
                execute(EditorCommand::Undo, &undo_buffer, None);
            }
        });
        let redo_buffer = buffer.clone();
        let redo_editable = editable.clone();
        toolbar.redo.connect_clicked(move |_| {
            if redo_editable.get() {
                execute(EditorCommand::Redo, &redo_buffer, None);
            }
        });
        let undo = toolbar.undo.clone();
        let editable_state = editable.clone();
        buffer.connect_can_undo_notify(move |buffer| {
            undo.set_sensitive(editable_state.get() && buffer.can_undo())
        });
        let redo = toolbar.redo.clone();
        let editable_state = editable.clone();
        buffer.connect_can_redo_notify(move |buffer| {
            redo.set_sensitive(editable_state.get() && buffer.can_redo())
        });
    }
    for (button, action) in [
        (&toolbar.bold, EditorCommand::Bold),
        (&toolbar.italic, EditorCommand::Italic),
        (&toolbar.underline, EditorCommand::Underline),
        (&toolbar.strikethrough, EditorCommand::Strikethrough),
        (&toolbar.quick_underline, EditorCommand::Underline),
        (&toolbar.quick_strikethrough, EditorCommand::Strikethrough),
    ] {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let toolbar = toolbar.clone();
        let editable = editable.clone();
        let syncing = syncing.clone();
        button.connect_toggled(move |_| {
            if syncing.get() || !editable.get() {
                return;
            }
            execute(action, &buffer, None);
            sync_format_buttons(&buffer, &toolbar, &syncing);
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
        let editable = editable.clone();
        let syncing = syncing.clone();
        button.connect_clicked(move |_| {
            if syncing.get() || !editable.get() {
                return;
            }
            execute(
                if kind == ListKind::Bullet {
                    EditorCommand::ToggleBulletList
                } else {
                    EditorCommand::ToggleNumberedList
                },
                &buffer,
                None,
            );
            sync_list_buttons(&buffer, &bullets, &numbered);
            syncing.set(true);
            quick_numbered.set_active(numbered.is_active());
            syncing.set(false);
            editor.grab_focus();
        });
    }
    {
        let bullets = toolbar.bullets.clone();
        let numbered = toolbar.numbered.clone();
        let quick_numbered = toolbar.quick_numbered.clone();
        let list_syncing = syncing.clone();
        buffer.connect_mark_set(move |buffer, _, _| {
            sync_list_buttons(buffer, &bullets, &numbered);
            list_syncing.set(true);
            quick_numbered.set_active(numbered.is_active());
            list_syncing.set(false);
        });
        let toolbar = toolbar.clone();
        let format_syncing = syncing.clone();
        buffer.connect_mark_set(move |buffer, _, _| {
            sync_format_buttons(buffer, &toolbar, &format_syncing);
        });
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let syncing = syncing.clone();
        toolbar.font_size.connect_selected_notify(move |dropdown| {
            if syncing.get() || !editable.get() {
                return;
            }
            let sizes = [12, 14, 16, 18, 24];
            if let Some(size) = sizes.get(dropdown.selected() as usize) {
                execute(EditorCommand::FontSize, &buffer, Some(&size.to_string()));
            }
            editor.grab_focus();
        });
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let syncing = syncing.clone();
        toolbar
            .quick_font_size
            .connect_selected_notify(move |dropdown| {
                if syncing.get() || !editable.get() {
                    return;
                }
                let sizes = [12, 14, 16, 18, 24];
                if let Some(size) = sizes.get(dropdown.selected() as usize) {
                    execute(EditorCommand::FontSize, &buffer, Some(&size.to_string()));
                }
                editor.grab_focus();
            });
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let entry = toolbar.custom_font_size.clone();
        toolbar.apply_font_size.connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
            if let Some(size) = RichBuffer::parse_font_size(&entry.text()) {
                execute(EditorCommand::FontSize, &buffer, Some(&size.to_string()));
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
        let editable = editable.clone();
        toolbar.custom_font_size.connect_activate(move |entry| {
            if !editable.get() {
                return;
            }
            if let Some(size) = RichBuffer::parse_font_size(&entry.text()) {
                execute(EditorCommand::FontSize, &buffer, Some(&size.to_string()));
                entry.remove_css_class("error");
                editor.grab_focus();
            } else {
                entry.add_css_class("error");
            }
        });
    }
    connect_color_palette(&toolbar.foreground_palette, buffer, editor, &editable);
    connect_color_palette(&toolbar.highlight_palette, buffer, editor, &editable);
    for (button, alignment) in toolbar.alignment_buttons.iter().zip([
        Alignment::Start,
        Alignment::Center,
        Alignment::End,
        Alignment::Justify,
    ]) {
        let editor = editor.clone();
        let editable = editable.clone();
        button.connect_toggled(move |button| {
            if button.is_active() && editable.get() {
                RichBuffer::align(&editor.buffer(), alignment);
            }
            editor.grab_focus();
        });
    }

    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        toolbar.clear_formatting.connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
            execute(EditorCommand::ClearFormatting, &buffer, None);
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
    let editable_for_keys = editable.clone();
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
            return if editable_for_keys.get() && RichBuffer::continue_list(&shortcut_buffer) {
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
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            execute(EditorCommand::Redo, &shortcut_buffer, None);
            true
        } else if key == gtk::gdk::Key::z {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            execute(EditorCommand::Undo, &shortcut_buffer, None);
            true
        } else if key == gtk::gdk::Key::y {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            execute(EditorCommand::Redo, &shortcut_buffer, None);
            true
        } else if key == gtk::gdk::Key::b {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            execute(EditorCommand::Bold, &shortcut_buffer, None);
            true
        } else if key == gtk::gdk::Key::i {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            execute(EditorCommand::Italic, &shortcut_buffer, None);
            true
        } else if key == gtk::gdk::Key::u {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            execute(EditorCommand::Underline, &shortcut_buffer, None);
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
        let editable = editable.clone();
        button.connect_clicked(move |button| {
            if !editable.get() {
                return;
            }
            if let Some(emoji) = button.label() {
                execute(EditorCommand::InsertEmoji, &buffer, Some(&emoji));
            }
            editor.grab_focus();
        });
    }
}

fn connect_color_palette(
    palette: &RichColorPalette,
    buffer: &gtk::TextBuffer,
    editor: &gtk::TextView,
    editable: &std::rc::Rc<std::cell::Cell<bool>>,
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
        let editable = editable.clone();
        let color = preset.id;
        button.connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
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
        let editable = editable.clone();
        palette.reset.clone().connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
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
        let editable = editable.clone();
        palette.custom.clone().connect_rgba_notify(move |button| {
            if !editable.get() {
                return;
            }
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

fn sync_format_buttons(
    buffer: &gtk::TextBuffer,
    toolbar: &EditorToolbar,
    syncing: &std::rc::Rc<std::cell::Cell<bool>>,
) {
    let marks = RichBuffer::marks_at_cursor(buffer);
    syncing.set(true);
    toolbar.bold.set_active(marks.bold);
    toolbar.italic.set_active(marks.italic);
    toolbar.underline.set_active(marks.underline);
    toolbar.quick_underline.set_active(marks.underline);
    toolbar.strikethrough.set_active(marks.strikethrough);
    toolbar.quick_strikethrough.set_active(marks.strikethrough);
    if let Some(size) = marks.font_size {
        let selected = [12, 14, 16, 18, 24]
            .iter()
            .position(|candidate| *candidate == size)
            .unwrap_or(2) as u32;
        toolbar.font_size.set_selected(selected);
        toolbar.quick_font_size.set_selected(selected);
    }
    syncing.set(false);
}
