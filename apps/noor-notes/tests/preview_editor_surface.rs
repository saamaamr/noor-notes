use adw::prelude::*;
use chrono::Utc;
use noor_domain::Note;
use noor_notes::ui::note_preview::NotePreview;

const PREVIEW: &str = include_str!("../src/ui/note_preview.rs");
const LIBRARY: &str = include_str!("../src/ui/library_window.rs");
const LIB: &str = include_str!("../src/lib.rs");

#[test]
fn preview_owns_shared_editor_surface_and_read_only_sticky_flow() {
    assert!(PREVIEW.contains("NoteEditorSurface"));
    assert!(PREVIEW.contains("on_read_only_changed"));
    assert!(PREVIEW.contains("title_entry"));
    assert!(PREVIEW.contains("EditorToolbar"));
    assert!(PREVIEW.contains("toolbar.format.set_tooltip_text(Some(\"Formatting\"))"));
    assert!(PREVIEW.contains("note.editor_mode = EditorMode::Rich"));
    assert!(PREVIEW.contains("set_sticky_read_only"));
    assert!(LIB.contains("sticky_note_window"));
    assert!(LIBRARY.contains("StickyNoteWindow"));
}

#[test]
fn library_actions_never_close_main_window() {
    let action_handler = LIBRARY
        .split("fn handle_card_action")
        .nth(1)
        .expect("MainWindow action handler");
    assert!(!action_handler.contains("window.close()"));
    assert!(!action_handler.contains("app.quit"));
}

#[test]
fn reading_and_editing_share_one_document_grid_without_exposing_form_chrome() {
    adw::init().unwrap();
    let mut note = Note::new(Utc::now());
    note.title = "Calm writing surface".into();
    note.content = "A".repeat(2_000);

    let preview = NotePreview::new();
    preview.show_note(&note);
    assert_eq!(preview.title_stack_child_name().as_deref(), Some("label"));
    assert!(!preview.toolbar_visible());

    preview.begin_editing();
    assert_eq!(preview.title_stack_child_name().as_deref(), Some("entry"));
    assert!(preview.toolbar_visible());
    assert_eq!(preview.editor().left_margin(), 8);
    assert_eq!(preview.editor().right_margin(), 8);
    assert_eq!(preview.editor().top_margin(), 5);
    assert_eq!(preview.editor().bottom_margin(), 5);
    assert!(preview.editor().tooltip_text().is_none());

    preview.finish_editing();
    assert_eq!(preview.title_stack_child_name().as_deref(), Some("label"));
    assert!(!preview.toolbar_visible());

    let window = gtk::Window::builder()
        .default_width(1_180)
        .default_height(720)
        .child(&preview.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}
    let document = descendants(preview.widget.clone().upcast())
        .into_iter()
        .find(|widget| widget.has_css_class("nn-preview"))
        .expect("document grid");
    let bounds = document
        .compute_bounds(&preview.widget)
        .expect("document bounds");
    assert!(bounds.width() <= 860.0, "document width={}", bounds.width());
    window.close();
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
