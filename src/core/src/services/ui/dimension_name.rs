// the player-facing name of a dimension id
pub struct DimensionName;

impl DimensionName {
    pub const OVERWORLD: i32 = 0;

    pub fn of(dim: i32) -> &'static str {
        match dim {
            0 => "Overworld",
            1 => "Nether",
            2 => "The End",
            _ => "Unknown",
        }
    }
}
