use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use super::AppearancePreferences;

#[derive(Clone, Debug)]
pub struct AppearanceStore {
    path: PathBuf,
}

impl AppearanceStore {
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn for_current_user() -> Self {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        Self::at(base.join("noor-notes/appearance.json"))
    }

    pub fn load(&self) -> AppearancePreferences {
        fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, preferences: &AppearancePreferences) -> io::Result<()> {
        let Some(parent) = self.path.parent() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "missing parent",
            ));
        };
        fs::create_dir_all(parent)?;
        let temporary = self.path.with_extension("json.tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&temporary)?;
        serde_json::to_writer(&mut file, preferences).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(temporary, &self.path)
    }
}
