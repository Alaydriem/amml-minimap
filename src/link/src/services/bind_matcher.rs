use std::collections::VecDeque;

use amml_api::{Chord, KeyCode, Modifiers};
use minimap_core::config::KeysConfig;
use minimap_core::structs::input::{KeyPhase, KeybindAction};

// minimap's own bind matching while the world map holds the input lease: the lease blocks the keys, so the manager
// sends no keybind then. Each real press is matched against the chords of config.hcl (modifiers exactly, F3
// included); a match is queued as the manager's keybind of the same action would be, and its key's up queues the
// release. Repeats are not matched. At most QUEUE_CAPACITY binds wait; more are dropped
pub struct BindMatcher {
    chords: [Option<Chord>; KeybindAction::ALL.len()],
    down: [bool; KeybindAction::ALL.len()],
    queue: VecDeque<(KeybindAction, KeyPhase)>,
}

impl BindMatcher {
    pub const QUEUE_CAPACITY: usize = 16;

    pub fn new(keys: &KeysConfig) -> Self {
        Self {
            chords: Self::parse(keys),
            down: [false; KeybindAction::ALL.len()],
            queue: VecDeque::with_capacity(Self::QUEUE_CAPACITY),
        }
    }

    // a settings change: the held binds go up, then the new chords count
    pub fn set_keys(&mut self, keys: &KeysConfig) {
        self.release_all();
        self.chords = Self::parse(keys);
    }

    // one real key event; true when it pressed or released a bind
    pub fn on_key(&mut self, key: KeyCode, down: bool, repeat: bool, modifiers: Modifiers) -> bool {
        if !down {
            return self.release_key(key);
        }
        if repeat {
            return false;
        }
        let mut matched = false;
        for (index, action) in KeybindAction::ALL.into_iter().enumerate() {
            let Some(chord) = self.chords[index] else { continue };
            if self.down[index] || !chord.matches(key, modifiers) {
                continue;
            }
            self.down[index] = true;
            self.push(action, KeyPhase::Pressed);
            matched = true;
        }
        matched
    }

    pub fn release_all(&mut self) {
        for index in 0..self.down.len() {
            if self.down[index] {
                self.release(index);
            }
        }
    }

    // the binds queued since the last call, into `out`
    pub fn take(&mut self, out: &mut Vec<(KeybindAction, KeyPhase)>) {
        out.extend(self.queue.drain(..));
    }

    fn release_key(&mut self, key: KeyCode) -> bool {
        let mut released = false;
        for index in 0..self.down.len() {
            let same_key = self.chords[index].is_some_and(|chord| chord.key == key || chord.key == key.generic());
            if self.down[index] && same_key {
                self.release(index);
                released = true;
            }
        }
        released
    }

    fn release(&mut self, index: usize) {
        self.down[index] = false;
        self.push(KeybindAction::ALL[index], KeyPhase::Released);
    }

    fn push(&mut self, action: KeybindAction, phase: KeyPhase) {
        if self.queue.len() < Self::QUEUE_CAPACITY {
            self.queue.push_back((action, phase));
        }
    }

    fn parse(keys: &KeysConfig) -> [Option<Chord>; KeybindAction::ALL.len()] {
        KeybindAction::ALL.map(|action| {
            let text = Self::text(keys, action);
            match Chord::parse(text) {
                Ok(chord) => Some(chord),
                Err(error) => {
                    curia::warn!("a bind chord does not parse; the bind is off while the world map is open", {
                        bind: action.id(),
                        chord: text,
                        error: error.to_string(),
                    });
                    None
                }
            }
        })
    }

    fn text(keys: &KeysConfig, action: KeybindAction) -> &str {
        match action {
            KeybindAction::ToggleWorldMap => &keys.toggle_world_map,
            KeybindAction::ToggleMinimap => &keys.toggle_minimap,
            KeybindAction::ZoomIn => &keys.zoom_in,
            KeybindAction::ZoomOut => &keys.zoom_out,
            KeybindAction::AddWaypoint => &keys.add_waypoint,
            KeybindAction::ToggleCave => &keys.toggle_cave,
            KeybindAction::ToggleCoordinates => &keys.toggle_coordinates,
        }
    }
}
