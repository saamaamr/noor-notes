const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn component_stylesheet_consumes_but_never_defines_the_active_palette() {
    assert!(!CSS.contains("@define-color"));
    for role in [
        "@nn_app_bg",
        "@nn_sidebar_bg",
        "@nn_note_list_bg",
        "@nn_editor_bg",
        "@nn_surface",
        "@nn_popover_bg",
        "@nn_text",
        "@nn_text_secondary",
        "@nn_text_muted",
        "@nn_border",
        "@nn_accent",
        "@nn_selection_bg",
    ] {
        assert!(CSS.contains(role), "component CSS does not consume {role}");
    }
}

#[test]
fn curated_atomic_utilities_cover_the_approved_scale() {
    for utility in [
        ".nn-p-8",
        ".nn-p-12",
        ".nn-m-4",
        ".nn-h-32",
        ".nn-h-36",
        ".nn-radius-6",
        ".nn-radius-8",
        ".nn-text-body",
        ".nn-text-meta",
        ".nn-text-muted",
        ".nn-surface",
        ".nn-icon-button",
        ".nn-focus-ring",
    ] {
        assert!(CSS.contains(utility), "missing atomic utility: {utility}");
    }
}

#[test]
fn only_snow_and_midnight_theme_layers_remain() {
    for obsolete in [
        ".nn-theme-light",
        ".nn-theme-warm-paper",
        ".nn-theme-cool-mist",
        ".nn-theme-graphite",
        ".nn-theme-oled",
    ] {
        assert!(!CSS.contains(obsolete), "obsolete theme layer: {obsolete}");
    }
    assert!(CSS.lines().count() <= 500);
}

#[test]
fn semantic_components_preserve_calm_readable_states() {
    for token in [
        ".nn-control-compact",
        ".nn-control-primary",
        ".nn-surface-elevated",
        ".nn-menu-surface",
        ".nn-document-title",
        "popover > contents",
        "popover modelbutton",
        "dropdown > button",
        "text selection",
    ] {
        assert!(CSS.contains(token), "missing semantic role: {token}");
    }
    let selected = rule_after(".nn-note-list row:selected .nn-note-card");
    assert!(selected.contains("background: @nn_selected"));
    assert!(selected.contains("border-color: alpha(@nn_accent"));
    assert!(!selected.contains("color: white"));
    let navigation = rule_after(".nn-sidebar-row:selected");
    assert!(navigation.contains("@nn_selected"));
    assert!(navigation.contains("@nn_accent"));
}

#[test]
fn document_and_compact_chrome_use_one_typography_scale() {
    for rule in [
        ".nn-app-header { min-height: 44px;",
        ".nn-sidebar-row { min-height: 40px;",
        ".nn-card-action { min-width: 32px; min-height: 32px;",
        ".nn-note-title { font-size: 16px; font-weight: 600; }",
        ".nn-preview-title { font-size: 20px; font-weight: 700;",
        ".nn-preview-editor { min-height: 280px; font-size: 16px;",
        ".nn-preview-metadata { font-size: 13px;",
        ".nn-statusbar { min-height: 30px;",
    ] {
        assert!(CSS.contains(rule), "missing hierarchy rule: {rule}");
    }
}

#[test]
fn note_colors_remain_identity_rails_and_rich_swatches_are_complete() {
    for color in ["yellow", "cream", "blue", "green", "rose", "lavender"] {
        assert!(CSS.contains(&format!(".note-{color} .nn-color-strip")));
    }
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
            assert!(CSS.contains(&format!(".nn-{role}-swatch.nn-color-{color}")));
        }
    }
}

#[test]
fn source_canvas_layout_does_not_override_sourceview_palette_colors() {
    for selector in [".nn-writing-canvas {", ".nn-source-canvas {"] {
        let rules = rule_after(selector);
        assert!(!rules.contains("background:"));
        assert!(!rules.contains("color:"));
        assert!(!rules.contains("caret-color:"));
    }
    let rich = rule_after(".nn-rich-writing-canvas {");
    assert!(rich.contains("background:"));
    assert!(rich.contains("color:"));
    assert!(rich.contains("caret-color:"));
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

fn rule_after(marker: &str) -> &str {
    CSS.split(marker)
        .nth(1)
        .and_then(|rules| rules.split('}').next())
        .unwrap_or_else(|| panic!("missing CSS rule: {marker}"))
}
