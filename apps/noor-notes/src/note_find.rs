#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FindOptions {
    pub match_case: bool,
    pub whole_word: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FindResults {
    ranges: Vec<(usize, usize)>,
    current: Option<usize>,
}

impl FindResults {
    pub fn new(text: &str, query: &str) -> Self {
        let mut value = Self::default();
        value.update(text, query);
        value
    }
    pub fn with_options(text: &str, query: &str, options: FindOptions) -> Self {
        let ranges = find_ranges(text, query, options);
        let current = if ranges.is_empty() { None } else { Some(0) };
        Self { ranges, current }
    }
    pub fn update_with_options(&mut self, text: &str, query: &str, options: FindOptions) {
        let updated = Self::with_options(text, query, options);
        self.ranges = updated.ranges;
        self.current = updated.current;
    }
    pub fn update(&mut self, text: &str, query: &str) {
        self.ranges.clear();
        self.current = None;
        let needle: Vec<char> = query.to_lowercase().chars().collect();
        if needle.is_empty() {
            return;
        }
        let mut folded = Vec::new();
        let mut origins = Vec::new();
        for (index, character) in text.chars().enumerate() {
            for lower in character.to_lowercase() {
                folded.push(lower);
                origins.push(index);
            }
        }
        if folded.len() < needle.len() {
            return;
        }
        for start in 0..=folded.len().saturating_sub(needle.len()) {
            if folded[start..start + needle.len()] == needle {
                let first = origins[start];
                let last = origins[start + needle.len() - 1] + 1;
                if self.ranges.last().is_none_or(|range| range.1 <= first) {
                    self.ranges.push((first, last));
                }
            }
        }
        if !self.ranges.is_empty() {
            self.current = Some(0);
        }
    }
    pub fn ranges(&self) -> &[(usize, usize)] {
        &self.ranges
    }
    pub fn position(&self) -> Option<(usize, usize)> {
        self.current.map(|current| (current + 1, self.ranges.len()))
    }
    pub fn current_range(&self) -> Option<(usize, usize)> {
        self.current.map(|current| self.ranges[current])
    }
    pub fn next(&mut self) {
        if !self.ranges.is_empty() {
            self.current = Some((self.current.unwrap_or(0) + 1) % self.ranges.len());
        }
    }
    pub fn previous(&mut self) {
        if !self.ranges.is_empty() {
            self.current =
                Some((self.current.unwrap_or(0) + self.ranges.len() - 1) % self.ranges.len());
        }
    }
}

pub fn replace_current(
    text: &str,
    results: &FindResults,
    replacement: &str,
) -> Option<(String, usize)> {
    let (start, end) = results.current_range()?;
    let start_byte = char_to_byte(text, start);
    let end_byte = char_to_byte(text, end);
    let mut output = String::with_capacity(text.len() + replacement.len());
    output.push_str(&text[..start_byte]);
    output.push_str(replacement);
    output.push_str(&text[end_byte..]);
    Some((output, start + replacement.chars().count()))
}

pub fn replace_all(
    text: &str,
    query: &str,
    replacement: &str,
    options: FindOptions,
) -> (String, usize) {
    let results = FindResults::with_options(text, query, options);
    let mut output = String::with_capacity(text.len());
    let mut previous = 0;
    for &(start, end) in results.ranges() {
        let start_byte = char_to_byte(text, start);
        let end_byte = char_to_byte(text, end);
        output.push_str(&text[previous..start_byte]);
        output.push_str(replacement);
        previous = end_byte;
    }
    output.push_str(&text[previous..]);
    (output, results.ranges().len())
}

fn find_ranges(text: &str, query: &str, options: FindOptions) -> Vec<(usize, usize)> {
    let needle: Vec<char> = comparable(query, options.match_case);
    if needle.is_empty() {
        return Vec::new();
    }
    let mut haystack = Vec::new();
    let mut origins = Vec::new();
    for (index, character) in text.chars().enumerate() {
        let value = character.to_string();
        for comparable in comparable(&value, options.match_case) {
            haystack.push(comparable);
            origins.push(index);
        }
    }
    if haystack.len() < needle.len() {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    for start in 0..=haystack.len() - needle.len() {
        if haystack[start..start + needle.len()] != needle {
            continue;
        }
        let range = (origins[start], origins[start + needle.len() - 1] + 1);
        if options.whole_word && !is_whole_word(text, range.0, range.1) {
            continue;
        }
        if ranges
            .last()
            .is_none_or(|previous: &(usize, usize)| previous.1 <= range.0)
        {
            ranges.push(range);
        }
    }
    ranges
}

fn comparable(value: &str, match_case: bool) -> Vec<char> {
    if match_case {
        value.chars().collect()
    } else {
        value.to_lowercase().chars().collect()
    }
}

fn char_to_byte(text: &str, char_offset: usize) -> usize {
    text.char_indices()
        .nth(char_offset)
        .map_or(text.len(), |(offset, _)| offset)
}

fn is_whole_word(text: &str, start: usize, end: usize) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let word = |character: char| character.is_alphanumeric() || character == '_';
    (start == 0 || !word(chars[start - 1])) && (end >= chars.len() || !word(chars[end]))
}
