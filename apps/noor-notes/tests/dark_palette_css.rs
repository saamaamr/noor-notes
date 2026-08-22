use noor_notes::appearance::{EffectiveTheme, semantic_stylesheet};

const COMPONENT_CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn midnight_uses_one_display_wide_semantic_palette_for_every_major_surface() {
    let css = semantic_stylesheet(EffectiveTheme::Midnight);
    for declaration in [
        "@define-color nn_app_bg #0f1724;",
        "@define-color nn_sidebar_bg #111a2a;",
        "@define-color nn_note_list_bg #121c2d;",
        "@define-color nn_editor_bg #0f1724;",
        "@define-color nn_surface #172235;",
        "@define-color nn_popover_bg #172235;",
        "@define-color nn_text #f1f5f9;",
        "@define-color nn_text_secondary #cbd5e1;",
        "@define-color nn_border #26364d;",
        "@define-color nn_accent #6d8bff;",
    ] {
        assert!(css.contains(declaration), "missing Midnight role: {declaration}");
    }
    for component_role in [
        "window, .background { background: @nn_app_bg; color: @nn_text; }",
        ".nn-sidebar { background: @nn_sidebar_bg;",
        ".nn-note-list { background: @nn_note_list_bg;",
        ".nn-preview-surface, .nn-preview { background: @nn_editor_bg; }",
        ".nn-editor-toolbar {",
        ".nn-statusbar {",
        "popover > contents",
    ] {
        assert!(
            COMPONENT_CSS.contains(component_role),
            "component does not consume semantic role: {component_role}"
        );
    }
}

#[test]
fn theme_specific_selectors_are_limited_to_user_content_palettes() {
    assert!(!COMPONENT_CSS.contains("--nn-"));
    assert!(!COMPONENT_CSS.contains(".theme-dark"));
    let marker = "/* Theme-specific content colors: user data, not application chrome. */";
    let marker_index = COMPONENT_CSS.find(marker).expect("content palette marker");
    let chrome = &COMPONENT_CSS[..marker_index];
    assert!(!chrome.contains(".nn-theme-snow"));
    assert!(!chrome.contains(".nn-theme-midnight"));
    assert!(!chrome.contains("@nn_snow_"));
    assert!(!chrome.contains("@nn_midnight_"));
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
            COMPONENT_CSS.contains(&format!(".nn-theme-midnight .{paper}")),
            "missing Midnight paper mapping: {paper}"
        );
    }
}
