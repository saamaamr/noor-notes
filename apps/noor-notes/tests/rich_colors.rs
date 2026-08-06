use noor_notes::{
    appearance::EffectiveTheme,
    rich_color::{
        ColorRole, normalize_stored, presets, rendered_color, stored_value_from_tag, tag_name,
    },
};

#[test]
fn professional_palettes_are_complete_and_custom_rgb_is_canonical() {
    assert_eq!(presets(ColorRole::Foreground).len(), 7);
    assert_eq!(presets(ColorRole::Highlight).len(), 7);
    assert_eq!(
        normalize_stored(ColorRole::Foreground, "#1a2b3c").as_deref(),
        Some("#1A2B3C")
    );
    assert_eq!(normalize_stored(ColorRole::Highlight, "not-a-color"), None);
    assert_eq!(
        normalize_stored(ColorRole::Foreground, "charcoal").as_deref(),
        Some("slate")
    );
}

#[test]
fn preset_rendering_is_theme_adaptive_and_custom_rgb_is_exact() {
    assert_eq!(
        rendered_color(ColorRole::Foreground, "blue", EffectiveTheme::Light).as_deref(),
        Some("#1D4ED8")
    );
    assert_eq!(
        rendered_color(ColorRole::Foreground, "blue", EffectiveTheme::Oled).as_deref(),
        Some("#93C5FD")
    );
    assert_eq!(
        rendered_color(ColorRole::Highlight, "#ABCDEF", EffectiveTheme::Midnight).as_deref(),
        Some("#ABCDEF")
    );
}

#[test]
fn tag_encoding_round_trips_without_embedding_untrusted_input() {
    let name = tag_name(ColorRole::Foreground, "#1A2B3C").unwrap();
    assert_eq!(name, "noor-fg-hex-1A2B3C");
    assert_eq!(
        stored_value_from_tag(ColorRole::Foreground, &name).as_deref(),
        Some("#1A2B3C")
    );
    assert!(tag_name(ColorRole::Highlight, "invalid value").is_none());
}
