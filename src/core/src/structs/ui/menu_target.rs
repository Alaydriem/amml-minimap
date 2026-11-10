use crate::structs::waypoint::WaypointId;

// what was under the cursor when the menu opened
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTarget {
    // y is the stored map height of the column, or the default when it is not explored
    Ground { dim: i32, x: i32, y: i32, z: i32 },
    Marker(WaypointId),
}
