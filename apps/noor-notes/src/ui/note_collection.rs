use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use noor_domain::{Note, NoteId};

use super::note_card::{self, CardAction};

#[derive(Clone)]
pub struct NoteCollection {
    pub widget: gtk::ListView,
    model: gtk::StringList,
    notes: Rc<RefCell<HashMap<String, Note>>>,
}

impl NoteCollection {
    pub fn new(action: Rc<dyn Fn(NoteId, CardAction)>) -> Self {
        let model = gtk::StringList::new(&[]);
        let notes = Rc::new(RefCell::new(HashMap::<String, Note>::new()));
        let selection = gtk::SingleSelection::new(Some(model.clone()));
        selection.set_autoselect(true);
        selection.set_can_unselect(true);
        let factory = gtk::SignalListItemFactory::new();
        {
            let notes = notes.clone();
            factory.connect_bind(move |_, object| {
                let Some(item) = object.downcast_ref::<gtk::ListItem>() else {
                    return;
                };
                let Some(string) = item.item().and_downcast::<gtk::StringObject>() else {
                    return;
                };
                let notes = notes.borrow();
                let Some(note) = notes.get(string.string().as_str()) else {
                    return;
                };
                let card = note_card::build(note, action.clone());
                item.set_child(Some(&card.widget));
            });
        }
        factory.connect_unbind(|_, object| {
            if let Some(item) = object.downcast_ref::<gtk::ListItem>() {
                item.set_child(gtk::Widget::NONE);
            }
        });
        let widget = gtk::ListView::new(Some(selection), Some(factory));
        widget.add_css_class("nn-note-list");
        widget.update_property(&[gtk::accessible::Property::Label("Notes")]);
        widget.set_single_click_activate(false);
        widget.set_show_separators(false);
        Self {
            widget,
            model,
            notes,
        }
    }

    pub fn set_notes(&self, notes: &[Note]) {
        self.notes.borrow_mut().clear();
        self.model.splice(0, self.model.n_items(), &[]);
        let ids: Vec<String> = notes
            .iter()
            .map(|note| {
                let id = note.id.value().to_string();
                self.notes.borrow_mut().insert(id.clone(), note.clone());
                id
            })
            .collect();
        let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
        self.model.splice(0, 0, &ids);
    }

    pub fn update_note(&self, note: &Note) {
        let id = note.id.value().to_string();
        if self.notes.borrow().contains_key(&id) {
            self.notes.borrow_mut().insert(id, note.clone());
        }
    }

    pub fn connect_activate<F: Fn(Note) + 'static>(&self, callback: F) {
        let model = self.model.clone();
        let notes = self.notes.clone();
        self.widget.connect_activate(move |_, position| {
            let Some(id) = model.string(position) else {
                return;
            };
            if let Some(note) = notes.borrow().get(id.as_str()).cloned() {
                callback(note);
            }
        });
    }

    pub fn connect_selected<F: Fn(Option<Note>) + 'static>(&self, callback: F) {
        let Some(selection) = self.widget.model().and_downcast::<gtk::SingleSelection>() else {
            return;
        };
        let model = self.model.clone();
        let notes = self.notes.clone();
        selection.connect_selected_notify(move |selection| {
            let position = selection.selected();
            let note = if position == gtk::INVALID_LIST_POSITION {
                None
            } else {
                model
                    .string(position)
                    .and_then(|id| notes.borrow().get(id.as_str()).cloned())
            };
            callback(note);
        });
    }
}
