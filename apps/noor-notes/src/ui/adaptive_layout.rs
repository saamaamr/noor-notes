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
        if width >= 1_200 {
            Self::Wide
        } else if width >= 760 {
            Self::Medium
        } else {
            Self::Narrow
        }
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
}
