use crate::structs::draw::{SamplerKind, TextAlign};

// one map shape in physical pixels; colours are packed like IM_COL32 (r in the low byte, a in the high byte)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawCommand {
    RectFilled { p0: (f32, f32), p1: (f32, f32), colour: u32 },
    Line { a: (f32, f32), b: (f32, f32), colour: u32, thickness: f32 },
    TriangleFilled { a: (f32, f32), b: (f32, f32), c: (f32, f32), colour: u32 },
    CircleFilled { centre: (f32, f32), radius: f32, colour: u32 },
    Circle { centre: (f32, f32), radius: f32, colour: u32, thickness: f32 },
    // the text bytes are text[start..start + len] of the owning DrawList
    Text { pos: (f32, f32), colour: u32, size: f32, align: TextAlign, start: u32, len: u32 },
    Image { tex: u32, p0: (f32, f32), p1: (f32, f32), uv0: (f32, f32), uv1: (f32, f32) },
    // an axis-aligned image turned by `angle` radians around `centre`; half is its half size
    ImageRotated { tex: u32, centre: (f32, f32), half: (f32, f32), uv0: (f32, f32), uv1: (f32, f32), angle: f32 },
    // a disc of the texture: screen circle (centre, radius) shows the texture circle (uv_centre, uv_radius)
    // turned by `angle` radians
    ImageDisc { tex: u32, centre: (f32, f32), radius: f32, uv_centre: (f32, f32), uv_radius: f32, angle: f32 },
    Sampler(SamplerKind),
}
