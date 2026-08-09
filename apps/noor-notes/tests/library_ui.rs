use std::rc::Rc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::library::LibrarySection;
use noor_notes::ui::library_sidebar::LibrarySidebar;
use noor_notes::ui::note_card;
use noor_notes::ui::note_collection::NoteCollection;

#[test]
fn redesigned_library_uses_sidebar_virtualized_list_and_cards() {
    gtk::init().unwrap();
    let sidebar = LibrarySidebar::new();
    assert!(sidebar.widget.has_css_class("nn-sidebar"));
    sidebar.set_count(LibrarySection::AllNotes, 42);
    let list = sidebar
        .widget
        .first_child()
        .and_then(|child| child.next_sibling())
        .and_downcast::<gtk::ListBox>()
        .unwrap();
    assert_eq!(list.observe_children().n_items(), 7);
    assert_eq!(list.selection_mode(), gtk::SelectionMode::Single);
    for index in 0..7 {
        let row = list.row_at_index(index).unwrap();
        assert!(row.tooltip_text().is_some());
    }

    let collection = NoteCollection::new(Rc::new(|_, _| {}));
    assert!(collection.widget.has_css_class("nn-note-list"));
    assert!(
        collection
            .widget
            .model()
            .and_downcast::<gtk::SingleSelection>()
            .is_some()
    );

    let mut note = Note::new(Utc::now());
    note.title = "A complete redesign".into();
    note.content = "First line\nSecond line\nThird line".into();
    note.tags = vec!["design".into(), "gtk".into(), "hidden".into()];
    collection.set_notes(&[note.clone()]);
    assert_eq!(collection.widget.model().unwrap().n_items(), 1);

    let card = note_card::build(&note, Rc::new(|_, _| {}));
    assert!(card.widget.has_css_class("nn-note-card"));
    assert!(!card.widget.has_css_class("boxed-list"));
}
