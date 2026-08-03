use adw::prelude::*;

#[derive(Clone)]
pub struct ModernToolbar {
    pub widget: gtk::Box,
    pub new_note: gtk::Button,
    pub pin: gtk::ToggleButton,
    pub bold: gtk::ToggleButton,
    pub italic: gtk::ToggleButton,
    pub underline: gtk::ToggleButton,
    pub strikethrough: gtk::ToggleButton,
    pub bullets: gtk::ToggleButton,
    pub numbered: gtk::ToggleButton,
    pub emoji_buttons: Vec<gtk::Button>,
    pub all_workspaces: gtk::ToggleButton,
    pub opacity: gtk::Scale,
    pub archive: gtk::Button,
    pub trash: gtk::Button,
}

impl ModernToolbar {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        widget.add_css_class("modern-toolbar");

        let new_note = icon_button("list-add-symbolic", "New note");
        let pin = toggle_button("view-pin-symbolic", "Always on Top");
        let left = group();
        left.append(&pin);
        left.append(&new_note);

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
        settings_box.append(&all_workspaces);
        settings_box.append(&opacity);
        let settings_popover = gtk::Popover::builder().child(&settings_box).build();
        let settings = gtk::MenuButton::builder()
            .icon_name("emblem-system-symbolic")
            .tooltip_text("Note settings")
            .popover(&settings_popover)
            .build();
        settings.add_css_class("toolbar-button");
        let archive = icon_button("folder-symbolic", "Archive");
        let trash = icon_button("user-trash-symbolic", "Move to Trash");
        trash.add_css_class("destructive-hover");
        let right = group();
        right.append(&archive);
        right.append(&trash);
        right.append(&settings);

        widget.append(&left);
        widget.append(&center);
        widget.append(&right);
        Self {
            widget,
            new_note,
            pin,
            bold,
            italic,
            underline,
            strikethrough,
            bullets,
            numbered,
            emoji_buttons,
            all_workspaces,
            opacity,
            archive,
            trash,
        }
    }
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
