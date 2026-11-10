// the smoothed camera: block position and unwrapped yaw in degrees
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CameraPose {
    pub x: f32,
    pub z: f32,
    pub yaw: f32,
}

impl CameraPose {
    // map rotation: yaw + 180 degrees, in radians
    pub fn render_yaw_rad(&self) -> f32 {
        (self.yaw + 180.0).to_radians()
    }
}
