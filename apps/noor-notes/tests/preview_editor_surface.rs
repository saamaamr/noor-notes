use adw::prelude::*;
use chrono::Utc;
use noor_domain::{EditorMode, Note, SourceLanguage};
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
    assert!(!PREVIEW.contains("note.editor_mode = EditorMode::Rich"));
    assert!(PREVIEW.contains("source_palette::apply"));
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
    preview_menu_controls_fit_their_content_instead_of_spanning_the_document();
    long_editor_body_scrolls_without_moving_the_document_header();
    unwrapped_source_editor_exposes_horizontal_scrolling();
}

fn preview_menu_controls_fit_their_content_instead_of_spanning_the_document() {
    let preview = NotePreview::new();
    preview.show_note(&Note::new(Utc::now()));
    preview.begin_editing();
    preview.set_available_width(900);
    let window = gtk::Window::builder()
        .default_width(900)
        .default_height(600)
        .child(&preview.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    let widgets = descendants(preview.widget.clone().upcast());
    let document = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-preview"))
        .expect("preview document");
    let menu = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-editor-menu-bar"))
        .expect("preview editor menu");
    let document_bounds = document
        .compute_bounds(&preview.widget)
        .expect("document bounds");
    let menu_bounds = menu.compute_bounds(&preview.widget).expect("menu bounds");

    assert!(
        menu_bounds.width() < document_bounds.width() * 0.6,
        "compact menu consumed the document width: menu={menu_bounds:?}, document={document_bounds:?}"
    );
    window.close();
}

fn long_editor_body_scrolls_without_moving_the_document_header() {
    let mut note = Note::new(Utc::now());
    note.title = "Long writing session".into();
    note.content = (0..400)
        .map(|line| format!("Line {line}: focused writing stays inside the editor body."))
        .collect::<Vec<_>>()
        .join("\n");

    let preview = NotePreview::new();
    preview.show_note(&note);
    preview.begin_editing();
    preview.set_available_width(900);
    let window = gtk::Window::builder()
        .default_width(900)
        .default_height(600)
        .child(&preview.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    let heading = descendants(preview.widget.clone().upcast())
        .into_iter()
        .find(|widget| widget.has_css_class("nn-preview-heading"))
        .expect("document heading");
    let body_scroll = ancestor_scrolled_window(preview.editor().upcast())
        .expect("editor body must own an internal scroller");
    let heading_bounds = heading
        .compute_bounds(&preview.widget)
        .expect("document heading bounds");
    let scroll_bounds = body_scroll
        .compute_bounds(&preview.widget)
        .expect("editor body scroller bounds");
    assert!(
        scroll_bounds.y() >= heading_bounds.y() + heading_bounds.height(),
        "editor body scroller must start below the fixed header: heading={heading_bounds:?}, scroll={scroll_bounds:?}"
    );
    let adjustment = body_scroll.vadjustment();
    assert!(
        adjustment.upper() > adjustment.page_size() + 100.0,
        "long note must scroll inside the editor body: upper={}, page={}",
        adjustment.upper(),
        adjustment.page_size()
    );
    let before = heading
        .compute_bounds(&preview.widget)
        .expect("heading bounds before body scroll")
        .y();
    adjustment.set_value(100.0);
    while gtk::glib::MainContext::default().iteration(false) {}
    let after = heading
        .compute_bounds(&preview.widget)
        .expect("heading bounds after body scroll")
        .y();

    assert!(
        (after - before).abs() < 1.0,
        "body scrolling moved the document header from {before} to {after}"
    );
    window.close();
}

fn unwrapped_source_editor_exposes_horizontal_scrolling() {
    let mut note = Note::new(Utc::now());
    note.editor_mode = EditorMode::Code;
    note.editor_preferences.word_wrap = false;
    note.source_language = SourceLanguage::new("rust").unwrap();
    note.content = format!("let uninterrupted_line = \"{}\";", "x".repeat(4_000));

    let preview = NotePreview::new();
    preview.show_note(&note);
    preview.begin_editing();
    preview.set_available_width(620);
    let window = gtk::Window::builder()
        .default_width(620)
        .default_height(480)
        .child(&preview.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    let body_scroll =
        ancestor_scrolled_window(preview.editor().upcast()).expect("source editor body scroller");
    let adjustment = body_scroll.hadjustment();
    assert!(
        adjustment.upper() > adjustment.page_size() + 100.0,
        "unwrapped source line must exceed the visible page: upper={}, page={}",
        adjustment.upper(),
        adjustment.page_size()
    );
    assert!(
        body_scroll.hscrollbar().is_mapped(),
        "unwrapped source mode must expose its horizontal scrollbar"
    );
    window.close();
}

fn ancestor_scrolled_window(mut widget: gtk::Widget) -> Option<gtk::ScrolledWindow> {
    while let Some(parent) = widget.parent() {
        if let Ok(scrolled) = parent.clone().downcast::<gtk::ScrolledWindow>() {
            return Some(scrolled);
        }
        widget = parent;
    }
    None
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
