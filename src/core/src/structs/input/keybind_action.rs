// what a manager keybind asks for; the order is the order of the keybind blocks in descriptor.hcl,
// which is the index the manager sends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeybindAction {
    ToggleWorldMap,
    ToggleMinimap,
    ZoomIn,
    ZoomOut,
    AddWaypoint,
    ToggleCave,
    ToggleCoordinates,
}

impl KeybindAction {
    pub const ALL: [KeybindAction; 7] = [
        Self::ToggleWorldMap,
        Self::ToggleMinimap,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::AddWaypoint,
        Self::ToggleCave,
        Self::ToggleCoordinates,
    ];

    pub fn from_index(index: u16) -> Option<KeybindAction> {
        Self::ALL.get(usize::from(index)).copied()
    }

    // the action label of the keybind block in descriptor.hcl
    pub fn id(self) -> &'static str {
        match self {
            Self::ToggleWorldMap => "toggle_world_map",
            Self::ToggleMinimap => "toggle_minimap",
            Self::ZoomIn => "zoom_in",
            Self::ZoomOut => "zoom_out",
            Self::AddWaypoint => "add_waypoint",
            Self::ToggleCave => "toggle_cave",
            Self::ToggleCoordinates => "toggle_coordinates",
        }
    }
}
