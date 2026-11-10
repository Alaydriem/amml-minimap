use crate::structs::input::{KeyPhase, KeybindAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeybindInput {
    pub action: KeybindAction,
    pub phase: KeyPhase,
    // the game holds the mouse: no chat, menu or other screen is open
    pub mouse_grabbed: bool,
}
