use crate::errors::WaypointError;
use crate::structs::waypoint::{Waypoint, WaypointEdit, WaypointIcon, WaypointId, WaypointSource};

// the waypoints of the current world, every dimension; at most `capacity`, and the death point may
// take one place over it, so a death is never lost to a full list
pub struct WaypointStore {
    capacity: usize,
    waypoints: Vec<Waypoint>,
    seed: u64,
    next_colour: usize,
    revision: u64,
    dirty: bool,
}

impl WaypointStore {
    pub const MAX_NAME_CHARS: usize = 32;
    pub const PALETTE: [[u8; 4]; 12] = [
        [80, 170, 255, 255],
        [255, 90, 90, 255],
        [120, 220, 100, 255],
        [255, 200, 60, 255],
        [200, 120, 255, 255],
        [60, 220, 220, 255],
        [255, 140, 40, 255],
        [255, 110, 200, 255],
        [190, 240, 70, 255],
        [160, 105, 60, 255],
        [110, 120, 255, 255],
        [250, 245, 235, 255],
    ];
    pub const DEATH_NAME: &'static str = "Death";
    pub const DEATH_COLOUR: [u8; 4] = [230, 230, 230, 255];

    // seed makes the ids; the runtime passes the clock, tests pass a constant
    pub fn new(capacity: usize, seed: u64) -> Self {
        Self { capacity, waypoints: Vec::new(), seed, next_colour: 0, revision: 0, dirty: false }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }

    pub fn waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }

    // goes up on every change, so a reader can tell when its copy is old
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn get(&self, id: WaypointId) -> Option<&Waypoint> {
        self.waypoints.iter().find(|w| w.id == id)
    }

    pub fn death_point(&self) -> Option<&Waypoint> {
        self.waypoints.iter().find(|w| w.source == WaypointSource::Death)
    }

    // the colour the next add takes
    pub fn next_colour(&self) -> [u8; 4] {
        Self::PALETTE[self.next_colour % Self::PALETTE.len()]
    }

    // adds with the next palette colour; a blank name becomes "Waypoint N"
    pub fn add(
        &mut self,
        name: &str,
        dim: i32,
        pos: (i32, i32, i32),
        source: WaypointSource,
        now: u64,
    ) -> Result<WaypointId, WaypointError> {
        if self.waypoints.len() >= self.capacity {
            return Err(WaypointError::Full(self.capacity));
        }
        let id = self.next_id();
        let name = self.clean_name(name);
        let colour = Self::PALETTE[self.next_colour % Self::PALETTE.len()];
        self.next_colour += 1;
        let (x, y, z) = pos;
        self.waypoints.push(Waypoint {
            id,
            name,
            dim,
            x,
            y,
            z,
            colour,
            icon: WaypointIcon::Diamond,
            show_on_edge: true,
            show_distance: true,
            source,
            created: now,
        });
        self.touch();
        Ok(id)
    }

    // moves the death point to the new place, or makes it; never refused
    pub fn set_death(&mut self, dim: i32, pos: (i32, i32, i32), now: u64) -> WaypointId {
        let (x, y, z) = pos;
        if let Some(death) = self.waypoints.iter_mut().find(|w| w.source == WaypointSource::Death) {
            (death.dim, death.x, death.y, death.z, death.created) = (dim, x, y, z, now);
            let id = death.id;
            self.touch();
            return id;
        }
        let id = self.next_id();
        self.waypoints.push(Waypoint {
            id,
            name: Self::DEATH_NAME.to_string(),
            dim,
            x,
            y,
            z,
            colour: Self::DEATH_COLOUR,
            icon: WaypointIcon::Skull,
            show_on_edge: true,
            show_distance: true,
            source: WaypointSource::Death,
            created: now,
        });
        self.touch();
        id
    }

    // the waypoint and where it was in the list, for `restore`
    pub fn remove(&mut self, id: WaypointId) -> Option<(usize, Waypoint)> {
        let index = self.waypoints.iter().position(|w| w.id == id)?;
        let removed = self.waypoints.remove(index);
        self.touch();
        Some((index, removed))
    }

    // puts a removed waypoint back at its place; refused when its id is in use or there is no room
    pub fn restore(&mut self, index: usize, waypoint: Waypoint) -> bool {
        let room = if waypoint.source == WaypointSource::Death {
            self.death_point().is_none()
        } else {
            self.waypoints.len() < self.capacity
        };
        if !room || self.get(waypoint.id).is_some() {
            return false;
        }
        self.waypoints.insert(index.min(self.waypoints.len()), waypoint);
        self.touch();
        true
    }

    // the name is cleaned as for `add`, but a blank name keeps the current one
    pub fn update(&mut self, id: WaypointId, edit: &WaypointEdit) -> bool {
        let name: String = edit.name.trim().chars().take(Self::MAX_NAME_CHARS).collect();
        let Some(waypoint) = self.waypoints.iter_mut().find(|w| w.id == id) else { return false };
        if !name.is_empty() {
            waypoint.name = name;
        }
        waypoint.colour = edit.colour;
        waypoint.show_on_edge = edit.show_on_edge;
        waypoint.show_distance = edit.show_distance;
        self.touch();
        true
    }

    // waypoints read from the file join any made since the world was entered; entries with an id
    // already present, a second death point, or no room are dropped. Returns how many were dropped
    pub fn merge_loaded(&mut self, loaded: Vec<Waypoint>) -> usize {
        let mut dropped = 0;
        for mut waypoint in loaded {
            let duplicate = self.get(waypoint.id).is_some();
            let is_death = waypoint.source == WaypointSource::Death;
            let refused = if is_death { self.death_point().is_some() } else { self.waypoints.len() >= self.capacity };
            if duplicate || refused {
                dropped += 1;
                continue;
            }
            waypoint.name = self.clean_name(&waypoint.name);
            self.waypoints.push(waypoint);
        }
        self.next_colour = self.waypoints.len();
        self.revision += 1;
        dropped
    }

    // leaving the world; nothing is marked for saving
    pub fn clear(&mut self) {
        self.waypoints.clear();
        self.next_colour = 0;
        self.revision += 1;
        self.dirty = false;
    }

    // true once after each change, until the next change
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    fn touch(&mut self) {
        self.revision += 1;
        self.dirty = true;
    }

    // splitmix64 over the seed; never 0 and never an id in use
    fn next_id(&mut self) -> WaypointId {
        loop {
            self.seed = self.seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.seed;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            let id = WaypointId(z ^ (z >> 31));
            if id.0 != 0 && self.get(id).is_none() {
                return id;
            }
        }
    }

    // trimmed and cut to MAX_NAME_CHARS; blank becomes the first free "Waypoint N"
    fn clean_name(&self, name: &str) -> String {
        let trimmed: String = name.trim().chars().take(Self::MAX_NAME_CHARS).collect();
        if !trimmed.is_empty() {
            return trimmed;
        }
        (1..=self.waypoints.len() + 1)
            .map(|n| format!("Waypoint {n}"))
            .find(|candidate| !self.waypoints.iter().any(|w| &w.name == candidate))
            .unwrap_or_else(|| "Waypoint".to_string())
    }
}
