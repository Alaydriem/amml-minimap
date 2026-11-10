use crate::services::ui::MapUi;
use crate::structs::ui::{HintKeys, HoverInfo};
use crate::structs::view::ViewState;
use crate::structs::waypoint::Waypoint;

// what the world map panels read in one frame; they never change it, they return MapActions
#[derive(Clone, Copy)]
pub struct UiFrame<'a> {
    // the world map is open with its panels; false shows only the message line, as on the minimap
    pub panels: bool,
    pub ui: &'a MapUi,
    pub view: &'a ViewState,
    pub waypoints: &'a [Waypoint],
    // indices into `waypoints` the list shows, after the search
    pub rows: &'a [usize],
    // waypoints in the player's dimension, before the search
    pub in_dimension: usize,
    pub player_block: (i32, i32, i32),
    pub dim: i32,
    pub hover: Option<HoverInfo<'a>>,
    pub keys: HintKeys<'a>,
}
