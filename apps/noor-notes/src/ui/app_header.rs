use std::rc::Rc;

use adw::prelude::*;
use noor_storage::NoteSort;

use crate::appearance::AppearanceManager;
use crate::library::LibrarySection;

use super::appearance_button::AppearanceButton;
use super::popover_primitives::style_popover;

#[derive(Clone)]
pub struct AppHeader {
    pub widget: adw::HeaderBar,
    pub back: gtk::Button,
    pub new_note: gtk::Button,
    pub search_toggle: gtk::ToggleButton,
    pub sort: gtk::DropDown,
    pub compact_sort: gtk::MenuButton,
    pub navigation: gtk::MenuButton,
    pub main_menu: gtk::MenuButton,
    new_note_label: gtk::Label,
    navigation_buttons: Vec<(LibrarySection, gtk::Button)>,
    compact_sort_buttons: Vec<gtk::Button>,
}

impl AppHeader {
    pub fn new(appearance: AppearanceManager, initial_sort: NoteSort) -> Self {
        let widget = adw::HeaderBar::new();
        widget.add_css_class("nn-app-header");
        widget.add_css_class("nn-surface");
        widget.set_title_widget(Some(&crate::identity::window_title()));

        let back = gtk::Button::builder()
            .icon_name("go-previous-symbolic")
            .tooltip_text("Back to notes")
            .visible(false)
            .build();
        style_header_icon(&back, "Back to notes");
        widget.pack_start(&back);

        let navigation_popover = gtk::Popover::new();
        style_popover(&navigation_popover);
        let navigation_content = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let mut navigation_buttons = Vec::new();
        for section in LibrarySection::NAVIGATION {
            let button = menu_row(section.label());
            navigation_content.append(&button);
            navigation_buttons.push((section, button));
        }
        navigation_popover.set_child(Some(&navigation_content));
        let navigation = gtk::MenuButton::builder()
            .icon_name("sidebar-show-symbolic")
            .tooltip_text("Choose library section")
            .popover(&navigation_popover)
            .visible(false)
            .build();
        style_header_menu(&navigation, "Choose library section");
        widget.pack_start(&navigation);

        let new_note_content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        new_note_content.append(&gtk::Image::from_icon_name("list-add-symbolic"));
        let new_note_label = gtk::Label::new(Some("New Note"));
        new_note_content.append(&new_note_label);
        let new_note = gtk::Button::builder()
            .child(&new_note_content)
            .tooltip_text("Create a new note (Ctrl+N)")
            .action_name("app.new-note")
            .build();
        new_note.add_css_class("suggested-action");
        new_note.add_css_class("nn-new-note");
        new_note.add_css_class("nn-control-primary");
        new_note.add_css_class("nn-h-36");
        new_note.add_css_class("nn-radius-8");
        new_note.add_css_class("nn-focus-ring");
        new_note.update_property(&[gtk::accessible::Property::Label("New Note")]);
        widget.pack_start(&new_note);

        let main_menu = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Main menu")
            .menu_model(&application_menu())
            .build();
        style_header_menu(&main_menu, "Main menu");
        widget.pack_end(&main_menu);

        let appearance_button = AppearanceButton::new(appearance);
        appearance_button.button.add_css_class("nn-header-control");
        appearance_button.button.add_css_class("nn-control-compact");
        widget.pack_end(&appearance_button.button);

        let sort = gtk::DropDown::from_strings(&[
            "Recently updated",
            "Recently created",
            "Title A–Z",
            "Title Z–A",
        ]);
        sort.set_selected(match initial_sort {
            NoteSort::UpdatedDesc => 0,
            NoteSort::CreatedDesc => 1,
            NoteSort::TitleAsc => 2,
            NoteSort::TitleDesc => 3,
        });
        sort.set_tooltip_text(Some("Sort notes"));
        sort.add_css_class("flat");
        sort.add_css_class("nn-sort-control");
        sort.add_css_class("nn-h-36");
        sort.add_css_class("nn-radius-8");
        sort.add_css_class("nn-focus-ring");
        sort.update_property(&[gtk::accessible::Property::Label("Sort notes")]);
        widget.pack_end(&sort);

        let sort_popover = gtk::Popover::new();
        style_popover(&sort_popover);
        let sort_content = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let mut compact_sort_buttons = Vec::new();
        for (index, label) in [
            "Recently updated",
            "Recently created",
            "Title A–Z",
            "Title Z–A",
        ]
        .into_iter()
        .enumerate()
        {
            let button = menu_row(label);
            let selected = index as u32 == sort.selected();
            button.update_state(&[gtk::accessible::State::Selected(Some(selected))]);
            if selected {
                button.add_css_class("nn-selected-menu-row");
            }
            {
                let sort = sort.clone();
                let sort_popover = sort_popover.clone();
                button.connect_clicked(move |_| {
                    sort.set_selected(index as u32);
                    sort_popover.popdown();
                });
            }
            sort_content.append(&button);
            compact_sort_buttons.push(button);
        }
        sort_popover.set_child(Some(&sort_content));
        let compact_sort = gtk::MenuButton::builder()
            .icon_name("view-sort-ascending-symbolic")
            .tooltip_text("Sort notes")
            .popover(&sort_popover)
            .visible(false)
            .build();
        style_header_menu(&compact_sort, "Sort notes");
        widget.pack_end(&compact_sort);
        {
            let buttons = compact_sort_buttons.clone();
            sort.connect_selected_notify(move |sort| {
                for (index, button) in buttons.iter().enumerate() {
                    let selected = index as u32 == sort.selected();
                    button.update_state(&[gtk::accessible::State::Selected(Some(selected))]);
                    if selected {
                        button.add_css_class("nn-selected-menu-row");
                    } else {
                        button.remove_css_class("nn-selected-menu-row");
                    }
                }
            });
        }

        let search_toggle = gtk::ToggleButton::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search notes (Ctrl+F)")
            .build();
        search_toggle.add_css_class("flat");
        search_toggle.add_css_class("nn-header-control");
        search_toggle.add_css_class("nn-control-compact");
        search_toggle.add_css_class("nn-icon-neutral");
        search_toggle.add_css_class("nn-icon-button");
        search_toggle.add_css_class("nn-focus-ring");
        search_toggle.update_property(&[gtk::accessible::Property::Label("Search notes (Ctrl+F)")]);
        super::toolbar_primitives::expose_toggle_checked(&search_toggle);
        widget.pack_end(&search_toggle);

        Self {
            widget,
            back,
            new_note,
            search_toggle,
            sort,
            compact_sort,
            navigation,
            main_menu,
            new_note_label,
            navigation_buttons,
            compact_sort_buttons,
        }
    }

    pub fn set_adaptive(&self, navigation_required: bool, compact: bool) {
        self.navigation.set_visible(navigation_required);
        self.new_note_label.set_visible(!compact);
        self.sort.set_visible(!compact);
        self.compact_sort.set_visible(compact);
    }

    pub fn is_compact(&self) -> bool {
        !self.sort.is_visible()
    }

    pub fn connect_navigation_selected<F: Fn(LibrarySection) + 'static>(&self, callback: F) {
        let callback = Rc::new(callback);
        for (section, button) in &self.navigation_buttons {
            let callback = callback.clone();
            let section = *section;
            let navigation = self.navigation.clone();
            button.connect_clicked(move |_| {
                callback(section);
                navigation.popdown();
            });
        }
    }

    pub fn navigation_button(&self, section: LibrarySection) -> Option<gtk::Button> {
        self.navigation_buttons
            .iter()
            .find_map(|(candidate, button)| (*candidate == section).then(|| button.clone()))
    }

    pub fn compact_sort_buttons(&self) -> &[gtk::Button] {
        &self.compact_sort_buttons
    }
}

fn menu_row(label: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("flat");
    button.add_css_class("nn-menu-row");
    button.set_halign(gtk::Align::Fill);
    button.update_property(&[gtk::accessible::Property::Label(label)]);
    button
}

fn style_header_icon(button: &gtk::Button, accessible_label: &str) {
    button.add_css_class("flat");
    button.add_css_class("nn-header-control");
    button.add_css_class("nn-control-compact");
    button.add_css_class("nn-icon-button");
    button.add_css_class("nn-h-32");
    button.add_css_class("nn-radius-6");
    button.add_css_class("nn-focus-ring");
    button.add_css_class("nn-icon-neutral");
    button.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
}

fn style_header_menu(button: &gtk::MenuButton, accessible_label: &str) {
    button.add_css_class("flat");
    button.add_css_class("nn-header-control");
    button.add_css_class("nn-control-compact");
    button.add_css_class("nn-icon-button");
    button.add_css_class("nn-h-32");
    button.add_css_class("nn-radius-6");
    button.add_css_class("nn-focus-ring");
    button.add_css_class("nn-icon-neutral");
    button.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
}

fn application_menu() -> gtk::gio::Menu {
    let menu = gtk::gio::Menu::new();
    menu.append(Some("Import Xpad Notes…"), Some("app.import-xpad"));
    menu.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));

    let appearance = gtk::gio::Menu::new();
    for (label, action) in [
        ("Snow", "app.appearance::snow"),
        ("Midnight", "app.appearance::midnight"),
    ] {
        appearance.append(Some(label), Some(action));
    }
    menu.append_submenu(Some("Appearance"), &appearance);
    menu.append(
        Some("Appearance Settings…"),
        Some("app.appearance-settings"),
    );
    menu.append(
        Some("Writing Assistance…"),
        Some("app.writing-assistance-settings"),
    );
    #[cfg(feature = "development")]
    menu.append(
        Some("Theme Contrast Test"),
        Some("app.theme-contrast-test"),
    );
    menu.append(Some("Quit"), Some("app.quit"));
    menu
}
