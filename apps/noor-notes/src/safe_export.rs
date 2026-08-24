use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportExtension {
    Docx,
    Pdf,
    Html,
    PlainText,
    Markdown,
}

impl ExportExtension {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Pdf => "pdf",
            Self::Html => "html",
            Self::PlainText => "txt",
            Self::Markdown => "md",
        }
    }
}

pub fn sanitize_export_name(title: &str, extension: ExportExtension) -> String {
    let replaced: String = title
        .chars()
        .map(|character| {
            if character.is_control() || matches!(character, '/' | '\\') {
                ' '
            } else {
                character
            }
        })
        .collect();
    let collapsed = replaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let clean = collapsed.trim_matches(['.', ' ']);
    let stem: String = if clean.is_empty() {
        "Untitled".into()
    } else {
        clean.chars().take(120).collect()
    };
    format!("{stem}.{}", extension.as_str())
}

pub fn ensure_export_extension(path: &Path, extension: ExportExtension) -> PathBuf {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension.as_str()))
    {
        path.to_path_buf()
    } else {
        path.with_extension(extension.as_str())
    }
}

#[cfg(unix)]
pub fn set_owner_only(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
pub fn set_owner_only(_path: &Path) -> std::io::Result<()> {
    Ok(())
}
