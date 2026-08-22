#![cfg(feature = "development")]

use adw::prelude::*;
use noor_notes::appearance::{AppearanceManager, AppearanceMode, AppearanceStore, EffectiveTheme};
use noor_notes::ui::app_header::AppHeader;
use noor_storage::NoteSort;

#[test]
fn development_theme_contrast_action_cycles_the_real_application_theme() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    ));
    manager.set_mode(AppearanceMode::Snow).unwrap();
    let app = noor_notes::identity::application();
    manager.install_theme_contrast_test_action(&app);

    let action = app
        .lookup_action("theme-contrast-test")
        .expect("development contrast action");
    assert_eq!(manager.effective_theme(), EffectiveTheme::Snow);
    action.activate(None);
    assert_eq!(manager.effective_theme(), EffectiveTheme::Midnight);
    action.activate(None);
    assert_eq!(manager.effective_theme(), EffectiveTheme::Snow);

    let header = AppHeader::new(manager, NoteSort::UpdatedDesc);
    let model = header.main_menu.menu_model().unwrap();
    assert!(menu_has_action(&model, "app.theme-contrast-test"));
}

fn menu_has_action(model: &gtk::gio::MenuModel, expected: &str) -> bool {
    for index in 0..model.n_items() {
        if model
            .item_attribute_value(
                index,
                gtk::gio::MENU_ATTRIBUTE_ACTION,
                Some(gtk::glib::VariantTy::STRING),
            )
            .and_then(|value| value.str().map(str::to_owned))
            .is_some_and(|action| action == expected)
        {
            return true;
        }
        for link in [gtk::gio::MENU_LINK_SECTION, gtk::gio::MENU_LINK_SUBMENU] {
            if model
                .item_link(index, link)
                .is_some_and(|child| menu_has_action(&child, expected))
            {
                return true;
            }
        }
    }
    false
}
