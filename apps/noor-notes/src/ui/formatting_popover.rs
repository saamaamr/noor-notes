use adw::prelude::*;

use crate::rich_color::ColorRole;

use super::rich_color_palette::RichColorPalette;
use super::toolbar_primitives::expose_toggle_checked;

#[derive(Clone)]
pub struct FormattingPopover {
    pub widget: gtk::Popover,
    pub section_labels: Vec<gtk::Label>,
    pub underline: gtk::ToggleButton,
    pub strikethrough: gtk::ToggleButton,
    pub bullets: gtk::ToggleButton,
    pub numbered: gtk::ToggleButton,
    pub font_size: gtk::DropDown,
    pub custom_font_size: gtk::Entry,
    pub apply_font_size: gtk::Button,
    pub clear_formatting: gtk::Button,
    pub alignment_buttons: Vec<gtk::ToggleButton>,
    pub foreground_palette: RichColorPalette,
    pub highlight_palette: RichColorPalette,
}

impl FormattingPopover {
    pub fn new() -> Self {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 10);
        content.add_css_class("nn-formatting-popover");
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);

        let typography = section_label("Typography");
        content.append(&typography);
        let type_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let font_size = gtk::DropDown::from_strings(&["12 px", "14 px", "16 px", "18 px", "24 px"]);
        font_size.set_selected(2);
        font_size.set_tooltip_text(Some("Font size"));
        font_size.set_hexpand(true);
        let custom_font_size = gtk::Entry::builder()
            .placeholder_text("Custom px")
            .input_purpose(gtk::InputPurpose::Digits)
            .width_chars(8)
            .tooltip_text("Custom positive whole-number font size")
            .build();
        let apply_font_size = gtk::Button::with_label("Apply");
        apply_font_size.set_tooltip_text(Some("Apply custom font size"));
        type_row.append(&font_size);
        type_row.append(&custom_font_size);
        type_row.append(&apply_font_size);
        content.append(&type_row);

        let formatting = section_label("Formatting");
        content.append(&formatting);
        let format_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let underline = text_toggle("U", "Underline (Ctrl+U)", "format-underline");
        let strikethrough = text_toggle("S", "Strikethrough", "format-strike");
        format_row.append(&underline);
        format_row.append(&strikethrough);
        content.append(&format_row);

        let alignment = section_label("Alignment");
        content.append(&alignment);
        let alignment_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let alignment_buttons = [
            ("format-justify-left-symbolic", "Align left"),
            ("format-justify-center-symbolic", "Align center"),
            ("format-justify-right-symbolic", "Align right"),
            ("format-justify-fill-symbolic", "Justify"),
        ]
        .iter()
        .map(|(icon, tooltip)| icon_toggle(icon, tooltip))
        .collect::<Vec<_>>();
        for button in &alignment_buttons {
            alignment_row.append(button);
        }
        content.append(&alignment_row);

        let colors = section_label("Colors");
        content.append(&colors);
        let colors_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        colors_box.add_css_class("nn-format-colors");
        let foreground_palette = RichColorPalette::new(ColorRole::Foreground);
        colors_box.append(&foreground_palette.widget);
        let highlight_palette = RichColorPalette::new(ColorRole::Highlight);
        colors_box.append(&highlight_palette.widget);
        content.append(&colors_box);

        let lists = section_label("Lists");
        content.append(&lists);
        let list_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let bullets = icon_toggle("view-list-bullet-symbolic", "Bullet list");
        let numbered = icon_toggle("view-list-ordered-symbolic", "Numbered list");
        list_row.append(&bullets);
        list_row.append(&numbered);
        content.append(&list_row);

        let clear_formatting = gtk::Button::with_label("Clear Formatting");
        clear_formatting.set_tooltip_text(Some("Remove formatting from the selection"));
        clear_formatting.add_css_class("flat");
        content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        content.append(&clear_formatting);

        let widget = gtk::Popover::builder().child(&content).build();
        widget.add_css_class("nn-menu-surface");
        Self {
            widget,
            section_labels: vec![typography, formatting, alignment, colors, lists],
            underline,
            strikethrough,
            bullets,
            numbered,
            font_size,
            custom_font_size,
            apply_font_size,
            clear_formatting,
            alignment_buttons,
            foreground_palette,
            highlight_palette,
        }
    }

    pub fn section_names(&self) -> Vec<String> {
        self.section_labels
            .iter()
            .map(|label| label.text().to_string())
            .collect()
    }
}

impl Default for FormattingPopover {
    fn default() -> Self {
        Self::new()
    }
}

fn section_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("nn-format-section-label");
    label.set_halign(gtk::Align::Start);
    label
}

fn icon_toggle(icon: &str, tooltip: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .build();
    button.add_css_class("toolbar-button");
    button.update_property(&[gtk::accessible::Property::Label(tooltip)]);
    expose_toggle_checked(&button);
    button
}

fn text_toggle(label: &str, tooltip: &str, class: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder()
        .label(label)
        .tooltip_text(tooltip)
        .build();
    button.add_css_class("format-choice");
    button.add_css_class(class);
    button.update_property(&[gtk::accessible::Property::Label(tooltip)]);
    expose_toggle_checked(&button);
    button
}
