#[derive(Debug, Default)]
pub struct EditSaveGate {
    dirty: bool,
    timer_armed: bool,
}

impl EditSaveGate {
    pub fn mark_changed(&mut self) -> bool {
        self.dirty = true;
        if self.timer_armed {
            false
        } else {
            self.timer_armed = true;
            true
        }
    }

    pub fn take_snapshot(&mut self) -> bool {
        self.timer_armed = false;
        std::mem::take(&mut self.dirty)
    }
}
