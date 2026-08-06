use adw::prelude::*;

#[derive(Clone)]
pub struct ModernToolbar {
    pub widget: gtk::Box,
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
    pub duplicate: gtk::Button,
    pub export_text: gtk::Button,
    pub export_markdown: gtk::Button,
    pub alignment_buttons: Vec<gtk::ToggleButton>,
    pub foreground_buttons: Vec<gtk::Button>,
    pub highlight_buttons: Vec<gtk::Button>,
    pub emoji_buttons: Vec<gtk::Button>,
    pub all_workspaces: gtk::ToggleButton,
    pub opacity: gtk::Scale,
    pub note_color_buttons: Vec<gtk::Button>,
    pub archive: gtk::Button,
    pub trash: gtk::Button,
    pub restore: gtk::Button,
    pub permanent_delete: gtk::Button,
}

impl ModernToolbar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        widget.add_css_class("modern-toolbar");

        let new_note = icon_button("list-add-symbolic", "New note");
        let undo = icon_button("edit-undo-symbolic", "Undo (Ctrl+Z)");
        let redo = icon_button("edit-redo-symbolic", "Redo (Ctrl+Shift+Z)");
        undo.set_sensitive(false);
        redo.set_sensitive(false);
        let pin = toggle_button("view-pin-symbolic", "Always on Top");
        let left = group();
        left.append(&pin);
        left.append(&new_note);
        left.append(&undo);
        left.append(&redo);

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
        for (index, button) in [&bold, &italic, &underline, &strikethrough]
            .iter()
            .enumerate()
        {
            format_grid.attach(*button, index as i32, 0, 1, 1);
        }
        format_grid.attach(&bullets, 0, 1, 1, 1);
        format_grid.attach(&numbered, 1, 1, 1, 1);
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
        let foreground_buttons = color_buttons(&format_grid, 5, "Text", "text-color");
        let highlight_buttons = color_buttons(&format_grid, 4, "Highlight", "highlight-color");
        let format_popover = gtk::Popover::builder().child(&format_grid).build();
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
        let center = group();
        center.append(&format);
        center.append(&emoji);

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
        let settings = gtk::MenuButton::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Note settings")
            .popover(&settings_popover)
            .build();
        settings.add_css_class("toolbar-button");
        let find = toggle_button("edit-find-symbolic", "Find in note (Ctrl+F)");
        let word_wrap = toggle_button("format-justify-left-symbolic", "Word wrap");
        word_wrap.set_active(true);
        let zoom_in = icon_button("zoom-in-symbolic", "Zoom in (Ctrl++)");
        let zoom_out = icon_button("zoom-out-symbolic", "Zoom out (Ctrl+-)");
        let zoom_reset = icon_button("zoom-original-symbolic", "Reset zoom (Ctrl+0)");
        let go_to_line = icon_button("go-jump-symbolic", "Go to line (Ctrl+G)");
        let fullscreen = toggle_button("view-fullscreen-symbolic", "Full screen (F11)");
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
        let archive = icon_button("folder-symbolic", "Archive");
        let trash = icon_button("user-trash-symbolic", "Move to Trash");
        trash.add_css_class("destructive-hover");
        let restore = icon_button("edit-undo-symbolic", "Restore");
        let permanent_delete = icon_button("edit-delete-symbolic", "Permanently Delete");
        permanent_delete.add_css_class("destructive-hover");
        let more_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        more_box.append(&rename);
        more_box.append(&duplicate);
        more_box.append(&export);
        more_box.append(&view);
        more_box.append(&settings);
        let more_popover = gtk::Popover::builder().child(&more_box).build();
        let more = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("More note actions")
            .popover(&more_popover)
            .build();
        more.add_css_class("toolbar-button");
        let right = group();
        right.append(&find);
        right.append(&archive);
        right.append(&trash);
        right.append(&restore);
        right.append(&permanent_delete);
        right.append(&more);

        widget.append(&left);
        widget.append(&center);
        widget.append(&right);
        Self {
            widget,
            new_note,
            undo,
            redo,
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
            find,
            word_wrap,
            zoom_in,
            zoom_out,
            zoom_reset,
            go_to_line,
            fullscreen,
            duplicate,
            export_text,
            export_markdown,
            alignment_buttons,
            foreground_buttons,
            highlight_buttons,
            emoji_buttons,
            all_workspaces,
            opacity,
            note_color_buttons,
            archive,
            trash,
            restore,
            permanent_delete,
        }
    }
}

fn color_buttons(grid: &gtk::Grid, row: i32, label: &str, class: &str) -> Vec<gtk::Button> {
    let title = gtk::Label::new(Some(label));
    title.set_xalign(0.0);
    grid.attach(&title, 0, row, 1, 1);
    ["charcoal", "blue", "green", "red"]
        .iter()
        .enumerate()
        .map(|(index, color)| {
            let button = gtk::Button::new();
            button.set_tooltip_text(Some(&format!("{label}: {color}")));
            button.add_css_class("color-choice");
            button.add_css_class(class);
            button.add_css_class(color);
            grid.attach(&button, index as i32, row, 1, 1);
            button
        })
        .collect()
}

fn group() -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    group.add_css_class("toolbar-group");
    group
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
impl Default for ModernToolbar {
    fn default() -> Self {
        Self::new()
    }
}
