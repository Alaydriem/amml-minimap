use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct CaveModeConfig {
    // RGBA of columns with no open space under the band top
    pub solid_color: [u8; 4],
    // blocks per band: 8, 16 or 32
    pub band_size: i32,
    // blocks the feet may go past a band's edge before the next band is shown
    pub band_margin: i32,
    // blocks scanned around the player in the cave view
    pub scan_radius: i32,
}

impl Default for CaveModeConfig {
    fn default() -> Self {
        Self { solid_color: [96, 96, 96, 255], band_size: 16, band_margin: 2, scan_radius: 64 }
    }
}
