use noor_notes::appearance::{EffectiveTheme, ThemeStyleState, semantic_stylesheet};

#[test]
fn generated_stylesheet_uses_the_active_palette_before_shared_components() {
    let snow = semantic_stylesheet(EffectiveTheme::Snow);
    let midnight = semantic_stylesheet(EffectiveTheme::Midnight);

    assert!(snow.starts_with("@define-color nn_bg #f6f7f9;"));
    assert!(midnight.starts_with("@define-color nn_bg #0f1724;"));
    for css in [&snow, &midnight] {
        let palette_end = css.find("/* Noor Notes GTK design system").unwrap();
        let semantic_definition = css.find("@define-color nn_text ").unwrap();
        assert!(semantic_definition < palette_end);
        assert_eq!(css.matches("/* Noor Notes GTK design system").count(), 1);
        for token in [
            "nn_bg",
            "nn_surface",
            "nn_popover_bg",
            "nn_text",
            "nn_text_secondary",
            "nn_border",
            "nn_accent",
            "nn_selection_bg",
        ] {
            assert_eq!(
                css.matches(&format!("@define-color {token} ")).count(),
                1,
                "{token} must have exactly one active definition"
            );
        }
    }
}

#[test]
fn style_state_reloads_only_when_effective_theme_changes() {
    let mut state = ThemeStyleState::default();
    assert!(state.activate(EffectiveTheme::Snow));
    assert_eq!(state.active(), Some(EffectiveTheme::Snow));
    assert!(!state.activate(EffectiveTheme::Snow));
    assert!(state.activate(EffectiveTheme::Midnight));
    assert_eq!(state.active(), Some(EffectiveTheme::Midnight));
    assert!(!state.activate(EffectiveTheme::Midnight));
}
