use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;

/// Owns one recreatable application window and forgets it when GTK destroys it.
#[derive(Clone)]
pub struct CachedWindow<T> {
    value: Rc<RefCell<Option<(gtk::Window, T)>>>,
    observing_application: Rc<Cell<bool>>,
}

impl<T> Default for CachedWindow<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> CachedWindow<T> {
    pub fn new() -> Self {
        Self {
            value: Rc::new(RefCell::new(None)),
            observing_application: Rc::new(Cell::new(false)),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.value.borrow().is_some()
    }
}

impl<T: Clone + 'static> CachedWindow<T> {
    pub fn with<R>(&self, use_window: impl FnOnce(&T) -> R) -> Option<R> {
        let managed = self
            .value
            .borrow()
            .as_ref()
            .map(|(_, window)| window.clone());
        managed.as_ref().map(use_window)
    }

    pub fn present_or_create(
        &self,
        create: impl FnOnce() -> T,
        gtk_window: impl Fn(&T) -> gtk::Window,
    ) {
        if self.value.borrow().is_none() {
            let managed = create();
            let window = gtk_window(&managed);
            let value = Rc::downgrade(&self.value);
            if let (false, Some(application)) =
                (self.observing_application.get(), window.application())
            {
                application.connect_window_removed({
                    let value = value.clone();
                    move |_, removed| evict_if_current(&value, removed)
                });
                self.observing_application.set(true);
            }
            window.connect_destroy(move |destroyed| {
                evict_if_current(&value, destroyed);
            });
            self.value.replace(Some((window, managed)));
        }

        let window = self
            .value
            .borrow()
            .as_ref()
            .map(|(window, _)| window.clone());
        if let Some(window) = window {
            window.present();
        }
    }
}

fn evict_if_current<T>(
    value: &std::rc::Weak<RefCell<Option<(gtk::Window, T)>>>,
    closing: &gtk::Window,
) {
    let Some(value) = value.upgrade() else {
        return;
    };
    let is_current = value
        .borrow()
        .as_ref()
        .is_some_and(|(cached, _)| cached == closing);
    if is_current {
        value.borrow_mut().take();
    }
}
