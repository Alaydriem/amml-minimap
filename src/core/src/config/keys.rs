use serde::Deserialize;

// the chord text of each keybind as the manager saved it; the manager matches the keys, the map shows the text
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct KeysConfig {
    pub toggle_world_map: String,
    pub toggle_minimap: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub add_waypoint: String,
    pub toggle_cave: String,
    pub toggle_coordinates: String,
}

impl Default for KeysConfig {
    fn default() -> Self {
        Self {
            toggle_world_map: "M".to_string(),
            toggle_minimap: "F3+M".to_string(),
            zoom_in: "Equals".to_string(),
            zoom_out: "Minus".to_string(),
            add_waypoint: "N".to_string(),
            toggle_cave: "F3+K".to_string(),
            toggle_coordinates: "Ctrl+K".to_string(),
        }
    }
}
