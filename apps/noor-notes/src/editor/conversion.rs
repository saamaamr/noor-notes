use noor_domain::{EditorMode, Note, RichDocument, SourceLanguage};

use crate::export::export_markdown;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversionPreview {
    pub from: EditorMode,
    pub to: EditorMode,
    pub converted_content: String,
    pub warnings: Vec<String>,
}

pub fn preview_conversion(note: &Note, target: EditorMode) -> ConversionPreview {
    let mut warnings = Vec::new();
    let converted_content = match (&note.editor_mode, &target) {
        (EditorMode::Rich, EditorMode::Markdown) => {
            if rich_has_colour_or_size(note) {
                warnings.push(
                    "Text colour, highlight colour, and custom font sizes cannot be represented in Markdown."
                        .into(),
                );
            }
            export_markdown(note)
        }
        (EditorMode::Rich, EditorMode::PlainText | EditorMode::Code) => {
            if note.rich_content.is_some() {
                warnings
                    .push("Rich formatting will be removed; the text remains unchanged.".into());
            }
            note.content.clone()
        }
        (_, EditorMode::Rich) => note.content.clone(),
        _ => note.content.clone(),
    };
    ConversionPreview {
        from: note.editor_mode.clone(),
        to: target,
        converted_content,
        warnings,
    }
}

pub fn apply_conversion(note: &mut Note, preview: ConversionPreview) {
    debug_assert_eq!(note.editor_mode, preview.from);
    note.content = preview.converted_content;
    note.editor_mode = preview.to;
    match note.editor_mode {
        EditorMode::Rich => {
            note.rich_content = Some(RichDocument::from_plain_text(&note.content));
        }
        EditorMode::Markdown => {
            note.rich_content = None;
            note.source_language = SourceLanguage::Markdown;
        }
        EditorMode::PlainText | EditorMode::Code => {
            note.rich_content = None;
        }
    }
}

fn rich_has_colour_or_size(note: &Note) -> bool {
    note.rich_content.as_ref().is_some_and(|document| {
        document.blocks.iter().any(|block| {
            block.spans.iter().any(|span| {
                span.marks.foreground.is_some()
                    || span.marks.highlight.is_some()
                    || span.marks.font_size.is_some()
            })
        })
    })
}
