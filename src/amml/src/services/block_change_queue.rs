use std::collections::VecDeque;

use crate::types::QueuedColumn;

// the columns block changes ask to read again, until the next client tick takes them. A column already queued is
// not queued twice; past CAPACITY columns a change is dropped (the scan plan reads the chunk again later)
pub struct BlockChangeQueue {
    columns: VecDeque<QueuedColumn>,
    dropped: u64,
}

impl BlockChangeQueue {
    pub const CAPACITY: usize = 256;

    pub fn new() -> Self {
        Self { columns: VecDeque::with_capacity(Self::CAPACITY), dropped: 0 }
    }

    // false when the queue is full and the column was dropped
    pub fn push(&mut self, column: QueuedColumn) -> bool {
        if self.columns.contains(&column) {
            return true;
        }
        if self.columns.len() >= Self::CAPACITY {
            self.dropped += 1;
            return false;
        }
        self.columns.push_back(column);
        true
    }

    pub fn pop(&mut self) -> Option<QueuedColumn> {
        self.columns.pop_front()
    }

    pub fn clear(&mut self) {
        self.columns.clear();
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    // the changes dropped on a full queue since the queue was made
    pub fn dropped(&self) -> u64 {
        self.dropped
    }
}

impl Default for BlockChangeQueue {
    fn default() -> Self {
        Self::new()
    }
}
