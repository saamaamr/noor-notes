use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;
use noor_domain::{Alignment, ListKind, RichBlock, RichDocument, RichSpan, TextMarks};

use crate::appearance::EffectiveTheme;
use crate::rich_color::{
    ColorRole, normalize_stored, presets, rendered_color, stored_value_from_tag, tag_name,
};

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

/// A focus-independent snapshot of the logical insertion and selection marks.
/// Offsets are clamped on restore so an intervening buffer change cannot make
/// a toolbar command address an invalid range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SavedTextRange {
    insert: i32,
    bound: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RichHistorySnapshot {
    content: String,
    document: RichDocument,
    range: SavedTextRange,
}

#[derive(Default)]
struct RichHistory {
    snapshots: Vec<RichHistorySnapshot>,
    position: usize,
    applying: bool,
    action_depth: usize,
    dirty: bool,
    ignore_next_changed: bool,
}

impl SavedTextRange {
    pub fn capture(buffer: &gtk::TextBuffer) -> Self {
        Self {
            insert: buffer.iter_at_mark(&buffer.get_insert()).offset(),
            bound: buffer.iter_at_mark(&buffer.selection_bound()).offset(),
        }
    }

    pub fn restore(self, buffer: &gtk::TextBuffer) {
        let last = buffer.char_count();
        let insert = buffer.iter_at_offset(self.insert.clamp(0, last));
        let bound = buffer.iter_at_offset(self.bound.clamp(0, last));
        if insert.offset() == bound.offset() {
            buffer.place_cursor(&insert);
        } else {
            buffer.select_range(&insert, &bound);
        }
    }
}

pub struct RichBuffer;

impl RichBuffer {
    pub fn prepare(buffer: &gtk::TextBuffer) {
        buffer.set_enable_undo(true);
        ensure_history(buffer);
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
        for role in [ColorRole::Foreground, ColorRole::Highlight] {
            for preset in presets(role) {
                ensure_color_tag(buffer, role, preset.id, EffectiveTheme::Light);
            }
        }
        ensure_color_tag(
            buffer,
            ColorRole::Highlight,
            "charcoal",
            EffectiveTheme::Light,
        );
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
        let history = history(buffer);
        history.borrow_mut().applying = true;
        let Some(document) = document.filter(|document| document.is_supported()) else {
            buffer.set_text(content);
            Self::tag_urls(buffer);
            let mut history = history.borrow_mut();
            history.applying = false;
            reset_history(buffer, &mut history);
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
        let mut history = history.borrow_mut();
        history.applying = false;
        reset_history(buffer, &mut history);
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
        begin_history_action(buffer, true);
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
        end_history_action(buffer, true);
    }

    pub fn can_undo(buffer: &gtk::TextBuffer) -> bool {
        history(buffer).borrow().position > 0
    }
    pub fn can_redo(buffer: &gtk::TextBuffer) -> bool {
        let history = history(buffer);
        let history = history.borrow();
        history.position + 1 < history.snapshots.len()
    }
    pub fn undo(buffer: &gtk::TextBuffer) {
        restore_history(buffer, false);
    }
    pub fn redo(buffer: &gtk::TextBuffer) {
        restore_history(buffer, true);
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
        apply_color(buffer, ColorRole::Foreground, color);
    }

    pub fn clear_foreground(buffer: &gtk::TextBuffer) {
        clear_selection_tags(buffer, ColorRole::Foreground.tag_prefix());
    }

    pub fn clear_highlight(buffer: &gtk::TextBuffer) {
        clear_selection_tags(buffer, ColorRole::Highlight.tag_prefix());
    }

    pub fn apply_color_theme(buffer: &gtk::TextBuffer, theme: EffectiveTheme) {
        for role in [ColorRole::Foreground, ColorRole::Highlight] {
            for preset in presets(role) {
                update_color_tag(buffer, role, preset.id, theme);
            }
        }
        update_color_tag(buffer, ColorRole::Highlight, "charcoal", theme);
    }

    pub fn align(buffer: &gtk::TextBuffer, alignment: Alignment) {
        let (first, last) = selected_lines(buffer);
        begin_history_action(buffer, true);
        for line in first..=last {
            let Some(start) = buffer.iter_at_line(line) else {
                continue;
            };
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
        end_history_action(buffer, true);
    }

    pub fn highlight(buffer: &gtk::TextBuffer, color: &str) {
        apply_color(buffer, ColorRole::Highlight, color);
    }

    pub fn clear_formatting(buffer: &gtk::TextBuffer) {
        let Some((start, end)) = buffer.selection_bounds() else {
            return;
        };
        let mut names = Vec::new();
        buffer.tag_table().foreach(|tag| {
            if let Some(name) = tag.name().filter(|name| name.starts_with("noor-")) {
                names.push(name.to_string());
            }
        });
        begin_history_action(buffer, true);
        for name in names {
            buffer.remove_tag_by_name(&name, &start, &end);
        }
        end_history_action(buffer, true);
    }

    pub fn toggle_list(buffer: &gtk::TextBuffer, kind: ListKind) {
        let (first, last) = selected_lines(buffer);
        let all_match = (first..=last).all(|line| {
            line_text(buffer, line)
                .as_deref()
                .and_then(list_marker)
                .is_some_and(|(current, _, _)| current == kind)
        });
        begin_history_action(buffer, false);
        for line in (first..=last).rev() {
            replace_line_marker(
                buffer,
                line,
                if all_match { None } else { Some(kind) },
                line - first + 1,
            );
        }
        end_history_action(buffer, false);
    }

    pub fn list_kind_at_cursor(buffer: &gtk::TextBuffer) -> Option<ListKind> {
        let line = buffer.iter_at_mark(&buffer.get_insert()).line();
        line_text(buffer, line)
            .as_deref()
            .and_then(list_marker)
            .map(|value| value.0)
    }

    pub fn list_kind_for_selection(buffer: &gtk::TextBuffer) -> Option<ListKind> {
        let (first, last) = selected_lines(buffer);
        let first_kind = line_text(buffer, first).as_deref().and_then(list_marker);
        let kind = first_kind.map(|value| value.0);
        (first..=last)
            .all(|line| {
                line_text(buffer, line)
                    .as_deref()
                    .and_then(list_marker)
                    .map(|value| value.0)
                    == kind
            })
            .then_some(kind)
            .flatten()
    }

    pub fn marks_at_cursor(buffer: &gtk::TextBuffer) -> TextMarks {
        let mut iter = buffer.iter_at_mark(&buffer.get_insert());
        if iter.offset() > 0 && iter.is_end() {
            iter.backward_char();
        }
        marks_at(&iter)
    }

    /// Returns the uniform marks for the selection/cursor. `None` represents a
    /// mixed selection so checked controls can show a neutral state.
    pub fn marks_for_selection(buffer: &gtk::TextBuffer) -> Option<TextMarks> {
        let Some((start, end)) = buffer.selection_bounds() else {
            return Some(Self::marks_at_cursor(buffer));
        };
        if start.offset() == end.offset() {
            return Some(marks_at(&start));
        }
        let expected = marks_at(&start);
        let mut iter = start;
        while iter.offset() < end.offset() {
            if marks_at(&iter) != expected {
                return None;
            }
            if !iter.forward_char() {
                break;
            }
        }
        Some(expected)
    }

    pub fn alignment_for_selection(buffer: &gtk::TextBuffer) -> Option<Alignment> {
        let (first, last) = selected_lines(buffer);
        let first_alignment = buffer.iter_at_line(first).map(|iter| alignment_at(&iter))?;
        (first..=last)
            .all(|line| {
                buffer.iter_at_line(line).map(|iter| alignment_at(&iter)) == Some(first_alignment)
            })
            .then_some(first_alignment)
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
            begin_history_action(buffer, false);
            replace_line_marker(buffer, line, None, 1);
            end_history_action(buffer, false);
            return true;
        }
        let prefix = match kind {
            ListKind::Bullet => "\n• ".to_string(),
            ListKind::Numbered => format!("\n{}. ", number.unwrap_or(1) + 1),
        };
        begin_history_action(buffer, false);
        buffer.insert_at_cursor(&prefix);
        end_history_action(buffer, false);
        true
    }

    pub fn insert_emoji(buffer: &gtk::TextBuffer, emoji: &str) {
        begin_history_action(buffer, false);
        buffer.insert_at_cursor(emoji);
        end_history_action(buffer, false);
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

fn history_quark() -> glib::Quark {
    glib::Quark::from_str("noor-rich-document-history")
}

fn ensure_history(buffer: &gtk::TextBuffer) {
    let key = history_quark();
    // SAFETY: this private quark is written and read only by this module with
    // exactly the same `Rc<RefCell<RichHistory>>` type.
    if unsafe { buffer.qdata::<Rc<RefCell<RichHistory>>>(key).is_some() } {
        return;
    }

    let history_cell = Rc::new(RefCell::new(RichHistory::default()));
    // SAFETY: see the type invariant above; the buffer owns the qdata value.
    unsafe { buffer.set_qdata(key, history_cell) };

    buffer.connect_begin_user_action(|buffer| {
        let history = history(buffer);
        let mut history = history.borrow_mut();
        if !history.applying {
            history.action_depth += 1;
        }
    });
    buffer.connect_end_user_action(|buffer| {
        let history = history(buffer);
        let mut history = history.borrow_mut();
        if history.applying || history.action_depth == 0 {
            return;
        }
        history.action_depth -= 1;
        if history.action_depth == 0 && history.dirty {
            history.dirty = false;
            record_history(buffer, &mut history);
        }
    });
    buffer.connect_changed(|buffer| {
        let history = history(buffer);
        let mut history = history.borrow_mut();
        if history.applying {
            return;
        }
        if history.ignore_next_changed {
            history.ignore_next_changed = false;
            return;
        }
        if history.action_depth > 0 {
            history.dirty = true;
        } else {
            record_history(buffer, &mut history);
        }
    });

    let history = history(buffer);
    reset_history(buffer, &mut history.borrow_mut());
}

fn history(buffer: &gtk::TextBuffer) -> Rc<RefCell<RichHistory>> {
    let key = history_quark();
    // SAFETY: `ensure_history` establishes this module-private qdata type.
    unsafe {
        buffer
            .qdata::<Rc<RefCell<RichHistory>>>(key)
            .expect("RichBuffer::prepare installs history")
            .as_ref()
            .clone()
    }
}

fn history_snapshot(buffer: &gtk::TextBuffer) -> RichHistorySnapshot {
    let (content, document) = RichBuffer::snapshot(buffer);
    RichHistorySnapshot {
        content,
        document,
        range: SavedTextRange::capture(buffer),
    }
}

fn reset_history(buffer: &gtk::TextBuffer, history: &mut RichHistory) {
    history.snapshots.clear();
    history.snapshots.push(history_snapshot(buffer));
    history.position = 0;
    history.action_depth = 0;
    history.dirty = false;
    history.ignore_next_changed = false;
}

fn record_history(buffer: &gtk::TextBuffer, history: &mut RichHistory) {
    let snapshot = history_snapshot(buffer);
    if let Some(current) = history.snapshots.get_mut(history.position) {
        if current.content == snapshot.content && current.document == snapshot.document {
            current.range = snapshot.range;
            return;
        }
    }
    history.snapshots.truncate(history.position + 1);
    history.snapshots.push(snapshot);
    history.position = history.snapshots.len() - 1;
    const MAX_HISTORY: usize = 250;
    if history.snapshots.len() > MAX_HISTORY {
        history.snapshots.remove(0);
        history.position = history.position.saturating_sub(1);
    }
}

fn restore_history(buffer: &gtk::TextBuffer, redo: bool) {
    let history_cell = history(buffer);
    let snapshot = {
        let mut history = history_cell.borrow_mut();
        let target = if redo {
            history.position.checked_add(1)
        } else {
            history.position.checked_sub(1)
        };
        let Some(target) = target.filter(|target| *target < history.snapshots.len()) else {
            return;
        };
        history.position = target;
        history.applying = true;
        history.snapshots[target].clone()
    };

    restore_snapshot(buffer, &snapshot);

    {
        let mut history = history_cell.borrow_mut();
        history.applying = false;
        history.ignore_next_changed = true;
    }
    buffer.emit_by_name::<()>("changed", &[]);
}

fn restore_snapshot(buffer: &gtk::TextBuffer, snapshot: &RichHistorySnapshot) {
    buffer.set_text("");
    let mut cursor = buffer.end_iter();
    for (block_index, block) in snapshot.document.blocks.iter().enumerate() {
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
    snapshot.range.restore(buffer);
}

fn begin_history_action(buffer: &gtk::TextBuffer, tag_only: bool) {
    buffer.begin_user_action();
    if tag_only {
        history(buffer).borrow_mut().dirty = true;
    }
}

fn end_history_action(buffer: &gtk::TextBuffer, notify_tag_change: bool) {
    buffer.end_user_action();
    if notify_tag_change {
        history(buffer).borrow_mut().ignore_next_changed = true;
        buffer.emit_by_name::<()>("changed", &[]);
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
        if let Some(name) =
            ensure_color_tag(buffer, ColorRole::Foreground, color, EffectiveTheme::Light)
        {
            buffer.apply_tag_by_name(&name, start, end);
        }
    }
    if let Some(color) = &marks.highlight {
        if let Some(name) =
            ensure_color_tag(buffer, ColorRole::Highlight, color, EffectiveTheme::Light)
        {
            buffer.apply_tag_by_name(&name, start, end);
        }
    }
}
fn apply_color(buffer: &gtk::TextBuffer, role: ColorRole, value: &str) {
    let Some(name) = ensure_color_tag(buffer, role, value, EffectiveTheme::Light) else {
        return;
    };
    replace_selection_tag(buffer, role.tag_prefix(), &name);
}

fn ensure_color_tag(
    buffer: &gtk::TextBuffer,
    role: ColorRole,
    value: &str,
    theme: EffectiveTheme,
) -> Option<String> {
    let stored = normalize_stored(role, value)?;
    let name = tag_name(role, &stored)?;
    if buffer.tag_table().lookup(&name).is_none() {
        let rendered = rendered_color(role, &stored, theme)?;
        let tag = match role {
            ColorRole::Foreground => gtk::TextTag::builder()
                .name(&name)
                .foreground(&rendered)
                .build(),
            ColorRole::Highlight => gtk::TextTag::builder()
                .name(&name)
                .background(&rendered)
                .build(),
        };
        buffer.tag_table().add(&tag);
    }
    Some(name)
}

fn update_color_tag(buffer: &gtk::TextBuffer, role: ColorRole, value: &str, theme: EffectiveTheme) {
    let Some(name) = ensure_color_tag(buffer, role, value, theme) else {
        return;
    };
    let Some(rendered) = rendered_color(role, value, theme) else {
        return;
    };
    let Some(tag) = buffer.tag_table().lookup(&name) else {
        return;
    };
    match role {
        ColorRole::Foreground => tag.set_foreground(Some(&rendered)),
        ColorRole::Highlight => tag.set_background(Some(&rendered)),
    }
}

fn clear_selection_tags(buffer: &gtk::TextBuffer, prefix: &str) {
    let Some((start, end)) = buffer.selection_bounds() else {
        return;
    };
    let mut names = Vec::new();
    buffer.tag_table().foreach(|candidate| {
        if let Some(name) = candidate.name().filter(|name| name.starts_with(prefix)) {
            names.push(name.to_string());
        }
    });
    begin_history_action(buffer, true);
    for name in names {
        buffer.remove_tag_by_name(&name, &start, &end);
    }
    end_history_action(buffer, true);
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
    buffer.begin_user_action();
    for name in names {
        buffer.remove_tag_by_name(&name, &start, &end);
    }
    buffer.apply_tag_by_name(tag, &start, &end);
    buffer.end_user_action();
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
        foreground: names
            .iter()
            .find_map(|name| stored_value_from_tag(ColorRole::Foreground, name)),
        highlight: names
            .iter()
            .find_map(|name| stored_value_from_tag(ColorRole::Highlight, name)),
    }
}
