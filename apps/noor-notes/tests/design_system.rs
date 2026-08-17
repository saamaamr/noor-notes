const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn replacement_design_system_defines_semantic_light_dark_and_accessible_states() {
    for token in [
        "@define-color nn_bg",
        "@define-color nn_app_bg",
        "@define-color nn_surface",
        "@define-color nn_editor_bg",
        "@define-color nn_sidebar_bg",
        "@define-color nn_note_list_bg",
        "@define-color nn_border",
        "@define-color nn_border_subtle",
        "@define-color nn_text",
        "@define-color nn_text_muted",
        "@define-color nn_accent",
        "@define-color nn_accent_hover",
        "@define-color nn_accent_soft",
        "@define-color nn_danger",
        "@define-color nn_focus",
        "@define-color nn_focus_ring",
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
fn light_mode_uses_professional_semantic_tokens_and_neutral_interactions() {
    for declaration in [
        "@define-color nn_app_bg #f7f8fa;",
        "@define-color nn_sidebar_bg #f6f7f9;",
        "@define-color nn_note_list_bg #fafafb;",
        "@define-color nn_surface #ffffff;",
        "@define-color nn_hover #f1f3f6;",
        "@define-color nn_text #1f2937;",
        "@define-color nn_text_secondary #667085;",
        "@define-color nn_text_muted #6b7280;",
        "@define-color nn_border #e5e7eb;",
        "@define-color nn_border_subtle #eef0f2;",
        "@define-color nn_accent #4f6fe8;",
        "@define-color nn_accent_hover #425fcc;",
        "@define-color nn_accent_soft #eef2ff;",
        "@define-color nn_danger #dc2626;",
        "@define-color nn_success #16a34a;",
        "@define-color nn_scrollbar #c7cdd6;",
        "@define-color nn_scrollbar_hover #aeb7c4;",
    ] {
        assert!(
            CSS.contains(declaration),
            "missing Light token: {declaration}"
        );
    }

    let button_hover = CSS
        .split("button:hover")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("button hover rules");
    assert!(button_hover.contains("@nn_hover"));
    assert!(!button_hover.contains("@nn_accent"));

    for theme in ["graphite", "midnight", "oled"] {
        assert!(
            CSS.contains(&format!(".nn-theme-{theme} .nn-sidebar-row:selected")),
            "dark sidebar selection must be explicit for {theme}"
        );
    }
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
fn light_library_layers_sidebar_and_note_list_without_heavy_borders() {
    assert!(CSS.contains(".nn-sidebar { background: @nn_sidebar_bg;"));
    assert!(CSS.contains(".nn-note-list { background: @nn_note_list_bg;"));
    assert!(CSS.contains(".nn-sidebar-row { min-height: 42px;"));
    assert!(CSS.contains(".nn-pane-separator { min-width: 1px; background: @nn_border;"));
}

#[test]
fn note_colors_are_identity_rails_and_selection_remains_calm() {
    for color in ["yellow", "cream", "blue", "green", "rose", "lavender"] {
        assert!(CSS.contains(&format!(".note-{color} .nn-color-strip")));
    }
    assert!(CSS.contains(".nn-card-action { min-width: 32px; min-height: 32px;"));
    let selected = CSS
        .split(".nn-note-list row:selected .nn-note-card")
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .expect("selected card rules");
    assert!(selected.contains("@nn_selected"));
    assert!(!selected.contains("color: white"));
}

#[test]
fn light_header_search_sort_and_status_share_compact_chrome() {
    for rule in [
        ".nn-app-header { min-height: 44px;",
        ".nn-header-control { min-width: 36px; min-height: 36px;",
        ".nn-new-note { min-height: 36px;",
        ".nn-sort-control { min-height: 36px;",
        ".nn-search-entry { min-height: 36px;",
        ".nn-statusbar { min-height: 30px;",
        ".nn-theme-light scrollbar slider {",
    ] {
        assert!(CSS.contains(rule), "missing compact chrome rule: {rule}");
    }
    assert!(CSS.contains(".nn-theme-light { background: @nn_app_bg; color: @nn_text; }"));
    assert!(CSS.contains(".nn-theme-light windowcontrols button"));
}

#[test]
fn dark_palettes_override_light_specific_library_colors() {
    for theme in ["graphite", "midnight", "oled"] {
        for selector in [
            "nn-preview-title",
            "nn-preview-body",
            "nn-sidebar-row:hover",
            "nn-sort-control",
            "nn-pane-separator",
            "nn-note-card-tags",
        ] {
            assert!(
                CSS.contains(&format!(".nn-theme-{theme} .{selector}")),
                "{theme} must override shared Light styling for {selector}"
            );
        }
    }

    for theme in ["graphite", "midnight", "oled"] {
        let hover_marker = format!(".nn-theme-{theme} .nn-note-card:hover");
        let hover = CSS
            .split(&hover_marker)
            .nth(1)
            .and_then(|rules| rules.split('}').next())
            .expect("dark card hover rules");
        assert!(hover.contains("border-color:"));

        let selected_marker = format!(".nn-theme-{theme} .nn-note-list row:selected .nn-note-card");
        let selected = CSS
            .split(&selected_marker)
            .nth(1)
            .and_then(|rules| rules.split('}').next())
            .expect("dark selected card rules");
        assert!(selected.contains("box-shadow:"));
    }
}

#[test]
fn note_card_title_keeps_the_compact_typography_contract() {
    assert_eq!(
        CSS.matches(".nn-note-title {").count(),
        1,
        "a later duplicate selector can silently override compact card typography"
    );
    assert!(CSS.contains(".nn-note-title { font-size: 16px; font-weight: 600; }"));
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
