use cairo::{Context, PdfSurface, PdfVersion};
use gtk::pango::{self, FontDescription, Layout};
use noor_domain::{Alignment, ListKind, TextMarks};

use super::{ExportBlock, ExportBlockKind, ExportDocument, ExportError};
use crate::appearance::EffectiveTheme;
use crate::rich_color::{ColorRole, rendered_color};

const PAGE_WIDTH: f64 = 595.28;
const PAGE_HEIGHT: f64 = 841.89;
const PAGE_MARGIN: f64 = 54.0;
const CONTENT_WIDTH: f64 = PAGE_WIDTH - (PAGE_MARGIN * 2.0);
const CONTENT_BOTTOM: f64 = PAGE_HEIGHT - PAGE_MARGIN;
const BLOCK_GAP: f64 = 8.0;

pub fn render(document: &ExportDocument) -> Result<Vec<u8>, ExportError> {
    let surface =
        PdfSurface::for_stream(PAGE_WIDTH, PAGE_HEIGHT, Vec::<u8>::new()).map_err(render_error)?;
    surface.restrict(PdfVersion::_1_4).map_err(render_error)?;
    let context = Context::new(&surface).map_err(render_error)?;
    paint_page(&context)?;

    let mut cursor_y = PAGE_MARGIN;
    let title = layout(
        &context,
        &format!(
            "<span weight=\"bold\" size=\"{}\">{}</span>",
            26 * pango::SCALE,
            gtk::glib::markup_escape_text(&document.title)
        ),
        "Noto Sans 26",
        Alignment::Start,
        0,
    );
    draw_paginated(&context, &title, &mut cursor_y, 18.0)?;

    for block in &document.blocks {
        let source = matches!(
            block.kind,
            ExportBlockKind::CodeBlock | ExportBlockKind::MarkdownSource
        );
        let block_layout = layout(
            &context,
            &block_markup(block, source),
            if source {
                "Noto Sans Mono 11"
            } else {
                "Noto Sans 12"
            },
            block.alignment,
            3 * pango::SCALE,
        );
        draw_paginated(&context, &block_layout, &mut cursor_y, BLOCK_GAP)?;
    }

    context.status().map_err(render_error)?;
    drop(context);
    let stream = surface
        .finish_output_stream()
        .map_err(|error| ExportError::Render(error.to_string()))?;
    stream
        .downcast::<Vec<u8>>()
        .map(|bytes| *bytes)
        .map_err(|_| ExportError::Render("Cairo returned an unexpected PDF stream".into()))
}

fn layout(
    context: &Context,
    markup: &str,
    font: &str,
    alignment: Alignment,
    spacing: i32,
) -> Layout {
    let layout = pangocairo::functions::create_layout(context);
    pangocairo::functions::context_set_resolution(&layout.context(), 72.0);
    layout.set_width((CONTENT_WIDTH * f64::from(pango::SCALE)).round() as i32);
    layout.set_wrap(pango::WrapMode::WordChar);
    layout.set_font_description(Some(&FontDescription::from_string(font)));
    layout.set_spacing(spacing);
    layout.set_alignment(match alignment {
        Alignment::Start | Alignment::Justify => pango::Alignment::Left,
        Alignment::Center => pango::Alignment::Center,
        Alignment::End => pango::Alignment::Right,
    });
    layout.set_justify(alignment == Alignment::Justify);
    layout.set_markup(markup);
    layout
}

fn block_markup(block: &ExportBlock, source: bool) -> String {
    let mut output = String::new();
    match block.kind {
        ExportBlockKind::ListItem {
            kind: ListKind::Bullet,
            ..
        } => output.push_str("•  "),
        ExportBlockKind::ListItem {
            kind: ListKind::Numbered,
            ordinal,
        } => output.push_str(&format!("{ordinal}.  ")),
        _ => {}
    }

    for run in &block.runs {
        let text = gtk::glib::markup_escape_text(&run.text);
        let attributes = markup_attributes(&run.marks, source);
        if attributes.is_empty() {
            output.push_str(&text);
        } else {
            output.push_str("<span ");
            output.push_str(&attributes);
            output.push('>');
            output.push_str(&text);
            output.push_str("</span>");
        }
    }
    output
}

fn markup_attributes(marks: &TextMarks, source: bool) -> String {
    let mut attributes = Vec::new();
    if source {
        attributes.push("font_family=\"Noto Sans Mono\"".to_string());
    }
    if marks.bold {
        attributes.push("weight=\"bold\"".to_string());
    }
    if marks.italic {
        attributes.push("style=\"italic\"".to_string());
    }
    if marks.underline {
        attributes.push("underline=\"single\"".to_string());
    }
    if marks.strikethrough {
        attributes.push("strikethrough=\"true\"".to_string());
    }
    if let Some(size) = marks.font_size {
        attributes.push(format!(
            "size=\"{}\"",
            size.saturating_mul(pango::SCALE as u32)
        ));
    }
    if let Some(color) = marks
        .foreground
        .as_deref()
        .and_then(|value| rendered_color(ColorRole::Foreground, value, EffectiveTheme::Snow))
    {
        attributes.push(format!("foreground=\"{color}\""));
    }
    if let Some(color) = marks
        .highlight
        .as_deref()
        .and_then(|value| rendered_color(ColorRole::Highlight, value, EffectiveTheme::Snow))
    {
        attributes.push(format!("background=\"{color}\""));
    }
    attributes.join(" ")
}

fn draw_paginated(
    context: &Context,
    layout: &Layout,
    cursor_y: &mut f64,
    gap: f64,
) -> Result<(), ExportError> {
    let lines = line_ranges(layout);
    let mut first_line = 0;

    while first_line < lines.len() {
        let available = CONTENT_BOTTOM - *cursor_y;
        let segment_top = lines[first_line].0;
        let mut after_last = first_line;
        while after_last < lines.len() && lines[after_last].1 - segment_top <= available.max(0.0) {
            after_last += 1;
        }

        if after_last == first_line && *cursor_y > PAGE_MARGIN {
            new_page(context)?;
            *cursor_y = PAGE_MARGIN;
            continue;
        }
        if after_last == first_line {
            after_last += 1;
        }

        let segment_bottom = lines[after_last - 1].1;
        let segment_height = (segment_bottom - segment_top).max(1.0);
        context.save().map_err(render_error)?;
        context.rectangle(PAGE_MARGIN, *cursor_y, CONTENT_WIDTH, segment_height);
        context.clip();
        context.move_to(PAGE_MARGIN, *cursor_y - segment_top);
        context.set_source_rgb(0.122, 0.161, 0.216);
        pangocairo::functions::show_layout(context, layout);
        context.restore().map_err(render_error)?;

        *cursor_y += segment_height + gap;
        first_line = after_last;
        if first_line < lines.len() {
            new_page(context)?;
            *cursor_y = PAGE_MARGIN;
        }
    }
    Ok(())
}

fn line_ranges(layout: &Layout) -> Vec<(f64, f64)> {
    let mut ranges = Vec::new();
    let mut iter = layout.iter();
    loop {
        let (top, bottom) = iter.line_yrange();
        ranges.push((
            f64::from(top) / f64::from(pango::SCALE),
            f64::from(bottom) / f64::from(pango::SCALE),
        ));
        if !iter.next_line() {
            break;
        }
    }
    ranges
}

fn new_page(context: &Context) -> Result<(), ExportError> {
    context.show_page().map_err(render_error)?;
    paint_page(context)
}

fn paint_page(context: &Context) -> Result<(), ExportError> {
    context.set_source_rgb(1.0, 1.0, 1.0);
    context.paint().map_err(render_error)
}

fn render_error(error: cairo::Error) -> ExportError {
    ExportError::Render(error.to_string())
}
