use adw::prelude::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SaveState {
    #[default]
    Idle,
    Saving,
    Saved,
    Failed(String),
}

#[derive(Clone)]
pub struct SaveStatusIndicator {
    pub widget: gtk::Box,
    pub label: gtk::Label,
    pub retry: gtk::Button,
}

impl SaveStatusIndicator {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        widget.add_css_class("save-status");
        let label = gtk::Label::new(None);
        label.add_css_class("dim-label");
        let retry = gtk::Button::builder()
            .label("Retry")
            .tooltip_text("Retry saving this note")
            .build();
        retry.add_css_class("flat");
        retry.set_visible(false);
        widget.append(&label);
        widget.append(&retry);
        Self {
            widget,
            label,
            retry,
        }
    }

    pub fn set_state(&self, state: &SaveState) {
        match state {
            SaveState::Idle => {
                self.label.set_text("");
                self.retry.set_visible(false);
            }
            SaveState::Saving => {
                self.label.set_text("Saving…");
                self.retry.set_visible(false);
            }
            SaveState::Saved => {
                self.label.set_text("Saved");
                self.retry.set_visible(false);
            }
            SaveState::Failed(message) => {
                self.label.set_text("Save failed");
                self.label.set_tooltip_text(Some(message));
                self.retry.set_visible(true);
            }
        }
    }
}

impl Default for SaveStatusIndicator {
    fn default() -> Self {
        Self::new()
    }
}
