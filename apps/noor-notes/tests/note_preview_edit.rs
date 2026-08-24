use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{
    EditorMode, Note, NoteState, RichBlock, RichDocument, RichSpan, SourceLanguage, TextMarks,
};
use noor_notes::appearance::{AppearanceManager, AppearanceStore, install_global};
use noor_notes::ui::note_preview::NotePreview;
use sourceview5::prelude::*;

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
    let title_entry = widgets
        .iter()
        .find_map(|widget| widget.clone().downcast::<gtk::Entry>().ok())
        .filter(|entry| entry.has_css_class("nn-preview-title-entry"))
        .expect("inline title editor");
    let body = widgets
        .iter()
        .filter_map(|widget| widget.clone().downcast::<gtk::Label>().ok())
        .find(|label| label.has_css_class("nn-preview-body"))
        .expect("read-only body preview");

    assert_eq!(edit.label().as_deref(), Some("Edit"));
    assert!(!editor.is_editable());
    assert!(!title_entry.is_editable());
    edit.emit_clicked();
    assert_eq!(edit.label().as_deref(), Some("Done"));
    assert!(editor.is_editable());
    assert!(title_entry.is_editable());
    assert_eq!(editor.left_margin(), 8);
    assert_eq!(editor.right_margin(), 8);
    assert_eq!(editor.top_margin(), 5);
    assert_eq!(editor.bottom_margin(), 5);

    title_entry.set_text("Renamed note");
    assert_eq!(
        saved.borrow().last().map(|note| note.title.as_str()),
        Some("Renamed note")
    );

    let buffer = editor.buffer();
    let mut end = buffer.end_iter();
    buffer.insert(&mut end, "!");
    let latest = saved.borrow().last().cloned().expect("edited draft");
    assert_eq!(latest.title, "Renamed note");
    assert_eq!(latest.content, "Important body!");
    let rich = latest.rich_content.expect("rich document remains native");
    assert!(rich.blocks[0].spans[0].marks.bold);

    edit.emit_clicked();
    assert_eq!(edit.label().as_deref(), Some("Edit"));
    assert!(!editor.is_editable());
    assert!(!title_entry.is_editable());
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

    let mut code = Note::new(Utc::now());
    code.title = "Rust example".into();
    code.content = "fn main() {}".into();
    code.editor_mode = EditorMode::Code;
    code.source_language = SourceLanguage::new("rust").unwrap();
    code.editor_preferences.word_wrap = false;
    preview.show_note(&code);
    assert_eq!(preview.active_mode(), EditorMode::Code);
    assert_eq!(
        preview.source_buffer().language().unwrap().id().as_str(),
        "rust"
    );
    assert!(preview.source_view().shows_line_numbers());
    assert_eq!(preview.source_view().wrap_mode(), gtk::WrapMode::None);
    preview.begin_editing();
    assert!(!preview.toolbar().bold.is_visible());
    assert!(!preview.toolbar().bold.is_sensitive());
    let code_buffer = preview.source_buffer();
    let mut code_end = code_buffer.end_iter();
    code_buffer.insert(&mut code_end, "\n// saved mode");
    let code_draft = saved.borrow().last().cloned().expect("code draft");
    assert_eq!(code_draft.editor_mode, EditorMode::Code);
    assert!(code_draft.rich_content.is_none());
    assert!(code_draft.content.ends_with("// saved mode"));
    assert!(preview.toolbar().undo.is_sensitive());
    preview.toolbar().undo.emit_clicked();
    assert_eq!(
        preview
            .source_buffer()
            .text(
                &preview.source_buffer().start_iter(),
                &preview.source_buffer().end_iter(),
                true,
            )
            .as_str(),
        "fn main() {}"
    );
    preview.toolbar().redo.emit_clicked();
    assert!(
        preview
            .source_buffer()
            .text(
                &preview.source_buffer().start_iter(),
                &preview.source_buffer().end_iter(),
                true,
            )
            .ends_with("// saved mode")
    );
    preview.finish_editing();

    let mut markdown = Note::new(Utc::now());
    markdown.content = "# Heading".into();
    markdown.editor_mode = EditorMode::Markdown;
    markdown.source_language = SourceLanguage::Markdown;
    preview.show_note(&markdown);
    assert_eq!(preview.active_mode(), EditorMode::Markdown);
    assert_eq!(
        preview.source_buffer().language().unwrap().id().as_str(),
        "markdown"
    );
    assert!(preview.source_buffer().is_highlight_syntax());
    preview.begin_editing();
    assert!(preview.toolbar().emoji.is_visible());
    assert!(!preview.toolbar().bold.is_visible());
    preview.finish_editing();

    let mut plain = Note::new(Utc::now());
    plain.content = "Plain text".into();
    plain.editor_mode = EditorMode::PlainText;
    preview.show_note(&plain);
    assert_eq!(preview.active_mode(), EditorMode::PlainText);
    assert!(preview.source_buffer().language().is_none());
    assert!(!preview.source_buffer().is_highlight_syntax());
    let mode_requests = Rc::new(RefCell::new(Vec::new()));
    preview.connect_editor_mode_requested({
        let mode_requests = mode_requests.clone();
        move |note, target| mode_requests.borrow_mut().push((note.id, target))
    });
    preview.begin_editing();
    preview.toolbar().mode_rich.emit_clicked();
    assert_eq!(
        mode_requests.borrow().as_slice(),
        &[(plain.id, EditorMode::Rich)]
    );
    preview.finish_editing();

    let mut note = Note::new(Utc::now());
    note.title = "Sticky state".into();
    preview.show_note(&note);

    preview.set_read_only_open(true);
    assert_eq!(preview.read_only_label(), "Exit read-only");
    assert!(preview.is_read_only_open());

    preview.set_read_only_open(false);
    assert_eq!(preview.read_only_label(), "Read-only");
    assert!(!preview.is_read_only_open());
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
