use adw::prelude::*;

use crate::rich_color::{ColorRole, presets};

#[derive(Clone)]
pub struct RichColorPalette {
    pub widget: gtk::Box,
    pub reset: gtk::ToggleButton,
    pub preset_buttons: Vec<gtk::ToggleButton>,
    pub custom: gtk::ColorDialogButton,
    pub role: ColorRole,
}

impl RichColorPalette {
    pub fn new(role: ColorRole) -> Self {
        let (heading, reset_tooltip, custom_tooltip) = match role {
            ColorRole::Foreground => ("Text color", "Automatic text color", "Custom text color"),
            ColorRole::Highlight => ("Highlight", "No highlight", "Custom highlight color"),
        };

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
        widget.add_css_class("nn-rich-color-palette");

        let title = gtk::Label::new(Some(heading));
        title.set_xalign(0.0);
        title.add_css_class("caption-heading");
        widget.append(&title);

        let choices = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .min_children_per_line(1)
            .max_children_per_line(9)
            .column_spacing(4)
            .row_spacing(4)
            .build();
        choices.add_css_class("nn-color-choices");

        let reset = gtk::ToggleButton::builder()
            .icon_name("edit-clear-symbolic")
            .tooltip_text(reset_tooltip)
            .focusable(true)
            .build();
        reset.add_css_class("nn-color-button");
        reset.update_property(&[gtk::accessible::Property::Label(reset_tooltip)]);
        choices.insert(&reset, -1);

        let mut preset_buttons = Vec::new();
        for preset in presets(role) {
            let tooltip = match role {
                ColorRole::Foreground => format!("{} text", preset.label),
                ColorRole::Highlight => format!("{} highlight", preset.label),
            };
            let button = gtk::ToggleButton::builder()
                .tooltip_text(&tooltip)
                .focusable(true)
                .build();
            button.add_css_class("nn-color-button");
            button.update_property(&[gtk::accessible::Property::Label(&tooltip)]);

            let overlay = gtk::Overlay::new();
            let swatch = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            swatch.add_css_class("nn-color-swatch");
            swatch.add_css_class(match role {
                ColorRole::Foreground => "nn-text-swatch",
                ColorRole::Highlight => "nn-highlight-swatch",
            });
            swatch.add_css_class(&format!("nn-color-{}", preset.id));
            overlay.set_child(Some(&swatch));

            let check = gtk::Image::from_icon_name("object-select-symbolic");
            check.add_css_class("nn-color-check");
            check.set_halign(gtk::Align::Center);
            check.set_valign(gtk::Align::Center);
            check.set_visible(false);
            overlay.add_overlay(&check);
            let check_weak = check.downgrade();
            button.connect_active_notify(move |button| {
                if let Some(check) = check_weak.upgrade() {
                    check.set_visible(button.is_active());
                }
            });
            button.set_child(Some(&overlay));
            choices.insert(&button, -1);
            preset_buttons.push(button);
        }

        let dialog = gtk::ColorDialog::builder()
            .title(custom_tooltip)
            .modal(true)
            .with_alpha(false)
            .build();
        let custom = gtk::ColorDialogButton::new(Some(dialog));
        custom.set_tooltip_text(Some(custom_tooltip));
        custom.set_focusable(true);
        custom.add_css_class("nn-custom-color-button");
        custom.update_property(&[gtk::accessible::Property::Label(custom_tooltip)]);
        choices.insert(&custom, -1);

        widget.append(&choices);
        Self {
            widget,
            reset,
            preset_buttons,
            custom,
            role,
        }
    }

    pub fn clear_selection(&self) {
        self.reset.set_active(false);
        for button in &self.preset_buttons {
            button.set_active(false);
        }
    }

    pub fn select_preset(&self, selected: Option<usize>) {
        self.reset.set_active(selected.is_none());
        for (index, button) in self.preset_buttons.iter().enumerate() {
            button.set_active(selected == Some(index));
        }
    }
}
