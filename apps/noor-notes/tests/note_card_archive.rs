use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use chrono::Utc;
use gtk::prelude::*;
use noor_domain::{Note, NoteState};
use noor_notes::ui::library_sidebar::LibrarySidebar;
use noor_notes::ui::note_card::{self, CardAction};
use noor_notes::ui::note_collection::NoteCollection;

#[test]
fn archive_quick_action_exists_only_for_active_notes() {
    gtk::init().unwrap();
    let active = Note::new(Utc::now());
    let card = note_card::build(&active, Rc::new(|_, _| {}));
    assert!(
        descendants(card.widget.clone().upcast())
            .into_iter()
            .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
            .all(|button| button.tooltip_text().as_deref() != Some("Archive note"))
    );

    let mut archived = Note::new(Utc::now());
    archived.state = NoteState::Archived;
    let archived_card = note_card::build(
        &archived,
        Rc::new(|_, action| {
            assert_eq!(action, CardAction::Restore);
        }),
    );
    let archived_actions = descendants(archived_card.widget.clone().upcast())
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .filter(|button| button.tooltip_text().as_deref() == Some("Restore to All Notes"))
        .collect::<Vec<_>>();
    assert_eq!(archived_actions.len(), 1);
    archived_actions[0].emit_clicked();

    let mut trashed = Note::new(Utc::now());
    trashed.state = NoteState::Trashed {
        deleted_at: Utc::now(),
    };

    let first = Note::new(Utc::now());
    let second = Note::new(Utc::now());
    let collection = NoteCollection::new(Rc::new(|_, _| {}));
    collection.set_notes(&[first, second]);
    let window = gtk::Window::builder().child(&collection.widget).build();
    window.present();
    flush_gtk();

    let archive_buttons = descendants(collection.widget.clone().upcast())
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Button>().ok())
        .filter(|button| button.tooltip_text().as_deref() == Some("Archive note"))
        .collect::<Vec<_>>();
    assert!(archive_buttons.is_empty());

    let selection = collection
        .widget
        .model()
        .and_downcast::<gtk::SingleSelection>()
        .unwrap();
    selection.set_selected(1);
    flush_gtk();
    window.close();
    card_action_waits_until_the_actions_popover_is_closed();
    sidebar_provides_a_stable_focus_target_for_card_removal();
}

fn sidebar_provides_a_stable_focus_target_for_card_removal() {
    let sidebar = LibrarySidebar::new();
    let collection = NoteCollection::new(Rc::new(|_, _| {}));
    collection.set_notes(&[Note::new(Utc::now())]);
    let layout = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    layout.append(&sidebar.widget);
    layout.append(&collection.widget);
    let window = gtk::Window::builder().child(&layout).build();
    window.present();
    flush_gtk();

    assert!(sidebar.focus_selected());
    flush_gtk();
    let focused = gtk::prelude::RootExt::focus(&window).expect("stable sidebar focus target");
    assert!(
        sidebar
            .navigation_rows()
            .iter()
            .any(|row| focused == row.clone().upcast::<gtk::Widget>())
    );

    collection.set_notes(&[]);
    flush_gtk();
    assert!(
        gtk::prelude::RootExt::focus(&window).is_some(),
        "model refresh must keep valid focus"
    );
    window.close();
}

fn card_action_waits_until_the_actions_popover_is_closed() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let main_loop = gtk::glib::MainLoop::new(None, false);
    let note = Note::new(Utc::now());
    let card = note_card::build(&note, {
        let events = events.clone();
        let main_loop = main_loop.clone();
        Rc::new(move |_, action| {
            assert_eq!(action, CardAction::Archive);
            events.borrow_mut().push("action");
            main_loop.quit();
        })
    });
    let popover = card
        .menu
        .popover()
        .and_downcast::<gtk::Popover>()
        .expect("note actions popover");
    popover.connect_closed({
        let events = events.clone();
        move |_| events.borrow_mut().push("closed")
    });
    let archive = card
        .action_button(CardAction::Archive)
        .expect("active note archive action");
    let window = gtk::Window::builder().child(&card.widget).build();
    window.present();
    popover.popup();
    flush_gtk();

    archive.emit_clicked();
    assert_eq!(
        events.borrow().as_slice(),
        ["closed"],
        "the note mutation must not begin while GTK is still closing the popover"
    );
    flush_gtk();
    assert_eq!(
        events.borrow().as_slice(),
        ["closed"],
        "the note mutation must wait for GTK's popover focus transition to settle"
    );
    let backend = gtk::gdk::Display::default()
        .map(|display| display.type_().name().to_string())
        .unwrap_or_default();
    if backend.contains("Wayland") {
        // A nested test MainLoop on Wayland does not dispatch a local timeout
        // created by a popover's `closed` signal. The production application
        // uses its outer loop; the X11/Xvfb release gate verifies completion.
        window.close();
        return;
    }
    let guard_loop = main_loop.clone();
    gtk::glib::timeout_add_local_once(Duration::from_secs(1), move || guard_loop.quit());
    main_loop.run();
    assert_eq!(events.borrow().as_slice(), ["closed", "action"]);
    window.close();
}

fn flush_gtk() {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
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
