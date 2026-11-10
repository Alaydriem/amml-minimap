use crate::config::TerrainConfig;
use crate::structs::LightDirection;
use crate::structs::bake::RenderStyle;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BakeSettings {
    pub style: RenderStyle,
    pub shadow_level: i32,
    pub scale: u8,
    pub pcf_radius: usize,
    pub light_azimuth: f32,
    pub light_zenith: f32,
    pub transparent_water: bool,
    pub water_opacity_per_depth: f32,
    pub water_max_opacity: f32,
}

impl BakeSettings {
    pub fn new(terrain: &TerrainConfig) -> Self {
        let shadow = &terrain.shadow;
        Self {
            style: RenderStyle::from_config(shadow.render_style),
            shadow_level: shadow.shadow_level,
            scale: shadow.render_scale.clamp(1, 16) as u8,
            pcf_radius: shadow.pcf_radius.clamp(0, 8) as usize,
            light_azimuth: shadow.light_azimuth,
            light_zenith: shadow.light_zenith,
            transparent_water: terrain.enable_transparent_water,
            water_opacity_per_depth: terrain.water_opacity_per_depth,
            water_max_opacity: terrain.water_max_opacity,
        }
    }

    pub fn light(&self) -> LightDirection {
        LightDirection::new(self.light_azimuth, self.light_zenith)
    }

    // highest block Y a shadow ray may travel to before it stops
    pub fn dimension_top(dim: i32) -> i32 {
        match dim {
            0 => 320,
            1 => 128,
            2 => 256,
            _ => 0,
        }
    }

    // FNV-1a over the fields in a fixed order; stored on disk, so the order is a format contract
    pub fn settings_hash(&self) -> u64 {
        let mut bytes = [0u8; 27];
        bytes[0] = self.style.code();
        bytes[1..5].copy_from_slice(&self.shadow_level.to_le_bytes());
        bytes[5] = self.scale;
        bytes[6] = self.pcf_radius as u8;
        bytes[7..11].copy_from_slice(&self.light_azimuth.to_bits().to_le_bytes());
        bytes[11..15].copy_from_slice(&self.light_zenith.to_bits().to_le_bytes());
        bytes[15] = u8::from(self.transparent_water);
        bytes[16..20].copy_from_slice(&self.water_opacity_per_depth.to_bits().to_le_bytes());
        bytes[20..24].copy_from_slice(&self.water_max_opacity.to_bits().to_le_bytes());
        bytes[24..27].copy_from_slice(b"LM1");
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }
}
