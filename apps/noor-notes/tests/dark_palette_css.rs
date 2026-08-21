const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn midnight_styles_every_major_surface_and_semantic_icon_state() {
    let root = ".nn-theme-midnight";
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
fn gtk_palette_switching_uses_only_namespaced_snow_and_midnight_layers() {
    assert!(!CSS.contains("--nn-"));
    assert!(!CSS.contains(".theme-dark"));
    assert!(CSS.contains(".nn-theme-snow"));
    assert!(CSS.contains(".nn-theme-midnight"));
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
            CSS.contains(&format!(".nn-theme-midnight .{paper}")),
            "missing Midnight paper mapping: {paper}"
        );
    }
}
