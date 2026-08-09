use adw::prelude::*;

use crate::rich_color::ColorRole;
use crate::ui::rich_color_palette::RichColorPalette;

#[derive(Clone)]
pub struct EditorToolbar {
    pub widget: gtk::FlowBox,
    pub more: gtk::MenuButton,
    pub more_actions: gtk::FlowBox,
    pub format: gtk::MenuButton,
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
}

impl EditorToolbar {
    pub fn new() -> Self {
        let widget = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .min_children_per_line(1)
            .max_children_per_line(9)
            .column_spacing(2)
            .row_spacing(2)
            .hexpand(true)
            .build();
        widget.add_css_class("nn-editor-toolbar");

        let new_note = icon_button("list-add-symbolic", "New note");
        let undo = icon_button("edit-undo-symbolic", "Undo (Ctrl+Z)");
        let redo = icon_button("edit-redo-symbolic", "Redo (Ctrl+Shift+Z)");
        undo.set_sensitive(false);
        redo.set_sensitive(false);
        let pin = toggle_button("view-pin-symbolic", "Always on Top");

        let bold = text_toggle("B", "Bold (Ctrl+B)", "format-bold");
        let italic = text_toggle("I", "Italic (Ctrl+I)", "format-italic");
        let underline = text_toggle("U", "Underline (Ctrl+U)", "format-underline");
        let strikethrough = text_toggle("S", "Strikethrough", "format-strike");
        let bullets = icon_toggle("view-list-bullet-symbolic", "Bullet list");
        let numbered = icon_toggle("view-list-ordered-symbolic", "Numbered list");
        let format_grid = gtk::Grid::builder()
            .column_spacing(6)
            .row_spacing(6)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        for (index, button) in [&underline, &strikethrough].iter().enumerate() {
            format_grid.attach(*button, index as i32, 0, 1, 1);
        }
        format_grid.attach(&numbered, 0, 1, 1, 1);
        let font_size = gtk::DropDown::from_strings(&["12 px", "14 px", "16 px", "18 px", "24 px"]);
        font_size.set_selected(2);
        font_size.set_tooltip_text(Some("Font size"));
        format_grid.attach(&font_size, 2, 1, 2, 1);
        let custom_font_size = gtk::Entry::builder()
            .placeholder_text("Custom px")
            .input_purpose(gtk::InputPurpose::Digits)
            .width_chars(8)
            .tooltip_text("Custom positive whole-number font size")
            .build();
        let apply_font_size = gtk::Button::with_label("Apply");
        apply_font_size.set_tooltip_text(Some("Apply custom font size"));
        format_grid.attach(&custom_font_size, 0, 2, 3, 1);
        format_grid.attach(&apply_font_size, 3, 2, 1, 1);
        let alignment_buttons = [
            ("format-justify-left-symbolic", "Align left"),
            ("format-justify-center-symbolic", "Align center"),
            ("format-justify-right-symbolic", "Align right"),
            ("format-justify-fill-symbolic", "Justify"),
        ]
        .iter()
        .map(|(icon, tooltip)| icon_toggle(icon, tooltip))
        .collect::<Vec<_>>();
        for (index, button) in alignment_buttons.iter().enumerate() {
            format_grid.attach(button, index as i32, 3, 1, 1);
        }
        let foreground_palette = RichColorPalette::new(ColorRole::Foreground);
        let highlight_palette = RichColorPalette::new(ColorRole::Highlight);
        format_grid.attach(&foreground_palette.widget, 0, 4, 4, 1);
        format_grid.attach(&highlight_palette.widget, 0, 5, 4, 1);
        let format_popover = gtk::Popover::builder().child(&format_grid).build();
        let clear_formatting = gtk::Button::with_label("Clear Formatting");
        clear_formatting.set_tooltip_text(Some("Remove formatting from the selection"));
        clear_formatting.add_css_class("flat");
        format_grid.attach(&clear_formatting, 0, 6, 4, 1);
        let format = gtk::MenuButton::builder()
            .icon_name("format-text-rich-symbolic")
            .tooltip_text("Formatting")
            .popover(&format_popover)
            .build();
        format.add_css_class("toolbar-button");

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
        emoji.add_css_class("toolbar-button");
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
        view_only.set_tooltip_text(Some("Read this note without editing controls"));
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
            view_only.upcast_ref(),
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
        more.add_css_class("toolbar-button");
        for action in [
            undo.upcast_ref::<gtk::Widget>(),
            redo.upcast_ref(),
            find.upcast_ref(),
            bold.upcast_ref(),
            italic.upcast_ref(),
            bullets.upcast_ref(),
            format.upcast_ref(),
            emoji.upcast_ref(),
            more.upcast_ref(),
        ] {
            widget.insert(action, -1);
        }
        Self {
            widget,
            more,
            more_actions,
            format,
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
            all_workspaces,
            opacity,
            note_color_buttons,
            archive,
            header_archive,
            trash,
            header_trash,
            restore,
            permanent_delete,
        }
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
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .build();
    button.add_css_class("toolbar-button");
    button
}

fn toggle_button(icon: &str, tooltip: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .build();
    button.add_css_class("toolbar-button");
    button
}

fn icon_toggle(icon: &str, tooltip: &str) -> gtk::ToggleButton {
    toggle_button(icon, tooltip)
}

fn text_toggle(label: &str, tooltip: &str, class: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder()
        .label(label)
        .tooltip_text(tooltip)
        .build();
    button.add_css_class("format-choice");
    button.add_css_class(class);
    button
}
impl Default for EditorToolbar {
    fn default() -> Self {
        Self::new()
    }
}
