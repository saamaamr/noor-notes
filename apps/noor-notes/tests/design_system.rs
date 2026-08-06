const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn replacement_design_system_defines_semantic_light_dark_and_accessible_states() {
    for token in [
        "@define-color nn_bg",
        "@define-color nn_surface",
        "@define-color nn_border",
        "@define-color nn_text",
        "@define-color nn_accent",
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
    ] {
        assert!(CSS.contains(token), "missing design token/state: {token}");
    }
    assert!(!CSS.contains("linear-gradient"));
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
