use noor_domain::NoteId;

#[derive(Clone, Debug)]
pub struct RecentItems {
    items: Vec<NoteId>,
    limit: usize,
}

impl RecentItems {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            items: Vec::new(),
            limit,
        }
    }

    pub fn touch(&mut self, id: NoteId) {
        self.items.retain(|candidate| *candidate != id);
        self.items.insert(0, id);
        self.items.truncate(self.limit);
    }

    pub fn items(&self) -> &[NoteId] {
        &self.items
    }
}
