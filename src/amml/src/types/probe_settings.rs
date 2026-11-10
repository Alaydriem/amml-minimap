// the scan plan's settings the column walk needs; the block listener reads them between ticks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeSettings {
    pub camera_height: i32,
    pub transparent_water: bool,
}

impl ProbeSettings {
    pub fn new(camera_height: i32, transparent_water: bool) -> Self {
        Self { camera_height, transparent_water }
    }
}

impl Default for ProbeSettings {
    fn default() -> Self {
        Self::new(320, true)
    }
}
