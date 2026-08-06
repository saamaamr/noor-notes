#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchToken(u64);

#[derive(Clone, Debug, Default)]
pub struct SearchGeneration {
    current: u64,
}

impl SearchGeneration {
    pub fn begin(&mut self) -> SearchToken {
        self.current = self.current.wrapping_add(1);
        SearchToken(self.current)
    }

    pub const fn is_current(&self, token: SearchToken) -> bool {
        token.0 == self.current
    }
}
