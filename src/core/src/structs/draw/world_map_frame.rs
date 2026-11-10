use crate::structs::pos::RegionPos;
use crate::structs::view::CameraPose;

// everything the world map painter needs for one frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldMapFrame<'a> {
    pub display: (f32, f32),
    // world block at the screen centre
    pub centre: (f64, f64),
    pub zoom: f32,
    pub camera: CameraPose,
    pub player_block: (i32, i32, i32),
    pub font_size: f32,
    pub show_coordinates: bool,
    pub show_nether_coordinates: bool,
    pub dim: i32,
    // map turn in radians around the screen centre; 0 is north-up
    pub angle: f32,
    // each visible region and the texture to draw for it, if one is resident
    pub regions: &'a [(RegionPos, Option<u32>)],
}
