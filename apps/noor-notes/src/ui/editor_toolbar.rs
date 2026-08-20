use adw::prelude::*;
use noor_domain::EditorMode;
use std::cell::Cell;
use std::rc::Rc;

use crate::ui::formatting_popover::FormattingPopover;
use crate::ui::rich_color_palette::RichColorPalette;
use crate::ui::toolbar_primitives::{
    ToolbarGroup, icon_button, icon_toggle, style_menu_button, text_toggle, toggle_button,
};

#[derive(Clone)]
pub struct EditorToolbar {
    pub widget: gtk::Box,
    pub more: gtk::MenuButton,
    pub more_actions: gtk::FlowBox,
    pub format: gtk::MenuButton,
    pub emoji: gtk::MenuButton,
    pub formatting: FormattingPopover,
    pub new_note: gtk::Button,
    pub undo: gtk::Button,
    pub redo: gtk::Button,
    pub pin: gtk::ToggleButton,
    pub bold: gtk::ToggleButton,
    pub italic: gtk::ToggleButton,
    pub underline: gtk::ToggleButton,
    pub strikethrough: gtk::ToggleButton,
    pub bullets: gtk::ToggleButton,
    pub numbered: gtk::ToggleButton,
    pub font_size: gtk::DropDown,
    pub quick_underline: gtk::ToggleButton,
    pub quick_strikethrough: gtk::ToggleButton,
    pub quick_numbered: gtk::ToggleButton,
    pub quick_font_size: gtk::DropDown,
    pub clear_formatting: gtk::Button,
    pub custom_font_size: gtk::Entry,
    pub apply_font_size: gtk::Button,
    pub rename: gtk::Button,
    pub find: gtk::ToggleButton,
    pub word_wrap: gtk::ToggleButton,
    pub zoom_in: gtk::Button,
    pub zoom_out: gtk::Button,
    pub zoom_reset: gtk::Button,
    pub go_to_line: gtk::Button,
    pub fullscreen: gtk::ToggleButton,
    pub view_only: gtk::Button,
    pub duplicate: gtk::Button,
    pub export_text: gtk::Button,
    pub export_markdown: gtk::Button,
    pub mode_rich: gtk::Button,
    pub mode_markdown: gtk::Button,
    pub mode_plain: gtk::Button,
    pub mode_code: gtk::Button,
    pub alignment_buttons: Vec<gtk::ToggleButton>,
    pub foreground_palette: RichColorPalette,
    pub appearance: gtk::MenuButton,
    pub writing_assistance: gtk::MenuButton,
    pub highlight_palette: RichColorPalette,
    pub emoji_buttons: Vec<gtk::Button>,
    pub all_workspaces: gtk::ToggleButton,
    pub opacity: gtk::Scale,
    pub note_color_buttons: Vec<gtk::Button>,
    pub archive: gtk::Button,
    pub header_archive: gtk::Button,
    pub trash: gtk::Button,
    pub header_trash: gtk::Button,
    pub restore: gtk::Button,
    pub permanent_delete: gtk::Button,
    groups: Vec<ToolbarGroup>,
    group_separators: Vec<gtk::Separator>,
    can_edit: Rc<Cell<bool>>,
}

impl EditorToolbar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        widget.set_hexpand(false);
        widget.set_halign(gtk::Align::Start);
        widget.add_css_class("nn-editor-toolbar");
        widget.add_css_class("nn-command-bar");

        let new_note = icon_button("list-add-symbolic", "New note");
        let undo = icon_button("edit-undo-symbolic", "Undo (Ctrl+Z)");
        let redo = icon_button("edit-redo-symbolic", "Redo (Ctrl+Shift+Z)");
        undo.set_sensitive(false);
        redo.set_sensitive(false);
        let pin = toggle_button("view-pin-symbolic", "Always on Top");

        let bold = text_toggle("B", "Bold (Ctrl+B)", "format-bold");
        let italic = text_toggle("I", "Italic (Ctrl+I)", "format-italic");
        let bullets = icon_toggle("view-list-bullet-symbolic", "Bullet list");
        let quick_underline = text_toggle("U", "Underline (Ctrl+U)", "format-underline");
        let quick_strikethrough = text_toggle("S", "Strikethrough", "format-strike");
        let quick_numbered = icon_toggle("view-list-ordered-symbolic", "Numbered list");
        let quick_font_size = gtk::DropDown::from_strings(&["12", "14", "16", "18", "24"]);
        quick_font_size.set_selected(2);
        quick_font_size.set_tooltip_text(Some("Font size"));
        quick_font_size.set_width_request(64);
        quick_font_size.add_css_class("nn-toolbar-font-size");
        quick_font_size.add_css_class("nn-control-compact");
        let formatting = FormattingPopover::new();
        let underline = formatting.underline.clone();
        let strikethrough = formatting.strikethrough.clone();
        let numbered = formatting.numbered.clone();
        let font_size = formatting.font_size.clone();
        let custom_font_size = formatting.custom_font_size.clone();
        let apply_font_size = formatting.apply_font_size.clone();
        let clear_formatting = formatting.clear_formatting.clone();
        let alignment_buttons = formatting.alignment_buttons.clone();
        let foreground_palette = formatting.foreground_palette.clone();
        let highlight_palette = formatting.highlight_palette.clone();
        let format = gtk::MenuButton::builder()
            .icon_name("format-text-rich-symbolic")
            .tooltip_text("Formatting")
            .popover(&formatting.widget)
            .build();
        style_menu_button(&format, "Formatting");

        let emoji_grid = gtk::Grid::builder()
            .column_spacing(4)
            .row_spacing(4)
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(10)
            .margin_end(10)
            .build();
        let emojis = [
            "😀", "😂", "😍", "🤲", "✨", "❤️", "👍", "✅", "📌", "💡", "🎉", "🌙",
        ];
        let mut emoji_buttons = Vec::new();
        for (index, emoji) in emojis.iter().enumerate() {
            let button = gtk::Button::with_label(emoji);
            button.add_css_class("emoji-choice");
            emoji_grid.attach(&button, (index % 6) as i32, (index / 6) as i32, 1, 1);
            emoji_buttons.push(button);
        }
        let emoji_popover = gtk::Popover::builder().child(&emoji_grid).build();
        let emoji = gtk::MenuButton::builder()
            .icon_name("face-smile-symbolic")
            .tooltip_text("Insert emoji")
            .popover(&emoji_popover)
            .build();
        style_menu_button(&emoji, "Insert emoji");
        let find = toggle_button("edit-find-symbolic", "Find in note (Ctrl+F)");

        let all_workspaces = toggle_button("focus-windows-symbolic", "Show on all workspaces");
        let opacity = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.35, 1.0, 0.05);
        opacity.set_width_request(150);
        opacity.set_draw_value(false);
        opacity.set_tooltip_text(Some("Note opacity"));
        let settings_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        settings_box.set_margin_top(12);
        settings_box.set_margin_bottom(12);
        settings_box.set_margin_start(12);
        settings_box.set_margin_end(12);
        settings_box.append(&gtk::Label::new(Some("Window settings")));
        settings_box.append(&gtk::Label::new(Some("Note colour")));
        let color_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let note_color_buttons = ["Yellow", "Cream", "Blue", "Green", "Rose", "Lavender"]
            .iter()
            .map(|name| {
                let button = gtk::Button::with_label(name);
                button.set_tooltip_text(Some(&format!("Use {name} note colour")));
                color_row.append(&button);
                button
            })
            .collect::<Vec<_>>();
        settings_box.append(&color_row);
        settings_box.append(&all_workspaces);
        settings_box.append(&opacity);
        let settings_popover = gtk::Popover::builder().child(&settings_box).build();
        let appearance = gtk::MenuButton::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Note settings")
            .popover(&settings_popover)
            .build();
        appearance.add_css_class("flat");
        let word_wrap = toggle_button("format-justify-left-symbolic", "Word wrap");
        word_wrap.set_active(true);
        let zoom_in = icon_button("zoom-in-symbolic", "Zoom in (Ctrl++)");
        let zoom_out = icon_button("zoom-out-symbolic", "Zoom out (Ctrl+-)");
        let zoom_reset = icon_button("zoom-original-symbolic", "Reset zoom (Ctrl+0)");
        let go_to_line = icon_button("go-jump-symbolic", "Go to line (Ctrl+G)");
        let fullscreen = toggle_button("view-fullscreen-symbolic", "Full screen (F11)");
        let view_only = gtk::Button::with_label("View Only");
        view_only.set_tooltip_text(Some("Read-only mode"));
        view_only.set_tooltip_text(Some("Read this note without editing controls"));
        view_only.update_property(&[gtk::accessible::Property::Label(
            "Read this note without editing controls",
        )]);
        view_only.add_css_class("nn-view-only-toggle");
        let view_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        view_box.append(&word_wrap);
        view_box.append(&zoom_in);
        view_box.append(&zoom_out);
        view_box.append(&zoom_reset);
        view_box.append(&go_to_line);
        view_box.append(&fullscreen);
        let view_popover = gtk::Popover::builder().child(&view_box).build();
        let view = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("Editor view options")
            .popover(&view_popover)
            .build();
        view.add_css_class("toolbar-button");
        let duplicate = icon_button("edit-copy-symbolic", "Duplicate note");
        let export_text = gtk::Button::with_label("Plain text (.txt)");
        let export_markdown = gtk::Button::with_label("Markdown (.md)");
        let export_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        export_box.append(&export_text);
        export_box.append(&export_markdown);
        let export_popover = gtk::Popover::builder().child(&export_box).build();
        let export = gtk::MenuButton::builder()
            .icon_name("document-save-symbolic")
            .tooltip_text("Export note")
            .popover(&export_popover)
            .build();
        export.add_css_class("toolbar-button");
        let rename = icon_button("document-edit-symbolic", "Rename note");
        let archive = icon_button("folder-symbolic", "Archive note");
        let header_archive = icon_button("folder-symbolic", "Archive note");
        let trash = icon_button("user-trash-symbolic", "Move to Trash");
        trash.add_css_class("destructive-hover");
        let header_trash = icon_button("user-trash-symbolic", "Move to Trash");
        header_trash.add_css_class("destructive-hover");
        let mode_label = gtk::Label::new(Some("Editor mode"));
        mode_label.add_css_class("heading");
        mode_label.set_halign(gtk::Align::Start);
        let mode_rich = gtk::Button::with_label("Rich Text");
        mode_rich.set_tooltip_text(Some("Convert to rich text"));
        let mode_markdown = gtk::Button::with_label("Markdown");
        mode_markdown.set_tooltip_text(Some("Convert to Markdown"));
        let mode_plain = gtk::Button::with_label("Plain Text");
        mode_plain.set_tooltip_text(Some("Convert to plain text"));
        let mode_code = gtk::Button::with_label("Code");
        mode_code.set_tooltip_text(Some("Convert to code mode"));

        let restore = icon_button("edit-undo-symbolic", "Restore");
        let permanent_delete = icon_button("edit-delete-symbolic", "Permanently Delete");
        permanent_delete.add_css_class("destructive-hover");
        let writing_assistance = gtk::MenuButton::builder()
            .label("Writing Assistance")
            .tooltip_text("Writing assistance for this note")
            .build();
        writing_assistance.update_property(&[gtk::accessible::Property::Label(
            "Writing assistance for this note",
        )]);
        let more_actions = gtk::FlowBox::builder()
            .orientation(gtk::Orientation::Vertical)
            .selection_mode(gtk::SelectionMode::None)
            .min_children_per_line(1)
            .max_children_per_line(6)
            .column_spacing(6)
            .row_spacing(4)
            .build();
        more_actions.add_css_class("nn-more-actions");
        for action in [
            new_note.upcast_ref::<gtk::Widget>(),
            rename.upcast_ref(),
            duplicate.upcast_ref(),
            pin.upcast_ref(),
            writing_assistance.upcast_ref(),
            archive.upcast_ref(),
            trash.upcast_ref(),
            restore.upcast_ref(),
            permanent_delete.upcast_ref(),
            export.upcast_ref(),
            view.upcast_ref(),
        ] {
            more_actions.insert(action, -1);
        }

        let mode_actions = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .min_children_per_line(1)
            .max_children_per_line(4)
            .column_spacing(4)
            .row_spacing(4)
            .build();
        for action in [
            mode_rich.upcast_ref::<gtk::Widget>(),
            mode_markdown.upcast_ref(),
            mode_plain.upcast_ref(),
            mode_code.upcast_ref(),
        ] {
            mode_actions.insert(action, -1);
        }

        let more_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        more_box.set_margin_top(6);
        more_box.set_margin_bottom(6);
        more_box.set_margin_start(6);
        more_box.set_margin_end(6);
        more_box.append(&more_actions);
        more_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        more_box.append(&mode_label);
        more_box.append(&mode_actions);
        let more_popover = gtk::Popover::builder().child(&more_box).build();
        let more = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("More note actions")
            .popover(&more_popover)
            .build();
        style_menu_button(&more, "More note actions");
        for button in [
            &new_note,
            &rename,
            &duplicate,
            &archive,
            &trash,
            &restore,
            &permanent_delete,
        ] {
            close_more_on_click(button, &more_popover);
        }
        close_more_on_toggle(&pin, &more_popover);
        let history_group = ToolbarGroup::new("nn-command-primary");
        history_group.append(&undo);
        history_group.append(&redo);
        let typography_group = ToolbarGroup::new("nn-command-primary");
        // RichDocument has no block-style model, so only its real size command
        // is exposed here.
        typography_group.append(&quick_font_size);
        let inline_group = ToolbarGroup::new("nn-command-primary");
        inline_group.append(&bold);
        inline_group.append(&italic);
        inline_group.append(&quick_underline);
        inline_group.append(&quick_strikethrough);
        inline_group.append(&format);
        let insert_group = ToolbarGroup::new("nn-command-secondary");
        insert_group.append(&bullets);
        insert_group.append(&quick_numbered);
        insert_group.append(&emoji);
        let more_group = ToolbarGroup::new("nn-command-overflow");
        more_group.append(&find);
        more_group.append(&more);
        let groups = vec![
            history_group,
            typography_group,
            inline_group,
            insert_group,
            more_group,
        ];
        let mut group_separators = Vec::new();
        for (index, group) in groups.iter().enumerate() {
            if index > 0 {
                let separator = gtk::Separator::new(gtk::Orientation::Vertical);
                widget.append(&separator);
                group_separators.push(separator);
            }
            widget.append(&group.widget);
        }
        Self {
            widget,
            more,
            more_actions,
            format,
            emoji,
            formatting,
            new_note,
            undo,
            redo,
            mode_rich,
            mode_markdown,
            mode_plain,
            mode_code,
            pin,
            bold,
            italic,
            underline,
            strikethrough,
            bullets,
            numbered,
            font_size,
            quick_underline,
            quick_strikethrough,
            quick_numbered,
            quick_font_size,
            custom_font_size,
            apply_font_size,
            rename,
            clear_formatting,
            find,
            word_wrap,
            zoom_in,
            zoom_out,
            zoom_reset,
            go_to_line,
            fullscreen,
            view_only,
            duplicate,
            export_text,
            export_markdown,
            alignment_buttons,
            foreground_palette,
            highlight_palette,
            emoji_buttons,
            appearance,
            writing_assistance,
            all_workspaces,
            opacity,
            note_color_buttons,
            archive,
            header_archive,
            trash,
            header_trash,
            restore,
            permanent_delete,
            groups,
            group_separators,
            can_edit: Rc::new(Cell::new(true)),
        }
    }

    pub fn set_editable(&self, enabled: bool) {
        self.can_edit.set(enabled);
        for control in [
            self.bold.upcast_ref::<gtk::Widget>(),
            self.italic.upcast_ref(),
            self.quick_underline.upcast_ref(),
            self.quick_strikethrough.upcast_ref(),
            self.bullets.upcast_ref(),
            self.quick_numbered.upcast_ref(),
            self.quick_font_size.upcast_ref(),
            self.format.upcast_ref(),
            self.emoji.upcast_ref(),
        ] {
            control.set_sensitive(enabled);
        }
        if !enabled {
            self.undo.set_sensitive(false);
            self.redo.set_sensitive(false);
        }
        self.set_rich_formatting_enabled(enabled);
    }

    pub fn is_editable(&self) -> bool {
        self.can_edit.get()
    }

    pub fn edit_state(&self) -> Rc<Cell<bool>> {
        self.can_edit.clone()
    }

    pub fn set_rich_formatting_enabled(&self, enabled: bool) {
        for control in [
            self.format.upcast_ref::<gtk::Widget>(),
            self.bold.upcast_ref::<gtk::Widget>(),
            self.italic.upcast_ref(),
            self.underline.upcast_ref(),
            self.strikethrough.upcast_ref(),
            self.bullets.upcast_ref(),
            self.numbered.upcast_ref(),
            self.font_size.upcast_ref(),
            self.quick_underline.upcast_ref(),
            self.quick_strikethrough.upcast_ref(),
            self.quick_numbered.upcast_ref(),
            self.quick_font_size.upcast_ref(),
            self.custom_font_size.upcast_ref(),
            self.apply_font_size.upcast_ref(),
            self.clear_formatting.upcast_ref(),
            self.foreground_palette.widget.upcast_ref(),
            self.highlight_palette.widget.upcast_ref(),
        ] {
            control.set_sensitive(enabled);
        }
        for button in &self.alignment_buttons {
            button.set_sensitive(enabled);
        }
    }

    pub fn set_editor_mode(&self, mode: EditorMode) {
        let rich = mode == EditorMode::Rich;
        for control in [
            self.bold.upcast_ref::<gtk::Widget>(),
            self.italic.upcast_ref(),
            self.quick_underline.upcast_ref(),
            self.quick_strikethrough.upcast_ref(),
            self.bullets.upcast_ref(),
            self.quick_numbered.upcast_ref(),
            self.quick_font_size.upcast_ref(),
            self.format.upcast_ref(),
        ] {
            control.set_visible(rich);
        }
        self.emoji.set_visible(mode != EditorMode::Code);
        self.find.set_visible(!rich);
    }

    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    pub fn set_compact(&self, compact: bool) {
        self.groups[1].widget.set_visible(!compact);
        self.groups[3].widget.set_visible(!compact);
        self.group_separators[0].set_visible(true);
        self.group_separators[1].set_visible(!compact);
        self.group_separators[2].set_visible(true);
        self.group_separators[3].set_visible(!compact);
    }

    pub fn group_visible(&self, index: usize) -> bool {
        self.groups
            .get(index)
            .is_some_and(|group| group.widget.is_visible())
    }

    pub fn set_view_only_state(&self, enabled: bool) {
        let (label, description) = if enabled {
            ("Exit View Only", "Return to editing this note")
        } else {
            ("View Only", "Read this note without editing controls")
        };
        self.view_only.set_label(label);
        self.view_only.set_tooltip_text(Some(description));
        self.view_only
            .update_property(&[gtk::accessible::Property::Label(description)]);
    }
}

fn close_more_on_click(button: &gtk::Button, more_popover: &gtk::Popover) {
    let more_popover = more_popover.clone();
    button.connect_clicked(move |_| more_popover.popdown());
}

fn close_more_on_toggle(button: &gtk::ToggleButton, more_popover: &gtk::Popover) {
    let more_popover = more_popover.clone();
    button.connect_toggled(move |_| more_popover.popdown());
}
impl Default for EditorToolbar {
    fn default() -> Self {
        Self::new()
    }
}
