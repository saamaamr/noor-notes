use adw::prelude::*;
use noor_domain::{Alignment, ListKind, RichBlock, RichDocument, RichSpan, TextMarks};

const BOLD: &str = "noor-bold";
const ITALIC: &str = "noor-italic";
const UNDERLINE: &str = "noor-underline";
const STRIKE: &str = "noor-strike";
const LINK: &str = "noor-link";
const SIZE_TAGS: [(&str, u16); 5] = [
    ("noor-size-12", 12),
    ("noor-size-14", 14),
    ("noor-size-16", 16),
    ("noor-size-18", 18),
    ("noor-size-24", 24),
];
const FG_TAGS: [(&str, &str); 4] = [
    ("noor-fg-charcoal", "#29251f"),
    ("noor-fg-blue", "#174a7e"),
    ("noor-fg-green", "#276749"),
    ("noor-fg-red", "#a12c2c"),
];
const BG_TAGS: [(&str, &str); 4] = [
    ("noor-bg-charcoal", "#d8c99b"),
    ("noor-bg-blue", "#b9d9ee"),
    ("noor-bg-green", "#bfe3c0"),
    ("noor-bg-red", "#efb9bd"),
];

pub struct RichBuffer;

impl RichBuffer {
    pub fn prepare(buffer: &gtk::TextBuffer) {
        buffer.set_enable_undo(true);
        let table = buffer.tag_table();
        add_tag(
            &table,
            gtk::TextTag::builder().name(BOLD).weight(700).build(),
        );
        add_tag(
            &table,
            gtk::TextTag::builder()
                .name(ITALIC)
                .style(gtk::pango::Style::Italic)
                .build(),
        );
        add_tag(
            &table,
            gtk::TextTag::builder()
                .name(UNDERLINE)
                .underline(gtk::pango::Underline::Single)
                .build(),
        );
        add_tag(
            &table,
            gtk::TextTag::builder()
                .name(STRIKE)
                .strikethrough(true)
                .build(),
        );
        add_tag(
            &table,
            gtk::TextTag::builder()
                .name(LINK)
                .foreground("#174a7e")
                .underline(gtk::pango::Underline::Single)
                .build(),
        );
        for (name, size) in SIZE_TAGS {
            add_tag(
                &table,
                gtk::TextTag::builder()
                    .name(name)
                    .size_points(size as f64)
                    .build(),
            );
        }
        for (name, color) in FG_TAGS {
            add_tag(
                &table,
                gtk::TextTag::builder().name(name).foreground(color).build(),
            );
        }
        for (name, color) in BG_TAGS {
            add_tag(
                &table,
                gtk::TextTag::builder().name(name).background(color).build(),
            );
        }
        for (name, justification) in [
            ("noor-align-start", gtk::Justification::Left),
            ("noor-align-center", gtk::Justification::Center),
            ("noor-align-end", gtk::Justification::Right),
            ("noor-align-justify", gtk::Justification::Fill),
        ] {
            add_tag(
                &table,
                gtk::TextTag::builder()
                    .name(name)
                    .justification(justification)
                    .build(),
            );
        }
    }

    pub fn load(buffer: &gtk::TextBuffer, content: &str, document: Option<&RichDocument>) {
        Self::prepare(buffer);
        let Some(document) = document.filter(|document| document.is_supported()) else {
            buffer.set_text(content);
            Self::tag_urls(buffer);
            return;
        };
        buffer.set_text("");
        let mut cursor = buffer.end_iter();
        for (block_index, block) in document.blocks.iter().enumerate() {
            if block_index > 0 {
                buffer.insert(&mut cursor, "\n");
            }
            let block_start = cursor.offset();
            for span in &block.spans {
                let start_offset = cursor.offset();
                buffer.insert(&mut cursor, &span.text);
                let start = buffer.iter_at_offset(start_offset);
                let end = buffer.iter_at_offset(cursor.offset());
                apply_marks(buffer, &start, &end, &span.marks);
            }
            let start = buffer.iter_at_offset(block_start);
            let end = buffer.iter_at_offset(cursor.offset());
            buffer.apply_tag_by_name(alignment_tag(block.alignment), &start, &end);
        }
        Self::tag_urls(buffer);
    }

    pub fn snapshot(buffer: &gtk::TextBuffer) -> (String, RichDocument) {
        let content = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
        let mut blocks = vec![RichBlock::default()];
        let mut iter = buffer.start_iter();
        while !iter.is_end() {
            let character = iter.char();
            if character == '\n' {
                blocks.push(RichBlock::default());
                iter.forward_char();
                continue;
            }
            if iter.starts_line() {
                blocks
                    .last_mut()
                    .expect("document always has a block")
                    .alignment = alignment_at(&iter);
            }
            let marks = marks_at(&iter);
            let block = blocks.last_mut().expect("document always has a block");
            if let Some(last) = block.spans.last_mut().filter(|span| span.marks == marks) {
                last.text.push(character);
            } else {
                block.spans.push(RichSpan {
                    text: character.to_string(),
                    marks,
                });
            }
            iter.forward_char();
        }
        (content, RichDocument { version: 1, blocks })
    }

    pub fn toggle_selection(buffer: &gtk::TextBuffer, tag_name: &str) {
        let Some((start, end)) = buffer.selection_bounds() else {
            return;
        };
        if start.has_tag(
            &buffer
                .tag_table()
                .lookup(tag_name)
                .expect("known formatting tag"),
        ) {
            buffer.remove_tag_by_name(tag_name, &start, &end);
        } else {
            buffer.apply_tag_by_name(tag_name, &start, &end);
        }
    }

    pub fn can_undo(buffer: &gtk::TextBuffer) -> bool {
        buffer.can_undo()
    }
    pub fn can_redo(buffer: &gtk::TextBuffer) -> bool {
        buffer.can_redo()
    }
    pub fn undo(buffer: &gtk::TextBuffer) {
        if buffer.can_undo() {
            buffer.undo();
        }
    }
    pub fn redo(buffer: &gtk::TextBuffer) {
        if buffer.can_redo() {
            buffer.redo();
        }
    }

    pub fn bold(buffer: &gtk::TextBuffer) {
        Self::toggle_selection(buffer, BOLD);
    }

    pub fn italic(buffer: &gtk::TextBuffer) {
        Self::toggle_selection(buffer, ITALIC);
    }

    pub fn underline(buffer: &gtk::TextBuffer) {
        Self::toggle_selection(buffer, UNDERLINE);
    }

    pub fn strikethrough(buffer: &gtk::TextBuffer) {
        Self::toggle_selection(buffer, STRIKE);
    }

    pub fn font_size(buffer: &gtk::TextBuffer, size: u32) {
        let name = format!("noor-size-{size}");
        if buffer.tag_table().lookup(&name).is_none() {
            buffer.tag_table().add(
                &gtk::TextTag::builder()
                    .name(&name)
                    .size_points(size as f64)
                    .build(),
            );
        }
        replace_selection_tag(buffer, "noor-size-", &name);
    }

    pub fn parse_font_size(value: &str) -> Option<u32> {
        value.trim().parse::<u32>().ok().filter(|size| *size > 0)
    }

    pub fn foreground(buffer: &gtk::TextBuffer, color: &str) {
        replace_selection_tag(buffer, "noor-fg-", &format!("noor-fg-{color}"));
    }

    pub fn align(buffer: &gtk::TextBuffer, alignment: Alignment) {
        let mut start = buffer.iter_at_mark(&buffer.get_insert());
        start.set_line_offset(0);
        let mut end = start;
        end.forward_to_line_end();
        for tag in [
            "noor-align-start",
            "noor-align-center",
            "noor-align-end",
            "noor-align-justify",
        ] {
            buffer.remove_tag_by_name(tag, &start, &end);
        }
        buffer.apply_tag_by_name(alignment_tag(alignment), &start, &end);
    }

    pub fn highlight(buffer: &gtk::TextBuffer, color: &str) {
        replace_selection_tag(buffer, "noor-bg-", &format!("noor-bg-{color}"));
    }

    pub fn toggle_list(buffer: &gtk::TextBuffer, kind: ListKind) {
        let (first, last) = selected_lines(buffer);
        let all_match = (first..=last).all(|line| {
            line_text(buffer, line)
                .as_deref()
                .and_then(list_marker)
                .is_some_and(|(current, _, _)| current == kind)
        });
        for line in (first..=last).rev() {
            replace_line_marker(
                buffer,
                line,
                if all_match { None } else { Some(kind) },
                line - first + 1,
            );
        }
    }

    pub fn list_kind_at_cursor(buffer: &gtk::TextBuffer) -> Option<ListKind> {
        let line = buffer.iter_at_mark(&buffer.get_insert()).line();
        line_text(buffer, line)
            .as_deref()
            .and_then(list_marker)
            .map(|value| value.0)
    }

    pub fn continue_list(buffer: &gtk::TextBuffer) -> bool {
        let cursor = buffer.iter_at_mark(&buffer.get_insert());
        let line = cursor.line();
        let Some(text) = line_text(buffer, line) else {
            return false;
        };
        let Some((kind, marker_len, number)) = list_marker(&text) else {
            return false;
        };
        if text[marker_len..].trim().is_empty() {
            replace_line_marker(buffer, line, None, 1);
            return true;
        }
        let prefix = match kind {
            ListKind::Bullet => "\n• ".to_string(),
            ListKind::Numbered => format!("\n{}. ", number.unwrap_or(1) + 1),
        };
        buffer.insert_at_cursor(&prefix);
        true
    }

    pub fn insert_emoji(buffer: &gtk::TextBuffer, emoji: &str) {
        buffer.insert_at_cursor(emoji);
    }

    fn tag_urls(buffer: &gtk::TextBuffer) {
        let text = buffer
            .text(&buffer.start_iter(), &buffer.end_iter(), true)
            .to_string();
        for token in text.split_whitespace() {
            if !(token.starts_with("https://") || token.starts_with("http://")) {
                continue;
            }
            if let Some(byte_start) = text.find(token) {
                let char_start = text[..byte_start].chars().count() as i32;
                let char_end = char_start + token.chars().count() as i32;
                buffer.apply_tag_by_name(
                    LINK,
                    &buffer.iter_at_offset(char_start),
                    &buffer.iter_at_offset(char_end),
                );
            }
        }
    }
}

fn selected_lines(buffer: &gtk::TextBuffer) -> (i32, i32) {
    let (start, mut end) = buffer.selection_bounds().unwrap_or_else(|| {
        let cursor = buffer.iter_at_mark(&buffer.get_insert());
        (cursor, cursor)
    });
    if end.line_offset() == 0 && end.offset() > start.offset() {
        end.backward_char();
    }
    (start.line(), end.line())
}

fn line_text(buffer: &gtk::TextBuffer, line: i32) -> Option<String> {
    let start = buffer.iter_at_line(line)?;
    let mut end = start;
    end.forward_to_line_end();
    Some(buffer.text(&start, &end, true).to_string())
}

fn list_marker(text: &str) -> Option<(ListKind, usize, Option<u32>)> {
    if text.starts_with("• ") {
        return Some((ListKind::Bullet, "• ".len(), None));
    }
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 && text.get(digits..digits + 2) == Some(". ") {
        return text[..digits]
            .parse()
            .ok()
            .map(|number| (ListKind::Numbered, digits + 2, Some(number)));
    }
    None
}

fn replace_line_marker(buffer: &gtk::TextBuffer, line: i32, kind: Option<ListKind>, ordinal: i32) {
    let Some(mut start) = buffer.iter_at_line(line) else {
        return;
    };
    if let Some(text) = line_text(buffer, line) {
        if let Some((_, marker_len, _)) = list_marker(&text) {
            let mut marker_end = start;
            marker_end.forward_chars(text[..marker_len].chars().count() as i32);
            buffer.delete(&mut start, &mut marker_end);
        }
    }
    if let Some(kind) = kind {
        let prefix = match kind {
            ListKind::Bullet => "• ".to_string(),
            ListKind::Numbered => format!("{ordinal}. "),
        };
        buffer.insert(&mut start, &prefix);
    }
}

fn add_tag(table: &gtk::TextTagTable, tag: gtk::TextTag) {
    if tag.name().is_none_or(|name| table.lookup(&name).is_none()) {
        table.add(&tag);
    }
}

fn apply_marks(
    buffer: &gtk::TextBuffer,
    start: &gtk::TextIter,
    end: &gtk::TextIter,
    marks: &TextMarks,
) {
    for (enabled, tag) in [
        (marks.bold, BOLD),
        (marks.italic, ITALIC),
        (marks.underline, UNDERLINE),
        (marks.strikethrough, STRIKE),
    ] {
        if enabled {
            buffer.apply_tag_by_name(tag, start, end);
        }
    }
    if let Some(size) = marks.font_size {
        let name = format!("noor-size-{size}");
        if buffer.tag_table().lookup(&name).is_none() {
            buffer.tag_table().add(
                &gtk::TextTag::builder()
                    .name(&name)
                    .size_points(size as f64)
                    .build(),
            );
        }
        buffer.apply_tag_by_name(&name, start, end);
    }
    if let Some(color) = &marks.foreground {
        buffer.apply_tag_by_name(&format!("noor-fg-{color}"), start, end);
    }
    if let Some(color) = &marks.highlight {
        buffer.apply_tag_by_name(&format!("noor-bg-{color}"), start, end);
    }
}
fn replace_selection_tag(buffer: &gtk::TextBuffer, prefix: &str, tag: &str) {
    let Some((start, end)) = buffer.selection_bounds() else {
        return;
    };
    let mut names = Vec::new();
    buffer.tag_table().foreach(|candidate| {
        if let Some(name) = candidate.name().filter(|name| name.starts_with(prefix)) {
            names.push(name.to_string());
        }
    });
    for name in names {
        buffer.remove_tag_by_name(&name, &start, &end);
    }
    buffer.apply_tag_by_name(tag, &start, &end);
}

fn tag_suffix(names: &[String], prefix: &str) -> Option<String> {
    names
        .iter()
        .find_map(|name| name.strip_prefix(prefix).map(str::to_string))
}

fn alignment_tag(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Start => "noor-align-start",
        Alignment::Center => "noor-align-center",
        Alignment::End => "noor-align-end",
        Alignment::Justify => "noor-align-justify",
    }
}

fn alignment_at(iter: &gtk::TextIter) -> Alignment {
    let names = iter
        .tags()
        .into_iter()
        .filter_map(|tag| tag.name().map(|name| name.to_string()))
        .collect::<Vec<_>>();
    if names.iter().any(|name| name == "noor-align-center") {
        Alignment::Center
    } else if names.iter().any(|name| name == "noor-align-end") {
        Alignment::End
    } else if names.iter().any(|name| name == "noor-align-justify") {
        Alignment::Justify
    } else {
        Alignment::Start
    }
}

fn marks_at(iter: &gtk::TextIter) -> TextMarks {
    let names = iter
        .tags()
        .into_iter()
        .filter_map(|tag| tag.name().map(|name| name.to_string()))
        .collect::<Vec<_>>();
    TextMarks {
        bold: names.iter().any(|name| name == BOLD),
        italic: names.iter().any(|name| name == ITALIC),
        underline: names.iter().any(|name| name == UNDERLINE),
        strikethrough: names.iter().any(|name| name == STRIKE),
        font_size: tag_suffix(&names, "noor-size-").and_then(|value| value.parse().ok()),
        foreground: tag_suffix(&names, "noor-fg-"),
        highlight: tag_suffix(&names, "noor-bg-"),
    }
}
