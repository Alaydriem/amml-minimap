// the fields the dialog can change
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointEdit {
    pub name: String,
    pub colour: [u8; 4],
    pub show_on_edge: bool,
    pub show_distance: bool,
}
