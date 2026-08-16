use noor_domain::EditorMode;
use sourceview5::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckRegion {
    pub start: usize,
    pub end: usize,
}

pub fn plain_text_regions(text: &str) -> Vec<CheckRegion> {
    let end = text.chars().count();
    (end > 0)
        .then_some(CheckRegion { start: 0, end })
        .into_iter()
        .collect()
}

pub fn checkable_regions(buffer: &sourceview5::Buffer, mode: EditorMode) -> Vec<CheckRegion> {
    let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
    if matches!(mode, EditorMode::Rich | EditorMode::PlainText) {
        return plain_text_regions(&text);
    }

    buffer.ensure_highlight(&buffer.start_iter(), &buffer.end_iter());
    let character_count = text.chars().count();
    let mut regions = Vec::new();
    let mut region_start = None;

    for offset in 0..character_count {
        let iter = buffer.iter_at_offset(offset as i32);
        let no_spell = buffer.iter_has_context_class(&iter, "no-spell-check");
        let path = buffer.iter_has_context_class(&iter, "path");
        let included = match mode {
            EditorMode::Markdown => !no_spell && !path,
            EditorMode::Code => {
                !path
                    && (buffer.iter_has_context_class(&iter, "comment")
                        || buffer.iter_has_context_class(&iter, "string"))
            }
            EditorMode::Rich | EditorMode::PlainText => true,
        };

        match (region_start, included) {
            (None, true) => region_start = Some(offset),
            (Some(start), false) => {
                regions.push(CheckRegion { start, end: offset });
                region_start = None;
            }
            _ => {}
        }
    }

    if let Some(start) = region_start {
        regions.push(CheckRegion {
            start,
            end: character_count,
        });
    }
    regions
}
