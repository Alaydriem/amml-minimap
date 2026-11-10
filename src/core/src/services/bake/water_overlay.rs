use crate::services::bake::ColourMath;
use crate::structs::Rgba;
use crate::structs::chunk::ChunkSurface;

// runs after shading, so water is not darkened by terrain shadow
pub struct WaterOverlay;

impl WaterOverlay {
    pub fn apply(center: &ChunkSurface, per_depth: f32, max_opacity: f32, colours: &mut [Rgba; 256]) {
        for (i, colour) in colours.iter_mut().enumerate() {
            let depth = center.water_depth(i);
            if depth == 0 {
                continue;
            }
            let opacity = (per_depth * f32::from(depth)).clamp(0.0, max_opacity);
            *colour = ColourMath::blend(*colour, center.water_colour(i), opacity);
        }
    }
}
