use adw::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LibraryLayoutMode {
    Wide,
    Medium,
    Narrow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibraryPaneVisibility {
    pub sidebar: bool,
    pub collection: bool,
    pub content: bool,
    pub back: bool,
}

impl LibraryLayoutMode {
    pub const fn for_width(width: i32) -> Self {
        if width >= 1_080 {
            Self::Wide
        } else if width >= 760 {
            Self::Medium
        } else {
            Self::Narrow
        }
    }

    pub const fn for_window_width(allocated: i32, configured_default: i32) -> Self {
        let width = if allocated <= 1 {
            configured_default
        } else {
            allocated
        };
        Self::for_width(width)
    }

    pub const fn visibility(self, showing_content: bool) -> LibraryPaneVisibility {
        match self {
            Self::Wide => LibraryPaneVisibility {
                sidebar: true,
                collection: true,
                content: true,
                back: false,
            },
            Self::Medium => LibraryPaneVisibility {
                sidebar: false,
                collection: true,
                content: true,
                back: false,
            },
            Self::Narrow if showing_content => LibraryPaneVisibility {
                sidebar: false,
                collection: false,
                content: true,
                back: true,
            },
            Self::Narrow => LibraryPaneVisibility {
                sidebar: false,
                collection: true,
                content: false,
                back: false,
            },
        }
    }

    pub const fn pane_position(self, window_width: i32, showing_content: bool) -> i32 {
        match self {
            Self::Wide => 569,
            Self::Medium => 336,
            Self::Narrow if showing_content => 0,
            Self::Narrow => window_width,
        }
    }
}

pub fn apply_paned_layout(
    panes: &gtk::Paned,
    navigation: &impl IsA<gtk::Widget>,
    content: &impl IsA<gtk::Widget>,
    mode: LibraryLayoutMode,
    window_width: i32,
    showing_content: bool,
) {
    let visibility = mode.visibility(showing_content);
    navigation.set_visible(visibility.sidebar || visibility.collection);
    content.set_visible(visibility.content);
    panes.set_position(mode.pane_position(window_width, showing_content));
}
