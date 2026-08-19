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
    assert_eq!(sidebar.widget.width_request(), 180);
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
        let content = row.child().and_downcast::<gtk::Box>().unwrap();
        let icon = content.first_child().unwrap();
        let label = icon.next_sibling().unwrap();
        let count = label.next_sibling().unwrap();
        assert!(icon.has_css_class("nn-sidebar-icon"));
        assert!(label.has_css_class("nn-sidebar-label"));
        assert!(count.has_css_class("nn-sidebar-count"));
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
    assert_eq!(sidebar.widget.width_request(), 180);

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
    note.content = format!("{} বাংলা نص عربي", "A".repeat(300));
    note.tags = vec!["design".into(), "gtk".into(), "hidden".into()];
    collection.set_notes(&[note.clone()]);
    assert_eq!(collection.widget.model().unwrap().n_items(), 1);

    let activated = Rc::new(std::cell::RefCell::new(Vec::<Note>::new()));
    collection.connect_activate({
        let activated = activated.clone();
        move |note| activated.borrow_mut().push(note)
    });
    let mut edited = note.clone();
    edited.content = "Edited from the preview".into();
    collection.update_note(&edited);
    collection.widget.emit_by_name::<()>("activate", &[&0_u32]);
    assert_eq!(
        activated
            .borrow()
            .last()
            .map(|note| (note.id, note.content.as_str())),
        Some((note.id, "Edited from the preview")),
        "selecting the note again must use the preview's latest cached content"
    );

    let card = note_card::build(&note, Rc::new(|_, _| {}));
    assert!(card.widget.has_css_class("nn-note-card"));
    assert!(!card.widget.has_css_class("boxed-list"));
    let color_rail = card.widget.first_child().expect("note color rail");
    assert!(color_rail.has_css_class("nn-color-strip"));
    assert_eq!(color_rail.width_request(), 4);
    assert!(card.menu.has_css_class("nn-card-action"));
    assert_eq!(card.menu.valign(), gtk::Align::Center);
    let archive = card.archive.as_ref().expect("active card archive action");
    assert!(archive.has_css_class("nn-card-action"));
    assert_eq!(archive.valign(), gtk::Align::Center);
    assert_eq!(card.menu.tooltip_text().as_deref(), Some("Note actions"));
    let card_descendants = descendants(card.widget.clone().upcast());
    assert!(
        card_descendants
            .iter()
            .any(|widget| widget.has_css_class("nn-note-card-preview"))
    );
    assert!(
        card_descendants
            .iter()
            .any(|widget| widget.has_css_class("nn-note-card-tags"))
    );
    assert!(
        card_descendants
            .iter()
            .any(|widget| widget.has_css_class("nn-note-card-meta"))
    );

    let preview = NotePreview::new();
    assert!(preview.widget.has_css_class("nn-preview-surface"));
    preview.set_compact(true);
    assert!(preview.widget.has_css_class("compact"));
    preview.set_compact(false);
    assert!(!preview.widget.has_css_class("compact"));
    preview.show_note(&note);
    let preview_labels: Vec<gtk::Label> = descendants(preview.widget.clone().upcast())
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Label>().ok())
        .collect();
    let title = preview_labels
        .iter()
        .find(|label| label.has_css_class("nn-preview-title"))
        .expect("preview title");
    let metadata = preview_labels
        .iter()
        .find(|label| label.has_css_class("nn-preview-metadata"))
        .expect("preview metadata");
    let body = preview_labels
        .iter()
        .find(|label| label.has_css_class("nn-preview-body"))
        .expect("preview body");
    assert_eq!(title.wrap_mode(), gtk::pango::WrapMode::WordChar);
    assert_eq!(metadata.wrap_mode(), gtk::pango::WrapMode::WordChar);
    assert_eq!(body.wrap_mode(), gtk::pango::WrapMode::WordChar);
    assert!(body.is_selectable());
    preview.clear();
    let preview_text = label_texts(preview.widget.clone().upcast());
    assert!(preview_text.iter().any(|text| text == "Select a note"));
    assert!(
        preview_text
            .iter()
            .any(|text| text.contains("Choose a note"))
    );
    let preview_window = gtk::Window::builder()
        .default_width(1_400)
        .default_height(760)
        .child(&preview.widget)
        .build();
    preview_window.present();
    while gtk::glib::MainContext::default().iteration(false) {}
    preview.widget.allocate(1_400, 760, -1, None);
    let document = descendants(preview.widget.clone().upcast())
        .into_iter()
        .find(|widget| widget.has_css_class("nn-preview"))
        .expect("preview document");
    let bounds = document
        .compute_bounds(&preview.widget)
        .expect("preview document bounds");
    assert!(
        bounds.width() >= 640.0,
        "wide preview document collapsed to {}px at x={}",
        bounds.width(),
        bounds.x()
    );
    assert!(
        bounds.width() <= 860.0,
        "wide preview document exceeded its readable 860px limit: {}px",
        bounds.width()
    );
    preview_window.close();

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
        assert!(
            label_texts(empty.widget.clone().upcast())
                .iter()
                .any(|text| text == expected)
        );
    }
    empty.update(LibrarySection::AllNotes, true);
    assert!(
        label_texts(empty.widget.clone().upcast())
            .iter()
            .any(|text| text == "No notes found")
    );
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
