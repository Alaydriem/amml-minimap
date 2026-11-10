#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba { r: 0, g: 0, b: 0, a: 0 };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_packed(value: u32) -> Self {
        let [r, g, b, a] = value.to_le_bytes();
        Self { r, g, b, a }
    }

    pub fn packed(&self) -> u32 {
        u32::from_le_bytes([self.r, self.g, self.b, self.a])
    }
}
