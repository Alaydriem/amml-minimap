pub mod edit;
pub mod icon;
pub mod id;
pub mod source;

pub use edit::WaypointEdit;
pub use icon::WaypointIcon;
pub use id::WaypointId;
pub use source::WaypointSource;

use serde::{Deserialize, Serialize};

// a named place in one dimension of the current world; also the waypoints.json entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Waypoint {
    pub id: WaypointId,
    pub name: String,
    pub dim: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    // RGBA
    pub colour: [u8; 4],
    pub icon: WaypointIcon,
    pub show_on_edge: bool,
    pub show_distance: bool,
    pub source: WaypointSource,
    // unix seconds
    pub created: u64,
}
