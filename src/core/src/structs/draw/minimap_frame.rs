use crate::structs::view::CameraPose;

// everything the minimap painter needs for one frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapFrame {
    pub display_width: f32,
    pub camera: CameraPose,
    pub player_block: (i32, i32, i32),
    pub radius_chunks: i32,
    pub font_size: f32,
    // None until the minimap texture has been uploaded once
    pub texture: Option<u32>,
    // world block at the top-left texel of the minimap texture, and its side in texels
    pub image_origin: (i32, i32),
    pub image_side: usize,
    pub dim: i32,
    pub show_coordinates: bool,
    // map turn in radians; 0 is north-up
    pub angle: f32,
}
