use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct MiniMapConfig {
    pub size_ratio: f32,
    pub radius_chunks: i32,
    pub margin_right: f32,
    pub margin_top: f32,
    pub background_color: u32,
    pub chunk_line_color: u32,
    pub current_chunk_color: u32,
    pub border_color: u32,
    pub player_arrow_outline_color: u32,
    pub player_arrow_fill_color: u32,
    pub coord_text_color: u32,
    pub coord_shadow_color: u32,
    pub line_thickness: f32,
    pub border_thickness: f32,
    pub coord_text_margin: f32,
    pub min_radius_chunks: i32,
    pub max_radius_chunks: i32,
    pub widget_scale: f32,
    pub show_coordinates: bool,
    pub show_nether_coordinates: bool,
    // turn the map so the player's facing is at the top
    pub rotate_with_player: bool,
}

impl Default for MiniMapConfig {
    fn default() -> Self {
        Self {
            size_ratio: 0.15,
            radius_chunks: 5,
            margin_right: 20.0,
            margin_top: 20.0,
            background_color: 0x0000_00FF,
            chunk_line_color: 0x7878_78FF,
            current_chunk_color: 0x3AA0_FFFF,
            border_color: 0x5050_50FF,
            player_arrow_outline_color: 0x0000_00FF,
            player_arrow_fill_color: 0xDC32_32FF,
            coord_text_color: 0xFFFF_FFFF,
            coord_shadow_color: 0x0000_00C8,
            line_thickness: 1.0,
            border_thickness: 2.0,
            coord_text_margin: 8.0,
            min_radius_chunks: 2,
            max_radius_chunks: 16,
            widget_scale: 1.0,
            show_coordinates: true,
            show_nether_coordinates: true,
            rotate_with_player: false,
        }
    }
}
