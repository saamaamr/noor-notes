use adw::prelude::*;
use chrono::Utc;
use noor_domain::{EditorMode, Note, SourceLanguage};
use noor_notes::ui::note_preview::NotePreview;
use std::time::Duration;

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
    assert_eq!(
        preview.editor().left_margin(),
        preview.editor().right_margin()
    );
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
    preview.set_available_width(window.width());
    while gtk::glib::MainContext::default().iteration(false) {}
    let document = descendants(preview.widget.clone().upcast())
        .into_iter()
        .find(|widget| widget.has_css_class("nn-preview"))
        .expect("document grid");
    let bounds = document
        .compute_bounds(&preview.widget)
        .expect("document bounds");
    assert!(
        bounds.width() >= 1_160.0,
        "document width={}",
        bounds.width()
    );
    assert!(preview.content_maximum_width() <= 860);
    window.close();
    preview_chrome_rows_fill_workspace_while_controls_stay_compact();
    preview_metadata_keeps_tags_without_spending_space_on_edit_time();
    editor_ruler_controls_session_only_margins_without_default_gutters();
    long_editor_body_scrolls_without_moving_the_document_header();
    unwrapped_source_editor_exposes_horizontal_scrolling();
    maximized_editor_text_uses_the_workspace_without_automatic_gutters();
}

fn preview_metadata_keeps_tags_without_spending_space_on_edit_time() {
    let preview = NotePreview::new();
    let note = Note::new(Utc::now());
    preview.show_note(&note);
    let widgets = descendants(preview.widget.clone().upcast());
    let metadata = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-preview-metadata"))
        .expect("preview metadata")
        .clone()
        .downcast::<gtk::Label>()
        .expect("metadata label");
    assert!(!metadata.is_visible(), "empty metadata row must collapse");

    let mut tagged = note;
    tagged.tags = vec!["release".into(), "private".into()];
    preview.show_note(&tagged);
    assert!(metadata.is_visible());
    assert_eq!(metadata.text(), "#release  #private");
    assert!(!metadata.text().contains("Edited"));

    preview.begin_editing();
    let window = gtk::Window::builder()
        .default_width(900)
        .default_height(600)
        .child(&preview.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}
    let heading = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-preview-heading"))
        .expect("preview heading");
    let bounds = heading
        .compute_bounds(&preview.widget)
        .expect("preview heading bounds");
    assert!(
        bounds.height() <= 40.0,
        "compact title row must preserve writing height: {bounds:?}"
    );
    window.close();
}

fn editor_ruler_controls_session_only_margins_without_default_gutters() {
    let mut first = Note::new(Utc::now());
    first.title = "Ruler workspace".into();
    let preview = NotePreview::new();
    preview.show_note(&first);
    preview.begin_editing();
    preview.set_available_width(1_600);

    let window = gtk::Window::builder()
        .default_width(900)
        .default_height(700)
        .child(&preview.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    assert_eq!(preview.editor().left_margin(), 8);
    assert_eq!(preview.editor().right_margin(), 8);

    let widgets = descendants(preview.widget.clone().upcast());
    let ruler = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-editor-margin-ruler"))
        .expect("editor margin ruler");
    let button = widgets
        .iter()
        .filter_map(|widget| widget.clone().downcast::<gtk::MenuButton>().ok())
        .find(|button| button.tooltip_text().as_deref() == Some("Margins"))
        .expect("Margins must open from a compact button, not a permanent row");
    let popover = button.popover().expect("margin controls popover");
    assert_eq!(popover.position(), gtk::PositionType::Bottom);
    assert!(
        !ruler.is_mapped(),
        "Closed margins must not consume writing space"
    );
    button.popup();
    for _ in 0..20 {
        while gtk::glib::MainContext::default().iteration(false) {}
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(ruler.is_mapped());
    let left = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-editor-left-margin"))
        .expect("left margin control")
        .clone()
        .downcast::<gtk::Scale>()
        .expect("left margin scale");
    let right = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-editor-right-margin"))
        .expect("right margin control")
        .clone()
        .downcast::<gtk::Scale>()
        .expect("right margin scale");

    left.set_value(64.0);
    right.set_value(96.0);
    while gtk::glib::MainContext::default().iteration(false) {}
    assert_eq!(preview.editor().left_margin(), 72);
    assert_eq!(preview.editor().right_margin(), 104);

    let reset = widgets
        .iter()
        .filter_map(|widget| widget.clone().downcast::<gtk::Button>().ok())
        .find(|button| button.tooltip_text().as_deref() == Some("Reset margins"))
        .expect("Reset margins action");
    reset.emit_clicked();
    assert_eq!(preview.editor().left_margin(), 8);
    assert_eq!(preview.editor().right_margin(), 8);
    left.set_value(64.0);
    right.set_value(96.0);
    popover.popdown();
    preview.show_note(&first);
    assert_eq!(preview.editor().left_margin(), 72);
    assert_eq!(preview.editor().right_margin(), 104);

    preview.show_note(&Note::new(Utc::now()));
    assert_eq!(preview.editor().left_margin(), 8);
    assert_eq!(preview.editor().right_margin(), 8);
    window.close();
}

fn maximized_editor_text_uses_the_workspace_without_automatic_gutters() {
    let preview = NotePreview::new();
    preview.show_note(&Note::new(Utc::now()));
    preview.begin_editing();
    preview.set_available_width(1_600);
    let window = gtk::Window::builder()
        .default_width(1_600)
        .default_height(800)
        .child(&preview.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    let body_scroll = ancestor_scrolled_window(preview.editor().upcast())
        .expect("editor body must own an internal scroller");
    let body_bounds = body_scroll
        .compute_bounds(&preview.widget)
        .expect("editor body bounds");

    assert!(
        body_bounds.width() >= 1_480.0,
        "maximized writing surface must fill the workspace: body={body_bounds:?}"
    );
    assert!(
        preview.editor().left_margin() <= 16 && preview.editor().right_margin() <= 16,
        "maximized editor must not add automatic gutters: left={}, right={}",
        preview.editor().left_margin(),
        preview.editor().right_margin()
    );
    let editor_bounds = preview
        .editor()
        .compute_bounds(&preview.widget)
        .expect("maximized editor bounds");
    let text_width = editor_bounds.width()
        - preview.editor().left_margin() as f32
        - preview.editor().right_margin() as f32;
    assert!(
        text_width >= editor_bounds.width() - 32.0,
        "maximized text viewport must use the workspace: editor={editor_bounds:?}, text_width={text_width}"
    );

    window.close();

    let compact_preview = NotePreview::new();
    compact_preview.show_note(&Note::new(Utc::now()));
    compact_preview.begin_editing();
    compact_preview.set_available_width(620);
    let compact_window = gtk::Window::builder()
        .default_width(620)
        .default_height(480)
        .child(&compact_preview.widget)
        .build();
    compact_window.present();
    for _ in 0..80 {
        while gtk::glib::MainContext::default().iteration(false) {}
        std::thread::sleep(Duration::from_millis(5));
    }
    let compact_body_scroll = ancestor_scrolled_window(compact_preview.editor().upcast())
        .expect("compact editor body must own an internal scroller");
    let compact_bounds = compact_body_scroll
        .compute_bounds(&compact_preview.widget)
        .expect("compact editor body bounds");
    assert!(
        (560.0..=620.0).contains(&compact_bounds.width()),
        "compact writing surface must contract with the window: body={compact_bounds:?}"
    );
    assert!(
        compact_preview.editor().left_margin() <= 16
            && compact_preview.editor().right_margin() <= 16,
        "compact text margins must release workspace: left={}, right={}",
        compact_preview.editor().left_margin(),
        compact_preview.editor().right_margin()
    );
    compact_window.close();
}

fn preview_chrome_rows_fill_workspace_while_controls_stay_compact() {
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
    let toolbar = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-editor-toolbar"))
        .expect("preview editor toolbar");
    let heading = widgets
        .iter()
        .find(|widget| widget.has_css_class("nn-preview-heading"))
        .expect("preview heading");
    let document_bounds = document
        .compute_bounds(&preview.widget)
        .expect("document bounds");
    let menu_bounds = menu.compute_bounds(&preview.widget).expect("menu bounds");
    let toolbar_bounds = toolbar
        .compute_bounds(&preview.widget)
        .expect("toolbar bounds");
    let heading_bounds = heading
        .compute_bounds(&preview.widget)
        .expect("heading bounds");

    for (name, bounds) in [("heading", heading_bounds), ("menu", menu_bounds)] {
        assert!(
            bounds.width() >= document_bounds.width() * 0.95,
            "{name} row must fill the workspace: row={bounds:?}, document={document_bounds:?}"
        );
    }
    let menu_controls = visible_children_span(menu, &preview.widget);
    let toolbar_controls = visible_children_span(toolbar, &preview.widget);
    assert!(
        menu_controls < menu_bounds.width() * 0.7,
        "menu controls must remain compact: controls={menu_controls}, menu={menu_bounds:?}"
    );
    assert!(
        toolbar_bounds.width() <= toolbar_controls + 24.0,
        "toolbar must fit its controls without a large empty tail: controls={toolbar_controls}, toolbar={toolbar_bounds:?}"
    );
    assert!(
        (toolbar_bounds.x() - heading_bounds.x()).abs() <= 1.0,
        "toolbar must align with the heading"
    );
    assert!(toolbar_bounds.width() < document_bounds.width() * 0.75);
    window.close();
}

fn visible_children_span(container: &gtk::Widget, relative_to: &impl IsA<gtk::Widget>) -> f32 {
    let mut child = container.first_child();
    let mut left = f32::MAX;
    let mut right = f32::MIN;
    while let Some(widget) = child {
        if widget.is_visible() {
            let bounds = widget
                .compute_bounds(relative_to)
                .expect("visible child bounds");
            left = left.min(bounds.x());
            right = right.max(bounds.x() + bounds.width());
        }
        child = widget.next_sibling();
    }
    right - left
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
