use crate::config::Config;
use crate::services::cave::LayerIds;
use crate::structs::layer::MapLayer;

// picks the layer to show: the surface, or in cave mode the band around the feet, which changes
// only when the feet are more than `margin` blocks outside it
pub struct LayerSelector {
    cave: bool,
    band: Option<i32>,
    band_size: i32,
    margin: i32,
    surface_scan_chunks: i32,
    cave_scan_chunks: i32,
}

impl LayerSelector {
    pub fn new(config: &Config) -> Self {
        let cave = &config.cave_mode;
        Self {
            cave: false,
            band: None,
            band_size: cave.band_size.max(1),
            margin: cave.band_margin.max(0),
            surface_scan_chunks: config.terrain.scan_radius / 16,
            cave_scan_chunks: (cave.scan_radius / 16).max(1),
        }
    }

    pub fn is_cave(&self) -> bool {
        self.cave
    }

    // returns whether cave mode is now on
    pub fn toggle(&mut self) -> bool {
        self.cave = !self.cave;
        self.band = None;
        self.cave
    }

    // a new world starts on the surface
    pub fn reset(&mut self) {
        self.cave = false;
        self.band = None;
    }

    pub fn select(&mut self, dim: i32, feet_y: i32) -> MapLayer {
        if !self.cave {
            return MapLayer {
                id: LayerIds::surface(dim),
                dim,
                band: None,
                camera_height: Self::surface_height(dim),
                scan_radius_chunks: self.surface_scan_chunks,
            };
        }
        let band = match self.band {
            Some(band)
                if feet_y >= band * self.band_size - self.margin
                    && feet_y < (band + 1) * self.band_size + self.margin =>
            {
                band
            }
            _ => feet_y.div_euclid(self.band_size),
        };
        self.band = Some(band);
        MapLayer {
            id: LayerIds::cave(dim, band),
            dim,
            band: Some(band),
            camera_height: self.band_top(band),
            scan_radius_chunks: self.cave_scan_chunks,
        }
    }

    pub fn band_bottom(&self, band: i32) -> i32 {
        band * self.band_size
    }

    pub fn band_top(&self, band: i32) -> i32 {
        (band + 1) * self.band_size - 1
    }

    // the surface probe starts here: overworld 320, nether 127 (bedrock ceiling), end 256
    pub fn surface_height(dim: i32) -> i32 {
        match dim {
            1 => 127,
            2 => 256,
            _ => 320,
        }
    }
}
