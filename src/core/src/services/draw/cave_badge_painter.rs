use crate::structs::Rgba;
use crate::structs::draw::{DrawCommand, DrawList, TextAlign};

// "CAVE · Y <bottom> to <top>" on a dark band, centred under `top_centre`
pub struct CaveBadgePainter;

impl CaveBadgePainter {
    const BAND: Rgba = Rgba { r: 0, g: 0, b: 0, a: 190 };
    const TEXT: Rgba = Rgba { r: 255, g: 220, b: 120, a: 255 };
    // a rough glyph width in text sizes; core cannot measure the game's font
    const CHAR_WIDTH: f32 = 0.55;
    const PAD: f32 = 4.0;

    pub fn text(bottom: i32, top: i32) -> String {
        format!("CAVE \u{b7} Y {bottom} to {top}")
    }

    pub fn paint(draw: &mut DrawList, top_centre: (f32, f32), size: f32, range: (i32, i32)) {
        let text = Self::text(range.0, range.1);
        let half = text.chars().count() as f32 * size * Self::CHAR_WIDTH * 0.5;
        let (x, y) = top_centre;
        draw.push(DrawCommand::RectFilled {
            p0: (x - half - Self::PAD, y),
            p1: (x + half + Self::PAD, y + size + 2.0 * Self::PAD),
            colour: Self::BAND.packed(),
        });
        draw.text((x, y + Self::PAD), Self::TEXT.packed(), size, TextAlign::Centre, &text);
    }
}
