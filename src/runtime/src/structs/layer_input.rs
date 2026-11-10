use minimap_core::structs::draw::MapLayer;

// one manager layer pass: which layer, the screen in physical pixels, its scale and the frame time
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerInput {
    pub layer: MapLayer,
    pub width: u32,
    pub height: u32,
    pub pixels_per_point: f32,
    pub time_secs: f64,
}
