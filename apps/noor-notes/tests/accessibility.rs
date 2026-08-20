use std::rc::Rc;

use adw::prelude::*;
use noor_domain::Note;
use noor_notes::appearance::{AppearanceManager, AppearanceStore};
use noor_notes::ui::editor_menu_bar::EditorMenuBar;
use noor_notes::ui::editor_toolbar::EditorToolbar;
use noor_notes::ui::library_sidebar::LibrarySidebar;
use noor_notes::ui::note_card::CardAction;
use noor_notes::ui::note_collection::NoteCollection;

const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn professional_controls_expose_native_accessibility_contracts() {
    gtk::init().unwrap();
    assert_editor_icon_controls();
    assert_navigation_and_note_selection();
    assert_menu_bar_and_popovers();
    assert_reduced_motion_setting();
}

fn assert_editor_icon_controls() {
    let toolbar = EditorToolbar::new();
    let controls = [
        toolbar.new_note.upcast_ref::<gtk::Widget>(),
        toolbar.undo.upcast_ref(),
        toolbar.redo.upcast_ref(),
        toolbar.pin.upcast_ref(),
        toolbar.format.upcast_ref(),
        toolbar.emoji.upcast_ref(),
        toolbar.find.upcast_ref(),
        toolbar.appearance.upcast_ref(),
        toolbar.zoom_in.upcast_ref(),
        toolbar.zoom_out.upcast_ref(),
        toolbar.zoom_reset.upcast_ref(),
        toolbar.go_to_line.upcast_ref(),
        toolbar.fullscreen.upcast_ref(),
        toolbar.rename.upcast_ref(),
        toolbar.archive.upcast_ref(),
        toolbar.trash.upcast_ref(),
        toolbar.more.upcast_ref(),
    ];
    for control in controls {
        assert!(
            control.tooltip_text().is_some(),
            "icon control is missing a tooltip: {:?}",
            control
        );
        assert!(
            gtk::test_accessible_has_property(control, gtk::AccessibleProperty::Label),
            "icon control is missing an explicit accessible label: {:?}",
            control
        );
    }
    for control in [
        toolbar.quick_font_size.upcast_ref::<gtk::Widget>(),
        toolbar.opacity.upcast_ref(),
    ] {
        assert!(gtk::test_accessible_has_property(
            control,
            gtk::AccessibleProperty::Label
        ));
    }
    for toggle in [
        &toolbar.pin,
        &toolbar.bold,
        &toolbar.italic,
        &toolbar.quick_underline,
        &toolbar.quick_strikethrough,
        &toolbar.bullets,
        &toolbar.quick_numbered,
        &toolbar.find,
    ] {
        assert!(gtk::test_accessible_has_state(
            toggle,
            gtk::AccessibleState::Checked
        ));
    }
}

fn assert_navigation_and_note_selection() {
    let sidebar = LibrarySidebar::new();
    let rows = sidebar.navigation_rows();
    assert_eq!(rows.len(), 7);
    for row in &rows {
        assert!(gtk::test_accessible_has_property(
            row,
            gtk::AccessibleProperty::Label
        ));
        assert!(gtk::test_accessible_has_state(
            row,
            gtk::AccessibleState::Selected
        ));
    }

    let collection = NoteCollection::new(Rc::new(|_, _: CardAction| {}));
    collection.set_notes(&[Note::new(chrono::Utc::now())]);
    let window = gtk::Window::builder()
        .default_width(360)
        .default_height(420)
        .child(&collection.widget)
        .build();
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}
    collection.widget.allocate(360, 420, -1, None);
    while gtk::glib::MainContext::default().iteration(false) {}
    let item = descendants(collection.widget.clone().upcast())
        .into_iter()
        .find(|widget| widget.has_css_class("nn-note-item"))
        .expect("virtualized note item");
    assert!(gtk::test_accessible_has_state(
        &item,
        gtk::AccessibleState::Selected
    ));
    window.close();
}

fn assert_menu_bar_and_popovers() {
    let toolbar = EditorToolbar::new();
    let menu = EditorMenuBar::new(&toolbar);
    assert_eq!(menu.menu_buttons().len(), 6);
    for button in menu.menu_buttons() {
        assert!(button.tooltip_text().is_some());
        assert!(gtk::test_accessible_has_property(
            button,
            gtk::AccessibleProperty::Label
        ));
    }
    assert!(menu.popovers().iter().all(gtk::Popover::is_autohide));
}

#[test]
fn reduced_motion_css_disables_nonessential_transitions() {
    assert!(CSS.contains(".nn-reduced-motion"));
    assert!(CSS.contains("transition: none"));
    assert!(CSS.contains("button:focus-visible"));
    assert!(CSS.contains("outline: 2px solid @nn_focus"));
}

fn assert_reduced_motion_setting() {
    let settings = gtk::Settings::default().expect("GTK settings");
    let previous = settings.is_gtk_enable_animations();
    settings.set_gtk_enable_animations(false);
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    ));
    let window = gtk::Window::new();
    manager.register_window(&window);
    assert!(window.has_css_class("nn-reduced-motion"));
    settings.set_gtk_enable_animations(true);
    while gtk::glib::MainContext::default().iteration(false) {}
    assert!(!window.has_css_class("nn-reduced-motion"));
    settings.set_gtk_enable_animations(previous);
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
