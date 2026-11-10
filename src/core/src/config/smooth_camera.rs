use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct SmoothCameraConfig {
    pub spring_strength: f32,
    pub damping: f32,
    pub max_delta_time: f32,
}

impl Default for SmoothCameraConfig {
    fn default() -> Self {
        Self { spring_strength: 36.0, damping: 7.0, max_delta_time: 0.1 }
    }
}
