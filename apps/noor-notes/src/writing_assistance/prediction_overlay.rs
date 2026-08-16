use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use gtk::gdk;

#[derive(Clone)]
pub struct PredictionOverlay {
    inner: Rc<PredictionOverlayInner>,
}

struct PredictionOverlayInner {
    view: gtk::TextView,
    buffer: gtk::TextBuffer,
    ghost: gtk::Label,
    ghost_layer: gtk::Fixed,
    popover: gtk::Popover,
    alternatives: gtk::Box,
    announcement: gtk::Label,
    suggestions: RefCell<Vec<String>>,
    selected: Cell<usize>,
}

impl PredictionOverlay {
    pub fn new(canvas: &gtk::Overlay, view: &gtk::TextView) -> Self {
        let ghost = gtk::Label::new(None);
        ghost.add_css_class("nn-prediction-ghost");
        ghost.set_halign(gtk::Align::Start);
        ghost.set_valign(gtk::Align::Start);
        ghost.set_can_target(false);
        ghost.set_visible(false);

        let ghost_layer = gtk::Fixed::new();
        ghost_layer.set_can_target(false);
        ghost_layer.put(&ghost, 0.0, 0.0);
        canvas.add_overlay(&ghost_layer);

        let alternatives = gtk::Box::new(gtk::Orientation::Vertical, 4);
        alternatives.set_margin_top(8);
        alternatives.set_margin_bottom(8);
        alternatives.set_margin_start(8);
        alternatives.set_margin_end(8);
        let popover = gtk::Popover::builder().child(&alternatives).build();
        popover.set_parent(view);
        popover.set_autohide(true);

        let announcement = gtk::Label::new(None);
        announcement.set_visible(false);
        canvas.add_overlay(&announcement);

        if !gtk::Settings::default().is_some_and(|settings| settings.is_gtk_enable_animations()) {
            ghost.add_css_class("nn-reduced-motion");
        }

        let value = Self {
            inner: Rc::new(PredictionOverlayInner {
                view: view.clone(),
                buffer: view.buffer(),
                ghost,
                ghost_layer,
                popover,
                alternatives,
                announcement,
                suggestions: RefCell::new(Vec::new()),
                selected: Cell::new(0),
            }),
        };
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let prediction = value.clone();
        keys.connect_key_pressed(move |_, key, _, state| prediction.handle_key(key, state));
        view.add_controller(keys);

        let prediction = value.clone();
        view.connect_has_focus_notify(move |view| {
            if !view.has_focus() && !prediction.inner.popover.is_visible() {
                prediction.dismiss();
            }
        });
        let prediction = value.clone();
        value.inner.buffer.connect_mark_set(move |_, _, mark| {
            if matches!(mark.name().as_deref(), Some("insert" | "selection_bound")) {
                prediction.dismiss();
            }
        });
        value
    }

    pub fn show(&self, suggestions: &[String]) {
        let mut seen = std::collections::BTreeSet::new();
        let suggestions = suggestions
            .iter()
            .filter_map(|suggestion| {
                if suggestion.chars().any(char::is_control) {
                    return None;
                }
                let suggestion = suggestion.trim();
                (!suggestion.is_empty() && suggestion.chars().count() <= 256)
                    .then(|| suggestion.to_owned())
            })
            .filter(|suggestion| seen.insert(suggestion.to_lowercase()))
            .take(3)
            .collect::<Vec<_>>();
        if suggestions.is_empty() {
            self.dismiss();
            return;
        }
        self.inner.suggestions.replace(suggestions.clone());
        self.inner.selected.set(0);
        self.inner.ghost.set_text(&suggestions[0]);
        self.position_at_cursor();
        self.inner.ghost.set_visible(true);
        self.set_announcement(&format!(
            "Suggestion: {}. Press Tab to accept or Alt+Down for alternatives.",
            suggestions[0]
        ));
        self.rebuild_alternatives();
    }

    pub fn dismiss(&self) {
        let was_visible = self.is_visible();
        self.inner.ghost.set_visible(false);
        self.inner.popover.popdown();
        self.inner.suggestions.borrow_mut().clear();
        if was_visible {
            self.set_announcement("Suggestion dismissed");
        }
    }

    pub fn show_alternatives(&self) {
        if self.inner.suggestions.borrow().is_empty() {
            return;
        }
        self.inner.popover.popup();
    }

    pub fn accept_selected(&self) -> bool {
        let Some(suggestion) = self
            .inner
            .suggestions
            .borrow()
            .get(self.inner.selected.get())
            .cloned()
        else {
            return false;
        };
        self.inner.buffer.begin_user_action();
        self.inner.buffer.insert_at_cursor(&suggestion);
        self.inner.buffer.end_user_action();
        self.dismiss();
        self.inner.view.grab_focus();
        true
    }

    pub fn handle_key(&self, key: gdk::Key, state: gdk::ModifierType) -> gtk::glib::Propagation {
        if self.inner.suggestions.borrow().is_empty() {
            return gtk::glib::Propagation::Proceed;
        }
        if state.contains(gdk::ModifierType::ALT_MASK) && key == gdk::Key::Down {
            self.show_alternatives();
            return gtk::glib::Propagation::Stop;
        }
        if key == gdk::Key::Escape {
            self.dismiss();
            return gtk::glib::Propagation::Stop;
        }
        if self.inner.popover.is_visible() {
            if key == gdk::Key::Down {
                let last = self.inner.suggestions.borrow().len().saturating_sub(1);
                self.inner
                    .selected
                    .set((self.inner.selected.get() + 1).min(last));
                self.update_selected_style();
                return gtk::glib::Propagation::Stop;
            }
            if key == gdk::Key::Up {
                self.inner
                    .selected
                    .set(self.inner.selected.get().saturating_sub(1));
                self.update_selected_style();
                return gtk::glib::Propagation::Stop;
            }
            if matches!(key, gdk::Key::Return | gdk::Key::KP_Enter) {
                self.accept_selected();
                return gtk::glib::Propagation::Stop;
            }
        }
        if key == gdk::Key::Tab {
            self.accept_selected();
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    }

    pub fn suggestions(&self) -> Vec<String> {
        self.inner.suggestions.borrow().clone()
    }

    pub fn is_visible(&self) -> bool {
        self.inner.ghost.is_visible() || self.inner.popover.is_visible()
    }

    pub fn announcement(&self) -> String {
        self.inner.announcement.text().to_string()
    }

    fn rebuild_alternatives(&self) {
        while let Some(child) = self.inner.alternatives.first_child() {
            self.inner.alternatives.remove(&child);
        }
        for (index, suggestion) in self.inner.suggestions.borrow().iter().enumerate() {
            let button = gtk::Button::with_label(suggestion);
            button.set_halign(gtk::Align::Fill);
            button.update_property(&[gtk::accessible::Property::Label(&format!(
                "Use suggestion {suggestion}"
            ))]);
            if index == self.inner.selected.get() {
                button.add_css_class("suggested-action");
            }
            let prediction = self.clone();
            button.connect_clicked(move |_| {
                prediction.inner.selected.set(index);
                prediction.accept_selected();
            });
            self.inner.alternatives.append(&button);
        }
    }

    fn update_selected_style(&self) {
        let mut child = self.inner.alternatives.first_child();
        let mut index = 0;
        while let Some(widget) = child {
            if index == self.inner.selected.get() {
                widget.add_css_class("suggested-action");
            } else {
                widget.remove_css_class("suggested-action");
            }
            child = widget.next_sibling();
            index += 1;
        }
    }

    fn position_at_cursor(&self) {
        let iter = self
            .inner
            .buffer
            .iter_at_offset(self.inner.buffer.cursor_position());
        let location = self.inner.view.iter_location(&iter);
        let (x, y) = self.inner.view.buffer_to_window_coords(
            gtk::TextWindowType::Widget,
            location.x() + location.width(),
            location.y(),
        );
        self.inner
            .ghost_layer
            .move_(&self.inner.ghost, f64::from(x + 4), f64::from(y));
        self.inner
            .popover
            .set_pointing_to(Some(&gtk::gdk::Rectangle::new(
                x,
                y,
                location.width().max(1),
                location.height().max(1),
            )));
    }

    fn set_announcement(&self, text: &str) {
        self.inner.announcement.set_text(text);
        self.inner
            .announcement
            .update_property(&[gtk::accessible::Property::Label(text)]);
    }
}

impl Drop for PredictionOverlayInner {
    fn drop(&mut self) {
        self.popover.unparent();
    }
}
