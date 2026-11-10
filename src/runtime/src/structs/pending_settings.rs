use minimap_core::structs::settings::{SettingValue, SettingWrite};

// the values the player changes in play, waiting to be saved as manager settings; one slot per path, so the
// last change wins and the queue never holds more than five writes
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PendingSettings {
    slots: [Option<SettingValue>; 5],
}

impl PendingSettings {
    pub const RADIUS: &'static str = "mini_map.radius_chunks";
    pub const MINI_MAP_COORDINATES: &'static str = "mini_map.show_coordinates";
    pub const WORLD_MAP_COORDINATES: &'static str = "world_map.show_coordinates";
    pub const CHUNK_GRID: &'static str = "world_map.show_chunk_grid";
    pub const SLIME_CHUNKS: &'static str = "world_map.show_slime_chunks";
    const PATHS: [&'static str; 5] =
        [Self::RADIUS, Self::MINI_MAP_COORDINATES, Self::WORLD_MAP_COORDINATES, Self::CHUNK_GRID, Self::SLIME_CHUNKS];

    pub fn new() -> Self {
        Self::default()
    }

    // a path that is not one of the five is ignored
    pub fn set(&mut self, path: &'static str, value: SettingValue) {
        if let Some(index) = Self::PATHS.iter().position(|known| *known == path) {
            self.slots[index] = Some(value);
        }
    }

    // appends the waiting writes to `out`, in path order, and empties the queue
    pub fn drain(&mut self, out: &mut Vec<SettingWrite>) {
        for (path, slot) in Self::PATHS.iter().zip(self.slots.iter_mut()) {
            if let Some(value) = slot.take() {
                out.push(SettingWrite { path, value });
            }
        }
    }
}
