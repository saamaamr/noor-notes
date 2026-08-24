use noor_notes::icon_theme::{FALLBACK_ICON_THEME, REQUIRED_SYMBOLIC_ICONS};

fn initialize_gtk() {
    gtk::init().expect("GTK test display must be available");
}

#[test]
fn missing_desktop_theme_falls_back_to_complete_adwaita_icons() {
    initialize_gtk();
    let display = gtk::gdk::Display::default().expect("GTK display");
    let theme = gtk::IconTheme::for_display(&display);
    let settings = gtk::Settings::for_display(&display);
    let original_theme = settings.gtk_icon_theme_name().map(|name| name.to_string());

    settings.set_gtk_icon_theme_name(Some("NoorMissingDesktopTheme"));
    assert!(
        REQUIRED_SYMBOLIC_ICONS
            .iter()
            .any(|name| !theme.has_icon(name)),
        "the simulated desktop theme must be incomplete"
    );

    assert!(noor_notes::icon_theme::ensure_required_icons(&display));
    assert_eq!(
        settings.gtk_icon_theme_name().as_deref(),
        Some(FALLBACK_ICON_THEME)
    );
    let unresolved: Vec<_> = REQUIRED_SYMBOLIC_ICONS
        .iter()
        .copied()
        .filter(|name| !theme.has_icon(name))
        .collect();
    assert!(
        unresolved.is_empty(),
        "fallback theme still misses required icons: {unresolved:?}"
    );

    settings.set_gtk_icon_theme_name(Some(FALLBACK_ICON_THEME));
    assert!(!noor_notes::icon_theme::ensure_required_icons(&display));
    assert_eq!(
        settings.gtk_icon_theme_name().as_deref(),
        Some(FALLBACK_ICON_THEME)
    );

    settings.set_gtk_icon_theme_name(original_theme.as_deref());
}
