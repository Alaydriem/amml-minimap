// what the player is looking at; shared by input handling and drawing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewState {
    pub show_minimap: bool,
    pub world_map_open: bool,
    // world map centre relative to the camera, in blocks
    pub offset_x: f32,
    pub offset_z: f32,
    // world map pixels per block
    pub zoom: f32,
    pub minimap_radius: i32,
    // coordinates under the minimap and along the world map's bottom; Ctrl+K switches them
    pub show_coordinates: bool,
    // world map overlays, switched from the tools block and saved to config
    pub show_chunk_grid: bool,
    pub show_slime_chunks: bool,
    // a fixed world map centre in blocks, set by picking a waypoint; it wins over the offset
    pub anchor: Option<(f64, f64)>,
}

impl ViewState {
    pub fn new(minimap_radius: i32) -> Self {
        Self {
            show_minimap: true,
            world_map_open: false,
            offset_x: 0.0,
            offset_z: 0.0,
            zoom: 1.0,
            minimap_radius,
            show_coordinates: true,
            show_chunk_grid: false,
            show_slime_chunks: false,
            anchor: None,
        }
    }

    // the world map centre: the anchor when set, else the camera plus the offset
    pub fn centre(&self, camera: (f32, f32)) -> (f64, f64) {
        self.anchor
            .unwrap_or((f64::from(camera.0) + f64::from(self.offset_x), f64::from(camera.1) + f64::from(self.offset_z)))
    }
}
