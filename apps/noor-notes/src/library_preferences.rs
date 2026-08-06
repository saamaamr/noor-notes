use noor_storage::NoteSort;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct LibraryPreferences {
    path: PathBuf,
}

impl LibraryPreferences {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }
    pub fn for_current_user() -> Self {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        Self::at(base.join("noor-notes/preferences"))
    }
    pub fn load_sort(&self) -> NoteSort {
        match std::fs::read_to_string(&self.path)
            .ok()
            .as_deref()
            .map(str::trim)
        {
            Some("title") => NoteSort::TitleAsc,
            Some("title-desc") => NoteSort::TitleDesc,
            Some("created") => NoteSort::CreatedDesc,
            _ => NoteSort::UpdatedDesc,
        }
    }
    pub fn save_sort(&self, sort: NoteSort) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &self.path,
            match sort {
                NoteSort::UpdatedDesc => "updated",
                NoteSort::TitleAsc => "title",
                NoteSort::TitleDesc => "title-desc",
                NoteSort::CreatedDesc => "created",
            },
        )
    }
}
