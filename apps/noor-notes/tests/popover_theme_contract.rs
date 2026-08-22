use std::rc::Rc;

use adw::prelude::*;
use chrono::Utc;
use noor_domain::{EditorMode, Note, WritingAssistanceOverrides};
use noor_notes::appearance::{AppearanceManager, AppearanceStore};
use noor_notes::ui::app_header::AppHeader;
use noor_notes::ui::editor_menu_bar::EditorMenuBar;
use noor_notes::ui::editor_toolbar::EditorToolbar;
use noor_notes::ui::note_card;
use noor_notes::ui::note_writing_assistance::NoteWritingAssistancePopover;
use noor_notes::ui::popover_primitives::themed_popover;
use noor_notes::writing_assistance::{
    IssuePopover, PredictionOverlay, WritingAssistancePreferences,
};
use noor_storage::NoteSort;

#[test]
fn every_application_owned_popover_uses_the_shared_theme_surface() {
    gtk::init().unwrap();

    let child = gtk::Label::new(Some("Themed content"));
    assert_themed(&themed_popover(&child));

    let toolbar = EditorToolbar::new();
    assert_themed(&toolbar.formatting.widget);
    assert_themed(&toolbar.emoji_popover);
    assert_themed(&toolbar.more_popover);
    assert_menu_button_popover(&toolbar.appearance);

    let menu_bar = EditorMenuBar::new(&toolbar);
    for popover in menu_bar.popovers() {
        assert_themed(popover);
    }

    let directory = tempfile::tempdir().unwrap();
    let appearance = AppearanceManager::new(AppearanceStore::at(
        directory.path().join("appearance.json"),
    ));
    let header = AppHeader::new(appearance, NoteSort::UpdatedDesc);
    assert_menu_button_popover(&header.navigation);
    assert_menu_button_popover(&header.compact_sort);

    let card = note_card::build(&Note::new(Utc::now()), Rc::new(|_, _| {}));
    assert_menu_button_popover(&card.menu);

    let note_assistance = NoteWritingAssistancePopover::new(
        &WritingAssistancePreferences::default(),
        &WritingAssistanceOverrides::default(),
        EditorMode::Rich,
    );
    assert_themed(&note_assistance.widget);

    let issue = IssuePopover::new("Grammar", "Use a clearer phrase", &["Replace".into()]);
    assert_themed(&issue.widget);

    let overlay = gtk::Overlay::new();
    let view = gtk::TextView::new();
    overlay.set_child(Some(&view));
    let _prediction = PredictionOverlay::new(&overlay, &view);
    let prediction_popovers = descendants(view.upcast_ref())
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Popover>().ok())
        .collect::<Vec<_>>();
    assert_eq!(prediction_popovers.len(), 1);
    assert_themed(&prediction_popovers[0]);
}

fn assert_menu_button_popover(button: &gtk::MenuButton) {
    let popover = button.popover().expect("menu button popover");
    let popover = popover.downcast::<gtk::Popover>().unwrap();
    assert_themed(&popover);
}

fn assert_themed(popover: &gtk::Popover) {
    assert!(
        popover.has_css_class("nn-menu-surface"),
        "{} missed the shared semantic popover surface",
        popover.type_().name()
    );
}

fn descendants(root: &gtk::Widget) -> Vec<gtk::Widget> {
    let mut result = Vec::new();
    let mut next = root.first_child();
    while let Some(child) = next {
        result.push(child.clone());
        result.extend(descendants(&child));
        next = child.next_sibling();
    }
    result
}
