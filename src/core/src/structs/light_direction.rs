#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightDirection {
    azimuth_rad: f32,
    zenith_rad: f32,
}

impl LightDirection {
    pub fn new(azimuth_degrees: f32, zenith_degrees: f32) -> Self {
        let deg2rad = std::f32::consts::PI / 180.0;
        Self { azimuth_rad: azimuth_degrees * deg2rad, zenith_rad: zenith_degrees * deg2rad }
    }

    // horizontal direction shadows fall toward; zero components below 1e-5 are snapped to 0
    pub fn downstream(&self) -> (f32, f32) {
        let snap = |v: f32| if v.abs() < 1e-5 { 0.0 } else { v };
        (snap(-self.azimuth_rad.sin()), snap(self.azimuth_rad.cos()))
    }

    pub fn tan_zenith(&self) -> f32 {
        self.zenith_rad.tan()
    }

    // vector from the ground toward the light: (x, y, z)
    pub fn sun(&self) -> (f32, f32, f32) {
        (
            self.azimuth_rad.sin() * self.zenith_rad.cos(),
            self.zenith_rad.sin(),
            -self.azimuth_rad.cos() * self.zenith_rad.cos(),
        )
    }
}
