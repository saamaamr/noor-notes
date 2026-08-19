#![cfg(feature = "development")]

use adw::prelude::*;

#[test]
fn development_application_uses_a_distinct_desktop_identity() {
    let app = noor_notes::identity::application();

    assert_eq!(
        app.application_id().as_deref(),
        Some("io.github.saamaamr.NoorNotes.Devel")
    );
}

#[test]
fn development_window_title_is_impossible_to_confuse_with_store_build() {
    gtk::init().unwrap();
    let title = noor_notes::identity::window_title();

    assert_eq!(title.title(), "Noor Notes Dev");
    assert_eq!(title.subtitle(), "Development build · Private notebook");
}
