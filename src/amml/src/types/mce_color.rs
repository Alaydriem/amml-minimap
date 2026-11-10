use minimap_core::structs::Rgba;

// the game's mce::Color: four floats from 0 to 1. A member function returns it through a hidden pointer
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MceColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl MceColor {
    // the colour times `tint`, channel by channel; the alpha stays this colour's
    pub fn tinted(self, tint: MceColor) -> Self {
        Self { r: self.r * tint.r, g: self.g * tint.g, b: self.b * tint.b, a: self.a }
    }

    pub fn to_rgba(self) -> Rgba {
        Rgba::new(Self::channel(self.r), Self::channel(self.g), Self::channel(self.b), Self::channel(self.a))
    }

    fn channel(value: f32) -> u8 {
        (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    }
}
