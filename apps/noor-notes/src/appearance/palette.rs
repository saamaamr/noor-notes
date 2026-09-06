use super::EffectiveTheme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemePalette {
    pub app_bg: &'static str,
    pub sidebar_bg: &'static str,
    pub note_list_bg: &'static str,
    pub editor_bg: &'static str,
    pub surface: &'static str,
    pub surface_raised: &'static str,
    pub popover_bg: &'static str,
    pub modal_bg: &'static str,
    pub input_bg: &'static str,
    pub hover: &'static str,
    pub active: &'static str,
    pub selected: &'static str,
    pub text_primary: &'static str,
    pub text_secondary: &'static str,
    pub text_muted: &'static str,
    pub text_disabled: &'static str,
    pub text_inverse: &'static str,
    pub border: &'static str,
    pub border_subtle: &'static str,
    pub border_strong: &'static str,
    pub accent: &'static str,
    pub accent_hover: &'static str,
    pub accent_soft: &'static str,
    pub accent_strong: &'static str,
    pub focus: &'static str,
    pub success: &'static str,
    pub warning: &'static str,
    pub danger: &'static str,
    pub danger_soft: &'static str,
    pub error: &'static str,
    pub info: &'static str,
    pub scrollbar: &'static str,
    pub scrollbar_hover: &'static str,
    pub selection_bg: &'static str,
    pub selection_fg: &'static str,
    pub rich_foreground: [(&'static str, &'static str); 7],
    pub rich_highlight: [(&'static str, &'static str); 7],
}

impl ThemePalette {
    pub const fn for_theme(theme: EffectiveTheme) -> Self {
        match theme {
            EffectiveTheme::Snow => Self {
                app_bg: "#f6f7f9",
                sidebar_bg: "#f4f6f8",
                note_list_bg: "#f8f9fb",
                editor_bg: "#ffffff",
                surface: "#ffffff",
                surface_raised: "#ffffff",
                popover_bg: "#ffffff",
                modal_bg: "#ffffff",
                input_bg: "#ffffff",
                hover: "#f1f3f5",
                active: "#e9edf3",
                selected: "#eef2ff",
                text_primary: "#1f2937",
                text_secondary: "#475467",
                text_muted: "#667085",
                text_disabled: "#98a2b3",
                text_inverse: "#ffffff",
                border: "#e4e7ec",
                border_subtle: "#eef0f2",
                border_strong: "#d0d5dd",
                accent: "#4b69dc",
                accent_hover: "#425fcc",
                accent_soft: "#eef2ff",
                accent_strong: "#344fc4",
                focus: "#4b69dc",
                success: "#15803d",
                warning: "#b45309",
                danger: "#dc2626",
                danger_soft: "#fef2f2",
                error: "#dc2626",
                info: "#2563eb",
                scrollbar: "#c7cdd6",
                scrollbar_hover: "#aeb7c4",
                selection_bg: "#c7d2fe",
                selection_fg: "#1f2937",
                rich_foreground: [
                    ("slate", "#334155"),
                    ("blue", "#1d4ed8"),
                    ("teal", "#0f766e"),
                    ("green", "#15803d"),
                    ("amber", "#a16207"),
                    ("red", "#b91c1c"),
                    ("purple", "#7e22ce"),
                ],
                rich_highlight: [
                    ("yellow", "#fef3c7"),
                    ("blue", "#dbeafe"),
                    ("mint", "#ccfbf1"),
                    ("green", "#dcfce7"),
                    ("peach", "#ffedd5"),
                    ("pink", "#fce7f3"),
                    ("lavender", "#ede9fe"),
                ],
            },
            EffectiveTheme::Midnight => Self {
                app_bg: "#0f1724",
                sidebar_bg: "#111a2a",
                note_list_bg: "#121c2d",
                editor_bg: "#0f1724",
                surface: "#172235",
                surface_raised: "#1d2a40",
                popover_bg: "#172235",
                modal_bg: "#172235",
                input_bg: "#172235",
                hover: "#1d2a40",
                active: "#223250",
                selected: "#1d2a4a",
                text_primary: "#f1f5f9",
                text_secondary: "#cbd5e1",
                text_muted: "#94a3b8",
                text_disabled: "#64748b",
                text_inverse: "#0f1724",
                border: "#26364d",
                border_subtle: "#1d2a3d",
                border_strong: "#33465f",
                accent: "#6d8bff",
                accent_hover: "#819aff",
                accent_soft: "#1d2a4a",
                accent_strong: "#9aabff",
                focus: "#6d8bff",
                success: "#4ade80",
                warning: "#fbbf24",
                danger: "#f87171",
                danger_soft: "#37242d",
                error: "#f87171",
                info: "#60a5fa",
                scrollbar: "#33465f",
                scrollbar_hover: "#465c77",
                selection_bg: "#334a7a",
                selection_fg: "#f1f5f9",
                rich_foreground: [
                    ("slate", "#e2e8f0"),
                    ("blue", "#93c5fd"),
                    ("teal", "#5eead4"),
                    ("green", "#86efac"),
                    ("amber", "#fcd34d"),
                    ("red", "#fca5a5"),
                    ("purple", "#d8b4fe"),
                ],
                rich_highlight: [
                    ("yellow", "#5f4b16"),
                    ("blue", "#1e3a5f"),
                    ("mint", "#134e4a"),
                    ("green", "#14532d"),
                    ("peach", "#7c2d12"),
                    ("pink", "#6b214b"),
                    ("lavender", "#4c3575"),
                ],
            },
        }
    }

    pub fn gtk_css(self) -> String {
        let declarations = [
            ("nn_bg", self.app_bg),
            ("nn_app_bg", self.app_bg),
            ("nn_sidebar_bg", self.sidebar_bg),
            ("nn_note_list_bg", self.note_list_bg),
            ("nn_editor_bg", self.editor_bg),
            ("nn_surface", self.surface),
            ("nn_surface_raised", self.surface_raised),
            ("nn_popover_bg", self.popover_bg),
            ("nn_modal_bg", self.modal_bg),
            ("nn_input_bg", self.input_bg),
            ("nn_hover", self.hover),
            ("nn_active", self.active),
            ("nn_selected", self.selected),
            ("nn_text", self.text_primary),
            ("nn_text_secondary", self.text_secondary),
            ("nn_text_muted", self.text_muted),
            ("nn_text_disabled", self.text_disabled),
            ("nn_text_inverse", self.text_inverse),
            ("nn_border", self.border),
            ("nn_border_subtle", self.border_subtle),
            ("nn_border_strong", self.border_strong),
            ("nn_accent", self.accent),
            ("nn_accent_hover", self.accent_hover),
            ("nn_accent_soft", self.accent_soft),
            ("nn_accent_strong", self.accent_strong),
            ("nn_focus", self.focus),
            ("nn_success", self.success),
            ("nn_warning", self.warning),
            ("nn_danger", self.danger),
            ("nn_danger_soft", self.danger_soft),
            ("nn_error", self.error),
            ("nn_info", self.info),
            ("nn_scrollbar", self.scrollbar),
            ("nn_scrollbar_hover", self.scrollbar_hover),
            ("nn_selection_bg", self.selection_bg),
            ("nn_selection_fg", self.selection_fg),
        ];
        let mut css = String::with_capacity(declarations.len() * 42);
        for (name, value) in declarations {
            css.push_str("@define-color ");
            css.push_str(name);
            css.push(' ');
            css.push_str(value);
            css.push_str(";\n");
        }
        for (id, value) in self.rich_foreground {
            css.push_str("@define-color nn_rich_fg_");
            css.push_str(id);
            css.push(' ');
            css.push_str(value);
            css.push_str(";\n");
        }
        for (id, value) in self.rich_highlight {
            css.push_str("@define-color nn_rich_highlight_");
            css.push_str(id);
            css.push(' ');
            css.push_str(value);
            css.push_str(";\n");
        }
        css.push_str("@define-color nn_focus_ring alpha(@nn_focus,.24);\n");
        // Native libadwaita controls must use the same semantic palette as our
        // custom widgets, including their checked, focus and dialog states.
        for (native, semantic) in [
            ("accent_bg_color", "nn_accent"),
            ("accent_fg_color", "nn_text_inverse"),
            ("accent_color", "nn_accent"),
            ("window_bg_color", "nn_app_bg"),
            ("window_fg_color", "nn_text"),
            ("view_bg_color", "nn_editor_bg"),
            ("view_fg_color", "nn_text"),
            ("headerbar_bg_color", "nn_surface"),
            ("headerbar_fg_color", "nn_text"),
            ("card_bg_color", "nn_surface"),
            ("card_fg_color", "nn_text"),
            ("popover_bg_color", "nn_popover_bg"),
            ("popover_fg_color", "nn_text"),
            ("dialog_bg_color", "nn_modal_bg"),
            ("dialog_fg_color", "nn_text"),
            ("destructive_color", "nn_danger"),
            ("success_color", "nn_success"),
            ("warning_color", "nn_warning"),
            ("error_color", "nn_error"),
        ] {
            css.push_str(&format!("@define-color {native} @{semantic};\n"));
        }
        css
    }
}
