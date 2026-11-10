use crate::config::{CaveModeConfig, Config, MiniMapConfig, TerrainConfig, WorldMapConfig};

// the rules the manager's per-field schema cannot hold: one value bounded by another, and the size of the
// name-to-colour maps, which are not schema fields
pub struct ConfigClamp;

impl ConfigClamp {
    pub fn apply(config: Config) -> Config {
        Config {
            mini_map: Self::mini_map(config.mini_map),
            world_map: Self::world_map(config.world_map),
            cave_mode: Self::cave_mode(config.cave_mode),
            terrain: Self::terrain(config.terrain),
            ..config
        }
    }

    fn mini_map(c: MiniMapConfig) -> MiniMapConfig {
        let max_radius = c.max_radius_chunks.max(c.min_radius_chunks);
        MiniMapConfig {
            max_radius_chunks: max_radius,
            radius_chunks: c.radius_chunks.clamp(c.min_radius_chunks, max_radius),
            ..c
        }
    }

    fn world_map(c: WorldMapConfig) -> WorldMapConfig {
        WorldMapConfig { max_zoom: c.max_zoom.max(c.min_zoom), ..c }
    }

    fn cave_mode(c: CaveModeConfig) -> CaveModeConfig {
        CaveModeConfig { band_margin: c.band_margin.clamp(0, (c.band_size / 2 - 1).max(0)), ..c }
    }

    fn terrain(mut c: TerrainConfig) -> TerrainConfig {
        c.block_colors.remove("");
        c.biome_color_overrides.remove("");
        if c.block_colors.len() > TerrainConfig::MAX_BLOCK_COLORS {
            curia::warn!("terrain.block_colors has too many entries; the first are used", {
                entries: c.block_colors.len(),
                used: TerrainConfig::MAX_BLOCK_COLORS,
            });
            c.block_colors = c.block_colors.into_iter().take(TerrainConfig::MAX_BLOCK_COLORS).collect();
        }
        if c.biome_color_overrides.len() > TerrainConfig::MAX_BIOME_COLOR_OVERRIDES {
            curia::warn!("terrain.biome_color_overrides has too many entries; the first are used", {
                entries: c.biome_color_overrides.len(),
                used: TerrainConfig::MAX_BIOME_COLOR_OVERRIDES,
            });
            c.biome_color_overrides =
                c.biome_color_overrides.into_iter().take(TerrainConfig::MAX_BIOME_COLOR_OVERRIDES).collect();
        }
        c
    }
}
