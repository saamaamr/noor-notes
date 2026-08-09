use std::rc::Rc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::library::LibrarySection;
use noor_notes::ui::empty_state::EmptyState;
use noor_notes::ui::library_sidebar::LibrarySidebar;
use noor_notes::ui::note_card;
use noor_notes::ui::note_collection::NoteCollection;
use noor_notes::ui::note_preview::NotePreview;

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

    sidebar.set_collapsed(true);
    assert_eq!(sidebar.widget.width_request(), 64);
    for index in 0..7 {
        let row = list.row_at_index(index).unwrap();
        let content = row.child().and_downcast::<gtk::Box>().unwrap();
        let label = content.first_child().unwrap().next_sibling().unwrap();
        let count = label.next_sibling().unwrap();
        assert!(!label.is_visible());
        assert!(!count.is_visible());
        assert!(row.tooltip_text().is_some());
    }
    sidebar.set_collapsed(false);
    assert_eq!(sidebar.widget.width_request(), 220);

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
    assert_eq!(card.menu.tooltip_text().as_deref(), Some("Note actions"));
    let descendants = descendants(card.widget.clone().upcast());
    assert!(descendants.iter().any(|widget| widget.has_css_class("nn-note-card-tags")));
    assert!(descendants.iter().any(|widget| widget.has_css_class("nn-note-card-meta")));

    let preview = NotePreview::new();
    assert!(preview.widget.has_css_class("nn-preview-surface"));
    preview.show_note(&note);
    preview.clear();
    let preview_text = label_texts(preview.widget.clone().upcast());
    assert!(preview_text.iter().any(|text| text == "Select a note"));
    assert!(preview_text.iter().any(|text| text.contains("Choose a note")));

    let empty = EmptyState::new();
    for (section, expected) in [
        (LibrarySection::AllNotes, "No notes yet"),
        (LibrarySection::Pinned, "No pinned notes"),
        (LibrarySection::Favorites, "No favorite notes"),
        (LibrarySection::Tags, "No tagged notes"),
        (LibrarySection::Archived, "Archive is empty"),
        (LibrarySection::Trash, "Trash is empty"),
        (LibrarySection::Recent, "No recent notes"),
    ] {
        empty.update(section, false);
        assert!(label_texts(empty.widget.clone().upcast()).iter().any(|text| text == expected));
    }
    empty.update(LibrarySection::AllNotes, true);
    assert!(label_texts(empty.widget.clone().upcast()).iter().any(|text| text == "No notes found"));
}

fn label_texts(root: gtk::Widget) -> Vec<String> {
    descendants(root)
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
        .map(|label| label.text().to_string())
        .collect()
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
