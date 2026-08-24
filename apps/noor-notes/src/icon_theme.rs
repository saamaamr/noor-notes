pub const FALLBACK_ICON_THEME: &str = "Adwaita";

pub const REQUIRED_SYMBOLIC_ICONS: &[&str] = &[
    "accessories-text-editor-symbolic",
    "applications-graphics-symbolic",
    "dialog-password-symbolic",
    "dialog-warning-symbolic",
    "document-edit-symbolic",
    "document-open-recent-symbolic",
    "edit-clear-symbolic",
    "edit-copy-symbolic",
    "edit-delete-symbolic",
    "edit-find-symbolic",
    "edit-redo-symbolic",
    "edit-undo-symbolic",
    "emblem-system-symbolic",
    "face-smile-symbolic",
    "focus-windows-symbolic",
    "folder-symbolic",
    "format-justify-center-symbolic",
    "format-justify-fill-symbolic",
    "format-justify-left-symbolic",
    "format-justify-right-symbolic",
    "format-text-rich-symbolic",
    "go-down-symbolic",
    "go-jump-symbolic",
    "go-previous-symbolic",
    "go-up-symbolic",
    "list-add-symbolic",
    "network-offline-symbolic",
    "non-starred-symbolic",
    "object-select-symbolic",
    "open-menu-symbolic",
    "package-x-generic-symbolic",
    "sidebar-show-symbolic",
    "starred-symbolic",
    "system-search-symbolic",
    "user-bookmarks-symbolic",
    "user-trash-symbolic",
    "view-fullscreen-symbolic",
    "view-list-bullet-symbolic",
    "view-list-ordered-symbolic",
    "view-more-symbolic",
    "view-pin-symbolic",
    "view-refresh-symbolic",
    "view-sort-ascending-symbolic",
    "weather-clear-night-symbolic",
    "weather-clear-symbolic",
    "window-close-symbolic",
    "x-office-document-symbolic",
    "zoom-in-symbolic",
    "zoom-original-symbolic",
    "zoom-out-symbolic",
];

/// Keeps the desktop icon theme when it covers Noor Notes' UI. If a confined
/// runtime cannot access that theme, use GTK's bundled complete theme instead.
/// Returns `true` when the fallback was applied.
pub fn ensure_required_icons(display: &gtk::gdk::Display) -> bool {
    let theme = gtk::IconTheme::for_display(display);
    if REQUIRED_SYMBOLIC_ICONS
        .iter()
        .all(|name| theme.has_icon(name))
    {
        return false;
    }

    gtk::Settings::for_display(display).set_gtk_icon_theme_name(Some(FALLBACK_ICON_THEME));
    true
}
