use crate::services::bake::{Bevel, BevelTable, GradientStyle, PcfBlur, ShadowCaster, WaterOverlay};
use crate::structs::Rgba;
use crate::structs::bake::{BakeOutput, BakeSettings, RenderStyle};
use crate::structs::chunk::{ChunkSurface, ShadowMask};
use crate::structs::pos::ChunkPos;
use crate::traits::BakeEnvironment;

// bakes one chunk: shading for the configured style, then the water overlay
pub struct ChunkBaker {
    pcf: PcfBlur,
    bevel: Option<BevelTable>,
    colours: [Rgba; 256],
}

impl ChunkBaker {
    pub fn new() -> Self {
        Self { pcf: PcfBlur::new(), bevel: None, colours: [Rgba::TRANSPARENT; 256] }
    }

    pub fn bake(
        &mut self,
        pos: ChunkPos,
        center: &ChunkSurface,
        env: &impl BakeEnvironment,
        settings: &BakeSettings,
        out: &mut BakeOutput,
    ) {
        if settings.style == RenderStyle::ShadowMap {
            ShadowCaster::cast(pos, center, env, settings.light(), settings.scale, &mut out.mask);
        } else {
            out.mask.reset(0);
        }
        self.render(pos, center, &out.mask, env, settings, &mut out.pixels);
    }

    // redoes PCF and bevel from a stored mask; a mask at another scale needs the full bake
    pub fn bake_soft(
        &mut self,
        pos: ChunkPos,
        center: &ChunkSurface,
        stored: &ShadowMask,
        env: &impl BakeEnvironment,
        settings: &BakeSettings,
        out: &mut BakeOutput,
    ) {
        if settings.style != RenderStyle::ShadowMap || !stored.is_baked_at(settings.scale) {
            self.bake(pos, center, env, settings, out);
            return;
        }
        out.mask.copy_from(stored);
        self.render(pos, center, &out.mask, env, settings, &mut out.pixels);
    }

    // shades with an already cast mask and writes packed RGBA pixels
    pub fn render(
        &mut self,
        pos: ChunkPos,
        center: &ChunkSurface,
        mask: &ShadowMask,
        env: &impl BakeEnvironment,
        settings: &BakeSettings,
        pixels: &mut [u32; 256],
    ) {
        for (i, colour) in self.colours.iter_mut().enumerate() {
            *colour = center.surface_colour(i);
        }
        match settings.style {
            RenderStyle::Flat => {}
            RenderStyle::Gradient => GradientStyle::apply(pos, center, env, settings.shadow_level, &mut self.colours),
            RenderStyle::ShadowMap => {
                self.pcf.apply(pos, mask, env, settings.pcf_radius, &mut self.colours);
                let table = match self.bevel.take() {
                    Some(table) if table.scale() == settings.scale => table,
                    _ => BevelTable::new(settings.scale),
                };
                Bevel::apply(pos, center, env, &table, &mut self.colours);
                self.bevel = Some(table);
            }
        }
        if settings.transparent_water {
            WaterOverlay::apply(
                center,
                settings.water_opacity_per_depth,
                settings.water_max_opacity,
                &mut self.colours,
            );
        }
        for (pixel, colour) in pixels.iter_mut().zip(self.colours.iter()) {
            *pixel = colour.packed();
        }
    }
}

impl Default for ChunkBaker {
    fn default() -> Self {
        Self::new()
    }
}
