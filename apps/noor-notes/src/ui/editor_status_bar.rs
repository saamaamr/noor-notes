use adw::prelude::*;

use crate::editor_status::EditorStatistics;

#[derive(Clone)]
pub struct EditorStatusBar {
    pub widget: gtk::Box,
    pub statistics: gtk::Label,
    pub mode: gtk::Label,
}

impl EditorStatusBar {
    pub fn new(mode_name: &str) -> Self {
        let statistics = gtk::Label::new(Some(
            "Ln 1, Col 1  ·  1 line  ·  0 words  ·  0 characters  ·  100%",
        ));
        statistics.set_halign(gtk::Align::Start);
        statistics.update_property(&[gtk::accessible::Property::Label("Editor statistics")]);

        let mode = gtk::Label::new(Some(&format!("{mode_name}  ·  UTF-8")));
        mode.set_halign(gtk::Align::End);
        mode.set_hexpand(true);
        mode.update_property(&[gtk::accessible::Property::Label("Editor mode and encoding")]);

        let widget = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        widget.add_css_class("nn-statusbar");
        widget.append(&statistics);
        widget.append(&mode);
        Self {
            widget,
            statistics,
            mode,
        }
    }

    pub fn update_statistics(&self, stats: EditorStatistics) {
        let selected = if stats.selection > 0 {
            format!("  ·  {} selected", stats.selection)
        } else {
            String::new()
        };
        self.statistics.set_text(&format!(
            "Ln {}, Col {}  ·  {} {}  ·  {} words  ·  {} characters{}  ·  {}%",
            stats.line,
            stats.column,
            stats.lines,
            if stats.lines == 1 { "line" } else { "lines" },
            stats.words,
            stats.characters,
            selected,
            stats.zoom
        ));
    }
}
