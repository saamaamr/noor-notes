mod document;
mod docx;
mod html;
mod markdown;
mod pdf;
mod text;

use noor_domain::{ListKind, Note, TextMarks};
use thiserror::Error;

use crate::safe_export::ExportExtension;

pub use document::{ExportBlock, ExportBlockKind, ExportDocument, ExportRun};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportFormat {
    Docx,
    Pdf,
    Html,
    PlainText,
    Markdown,
}

impl ExportFormat {
    pub const fn extension(self) -> ExportExtension {
        match self {
            Self::Docx => ExportExtension::Docx,
            Self::Pdf => ExportExtension::Pdf,
            Self::Html => ExportExtension::Html,
            Self::PlainText => ExportExtension::PlainText,
            Self::Markdown => ExportExtension::Markdown,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Docx => "Word Document",
            Self::Pdf => "PDF Document",
            Self::Html => "HTML Document",
            Self::PlainText => "Plain Text",
            Self::Markdown => "Markdown",
        }
    }

    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Pdf => "application/pdf",
            Self::Html => "text/html",
            Self::PlainText => "text/plain",
            Self::Markdown => "text/markdown",
        }
    }
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("{0} export is not available")]
    UnsupportedFormat(&'static str),
    #[error("could not render the exported document: {0}")]
    Render(String),
}

pub fn render_export(
    document: &ExportDocument,
    format: ExportFormat,
) -> Result<Vec<u8>, ExportError> {
    match format {
        ExportFormat::Docx => docx::render(document),
        ExportFormat::Html => Ok(html::render(document).into_bytes()),
        ExportFormat::PlainText => Ok(text::render(document).into_bytes()),
        ExportFormat::Markdown => Ok(markdown::render(document).into_bytes()),
        ExportFormat::Pdf => pdf::render(document),
    }
}

pub fn export_plain(note: &Note) -> String {
    note.content.clone()
}

pub fn export_markdown(note: &Note) -> String {
    let Some(document) = note
        .rich_content
        .as_ref()
        .filter(|document| document.is_supported())
    else {
        return format!("{}\n", note.content);
    };
    let mut output = String::new();
    for (index, block) in document.blocks.iter().enumerate() {
        match block.list {
            Some(ListKind::Bullet) => output.push_str("- "),
            Some(ListKind::Numbered) => output.push_str(&format!("{}. ", index + 1)),
            None => {}
        }
        for span in &block.spans {
            output.push_str(&markdown_span(&span.text, &span.marks));
        }
        output.push('\n');
    }
    output
}

fn markdown_span(text: &str, marks: &TextMarks) -> String {
    let mut value = text.to_string();
    if marks.strikethrough {
        value = format!("~~{value}~~");
    }
    if marks.underline {
        value = format!("<u>{value}</u>");
    }
    if marks.italic {
        value = format!("*{value}*");
    }
    if marks.bold {
        value = format!("**{value}**");
    }
    value
}
