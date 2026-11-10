use minimap_core::structs::draw::{DrawCommand, DrawList, WorldMapFrame};
use minimap_core::structs::view::CameraPose;

// world map frames and readers of what a painter drew
pub struct TestFrames;

impl TestFrames {
    pub fn world_frame(
        centre: (f64, f64),
        zoom: f32,
        display: (f32, f32),
        angle: f32,
        dim: i32,
    ) -> WorldMapFrame<'static> {
        WorldMapFrame {
            display,
            centre,
            zoom,
            camera: CameraPose { x: 0.0, z: 0.0, yaw: 0.0 },
            player_block: (0, 64, 0),
            font_size: 18.0,
            show_coordinates: false,
            show_nether_coordinates: false,
            dim,
            angle,
            regions: &[],
        }
    }

    pub fn lines(draw: &DrawList) -> usize {
        draw.commands.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count()
    }

    pub fn rects(draw: &DrawList) -> Vec<((f32, f32), (f32, f32))> {
        draw.commands
            .iter()
            .filter_map(|c| match c {
                DrawCommand::RectFilled { p0, p1, .. } => Some((*p0, *p1)),
                _ => None,
            })
            .collect()
    }

    // every text the list draws, with its colour
    pub fn texts(draw: &DrawList) -> Vec<(String, u32)> {
        draw.commands
            .iter()
            .filter_map(|c| match c {
                DrawCommand::Text { start, len, colour, .. } => Some((draw.text_of(*start, *len).to_string(), *colour)),
                _ => None,
            })
            .collect()
    }

    pub fn draws(draw: &DrawList, text: &str) -> bool {
        Self::texts(draw).iter().any(|(t, _)| t == text)
    }
}
