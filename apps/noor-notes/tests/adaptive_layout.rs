use adw::prelude::*;
use noor_notes::ui::adaptive_layout::{
    LibraryLayoutMode, LibraryPaneVisibility, allocation_for_width, apply_library_layout,
};

#[test]
fn wide_allocation_targets_ten_twenty_seventy_with_readability_guards() {
    let standard = allocation_for_width(LibraryLayoutMode::Wide, 1_180, false);
    assert_eq!(standard.sidebar, 160);
    assert_eq!(standard.collection, 280);
    assert_eq!(standard.navigation, 440);

    let large = allocation_for_width(LibraryLayoutMode::Wide, 1_920, false);
    assert_eq!(large.sidebar, 192);
    assert_eq!(large.collection, 360);
    assert_eq!(large.navigation, 552);
}

#[test]
fn medium_and_narrow_allocations_prioritize_the_visible_destination() {
    let medium = allocation_for_width(LibraryLayoutMode::Medium, 900, false);
    assert_eq!(medium.sidebar, 0);
    assert_eq!(medium.collection, 306);
    assert_eq!(medium.navigation, 306);

    assert_eq!(
        allocation_for_width(LibraryLayoutMode::Narrow, 620, true).navigation,
        0
    );
    assert_eq!(
        allocation_for_width(LibraryLayoutMode::Narrow, 620, false).navigation,
        620
    );
}

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
    assert_eq!(LibraryLayoutMode::Wide.pane_position(1_180, false), 440);
    assert_eq!(LibraryLayoutMode::Medium.pane_position(900, false), 306);
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
fn real_shell_allocation_tracks_ratios_and_gives_narrow_preview_the_window_width() {
    gtk::init().unwrap();
    let window = gtk::Window::builder()
        .default_width(1_180)
        .default_height(480)
        .build();
    let panes = gtk::Paned::new(gtk::Orientation::Horizontal);
    let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let collection = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let preview = gtk::Box::new(gtk::Orientation::Vertical, 0);
    navigation.append(&sidebar);
    navigation.append(&collection);
    panes.set_start_child(Some(&navigation));
    panes.set_end_child(Some(&preview));
    panes.set_resize_start_child(false);
    panes.set_shrink_start_child(false);
    window.set_child(Some(&panes));
    window.present();
    while gtk::glib::MainContext::default().iteration(false) {}

    apply_library_layout(
        &panes,
        &navigation,
        &sidebar,
        &collection,
        &preview,
        LibraryLayoutMode::Wide,
        1_180,
        false,
    );
    panes.allocate(1_180, 480, -1, None);
    while gtk::glib::MainContext::default().iteration(false) {}

    assert_eq!(sidebar.width_request(), 160);
    assert_eq!(collection.width_request(), 280);
    assert_eq!(panes.position(), 440);
    assert!(preview.width() >= 700, "preview={}", preview.width());

    apply_library_layout(
        &panes,
        &navigation,
        &sidebar,
        &collection,
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
