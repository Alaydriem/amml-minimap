// the two view states the manager leases follow: the corner map on the Hud layer, the world map on Overlay and Input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewFlags {
    pub world_map_open: bool,
    pub show_minimap: bool,
}
