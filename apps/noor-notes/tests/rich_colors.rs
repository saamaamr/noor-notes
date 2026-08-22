use gtk::prelude::*;
use noor_notes::{
    appearance::{EffectiveTheme, ThemePalette},
    rich_buffer::RichBuffer,
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
        rendered_color(ColorRole::Foreground, "blue", EffectiveTheme::Snow).as_deref(),
        Some("#1D4ED8")
    );
    assert_eq!(
        rendered_color(ColorRole::Foreground, "blue", EffectiveTheme::Midnight).as_deref(),
        Some("#93C5FD")
    );
    assert_eq!(
        rendered_color(ColorRole::Highlight, "#ABCDEF", EffectiveTheme::Midnight).as_deref(),
        Some("#ABCDEF")
    );
    assert_eq!(
        rendered_color(ColorRole::Foreground, "blue", EffectiveTheme::Snow).as_deref(),
        Some("#1D4ED8")
    );
    assert_eq!(
        rendered_color(ColorRole::Highlight, "yellow", EffectiveTheme::Snow).as_deref(),
        Some("#FEF3C7")
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

#[test]
fn rich_buffer_preset_tags_follow_the_active_theme() {
    gtk::init().unwrap();
    let buffer = gtk::TextBuffer::new(None);
    RichBuffer::prepare(&buffer);
    let tag = buffer.tag_table().lookup("noor-fg-blue").unwrap();

    RichBuffer::apply_color_theme(&buffer, EffectiveTheme::Snow);
    let light = tag.foreground_rgba().unwrap();
    assert!((light.red() - 0x1D as f32 / 255.0).abs() < 0.001);
    assert!((light.green() - 0x4E as f32 / 255.0).abs() < 0.001);
    assert!((light.blue() - 0xD8 as f32 / 255.0).abs() < 0.001);

    RichBuffer::apply_color_theme(&buffer, EffectiveTheme::Midnight);
    let dark = tag.foreground_rgba().unwrap();
    assert!((dark.red() - 0x93 as f32 / 255.0).abs() < 0.001);
    assert!((dark.green() - 0xC5 as f32 / 255.0).abs() < 0.001);
    assert!((dark.blue() - 0xFD as f32 / 255.0).abs() < 0.001);
}

#[test]
fn rich_link_foreground_follows_the_active_theme() {
    gtk::init().unwrap();
    let buffer = gtk::TextBuffer::new(None);
    RichBuffer::prepare(&buffer);
    let tag = buffer.tag_table().lookup("noor-link").unwrap();

    for theme in EffectiveTheme::ALL {
        RichBuffer::apply_color_theme(&buffer, theme);
        let actual = tag.foreground_rgba().unwrap();
        assert_rgb(&actual, ThemePalette::for_theme(theme).info);
    }
}

fn assert_rgb(actual: &gtk::gdk::RGBA, expected: &str) {
    let channel = |offset| u8::from_str_radix(&expected[offset..offset + 2], 16).unwrap() as f32
        / 255.0;
    assert!((actual.red() - channel(1)).abs() < 0.001);
    assert!((actual.green() - channel(3)).abs() < 0.001);
    assert!((actual.blue() - channel(5)).abs() < 0.001);
}
