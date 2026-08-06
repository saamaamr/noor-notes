#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorStatistics {
    pub line: usize,
    pub column: usize,
    pub lines: usize,
    pub words: usize,
    pub characters: usize,
    pub selection: usize,
    pub zoom: u16,
}

impl EditorStatistics {
    pub fn calculate(
        text: &str,
        cursor: usize,
        selection: Option<(usize, usize)>,
        zoom: u16,
    ) -> Self {
        let cursor = cursor.min(text.chars().count());
        let before: String = text.chars().take(cursor).collect();
        let line = before
            .chars()
            .filter(|character| *character == '\n')
            .count()
            + 1;
        let column = before
            .rsplit_once('\n')
            .map_or(before.chars().count() + 1, |(_, tail)| {
                tail.chars().count() + 1
            });
        Self {
            line,
            column,
            lines: if text.is_empty() {
                1
            } else {
                text.chars().filter(|character| *character == '\n').count() + 1
            },
            words: text.split_whitespace().count(),
            characters: text.chars().count(),
            selection: selection
                .map(|(start, end)| start.abs_diff(end))
                .unwrap_or_default(),
            zoom: clamp_zoom(zoom),
        }
    }
}

pub fn clamp_zoom(zoom: u16) -> u16 {
    zoom.clamp(50, 300)
}

pub fn line_offset(text: &str, requested_line: usize) -> usize {
    let requested_line = requested_line.max(1);
    let mut line = 1;
    for (offset, character) in text.chars().enumerate() {
        if line == requested_line {
            return offset;
        }
        if character == '\n' {
            line += 1;
        }
    }
    text.char_indices()
        .rev()
        .find_map(|(byte, character)| (character == '\n').then(|| text[..=byte].chars().count()))
        .unwrap_or(0)
}
