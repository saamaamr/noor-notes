use adw::prelude::*;
use noor_storage::NoteSort;

use crate::appearance::AppearanceManager;

use super::appearance_button::AppearanceButton;

#[derive(Clone)]
pub struct AppHeader {
    pub widget: adw::HeaderBar,
    pub back: gtk::Button,
    pub new_note: gtk::Button,
    pub search_toggle: gtk::ToggleButton,
    pub sort: gtk::DropDown,
    pub main_menu: gtk::MenuButton,
    new_note_label: gtk::Label,
}

impl AppHeader {
    pub fn new(appearance: AppearanceManager, initial_sort: NoteSort) -> Self {
        let widget = adw::HeaderBar::new();
        widget.add_css_class("nn-app-header");
        widget.set_title_widget(Some(&crate::identity::window_title()));

        let back = gtk::Button::builder()
            .icon_name("go-previous-symbolic")
            .tooltip_text("Back to notes")
            .visible(false)
            .build();
        style_header_icon(&back, "Back to notes");
        widget.pack_start(&back);

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
        sort.update_property(&[gtk::accessible::Property::Label("Sort notes")]);
        widget.pack_end(&sort);

        let search_toggle = gtk::ToggleButton::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search notes (Ctrl+F)")
            .build();
        search_toggle.add_css_class("flat");
        search_toggle.add_css_class("nn-header-control");
        search_toggle.add_css_class("nn-control-compact");
        search_toggle.add_css_class("nn-icon-neutral");
        search_toggle.update_property(&[gtk::accessible::Property::Label("Search notes (Ctrl+F)")]);
        super::toolbar_primitives::expose_toggle_checked(&search_toggle);
        widget.pack_end(&search_toggle);

        Self {
            widget,
            back,
            new_note,
            search_toggle,
            sort,
            main_menu,
            new_note_label,
        }
    }

    pub fn set_compact(&self, compact: bool) {
        self.new_note_label.set_visible(!compact);
        self.sort.set_visible(!compact);
    }

    pub fn is_compact(&self) -> bool {
        !self.sort.is_visible()
    }
}

fn style_header_icon(button: &gtk::Button, accessible_label: &str) {
    button.add_css_class("flat");
    button.add_css_class("nn-header-control");
    button.add_css_class("nn-control-compact");
    button.add_css_class("nn-icon-neutral");
    button.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
}

fn style_header_menu(button: &gtk::MenuButton, accessible_label: &str) {
    button.add_css_class("flat");
    button.add_css_class("nn-header-control");
    button.add_css_class("nn-control-compact");
    button.add_css_class("nn-icon-neutral");
    button.update_property(&[gtk::accessible::Property::Label(accessible_label)]);
}

fn application_menu() -> gtk::gio::Menu {
    let menu = gtk::gio::Menu::new();
    menu.append(Some("Import Xpad Notes…"), Some("app.import-xpad"));
    menu.append(Some("Keyboard Shortcuts"), Some("app.shortcuts"));

    let appearance = gtk::gio::Menu::new();
    for (label, action) in [
        ("System", "app.appearance::system"),
        ("Snow", "app.appearance::light"),
        ("Warm Paper", "app.appearance::warm-paper"),
        ("Cool Mist", "app.appearance::cool-mist"),
        ("Graphite", "app.appearance::graphite"),
        ("Midnight", "app.appearance::midnight"),
        ("OLED", "app.appearance::oled"),
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
    menu.append(Some("Quit"), Some("app.quit"));
    menu
}
