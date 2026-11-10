use serde::{Deserialize, Serialize};

// the camelCase aliases read the terrain block of captures recorded before the HCL config
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct ShadowConfig {
    #[serde(alias = "renderStyle")]
    pub render_style: i32,
    #[serde(alias = "shadowLevel")]
    pub shadow_level: i32,
    #[serde(alias = "renderScale")]
    pub render_scale: i32,
    #[serde(alias = "pcfRadius")]
    pub pcf_radius: i32,
    #[serde(alias = "lightAzimuth")]
    pub light_azimuth: f32,
    #[serde(alias = "lightZenith")]
    pub light_zenith: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            render_style: 2,
            shadow_level: 130,
            render_scale: 2,
            pcf_radius: 1,
            light_azimuth: 315.0,
            light_zenith: 60.0,
        }
    }
}
