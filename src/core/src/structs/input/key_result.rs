// what a keybind changed that the game side acts on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyResult {
    // the minimap radius changed and is saved as a setting
    pub radius_changed: bool,
    // add a waypoint at the player
    pub add_waypoint: bool,
    // switch between the surface and the cave view
    pub toggle_cave: bool,
    // the coordinates were shown or hidden and are saved as a setting
    pub coordinates_changed: bool,
}
