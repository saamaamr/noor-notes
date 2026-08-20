use std::collections::HashMap;

use adw::prelude::*;
use gtk::glib;

use super::editor_toolbar::EditorToolbar;

/// Compact document-editor menu row. Every item proxies a live toolbar
/// control, so menu, toolbar, and keyboard paths keep one implementation.
#[derive(Clone)]
pub struct EditorMenuBar {
    pub widget: gtk::Box,
    items: HashMap<&'static str, gtk::Button>,
    checked_items: HashMap<&'static str, gtk::ToggleButton>,
    _bindings: Vec<glib::Binding>,
    _popovers: Vec<gtk::Popover>,
}

impl EditorMenuBar {
    pub fn new(toolbar: &EditorToolbar) -> Self {
        Self::build(toolbar, true)
    }

    pub fn new_preview(toolbar: &EditorToolbar) -> Self {
        Self::build(toolbar, false)
    }

    pub fn item(&self, key: &str) -> gtk::Button {
        self.items
            .get(key)
            .unwrap_or_else(|| panic!("unknown editor menu item: {key}"))
            .clone()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.items.contains_key(key)
    }

    pub fn item_checked(&self, key: &str) -> bool {
        self.checked_items
            .get(key)
            .is_some_and(|item| item.is_active())
    }

    fn build(toolbar: &EditorToolbar, full: bool) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        widget.add_css_class("nn-editor-menu-bar");
        let mut menus = full_menu_definitions(toolbar);
        if !full {
            menus.retain(|menu| matches!(menu.label, "Edit" | "Insert" | "Format"));
            if let Some(edit) = menus.iter_mut().find(|menu| menu.label == "Edit") {
                edit.items.retain(|item| item.key != "edit.find");
            }
        }

        let mut items = HashMap::new();
        let mut checked_items = HashMap::new();
        let mut bindings = Vec::new();
        let mut popovers = Vec::new();
        for definition in menus {
            let built = build_menu(definition);
            for item in built.items {
                bindings.extend(item.bindings);
                if let Some(toggle) = item.checked {
                    checked_items.insert(item.key, toggle);
                }
                items.insert(item.key, item.button);
            }
            widget.append(&built.button);
            popovers.push(built.popover);
        }
        close_other_popovers_when_opened(&popovers);

        Self {
            widget,
            items,
            checked_items,
            _bindings: bindings,
            _popovers: popovers,
        }
    }
}

struct MenuDefinition {
    label: &'static str,
    items: Vec<MenuItemDefinition>,
}

struct MenuItemDefinition {
    key: &'static str,
    label: &'static str,
    source: MenuSource,
}

enum MenuSource {
    Button(gtk::Button),
    Toggle(gtk::ToggleButton),
    Menu(gtk::MenuButton),
}

struct BuiltMenu {
    button: gtk::MenuButton,
    popover: gtk::Popover,
    items: Vec<BuiltItem>,
}

struct BuiltItem {
    key: &'static str,
    button: gtk::Button,
    checked: Option<gtk::ToggleButton>,
    bindings: Vec<glib::Binding>,
}

fn full_menu_definitions(toolbar: &EditorToolbar) -> Vec<MenuDefinition> {
    vec![
        MenuDefinition {
            label: "File",
            items: vec![
                button_item("file.new-note", "New note", &toolbar.new_note),
                button_item("file.duplicate", "Duplicate", &toolbar.duplicate),
                button_item(
                    "file.export-text",
                    "Export plain text",
                    &toolbar.export_text,
                ),
                button_item(
                    "file.export-markdown",
                    "Export Markdown",
                    &toolbar.export_markdown,
                ),
                button_item("file.delete", "Move to Trash", &toolbar.trash),
            ],
        },
        MenuDefinition {
            label: "Edit",
            items: vec![
                button_item("edit.undo", "Undo", &toolbar.undo),
                button_item("edit.redo", "Redo", &toolbar.redo),
                toggle_item("edit.find", "Find", &toolbar.find),
            ],
        },
        MenuDefinition {
            label: "View",
            items: vec![
                toggle_item("view.word-wrap", "Word wrap", &toolbar.word_wrap),
                button_item("view.zoom-in", "Zoom in", &toolbar.zoom_in),
                button_item("view.zoom-out", "Zoom out", &toolbar.zoom_out),
                button_item("view.zoom-reset", "Reset zoom", &toolbar.zoom_reset),
                button_item("view.view-only", "View only", &toolbar.view_only),
            ],
        },
        MenuDefinition {
            label: "Insert",
            items: vec![menu_item("insert.emoji", "Emoji", &toolbar.emoji)],
        },
        MenuDefinition {
            label: "Format",
            items: vec![
                toggle_item("format.bold", "Bold", &toolbar.bold),
                toggle_item("format.italic", "Italic", &toolbar.italic),
                toggle_item("format.underline", "Underline", &toolbar.quick_underline),
                toggle_item(
                    "format.strikethrough",
                    "Strikethrough",
                    &toolbar.quick_strikethrough,
                ),
                toggle_item("format.bullets", "Bullet list", &toolbar.bullets),
                toggle_item("format.numbered", "Numbered list", &toolbar.quick_numbered),
                menu_item("format.more", "More formatting…", &toolbar.format),
            ],
        },
        MenuDefinition {
            label: "Tools",
            items: vec![
                button_item("tools.go-to-line", "Go to line", &toolbar.go_to_line),
                menu_item("tools.more", "Editor mode and more…", &toolbar.more),
            ],
        },
    ]
}

fn button_item(key: &'static str, label: &'static str, source: &gtk::Button) -> MenuItemDefinition {
    MenuItemDefinition {
        key,
        label,
        source: MenuSource::Button(source.clone()),
    }
}

fn toggle_item(
    key: &'static str,
    label: &'static str,
    source: &gtk::ToggleButton,
) -> MenuItemDefinition {
    MenuItemDefinition {
        key,
        label,
        source: MenuSource::Toggle(source.clone()),
    }
}

fn menu_item(
    key: &'static str,
    label: &'static str,
    source: &gtk::MenuButton,
) -> MenuItemDefinition {
    MenuItemDefinition {
        key,
        label,
        source: MenuSource::Menu(source.clone()),
    }
}

fn build_menu(definition: MenuDefinition) -> BuiltMenu {
    let MenuDefinition {
        label,
        items: definitions,
    } = definition;
    let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
    content.add_css_class("nn-editor-menu-popover");
    let popover = gtk::Popover::builder().child(&content).build();
    popover.add_css_class("nn-menu-surface");
    let mut items = Vec::new();
    for definition in definitions {
        let item = build_item(definition, &popover);
        item.button.set_halign(gtk::Align::Fill);
        item.button.set_hexpand(true);
        item.button.add_css_class("flat");
        item.button.add_css_class("nn-menu-row");
        content.append(&item.button);
        items.push(item);
    }
    let menu = gtk::MenuButton::builder()
        .label(label)
        .popover(&popover)
        .tooltip_text(label)
        .build();
    menu.add_css_class("nn-editor-menu-button");
    menu.update_property(&[gtk::accessible::Property::Label(label)]);
    BuiltMenu {
        button: menu,
        popover,
        items,
    }
}

fn build_item(definition: MenuItemDefinition, parent: &gtk::Popover) -> BuiltItem {
    let key = definition.key;
    let label = definition.label;
    match definition.source {
        MenuSource::Button(source) => {
            let proxy = gtk::Button::with_label(label);
            let bindings = bind_availability(&source, &proxy);
            let parent = parent.clone();
            proxy.connect_clicked(move |_| {
                parent.popdown();
                source.emit_clicked();
            });
            BuiltItem {
                key,
                button: proxy,
                checked: None,
                bindings,
            }
        }
        MenuSource::Toggle(source) => {
            let proxy = gtk::ToggleButton::with_label(label);
            let mut bindings = bind_availability(&source, &proxy);
            bindings.push(
                source
                    .bind_property("active", &proxy, "active")
                    .bidirectional()
                    .sync_create()
                    .build(),
            );
            let parent = parent.clone();
            proxy.connect_clicked(move |_| parent.popdown());
            BuiltItem {
                key,
                button: proxy.clone().upcast(),
                checked: Some(proxy),
                bindings,
            }
        }
        MenuSource::Menu(source) => {
            let proxy = gtk::Button::with_label(label);
            let bindings = bind_availability(&source, &proxy);
            let parent = parent.clone();
            proxy.connect_clicked(move |_| {
                parent.popdown();
                source.popup();
            });
            BuiltItem {
                key,
                button: proxy,
                checked: None,
                bindings,
            }
        }
    }
}

fn bind_availability(
    source: &impl IsA<gtk::Widget>,
    proxy: &impl IsA<gtk::Widget>,
) -> Vec<glib::Binding> {
    let visible = source
        .bind_property("visible", proxy, "visible")
        .sync_create()
        .build();
    let sensitive = source
        .bind_property("sensitive", proxy, "sensitive")
        .sync_create()
        .build();
    vec![visible, sensitive]
}

fn close_other_popovers_when_opened(popovers: &[gtk::Popover]) {
    for (active_index, active) in popovers.iter().enumerate() {
        let others = popovers
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != active_index)
            .map(|(_, popover)| popover.clone())
            .collect::<Vec<_>>();
        active.connect_notify_local(Some("visible"), move |active, _| {
            if active.is_visible() {
                for other in &others {
                    other.popdown();
                }
            }
        });
    }
}
