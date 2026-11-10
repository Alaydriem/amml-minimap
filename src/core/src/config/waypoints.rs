use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct WaypointsConfig {
    pub enabled: bool,
    pub max_per_world: i32,
    pub show_on_minimap: bool,
    pub max_edge_arrows: i32,
    // waypoint names are drawn at fontSize times this
    pub label_scale: f32,
}

impl Default for WaypointsConfig {
    fn default() -> Self {
        Self { enabled: true, max_per_world: 512, show_on_minimap: true, max_edge_arrows: 16, label_scale: 1.0 }
    }
}
