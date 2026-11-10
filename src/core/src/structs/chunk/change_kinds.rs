#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChangeKinds(u8);

impl ChangeKinds {
    pub const NONE: Self = Self(0);
    pub const COLOUR: Self = Self(1);
    pub const HEIGHT: Self = Self(2);
    pub const SOLID: Self = Self(4);
    pub const BIOME: Self = Self(8);
    // the kinds that change baked pixels
    pub const PIXELS: Self = Self(7);
    pub const ALL: Self = Self(15);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}
