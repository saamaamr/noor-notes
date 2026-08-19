use std::rc::Rc;

use chrono::Utc;
use gtk::prelude::*;
use noor_domain::{Note, NoteState};
use noor_notes::ui::note_card::{self, CardAction};
use noor_notes::ui::note_collection::NoteCollection;

#[test]
fn archive_quick_action_exists_only_for_active_notes() {
    gtk::init().unwrap();
    let active = Note::new(Utc::now());
    let card = note_card::build(&active, Rc::new(|_, _| {}));
    assert!(
        descendants(card.widget.clone().upcast())
            .into_iter()
            .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
            .all(|button| button.tooltip_text().as_deref() != Some("Archive note"))
    );

    let mut archived = Note::new(Utc::now());
    archived.state = NoteState::Archived;
    let archived_card = note_card::build(
        &archived,
        Rc::new(|_, action| {
            assert_eq!(action, CardAction::Restore);
        }),
    );
    let archived_actions = descendants(archived_card.widget.clone().upcast())
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .filter(|button| button.tooltip_text().as_deref() == Some("Restore to All Notes"))
        .collect::<Vec<_>>();
    assert_eq!(archived_actions.len(), 1);
    archived_actions[0].emit_clicked();

    let mut trashed = Note::new(Utc::now());
    trashed.state = NoteState::Trashed {
        deleted_at: Utc::now(),
    };

    let first = Note::new(Utc::now());
    let second = Note::new(Utc::now());
    let collection = NoteCollection::new(Rc::new(|_, _| {}));
    collection.set_notes(&[first, second]);
    let window = gtk::Window::builder().child(&collection.widget).build();
    window.present();
    flush_gtk();

    let archive_buttons = descendants(collection.widget.clone().upcast())
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .filter(|button| button.tooltip_text().as_deref() == Some("Archive note"))
        .collect::<Vec<_>>();
    assert!(archive_buttons.is_empty());

    let selection = collection
        .widget
        .model()
        .and_downcast::<gtk::SingleSelection>()
        .unwrap();
    selection.set_selected(1);
    flush_gtk();
    window.close();
}

fn flush_gtk() {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
}

fn descendants(root: gtk::Widget) -> Vec<gtk::Widget> {
    let mut widgets = vec![root.clone()];
    let mut child = root.first_child();
    while let Some(current) = child {
        widgets.extend(descendants(current.clone()));
        child = current.next_sibling();
    }
    widgets
}
