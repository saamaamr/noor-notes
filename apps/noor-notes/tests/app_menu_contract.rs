use adw::prelude::*;
use noor_notes::appearance::{AppearanceManager, AppearanceStore};
use noor_notes::ui::app_header::AppHeader;
use noor_storage::NoteSort;

#[test]
fn application_menu_exposes_only_canonical_theme_actions() {
    adw::init().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let manager = AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    ));
    let header = AppHeader::new(manager, NoteSort::UpdatedDesc);
    let model = header
        .main_menu
        .menu_model()
        .expect("application menu model");
    let mut actions = Vec::new();
    collect_actions(&model, &mut actions);

    for canonical in [
        "app.appearance::snow",
        "app.appearance::midnight",
        "app.account-settings",
    ] {
        assert!(actions.iter().any(|action| action == canonical));
    }
    assert_eq!(
        actions
            .iter()
            .filter(|action| action.as_str() == "app.account-settings")
            .count(),
        1
    );
    for historical in [
        "app.appearance::system",
        "app.appearance::light",
        "app.appearance::warm-paper",
        "app.appearance::cool-mist",
        "app.appearance::graphite",
        "app.appearance::oled",
    ] {
        assert!(!actions.iter().any(|action| action == historical));
    }
    assert!(
        !actions
            .iter()
            .any(|action| action == "app.theme-contrast-test"),
        "production menu must not expose developer theme diagnostics"
    );
}

fn collect_actions(model: &gtk::gio::MenuModel, actions: &mut Vec<String>) {
    for index in 0..model.n_items() {
        if let Some(action) = model
            .item_attribute_value(
                index,
                gtk::gio::MENU_ATTRIBUTE_ACTION,
                Some(gtk::glib::VariantTy::STRING),
            )
            .and_then(|value| value.str().map(str::to_owned))
        {
            let target = model
                .item_attribute_value(index, gtk::gio::MENU_ATTRIBUTE_TARGET, None)
                .and_then(|value| value.str().map(str::to_owned));
            actions.push(match target {
                Some(target) => format!("{action}::{target}"),
                None => action,
            });
        }
        for link in [gtk::gio::MENU_LINK_SECTION, gtk::gio::MENU_LINK_SUBMENU] {
            if let Some(child) = model.item_link(index, link) {
                collect_actions(&child, actions);
            }
        }
    }
}
