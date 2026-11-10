// turns screen points around a pivot: screen = pivot + R(angle) * (point - pivot); angle 0 is north-up
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapRotation {
    angle: f32,
    sin: f32,
    cos: f32,
    pivot: (f32, f32),
}

impl MapRotation {
    pub fn new(angle: f32, pivot: (f32, f32)) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self { angle, sin, cos, pivot }
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }

    pub fn is_turned(&self) -> bool {
        self.angle != 0.0
    }

    pub fn apply(&self, point: (f32, f32)) -> (f32, f32) {
        let (x, y) = (point.0 - self.pivot.0, point.1 - self.pivot.1);
        (self.pivot.0 + x * self.cos - y * self.sin, self.pivot.1 + x * self.sin + y * self.cos)
    }

    // a screen movement, turned back into the map's north-up frame
    pub fn invert_vector(&self, v: (f32, f32)) -> (f32, f32) {
        (v.0 * self.cos + v.1 * self.sin, -v.0 * self.sin + v.1 * self.cos)
    }

    // the map turn that puts the player's facing at the top of the screen
    pub fn player_up(render_yaw_rad: f32) -> f32 {
        -render_yaw_rad
    }
}
