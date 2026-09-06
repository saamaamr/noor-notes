use noor_notes::appearance::{EffectiveTheme, ThemePalette};

#[test]
fn snow_and_midnight_keep_text_readable_on_every_primary_surface() {
    for theme in EffectiveTheme::ALL {
        let palette = ThemePalette::for_theme(theme);
        for (label, foreground, background, minimum) in [
            ("primary/app", palette.text_primary, palette.app_bg, 4.5),
            (
                "primary/surface",
                palette.text_primary,
                palette.surface,
                4.5,
            ),
            (
                "primary/popover",
                palette.text_primary,
                palette.popover_bg,
                4.5,
            ),
            (
                "secondary/popover",
                palette.text_secondary,
                palette.popover_bg,
                4.5,
            ),
            ("muted/surface", palette.text_muted, palette.surface, 4.5),
            ("primary action", palette.text_inverse, palette.accent, 4.5),
            ("success/surface", palette.success, palette.surface, 4.5),
            ("warning/surface", palette.warning, palette.surface, 4.5),
            (
                "disabled/surface",
                palette.text_disabled,
                palette.surface,
                2.0,
            ),
            ("accent/soft", palette.accent, palette.accent_soft, 3.0),
            ("selection", palette.selection_fg, palette.selection_bg, 4.5),
            ("danger/surface", palette.danger, palette.surface, 3.0),
        ] {
            let ratio = contrast_ratio(foreground, background);
            assert!(
                ratio >= minimum,
                "{theme:?} {label} contrast {ratio:.2}:1 is below {minimum:.1}:1"
            );
        }
    }
}

#[test]
fn each_palette_renders_the_same_complete_semantic_gtk_contract() {
    for theme in EffectiveTheme::ALL {
        let css = ThemePalette::for_theme(theme).gtk_css();
        for token in [
            "nn_bg",
            "accent_bg_color",
            "accent_fg_color",
            "window_bg_color",
            "window_fg_color",
            "view_bg_color",
            "view_fg_color",
            "popover_bg_color",
            "popover_fg_color",
            "dialog_bg_color",
            "dialog_fg_color",
            "nn_app_bg",
            "nn_sidebar_bg",
            "nn_note_list_bg",
            "nn_editor_bg",
            "nn_surface",
            "nn_surface_raised",
            "nn_popover_bg",
            "nn_modal_bg",
            "nn_input_bg",
            "nn_hover",
            "nn_active",
            "nn_selected",
            "nn_text",
            "nn_text_secondary",
            "nn_text_muted",
            "nn_text_disabled",
            "nn_text_inverse",
            "nn_border",
            "nn_border_subtle",
            "nn_border_strong",
            "nn_accent",
            "nn_accent_hover",
            "nn_accent_soft",
            "nn_accent_strong",
            "nn_focus",
            "nn_focus_ring",
            "nn_success",
            "nn_warning",
            "nn_danger",
            "nn_danger_soft",
            "nn_error",
            "nn_info",
            "nn_scrollbar",
            "nn_scrollbar_hover",
            "nn_selection_bg",
            "nn_selection_fg",
            "nn_rich_fg_slate",
            "nn_rich_fg_blue",
            "nn_rich_fg_teal",
            "nn_rich_fg_green",
            "nn_rich_fg_amber",
            "nn_rich_fg_red",
            "nn_rich_fg_purple",
            "nn_rich_highlight_yellow",
            "nn_rich_highlight_blue",
            "nn_rich_highlight_mint",
            "nn_rich_highlight_green",
            "nn_rich_highlight_peach",
            "nn_rich_highlight_pink",
            "nn_rich_highlight_lavender",
        ] {
            assert!(
                css.contains(&format!("@define-color {token} ")),
                "{theme:?} did not render {token}"
            );
        }
    }
}

#[test]
fn palettes_use_the_approved_day_and_night_surface_families() {
    let snow = ThemePalette::for_theme(EffectiveTheme::Snow);
    assert_eq!(snow.app_bg, "#f6f7f9");
    assert_eq!(snow.popover_bg, "#ffffff");
    assert_eq!(snow.text_primary, "#1f2937");

    let midnight = ThemePalette::for_theme(EffectiveTheme::Midnight);
    assert_eq!(midnight.app_bg, "#0f1724");
    assert_eq!(midnight.popover_bg, "#172235");
    assert_eq!(midnight.text_primary, "#f1f5f9");
}

fn contrast_ratio(foreground: &str, background: &str) -> f64 {
    let foreground = luminance(foreground);
    let background = luminance(background);
    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
}

fn luminance(hex: &str) -> f64 {
    assert_eq!(hex.len(), 7, "expected #RRGGBB, got {hex}");
    let channel = |offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap() as f64 / 255.0;
    let linear = |value: f64| {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(channel(1)) + 0.7152 * linear(channel(3)) + 0.0722 * linear(channel(5))
}
