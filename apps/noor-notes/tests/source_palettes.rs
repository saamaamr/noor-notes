use noor_notes::appearance::EffectiveTheme;
use noor_notes::editor::source_palette;

#[test]
fn embedded_palettes_are_discoverable_complete_and_readable() {
    gtk::init().unwrap();
    assert!(source_palette::register());

    let expected = [
        (EffectiveTheme::Light, "noor-light"),
        (EffectiveTheme::Graphite, "noor-graphite"),
        (EffectiveTheme::Midnight, "noor-midnight"),
        (EffectiveTheme::Oled, "noor-oled"),
    ];
    let manager = sourceview5::StyleSchemeManager::default();

    for (theme, expected_id) in expected {
        assert_eq!(source_palette::scheme_id(theme), expected_id);
        let scheme = manager
            .scheme(expected_id)
            .unwrap_or_else(|| panic!("missing embedded scheme: {expected_id}"));
        for style_id in [
            "text",
            "cursor",
            "line-numbers",
            "current-line",
            "selection",
            "search-match",
        ] {
            assert!(
                scheme.style(style_id).is_some(),
                "{expected_id} is missing {style_id}"
            );
        }

        let text = scheme.style("text").unwrap();
        let foreground = text.foreground().expect("text foreground");
        let background = text.background().expect("text background");
        assert!(
            contrast_ratio(&foreground, &background) >= 4.5,
            "{expected_id} text contrast is below WCAG AA"
        );
    }
}

fn contrast_ratio(foreground: &str, background: &str) -> f64 {
    let luminance = |hex: &str| {
        let channel =
            |offset| u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap() as f64 / 255.0;
        let linear = |value: f64| {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(channel(1)) + 0.7152 * linear(channel(3)) + 0.0722 * linear(channel(5))
    };
    let (a, b) = (luminance(foreground), luminance(background));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
