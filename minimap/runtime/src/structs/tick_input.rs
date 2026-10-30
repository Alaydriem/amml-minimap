use minimap_core::structs::view::PlayerSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TickInput {
    pub player: PlayerSnapshot,
    pub foreground: bool,
    // the game has hidden the system cursor (it holds the mouse for camera control)
    pub cursor_hidden: bool,
    // the local player is dead: health 0, the death screen is up
    pub dead: bool,
    // the game holds the mouse: no chat, menu or other screen is open
    pub mouse_grabbed: bool,
    // process memory, sampled by the host
    pub working_set_bytes: u64,
    pub private_bytes: u64,
}
