use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{Note, NoteState, RichBlock, RichDocument, RichSpan, TextMarks};
use noor_notes::appearance::{AppearanceManager, AppearanceStore, install_global};
use noor_notes::ui::note_preview::NotePreview;

#[test]
fn active_note_body_edits_inline_and_preserves_rich_content() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    install_global(AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    )));
    let document = RichDocument {
        version: 1,
        blocks: vec![RichBlock {
            spans: vec![
                RichSpan {
                    text: "Important".into(),
                    marks: TextMarks {
                        bold: true,
                        ..TextMarks::default()
                    },
                },
                RichSpan {
                    text: " body".into(),
                    marks: TextMarks::default(),
                },
            ],
            ..RichBlock::default()
        }],
    };
    let mut note = Note::new(Utc::now());
    note.title = "Read-only title".into();
    note.content = document.plain_text();
    note.rich_content = Some(document);

    let saved = Rc::new(RefCell::new(Vec::<Note>::new()));
    let finished = Rc::new(RefCell::new(Vec::new()));
    let preview = NotePreview::new_with_handlers(
        {
            let saved = saved.clone();
            Rc::new(move |note| saved.borrow_mut().push(note))
        },
        {
            let finished = finished.clone();
            Rc::new(move |id| finished.borrow_mut().push(id))
        },
    );
    preview.show_note(&note);

    let widgets = descendants(preview.widget.clone().upcast());
    let edit = widgets
        .iter()
        .find_map(|widget| widget.clone().downcast::<gtk::Button>().ok())
        .filter(|button| button.has_css_class("nn-preview-edit"))
        .expect("body Edit button");
    let editor = widgets
        .iter()
        .find_map(|widget| widget.clone().downcast::<gtk::TextView>().ok())
        .filter(|editor| editor.has_css_class("nn-preview-editor"))
        .expect("inline body editor");
    let body = widgets
        .iter()
        .filter_map(|widget| widget.clone().downcast::<gtk::Label>().ok())
        .find(|label| label.has_css_class("nn-preview-body"))
        .expect("read-only body preview");

    assert_eq!(edit.label().as_deref(), Some("Edit"));
    assert!(!editor.is_editable());
    edit.emit_clicked();
    assert_eq!(edit.label().as_deref(), Some("Done"));
    assert!(editor.is_editable());
    assert_eq!(editor.left_margin(), 8);
    assert_eq!(editor.right_margin(), 8);
    assert_eq!(editor.top_margin(), 5);
    assert_eq!(editor.bottom_margin(), 5);

    let buffer = editor.buffer();
    let mut end = buffer.end_iter();
    buffer.insert(&mut end, "!");
    let latest = saved.borrow().last().cloned().expect("edited draft");
    assert_eq!(latest.title, "Read-only title");
    assert_eq!(latest.content, "Important body!");
    let rich = latest.rich_content.expect("rich document remains native");
    assert!(rich.blocks[0].spans[0].marks.bold);

    edit.emit_clicked();
    assert_eq!(edit.label().as_deref(), Some("Edit"));
    assert!(!editor.is_editable());
    assert_eq!(body.text(), "Important body!");
    assert_eq!(finished.borrow().as_slice(), &[note.id]);

    let save_count = saved.borrow().len();
    let mut view_only = note.clone();
    view_only.editor_preferences.view_only = true;
    preview.show_note(&view_only);
    edit.emit_clicked();
    assert!(editor.is_editable());
    assert_eq!(saved.borrow().len(), save_count + 1);
    assert!(
        !saved
            .borrow()
            .last()
            .expect("view-only exit draft")
            .editor_preferences
            .view_only
    );
    edit.emit_clicked();

    let save_count = saved.borrow().len();
    let mut trashed = note.clone();
    trashed.content = "Deleted body".into();
    trashed.state = NoteState::Trashed {
        deleted_at: Utc::now(),
    };
    preview.show_note(&trashed);
    assert!(!edit.is_sensitive());
    assert!(!editor.is_editable());
    assert_eq!(saved.borrow().len(), save_count);
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
