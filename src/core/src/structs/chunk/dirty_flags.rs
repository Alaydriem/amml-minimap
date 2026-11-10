use crate::structs::chunk::DirtyEpochs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DirtyFlags {
    full: bool,
    soft: bool,
    epochs: DirtyEpochs,
}

impl DirtyFlags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn is_soft(&self) -> bool {
        self.soft
    }

    pub fn epochs(&self) -> DirtyEpochs {
        self.epochs
    }

    pub fn mark_full(&mut self) {
        self.full = true;
        self.epochs.full = self.epochs.full.wrapping_add(1);
    }

    pub fn mark_soft(&mut self) {
        self.soft = true;
        self.epochs.soft = self.epochs.soft.wrapping_add(1);
    }

    // clears the full mark only when no mark arrived after the snapshot was taken
    pub fn clear_full_if(&mut self, snapshot: u32) -> bool {
        if self.epochs.full != snapshot {
            return false;
        }
        self.full = false;
        true
    }

    pub fn clear_soft_if(&mut self, snapshot: u32) -> bool {
        if self.epochs.soft != snapshot {
            return false;
        }
        self.soft = false;
        true
    }
}
