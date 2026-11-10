use crate::structs::ui::DialogTarget;

// the waypoint dialog's state; the position is shown, not edited. The name being typed lives in
// the UI, which starts from `initial_name` each time `serial` changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointDialog {
    pub target: DialogTarget,
    pub initial_name: String,
    pub serial: u64,
    pub colour: [u8; 4],
    pub show_on_edge: bool,
    pub show_distance: bool,
    pub position: (i32, i32, i32),
}
