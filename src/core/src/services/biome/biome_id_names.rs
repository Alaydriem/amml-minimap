use std::collections::{HashMap, VecDeque};

// the game's internal biome name for each numeric biome ID, learned by asking the game. An ID
// waits until the game answers it: the game can name a biome only while a chunk holding it is
// probed, so waiting IDs are asked again, a window of them each tick in turn. The answers and the
// waiting IDs together never pass `capacity`
pub struct BiomeIdNames {
    capacity: usize,
    // None: the game has no usable name for the ID
    names: HashMap<u16, Option<Box<str>>>,
    // asked and not answered yet, in the order they are asked again
    pending: VecDeque<u16>,
}

impl BiomeIdNames {
    pub const CAPACITY: usize = 1024;
    pub const REQUESTS_PER_TICK: usize = 16;
    pub const MAX_NAME_BYTES: usize = 64;

    pub fn new(capacity: usize) -> Self {
        Self { capacity, names: HashMap::new(), pending: VecDeque::new() }
    }

    pub fn name(&self, id: u16) -> Option<&str> {
        self.names.get(&id).and_then(|name| name.as_deref())
    }

    // whether the game has answered for the ID, with a name or with "no name"
    pub fn is_answered(&self, id: u16) -> bool {
        self.names.contains_key(&id)
    }

    // starts asking for the ID; an answered or waiting ID, or a full map, adds nothing
    pub fn request(&mut self, id: u16) {
        let full = self.names.len() + self.pending.len() >= self.capacity;
        if full || self.names.contains_key(&id) || self.pending.contains(&id) {
            return;
        }
        self.pending.push_back(id);
    }

    // the next window of waiting IDs; they move to the back, so every waiting ID is asked in turn
    pub fn take_requests(&mut self, out: &mut Vec<u16>) {
        out.clear();
        let count = self.pending.len().min(Self::REQUESTS_PER_TICK);
        out.extend(self.pending.iter().take(count).copied());
        self.pending.rotate_left(count);
    }

    // an empty or overlong name is recorded as "no name", so the ID is not asked again
    pub fn answer(&mut self, id: u16, name: &str) {
        self.pending.retain(|waiting| *waiting != id);
        if !self.names.contains_key(&id) && self.names.len() >= self.capacity {
            return;
        }
        let usable = !name.is_empty() && name.len() <= Self::MAX_NAME_BYTES;
        self.names.insert(id, usable.then(|| name.into()));
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}
