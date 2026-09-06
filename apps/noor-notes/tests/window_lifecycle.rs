use std::cell::Cell;
use std::rc::Rc;

use adw::prelude::*;
use noor_notes::window_lifecycle::CachedWindow;

const MANAGED_APP: &str = include_str!("../src/managed_app.rs");

#[test]
fn main_window_reopening_actions_share_one_factory() {
    assert_eq!(
        MANAGED_APP
            .matches("MainWindow::new_with_sticky_session(")
            .count(),
        1
    );
    for action in ["new-note", "show-notes", "search"] {
        let action_source = MANAGED_APP
            .split_once(&format!("\"{action}\""))
            .expect("registered main-window action")
            .1;
        assert!(
            action_source[..action_source.len().min(500)].contains("present_or_create"),
            "{action} must recreate a closed main window"
        );
    }
}

#[test]
fn cached_application_window_can_close_reopen_and_close_again() {
    adw::init().unwrap();
    let app = adw::Application::builder()
        .application_id("io.github.saamaamr.NoorNotes.WindowLifecycleTest")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.register(None::<&gtk::gio::Cancellable>).unwrap();
    let cache = CachedWindow::new();
    let created = Rc::new(Cell::new(0));

    for expected_count in 1..=2 {
        cache.present_or_create(
            {
                let app = app.clone();
                let created = created.clone();
                move || {
                    created.set(created.get() + 1);
                    adw::ApplicationWindow::builder()
                        .application(&app)
                        .title("Lifecycle test")
                        .build()
                }
            },
            |window| window.clone().upcast(),
        );
        settle();

        assert_eq!(created.get(), expected_count);
        assert!(cache.is_alive());
        assert_eq!(app.windows().len(), 1);

        cache.with(|window| window.close());
        settle();

        assert!(!cache.is_alive(), "destroy must evict the cached wrapper");
        assert!(
            app.windows().is_empty(),
            "GTK must stop tracking the window"
        );
    }

    cache.present_or_create(
        {
            let app = app.clone();
            move || {
                adw::ApplicationWindow::builder()
                    .application(&app)
                    .title("Veto test")
                    .build()
            }
        },
        |window| window.clone().upcast(),
    );
    let window = cache.with(Clone::clone).expect("cached veto test window");
    let veto = window.connect_close_request(|_| gtk::glib::Propagation::Stop);
    window.close();
    settle();
    assert!(cache.is_alive(), "a vetoed close must remain cached");
    assert_eq!(app.windows().len(), 1);
    window.disconnect(veto);
    window.destroy();
    settle();
    assert!(!cache.is_alive(), "direct destroy must evict the cache");
    assert!(app.windows().is_empty());
}

fn settle() {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
        context.iteration(false);
    }
}
