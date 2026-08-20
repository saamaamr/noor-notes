use adw::prelude::*;
use noor_domain::{Alignment, ListKind};

use crate::editor_commands::{EditorCommand, execute};
use crate::rich_buffer::{RichBuffer, SavedTextRange};
use crate::rich_color::{ColorRole, presets};
use crate::ui::editor_toolbar::EditorToolbar;
use crate::ui::rich_color_palette::RichColorPalette;

pub fn connect(toolbar: &EditorToolbar, buffer: &gtk::TextBuffer, editor: &gtk::TextView) {
    let editable = toolbar.edit_state();
    let syncing = std::rc::Rc::new(std::cell::Cell::new(false));
    let saved_range = std::rc::Rc::new(std::cell::RefCell::new(SavedTextRange::capture(buffer)));
    {
        let undo_buffer = buffer.clone();
        let undo_editor = editor.clone();
        let undo_editable = editable.clone();
        let undo_range = saved_range.clone();
        toolbar.undo.connect_clicked(move |_| {
            if undo_editable.get() {
                run_command(
                    EditorCommand::Undo,
                    &undo_buffer,
                    &undo_editor,
                    &undo_range,
                    None,
                );
            }
        });
        let redo_buffer = buffer.clone();
        let redo_editor = editor.clone();
        let redo_editable = editable.clone();
        let redo_range = saved_range.clone();
        toolbar.redo.connect_clicked(move |_| {
            if redo_editable.get() {
                run_command(
                    EditorCommand::Redo,
                    &redo_buffer,
                    &redo_editor,
                    &redo_range,
                    None,
                );
            }
        });
        let undo = toolbar.undo.clone();
        let redo = toolbar.redo.clone();
        let editable_state = editable.clone();
        buffer.connect_changed(move |buffer| {
            sync_history_buttons(buffer, &undo, &redo, editable_state.get());
        });
        sync_history_buttons(buffer, &toolbar.undo, &toolbar.redo, editable.get());
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
        let saved_range = saved_range.clone();
        button.connect_toggled(move |_| {
            if syncing.get() || !editable.get() {
                return;
            }
            run_command(action, &buffer, &editor, &saved_range, None);
            sync_editor_state(&buffer, &toolbar, &syncing);
        });
    }
    for (button, kind) in [
        (&toolbar.bullets, ListKind::Bullet),
        (&toolbar.formatting.bullets, ListKind::Bullet),
        (&toolbar.numbered, ListKind::Numbered),
        (&toolbar.quick_numbered, ListKind::Numbered),
    ] {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let state_toolbar = toolbar.clone();
        let editable = editable.clone();
        let syncing = syncing.clone();
        let saved_range = saved_range.clone();
        button.connect_clicked(move |_| {
            if syncing.get() || !editable.get() {
                return;
            }
            run_command(
                if kind == ListKind::Bullet {
                    EditorCommand::ToggleBulletList
                } else {
                    EditorCommand::ToggleNumberedList
                },
                &buffer,
                &editor,
                &saved_range,
                None,
            );
            syncing.set(true);
            sync_list_buttons(&buffer, &state_toolbar);
            syncing.set(false);
        });
    }
    {
        let mark_toolbar = toolbar.clone();
        let state_syncing = syncing.clone();
        let saved_range = saved_range.clone();
        buffer.connect_mark_set(move |buffer, _, _| {
            *saved_range.borrow_mut() = SavedTextRange::capture(buffer);
            sync_editor_state(buffer, &mark_toolbar, &state_syncing);
        });
        let changed_toolbar = toolbar.clone();
        let state_syncing = syncing.clone();
        buffer.connect_changed(move |buffer| {
            sync_editor_state(buffer, &changed_toolbar, &state_syncing);
        });
        sync_editor_state(buffer, toolbar, &syncing);
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let syncing = syncing.clone();
        let saved_range = saved_range.clone();
        toolbar.font_size.connect_selected_notify(move |dropdown| {
            if syncing.get() || !editable.get() {
                return;
            }
            let sizes = [12, 14, 16, 18, 24];
            if let Some(size) = sizes.get(dropdown.selected() as usize) {
                run_command(
                    EditorCommand::FontSize,
                    &buffer,
                    &editor,
                    &saved_range,
                    Some(&size.to_string()),
                );
            }
        });
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let syncing = syncing.clone();
        let saved_range = saved_range.clone();
        toolbar
            .quick_font_size
            .connect_selected_notify(move |dropdown| {
                if syncing.get() || !editable.get() {
                    return;
                }
                let sizes = [12, 14, 16, 18, 24];
                if let Some(size) = sizes.get(dropdown.selected() as usize) {
                    run_command(
                        EditorCommand::FontSize,
                        &buffer,
                        &editor,
                        &saved_range,
                        Some(&size.to_string()),
                    );
                }
            });
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let entry = toolbar.custom_font_size.clone();
        let saved_range = saved_range.clone();
        toolbar.apply_font_size.connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
            if let Some(size) = RichBuffer::parse_font_size(&entry.text()) {
                run_command(
                    EditorCommand::FontSize,
                    &buffer,
                    &editor,
                    &saved_range,
                    Some(&size.to_string()),
                );
                entry.remove_css_class("error");
            } else {
                entry.add_css_class("error");
            }
        });
    }
    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let saved_range = saved_range.clone();
        toolbar.custom_font_size.connect_activate(move |entry| {
            if !editable.get() {
                return;
            }
            if let Some(size) = RichBuffer::parse_font_size(&entry.text()) {
                run_command(
                    EditorCommand::FontSize,
                    &buffer,
                    &editor,
                    &saved_range,
                    Some(&size.to_string()),
                );
                entry.remove_css_class("error");
            } else {
                entry.add_css_class("error");
            }
        });
    }
    connect_color_palette(
        &toolbar.foreground_palette,
        buffer,
        editor,
        &editable,
        &saved_range,
    );
    connect_color_palette(
        &toolbar.highlight_palette,
        buffer,
        editor,
        &editable,
        &saved_range,
    );
    for (button, alignment) in toolbar.alignment_buttons.iter().zip([
        Alignment::Start,
        Alignment::Center,
        Alignment::End,
        Alignment::Justify,
    ]) {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let syncing = syncing.clone();
        let saved_range = saved_range.clone();
        button.connect_toggled(move |button| {
            if button.is_active() && editable.get() && !syncing.get() {
                with_saved_range(&buffer, &editor, &saved_range, |buffer| {
                    RichBuffer::align(buffer, alignment)
                });
            }
        });
    }

    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let editable = editable.clone();
        let saved_range = saved_range.clone();
        toolbar.clear_formatting.connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
            run_command(
                EditorCommand::ClearFormatting,
                &buffer,
                &editor,
                &saved_range,
                None,
            );
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
    let shortcut_editor = editor.clone();
    let shortcut_range = saved_range.clone();
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
            let handled = editable_for_keys.get()
                && with_saved_range(
                    &shortcut_buffer,
                    &shortcut_editor,
                    &shortcut_range,
                    RichBuffer::continue_list,
                );
            return if handled {
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
            run_command(
                EditorCommand::Redo,
                &shortcut_buffer,
                &shortcut_editor,
                &shortcut_range,
                None,
            );
            true
        } else if key == gtk::gdk::Key::z {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            run_command(
                EditorCommand::Undo,
                &shortcut_buffer,
                &shortcut_editor,
                &shortcut_range,
                None,
            );
            true
        } else if key == gtk::gdk::Key::y {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            run_command(
                EditorCommand::Redo,
                &shortcut_buffer,
                &shortcut_editor,
                &shortcut_range,
                None,
            );
            true
        } else if key == gtk::gdk::Key::b {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            run_command(
                EditorCommand::Bold,
                &shortcut_buffer,
                &shortcut_editor,
                &shortcut_range,
                None,
            );
            true
        } else if key == gtk::gdk::Key::i {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            run_command(
                EditorCommand::Italic,
                &shortcut_buffer,
                &shortcut_editor,
                &shortcut_range,
                None,
            );
            true
        } else if key == gtk::gdk::Key::u {
            if !editable_for_keys.get() {
                return gtk::glib::Propagation::Proceed;
            }
            run_command(
                EditorCommand::Underline,
                &shortcut_buffer,
                &shortcut_editor,
                &shortcut_range,
                None,
            );
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
        let emoji_menu = toolbar.emoji.clone();
        let editable = editable.clone();
        let saved_range = saved_range.clone();
        button.connect_clicked(move |button| {
            if !editable.get() {
                return;
            }
            if let Some(emoji) = button.label() {
                run_command(
                    EditorCommand::InsertEmoji,
                    &buffer,
                    &editor,
                    &saved_range,
                    Some(&emoji),
                );
                emoji_menu.popdown();
            }
        });
    }
}

fn connect_color_palette(
    palette: &RichColorPalette,
    buffer: &gtk::TextBuffer,
    editor: &gtk::TextView,
    editable: &std::rc::Rc<std::cell::Cell<bool>>,
    saved_range: &std::rc::Rc<std::cell::RefCell<SavedTextRange>>,
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
        let saved_range = saved_range.clone();
        let color = preset.id;
        button.connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
            with_saved_range(&buffer, &editor, &saved_range, |buffer| {
                if buffer.selection_bounds().is_none() {
                    palette.clear_selection();
                    return;
                }
                match palette.role {
                    ColorRole::Foreground => RichBuffer::foreground(buffer, color),
                    ColorRole::Highlight => RichBuffer::highlight(buffer, color),
                }
                palette.select_preset(Some(index));
            });
        });
    }

    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let palette = palette.clone();
        let editable = editable.clone();
        let saved_range = saved_range.clone();
        palette.reset.clone().connect_clicked(move |_| {
            if !editable.get() {
                return;
            }
            with_saved_range(&buffer, &editor, &saved_range, |buffer| {
                if buffer.selection_bounds().is_none() {
                    palette.clear_selection();
                    return;
                }
                match palette.role {
                    ColorRole::Foreground => RichBuffer::clear_foreground(buffer),
                    ColorRole::Highlight => RichBuffer::clear_highlight(buffer),
                }
                palette.select_preset(None);
            });
        });
    }

    {
        let buffer = buffer.clone();
        let editor = editor.clone();
        let palette = palette.clone();
        let editable = editable.clone();
        let saved_range = saved_range.clone();
        palette.custom.clone().connect_rgba_notify(move |button| {
            if !editable.get() {
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
            with_saved_range(&buffer, &editor, &saved_range, |buffer| {
                if buffer.selection_bounds().is_none() {
                    return;
                }
                match palette.role {
                    ColorRole::Foreground => RichBuffer::foreground(buffer, &color),
                    ColorRole::Highlight => RichBuffer::highlight(buffer, &color),
                }
                palette.clear_selection();
            });
        });
    }
}

fn sync_list_buttons(buffer: &gtk::TextBuffer, toolbar: &EditorToolbar) {
    let kind = RichBuffer::list_kind_for_selection(buffer);

    let bullets_active = kind == Some(ListKind::Bullet);
    let numbered_active = kind == Some(ListKind::Numbered);
    toolbar.bullets.set_active(bullets_active);
    toolbar.formatting.bullets.set_active(bullets_active);
    toolbar.numbered.set_active(numbered_active);
    toolbar.quick_numbered.set_active(numbered_active);
}

fn sync_editor_state(
    buffer: &gtk::TextBuffer,
    toolbar: &EditorToolbar,
    syncing: &std::rc::Rc<std::cell::Cell<bool>>,
) {
    syncing.set(true);
    let marks = RichBuffer::marks_for_selection(buffer);
    let bold = marks.as_ref().is_some_and(|marks| marks.bold);
    let italic = marks.as_ref().is_some_and(|marks| marks.italic);
    let underline = marks.as_ref().is_some_and(|marks| marks.underline);
    let strikethrough = marks.as_ref().is_some_and(|marks| marks.strikethrough);
    toolbar.bold.set_active(bold);
    toolbar.italic.set_active(italic);
    toolbar.underline.set_active(underline);
    toolbar.quick_underline.set_active(underline);
    toolbar.strikethrough.set_active(strikethrough);
    toolbar.quick_strikethrough.set_active(strikethrough);

    let selected_size = marks.as_ref().map(|marks| marks.font_size.unwrap_or(16));
    let selected_size = selected_size
        .and_then(|size| [12, 14, 16, 18, 24].iter().position(|value| *value == size))
        .map(|index| index as u32)
        .unwrap_or(gtk::INVALID_LIST_POSITION);
    toolbar.font_size.set_selected(selected_size);
    toolbar.quick_font_size.set_selected(selected_size);

    let alignment = RichBuffer::alignment_for_selection(buffer);
    for (button, value) in toolbar.alignment_buttons.iter().zip([
        Alignment::Start,
        Alignment::Center,
        Alignment::End,
        Alignment::Justify,
    ]) {
        button.set_active(alignment == Some(value));
    }

    sync_list_buttons(buffer, toolbar);

    if let Some(marks) = marks {
        sync_palette(&toolbar.foreground_palette, marks.foreground.as_deref());
        sync_palette(&toolbar.highlight_palette, marks.highlight.as_deref());
    } else {
        toolbar.foreground_palette.clear_selection();
        toolbar.highlight_palette.clear_selection();
    }
    syncing.set(false);
}

fn sync_palette(palette: &RichColorPalette, selected: Option<&str>) {
    let index = selected.and_then(|selected| {
        presets(palette.role)
            .iter()
            .position(|preset| preset.id == selected)
    });
    if selected.is_none() || index.is_some() {
        palette.select_preset(index);
    } else {
        palette.clear_selection();
    }
}

fn sync_history_buttons(
    buffer: &gtk::TextBuffer,
    undo: &gtk::Button,
    redo: &gtk::Button,
    editable: bool,
) {
    undo.set_sensitive(editable && RichBuffer::can_undo(buffer));
    redo.set_sensitive(editable && RichBuffer::can_redo(buffer));
}

fn run_command(
    command: EditorCommand,
    buffer: &gtk::TextBuffer,
    editor: &gtk::TextView,
    saved_range: &std::rc::Rc<std::cell::RefCell<SavedTextRange>>,
    argument: Option<&str>,
) -> bool {
    with_saved_range(buffer, editor, saved_range, |buffer| {
        execute(command, buffer, argument)
    })
}

fn with_saved_range<T>(
    buffer: &gtk::TextBuffer,
    editor: &gtk::TextView,
    saved_range: &std::rc::Rc<std::cell::RefCell<SavedTextRange>>,
    mutate: impl FnOnce(&gtk::TextBuffer) -> T,
) -> T {
    let range = *saved_range.borrow();
    range.restore(buffer);
    let result = mutate(buffer);
    *saved_range.borrow_mut() = SavedTextRange::capture(buffer);
    editor.grab_focus();
    result
}
