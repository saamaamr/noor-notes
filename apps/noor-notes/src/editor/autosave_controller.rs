#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveGeneration(u64);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SavePhase {
    Unsaved,
    Saving,
    #[default]
    Saved,
    Failed(String),
}

#[derive(Clone, Debug, Default)]
pub struct AutosaveController {
    generation: u64,
    phase: SavePhase,
}

impl AutosaveController {
    pub const fn phase(&self) -> &SavePhase {
        &self.phase
    }

    pub fn mark_dirty(&mut self) -> SaveGeneration {
        self.generation = self.generation.wrapping_add(1);
        self.phase = SavePhase::Unsaved;
        SaveGeneration(self.generation)
    }

    pub fn begin_save(&mut self, generation: SaveGeneration) {
        if self.is_current(generation) {
            self.phase = SavePhase::Saving;
        }
    }

    pub fn finish(&mut self, generation: SaveGeneration) {
        if self.is_current(generation) {
            self.phase = SavePhase::Saved;
        }
    }

    pub fn fail(&mut self, generation: SaveGeneration, message: impl Into<String>) {
        if self.is_current(generation) {
            self.phase = SavePhase::Failed(message.into());
        }
    }

    pub const fn is_current(&self, generation: SaveGeneration) -> bool {
        generation.0 == self.generation
    }
}
