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
