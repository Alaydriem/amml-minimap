#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PlayerSnapshot {
    pub present: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    // degrees, as the game reports it
    pub yaw: f32,
    pub dim: i32,
}

impl PlayerSnapshot {
    pub fn block(&self) -> (i32, i32, i32) {
        (self.x.floor() as i32, self.y.floor() as i32, self.z.floor() as i32)
    }
}
