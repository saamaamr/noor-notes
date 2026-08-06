const CSS: &str = include_str!("../resources/design-system.css");

#[test]
fn replacement_design_system_defines_semantic_light_dark_and_accessible_states() {
    for token in [
        "--nn-bg",
        "--nn-surface",
        "--nn-surface-raised",
        "--nn-border",
        "--nn-text",
        "--nn-text-secondary",
        "--nn-accent",
        "--nn-success",
        "--nn-warning",
        "--nn-error",
        ".theme-dark",
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
