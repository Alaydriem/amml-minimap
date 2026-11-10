use crate::structs::waypoint::WaypointId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogTarget {
    New { dim: i32, x: i32, y: i32, z: i32 },
    Edit(WaypointId),
}
