const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn every_effective_theme_defines_the_complete_semantic_palette() {
    for theme in noor_notes::appearance::EffectiveTheme::ALL {
        let theme = theme.palette_prefix();
        for role in [
            "app_bg",
            "sidebar_bg",
            "list_bg",
            "editor_bg",
            "surface",
            "surface_raised",
            "popover_bg",
            "input_bg",
            "text_primary",
            "text_secondary",
            "text_muted",
            "border",
            "border_strong",
            "accent",
            "accent_soft",
            "focus",
            "danger",
            "danger_soft",
        ] {
            let token = format!("@define-color nn_{theme}_{role}");
            assert!(CSS.contains(&token), "{theme} is missing {role}");
        }
    }
}

#[test]
fn snow_defaults_match_the_approved_semantic_palette() {
    for declaration in [
        "@define-color nn_app_bg #f6f7f9;",
        "@define-color nn_sidebar_bg #f4f6f8;",
        "@define-color nn_note_list_bg #f8f9fb;",
        "@define-color nn_editor_bg #ffffff;",
        "@define-color nn_surface #ffffff;",
        "@define-color nn_hover #f1f3f5;",
        "@define-color nn_text #1f2937;",
        "@define-color nn_text_secondary #475467;",
        "@define-color nn_text_muted #667085;",
        "@define-color nn_border #e4e7ec;",
        "@define-color nn_border_subtle #eef0f2;",
        "@define-color nn_accent #4f6fe8;",
        "@define-color nn_accent_hover #425fcc;",
        "@define-color nn_accent_soft #eef2ff;",
        "@define-color nn_danger #dc2626;",
        "@define-color nn_success #16a34a;",
    ] {
        assert!(
            CSS.contains(declaration),
            "missing Snow token: {declaration}"
        );
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
    for required in [".nn-theme-snow", ".nn-theme-midnight"] {
        assert!(CSS.contains(required));
    }
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
        "@define-color nn_popover_bg",
        "@define-color nn_modal_bg",
        "@define-color nn_input_bg",
        "@define-color nn_text_disabled",
        "@define-color nn_text_inverse",
        "@define-color nn_border_strong",
        "@define-color nn_accent_strong",
        "@define-color nn_danger_soft",
        "@define-color nn_info",
        ".nn-control-compact",
        ".nn-control-primary",
        ".nn-surface-elevated",
        ".nn-menu-surface",
        ".nn-document-title",
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
