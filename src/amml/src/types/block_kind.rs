// what the column walk needs to know of a block from its type name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    Water,
    // the client has not received the block yet
    Placeholder,
    Other,
}

impl BlockKind {
    pub const AIR: &'static [u8] = b"minecraft:air";
    pub const SHORT_AIR: &'static [u8] = b"air";
    pub const WATER: &'static [u8] = b"minecraft:water";
    pub const PLACEHOLDER: &'static [u8] = b"minecraft:client_request_placeholder_block";

    // a block without a readable name walks as air
    pub fn of(name: Option<&[u8]>) -> Self {
        match name {
            None => Self::Air,
            Some(name) if name == Self::AIR || name == Self::SHORT_AIR || name.is_empty() => Self::Air,
            Some(name) if name == Self::WATER => Self::Water,
            Some(name) if name == Self::PLACEHOLDER => Self::Placeholder,
            Some(_) => Self::Other,
        }
    }
}
