#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderStyle {
    Flat,
    Gradient,
    ShadowMap,
}

impl RenderStyle {
    pub fn from_config(value: i32) -> Self {
        match value {
            1 => Self::Gradient,
            2 => Self::ShadowMap,
            _ => Self::Flat,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::Flat => 0,
            Self::Gradient => 1,
            Self::ShadowMap => 2,
        }
    }
}
