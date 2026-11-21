// what the colour rules need to know of a block type, from its full name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Water,
    // the client has not received the block yet
    Placeholder,
    Other,
}

impl BlockKind {
    pub const WATER: &'static str = "minecraft:water";
    pub const PLACEHOLDER: &'static str = "minecraft:client_request_placeholder_block";

    pub fn of(name: &str) -> Self {
        match name {
            Self::WATER => Self::Water,
            Self::PLACEHOLDER => Self::Placeholder,
            _ => Self::Other,
        }
    }
}
