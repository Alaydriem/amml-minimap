// how far a waypoint is from the player, across the ground
pub struct WaypointDistance;

impl WaypointDistance {
    // at or under this many blocks the list says "Here"
    pub const HERE_BLOCKS: u32 = 3;

    // the horizontal distance in whole blocks, rounded; height is left out
    pub fn blocks(from: (i32, i32, i32), to: (i32, i32, i32)) -> u32 {
        let dx = f64::from(to.0) - f64::from(from.0);
        let dz = f64::from(to.2) - f64::from(from.2);
        dx.hypot(dz).round() as u32
    }

    pub fn is_here(blocks: u32) -> bool {
        blocks <= Self::HERE_BLOCKS
    }
}
