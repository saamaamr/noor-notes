const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn all_dark_palettes_style_every_major_surface_and_semantic_icon_state() {
    for palette in ["graphite", "midnight", "oled"] {
        let root = format!(".nn-theme-{palette}");
        assert!(CSS.contains(&root), "missing palette root: {root}");
        for component in [
            "headerbar",
            ".nn-sidebar",
            ".nn-note-list",
            ".nn-note-card",
            ".nn-preview",
            ".nn-rich-writing-canvas",
            ".nn-find-panel",
            ".nn-statusbar",
            "popover",
        ] {
            let selector = format!("{root} {component}");
            assert!(CSS.contains(&selector), "missing selector: {selector}");
        }
    }

    for class in [
        ".nn-icon-neutral",
        ".nn-icon-secondary",
        ".nn-icon-active",
        ".nn-icon-success",
        ".nn-icon-warning",
        ".nn-icon-destructive",
        ".nn-icon-destructive:hover",
    ] {
        assert!(CSS.contains(class), "missing adaptive icon state: {class}");
    }
}

#[test]
fn browser_custom_properties_are_not_used_for_gtk_palette_switching() {
    assert!(!CSS.contains("--nn-"));
    assert!(!CSS.contains(".theme-dark"));
    assert!(CSS.contains(".nn-theme-light"));
    for paper in [
        "paper-warm-white",
        "paper-cream",
        "paper-light-yellow",
        "paper-light-blue",
        "paper-light-green",
        "paper-light-pink",
        "paper-light-purple",
        "paper-dark-slate",
    ] {
        assert!(
            CSS.contains(&format!(".nn-theme-oled .{paper}")),
            "missing OLED paper mapping: {paper}"
        );
    }
}
