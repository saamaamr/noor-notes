use noor_notes::appearance::{AppearanceManager, AppearanceStore};

#[test]
fn appearance_preferences_can_load_before_gtk_initialization() {
    let directory = tempfile::tempdir().unwrap();
    let store = AppearanceStore::at(directory.path().join("appearance.json"));
    let manager = AppearanceManager::new(store);
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.StartupTest")
        .build();
    manager.install_action(&app);
}
