use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;

#[derive(Clone)]
pub struct EditorPresentation {
    editor: gtk::TextView,
    state_forces_read_only: bool,
    chrome: Vec<gtk::Widget>,
    previous_visibility: Rc<RefCell<Vec<bool>>>,
    view_only: Rc<Cell<bool>>,
}

impl EditorPresentation {
    pub fn new(
        editor: &gtk::TextView,
        state_forces_read_only: bool,
        chrome: Vec<gtk::Widget>,
    ) -> Self {
        Self {
            editor: editor.clone(),
            state_forces_read_only,
            chrome,
            previous_visibility: Rc::new(RefCell::new(Vec::new())),
            view_only: Rc::new(Cell::new(false)),
        }
    }

    pub fn set_view_only(&self, enabled: bool) {
        if enabled == self.view_only.get() {
            self.editor
                .set_editable(!enabled && !self.state_forces_read_only);
            return;
        }

        if enabled {
            self.previous_visibility.replace(
                self.chrome
                    .iter()
                    .map(gtk::prelude::WidgetExt::is_visible)
                    .collect(),
            );
            for widget in &self.chrome {
                widget.set_visible(false);
            }
            self.editor.set_editable(false);
            self.editor.set_cursor_visible(true);
            self.editor.grab_focus();
        } else {
            for (widget, visible) in self
                .chrome
                .iter()
                .zip(self.previous_visibility.borrow().iter().copied())
            {
                widget.set_visible(visible);
            }
            self.editor.set_editable(!self.state_forces_read_only);
        }
        self.view_only.set(enabled);
    }

    pub fn is_view_only(&self) -> bool {
        self.view_only.get()
    }
}
