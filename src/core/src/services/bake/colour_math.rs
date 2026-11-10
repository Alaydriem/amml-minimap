use crate::structs::Rgba;

// 8-bit channel maths with truncating float-to-int conversion, then a clamp to 0..=255
pub struct ColourMath;

impl ColourMath {
    pub fn multiply(c: Rgba, factor: f32) -> Rgba {
        let channel = |v: u8| ((f32::from(v) * factor) as i32).clamp(0, 255) as u8;
        Rgba::new(channel(c.r), channel(c.g), channel(c.b), c.a)
    }

    // (1 - opacity) * dst + opacity * src; alpha comes from dst
    pub fn blend(dst: Rgba, src: Rgba, opacity: f32) -> Rgba {
        let inv = 1.0 - opacity;
        let channel = |d: u8, s: u8| ((f32::from(d) * inv + f32::from(s) * opacity) as i32).clamp(0, 255) as u8;
        Rgba::new(channel(dst.r, src.r), channel(dst.g, src.g), channel(dst.b, src.b), dst.a)
    }

    // integer channel * numerator / denominator
    pub fn scale_ratio(c: Rgba, numerator: i32, denominator: i32) -> Rgba {
        let channel = |v: u8| (i32::from(v) * numerator / denominator).clamp(0, 255) as u8;
        Rgba::new(channel(c.r), channel(c.g), channel(c.b), c.a)
    }
}
