use adw::prelude::*;
use noor_notes::ui::adaptive_layout::{
    LibraryLayoutMode, LibraryPaneVisibility, apply_paned_layout,
};

#[test]
fn width_breakpoints_choose_one_stable_library_mode() {
    assert_eq!(LibraryLayoutMode::for_width(1_180), LibraryLayoutMode::Wide);
    assert_eq!(
        LibraryLayoutMode::for_width(1_079),
        LibraryLayoutMode::Medium
    );
    assert_eq!(LibraryLayoutMode::for_width(760), LibraryLayoutMode::Medium);
    assert_eq!(LibraryLayoutMode::for_width(759), LibraryLayoutMode::Narrow);
    assert_eq!(
        LibraryLayoutMode::for_window_width(720, 1_180),
        LibraryLayoutMode::Narrow
    );
    assert_eq!(
        LibraryLayoutMode::for_window_width(1, 1_180),
        LibraryLayoutMode::Wide
    );
}

#[test]
fn wide_and_medium_modes_keep_content_visible_without_squeezing_navigation() {
    assert_eq!(
        LibraryLayoutMode::Wide.visibility(false),
        LibraryPaneVisibility {
            sidebar: true,
            collection: true,
            content: true,
            back: false,
        }
    );
    assert_eq!(
        LibraryLayoutMode::Medium.visibility(false),
        LibraryPaneVisibility {
            sidebar: false,
            collection: true,
            content: true,
            back: false,
        }
    );
}

#[test]
fn narrow_mode_switches_between_collection_and_content_with_back_navigation() {
    assert_eq!(
        LibraryLayoutMode::Narrow.visibility(false),
        LibraryPaneVisibility {
            sidebar: false,
            collection: true,
            content: false,
            back: false,
        }
    );
    assert_eq!(
        LibraryLayoutMode::Narrow.visibility(true),
        LibraryPaneVisibility {
            sidebar: false,
            collection: false,
            content: true,
            back: true,
        }
    );
}

#[test]
fn narrow_paned_allocation_gives_the_visible_preview_the_window_width() {
    gtk::init().unwrap();
    let window = gtk::Window::builder()
        .default_width(620)
        .default_height(480)
        .build();
    let panes = gtk::Paned::new(gtk::Orientation::Horizontal);
    let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let preview = gtk::Box::new(gtk::Orientation::Vertical, 0);
    panes.set_start_child(Some(&navigation));
    panes.set_end_child(Some(&preview));
    window.set_child(Some(&panes));
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    apply_paned_layout(
        &panes,
        &navigation,
        &preview,
        LibraryLayoutMode::Narrow,
        620,
        true,
    );
    panes.allocate(620, 480, -1, None);
    while gtk::glib::MainContext::default().iteration(false) {}

    let position = panes.position();
    let preview_width = preview.width();
    assert!(!navigation.is_visible());
    assert!(preview.is_visible());
    assert!(
        preview_width > 500,
        "position={position} preview={preview_width}"
    );
    window.close();
}
