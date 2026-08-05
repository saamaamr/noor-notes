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
