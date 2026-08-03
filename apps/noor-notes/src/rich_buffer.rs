use adw::prelude::*;
use noor_domain::{RichBlock, RichDocument, RichSpan, TextMarks};

const BOLD: &str = "noor-bold";
const ITALIC: &str = "noor-italic";
const UNDERLINE: &str = "noor-underline";
const STRIKE: &str = "noor-strike";
const LINK: &str = "noor-link";

pub struct RichBuffer;

impl RichBuffer {
    pub fn prepare(buffer: &gtk::TextBuffer) {
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
            for span in &block.spans {
                let start = cursor;
                buffer.insert(&mut cursor, &span.text);
                apply_marks(buffer, &start, &cursor, &span.marks);
            }
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

    pub fn insert_list_prefix(buffer: &gtk::TextBuffer, prefix: &str) {
        let mut iter = buffer.iter_at_mark(&buffer.get_insert());
        iter.set_line_offset(0);
        buffer.insert(&mut iter, prefix);
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
        ..TextMarks::default()
    }
}
