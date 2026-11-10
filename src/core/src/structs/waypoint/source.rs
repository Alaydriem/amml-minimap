use serde::{Deserialize, Serialize};

// how the waypoint was made
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WaypointSource {
    Manual,
    Hotkey,
    Death,
}
