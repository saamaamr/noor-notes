use noor_notes::ui::adaptive_layout::{LibraryLayoutMode, LibraryPaneVisibility};

#[test]
fn width_breakpoints_choose_one_stable_library_mode() {
    assert_eq!(LibraryLayoutMode::for_width(1_200), LibraryLayoutMode::Wide);
    assert_eq!(LibraryLayoutMode::for_width(1_199), LibraryLayoutMode::Medium);
    assert_eq!(LibraryLayoutMode::for_width(760), LibraryLayoutMode::Medium);
    assert_eq!(LibraryLayoutMode::for_width(759), LibraryLayoutMode::Narrow);
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
