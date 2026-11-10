// what is under the cursor on the world map
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoverInfo<'a> {
    pub block: (i32, i32),
    pub chunk: (i32, i32),
    // the player-facing biome name; None when it is not known
    pub biome: Option<&'a str>,
}
