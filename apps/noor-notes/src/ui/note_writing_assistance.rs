use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use noor_domain::{EditorMode, WritingAssistanceOverrides};

use crate::writing_assistance::WritingAssistancePreferences;

type ChangedHandler = Box<dyn Fn(WritingAssistanceOverrides)>;

#[derive(Clone)]
pub struct NoteWritingAssistancePopover {
    pub widget: gtk::Popover,
    pub override_global: gtk::Switch,
    pub spelling: gtk::Switch,
    pub grammar: gtk::Switch,
    pub offline_prediction: gtk::Switch,
    pub cloud: gtk::Switch,
    handlers: Rc<RefCell<Vec<ChangedHandler>>>,
    description: String,
}

impl NoteWritingAssistancePopover {
    pub fn new(
        global: &WritingAssistancePreferences,
        overrides: &WritingAssistanceOverrides,
        mode: EditorMode,
    ) -> Self {
        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_width_request(340);
        let heading = gtk::Label::new(Some("Writing Assistance"));
        heading.add_css_class("heading");
        heading.set_halign(gtk::Align::Start);
        content.append(&heading);
        let override_global = labelled_switch(
            &content,
            "Override global settings for this note",
            overrides != &WritingAssistanceOverrides::default(),
        );
        let spelling = labelled_switch(
            &content,
            "Spelling",
            overrides.spelling.unwrap_or(global.spelling),
        );
        let grammar = labelled_switch(
            &content,
            "Grammar",
            overrides.grammar.unwrap_or(global.grammar),
        );
        let offline_prediction = labelled_switch(
            &content,
            "Offline predictions",
            overrides
                .offline_prediction
                .unwrap_or(global.offline_prediction),
        );
        let cloud = labelled_switch(
            &content,
            "Online AI assistance",
            overrides.cloud.unwrap_or(global.cloud_enabled),
        );
        let description = if mode == EditorMode::Code {
            "Checks comments and strings only".to_owned()
        } else {
            "Local checks and suggestions follow this note's editor mode".to_owned()
        };
        let description_label = gtk::Label::new(Some(&description));
        description_label.set_wrap(true);
        description_label.set_halign(gtk::Align::Start);
        description_label.add_css_class("dim-label");
        content.append(&description_label);
        let enabled = override_global.is_active();
        for switch in [&spelling, &grammar, &offline_prediction, &cloud] {
            switch.set_sensitive(enabled);
        }
        let widget = gtk::Popover::builder().child(&content).build();
        let value = Self {
            widget,
            override_global,
            spelling,
            grammar,
            offline_prediction,
            cloud,
            handlers: Rc::new(RefCell::new(Vec::new())),
            description,
        };
        value.connect_controls();
        value
    }

    pub fn connect_changed(&self, handler: impl Fn(WritingAssistanceOverrides) + 'static) {
        self.handlers.borrow_mut().push(Box::new(handler));
    }

    pub fn overrides(&self) -> WritingAssistanceOverrides {
        if !self.override_global.is_active() {
            return WritingAssistanceOverrides::default();
        }
        WritingAssistanceOverrides {
            spelling: Some(self.spelling.is_active()),
            grammar: Some(self.grammar.is_active()),
            offline_prediction: Some(self.offline_prediction.is_active()),
            cloud: Some(self.cloud.is_active()),
        }
    }

    pub fn text(&self) -> &str {
        &self.description
    }

    fn connect_controls(&self) {
        let value = self.clone();
        self.override_global.connect_active_notify(move |switch| {
            for control in [
                &value.spelling,
                &value.grammar,
                &value.offline_prediction,
                &value.cloud,
            ] {
                control.set_sensitive(switch.is_active());
            }
            value.emit_changed();
        });
        for control in [
            &self.spelling,
            &self.grammar,
            &self.offline_prediction,
            &self.cloud,
        ] {
            let value = self.clone();
            control.connect_active_notify(move |_| value.emit_changed());
        }
    }

    fn emit_changed(&self) {
        let values = self.overrides();
        for handler in self.handlers.borrow().iter() {
            handler(values.clone());
        }
    }
}

fn labelled_switch(container: &gtk::Box, label: &str, active: bool) -> gtk::Switch {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let title = gtk::Label::new(Some(label));
    title.set_halign(gtk::Align::Start);
    title.set_hexpand(true);
    let switch = gtk::Switch::builder()
        .active(active)
        .valign(gtk::Align::Center)
        .focusable(true)
        .build();
    switch.update_property(&[gtk::accessible::Property::Label(label)]);
    row.append(&title);
    row.append(&switch);
    container.append(&row);
    switch
}
