#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LibrarySection {
    #[default]
    AllNotes,
    Pinned,
    Favorites,
    Tags,
    Archived,
    Trash,
    Recent,
}

impl LibrarySection {
    pub const NAVIGATION: [Self; 7] = [
        Self::AllNotes,
        Self::Pinned,
        Self::Favorites,
        Self::Tags,
        Self::Archived,
        Self::Trash,
        Self::Recent,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::AllNotes => "All Notes",
            Self::Pinned => "Pinned",
            Self::Favorites => "Favorites",
            Self::Tags => "Tags",
            Self::Archived => "Archived",
            Self::Trash => "Trash",
            Self::Recent => "Recent",
        }
    }

    pub const fn icon_name(self) -> &'static str {
        match self {
            Self::AllNotes => "document-open-recent-symbolic",
            Self::Pinned => "view-pin-symbolic",
            Self::Favorites => "starred-symbolic",
            Self::Tags => "user-bookmarks-symbolic",
            Self::Archived => "package-x-generic-symbolic",
            Self::Trash => "user-trash-symbolic",
            Self::Recent => "document-open-recent-symbolic",
        }
    }
}
