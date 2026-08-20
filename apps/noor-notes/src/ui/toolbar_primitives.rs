use adw::prelude::*;

#[derive(Clone)]
pub struct ToolbarGroup {
    pub widget: gtk::Box,
}

impl ToolbarGroup {
    pub fn new(priority: &str) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        widget.add_css_class("nn-command-group");
        widget.add_css_class(priority);
        Self { widget }
    }

    pub fn append(&self, child: &impl IsA<gtk::Widget>) {
        self.widget.append(child);
    }
}

pub fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .build();
    style_control(&button, tooltip);
    button
}

pub fn toggle_button(icon: &str, tooltip: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .build();
    style_control(&button, tooltip);
    expose_toggle_checked(&button);
    button
}

pub fn icon_toggle(icon: &str, tooltip: &str) -> gtk::ToggleButton {
    toggle_button(icon, tooltip)
}

pub fn text_toggle(label: &str, tooltip: &str, class: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::builder()
        .label(label)
        .tooltip_text(tooltip)
        .build();
    button.add_css_class("format-choice");
    button.add_css_class("nn-control-compact");
    button.add_css_class(class);
    button.update_property(&[gtk::accessible::Property::Label(tooltip)]);
    expose_toggle_checked(&button);
    button
}

pub fn expose_toggle_checked(button: &gtk::ToggleButton) {
    update_checked(button);
    button.connect_active_notify(update_checked);
}

fn update_checked(button: &gtk::ToggleButton) {
    button.update_state(&[gtk::accessible::State::Checked(if button.is_active() {
        gtk::AccessibleTristate::True
    } else {
        gtk::AccessibleTristate::False
    })]);
}

pub fn style_menu_button(button: &gtk::MenuButton, accessible_label: &str) {
    button.add_css_class("toolbar-button");
    button.add_css_class("nn-control-compact");
    button.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
}

pub fn bind_menu_popover(button: &gtk::MenuButton, popover: &gtk::Popover) {
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let anchor = button.downgrade();
    let surface = popover.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key != gtk::gdk::Key::Escape {
            return gtk::glib::Propagation::Proceed;
        }
        if let Some(popover) = surface.upgrade() {
            popover.popdown();
        }
        if let Some(button) = anchor.upgrade() {
            button.grab_focus();
        }
        gtk::glib::Propagation::Stop
    });
    popover.add_controller(keys);
}

fn style_control<W: IsA<gtk::Widget> + IsA<gtk::Accessible>>(control: &W, label: &str) {
    control.add_css_class("toolbar-button");
    control.add_css_class("nn-control-compact");
    control.update_property(&[gtk::accessible::Property::Label(label)]);
}
