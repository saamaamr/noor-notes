const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn replacement_design_system_defines_semantic_light_dark_and_accessible_states() {
    for token in [
        "@define-color nn_bg",
        "@define-color nn_app_bg",
        "@define-color nn_surface",
        "@define-color nn_editor_bg",
        "@define-color nn_sidebar_bg",
        "@define-color nn_border",
        "@define-color nn_border_subtle",
        "@define-color nn_text",
        "@define-color nn_text_muted",
        "@define-color nn_accent",
        "@define-color nn_accent_hover",
        "@define-color nn_accent_soft",
        "@define-color nn_danger",
        "@define-color nn_focus",
        "@define-color nn_hover",
        "@define-color nn_selected",
        ".nn-theme-light",
        ".nn-theme-graphite",
        ".nn-theme-midnight",
        ".nn-theme-oled",
        ".nn-icon-active",
        ":focus-visible",
        ":disabled",
        "prefers-reduced-motion",
        ".paper-warm-white",
        ".paper-dark-slate",
        ".nn-source-canvas",
        ".nn-focus-ring",
    ] {
        assert!(CSS.contains(token), "missing design token/state: {token}");
    }
    assert!(!CSS.contains("linear-gradient"));
}

#[test]
fn selected_navigation_and_note_cards_use_calm_semantic_surfaces() {
    let sidebar_selection = CSS
        .split(".nn-sidebar-row:selected")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("sidebar selected rules");
    assert!(sidebar_selection.contains("@nn_selected"));
    assert!(!sidebar_selection.contains("color: white"));

    let card_selection = CSS
        .split(".nn-note-list row:selected .nn-note-card")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("note card selected rules");
    assert!(card_selection.contains("@nn_selected"));
    assert!(card_selection.contains("@nn_accent"));
}

#[test]
fn replacement_design_system_is_valid_gtk_css() {
    use std::cell::RefCell;
    use std::rc::Rc;

    gtk::init().unwrap();
    let errors = Rc::new(RefCell::new(Vec::new()));
    let captured = errors.clone();
    let provider = gtk::CssProvider::new();
    provider.connect_parsing_error(move |_, section, error| {
        captured.borrow_mut().push(format!("{section:?}: {error}"));
    });
    provider.load_from_string(CSS);
    assert!(errors.borrow().is_empty(), "{}", errors.borrow().join("\n"));
}

#[test]
fn source_canvas_layout_does_not_override_sourceview_palette_colors() {
    let source_canvas = CSS
        .split(".nn-writing-canvas {")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("source canvas rules");
    assert!(!source_canvas.contains("background:"));
    assert!(!source_canvas.contains("color:"));
    assert!(!source_canvas.contains("caret-color:"));

    let source_mode = CSS
        .split(".nn-source-canvas {")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("source mode canvas rules");
    assert!(!source_mode.contains("background:"));
    assert!(!source_mode.contains("color:"));
    assert!(!source_mode.contains("caret-color:"));

    let rich_canvas = CSS
        .split(".nn-rich-writing-canvas {")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("rich canvas rules");
    assert!(rich_canvas.contains("background:"));
    assert!(rich_canvas.contains("color:"));
    assert!(rich_canvas.contains("caret-color:"));
}

#[test]
fn rich_color_swatches_define_every_professional_light_and_dark_preset() {
    for (role, colors) in [
        (
            "text",
            &["slate", "blue", "teal", "green", "amber", "red", "purple"][..],
        ),
        (
            "highlight",
            &[
                "yellow", "blue", "mint", "green", "peach", "pink", "lavender",
            ][..],
        ),
    ] {
        for color in colors {
            let selector = format!(".nn-{role}-swatch.nn-color-{color}");
            assert!(
                CSS.contains(&selector),
                "missing swatch selector: {selector}"
            );
            for theme in ["graphite", "midnight", "oled"] {
                let dark = format!(".nn-theme-{theme} {selector}");
                assert!(CSS.contains(&dark), "missing dark swatch selector: {dark}");
            }
        }
    }
}
