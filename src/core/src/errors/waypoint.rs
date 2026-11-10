#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WaypointError {
    #[error("the world already has {0} waypoints, the most allowed")]
    Full(usize),
}
