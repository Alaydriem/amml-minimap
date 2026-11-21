use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use amml_api::{KeyCode, Modifiers};
use egui::{Event, Key, Modifiers as EguiModifiers, RawInput};
use minimap_core::config::KeysConfig;
use minimap_core::structs::input::{KeyPhase, KeybindAction};

use crate::services::BindMatcher;

// the keys the world map needs that the manager cannot route to it: while the map holds the loader input lease, the
// lease keeps the real keys from the game and from the manager's router, so the mod's `KeyInput` listener hands them
// here. Escape and Ctrl+Z go to the next pass of the Overlay layer as egui key events; while a minimap text field has
// the keyboard the editing keys go there too (the typed text still comes through the manager's `WindowChar` route,
// which the lease does not block), and no bind is matched; otherwise minimap's own binds are matched from the
// configured chords and queued for the next tick. Only keys the lease blocked count, and only while the session has
// armed the feed (the world map open, the manager panel closed). The listener may run on the input thread and the
// pass on the render thread: the counts are plain counters (Relaxed), the flags Relaxed, the rest behind locks
pub struct KeyFeed {
    armed: AtomicBool,
    // a minimap text field had the keyboard in the last Overlay pass
    typing: AtomicBool,
    escape: AtomicU32,
    undo: AtomicU32,
    paste: AtomicU32,
    // editing key events for the next Overlay pass, at most EDIT_CAPACITY
    edits: Mutex<VecDeque<Event>>,
    binds: Mutex<BindMatcher>,
}

impl KeyFeed {
    // presses of one key kept for one pass; more are dropped
    const MAX_PRESSES: u32 = 4;
    // editing events kept for one pass (a press and a release each); more are dropped
    const EDIT_CAPACITY: usize = 64;

    // a feed with the default chords; `set_keys` gives it the configured ones
    pub fn new() -> Self {
        Self {
            armed: AtomicBool::new(false),
            typing: AtomicBool::new(false),
            escape: AtomicU32::new(0),
            undo: AtomicU32::new(0),
            paste: AtomicU32::new(0),
            edits: Mutex::new(VecDeque::with_capacity(Self::EDIT_CAPACITY)),
            binds: Mutex::new(BindMatcher::new(&KeysConfig::default())),
        }
    }

    pub fn set_keys(&self, keys: &KeysConfig) {
        self.binds().set_keys(keys);
    }

    // once per tick; disarming drops the presses still waiting and releases the held binds
    pub fn set_armed(&self, armed: bool) {
        let was = self.armed.swap(armed, Ordering::Relaxed);
        if !armed {
            self.typing.store(false, Ordering::Relaxed);
            self.escape.store(0, Ordering::Relaxed);
            self.undo.store(0, Ordering::Relaxed);
            self.paste.store(0, Ordering::Relaxed);
            self.edits().clear();
            if was {
                self.binds().release_all();
            }
        }
    }

    pub fn armed(&self) -> bool {
        self.armed.load(Ordering::Relaxed)
    }

    // after each Overlay pass: whether a minimap text field has the keyboard
    pub fn set_typing(&self, typing: bool) {
        self.typing.store(typing, Ordering::Relaxed);
    }

    // one real key event; `modifiers` must carry F3; `blocked`: a lease (or an earlier listener) kept the key from the
    // game, so neither the game nor the manager acts on it. True when the feed took it. Only blocked presses are taken.
    // Escape, Ctrl+Z and binds ignore repeats; editing keys repeat as a held key does. An up is taken only as the
    // release of a held bind
    pub fn on_key(&self, key: KeyCode, down: bool, repeat: bool, modifiers: Modifiers, blocked: bool) -> bool {
        if !self.armed() {
            return false;
        }
        if !down {
            return self.binds().on_key(key, down, repeat, modifiers);
        }
        if !blocked {
            return false;
        }
        let generic = key.generic();
        if !repeat {
            let count = match generic {
                KeyCode::ESCAPE => Some(&self.escape),
                // while typing, Ctrl+Z is the field's own text undo (an editing key below)
                KeyCode::Z if modifiers.ctrl() && !self.typing.load(Ordering::Relaxed) => Some(&self.undo),
                _ => None,
            };
            if let Some(count) = count {
                Self::count(count);
                return true;
            }
        }
        if self.typing.load(Ordering::Relaxed) {
            return self.edit(generic, modifiers);
        }
        self.binds().on_key(key, down, repeat, modifiers)
    }

    // the binds matched since the last call, into `out`
    pub fn take_binds(&self, out: &mut Vec<(KeybindAction, KeyPhase)>) {
        self.binds().take(out);
    }

    // the waiting presses as a press and a release each, into the raw input of a pass of the Overlay layer
    pub fn drain_into(&self, raw: &mut RawInput) {
        let undo_modifiers = EguiModifiers { ctrl: true, command: true, ..EguiModifiers::default() };
        for _ in 0..self.escape.swap(0, Ordering::Relaxed) {
            Self::push(raw, Key::Escape, EguiModifiers::default());
        }
        for _ in 0..self.undo.swap(0, Ordering::Relaxed) {
            Self::push(raw, Key::Z, undo_modifiers);
        }
        raw.events.extend(self.edits().drain(..));
        for _ in 0..self.paste.swap(0, Ordering::Relaxed) {
            match clipboard_win::get_clipboard_string() {
                Ok(text) => raw.events.push(Event::Paste(text)),
                Err(error) => curia::warn!("clipboard not read for a paste", { error: error.to_string() }),
            }
        }
    }

    // an editing key of a focused text field; true when it is one
    fn edit(&self, key: KeyCode, modifiers: Modifiers) -> bool {
        let egui_modifiers = EguiModifiers {
            ctrl: modifiers.ctrl(),
            command: modifiers.ctrl(),
            shift: modifiers.shift(),
            alt: modifiers.alt(),
            ..EguiModifiers::default()
        };
        let mut edits = self.edits();
        if edits.len() + 2 > Self::EDIT_CAPACITY {
            return true;
        }
        let event_key = match key.0 {
            0x43 if modifiers.ctrl() => {
                edits.push_back(Event::Copy);
                return true;
            }
            0x58 if modifiers.ctrl() => {
                edits.push_back(Event::Cut);
                return true;
            }
            0x56 if modifiers.ctrl() => {
                Self::count(&self.paste);
                return true;
            }
            0x41 if modifiers.ctrl() => Key::A,
            0x5A if modifiers.ctrl() => Key::Z,
            0x08 => Key::Backspace,
            0x09 => Key::Tab,
            0x0D => Key::Enter,
            0x23 => Key::End,
            0x24 => Key::Home,
            0x25 => Key::ArrowLeft,
            0x26 => Key::ArrowUp,
            0x27 => Key::ArrowRight,
            0x28 => Key::ArrowDown,
            0x2E => Key::Delete,
            _ => return false,
        };
        for pressed in [true, false] {
            edits.push_back(Event::Key {
                key: event_key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui_modifiers,
            });
        }
        true
    }

    fn count(count: &AtomicU32) {
        let _ = count.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |presses| {
            (presses < Self::MAX_PRESSES).then_some(presses + 1)
        });
    }

    fn edits(&self) -> MutexGuard<'_, VecDeque<Event>> {
        self.edits.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn push(raw: &mut RawInput, key: Key, modifiers: EguiModifiers) {
        for pressed in [true, false] {
            raw.events.push(Event::Key { key, physical_key: None, pressed, repeat: false, modifiers });
        }
    }

    fn binds(&self) -> MutexGuard<'_, BindMatcher> {
        self.binds.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for KeyFeed {
    fn default() -> Self {
        Self::new()
    }
}
