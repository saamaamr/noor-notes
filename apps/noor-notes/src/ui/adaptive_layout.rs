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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LibraryPaneAllocation {
    pub sidebar: i32,
    pub collection: i32,
    pub navigation: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorLayoutDensity {
    Spacious,
    Compact,
    Narrow,
}

/// Keeps the document proportional to the editor pane instead of pinning it to
/// one desktop-sized pixel width. Narrow panes use all available space, medium
/// panes retain a little breathing room, and wide panes cap the document at a
/// readable line length.
pub fn editor_content_width(available_width: i32) -> i32 {
    let available_width = available_width.max(0);
    let proportional_width = if available_width < 640 {
        available_width
    } else if available_width < 1_000 {
        (available_width * 92 + 50) / 100
    } else {
        (available_width * 78 + 50) / 100
    };
    proportional_width.min(860)
}

pub const fn editor_layout_density(available_width: i32) -> EditorLayoutDensity {
    if available_width >= 700 {
        EditorLayoutDensity::Spacious
    } else if available_width >= 500 {
        EditorLayoutDensity::Compact
    } else {
        EditorLayoutDensity::Narrow
    }
}

pub fn allocation_for_width(
    mode: LibraryLayoutMode,
    width: i32,
    showing_content: bool,
) -> LibraryPaneAllocation {
    let width = width.max(0);
    match mode {
        LibraryLayoutMode::Wide => {
            let sidebar = ((width * 10 + 50) / 100).clamp(160, 220);
            let collection = ((width * 18 + 50) / 100).clamp(280, 360);
            LibraryPaneAllocation {
                sidebar,
                collection,
                navigation: sidebar + collection,
            }
        }
        LibraryLayoutMode::Medium => {
            let collection = ((width * 34 + 50) / 100).clamp(280, 360);
            LibraryPaneAllocation {
                sidebar: 0,
                collection,
                navigation: collection,
            }
        }
        LibraryLayoutMode::Narrow if showing_content => LibraryPaneAllocation {
            sidebar: 0,
            collection: 0,
            navigation: 0,
        },
        LibraryLayoutMode::Narrow => LibraryPaneAllocation {
            sidebar: 0,
            collection: width,
            navigation: width,
        },
    }
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

    pub fn pane_position(self, window_width: i32, showing_content: bool) -> i32 {
        allocation_for_width(self, window_width, showing_content).navigation
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

#[allow(
    clippy::too_many_arguments,
    reason = "this boundary applies one allocation atomically to the five concrete pane widgets"
)]
pub fn apply_library_layout(
    panes: &gtk::Paned,
    navigation: &impl IsA<gtk::Widget>,
    sidebar: &impl IsA<gtk::Widget>,
    collection: &impl IsA<gtk::Widget>,
    content: &impl IsA<gtk::Widget>,
    mode: LibraryLayoutMode,
    window_width: i32,
    showing_content: bool,
) {
    let visibility = mode.visibility(showing_content);
    let allocation = allocation_for_width(mode, window_width, showing_content);

    sidebar.set_width_request(allocation.sidebar);
    sidebar.set_visible(visibility.sidebar);
    collection.set_width_request(allocation.collection);
    collection.set_visible(visibility.collection);
    apply_paned_layout(
        panes,
        navigation,
        content,
        mode,
        window_width,
        showing_content,
    );
}
