use std::io::Cursor;

use docx_rs::{
    AbstractNumbering, AlignmentType, Docx, IndentLevel, Level, LevelJc, LevelText, NumberFormat,
    Numbering, NumberingId, Paragraph, Run, RunFonts, Shading, ShdType, SpecialIndentType, Start,
};
use noor_domain::{Alignment, ListKind, TextMarks};

use super::{ExportBlock, ExportBlockKind, ExportDocument, ExportError};
use crate::appearance::EffectiveTheme;
use crate::rich_color::{ColorRole, rendered_color};

const NUMBERED_LIST_ID: usize = 1;
const BULLET_LIST_ID: usize = 2;
const BODY_FONT: &str = "Noto Sans";
const CODE_FONT: &str = "Noto Sans Mono";

pub fn render(document: &ExportDocument) -> Result<Vec<u8>, ExportError> {
    let mut output = Docx::new()
        .add_abstract_numbering(bullet_numbering())
        .add_numbering(Numbering::new(BULLET_LIST_ID, BULLET_LIST_ID))
        .add_paragraph(title_paragraph(&document.title));

    for block in &document.blocks {
        output = output.add_paragraph(block_paragraph(block));
    }

    let mut bytes = Cursor::new(Vec::new());
    output
        .build()
        .pack(&mut bytes)
        .map_err(|error| ExportError::Render(error.to_string()))?;
    Ok(bytes.into_inner())
}

fn title_paragraph(title: &str) -> Paragraph {
    Paragraph::new().style("Title").add_run(
        Run::new()
            .fonts(unicode_fonts(BODY_FONT))
            .size(56)
            .bold()
            .add_text(title),
    )
}

fn block_paragraph(block: &ExportBlock) -> Paragraph {
    let monospace = matches!(
        block.kind,
        ExportBlockKind::CodeBlock | ExportBlockKind::MarkdownSource
    );
    let mut paragraph = Paragraph::new().align(alignment(block.alignment));
    paragraph = match block.kind {
        ExportBlockKind::ListItem {
            kind: ListKind::Bullet,
            ..
        } => paragraph.numbering(NumberingId::new(BULLET_LIST_ID), IndentLevel::new(0)),
        ExportBlockKind::ListItem {
            kind: ListKind::Numbered,
            ..
        } => paragraph.numbering(NumberingId::new(NUMBERED_LIST_ID), IndentLevel::new(0)),
        _ => paragraph,
    };

    for export_run in &block.runs {
        paragraph = paragraph.add_run(format_run(
            Run::new()
                .fonts(unicode_fonts(if monospace { CODE_FONT } else { BODY_FONT }))
                .add_text(&export_run.text),
            &export_run.marks,
        ));
    }
    paragraph
}

fn format_run(mut run: Run, marks: &TextMarks) -> Run {
    if marks.bold {
        run = run.bold();
    }
    if marks.italic {
        run = run.italic();
    }
    if marks.underline {
        run = run.underline("single");
    }
    if marks.strikethrough {
        run = run.strike();
    }
    if let Some(size) = marks.font_size {
        run = run.size(size.saturating_mul(2).min(400) as usize);
    }
    if let Some(color) = marks
        .foreground
        .as_deref()
        .and_then(|value| word_color(ColorRole::Foreground, value))
    {
        run = run.color(color);
    }
    if let Some(highlight) = marks
        .highlight
        .as_deref()
        .and_then(|value| word_color(ColorRole::Highlight, value))
    {
        run = run.shading(
            Shading::new()
                .shd_type(ShdType::Clear)
                .color("auto")
                .fill(highlight),
        );
    }
    run
}

fn word_color(role: ColorRole, value: &str) -> Option<String> {
    rendered_color(role, value, EffectiveTheme::Snow)
        .map(|color| color.trim_start_matches('#').to_string())
}

fn unicode_fonts(name: &str) -> RunFonts {
    RunFonts::new()
        .ascii(name)
        .hi_ansi(name)
        .east_asia(name)
        .cs(name)
}

const fn alignment(value: Alignment) -> AlignmentType {
    match value {
        Alignment::Start => AlignmentType::Left,
        Alignment::Center => AlignmentType::Center,
        Alignment::End => AlignmentType::Right,
        Alignment::Justify => AlignmentType::Both,
    }
}

fn bullet_numbering() -> AbstractNumbering {
    AbstractNumbering::new(BULLET_LIST_ID).add_level(
        Level::new(
            0,
            Start::new(1),
            NumberFormat::new("bullet"),
            LevelText::new("•"),
            LevelJc::new("left"),
        )
        .indent(Some(420), Some(SpecialIndentType::Hanging(420)), None, None),
    )
}
